use super::{
    LiveDocumentSmartsBridgeV1, LiveFailureV1, PyLiveDocumentSmartsCategoryV1,
    PyLiveDocumentSmartsReasonV1, PyLiveDocumentSmartsRecoveryV1,
    PyLiveDocumentSmartsSelectedQueryV1,
};
use crate::{RenderInteractionModifierV1, RenderInteractionQueryV1, RenderInteractionSessionV1};
use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, SmartsMatchOptions,
    SmartsMatchResult, SmilesMolecule,
};
use ferrum_document::{DocumentFenceV1, DocumentSession};
use pyo3::types::PyAnyMethods;

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/>",
    "</atom></molecule></cdml>"
);
const MUTATED_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"3\" y=\"2\"/>",
    "</atom></molecule></cdml>"
);

struct ReceiptLifecycleEngine;

impl ChemEngine for ReceiptLifecycleEngine {
    fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
        unavailable("smiles_to_molecule")
    }

    fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
        unavailable("generate_2d_coordinates")
    }

    fn smarts_match(
        &self,
        _: &str,
        target: &MolGraph,
        options: SmartsMatchOptions,
    ) -> Result<SmartsMatchResult, ChemistryError> {
        SmartsMatchResult::try_from_rows(target, options, vec![vec![0]], false).map_err(|_| {
            ChemistryError::SmartsMatchUnavailable {
                reason: ferrum_chemistry::SmartsMatchUnavailableReason::MalformedNativeResponse,
            }
        })
    }

    fn molecule_to_smarts(&self, _: &MolGraph) -> Result<String, ChemistryError> {
        Ok("C".to_owned())
    }

    fn kekulize(&self, _: &MolGraph, _: KekulizeOptions) -> Result<MolGraph, ChemistryError> {
        unavailable("kekulize")
    }
}

fn unavailable<T>(operation: &'static str) -> Result<T, ChemistryError> {
    Err(ChemistryError::OperationUnavailable { operation })
}

fn session() -> RenderInteractionSessionV1 {
    RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("fixture CDML loads"))
}

fn fence(session: &RenderInteractionSessionV1) -> DocumentFenceV1 {
    let snapshot = session.snapshot().expect("fixture session snapshots");
    DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
}

fn reason(py: pyo3::Python<'_>, error: pyo3::PyErr) -> PyLiveDocumentSmartsReasonV1 {
    *error
        .value(py)
        .getattr("reason")
        .expect("closed failure reason")
        .extract::<pyo3::PyRef<'_, PyLiveDocumentSmartsReasonV1>>()
        .expect("closed failure reason enum")
}

#[test]
fn live_failure_facts_are_closed_and_redacted() {
    let (category, reason, _) =
        LiveFailureV1::InvalidQuery(PyLiveDocumentSmartsReasonV1::InvalidQuery).facts();
    assert_eq!(category, PyLiveDocumentSmartsCategoryV1::InvalidQuery);
    assert_eq!(reason, PyLiveDocumentSmartsReasonV1::InvalidQuery);
    let rendered = format!("{:?} {:?}", category, reason);
    for forbidden in ["FCQ1", "FQM1", "CDML", "molecule_id", "libferrum_chem"] {
        assert!(!rendered.contains(forbidden));
    }
}

#[test]
fn live_failure_exposes_only_closed_python_recovery_facts() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let error = LiveFailureV1::Stale(PyLiveDocumentSmartsReasonV1::StaleSelection).into_pyerr();
        let value = error.value(py);
        let category = value
            .getattr("category")
            .expect("closed category is attached")
            .extract::<pyo3::PyRef<'_, PyLiveDocumentSmartsCategoryV1>>()
            .expect("category remains in the closed vocabulary");
        let reason = value
            .getattr("reason")
            .expect("closed reason is attached")
            .extract::<pyo3::PyRef<'_, PyLiveDocumentSmartsReasonV1>>()
            .expect("reason remains in the closed vocabulary");
        let recovery = value
            .getattr("recovery")
            .expect("closed recovery is attached")
            .extract::<pyo3::PyRef<'_, super::PyLiveDocumentSmartsRecoveryV1>>()
            .expect("recovery remains in the closed vocabulary");
        assert_eq!(*category, PyLiveDocumentSmartsCategoryV1::Stale);
        assert_eq!(*reason, PyLiveDocumentSmartsReasonV1::StaleSelection);
        assert_eq!(
            *recovery,
            super::PyLiveDocumentSmartsRecoveryV1::RefreshAndRerun
        );
        let rendered = format!("{error}");
        for forbidden in ["FCQ1", "FQM1", "CDML", "molecule_id", "libferrum_chem"] {
            assert!(!rendered.contains(forbidden));
        }
    });
}

#[test]
fn selected_query_token_has_no_python_data_surface() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let module = pyo3::types::PyModule::new(py, "ferrum_chem").expect("module");
        super::initialize(&module).expect("private classes register");
        let class = module
            .getattr("_LiveDocumentSmartsSelectedQueryV1")
            .expect("selected capability class");
        assert!(class.call0().is_err());
        for forbidden in [
            "issuer",
            "selection",
            "roots",
            "identifier",
            "graph",
            "query",
        ] {
            assert!(class.getattr(forbidden).is_err());
        }
        let _ = std::any::TypeId::of::<PyLiveDocumentSmartsSelectedQueryV1>();
    });
}

#[test]
fn selected_query_capture_refuses_an_empty_root_before_issuance() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridgeV1::new();
        bridge
            .publish(&session, expected.revision())
            .expect("published renderer plan");
        let observation = session
            .observe_render_interaction_v1(expected)
            .expect("renderer observation");
        let selection = session
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Point {
                    x: 1000.0,
                    y: 1000.0,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("blank canvas produces an empty renderer selection");
        let selection =
            super::super::direct_root_interaction_binding::test_selection_from_value_v1(selection);

        let error = bridge
            .capture_selected_query(py, &session, &selection)
            .expect_err("empty roots refuse before a selected-query token can be issued");
        assert_eq!(
            reason(py, error),
            PyLiveDocumentSmartsReasonV1::SelectedRootEmpty
        );
    });
}

fn selected_query_token(
    session: &RenderInteractionSessionV1,
    bridge: &LiveDocumentSmartsBridgeV1,
    expected: DocumentFenceV1,
) -> PyLiveDocumentSmartsSelectedQueryV1 {
    let observation = session
        .observe_render_interaction_v1(expected)
        .expect("renderer observation");
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                identifier: "m".to_owned(),
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("one selected molecule");
    PyLiveDocumentSmartsSelectedQueryV1 {
        issuer: bridge.issuer.clone(),
        selection,
    }
}

#[test]
fn selected_readiness_is_available_without_consuming_the_token() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridgeV1::new();
        bridge
            .publish(&session, expected.revision())
            .expect("published renderer plan");
        let selected = selected_query_token(&session, &bridge, expected);

        let readiness = bridge.selected_readiness(&session, &selected);
        assert!(readiness.available);
        assert!(readiness.reason.is_none());

        bridge
            .run_selected(py, &session, &ReceiptLifecycleEngine, &selected, 1, 1)
            .expect("readiness does not consume a valid selected token");
    });
}

#[test]
fn selected_readiness_reports_foreign_and_invalid_token_admission_failures() {
    let session = session();
    let expected = fence(&session);
    let mut owner = LiveDocumentSmartsBridgeV1::new();
    owner
        .publish(&session, expected.revision())
        .expect("published renderer plan");
    let selected = selected_query_token(&session, &owner, expected);

    let foreign = LiveDocumentSmartsBridgeV1::new().selected_readiness(&session, &selected);
    assert_eq!(
        foreign.reason,
        Some(PyLiveDocumentSmartsReasonV1::ForeignSelection)
    );

    let observation = session
        .observe_render_interaction_v1(expected)
        .expect("renderer observation");
    let empty = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Point {
                x: 1000.0,
                y: 1000.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("blank canvas produces an empty renderer selection");
    let invalid = PyLiveDocumentSmartsSelectedQueryV1 {
        issuer: owner.issuer.clone(),
        selection: empty,
    };
    let readiness = owner.selected_readiness(&session, &invalid);
    assert_eq!(
        (readiness.category, readiness.reason, readiness.recovery),
        (
            Some(PyLiveDocumentSmartsCategoryV1::Refused),
            Some(PyLiveDocumentSmartsReasonV1::SelectedRootEmpty),
            Some(PyLiveDocumentSmartsRecoveryV1::SelectOneMolecule),
        )
    );
}

#[test]
fn selected_readiness_reports_a_stale_token() {
    let mut session = session();
    let expected = fence(&session);
    let mut bridge = LiveDocumentSmartsBridgeV1::new();
    bridge
        .publish(&session, expected.revision())
        .expect("published renderer plan");
    let selected = selected_query_token(&session, &bridge, expected);
    session
        .commit_complete_cdml_transaction_v1(expected, MUTATED_SOURCE)
        .expect("changed document commits");

    let readiness = bridge.selected_readiness(&session, &selected);
    assert_eq!(
        (readiness.category, readiness.reason),
        (
            Some(PyLiveDocumentSmartsCategoryV1::Stale),
            Some(PyLiveDocumentSmartsReasonV1::StaleSelection),
        )
    );
}

#[test]
fn receipt_only_retirement_retains_the_plan_for_raw_and_selected_reruns() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridgeV1::new();
        bridge
            .publish(&session, expected.revision())
            .expect("published renderer plan");
        let engine = ReceiptLifecycleEngine;

        let raw = bridge
            .run(py, &session, &engine, "C".to_owned(), 1, 1)
            .expect("raw SMARTS run");
        let old_receipt = raw.bind(py).borrow().receipt.clone_ref(py);

        let observation = session
            .observe_render_interaction_v1(expected)
            .expect("renderer observation");
        let selection = session
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Root {
                    identifier: "m".to_owned(),
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("one selected molecule");
        let selected = PyLiveDocumentSmartsSelectedQueryV1 {
            issuer: bridge.issuer.clone(),
            selection,
        };

        bridge.retire_receipts();
        let retired = bridge
            .show(py, &session, old_receipt.bind(py).borrow(), 0)
            .expect_err("receipt-only retirement revokes the old opaque receipt");
        assert_eq!(
            reason(py, retired),
            PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
        );

        bridge
            .run(py, &session, &engine, "C".to_owned(), 1, 1)
            .expect("retained plan admits a raw rerun");
        bridge
            .run_selected(py, &session, &engine, &selected, 1, 1)
            .expect("retained plan admits a fresh opaque selected rerun");

        let live = bridge
            .run(py, &session, &engine, "C".to_owned(), 1, 1)
            .expect("fresh receipt exists before full retirement");
        let live_receipt = live.bind(py).borrow().receipt.clone_ref(py);
        bridge.retire();
        let retired_receipt = bridge
            .show(py, &session, live_receipt.bind(py).borrow(), 0)
            .expect_err("full retirement revokes the opaque receipt before plan lookup");
        assert_eq!(
            reason(py, retired_receipt),
            PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
        );
        let retired_plan = bridge
            .validate_raw_request(&session, "C", 1, 1)
            .expect_err("full retirement revokes the published plan");
        assert_eq!(
            reason(py, retired_plan),
            PyLiveDocumentSmartsReasonV1::PlanNotPublished
        );
    });
}

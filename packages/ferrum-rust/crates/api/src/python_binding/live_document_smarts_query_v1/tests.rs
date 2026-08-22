use super::{
    LiveDocumentSmartsBridgeV1, LiveFailureV1, LiveSmartsReadinessV1,
    PyLiveDocumentSmartsCategoryV1, PyLiveDocumentSmartsReasonV1,
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
const PARTIAL_RENDER_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/>",
    "<ftext><b>rich label</b></ftext></atom></molecule></cdml>"
);
const MUTATED_PARTIAL_RENDER_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/>",
    "<ftext><b>revised rich label</b></ftext></atom></molecule></cdml>"
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

fn partial_render_session() -> RenderInteractionSessionV1 {
    RenderInteractionSessionV1::new(
        DocumentSession::load(PARTIAL_RENDER_SOURCE).expect("fixture CDML loads"),
    )
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

#[test]
fn post_reservation_paint_failure_consumes_the_receipt_row() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridgeV1::new();
        bridge
            .publish(&session, expected.revision())
            .expect("published renderer plan");
        let summary = bridge
            .run(py, &session, &ReceiptLifecycleEngine, "C".to_owned(), 1, 1)
            .expect("issued receipt");
        let receipt = summary.bind(py).borrow().receipt.clone_ref(py);

        let LiveSmartsReadinessV1::Ready(plan) = &mut bridge.readiness else {
            panic!("fixture publishes a ready plan");
        };
        plan.atom_points_by_graph_position.clear();

        let paint_failure = bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect_err("corrupt plan reaches paint after reservation");
        assert_eq!(
            reason(py, paint_failure),
            PyLiveDocumentSmartsReasonV1::PaintUnavailable
        );
        let replay = bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect_err("post-reservation failure consumes the receipt row");
        assert_eq!(
            reason(py, replay),
            PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
        );
    });
}

#[test]
fn partial_render_publication_is_accepted_but_live_smarts_is_unsupported() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let mut session = partial_render_session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridgeV1::new();

        let observation = bridge
            .publish(&session, expected.revision())
            .expect("accepted partial render observation");
        assert!(!observation.issues().is_empty());
        assert!(matches!(
            bridge.readiness,
            LiveSmartsReadinessV1::UnsupportedDocument { .. }
        ));
        assert!(bridge.receipts.is_empty());

        let error = bridge
            .validate_raw_request(&session, "C", 1, 1)
            .expect_err("unrenderable molecule cannot receive a live SMARTS plan");
        assert_eq!(
            reason(py, error),
            PyLiveDocumentSmartsReasonV1::UnsupportedDocument
        );

        session
            .commit_complete_cdml_transaction_v1(expected, MUTATED_PARTIAL_RENDER_SOURCE)
            .expect("changed partial-render document commits");
        let error = bridge
            .validate_raw_request(&session, "C", 1, 1)
            .expect_err("changed document invalidates the old partial-render publication");
        assert_eq!(
            reason(py, error),
            PyLiveDocumentSmartsReasonV1::StaleDocument
        );
    });
}

use super::{
    LiveDocumentSmartsBridge, LiveFailure, LiveSmartsReadiness, LiveSmartsReceiptRow,
    PyLiveDocumentSmartsCategoryV1, PyLiveDocumentSmartsPaint, PyLiveDocumentSmartsReasonV1,
    PyLiveDocumentSmartsReceipt, PyLiveDocumentSmartsRecoveryV1, PyLiveDocumentSmartsRunSummary,
    PyLiveDocumentSmartsSelectedQuery,
};
use crate::protocol::document_smarts_snapshot::OwnedDocumentSmartsSnapshot;
use crate::{RenderInteractionModifierV1, RenderInteractionQueryV1, RenderInteractionSessionV1};
use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, SmartsMatchOptions,
    SmartsMatchResult, SmilesMolecule,
};
use ferrum_document::{
    DocumentFenceV1, DocumentSession, Point3V1, SessionOperation,
    SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
};
use pyo3::types::PyAnyMethods;

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/>",
    "</atom></molecule></cdml>"
);

#[test]
fn live_publication_and_stateless_snapshot_share_observation_derived_target_facts() {
    let session = session();
    let stateless = OwnedDocumentSmartsSnapshot::from_accepted_observation(
        session.observe(0).expect("accepted stateless observation"),
    )
    .expect("stateless snapshot lowers");
    let mut bridge = LiveDocumentSmartsBridge::new();
    bridge
        .publish(&session, 0)
        .expect("live publication accepts matching observation");
    let LiveSmartsReadiness::Ready(live) = &bridge.readiness else {
        panic!("live plan is published");
    };

    assert_eq!(stateless.revision(), live.snapshot.revision());
    assert_eq!(stateless.digest(), live.snapshot.digest());
    assert_eq!(stateless.targets().len(), live.snapshot.targets().len());
    for (left, right) in stateless.targets().iter().zip(live.snapshot.targets()) {
        assert_eq!(left.document_paint_order(), right.document_paint_order());
        assert_eq!(left.graph(), right.graph());
        assert_eq!(
            left.graph_position_to_document_object_ids(),
            right.graph_position_to_document_object_ids()
        );
    }
}
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

fn issue_receipt(
    py: pyo3::Python<'_>,
    bridge: &mut LiveDocumentSmartsBridge,
    session: &RenderInteractionSessionV1,
) -> pyo3::Py<PyLiveDocumentSmartsReceipt> {
    let summary: pyo3::Py<PyLiveDocumentSmartsRunSummary> = bridge
        .run(py, session, &ReceiptLifecycleEngine, "C".to_owned(), 1, 1)
        .expect("one native match issues one receipt row");
    summary.bind(py).borrow().receipt.clone_ref(py)
}

fn receipt_row_is_reserved(
    py: pyo3::Python<'_>,
    bridge: &LiveDocumentSmartsBridge,
    receipt: &pyo3::Py<PyLiveDocumentSmartsReceipt>,
    row_index: usize,
) -> bool {
    let key = receipt.bind(py).borrow().key.clone();
    matches!(
        bridge
            .receipts
            .get(&key)
            .and_then(|state| state.rows.get(row_index)),
        Some(LiveSmartsReceiptRow::Reserved)
    )
}

fn move_fixture_atom(session: &mut RenderInteractionSessionV1) {
    let expected = fence(session);
    let mut prepared = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            expected.revision(),
            SessionOperation::V1(SessionOperationV1::SetAtomPosition {
                atom_id: "a".to_owned(),
                position: Point3V1::new(3.0, 2.0, 0.0).expect("finite replacement point"),
            }),
            TransitionAuthorizationV1::None,
        ))
        .expect("changed document prepares");
    session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("changed document commits");
}

struct DuplicatePositionEngine;

impl ChemEngine for DuplicatePositionEngine {
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
        SmartsMatchResult::try_from_rows(target, options, vec![vec![0, 0]], false).map_err(|_| {
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

#[test]
fn live_failure_facts_are_closed_and_redacted() {
    let (category, reason, _) =
        LiveFailure::InvalidQuery(PyLiveDocumentSmartsReasonV1::InvalidQuery).facts();
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
        let error = LiveFailure::Stale(PyLiveDocumentSmartsReasonV1::StaleSelection).into_pyerr();
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
fn private_live_smarts_helpers_are_absent_from_the_module_while_the_bridge_issues_safe_facts() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let module = pyo3::types::PyModule::new(py, "ferrum_chem").expect("module");
        super::initialize(&module).expect("closed error facts register");
        let module_names = module
            .call_method0("__dir__")
            .expect("module names are inspectable")
            .extract::<Vec<String>>()
            .expect("module names are strings");
        for private_name in [
            "_LiveDocumentSmartsReceipt",
            "_LiveDocumentSmartsSelectedQuery",
            "_LiveDocumentSmartsSelectedReadiness",
            "_LiveDocumentSmartsMoleculeSummary",
            "_LiveDocumentSmartsRunSummary",
            "_LiveDocumentSmartsPaint",
        ] {
            assert!(module.getattr(private_name).is_err());
            assert!(!module_names.iter().any(|name| name == private_name));
        }
        assert!(module.getattr("LiveDocumentSmartsError").is_ok());
        assert!(module.getattr("LiveDocumentSmartsCategoryV1").is_ok());
        assert!(module.getattr("LiveDocumentSmartsReasonV1").is_ok());
        assert!(module.getattr("LiveDocumentSmartsRecoveryV1").is_ok());

        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new();
        bridge
            .publish(&session, expected.revision())
            .expect("published renderer plan");
        let summary = bridge
            .run(py, &session, &ReceiptLifecycleEngine, "C".to_owned(), 1, 1)
            .expect("the private seam issues copied query facts");
        let summary = summary.bind(py).borrow();
        assert_eq!(summary.traversal, "complete");
        assert_eq!(summary.molecules[0].bind(py).borrow().match_count, 1);
        let receipt = summary.receipt.clone_ref(py);
        drop(summary);
        let paint = bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect("the private receipt redeems only to finite copied bounds");
        assert!(
            paint
                .bind(py)
                .borrow()
                .atom_bounds
                .iter()
                .all(|(left, top, right, bottom)| {
                    left.is_finite() && top.is_finite() && right.is_finite() && bottom.is_finite()
                })
        );
    });
}

#[test]
fn selected_query_capture_refuses_an_empty_root_before_issuance() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new();
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
fn entropy_failure_refuses_raw_receipts_and_selected_capabilities_before_issuance() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new_without_entropy();
        bridge
            .publish(&session, expected.revision())
            .expect("a display plan does not require capability entropy");

        let raw = bridge
            .run(py, &session, &ReceiptLifecycleEngine, "C".to_owned(), 1, 1)
            .expect_err("missing entropy refuses before raw receipt issuance");
        assert_eq!(
            reason(py, raw),
            PyLiveDocumentSmartsReasonV1::MatchUnavailable
        );
        assert!(
            bridge.receipts.is_empty(),
            "refusal never publishes receipt state"
        );

        let observation = session
            .observe_render_interaction_v1(expected)
            .expect("renderer observation");
        let selection = session
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Root {
                    document_object_id: observation.roots()[0].document_object_id().clone(),
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("selected molecule root");
        let selection =
            super::super::direct_root_interaction_binding::test_selection_from_value_v1(selection);
        let captured = bridge
            .capture_selected_query(py, &session, &selection)
            .expect_err("missing entropy refuses selected capability issuance");
        assert_eq!(
            reason(py, captured),
            PyLiveDocumentSmartsReasonV1::MatchUnavailable
        );
        assert!(
            bridge.receipts.is_empty(),
            "selected refusal issues no receipt state"
        );
    });
}

fn selected_query_token(
    session: &RenderInteractionSessionV1,
    bridge: &LiveDocumentSmartsBridge,
    expected: DocumentFenceV1,
) -> PyLiveDocumentSmartsSelectedQuery {
    let observation = session
        .observe_render_interaction_v1(expected)
        .expect("renderer observation");
    let document_object_id = observation
        .roots()
        .first()
        .expect("fixture observation contains one molecule root")
        .document_object_id()
        .clone();
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                document_object_id,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("one selected molecule");
    PyLiveDocumentSmartsSelectedQuery {
        issuer: bridge
            .issuer
            .clone()
            .expect("normal test entropy issues an issuer"),
        selection,
    }
}

#[test]
fn selected_readiness_is_available_without_consuming_the_token() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new();
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
    let mut owner = LiveDocumentSmartsBridge::new();
    owner
        .publish(&session, expected.revision())
        .expect("published renderer plan");
    let selected = selected_query_token(&session, &owner, expected);

    let foreign = LiveDocumentSmartsBridge::new().selected_readiness(&session, &selected);
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
    let invalid = PyLiveDocumentSmartsSelectedQuery {
        issuer: owner
            .issuer
            .clone()
            .expect("normal test entropy issues an issuer"),
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
    let mut bridge = LiveDocumentSmartsBridge::new();
    bridge
        .publish(&session, expected.revision())
        .expect("published renderer plan");
    let selected = selected_query_token(&session, &bridge, expected);
    let mut prepared = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            expected.revision(),
            SessionOperation::V1(SessionOperationV1::SetAtomPosition {
                atom_id: "a".to_owned(),
                position: Point3V1::new(3.0, 2.0, 0.0).expect("finite replacement point"),
            }),
            TransitionAuthorizationV1::None,
        ))
        .expect("changed document prepares");
    session
        .commit_session_operation_transition_v1(&mut prepared)
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
fn receipt_only_clearing_retains_the_plan_for_raw_and_selected_reruns() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new();
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
        let document_object_id = observation
            .roots()
            .first()
            .expect("fixture observation contains one molecule root")
            .document_object_id()
            .clone();
        let selection = session
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Root {
                    document_object_id,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("one selected molecule");
        let selected = PyLiveDocumentSmartsSelectedQuery {
            issuer: bridge
                .issuer
                .clone()
                .expect("normal test entropy issues an issuer"),
            selection,
        };

        bridge.clear_receipts();
        let cleared = bridge
            .show(py, &session, old_receipt.bind(py).borrow(), 0)
            .expect_err("receipt-only clearing revokes the old opaque receipt");
        assert_eq!(
            reason(py, cleared),
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
            .expect("fresh receipt exists before full clearing");
        let live_receipt = live.bind(py).borrow().receipt.clone_ref(py);
        bridge.clear_published_plan();
        let cleared_receipt = bridge
            .show(py, &session, live_receipt.bind(py).borrow(), 0)
            .expect_err("full clearing revokes the opaque receipt before plan lookup");
        assert_eq!(
            reason(py, cleared_receipt),
            PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
        );
        let cleared_plan = bridge
            .validate_raw_request(&session, "C", 1, 1)
            .expect_err("full clearing revokes the published plan");
        assert_eq!(
            reason(py, cleared_plan),
            PyLiveDocumentSmartsReasonV1::PlanNotPublished
        );
    });
}

#[test]
fn show_issues_finite_identity_free_bounds_once_and_consumes_the_row() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new();
        bridge
            .publish(&session, expected.revision())
            .expect("published renderer plan");
        let receipt = issue_receipt(py, &mut bridge, &session);

        let paint: pyo3::Py<PyLiveDocumentSmartsPaint> = bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect("a current issued row reveals copied paint bounds");
        let bounds = &paint.bind(py).borrow().atom_bounds;
        assert_eq!(bounds.len(), 1);
        assert!(bounds.iter().all(|(left, top, right, bottom)| {
            left.is_finite() && top.is_finite() && right.is_finite() && bottom.is_finite()
        }));
        assert!(receipt_row_is_reserved(py, &bridge, &receipt, 0));

        let replay = bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect_err("a successful show consumes its row and cannot issue geometry twice");
        assert_eq!(
            reason(py, replay),
            PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
        );
    });
}

#[test]
fn foreign_receipt_refuses_before_the_receiving_ledger_or_geometry_is_touched() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut issuing_bridge = LiveDocumentSmartsBridge::new();
        issuing_bridge
            .publish(&session, expected.revision())
            .expect("issuing bridge publishes");
        let receipt = issue_receipt(py, &mut issuing_bridge, &session);
        let mut receiving_bridge = LiveDocumentSmartsBridge::new();
        receiving_bridge
            .publish(&session, expected.revision())
            .expect("same CDML may be published by a distinct bridge");

        let foreign = receiving_bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect_err("identical CDML never lets a different bridge redeem a receipt");
        assert_eq!(
            reason(py, foreign),
            PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
        );
        assert!(
            !receipt_row_is_reserved(py, &issuing_bridge, &receipt, 0),
            "foreign refusal happens before the issuer's row reservation"
        );

        issuing_bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect("foreign refusal did not consume the issuer-owned row");
    });
}

#[test]
fn out_of_range_row_refuses_without_consuming_an_available_row() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new();
        bridge
            .publish(&session, expected.revision())
            .expect("published renderer plan");
        let receipt = issue_receipt(py, &mut bridge, &session);

        let out_of_range = bridge
            .show(py, &session, receipt.bind(py).borrow(), 1)
            .expect_err("only the issued display-row ordinal is redeemable");
        assert_eq!(
            reason(py, out_of_range),
            PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
        );
        assert!(!receipt_row_is_reserved(py, &bridge, &receipt, 0));
        bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect("an out-of-range request did not reserve a valid row");
    });
}

#[test]
fn stale_document_after_issuance_consumes_the_reserved_row() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let mut session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new();
        bridge
            .publish(&session, expected.revision())
            .expect("published renderer plan");
        let receipt = issue_receipt(py, &mut bridge, &session);
        move_fixture_atom(&mut session);

        let stale = bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect_err("revision and digest changes fence an already issued receipt");
        assert_eq!(
            reason(py, stale),
            PyLiveDocumentSmartsReasonV1::StaleDocument
        );
        assert!(receipt_row_is_reserved(py, &bridge, &receipt, 0));
        let replay = bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect_err("a stale refusal after reservation is terminal");
        assert_eq!(
            reason(py, replay),
            PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
        );
    });
}

#[test]
fn replacement_publication_revokes_old_receipts_and_advances_the_plan_generation() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new();
        bridge
            .publish(&session, expected.revision())
            .expect("initial plan publishes");
        let receipt = issue_receipt(py, &mut bridge, &session);
        let first_generation = match &bridge.readiness {
            LiveSmartsReadiness::Ready(plan) => plan.generation,
            _ => panic!("initial plan remains ready"),
        };

        bridge
            .publish(&session, expected.revision())
            .expect("reprojection replaces the plan through the publication transaction");
        let replacement_generation = match &bridge.readiness {
            LiveSmartsReadiness::Ready(plan) => plan.generation,
            _ => panic!("replacement plan remains ready"),
        };
        assert!(replacement_generation > first_generation);
        let stale_generation = bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect_err("republication clears receipts before a stale generation can paint");
        assert_eq!(
            reason(py, stale_generation),
            PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
        );
    });
}

#[test]
fn post_reservation_paint_failure_is_terminal_and_never_restores_the_row() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new();
        bridge
            .publish(&session, expected.revision())
            .expect("published renderer plan");
        let receipt = issue_receipt(py, &mut bridge, &session);
        let LiveSmartsReadiness::Ready(plan) = &mut bridge.readiness else {
            panic!("published plan is ready");
        };
        plan.atom_points_by_graph_position[0][0] = (f64::NAN, 2.0);

        let failed = bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect_err("invalid paint geometry refuses only after reserving the row");
        assert_eq!(
            reason(py, failed),
            PyLiveDocumentSmartsReasonV1::PaintUnavailable
        );
        assert!(receipt_row_is_reserved(py, &bridge, &receipt, 0));
        let replay = bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect_err("failed paint issuance cannot be replayed for another geometry attempt");
        assert_eq!(
            reason(py, replay),
            PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
        );
    });
}

#[test]
fn clear_paths_revoke_receipts_before_any_row_can_be_reserved() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new();
        bridge
            .publish(&session, expected.revision())
            .expect("published renderer plan");
        let receipt = issue_receipt(py, &mut bridge, &session);
        bridge.clear_receipts();
        let cleared = bridge
            .show(py, &session, receipt.bind(py).borrow(), 0)
            .expect_err("receipt clearing revokes before lookup or geometry");
        assert_eq!(
            reason(py, cleared),
            PyLiveDocumentSmartsReasonV1::ReceiptUnavailable
        );
    });
}

#[test]
fn duplicate_native_positions_refuse_before_receipt_publication() {
    pyo3::Python::initialize();
    pyo3::Python::attach(|py| {
        let session = session();
        let expected = fence(&session);
        let mut bridge = LiveDocumentSmartsBridge::new();
        bridge
            .publish(&session, expected.revision())
            .expect("published renderer plan");

        let failure = bridge
            .run(py, &session, &DuplicatePositionEngine, "C".to_owned(), 1, 1)
            .expect_err("a native row may not map one atom position more than once");
        assert_eq!(
            reason(py, failure),
            PyLiveDocumentSmartsReasonV1::MatchUnavailable
        );
        assert!(
            bridge.receipts.is_empty(),
            "malformed native rows publish no redeemable receipt state"
        );
    });
}

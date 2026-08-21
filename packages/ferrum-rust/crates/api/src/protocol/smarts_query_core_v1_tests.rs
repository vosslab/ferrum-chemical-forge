use std::{cell::RefCell, collections::VecDeque};

use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, SmartsMatchOptions,
    SmartsMatchResult, SmilesMolecule,
};
use ferrum_document::DocumentSession;

use super::super::{
    DocumentSmartsQueryDocumentV1, DocumentSmartsQueryInputV1, DocumentSmartsQueryLimitsV1,
    DocumentSmartsQueryRequestV1, DocumentSmartsQueryTraversalSummaryV1,
    OperationProtocolOutcomeV1,
    runtime::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1},
};
use super::execute_document_smarts_query_v1;

const SOURCE: &str = concat!(
    "<cdml><molecule id=\"first\"><atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/></atom></molecule>",
    "<molecule id=\"second\"><atom id=\"b\" name=\"O\"><point x=\"3\" y=\"4\"/></atom></molecule>",
    "<molecule id=\"third\"><atom id=\"c\" name=\"N\"><point x=\"5\" y=\"6\"/></atom></molecule></cdml>"
);

struct FixtureEngine {
    matches: RefCell<VecDeque<FixtureMatch>>,
    calls: RefCell<Vec<(String, u32)>>,
}

struct FixtureMatch {
    rows: Vec<Vec<usize>>,
    truncated: bool,
}

impl FixtureEngine {
    fn new(matches: impl IntoIterator<Item = FixtureMatch>) -> Self {
        Self {
            matches: RefCell::new(matches.into_iter().collect()),
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl ChemEngine for FixtureEngine {
    fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
        unavailable("smiles")
    }
    fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
        unavailable("coordinates")
    }
    fn smarts_match(
        &self,
        query: &str,
        target: &MolGraph,
        options: SmartsMatchOptions,
    ) -> Result<SmartsMatchResult, ChemistryError> {
        self.calls
            .borrow_mut()
            .push((query.to_owned(), options.max_matches()));
        let fixture =
            self.matches
                .borrow_mut()
                .pop_front()
                .ok_or(ChemistryError::OperationUnavailable {
                    operation: "fixture",
                })?;
        SmartsMatchResult::try_from_rows(target, options, fixture.rows, fixture.truncated).map_err(
            |_| ChemistryError::SmartsMatchUnavailable {
                reason: ferrum_chemistry::SmartsMatchUnavailableReason::MalformedNativeResponse,
            },
        )
    }
    fn molecule_to_smarts(&self, _: &MolGraph) -> Result<String, ChemistryError> {
        Ok("fixture-selected-smarts".to_owned())
    }
    fn kekulize(&self, _: &MolGraph, _: KekulizeOptions) -> Result<MolGraph, ChemistryError> {
        unavailable("kekulize")
    }
}

struct FixtureRuntime(FixtureEngine);
impl ChemistryRuntimeV1 for FixtureRuntime {
    fn with_engine<T>(
        &self,
        operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
    ) -> Result<T, ChemistryRuntimeErrorV1> {
        operation(&self.0)
    }
}
fn unavailable<T>(operation: &'static str) -> Result<T, ChemistryError> {
    Err(ChemistryError::OperationUnavailable { operation })
}
fn observation() -> ferrum_document::SessionDocumentObservationV1 {
    DocumentSession::load(SOURCE).unwrap().observe(0).unwrap()
}
fn result(rows: Vec<Vec<usize>>, truncated: bool) -> FixtureMatch {
    FixtureMatch { rows, truncated }
}
fn request(
    observation: &ferrum_document::SessionDocumentObservationV1,
    query: DocumentSmartsQueryInputV1,
    per: u32,
    total: u32,
) -> DocumentSmartsQueryRequestV1 {
    DocumentSmartsQueryRequestV1 {
        document: DocumentSmartsQueryDocumentV1 {
            cdml: SOURCE.to_owned(),
            expected_revision: observation.snapshot().revision(),
            expected_digest_hex: observation
                .snapshot()
                .digest()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
        },
        query,
        limits: DocumentSmartsQueryLimitsV1 {
            max_matches_per_molecule: Some(per),
            max_total_matches: Some(total),
        },
    }
}

#[test]
fn raw_query_has_source_ordered_redacted_summary_and_per_target_truncation() {
    let observation = observation();
    let runtime = FixtureRuntime(FixtureEngine::new([
        result(vec![vec![0]], false),
        result(vec![], false),
        result(vec![vec![0]], true),
    ]));
    let OperationProtocolOutcomeV1::DocumentSmartsQuery { query } =
        execute_document_smarts_query_v1(
            &observation,
            request(
                &observation,
                DocumentSmartsQueryInputV1::Smarts {
                    value: "[#6]".to_owned(),
                },
                2,
                5,
            ),
            &runtime,
        )
        .unwrap()
    else {
        panic!("SMARTS outcome")
    };
    assert!(matches!(
        query.traversal,
        DocumentSmartsQueryTraversalSummaryV1::Complete
    ));
    assert_eq!(
        query
            .molecules
            .iter()
            .map(|item| (
                item.source_order,
                item.match_count,
                item.completeness.as_str()
            ))
            .collect::<Vec<_>>(),
        vec![(0, 1, "complete"), (2, 1, "truncated")]
    );
    assert_eq!(
        runtime
            .0
            .calls
            .borrow()
            .iter()
            .map(|call| call.0.as_str())
            .collect::<Vec<_>>(),
        vec!["[#6]", "[#6]", "[#6]"]
    );
}

#[test]
fn selected_query_is_snapshot_derived_and_total_budget_stops_before_another_target() {
    let observation = observation();
    let selected = observation.projection().molecules()[1]
        .id()
        .unwrap()
        .as_str()
        .to_owned();
    assert_ne!(
        selected, "second",
        "the protocol selector is not an authored ID"
    );
    let runtime = FixtureRuntime(FixtureEngine::new([
        result(vec![vec![0]], false),
        result(vec![vec![0]], false),
    ]));
    let OperationProtocolOutcomeV1::DocumentSmartsQuery { query } =
        execute_document_smarts_query_v1(
            &observation,
            request(
                &observation,
                DocumentSmartsQueryInputV1::SelectedMolecule {
                    molecule_id: selected,
                },
                2,
                2,
            ),
            &runtime,
        )
        .unwrap()
    else {
        panic!("SMARTS outcome")
    };
    assert!(
        matches!(query.traversal, DocumentSmartsQueryTraversalSummaryV1::Incomplete { ref reason } if reason == "total_match_budget_reached")
    );
    assert_eq!(query.molecules.len(), 2);
    assert_eq!(
        runtime
            .0
            .calls
            .borrow()
            .iter()
            .map(|call| call.0.as_str())
            .collect::<Vec<_>>(),
        vec!["fixture-selected-smarts", "fixture-selected-smarts"]
    );
}

#[test]
fn selected_query_refuses_an_authored_source_id_instead_of_crossing_identity_domains() {
    let observation = observation();
    let runtime = FixtureRuntime(FixtureEngine::new([]));
    let refusal = execute_document_smarts_query_v1(
        &observation,
        request(
            &observation,
            DocumentSmartsQueryInputV1::SelectedMolecule {
                molecule_id: "second".to_owned(),
            },
            1,
            1,
        ),
        &runtime,
    );
    assert!(
        refusal.is_err(),
        "authored source IDs are not protocol selectors"
    );
    assert!(
        runtime.0.calls.borrow().is_empty(),
        "an invalid selector must refuse before native work"
    );
}

#[test]
fn stale_fence_and_inconsistent_caps_refuse_before_runtime_work() {
    let observation = observation();
    let runtime = FixtureRuntime(FixtureEngine::new([]));
    let mut stale = request(
        &observation,
        DocumentSmartsQueryInputV1::Smarts {
            value: "C".to_owned(),
        },
        1,
        1,
    );
    stale.document.expected_revision = 1;
    assert!(execute_document_smarts_query_v1(&observation, stale, &runtime).is_err());
    assert!(
        execute_document_smarts_query_v1(
            &observation,
            request(
                &observation,
                DocumentSmartsQueryInputV1::Smarts {
                    value: "C".to_owned()
                },
                3,
                2
            ),
            &runtime
        )
        .is_err()
    );
    assert!(runtime.0.calls.borrow().is_empty());
}

struct InvalidRowsEngine;

impl ChemEngine for InvalidRowsEngine {
    fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
        unavailable("smiles")
    }
    fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
        unavailable("coordinates")
    }
    fn smarts_match(
        &self,
        _: &str,
        target: &MolGraph,
        options: SmartsMatchOptions,
    ) -> Result<SmartsMatchResult, ChemistryError> {
        let rejected = SmartsMatchResult::try_from_rows(
            target,
            options,
            vec![vec![target.atoms().len()]],
            false,
        );
        assert!(
            rejected.is_err(),
            "invalid custom rows must be unrepresentable"
        );
        Err(ChemistryError::SmartsMatchUnavailable {
            reason: ferrum_chemistry::SmartsMatchUnavailableReason::MalformedNativeResponse,
        })
    }
    fn molecule_to_smarts(&self, _: &MolGraph) -> Result<String, ChemistryError> {
        Ok("fixture-selected-smarts".to_owned())
    }
    fn kekulize(&self, _: &MolGraph, _: KekulizeOptions) -> Result<MolGraph, ChemistryError> {
        unavailable("kekulize")
    }
}

struct InvalidRowsRuntime(InvalidRowsEngine);
impl ChemistryRuntimeV1 for InvalidRowsRuntime {
    fn with_engine<T>(
        &self,
        operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
    ) -> Result<T, ChemistryRuntimeErrorV1> {
        operation(&self.0)
    }
}

#[test]
fn invalid_custom_engine_rows_produce_no_protocol_outcome() {
    let observation = observation();
    let result = execute_document_smarts_query_v1(
        &observation,
        request(
            &observation,
            DocumentSmartsQueryInputV1::Smarts {
                value: "C".to_owned(),
            },
            1,
            1,
        ),
        &InvalidRowsRuntime(InvalidRowsEngine),
    );
    assert!(
        result.is_err(),
        "invalid typed rows cannot reach a protocol outcome"
    );
}

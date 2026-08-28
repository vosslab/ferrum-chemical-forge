use std::cell::RefCell;

use ferrum_chemistry::{AtomicNumber, ChemEngine, ChemistryError, InchiMode, MolAtom, MolGraph};
use ferrum_document::DocumentSession;

use super::dto::DocumentMoleculeReportIdentifiersSummaryV1;
use super::execution::execute_operation_with_runtime_v1;
use super::molecule_report_core_v1::DocumentMoleculeReportErrorV1;
use super::molecule_report_identifiers_v1::{
    DocumentMoleculeReportIdentifierUnavailableReasonV1, DocumentMoleculeReportIdentifiersV1,
    evaluate_identifiers_v1,
};
use super::runtime::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1};
use super::{
    OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, OperationProtocolEnvelopeV1,
    OperationProtocolErrorCategoryV1,
};

#[derive(Clone, Copy)]
enum IdentifierStageV1 {
    Smiles,
    Inchi,
    InchiKey,
}

struct IdentifierEngineV1 {
    failure: Option<(IdentifierStageV1, ChemistryError)>,
    calls: RefCell<Vec<&'static str>>,
}

impl IdentifierEngineV1 {
    fn available() -> Self {
        Self {
            failure: None,
            calls: RefCell::new(Vec::new()),
        }
    }

    fn failing(stage: IdentifierStageV1, error: ChemistryError) -> Self {
        Self {
            failure: Some((stage, error)),
            calls: RefCell::new(Vec::new()),
        }
    }

    fn result_for(&self, stage: IdentifierStageV1) -> Result<(), ChemistryError> {
        match &self.failure {
            Some((failure_stage, error))
                if std::mem::discriminant(failure_stage) == std::mem::discriminant(&stage) =>
            {
                Err(error.clone())
            }
            _ => Ok(()),
        }
    }
}

impl ChemEngine for IdentifierEngineV1 {
    fn smiles_to_molecule(
        &self,
        _: &str,
    ) -> Result<ferrum_chemistry::SmilesMolecule, ChemistryError> {
        unreachable!("identifier tests do not parse SMILES")
    }

    fn generate_2d_coordinates(
        &self,
        _: &MolGraph,
    ) -> Result<ferrum_chemistry::Coordinates, ChemistryError> {
        unreachable!("identifier tests do not generate coordinates")
    }

    fn molecule_to_smiles(&self, _: &MolGraph) -> Result<String, ChemistryError> {
        self.calls.borrow_mut().push("smiles");
        self.result_for(IdentifierStageV1::Smiles)?;
        Ok("C".to_owned())
    }

    fn molecule_to_inchi(&self, _: &MolGraph, mode: InchiMode) -> Result<String, ChemistryError> {
        assert_eq!(mode, InchiMode::Standard);
        self.calls.borrow_mut().push("inchi");
        self.result_for(IdentifierStageV1::Inchi)?;
        Ok("InChI=1S/CH4/h1H4".to_owned())
    }

    fn inchi_to_inchi_key(&self, inchi: &str) -> Result<String, ChemistryError> {
        assert_eq!(inchi, "InChI=1S/CH4/h1H4");
        self.calls.borrow_mut().push("inchi_key");
        self.result_for(IdentifierStageV1::InchiKey)?;
        Ok("VNWKTOKETHGBQD-UHFFFAOYSA-N".to_owned())
    }

    fn kekulize(
        &self,
        _: &MolGraph,
        _: ferrum_chemistry::KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        unreachable!("identifier tests do not kekulize")
    }
}

fn graph() -> MolGraph {
    let carbon = MolAtom::new(
        AtomicNumber::try_from(6).expect("carbon is supported"),
        Some(0),
        None,
        None,
        false,
    )
    .expect("valid carbon");
    MolGraph::new(vec![carbon], Vec::new(), None).expect("valid graph")
}

#[test]
fn identifier_bundle_uses_the_required_dependency_order() {
    let engine = IdentifierEngineV1::available();

    let identifiers = evaluate_identifiers_v1(&engine, &graph()).expect("identifiers complete");

    assert_eq!(
        identifiers,
        DocumentMoleculeReportIdentifiersV1::Available {
            canonical_smiles: "C".to_owned(),
            standard_inchi: "InChI=1S/CH4/h1H4".to_owned(),
            standard_inchi_key: "VNWKTOKETHGBQD-UHFFFAOYSA-N".to_owned(),
        }
    );
    assert_eq!(engine.calls.into_inner(), ["smiles", "inchi", "inchi_key"]);
}

#[test]
fn identifier_engine_unavailability_is_a_closed_per_record_outcome() {
    let engine = IdentifierEngineV1::failing(
        IdentifierStageV1::Inchi,
        ChemistryError::OperationUnavailable {
            operation: "molecule_to_inchi",
        },
    );

    let identifiers = evaluate_identifiers_v1(&engine, &graph()).expect("unavailable is a receipt");

    assert_eq!(
        identifiers,
        DocumentMoleculeReportIdentifiersV1::Unavailable(
            DocumentMoleculeReportIdentifierUnavailableReasonV1::ChemistryUnavailable,
        )
    );
    assert_eq!(engine.calls.into_inner(), ["smiles", "inchi"]);
}

#[test]
fn identifier_graph_rejection_is_a_closed_unsupported_outcome_without_detail() {
    let engine = IdentifierEngineV1::failing(
        IdentifierStageV1::Smiles,
        ChemistryError::UnsupportedNativeRequest {
            reason: "private native detail".to_owned(),
        },
    );

    let identifiers = evaluate_identifiers_v1(&engine, &graph()).expect("unsupported is a receipt");

    assert_eq!(
        identifiers,
        DocumentMoleculeReportIdentifiersV1::Unavailable(
            DocumentMoleculeReportIdentifierUnavailableReasonV1::UnsupportedMolecule,
        )
    );
}

#[test]
fn identifier_resource_exhaustion_refuses_the_operation_boundary() {
    let engine = IdentifierEngineV1::failing(
        IdentifierStageV1::InchiKey,
        ChemistryError::ResourceExhausted {
            operation: "inchi_to_inchi_key",
        },
    );

    assert_eq!(
        evaluate_identifiers_v1(&engine, &graph()),
        Err(DocumentMoleculeReportErrorV1::ResourceAllocation)
    );
}

#[test]
fn identifier_dto_requires_one_complete_closed_outcome() {
    let available = serde_json::json!({
        "kind": "available",
        "canonical_smiles": "C",
        "standard_inchi": "InChI=1S/CH4/h1H4",
        "standard_inchi_key": "VNWKTOKETHGBQD-UHFFFAOYSA-N"
    });
    assert!(
        serde_json::from_value::<DocumentMoleculeReportIdentifiersSummaryV1>(available).is_ok()
    );
    for invalid in [
        serde_json::json!({
            "kind": "available",
            "canonical_smiles": "C",
            "standard_inchi": "InChI=1S/CH4/h1H4"
        }),
        serde_json::json!({
            "kind": "unavailable",
            "reason": "unsupported_molecule",
            "standard_inchi": "InChI=1S/CH4/h1H4"
        }),
        serde_json::json!({"kind": "unknown"}),
    ] {
        assert!(
            serde_json::from_value::<DocumentMoleculeReportIdentifiersSummaryV1>(invalid).is_err()
        );
    }
}

struct IdentifierResourceEngineV1;

impl ChemEngine for IdentifierResourceEngineV1 {
    fn smiles_to_molecule(
        &self,
        _: &str,
    ) -> Result<ferrum_chemistry::SmilesMolecule, ChemistryError> {
        unreachable!("report test does not parse SMILES")
    }

    fn generate_2d_coordinates(
        &self,
        _: &MolGraph,
    ) -> Result<ferrum_chemistry::Coordinates, ChemistryError> {
        unreachable!("report test does not generate coordinates")
    }

    fn molecule_composition(
        &self,
        _: &MolGraph,
    ) -> Result<ferrum_chemistry::MoleculeComposition, ChemistryError> {
        let key = ferrum_chemistry::CompositionElementKey::new(
            AtomicNumber::try_from(6).expect("carbon is supported"),
            None,
        );
        let entry = ferrum_chemistry::MoleculeCompositionEntry::new(key, 1, 12.0)
            .expect("valid composition entry");
        ferrum_chemistry::MoleculeComposition::from_entries(0, 12.0, vec![entry]).map_err(|error| {
            ChemistryError::MalformedNativeResponse {
                reason: error.to_string(),
            }
        })
    }

    fn molecule_to_smiles(&self, _: &MolGraph) -> Result<String, ChemistryError> {
        Err(ChemistryError::ResourceExhausted {
            operation: "molecule_to_smiles",
        })
    }

    fn kekulize(
        &self,
        _: &MolGraph,
        _: ferrum_chemistry::KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        unreachable!("report test does not kekulize")
    }
}

struct IdentifierResourceRuntimeV1;

impl ChemistryRuntimeV1 for IdentifierResourceRuntimeV1 {
    fn with_engine<T>(
        &self,
        operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
    ) -> Result<T, ChemistryRuntimeErrorV1> {
        operation(&IdentifierResourceEngineV1)
    }
}

#[test]
fn identifier_resource_exhaustion_becomes_the_typed_operation_refusal() {
    let session = DocumentSession::load(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect("document loads");
    let snapshot = session.snapshot().expect("document persists");
    let observation = session.observe(0).expect("document projects");
    let digest: String = snapshot
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let request = serde_json::json!({
        "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
        "request_id": "identifier-resource",
        "operation": {
            "kind": "document.molecule.report.v1",
            "snapshot": {
                "cdml": snapshot.cdml(),
                "revision": snapshot.revision(),
                "digest_hex": digest
            },
            "molecule_ids": [observation.projection().molecules()[0].document_object_id().as_str()]
        }
    });

    let envelope =
        execute_operation_with_runtime_v1(&request.to_string(), &IdentifierResourceRuntimeV1)
            .expect("request is valid");

    let OperationProtocolEnvelopeV1::Error(response) = envelope else {
        panic!("identifier resource exhaustion must refuse the operation");
    };
    assert_eq!(
        response.error.category,
        OperationProtocolErrorCategoryV1::ResourceLimit
    );
}

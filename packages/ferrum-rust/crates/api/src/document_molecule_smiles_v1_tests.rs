use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, NativeChemEngine,
    SmilesMolecule,
};
use ferrum_document::artifact_publication_v1::ArtifactPublicationDurabilityV1;
use ferrum_document::{DocumentObjectIdV1, DocumentSession};

use crate::document_molecule_smiles_v1::export_prepared_document_molecule_smiles_with_engine_v1;

use super::{
    DOCUMENT_MOLECULE_SMILES_PROFILE_V1, DOCUMENT_MOLECULE_SMILES_SCHEMA_V1,
    DocumentMoleculeSmilesErrorV1, DocumentMoleculeSmilesRequestV1,
    PreparedDocumentMoleculeSmilesV1, export_prepared_document_molecule_smiles_v1,
    prepare_document_molecule_smiles_v1, publish_document_molecule_smiles_v1,
};

static NEXT_PUBLICATION_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct RecordingEngine {
    requests: RefCell<Vec<MolGraph>>,
}

impl ChemEngine for RecordingEngine {
    fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "smiles_to_molecule",
        })
    }

    fn generate_2d_coordinates(&self, _molecule: &MolGraph) -> Result<Coordinates, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "generate_2d_coordinates",
        })
    }

    fn molecule_to_smiles(&self, molecule: &MolGraph) -> Result<String, ChemistryError> {
        self.requests.borrow_mut().push(molecule.clone());
        Ok("C[15NH3+]".to_owned())
    }

    fn kekulize(
        &self,
        _molecule: &MolGraph,
        _options: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "kekulize",
        })
    }
}

fn observation_and_request(
    bond_type: &str,
) -> (
    ferrum_document::SessionDocumentObservationV1,
    DocumentMoleculeSmilesRequestV1,
) {
    let source = format!(
        concat!(
            "<cdml version=\"1.0\"><molecule id=\"m1\">",
            "<atom id=\"a1\" name=\"N\" charge=\"1\" isotope=\"15\" ",
            "explicit_hydrogens=\"3\"><point x=\"0\" y=\"0\"/></atom>",
            "<atom id=\"a2\" name=\"C\"><point x=\"1\" y=\"0\"/></atom>",
            "<bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"{}\"/>",
            "</molecule></cdml>"
        ),
        bond_type
    );
    let session = DocumentSession::load(&source).expect("source loads");
    let observation = session.observe(0).expect("source projects");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("durable molecule")
        .clone();
    let request = DocumentMoleculeSmilesRequestV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        molecule_id,
    );
    (observation, request)
}

#[test]
fn exact_direct_root_prepares_before_engine_work_and_returns_owned_receipt() {
    let (observation, request) = observation_and_request("n1");
    let before = observation.clone();
    let prepared = prepare_document_molecule_smiles_v1(&observation, &request)
        .expect("supported direct root prepares");
    assert_eq!(prepared.source_revision(), 0);
    assert_eq!(prepared.source_digest(), observation.snapshot().digest());
    assert_eq!(prepared.molecule_id(), request.molecule_id());

    let engine = RecordingEngine::default();
    let result = export_prepared_document_molecule_smiles_with_engine_v1(&engine, prepared)
        .expect("prepared graph exports");
    assert_eq!(result.schema(), DOCUMENT_MOLECULE_SMILES_SCHEMA_V1);
    assert_eq!(result.profile(), DOCUMENT_MOLECULE_SMILES_PROFILE_V1);
    assert_eq!(result.source_revision(), 0);
    assert_eq!(result.source_digest(), observation.snapshot().digest());
    assert_eq!(result.molecule_id(), request.molecule_id());
    assert_eq!(result.smiles(), "C[15NH3+]");
    let requests = engine.requests.borrow();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].atoms()[0].atomic_number().symbol(), "N");
    assert_eq!(requests[0].atoms()[0].formal_charge(), Some(1));
    assert_eq!(requests[0].atoms()[0].isotope(), Some(15));
    assert_eq!(requests[0].atoms()[0].explicit_hydrogens(), Some(3));
    assert_eq!(observation, before);
}

#[test]
fn authenticated_receipt_publishes_one_exact_smiles_line_without_document_effects() {
    let (observation, request) = observation_and_request("n1");
    let before = observation.clone();
    let prepared = prepare_document_molecule_smiles_v1(&observation, &request)
        .expect("supported direct root prepares");
    let receipt = export_prepared_document_molecule_smiles_with_engine_v1(
        &RecordingEngine::default(),
        prepared,
    )
    .expect("prepared graph exports");
    let sequence = NEXT_PUBLICATION_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let temporary_root = std::fs::canonicalize(std::env::temp_dir())
        .expect("temporary root resolves without a symlink spelling");
    let directory = temporary_root.join(format!(
        "ferrum-document-smiles-publication-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("isolated output directory is created");
    let destination = directory.join("molecule.smi");

    let outcome = publish_document_molecule_smiles_v1(&receipt, destination.clone())
        .expect("receipt publishes");
    assert_eq!(outcome.receipt().destination(), destination);
    assert!(matches!(
        outcome.durability(),
        ArtifactPublicationDurabilityV1::Confirmed
            | ArtifactPublicationDurabilityV1::DirectoryEntryUnconfirmed
    ));
    assert_eq!(
        std::fs::read(&destination).expect("artifact is readable"),
        b"C[15NH3+]\n"
    );
    assert_eq!(observation, before);

    std::fs::remove_file(destination).expect("artifact cleanup succeeds");
    std::fs::remove_dir(directory).expect("directory cleanup succeeds");
}

#[test]
fn stale_foreign_and_drawing_style_requests_never_reach_the_engine() {
    let (observation, request) = observation_and_request("w1");
    let engine = RecordingEngine::default();
    assert!(matches!(
        prepare_document_molecule_smiles_v1(&observation, &request),
        Err(DocumentMoleculeSmilesErrorV1::UnsupportedMolecule(_))
    ));

    let stale = DocumentMoleculeSmilesRequestV1::new(
        1,
        *request.expected_digest(),
        request.molecule_id().clone(),
    );
    assert!(matches!(
        prepare_document_molecule_smiles_v1(&observation, &stale),
        Err(DocumentMoleculeSmilesErrorV1::Observation(_))
    ));

    let foreign = DocumentObjectIdV1::parse(
        "ferrum-document-object-v1/63646d6c2f6d6f6563756c65/source/666f726569676e",
    )
    .expect("foreign selector grammar");
    let foreign = DocumentMoleculeSmilesRequestV1::new(0, *request.expected_digest(), foreign);
    assert!(matches!(
        prepare_document_molecule_smiles_v1(&observation, &foreign),
        Err(DocumentMoleculeSmilesErrorV1::Observation(_))
    ));
    assert!(engine.requests.borrow().is_empty());
}

#[test]
fn public_execution_signature_requires_the_concrete_native_engine() {
    let function: fn(&NativeChemEngine, PreparedDocumentMoleculeSmilesV1) -> Result<_, _> =
        export_prepared_document_molecule_smiles_v1;
    let _ = function;
}

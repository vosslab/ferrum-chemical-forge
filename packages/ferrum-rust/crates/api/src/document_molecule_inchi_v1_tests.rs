use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, InchiMode, KekulizeOptions, MolGraph, SmilesMolecule,
};
use ferrum_document::{DocumentObjectIdV1, DocumentSession};

use super::{
    DocumentMoleculeInchiError, export_document_molecule_inchi_v1,
    export_prepared_document_molecule_inchi_receipt_v1, export_prepared_document_molecule_inchi_v1,
    prepare_document_molecule_inchi_v1, publish_document_molecule_inchi_v1,
};

static NEXT_PUBLICATION_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct RecordingEngine {
    requests: RefCell<Vec<(MolGraph, InchiMode)>>,
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

    fn molecule_to_inchi(
        &self,
        molecule: &MolGraph,
        mode: InchiMode,
    ) -> Result<String, ChemistryError> {
        self.requests.borrow_mut().push((molecule.clone(), mode));
        Ok(match mode {
            InchiMode::Standard => "InChI=1S/CH4/h1H4",
            InchiMode::FixedHydrogen => "InChI=1/CH4/h1H4/f/h1H4",
        }
        .to_owned())
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

fn source(bond_type: &str) -> String {
    format!(
        concat!(
            "<cdml version=\"1.0\"><molecule id=\"m1\" name=\"Methane\">",
            "<atom id=\"a1\" name=\"C\"><point x=\"10\" y=\"20\"/></atom>",
            "<atom id=\"a2\" name=\"H\"><point x=\"30\" y=\"20\"/></atom>",
            "<bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"{}\"/>",
            "</molecule></cdml>"
        ),
        bond_type
    )
}

fn observation_and_id(
    source: &str,
) -> (
    ferrum_document::SessionDocumentObservationV1,
    DocumentObjectIdV1,
) {
    let session = DocumentSession::load(source).expect("source must load");
    let observation = session.observe(0).expect("source must project");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("source molecule has durable identity")
        .clone();
    (observation, molecule_id)
}

#[test]
fn preparation_freezes_exact_provenance_before_native_execution() {
    let (observation, molecule_id) = observation_and_id(&source("n1"));
    let before = observation.clone();
    let prepared =
        prepare_document_molecule_inchi_v1(&observation, &molecule_id, InchiMode::FixedHydrogen)
            .expect("supported graph must prepare");

    assert_eq!(prepared.source_revision(), 0);
    assert_eq!(prepared.source_digest(), observation.snapshot().digest());
    assert_eq!(prepared.molecule_id(), &molecule_id);
    assert_eq!(prepared.mode(), InchiMode::FixedHydrogen);
    assert_eq!(observation, before);

    let engine = RecordingEngine::default();
    assert_eq!(
        export_prepared_document_molecule_inchi_v1(&engine, &prepared)
            .expect("prepared graph must export"),
        "InChI=1/CH4/h1H4/f/h1H4"
    );
    let requests = engine.requests.borrow();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].1, InchiMode::FixedHydrogen);
    assert_eq!(requests[0].0.atoms()[0].atomic_number().symbol(), "C");
    assert_eq!(requests[0].0.atoms()[1].atomic_number().symbol(), "H");
    assert_eq!(requests[0].0.bonds().len(), 1);
    assert!(requests[0].0.coordinates().is_none());
}

#[test]
fn one_shot_export_preserves_the_selected_closed_mode() {
    let (observation, molecule_id) = observation_and_id(&source("n1"));
    let engine = RecordingEngine::default();

    let value =
        export_document_molecule_inchi_v1(&engine, &observation, &molecule_id, InchiMode::Standard)
            .expect("supported graph must export");

    assert_eq!(value, "InChI=1S/CH4/h1H4");
    assert_eq!(engine.requests.borrow()[0].1, InchiMode::Standard);
}

#[test]
fn owned_receipt_publishes_one_exact_inchi_line_without_observation_effects() {
    let (observation, molecule_id) = observation_and_id(&source("n1"));
    let before = observation.clone();
    let prepared =
        prepare_document_molecule_inchi_v1(&observation, &molecule_id, InchiMode::Standard)
            .expect("supported graph prepares");
    let receipt =
        export_prepared_document_molecule_inchi_receipt_v1(&RecordingEngine::default(), prepared)
            .expect("prepared graph returns an owned receipt");
    assert_eq!(receipt.source_revision(), 0);
    assert_eq!(receipt.source_digest(), observation.snapshot().digest());
    assert_eq!(receipt.molecule_id(), &molecule_id);
    assert_eq!(receipt.mode(), InchiMode::Standard);
    assert_eq!(receipt.inchi(), "InChI=1S/CH4/h1H4");

    let sequence = NEXT_PUBLICATION_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let temporary_root = std::fs::canonicalize(std::env::temp_dir())
        .expect("temporary root resolves without a symlink spelling");
    let directory = temporary_root.join(format!(
        "ferrum-document-inchi-publication-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("isolated output directory is created");
    let destination = directory.join("methane.inchi");
    let outcome = publish_document_molecule_inchi_v1(&receipt, destination.clone())
        .expect("receipt publishes");
    assert_eq!(outcome.receipt().destination(), destination);
    assert_eq!(
        std::fs::read(&destination).expect("published InChI is readable"),
        b"InChI=1S/CH4/h1H4\n"
    );
    assert_eq!(observation, before);
    std::fs::remove_file(destination).expect("artifact cleanup succeeds");
    std::fs::remove_dir(directory).expect("directory cleanup succeeds");
}

#[test]
fn invalid_target_or_styled_bond_never_reaches_the_engine() {
    let (observation, _molecule_id) = observation_and_id(&source("w1"));
    let engine = RecordingEngine::default();
    let styled_id = observation.projection().molecules()[0]
        .id()
        .expect("durable molecule")
        .clone();
    let styled_error =
        export_document_molecule_inchi_v1(&engine, &observation, &styled_id, InchiMode::Standard)
            .expect_err("drawing style must not cross the chemistry boundary");
    assert!(matches!(
        &styled_error,
        DocumentMoleculeInchiError::UnsupportedMolecule(_)
    ));
    assert!(styled_error.to_string().contains("native InChI boundary"));
    assert!(!styled_error.to_string().contains("coordinate generation"));
    let unknown = DocumentObjectIdV1::parse(
        "ferrum-document-object-v1/6d6f6c6563756c65/source/6d697373696e67",
    )
    .expect("test selector uses the closed grammar");
    assert!(matches!(
        export_document_molecule_inchi_v1(&engine, &observation, &unknown, InchiMode::Standard,),
        Err(DocumentMoleculeInchiError::UnknownMolecule { .. })
    ));
    assert!(engine.requests.borrow().is_empty());
}

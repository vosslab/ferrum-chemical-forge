use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, MolblockVersion,
    NativeChemEngine, SmilesMolecule,
};
use ferrum_document::artifact_publication_v1::ArtifactPublicationDurabilityV1;
use ferrum_document::{DocumentObjectIdV1, DocumentSession};

use crate::document_molecule_molblock_v1::{
    PreparedDocumentMoleculeMolblockV1, export_prepared_document_molecule_molblock_with_engine_v1,
};

use super::{
    DOCUMENT_MOLECULE_MOLBLOCK_PROFILE_V1, DOCUMENT_MOLECULE_MOLBLOCK_SCHEMA_V1,
    DocumentMoleculeMolblockErrorV1, DocumentMoleculeMolblockRequestV1,
    export_prepared_document_molecule_molblock_v1, prepare_document_molecule_molblock_v1,
    publish_document_molecule_molblock_v1,
};

static NEXT_PUBLICATION_DIRECTORY: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct RecordingEngine {
    requests: RefCell<Vec<(MolGraph, MolblockVersion, Option<String>)>>,
    title_available: bool,
}

impl RecordingEngine {
    fn with_title() -> Self {
        Self {
            requests: RefCell::new(Vec::new()),
            title_available: true,
        }
    }
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

    fn molecule_to_molblock(
        &self,
        molecule: &MolGraph,
        version: MolblockVersion,
    ) -> Result<String, ChemistryError> {
        self.requests
            .borrow_mut()
            .push((molecule.clone(), version, None));
        Ok(match version {
            MolblockVersion::V2000 => "exact V2000\nM  END\n".to_owned(),
            MolblockVersion::V3000 => "exact V3000\nM  V30 END CTAB\nM  END\n".to_owned(),
        })
    }

    fn molecule_to_molblock_with_title(
        &self,
        molecule: &MolGraph,
        version: MolblockVersion,
        title: &str,
    ) -> Result<String, ChemistryError> {
        if !self.title_available {
            return Err(ChemistryError::OperationUnavailable {
                operation: "molecule_to_molblock_with_title",
            });
        }
        self.requests
            .borrow_mut()
            .push((molecule.clone(), version, Some(title.to_owned())));
        Ok(format!("{title}\n{}\nM  END\n", version_label(version)))
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
    version: MolblockVersion,
) -> (
    ferrum_document::SessionDocumentObservationV1,
    DocumentMoleculeMolblockRequestV1,
) {
    let source = format!(
        concat!(
            "<cdml version=\"1.0\"><molecule id=\"m1\">",
            "<atom id=\"a1\" name=\"N\" charge=\"1\" isotope=\"15\" ",
            "explicit_hydrogens=\"3\"><point x=\"2.5\" y=\"7.5\"/></atom>",
            "<atom id=\"a2\" name=\"C\"><point x=\"12.5\" y=\"-4\"/></atom>",
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
    let request = DocumentMoleculeMolblockRequestV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        molecule_id,
        version,
    );
    (observation, request)
}

#[test]
fn exact_root_retains_graph_and_transforms_document_coordinates_for_each_syntax() {
    for version in [MolblockVersion::V2000, MolblockVersion::V3000] {
        let (observation, request) = observation_and_request("n1", version);
        let before = observation.clone();
        let prepared = prepare_document_molecule_molblock_v1(&observation, &request)
            .expect("supported direct root prepares");
        assert_eq!(prepared.source_revision(), 0);
        assert_eq!(prepared.source_digest(), observation.snapshot().digest());
        assert_eq!(prepared.molecule_id(), request.molecule_id());
        assert_eq!(prepared.version(), version);
        assert_eq!(prepared.title(), None);

        let engine = RecordingEngine::default();
        let receipt = export_prepared_document_molecule_molblock_with_engine_v1(&engine, prepared)
            .expect("prepared graph exports");
        assert_eq!(receipt.schema(), DOCUMENT_MOLECULE_MOLBLOCK_SCHEMA_V1);
        assert_eq!(receipt.profile(), DOCUMENT_MOLECULE_MOLBLOCK_PROFILE_V1);
        assert_eq!(receipt.source_revision(), 0);
        assert_eq!(receipt.source_digest(), observation.snapshot().digest());
        assert_eq!(receipt.molecule_id(), request.molecule_id());
        assert_eq!(receipt.version(), version);
        assert_eq!(receipt.title(), None);
        assert!(receipt.molblock().contains(version_label(version)));

        let requests = engine.requests.borrow();
        let (graph, actual_version, title) = &requests[0];
        assert_eq!(*actual_version, version);
        assert_eq!(title, &None);
        assert_eq!(graph.atoms()[0].atomic_number().symbol(), "N");
        assert_eq!(graph.atoms()[0].formal_charge(), Some(1));
        assert_eq!(graph.atoms()[0].isotope(), Some(15));
        assert_eq!(graph.atoms()[0].explicit_hydrogens(), Some(3));
        let coordinates = graph.coordinates().expect("coordinate-bearing graph");
        assert_eq!(coordinates.points()[0].x(), 2.5);
        assert_eq!(coordinates.points()[0].y(), -7.5);
        assert_eq!(coordinates.points()[1].x(), 12.5);
        assert_eq!(coordinates.points()[1].y(), 4.0);
        assert_eq!(observation, before);
    }
}

#[test]
fn authenticated_receipt_publishes_exact_native_bytes_without_document_effects() {
    let (observation, request) = observation_and_request("n1", MolblockVersion::V3000);
    let before = observation.clone();
    let prepared = prepare_document_molecule_molblock_v1(&observation, &request)
        .expect("supported direct root prepares");
    let receipt = export_prepared_document_molecule_molblock_with_engine_v1(
        &RecordingEngine::default(),
        prepared,
    )
    .expect("prepared graph exports");
    let sequence = NEXT_PUBLICATION_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let temporary_root = std::fs::canonicalize(std::env::temp_dir())
        .expect("temporary root resolves without a symlink spelling");
    let directory = temporary_root.join(format!(
        "ferrum-document-molfile-publication-{}-{sequence}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).expect("isolated output directory is created");
    let destination = directory.join("molecule.mol");

    let outcome = publish_document_molecule_molblock_v1(&receipt, destination.clone())
        .expect("receipt publishes");
    assert_eq!(outcome.receipt().destination(), destination);
    assert!(matches!(
        outcome.durability(),
        ArtifactPublicationDurabilityV1::Confirmed
            | ArtifactPublicationDurabilityV1::DirectoryEntryUnconfirmed
    ));
    assert_eq!(
        std::fs::read(&destination).expect("artifact is readable"),
        receipt.molblock().as_bytes()
    );
    assert_eq!(observation, before);

    std::fs::remove_file(destination).expect("artifact cleanup succeeds");
    std::fs::remove_dir(directory).expect("directory cleanup succeeds");
}

#[test]
fn stale_foreign_and_drawing_style_requests_never_reach_the_engine() {
    let (observation, request) = observation_and_request("w1", MolblockVersion::V2000);
    let engine = RecordingEngine::default();
    assert!(matches!(
        prepare_document_molecule_molblock_v1(&observation, &request),
        Err(DocumentMoleculeMolblockErrorV1::UnsupportedMolecule(_))
    ));

    let stale = DocumentMoleculeMolblockRequestV1::new(
        1,
        *request.expected_digest(),
        request.molecule_id().clone(),
        request.version(),
    );
    assert!(matches!(
        prepare_document_molecule_molblock_v1(&observation, &stale),
        Err(DocumentMoleculeMolblockErrorV1::Observation(_))
    ));

    let foreign = DocumentObjectIdV1::parse(
        "ferrum-document-object-v1/63646d6c2f6d6f6563756c65/source/666f726569676e",
    )
    .expect("foreign selector grammar");
    let foreign = DocumentMoleculeMolblockRequestV1::new(
        0,
        *request.expected_digest(),
        foreign,
        request.version(),
    );
    assert!(matches!(
        prepare_document_molecule_molblock_v1(&observation, &foreign),
        Err(DocumentMoleculeMolblockErrorV1::Observation(_))
    ));

    assert!(engine.requests.borrow().is_empty());
}

#[test]
fn authored_title_is_frozen_and_written_without_text_postprocessing() {
    let named_source = concat!(
        "<cdml version=\"1.0\"><molecule id=\"m1\" name=\"authored title\">",
        "<atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "</molecule></cdml>"
    );
    let named_session = DocumentSession::load(named_source).expect("named source loads");
    let named_observation = named_session.observe(0).expect("named source projects");
    let named_id = named_observation.projection().molecules()[0]
        .id()
        .expect("named durable molecule")
        .clone();
    let named_request = DocumentMoleculeMolblockRequestV1::new(
        0,
        *named_observation.snapshot().digest(),
        named_id,
        MolblockVersion::V2000,
    );
    let prepared = prepare_document_molecule_molblock_v1(&named_observation, &named_request)
        .expect("named direct root prepares");
    assert_eq!(prepared.title(), Some("authored title"));

    let engine = RecordingEngine::with_title();
    let receipt = export_prepared_document_molecule_molblock_with_engine_v1(&engine, prepared)
        .expect("title-aware engine exports");
    assert_eq!(receipt.title(), Some("authored title"));
    assert!(receipt.molblock().starts_with("authored title\n"));
    let requests = engine.requests.borrow();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].2.as_deref(), Some("authored title"));

    let prepared = prepare_document_molecule_molblock_v1(&named_observation, &named_request)
        .expect("named direct root prepares again");
    let old_engine = RecordingEngine::default();
    let error = export_prepared_document_molecule_molblock_with_engine_v1(&old_engine, prepared)
        .expect_err("missing optional title operation cannot drop the title");
    assert!(matches!(
        error,
        DocumentMoleculeMolblockErrorV1::Chemistry(ChemistryError::OperationUnavailable {
            operation: "molecule_to_molblock_with_title"
        })
    ));
    assert!(old_engine.requests.borrow().is_empty());
}

#[test]
fn public_execution_signature_requires_the_concrete_native_engine() {
    let function: fn(&NativeChemEngine, PreparedDocumentMoleculeMolblockV1) -> Result<_, _> =
        export_prepared_document_molecule_molblock_v1;
    let _ = function;
}

const fn version_label(version: MolblockVersion) -> &'static str {
    match version {
        MolblockVersion::V2000 => "V2000",
        MolblockVersion::V3000 => "V3000",
    }
}

use std::cell::RefCell;

use crate::{DocumentObjectIdV1, DocumentSession};
use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, MolblockVersion,
    NativeChemEngine, SmilesMolecule,
};

use super::{PreparedDocumentMoleculeSdfV1, export_prepared_document_molecule_sdf_with_engine_v1};

use super::{
    DOCUMENT_MOLECULE_SDF_PROFILE_V1, DOCUMENT_MOLECULE_SDF_SCHEMA_V1, DocumentMoleculeSdfErrorV1,
    DocumentMoleculeSdfRequestV1, export_prepared_document_molecule_sdf_v1,
    prepare_document_molecule_sdf_v1,
};

#[derive(Default)]
struct RecordingEngine {
    requests: RefCell<Vec<(MolblockVersion, String)>>,
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
        _molecule: &MolGraph,
        version: MolblockVersion,
    ) -> Result<String, ChemistryError> {
        self.requests.borrow_mut().push((version, String::new()));
        Ok("\nFerrum\n2D\nM  END\n".to_owned())
    }

    fn molecule_to_molblock_with_title(
        &self,
        _molecule: &MolGraph,
        version: MolblockVersion,
        title: &str,
    ) -> Result<String, ChemistryError> {
        self.requests.borrow_mut().push((version, title.to_owned()));
        Ok(format!("{title}\nFerrum\n2D\nM  END\n"))
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

fn request_for(
    observation: &crate::SessionDocumentObservationV1,
    version: MolblockVersion,
) -> DocumentMoleculeSdfRequestV1 {
    let molecule_id = observation.projection().molecules()[0]
        .document_object_id()
        .clone();
    DocumentMoleculeSdfRequestV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        molecule_id,
        version,
    )
}

#[test]
fn imported_metadata_overrides_display_name_and_preserves_duplicate_property_order() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\" name=\"display name\">",
        "<atom id=\"a\" name=\"C\"><point x=\"2\" y=\"7\"/></atom>",
        "<f:interchange-record xmlns:f=\"urn:ferrum-chemical-forge:interchange-import:v1\" ",
        "encoding=\"utf8-hex-v1\" title=\"496d706f72746564207469746c65\">",
        "<f:property name=\"4e4f5445\" value=\"6669727374\"/>",
        "<f:property name=\"4e4f5445\" value=\"7365636f6e64\"/>",
        "</f:interchange-record></molecule></cdml>",
    );
    let session = DocumentSession::load(source).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
    let request = request_for(&observation, MolblockVersion::V3000);
    let before = observation.clone();

    let prepared = prepare_document_molecule_sdf_v1(&observation, &request)
        .expect("exact imported metadata prepares");
    assert_eq!(prepared.title(), "Imported title");
    assert_eq!(
        prepared
            .properties()
            .iter()
            .map(|property| (property.name(), property.value()))
            .collect::<Vec<_>>(),
        [("NOTE", "first"), ("NOTE", "second")],
    );

    let engine = RecordingEngine::default();
    let receipt = export_prepared_document_molecule_sdf_with_engine_v1(&engine, prepared)
        .expect("prepared SDF record exports");
    assert_eq!(receipt.schema(), DOCUMENT_MOLECULE_SDF_SCHEMA_V1);
    assert_eq!(receipt.profile(), DOCUMENT_MOLECULE_SDF_PROFILE_V1);
    assert_eq!(receipt.source_digest(), observation.snapshot().digest());
    assert_eq!(receipt.molecule_id(), request.molecule_id());
    assert_eq!(receipt.version(), MolblockVersion::V3000);
    assert_eq!(receipt.title(), "Imported title");
    assert!(receipt.sdf().starts_with("Imported title\nFerrum\n"));
    assert!(
        receipt
            .sdf()
            .contains(">  <NOTE>\nfirst\n\n>  <NOTE>\nsecond\n\n")
    );
    assert!(receipt.sdf().ends_with("$$$$\n"));
    assert_eq!(observation, before);
}

#[test]
fn ordinary_name_and_blank_title_take_the_exact_native_paths() {
    for (name_attribute, expected_title) in [(" name=\"ordinary\"", "ordinary"), ("", "")] {
        let source = format!(
            concat!(
                "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"{}><atom id=\"a\" name=\"O\">",
                "<point x=\"0\" y=\"0\"/></atom></molecule></cdml>"
            ),
            name_attribute,
        );
        let session = DocumentSession::load(&source).expect("fixture loads");
        let observation = session.observe(0).expect("fixture projects");
        let request = request_for(&observation, MolblockVersion::V2000);
        let prepared = prepare_document_molecule_sdf_v1(&observation, &request)
            .expect("ordinary molecule prepares");
        let engine = RecordingEngine::default();
        let receipt = export_prepared_document_molecule_sdf_with_engine_v1(&engine, prepared)
            .expect("ordinary record exports");

        assert_eq!(receipt.title(), expected_title);
        assert!(receipt.properties().is_empty());
        assert_eq!(engine.requests.borrow()[0].1, expected_title);
        assert!(receipt.sdf().ends_with("M  END\n$$$$\n"));
    }
}

#[test]
fn stale_foreign_and_malformed_metadata_are_rejected_before_native_execution() {
    let malformed = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
        "</atom><f:interchange-record xmlns:f=\"urn:ferrum-chemical-forge:interchange-import:v1\" ",
        "encoding=\"utf8-hex-v1\" title=\"0\"/></molecule></cdml>",
    );
    let session = DocumentSession::load(malformed).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
    let request = request_for(&observation, MolblockVersion::V2000);
    assert!(matches!(
        prepare_document_molecule_sdf_v1(&observation, &request),
        Err(DocumentMoleculeSdfErrorV1::Metadata(_))
    ));

    let stale = DocumentMoleculeSdfRequestV1::new(
        1,
        *request.expected_digest(),
        request.molecule_id().clone(),
        request.version(),
    );
    assert!(matches!(
        prepare_document_molecule_sdf_v1(&observation, &stale),
        Err(DocumentMoleculeSdfErrorV1::Observation(_))
    ));

    let foreign = DocumentObjectIdV1::from_entropy_bytes([0; 16]);
    let foreign = DocumentMoleculeSdfRequestV1::new(
        0,
        *request.expected_digest(),
        foreign,
        request.version(),
    );
    assert!(matches!(
        prepare_document_molecule_sdf_v1(&observation, &foreign),
        Err(DocumentMoleculeSdfErrorV1::Observation(_))
    ));
}

#[test]
fn public_execution_signature_requires_the_concrete_native_engine() {
    let function: fn(&NativeChemEngine, PreparedDocumentMoleculeSdfV1) -> Result<_, _> =
        export_prepared_document_molecule_sdf_v1;
    let _ = function;
}

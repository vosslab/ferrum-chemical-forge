use std::cell::RefCell;

use crate::{DocumentSession, SessionDocumentObservationV1};
use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, InchiMode, MolGraph, MolblockVersion,
    NativeTextOutputLimit, SdfRecord, SmilesMolecule,
};

use super::{
    DOCUMENT_MOLECULE_EXPORT_TEXT_UTF8_BYTES, DocumentMoleculeExportError,
    DocumentMoleculeExportFormat, DocumentMoleculeExportRequest, export_prepared_document_molecule,
    prepare_document_molecule_export,
};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\">",
    "<molecule id=\"root\" name=\"Example\">",
    "<atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
    "</molecule></cdml>",
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExportCall {
    Molfile(MolblockVersion, u64),
    Sdf(MolblockVersion, u64),
    Smiles(u64),
    Inchi(InchiMode, u64),
}

struct RecordingEngine {
    calls: RefCell<Vec<ExportCall>>,
}

impl RecordingEngine {
    fn new() -> Self {
        Self {
            calls: RefCell::new(Vec::new()),
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

    fn molecule_to_smiles(
        &self,
        _molecule: &MolGraph,
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        self.calls
            .borrow_mut()
            .push(ExportCall::Smiles(limit.bytes()));
        Ok("SMILES".to_owned())
    }

    fn molecule_to_molblock_with_title(
        &self,
        _molecule: &MolGraph,
        version: MolblockVersion,
        title: &str,
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        assert_eq!(title, "Example");
        self.calls
            .borrow_mut()
            .push(ExportCall::Molfile(version, limit.bytes()));
        Ok("MOLFILE".to_owned())
    }

    fn molecule_to_inchi(
        &self,
        _molecule: &MolGraph,
        mode: InchiMode,
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        self.calls
            .borrow_mut()
            .push(ExportCall::Inchi(mode, limit.bytes()));
        Ok("INCHI".to_owned())
    }

    fn records_to_sdf(
        &self,
        records: &[SdfRecord],
        version: MolblockVersion,
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].title(), "Example");
        assert!(records[0].properties().is_empty());
        self.calls
            .borrow_mut()
            .push(ExportCall::Sdf(version, limit.bytes()));
        Ok("SDF".to_owned())
    }

    fn kekulize(
        &self,
        molecule: &MolGraph,
        _options: ferrum_chemistry::KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        Ok(molecule.clone())
    }
}

struct LimitEngine;

impl ChemEngine for LimitEngine {
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

    fn molecule_to_smiles(
        &self,
        _molecule: &MolGraph,
        limit: NativeTextOutputLimit,
    ) -> Result<String, ChemistryError> {
        Err(ChemistryError::TextOutputLimitExceeded {
            codec: "SMILES",
            maximum: Some(limit.bytes()),
        })
    }

    fn kekulize(
        &self,
        molecule: &MolGraph,
        _options: ferrum_chemistry::KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        Ok(molecule.clone())
    }
}

fn observation() -> SessionDocumentObservationV1 {
    let session = DocumentSession::load(SOURCE).expect("fixture loads");
    session.observe(0).expect("fixture projects")
}

fn request(
    observation: &SessionDocumentObservationV1,
    format: DocumentMoleculeExportFormat,
) -> DocumentMoleculeExportRequest {
    DocumentMoleculeExportRequest::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        observation.projection().molecules()[0]
            .document_object_id()
            .clone(),
        format,
    )
}

#[test]
fn every_selected_export_format_uses_its_exact_native_writer_and_preserves_provenance() {
    let observation = observation();
    let before = observation.clone();
    let limit = DOCUMENT_MOLECULE_EXPORT_TEXT_UTF8_BYTES as u64;
    let cases = [
        (
            DocumentMoleculeExportFormat::MolfileV2000,
            ExportCall::Molfile(MolblockVersion::V2000, limit),
            "MOLFILE",
        ),
        (
            DocumentMoleculeExportFormat::MolfileV3000,
            ExportCall::Molfile(MolblockVersion::V3000, limit),
            "MOLFILE",
        ),
        (
            DocumentMoleculeExportFormat::SdfV2000,
            ExportCall::Sdf(MolblockVersion::V2000, limit),
            "SDF",
        ),
        (
            DocumentMoleculeExportFormat::SdfV3000,
            ExportCall::Sdf(MolblockVersion::V3000, limit),
            "SDF",
        ),
        (
            DocumentMoleculeExportFormat::CanonicalSmiles,
            ExportCall::Smiles(limit),
            "SMILES",
        ),
        (
            DocumentMoleculeExportFormat::InchiStandard,
            ExportCall::Inchi(InchiMode::Standard, limit),
            "INCHI",
        ),
        (
            DocumentMoleculeExportFormat::InchiFixedHydrogen,
            ExportCall::Inchi(InchiMode::FixedHydrogen, limit),
            "INCHI",
        ),
    ];

    for (format, expected_call, expected_text) in cases {
        let engine = RecordingEngine::new();
        let prepared =
            prepare_document_molecule_export(&observation, &request(&observation, format))
                .expect("valid observation prepares");
        let receipt = export_prepared_document_molecule(&engine, prepared)
            .expect("recording writer completes");

        assert_eq!(engine.calls.into_inner(), vec![expected_call]);
        assert_eq!(receipt.format(), format);
        assert_eq!(receipt.source_revision(), before.snapshot().revision());
        assert_eq!(receipt.source_digest(), before.snapshot().digest());
        assert_eq!(
            receipt.molecule_id(),
            before.projection().molecules()[0].document_object_id()
        );
        assert_eq!(receipt.text(), expected_text);
        assert_eq!(observation, before);
    }
}

#[test]
fn native_text_limit_refusal_remains_the_typed_chemistry_error() {
    let observation = observation();
    let before = observation.clone();
    let prepared = prepare_document_molecule_export(
        &observation,
        &request(&observation, DocumentMoleculeExportFormat::CanonicalSmiles),
    )
    .expect("valid observation prepares");

    assert!(matches!(
        export_prepared_document_molecule(&LimitEngine, prepared),
        Err(DocumentMoleculeExportError::Chemistry(ChemistryError::TextOutputLimitExceeded {
            codec: "SMILES",
            maximum: Some(limit),
        })) if limit == DOCUMENT_MOLECULE_EXPORT_TEXT_UTF8_BYTES as u64
    ));
    assert_eq!(observation, before);
}

#[test]
fn unexpanded_compact_group_refuses_before_any_coordinate_or_native_export_step() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"><molecule id=\"root\">",
        "<atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group>",
        "<bond id=\"bond\" start=\"anchor\" end=\"group\" type=\"n1\"/>",
        "</molecule></cdml>",
    );
    let session = DocumentSession::load(source).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
    let before = observation.clone();

    assert!(matches!(
        prepare_document_molecule_export(
            &observation,
            &request(&observation, DocumentMoleculeExportFormat::CanonicalSmiles),
        ),
        Err(DocumentMoleculeExportError::UnsupportedMolecule(
            super::document_molecule_graph_v1::DocumentMoleculeGraphError::UnsupportedVertex {
                kind: "group",
                count: 1,
            }
        ))
    ));
    assert_eq!(observation, before);
}

//! Unified export of one authenticated direct molecule root.
//!
//! This is the private document-side owner of the selected-root export
//! operation.  It intentionally carries no runtime, path, publication, or
//! protocol-envelope state.

use crate::{
    DocumentObjectIdV1, InterchangeRecordMetadataErrorV1, SessionDocumentObservationV1,
    TypedDocument, observe_interchange_record_metadata_v1,
};
use ferrum_chemistry::{
    ChemEngine, ChemistryError, InchiMode, MolGraph, MolblockVersion, NativeTextOutputLimit,
    SdfError, SdfProperty, SdfRecord, validate_molblock_title,
};
use ferrum_core::Molecule;
use thiserror::Error;

use super::document_molecule_graph_v1::{
    DocumentMoleculeGraphError, document_molecule_coordinate_graph_v1, document_molecule_graph_v1,
};
use super::document_molecule_inspection_v1::{
    DocumentMoleculeInspectionErrorV1, copied_object_id, direct_projection_molecule_v1,
    verify_molecule_observation_v1,
};

/// Largest text representation admitted by the selected-root export core.
///
/// The protocol response ceiling is one MiB.  This smaller, fixed portion
/// leaves enough room for worst-case JSON escaping and the closed envelope.
pub const DOCUMENT_MOLECULE_EXPORT_TEXT_UTF8_BYTES: usize = 128 * 1024;
const DOCUMENT_MOLECULE_EXPORT_TEXT_LIMIT: NativeTextOutputLimit =
    match NativeTextOutputLimit::new(DOCUMENT_MOLECULE_EXPORT_TEXT_UTF8_BYTES as u64) {
        Ok(limit) => limit,
        Err(_) => panic!("document export text limit must fit the adapter envelope"),
    };

/// Closed textual representations of one selected direct molecule root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentMoleculeExportFormat {
    MolfileV2000,
    MolfileV3000,
    SdfV2000,
    SdfV3000,
    CanonicalSmiles,
    InchiStandard,
    InchiFixedHydrogen,
}

/// Immutable request authenticated against one exact observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeExportRequest {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    format: DocumentMoleculeExportFormat,
}

impl DocumentMoleculeExportRequest {
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
        format: DocumentMoleculeExportFormat,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
            format,
        }
    }
}

/// Native-handle-free source facts prepared exactly once.
#[derive(Debug)]
pub struct PreparedDocumentMoleculeExport {
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    payload: PreparedDocumentMoleculeExportPayload,
}

#[derive(Debug)]
enum PreparedDocumentMoleculeExportPayload {
    GraphOnly {
        format: GraphOnlyExportFormat,
        graph: MolGraph,
    },
    CoordinateRequired {
        format: CoordinateRequiredExportFormat,
        coordinate_graph: MolGraph,
        title: String,
        properties: Vec<SdfProperty>,
    },
}

#[derive(Clone, Copy, Debug)]
enum GraphOnlyExportFormat {
    CanonicalSmiles,
    Inchi(InchiMode),
}

#[derive(Clone, Copy, Debug)]
enum CoordinateRequiredExportFormat {
    Molfile(MolblockVersion),
    Sdf(MolblockVersion),
}

impl GraphOnlyExportFormat {
    const fn public_format(self) -> DocumentMoleculeExportFormat {
        match self {
            Self::CanonicalSmiles => DocumentMoleculeExportFormat::CanonicalSmiles,
            Self::Inchi(InchiMode::Standard) => DocumentMoleculeExportFormat::InchiStandard,
            Self::Inchi(InchiMode::FixedHydrogen) => {
                DocumentMoleculeExportFormat::InchiFixedHydrogen
            }
        }
    }
}

impl CoordinateRequiredExportFormat {
    const fn public_format(self) -> DocumentMoleculeExportFormat {
        match self {
            Self::Molfile(MolblockVersion::V2000) => DocumentMoleculeExportFormat::MolfileV2000,
            Self::Molfile(MolblockVersion::V3000) => DocumentMoleculeExportFormat::MolfileV3000,
            Self::Sdf(MolblockVersion::V2000) => DocumentMoleculeExportFormat::SdfV2000,
            Self::Sdf(MolblockVersion::V3000) => DocumentMoleculeExportFormat::SdfV3000,
        }
    }
}

/// Completed text bound to its authenticated source observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeExport {
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    format: DocumentMoleculeExportFormat,
    text: String,
}

impl DocumentMoleculeExport {
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
    #[must_use]
    pub const fn format(&self) -> DocumentMoleculeExportFormat {
        self.format
    }
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Authenticate and freeze one selected direct root before runtime loading.
pub fn prepare_document_molecule_export(
    observation: &SessionDocumentObservationV1,
    request: &DocumentMoleculeExportRequest,
) -> Result<PreparedDocumentMoleculeExport, DocumentMoleculeExportError> {
    verify_molecule_observation_v1(
        observation,
        request.expected_revision,
        &request.expected_digest,
    )?;
    let snapshot = observation.snapshot();
    let root = direct_projection_molecule_v1(observation.projection(), &request.molecule_id)?;
    let source_id = root
        .source_id()
        .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
    let document = TypedDocument::parse(snapshot.cdml())
        .map_err(DocumentMoleculeInspectionErrorV1::Document)?;
    let molecule = document
        .core_molecule(&request.molecule_id)
        .map_err(DocumentMoleculeInspectionErrorV1::CoreProjection)?
        .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
    if molecule.source_id().as_str() != source_id {
        return Err(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch.into());
    }
    let graph = document_molecule_graph_v1(&molecule)?.into_parts().0;
    let payload = match request.format {
        DocumentMoleculeExportFormat::CanonicalSmiles => {
            PreparedDocumentMoleculeExportPayload::GraphOnly {
                format: GraphOnlyExportFormat::CanonicalSmiles,
                graph,
            }
        }
        DocumentMoleculeExportFormat::InchiStandard => {
            PreparedDocumentMoleculeExportPayload::GraphOnly {
                format: GraphOnlyExportFormat::Inchi(InchiMode::Standard),
                graph,
            }
        }
        DocumentMoleculeExportFormat::InchiFixedHydrogen => {
            PreparedDocumentMoleculeExportPayload::GraphOnly {
                format: GraphOnlyExportFormat::Inchi(InchiMode::FixedHydrogen),
                graph,
            }
        }
        DocumentMoleculeExportFormat::MolfileV2000 => {
            PreparedDocumentMoleculeExportPayload::CoordinateRequired {
                format: CoordinateRequiredExportFormat::Molfile(MolblockVersion::V2000),
                coordinate_graph: document_molecule_coordinate_graph_v1(&molecule)?
                    .into_parts()
                    .0,
                title: molecule.name().unwrap_or("").to_owned(),
                properties: Vec::new(),
            }
        }
        DocumentMoleculeExportFormat::MolfileV3000 => {
            PreparedDocumentMoleculeExportPayload::CoordinateRequired {
                format: CoordinateRequiredExportFormat::Molfile(MolblockVersion::V3000),
                coordinate_graph: document_molecule_coordinate_graph_v1(&molecule)?
                    .into_parts()
                    .0,
                title: molecule.name().unwrap_or("").to_owned(),
                properties: Vec::new(),
            }
        }
        DocumentMoleculeExportFormat::SdfV2000 => {
            prepare_sdf_export_payload(&document, &molecule, source_id, MolblockVersion::V2000)?
        }
        DocumentMoleculeExportFormat::SdfV3000 => {
            prepare_sdf_export_payload(&document, &molecule, source_id, MolblockVersion::V3000)?
        }
    };
    Ok(PreparedDocumentMoleculeExport {
        source_revision: snapshot.revision(),
        source_digest: *snapshot.digest(),
        molecule_id: copied_object_id(&request.molecule_id)?,
        payload,
    })
}

fn prepare_sdf_export_payload(
    document: &TypedDocument,
    molecule: &Molecule,
    source_id: &str,
    version: MolblockVersion,
) -> Result<PreparedDocumentMoleculeExportPayload, DocumentMoleculeExportError> {
    let metadata = observe_interchange_record_metadata_v1(document, source_id)?;
    let (title, metadata_properties) = match metadata {
        Some(metadata) => metadata.into_parts(),
        None => (molecule.name().unwrap_or("").to_owned(), Vec::new()),
    };
    validate_molblock_title(&title)?;
    let properties = metadata_properties
        .into_iter()
        .map(|property| SdfProperty::new(property.name().to_owned(), property.value().to_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PreparedDocumentMoleculeExportPayload::CoordinateRequired {
        format: CoordinateRequiredExportFormat::Sdf(version),
        coordinate_graph: document_molecule_coordinate_graph_v1(molecule)?
            .into_parts()
            .0,
        title,
        properties,
    })
}

/// Execute a prepared export through one caller-owned chemistry engine.
pub fn export_prepared_document_molecule<E: ChemEngine + ?Sized>(
    engine: &E,
    prepared: PreparedDocumentMoleculeExport,
) -> Result<DocumentMoleculeExport, DocumentMoleculeExportError> {
    let PreparedDocumentMoleculeExport {
        source_revision,
        source_digest,
        molecule_id,
        payload,
    } = prepared;
    let (format, text) = match payload {
        PreparedDocumentMoleculeExportPayload::GraphOnly { format, graph } => {
            let text = match format {
                GraphOnlyExportFormat::CanonicalSmiles => {
                    engine.molecule_to_smiles(&graph, DOCUMENT_MOLECULE_EXPORT_TEXT_LIMIT)?
                }
                GraphOnlyExportFormat::Inchi(mode) => {
                    engine.molecule_to_inchi(&graph, mode, DOCUMENT_MOLECULE_EXPORT_TEXT_LIMIT)?
                }
            };
            (format.public_format(), text)
        }
        PreparedDocumentMoleculeExportPayload::CoordinateRequired {
            format,
            coordinate_graph,
            title,
            properties,
        } => {
            let text = match format {
                CoordinateRequiredExportFormat::Molfile(version) => {
                    export_molfile(engine, &coordinate_graph, &title, version)?
                }
                CoordinateRequiredExportFormat::Sdf(version) => {
                    export_sdf(engine, coordinate_graph, title, properties, version)?
                }
            };
            (format.public_format(), text)
        }
    };
    Ok(DocumentMoleculeExport {
        source_revision,
        source_digest,
        molecule_id,
        format,
        text,
    })
}

fn export_molfile<E: ChemEngine + ?Sized>(
    engine: &E,
    graph: &MolGraph,
    title: &str,
    version: MolblockVersion,
) -> Result<String, DocumentMoleculeExportError> {
    if title.is_empty() {
        Ok(engine.molecule_to_molblock(graph, version, DOCUMENT_MOLECULE_EXPORT_TEXT_LIMIT)?)
    } else {
        Ok(engine.molecule_to_molblock_with_title(
            graph,
            version,
            title,
            DOCUMENT_MOLECULE_EXPORT_TEXT_LIMIT,
        )?)
    }
}

fn export_sdf<E: ChemEngine + ?Sized>(
    engine: &E,
    graph: MolGraph,
    title: String,
    properties: Vec<SdfProperty>,
    version: MolblockVersion,
) -> Result<String, DocumentMoleculeExportError> {
    let record = SdfRecord::new(graph, title, properties)?;
    Ok(engine.records_to_sdf(&[record], version, DOCUMENT_MOLECULE_EXPORT_TEXT_LIMIT)?)
}

/// Typed refusal while preparing or emitting one selected export.
#[derive(Debug, Error)]
pub enum DocumentMoleculeExportError {
    #[error(transparent)]
    Observation(#[from] DocumentMoleculeInspectionErrorV1),
    #[error(transparent)]
    Metadata(#[from] InterchangeRecordMetadataErrorV1),
    #[error("selected molecule is not representable by this export: {0}")]
    UnsupportedMolecule(#[from] DocumentMoleculeGraphError),
    #[error(transparent)]
    Sdf(#[from] SdfError),
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}

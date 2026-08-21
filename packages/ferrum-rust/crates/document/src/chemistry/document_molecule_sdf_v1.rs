//! Exact-observation single-record SDF export for one direct molecule.

use crate::{
    DocumentObjectIdV1, InterchangeRecordMetadataErrorV1, SessionDocumentObservationV1,
    TypedDocument, observe_interchange_record_metadata_v1,
};
use ferrum_chemistry::{
    ChemEngine, ChemistryError, MolGraph, MolblockVersion, NativeChemEngine, SdfError, SdfProperty,
    compose_sdf_record, validate_molblock_title,
};
use thiserror::Error;

use super::document_molecule_graph_v1::{
    DocumentMoleculeGraphError, document_molecule_coordinate_graph_v1,
};
use super::document_molecule_inspection_v1::{
    DocumentMoleculeInspectionErrorV1, copied_object_id, direct_projection_molecule_v1,
    verify_molecule_observation_v1,
};

/// Stable schema for one exact document interchange record receipt.
pub const DOCUMENT_MOLECULE_SDF_SCHEMA_V1: &str = "ferrum-document-molecule-sdf-v1";
/// Coordinate and record-envelope profile used by selected SDF export.
pub const DOCUMENT_MOLECULE_SDF_PROFILE_V1: &str =
    "document-xy-to-chemistry-x-minus-y-rust-sdf-envelope-v1";

/// Immutable exact-observation request for one direct-root interchange record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeSdfRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    version: MolblockVersion,
}

impl DocumentMoleculeSdfRequestV1 {
    /// Construct one request from an installed direct-root address and syntax.
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
        version: MolblockVersion,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
            version,
        }
    }

    /// Return the revision which must still own the selected root.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }

    /// Return the digest which must still own the selected root.
    #[must_use]
    pub const fn expected_digest(&self) -> &[u8; 32] {
        &self.expected_digest
    }

    /// Return the opaque durable direct-root selector.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return the explicit Molfile syntax inside the interchange record.
    #[must_use]
    pub const fn version(&self) -> MolblockVersion {
        self.version
    }
}

/// Native-handle-free graph and exact record facts frozen before adapter loading.
#[derive(Debug)]
pub struct PreparedDocumentMoleculeSdfV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    version: MolblockVersion,
    title: String,
    properties: Vec<SdfProperty>,
    molecule: MolGraph,
}

impl PreparedDocumentMoleculeSdfV1 {
    /// Return the frozen source revision.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the frozen source digest.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return the exact durable direct-root selector.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return the explicit Molfile syntax frozen into this request.
    #[must_use]
    pub const fn version(&self) -> MolblockVersion {
        self.version
    }

    /// Return the exact effective record title, including empty text.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Return exact ordered properties, including repeated names.
    #[must_use]
    pub fn properties(&self) -> &[SdfProperty] {
        &self.properties
    }
}

/// One complete interchange record bound to its exact source observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeSdfV1 {
    schema: &'static str,
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    profile: &'static str,
    version: MolblockVersion,
    title: String,
    properties: Vec<SdfProperty>,
    sdf: String,
}

impl DocumentMoleculeSdfV1 {
    /// Return the stable receipt schema.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Return the frozen source revision.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the frozen source digest.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return the exact durable direct-root selector.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return the closed coordinate and record-envelope profile.
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.profile
    }

    /// Return the explicit Molfile syntax inside the interchange record.
    #[must_use]
    pub const fn version(&self) -> MolblockVersion {
        self.version
    }

    /// Return the exact effective record title, including empty text.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Return exact ordered properties, including repeated names.
    #[must_use]
    pub fn properties(&self) -> &[SdfProperty] {
        &self.properties
    }

    /// Return the exact completed interchange record text.
    #[must_use]
    pub fn sdf(&self) -> &str {
        &self.sdf
    }
}

/// Authenticate one direct root and freeze its graph and persisted SDF facts.
pub fn prepare_document_molecule_sdf_v1(
    observation: &SessionDocumentObservationV1,
    request: &DocumentMoleculeSdfRequestV1,
) -> Result<PreparedDocumentMoleculeSdfV1, DocumentMoleculeSdfErrorV1> {
    verify_molecule_observation_v1(
        observation,
        request.expected_revision,
        &request.expected_digest,
    )?;
    let snapshot = observation.snapshot();
    let root = direct_projection_molecule_v1(observation.projection(), &request.molecule_id)?;
    let root_source_id = root
        .source_id()
        .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
    let document = TypedDocument::parse(snapshot.cdml())
        .map_err(DocumentMoleculeInspectionErrorV1::Document)?;
    let molecule = document
        .core_molecule(&request.molecule_id)
        .map_err(DocumentMoleculeInspectionErrorV1::CoreProjection)?
        .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
    if molecule.source_id().map(ferrum_core::Identifier::as_str) != Some(root_source_id) {
        return Err(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch.into());
    }

    let metadata = observe_interchange_record_metadata_v1(&document, root_source_id)?;
    let (title, metadata_properties) = match metadata {
        Some(metadata) => metadata.into_parts(),
        None => (copy_text(molecule.name().unwrap_or(""))?, Vec::new()),
    };
    validate_molblock_title(&title)?;
    let mut properties = Vec::new();
    properties
        .try_reserve_exact(metadata_properties.len())
        .map_err(|_| DocumentMoleculeSdfErrorV1::ResourceAllocation)?;
    for property in metadata_properties {
        let (name, value) = property.into_parts();
        properties.push(SdfProperty::new(name, value)?);
    }
    let molecule = document_molecule_coordinate_graph_v1(&molecule)?
        .into_parts()
        .0;
    Ok(PreparedDocumentMoleculeSdfV1 {
        source_revision: snapshot.revision(),
        source_digest: *snapshot.digest(),
        molecule_id: copied_object_id(&request.molecule_id)?,
        version: request.version,
        title,
        properties,
        molecule,
    })
}

/// Execute one prepared graph through the packaged native Molfile writer.
pub fn export_prepared_document_molecule_sdf_v1(
    engine: &NativeChemEngine,
    prepared: PreparedDocumentMoleculeSdfV1,
) -> Result<DocumentMoleculeSdfV1, DocumentMoleculeSdfErrorV1> {
    export_with_engine(engine, prepared)
}

fn export_with_engine(
    engine: &impl ChemEngine,
    prepared: PreparedDocumentMoleculeSdfV1,
) -> Result<DocumentMoleculeSdfV1, DocumentMoleculeSdfErrorV1> {
    let molblock = if prepared.title.is_empty() {
        engine.molecule_to_molblock(&prepared.molecule, prepared.version)?
    } else {
        engine.molecule_to_molblock_with_title(
            &prepared.molecule,
            prepared.version,
            &prepared.title,
        )?
    };
    let first_line = molblock
        .split_once('\n')
        .map(|(line, _)| line)
        .ok_or(DocumentMoleculeSdfErrorV1::NativeTitleMismatch)?;
    if first_line != prepared.title {
        return Err(DocumentMoleculeSdfErrorV1::NativeTitleMismatch);
    }
    let sdf = compose_sdf_record(&molblock, &prepared.properties)?;
    Ok(DocumentMoleculeSdfV1 {
        schema: DOCUMENT_MOLECULE_SDF_SCHEMA_V1,
        source_revision: prepared.source_revision,
        source_digest: prepared.source_digest,
        molecule_id: prepared.molecule_id,
        profile: DOCUMENT_MOLECULE_SDF_PROFILE_V1,
        version: prepared.version,
        title: prepared.title,
        properties: prepared.properties,
        sdf,
    })
}

#[cfg(test)]
pub(crate) fn export_prepared_document_molecule_sdf_with_engine_v1(
    engine: &impl ChemEngine,
    prepared: PreparedDocumentMoleculeSdfV1,
) -> Result<DocumentMoleculeSdfV1, DocumentMoleculeSdfErrorV1> {
    export_with_engine(engine, prepared)
}

fn copy_text(value: &str) -> Result<String, DocumentMoleculeSdfErrorV1> {
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_| DocumentMoleculeSdfErrorV1::ResourceAllocation)?;
    copied.push_str(value);
    Ok(copied)
}

/// Failure while authenticating, converting, or exporting one exact interchange record.
#[derive(Debug, Error)]
pub enum DocumentMoleculeSdfErrorV1 {
    /// Observation provenance or direct-root identity was rejected.
    #[error(transparent)]
    Observation(#[from] DocumentMoleculeInspectionErrorV1),
    /// Authoritative persisted interchange metadata was absent from its closed grammar.
    #[error(transparent)]
    Metadata(#[from] InterchangeRecordMetadataErrorV1),
    /// One authenticated source fact could not be copied into the prepared request.
    #[error("document SDF export could not reserve source-fact storage")]
    ResourceAllocation,
    /// Retained graph facts cannot cross the exact native Molfile boundary.
    #[error("document molecule cannot cross the native SDF structure boundary: {0}")]
    UnsupportedMolecule(#[from] DocumentMoleculeGraphError),
    /// Persisted record facts or the exact Rust envelope are not representable.
    #[error(transparent)]
    Sdf(#[from] SdfError),
    /// The packaged writer was unavailable or rejected the graph.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// Native output did not retain the exact effective record title.
    #[error("native Molfile output changed the exact SDF title")]
    NativeTitleMismatch,
}

#[cfg(test)]
#[path = "document_molecule_sdf_v1_tests.rs"]
mod tests;

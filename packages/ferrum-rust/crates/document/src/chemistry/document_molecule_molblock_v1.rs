//! Exact-observation Molfile export for one durable direct-root molecule.

use crate::{DocumentObjectIdV1, SessionDocumentObservationV1, TypedDocument};
use ferrum_chemistry::{
    ChemEngine, ChemistryError, MolGraph, MolblockVersion, NativeChemEngine,
    validate_molblock_title,
};
use thiserror::Error;

use super::document_molecule_graph_v1::{
    DocumentMoleculeGraphError, document_molecule_coordinate_graph_v1,
};
use super::document_molecule_inspection_v1::{
    DocumentMoleculeInspectionErrorV1, copied_object_id, direct_projection_molecule_v1,
    verify_molecule_observation_v1,
};

/// Stable schema for one exact Molfile export receipt.
pub const DOCUMENT_MOLECULE_MOLBLOCK_SCHEMA_V1: &str = "ferrum-document-molecule-molblock-v1";
/// Coordinate profile used at the document-to-chemistry boundary.
pub const DOCUMENT_MOLECULE_MOLBLOCK_PROFILE_V1: &str = "document-xy-to-chemistry-x-minus-y-v1";

/// Immutable exact-observation request for one direct-root molecule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeMolblockRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    version: MolblockVersion,
}

impl DocumentMoleculeMolblockRequestV1 {
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

    /// Return the requested explicit Molfile syntax.
    #[must_use]
    pub const fn version(&self) -> MolblockVersion {
        self.version
    }
}

/// Native-handle-free graph and source facts frozen before adapter loading.
#[derive(Debug)]
pub struct PreparedDocumentMoleculeMolblockV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    version: MolblockVersion,
    title: Option<String>,
    molecule: MolGraph,
}

impl PreparedDocumentMoleculeMolblockV1 {
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

    /// Return the explicit syntax frozen into this request.
    #[must_use]
    pub const fn version(&self) -> MolblockVersion {
        self.version
    }

    /// Return the exact authored title, including deliberate empty text.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
}

/// One complete Molfile receipt bound to its source observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeMolblockV1 {
    schema: &'static str,
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    profile: &'static str,
    version: MolblockVersion,
    title: Option<String>,
    molblock: String,
}

impl DocumentMoleculeMolblockV1 {
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

    /// Return the closed document-coordinate conversion profile.
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.profile
    }

    /// Return the explicit Molfile syntax.
    #[must_use]
    pub const fn version(&self) -> MolblockVersion {
        self.version
    }

    /// Return the exact authored title carried into the native writer.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Return the exact native Molfile text.
    #[must_use]
    pub fn molblock(&self) -> &str {
        &self.molblock
    }
}

/// Authenticate one exact direct root and freeze its coordinate-bearing graph.
pub fn prepare_document_molecule_molblock_v1(
    observation: &SessionDocumentObservationV1,
    request: &DocumentMoleculeMolblockRequestV1,
) -> Result<PreparedDocumentMoleculeMolblockV1, DocumentMoleculeMolblockErrorV1> {
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
    if molecule.source_id().as_str() != root_source_id {
        return Err(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch.into());
    }
    let title = molecule.name().map(copy_title).transpose()?;
    if let Some(title) = title.as_deref() {
        validate_molblock_title(title)?;
    }
    let molecule = document_molecule_coordinate_graph_v1(&molecule)?
        .into_parts()
        .0;
    Ok(PreparedDocumentMoleculeMolblockV1 {
        source_revision: snapshot.revision(),
        source_digest: *snapshot.digest(),
        molecule_id: copied_object_id(&request.molecule_id)?,
        version: request.version,
        title,
        molecule,
    })
}

/// Execute one prepared graph through the packaged native Molfile writer.
pub fn export_prepared_document_molecule_molblock_v1(
    engine: &NativeChemEngine,
    prepared: PreparedDocumentMoleculeMolblockV1,
) -> Result<DocumentMoleculeMolblockV1, DocumentMoleculeMolblockErrorV1> {
    export_with_engine(engine, prepared)
}

fn export_with_engine(
    engine: &impl ChemEngine,
    prepared: PreparedDocumentMoleculeMolblockV1,
) -> Result<DocumentMoleculeMolblockV1, DocumentMoleculeMolblockErrorV1> {
    let molblock = match prepared.title.as_deref() {
        Some(title) => {
            engine.molecule_to_molblock_with_title(&prepared.molecule, prepared.version, title)?
        }
        None => engine.molecule_to_molblock(&prepared.molecule, prepared.version)?,
    };
    Ok(DocumentMoleculeMolblockV1 {
        schema: DOCUMENT_MOLECULE_MOLBLOCK_SCHEMA_V1,
        source_revision: prepared.source_revision,
        source_digest: prepared.source_digest,
        molecule_id: prepared.molecule_id,
        profile: DOCUMENT_MOLECULE_MOLBLOCK_PROFILE_V1,
        version: prepared.version,
        title: prepared.title,
        molblock,
    })
}

#[cfg(test)]
pub(crate) fn export_prepared_document_molecule_molblock_with_engine_v1(
    engine: &impl ChemEngine,
    prepared: PreparedDocumentMoleculeMolblockV1,
) -> Result<DocumentMoleculeMolblockV1, DocumentMoleculeMolblockErrorV1> {
    export_with_engine(engine, prepared)
}

/// Failure while authenticating, converting, or exporting one exact root.
#[derive(Debug, Error)]
pub enum DocumentMoleculeMolblockErrorV1 {
    /// Observation provenance or direct-root identity was rejected.
    #[error(transparent)]
    Observation(#[from] DocumentMoleculeInspectionErrorV1),
    /// One authenticated source fact could not be copied into the prepared request.
    #[error("document Molfile export could not reserve source-fact storage")]
    ResourceAllocation,
    /// Retained graph facts cannot cross the exact native Molfile boundary.
    #[error("document molecule cannot cross the native Molfile boundary: {0}")]
    UnsupportedMolecule(#[from] DocumentMoleculeGraphError),
    /// The packaged native writer was unavailable or rejected the graph.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}

fn copy_title(value: &str) -> Result<String, DocumentMoleculeMolblockErrorV1> {
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_| DocumentMoleculeMolblockErrorV1::ResourceAllocation)?;
    copied.push_str(value);
    Ok(copied)
}

#[cfg(test)]
#[path = "document_molecule_molblock_v1_tests.rs"]
mod tests;

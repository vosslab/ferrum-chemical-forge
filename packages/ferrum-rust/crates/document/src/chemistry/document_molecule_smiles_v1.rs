//! Exact-observation canonical SMILES export for one durable direct-root molecule.

use crate::{DocumentObjectIdV1, SessionDocumentObservationV1, TypedDocument};
use ferrum_chemistry::{ChemEngine, ChemistryError, MolGraph, NativeChemEngine};
use thiserror::Error;

use super::document_molecule_graph_v1::{DocumentMoleculeGraphError, document_molecule_graph_v1};
use super::document_molecule_inspection_v1::{
    DocumentMoleculeInspectionErrorV1, copied_object_id, direct_projection_molecule_v1,
    verify_molecule_observation_v1,
};

/// Stable schema for one exact canonical SMILES export receipt.
pub const DOCUMENT_MOLECULE_SMILES_SCHEMA_V1: &str = "ferrum-document-molecule-smiles-v1";
/// Fixed native writer profile owned by this V1 operation.
pub const DOCUMENT_MOLECULE_SMILES_PROFILE_V1: &str = "canonical-isomeric-v1";

/// Immutable exact-observation request for one direct-root molecule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeSmilesRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
}

impl DocumentMoleculeSmilesRequestV1 {
    /// Construct one request from an installed direct-root address.
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
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
}

/// Native-handle-free graph and source facts frozen before adapter loading.
#[derive(Debug)]
pub struct PreparedDocumentMoleculeSmilesV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    molecule: MolGraph,
}

impl PreparedDocumentMoleculeSmilesV1 {
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
}

/// One complete canonical SMILES receipt bound to its source observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeSmilesV1 {
    schema: &'static str,
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    profile: &'static str,
    smiles: String,
}

impl DocumentMoleculeSmilesV1 {
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

    /// Return the closed canonical writer profile.
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.profile
    }

    /// Return the canonical isomeric SMILES line.
    #[must_use]
    pub fn smiles(&self) -> &str {
        &self.smiles
    }

    /// Consume this receipt into owned binding-friendly source and output facts.
    #[must_use]
    pub fn into_parts(self) -> (u64, [u8; 32], DocumentObjectIdV1, String) {
        (
            self.source_revision,
            self.source_digest,
            self.molecule_id,
            self.smiles,
        )
    }
}

/// Authenticate one exact direct root and freeze its complete supported graph.
pub fn prepare_document_molecule_smiles_v1(
    observation: &SessionDocumentObservationV1,
    request: &DocumentMoleculeSmilesRequestV1,
) -> Result<PreparedDocumentMoleculeSmilesV1, DocumentMoleculeSmilesErrorV1> {
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
    let molecule = document_molecule_graph_v1(&molecule)?.into_parts().0;
    Ok(PreparedDocumentMoleculeSmilesV1 {
        source_revision: snapshot.revision(),
        source_digest: *snapshot.digest(),
        molecule_id: copied_object_id(&request.molecule_id)?,
        molecule,
    })
}

/// Execute one prepared graph through the packaged native writer profile.
pub fn export_prepared_document_molecule_smiles_v1(
    engine: &NativeChemEngine,
    prepared: PreparedDocumentMoleculeSmilesV1,
) -> Result<DocumentMoleculeSmilesV1, DocumentMoleculeSmilesErrorV1> {
    export_with_engine(engine, prepared)
}

fn export_with_engine(
    engine: &impl ChemEngine,
    prepared: PreparedDocumentMoleculeSmilesV1,
) -> Result<DocumentMoleculeSmilesV1, DocumentMoleculeSmilesErrorV1> {
    let smiles = engine.molecule_to_smiles(&prepared.molecule)?;
    Ok(DocumentMoleculeSmilesV1 {
        schema: DOCUMENT_MOLECULE_SMILES_SCHEMA_V1,
        source_revision: prepared.source_revision,
        source_digest: prepared.source_digest,
        molecule_id: prepared.molecule_id,
        profile: DOCUMENT_MOLECULE_SMILES_PROFILE_V1,
        smiles,
    })
}

#[cfg(test)]
pub(crate) fn export_prepared_document_molecule_smiles_with_engine_v1(
    engine: &impl ChemEngine,
    prepared: PreparedDocumentMoleculeSmilesV1,
) -> Result<DocumentMoleculeSmilesV1, DocumentMoleculeSmilesErrorV1> {
    export_with_engine(engine, prepared)
}

/// Failure while authenticating, converting, or exporting one exact root.
#[derive(Debug, Error)]
pub enum DocumentMoleculeSmilesErrorV1 {
    /// Observation provenance or direct-root identity was rejected.
    #[error(transparent)]
    Observation(#[from] DocumentMoleculeInspectionErrorV1),
    /// Retained graph facts cannot cross the exact native SMILES boundary.
    #[error("document molecule cannot cross the native SMILES boundary: {0}")]
    UnsupportedMolecule(#[from] DocumentMoleculeGraphError),
    /// The packaged native writer was unavailable or rejected the graph.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}

#[cfg(test)]
#[path = "document_molecule_smiles_v1_tests.rs"]
mod tests;

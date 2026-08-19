//! Exact-revision InChI export for one Rust-authoritative CDML molecule.

use crate::{
    CoreProjectionError, DocumentObjectIdV1, SessionDocumentObservationV1, TypedDocument,
    TypedDocumentError,
};
use ferrum_chemistry::{ChemEngine, ChemistryError, InchiMode, MolGraph};
use thiserror::Error;

use super::document_molecule_graph_v1::{DocumentMoleculeGraphError, document_molecule_graph_v1};
use super::document_molecule_inspection_v1::{copied_object_id, direct_projection_molecule_v1};

/// A validated, native-handle-free InChI export request.
///
/// Construction resolves the durable target and converts its complete supported
/// chemistry graph before an adapter is loaded. The retained provenance lets a
/// caller discard a result when its displayed document has advanced meanwhile.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedDocumentMoleculeInchiV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    mode: InchiMode,
    molecule: MolGraph,
}

impl PreparedDocumentMoleculeInchiV1 {
    /// Return the source document revision.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the source document digest.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return the durable source molecule selector.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return the selected closed InChI mode.
    #[must_use]
    pub const fn mode(&self) -> InchiMode {
        self.mode
    }
}

/// One immutable InChI bound to its exact source observation and closed mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeInchiV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    mode: InchiMode,
    inchi: String,
}

impl DocumentMoleculeInchiV1 {
    /// Return the source document revision.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the source document digest.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return the durable direct-root molecule selector.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return the explicit closed InChI mode.
    #[must_use]
    pub const fn mode(&self) -> InchiMode {
        self.mode
    }

    /// Return the bounded native InChI line.
    #[must_use]
    pub fn inchi(&self) -> &str {
        &self.inchi
    }
}

/// Validate one exact observation and freeze its selected chemistry graph.
pub fn prepare_document_molecule_inchi_v1(
    observation: &SessionDocumentObservationV1,
    molecule_id: &DocumentObjectIdV1,
    mode: InchiMode,
) -> Result<PreparedDocumentMoleculeInchiV1, DocumentMoleculeInchiError> {
    let root = match direct_projection_molecule_v1(observation.projection(), molecule_id) {
        Ok(root) => root,
        Err(_) => return Err(unknown_molecule(molecule_id)?),
    };
    let root_source_id = root
        .source_id()
        .ok_or(DocumentMoleculeInchiError::ProjectionRootMismatch)?;
    let document = TypedDocument::parse(observation.snapshot().cdml())?;
    let molecule = document
        .core_molecule(molecule_id)?
        .ok_or(DocumentMoleculeInchiError::ProjectionRootMismatch)?;
    if molecule.source_id().map(ferrum_core::Identifier::as_str) != Some(root_source_id) {
        return Err(DocumentMoleculeInchiError::ProjectionRootMismatch);
    }
    let (molecule, _edges) = document_molecule_graph_v1(&molecule)
        .map_err(DocumentMoleculeInchiError::UnsupportedMolecule)?
        .into_parts();
    Ok(PreparedDocumentMoleculeInchiV1 {
        source_revision: observation.snapshot().revision(),
        source_digest: *observation.snapshot().digest(),
        molecule_id: copied_object_id(molecule_id)
            .map_err(|_| DocumentMoleculeInchiError::ResourceAllocation)?,
        mode,
        molecule,
    })
}

/// Execute and consume one prepared request into an owned exact-source receipt.
pub fn export_prepared_document_molecule_inchi_receipt_v1<E: ChemEngine>(
    engine: &E,
    prepared: PreparedDocumentMoleculeInchiV1,
) -> Result<DocumentMoleculeInchiV1, DocumentMoleculeInchiError> {
    let inchi = engine.molecule_to_inchi(&prepared.molecule, prepared.mode)?;
    Ok(DocumentMoleculeInchiV1 {
        source_revision: prepared.source_revision,
        source_digest: prepared.source_digest,
        molecule_id: prepared.molecule_id,
        mode: prepared.mode,
        inchi,
    })
}

/// Execute one already validated request through a chemistry engine.
pub fn export_prepared_document_molecule_inchi_v1<E: ChemEngine>(
    engine: &E,
    prepared: &PreparedDocumentMoleculeInchiV1,
) -> Result<String, DocumentMoleculeInchiError> {
    engine
        .molecule_to_inchi(&prepared.molecule, prepared.mode)
        .map_err(Into::into)
}

/// Validate and export one molecule in a single Rust call.
pub fn export_document_molecule_inchi_v1<E: ChemEngine>(
    engine: &E,
    observation: &SessionDocumentObservationV1,
    molecule_id: &DocumentObjectIdV1,
    mode: InchiMode,
) -> Result<String, DocumentMoleculeInchiError> {
    let prepared = prepare_document_molecule_inchi_v1(observation, molecule_id, mode)?;
    export_prepared_document_molecule_inchi_v1(engine, &prepared)
}

/// Failure while validating or executing a document-molecule InChI export.
#[derive(Debug, Error)]
pub enum DocumentMoleculeInchiError {
    /// The immutable source snapshot could not be parsed.
    #[error(transparent)]
    Document(#[from] TypedDocumentError),
    /// Typed CDML facts could not form a core molecule.
    #[error(transparent)]
    CoreProjection(#[from] CoreProjectionError),
    /// The selector did not name one durable direct-root molecule.
    #[error("document object is not a durable molecule in this snapshot: {object_id}")]
    UnknownMolecule { object_id: String },
    /// Projection and typed-core facts disagreed for an authenticated direct root.
    #[error("document direct-root molecule projection does not match typed core facts")]
    ProjectionRootMismatch,
    /// Owned request or receipt facts could not be allocated.
    #[error("document InChI export could not reserve owned result storage")]
    ResourceAllocation,
    /// The molecule contains a fact the current native graph cannot preserve.
    #[error("document molecule cannot cross the native InChI boundary: {0}")]
    UnsupportedMolecule(#[source] DocumentMoleculeGraphError),
    /// The selected chemistry engine rejected the validated graph.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}

fn unknown_molecule(
    molecule_id: &DocumentObjectIdV1,
) -> Result<DocumentMoleculeInchiError, DocumentMoleculeInchiError> {
    let mut object_id = String::new();
    object_id
        .try_reserve_exact(molecule_id.as_str().len())
        .map_err(|_| DocumentMoleculeInchiError::ResourceAllocation)?;
    object_id.push_str(molecule_id.as_str());
    Ok(DocumentMoleculeInchiError::UnknownMolecule { object_id })
}

#[cfg(test)]
#[path = "document_molecule_inchi_v1_tests.rs"]
mod tests;

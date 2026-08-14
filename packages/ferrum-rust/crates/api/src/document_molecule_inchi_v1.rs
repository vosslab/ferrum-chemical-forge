//! Exact-revision InChI export for one Rust-authoritative CDML molecule.

use ferrum_chemistry::{ChemEngine, ChemistryError, InchiMode, MolGraph};
use ferrum_document::{
    CoreProjectionError, DocumentObjectIdV1, SessionDocumentObservationV1, TypedDocument,
    TypedDocumentError,
};
use thiserror::Error;

use crate::document_molecule_graph_v1::{DocumentMoleculeGraphError, document_molecule_graph_v1};

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

/// Validate one exact observation and freeze its selected chemistry graph.
pub fn prepare_document_molecule_inchi_v1(
    observation: &SessionDocumentObservationV1,
    molecule_id: &DocumentObjectIdV1,
    mode: InchiMode,
) -> Result<PreparedDocumentMoleculeInchiV1, DocumentMoleculeInchiError> {
    let document = TypedDocument::parse(observation.snapshot().cdml())?;
    let molecule = document.core_molecule(molecule_id)?.ok_or_else(|| {
        DocumentMoleculeInchiError::UnknownMolecule {
            object_id: molecule_id.as_str().to_owned(),
        }
    })?;
    let (molecule, _edges) = document_molecule_graph_v1(&molecule)
        .map_err(DocumentMoleculeInchiError::UnsupportedMolecule)?
        .into_parts();
    Ok(PreparedDocumentMoleculeInchiV1 {
        source_revision: observation.snapshot().revision(),
        source_digest: *observation.snapshot().digest(),
        molecule_id: molecule_id.clone(),
        mode,
        molecule,
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
    /// The molecule contains a fact the current native graph cannot preserve.
    #[error("document molecule cannot cross the native InChI boundary: {0}")]
    UnsupportedMolecule(#[source] DocumentMoleculeGraphError),
    /// The selected chemistry engine rejected the validated graph.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}

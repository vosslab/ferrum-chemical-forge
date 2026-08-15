//! Authenticated explicit-fragment creation and read-only observation.

use ferrum_document::{
    DocumentExplicitFragmentObservationV1, DocumentExplicitFragmentRecordV1, DocumentObjectIdV1,
    DocumentSession, DocumentSessionError, PersistentId, SessionDocumentObservationV1,
    SessionOperationResultV1, TypedDocument, observe_explicit_fragments_v1,
};
use thiserror::Error;

use crate::document_molecule_inspection_v1::{
    DocumentMoleculeInspectionErrorV1, direct_projection_molecule_v1,
    verify_molecule_observation_v1,
};

/// Stable schema for exact explicit-fragment observation facts.
pub const DOCUMENT_EXPLICIT_FRAGMENT_SCHEMA_V1: &str = "ferrum-document-explicit-fragment-v1";

/// Immutable creation intent bound to one installed direct molecule observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentExplicitFragmentRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    name: String,
    selected_atom_ids: Vec<PersistentId>,
    selected_bond_ids: Vec<PersistentId>,
}

impl DocumentExplicitFragmentRequestV1 {
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
        name: String,
        selected_atom_ids: Vec<PersistentId>,
        selected_bond_ids: Vec<PersistentId>,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
            name,
            selected_atom_ids,
            selected_bond_ids,
        }
    }
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
    #[must_use]
    pub const fn expected_digest(&self) -> &[u8; 32] {
        &self.expected_digest
    }
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
}

/// Immutable accepted creation facts plus the authoritative changed observation.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentExplicitFragmentCreateResultV1 {
    record: DocumentExplicitFragmentRecordV1,
    operation: SessionOperationResultV1,
}
impl DocumentExplicitFragmentCreateResultV1 {
    #[must_use]
    pub fn record(&self) -> &DocumentExplicitFragmentRecordV1 {
        &self.record
    }
    #[must_use]
    pub fn operation(&self) -> &SessionOperationResultV1 {
        &self.operation
    }
    #[must_use]
    pub fn into_operation(self) -> SessionOperationResultV1 {
        self.operation
    }
}

/// Exact read-only metadata facts for one frozen observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentExplicitFragmentObservationReceiptV1 {
    schema: &'static str,
    source_revision: u64,
    source_digest: [u8; 32],
    facts: DocumentExplicitFragmentObservationV1,
}
impl DocumentExplicitFragmentObservationReceiptV1 {
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }
    #[must_use]
    pub fn records(&self) -> &[DocumentExplicitFragmentRecordV1] {
        self.facts.records()
    }
    #[must_use]
    pub const fn has_retained_fragment_metadata(&self) -> bool {
        self.facts.has_retained_fragment_metadata()
    }
}

#[derive(Debug, Error)]
pub enum DocumentExplicitFragmentErrorV1 {
    #[error(transparent)]
    Observation(#[from] DocumentMoleculeInspectionErrorV1),
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
    #[error("explicit fragment observation could not reauthenticate retained CDML: {0}")]
    Document(#[from] ferrum_document::TypedDocumentError),
}

/// Create exactly one explicit fragment through the authoritative one-use session flow.
pub fn create_document_explicit_fragment_v1(
    session: &mut DocumentSession,
    request: DocumentExplicitFragmentRequestV1,
) -> Result<DocumentExplicitFragmentCreateResultV1, DocumentExplicitFragmentErrorV1> {
    let observation = session.observe(request.expected_revision)?;
    verify_molecule_observation_v1(
        &observation,
        request.expected_revision,
        &request.expected_digest,
    )?;
    direct_projection_molecule_v1(observation.projection(), &request.molecule_id)?;
    let mut pending = session.prepare_create_explicit_fragment_v1(
        request.expected_revision,
        &request.molecule_id,
        &request.name,
        &request.selected_atom_ids,
        &request.selected_bond_ids,
    )?;
    let record = pending.record().clone();
    let operation =
        session.commit_create_explicit_fragment_v1(request.expected_revision, &mut pending)?;
    Ok(DocumentExplicitFragmentCreateResultV1 { record, operation })
}

/// Observe exact V1 metadata only after revision/digest reauthentication.
pub fn inspect_document_explicit_fragments_v1(
    observation: &SessionDocumentObservationV1,
    expected_revision: u64,
    expected_digest: [u8; 32],
) -> Result<DocumentExplicitFragmentObservationReceiptV1, DocumentExplicitFragmentErrorV1> {
    verify_molecule_observation_v1(observation, expected_revision, &expected_digest)?;
    let document = TypedDocument::parse(observation.snapshot().cdml())?;
    Ok(DocumentExplicitFragmentObservationReceiptV1 {
        schema: DOCUMENT_EXPLICIT_FRAGMENT_SCHEMA_V1,
        source_revision: expected_revision,
        source_digest: expected_digest,
        facts: observe_explicit_fragments_v1(&document),
    })
}

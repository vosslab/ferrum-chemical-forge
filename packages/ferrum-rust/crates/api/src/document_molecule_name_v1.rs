//! Revision- and digest-bound direct-root molecule-name mutation.

use ferrum_document::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, SessionOperation,
    SessionOperationResultV1, SessionOperationV1,
};
use thiserror::Error;

use crate::document_molecule_inspection_v1::{
    DocumentMoleculeInspectionErrorV1, direct_projection_molecule_v1,
    verify_molecule_observation_v1,
};

/// Immutable exact intent for one direct-root authored-name replacement or clear.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeNameRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    name: String,
}

impl DocumentMoleculeNameRequestV1 {
    /// Construct one complete request from an installed direct-root address.
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
        name: String,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
            name,
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

    /// Return the exact requested name; an empty string means remove the attribute.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Failure to authenticate or commit one direct-root molecule-name request.
#[derive(Debug, Error)]
pub enum DocumentMoleculeNameErrorV1 {
    /// The supplied observation address did not authenticate one current direct root.
    #[error(transparent)]
    Observation(#[from] DocumentMoleculeInspectionErrorV1),
    /// The authoritative document session rejected the mutation.
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

/// Replace or clear one exact direct-root molecule name in the authoritative session.
pub fn set_document_molecule_name_v1(
    session: &mut DocumentSession,
    request: DocumentMoleculeNameRequestV1,
) -> Result<SessionOperationResultV1, DocumentMoleculeNameErrorV1> {
    let observation = session.observe(request.expected_revision)?;
    verify_molecule_observation_v1(
        &observation,
        request.expected_revision,
        &request.expected_digest,
    )?;
    direct_projection_molecule_v1(observation.projection(), &request.molecule_id)?;
    let name = (!request.name.is_empty()).then_some(request.name);
    let operation = SessionOperation::V1(SessionOperationV1::SetMoleculeName {
        molecule_id: request.molecule_id,
        name,
    });
    session
        .submit(request.expected_revision, operation)
        .map_err(Into::into)
}

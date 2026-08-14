//! Authenticated immediate native linear-form conversion.

use ferrum_document::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, PersistentId,
    PreparedLinearFormConvertResultV1, SessionOperationResultV1,
};
use thiserror::Error;

use crate::document_molecule_inspection_v1::{
    DocumentMoleculeInspectionErrorV1, direct_projection_molecule_v1,
    verify_molecule_observation_v1,
};

/// Immutable exact intent for one direct-root linear-form conversion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentLinearFormRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    selected_atom_ids: Vec<PersistentId>,
}

impl DocumentLinearFormRequestV1 {
    /// Construct an owned request from one installed observation and atom selection.
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
        selected_atom_ids: Vec<PersistentId>,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
            selected_atom_ids,
        }
    }

    /// Return the revision that must still own the selected direct root.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }

    /// Return the digest that must still own the selected direct root.
    #[must_use]
    pub const fn expected_digest(&self) -> &[u8; 32] {
        &self.expected_digest
    }

    /// Return the opaque durable direct-root selector.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return the exact ordered durable atom selection.
    #[must_use]
    pub fn selected_atom_ids(&self) -> &[PersistentId] {
        &self.selected_atom_ids
    }
}

/// The closed outcome of an immediate linear-form conversion.
#[derive(Debug)]
pub enum DocumentLinearFormResultV1 {
    /// The session committed one authoritative changed observation.
    Changed(SessionOperationResultV1),
    /// The current source already was canonical and no history entry was added.
    NoChange(SessionOperationResultV1),
}

impl DocumentLinearFormResultV1 {
    /// Return the authoritative observation returned by the session.
    #[must_use]
    pub fn operation_result(&self) -> &SessionOperationResultV1 {
        match self {
            Self::Changed(result) | Self::NoChange(result) => result,
        }
    }

    /// Consume this closed outcome and return its authoritative session result.
    #[must_use]
    pub fn into_operation_result(self) -> SessionOperationResultV1 {
        match self {
            Self::Changed(result) | Self::NoChange(result) => result,
        }
    }
}

/// Failure to authenticate, prepare, or commit a linear-form conversion.
#[derive(Debug, Error)]
pub enum DocumentLinearFormErrorV1 {
    /// The supplied observation address did not authenticate one current direct root.
    #[error(transparent)]
    Observation(#[from] DocumentMoleculeInspectionErrorV1),
    /// The authoritative session rejected selection, planning, preparation, or commit.
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

/// Convert one exact selected-atom path immediately in the authoritative session.
///
/// This observes exactly once, authenticates its revision, digest, and direct root,
/// then consumes the session's pending candidate immediately when one is needed.
pub fn convert_document_linear_form_v1(
    session: &mut DocumentSession,
    request: DocumentLinearFormRequestV1,
) -> Result<DocumentLinearFormResultV1, DocumentLinearFormErrorV1> {
    let observation = session.observe(request.expected_revision)?;
    verify_molecule_observation_v1(
        &observation,
        request.expected_revision,
        &request.expected_digest,
    )?;
    direct_projection_molecule_v1(observation.projection(), &request.molecule_id)?;

    match session.prepare_convert_linear_form_v1(
        request.expected_revision,
        &request.molecule_id,
        &request.selected_atom_ids,
    )? {
        PreparedLinearFormConvertResultV1::NoChange(result) => {
            Ok(DocumentLinearFormResultV1::NoChange(*result))
        }
        PreparedLinearFormConvertResultV1::Pending(mut pending) => session
            .commit_convert_linear_form_v1(request.expected_revision, &mut pending)
            .map(DocumentLinearFormResultV1::Changed)
            .map_err(Into::into),
    }
}

//! Revision-bound preparation and commit for one Rust-owned Wavy root.

use super::{
    DocumentSession, DocumentSessionError, PersistentId, Point3V1, ProvisionalToken, RevisionState,
    SessionDocumentObservationV1, SessionOperationError, SessionOperationResultV1, WavyInsertionV1,
};

/// A one-use, revision-bound prepared Wavy insertion.
pub struct PendingCreateWavy {
    revision: u64,
    token: ProvisionalToken,
    identifier: PersistentId,
    candidate: Option<RevisionState>,
}

impl std::fmt::Debug for PendingCreateWavy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateWavy")
            .field("revision", &self.revision)
            .field("identifier", &self.identifier)
            .field("is_resolved", &self.candidate.is_none())
            .finish()
    }
}

impl PendingCreateWavy {
    /// Return the durable ID that will be created if this candidate is committed.
    #[must_use]
    pub fn identifier(&self) -> &PersistentId {
        &self.identifier
    }
}

impl DocumentSession {
    /// Prepare one bounded Wavy insertion at the current revision.
    pub fn prepare_create_wavy_v1(
        &mut self,
        expected_revision: u64,
        start: Point3V1,
        end: Point3V1,
    ) -> Result<PendingCreateWavy, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let insertion = WavyInsertionV1::new(start, end)
            .map_err(|error| SessionOperationError::InvalidWavyInsertion(error.to_string()))?;
        let (identifier, generated_ids) = self
            .generated_ids
            .reserve_presentation(self.history.current().document().indexed())?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_wavy(&identifier, &insertion)
            .map_err(SessionOperationError::Candidate)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let candidate_snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        SessionDocumentObservationV1::from_state(candidate.document(), candidate_snapshot)
            .map_err(DocumentSessionError::Projection)?;
        let token = self
            .history
            .current_mut()
            .document_mut()
            .issue_provisional_token();
        self.generated_ids = generated_ids;
        Ok(PendingCreateWavy {
            revision: expected_revision,
            token,
            identifier,
            candidate: Some(candidate),
        })
    }

    /// Accept one prepared Wavy insertion exactly once.
    pub fn commit_create_wavy(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateWavy,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_prepared_candidate(
            expected_revision,
            pending.revision,
            &pending.token,
            &mut pending.candidate,
        )
    }
}

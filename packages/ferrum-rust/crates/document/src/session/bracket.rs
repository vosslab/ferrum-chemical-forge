//! Revision-bound preparation and commit for one Rust-owned bracket pair.

use super::{
    BracketInsertionV1, BracketStyleV1, DocumentSession, DocumentSessionError, PersistentId,
    ProvisionalToken, RevisionState, SessionDocumentObservationV1, SessionOperationError,
    SessionOperationResultV1,
};

/// A one-use, revision-bound prepared bracket-pair insertion.
pub struct PendingCreateBracket {
    revision: u64,
    token: ProvisionalToken,
    left_identifier: PersistentId,
    right_identifier: PersistentId,
    candidate: Option<RevisionState>,
}

impl std::fmt::Debug for PendingCreateBracket {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateBracket")
            .field("revision", &self.revision)
            .field("left_identifier", &self.left_identifier)
            .field("right_identifier", &self.right_identifier)
            .field("is_resolved", &self.candidate.is_none())
            .finish()
    }
}

impl PendingCreateBracket {
    /// Return the durable pair ID, which is the left member ID.
    #[must_use]
    pub fn pair_identifier(&self) -> &PersistentId {
        &self.left_identifier
    }

    /// Return the durable left member ID.
    #[must_use]
    pub fn left_identifier(&self) -> &PersistentId {
        &self.left_identifier
    }

    /// Return the durable right member ID.
    #[must_use]
    pub fn right_identifier(&self) -> &PersistentId {
        &self.right_identifier
    }
}

impl DocumentSession {
    /// Prepare one finite bracket pair at the current revision.
    pub fn prepare_create_bracket_v1(
        &mut self,
        expected_revision: u64,
        style: BracketStyleV1,
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
    ) -> Result<PendingCreateBracket, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let insertion = BracketInsertionV1::new(style, left, top, right, bottom)
            .map_err(|error| SessionOperationError::InvalidBracketInsertion(error.to_string()))?;
        let ([left_identifier, right_identifier], generated_ids) = self
            .generated_ids
            .reserve_presentations(self.history.current().document().indexed())?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_bracket(&left_identifier, &right_identifier, &insertion)
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
        Ok(PendingCreateBracket {
            revision: expected_revision,
            token,
            left_identifier,
            right_identifier,
            candidate: Some(candidate),
        })
    }

    /// Accept one prepared bracket-pair insertion exactly once.
    pub fn commit_create_bracket(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateBracket,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_prepared_candidate(
            expected_revision,
            pending.revision,
            &pending.token,
            &mut pending.candidate,
        )
    }
}

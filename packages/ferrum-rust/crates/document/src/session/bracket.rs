//! Revision-bound preparation and commit for one Rust-owned bracket pair.

use super::{
    BracketInsertionV1, BracketStyleV1, DocumentSession, DocumentSessionError, PersistentId,
    PreparedSessionTransitionV1, RevisionState, SessionOperationError, SessionOperationResultV1,
};

/// A one-use, revision-bound prepared bracket-pair insertion.
pub struct PendingCreateBracket {
    left_identifier: PersistentId,
    right_identifier: PersistentId,
    transition: PreparedSessionTransitionV1,
}

impl std::fmt::Debug for PendingCreateBracket {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateBracket")
            .field("left_identifier", &self.left_identifier)
            .field("right_identifier", &self.right_identifier)
            .field("is_resolved", &self.transition.is_consumed_v1())
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
        let ([left_identifier, right_identifier], effects) = self
            .reserve_generated_ids_for_transition_v1(|sequences, indexed| {
                sequences.reserve_presentations(indexed)
            })?;
        let candidate = self
            .current_state_v1()
            .document()
            .with_insert_bracket(&left_identifier, &right_identifier, &insertion)
            .map_err(SessionOperationError::Candidate)?;
        let revision = self
            .current_state_v1()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let transition = self.prepare_changed_session_transition_v1(
            expected_revision,
            self.current_digest_v1(),
            candidate,
            effects,
        )?;
        Ok(PendingCreateBracket {
            left_identifier,
            right_identifier,
            transition,
        })
    }

    /// Accept one prepared bracket-pair insertion exactly once.
    pub fn commit_create_bracket(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateBracket,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))
    }
}

fn map_transition_refusal(
    session: &DocumentSession,
    expected_revision: u64,
    refusal: super::AdmittedSessionTransitionRefusalV1,
) -> DocumentSessionError {
    match refusal {
        super::AdmittedSessionTransitionRefusalV1::ForeignSession => {
            DocumentSessionError::PreparedOperationForeignSession
        }
        super::AdmittedSessionTransitionRefusalV1::Consumed
        | super::AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            DocumentSessionError::PreparedOperationConsumed
        }
        super::AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            DocumentSessionError::RevisionConflict {
                expected: expected_revision,
                actual: session.current_revision_v1(),
            }
        }
        super::AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            DocumentSessionError::RendererAdmission
        }
    }
}

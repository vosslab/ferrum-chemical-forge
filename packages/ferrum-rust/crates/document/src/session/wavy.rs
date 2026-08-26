//! Revision-bound preparation and commit for one Rust-owned Wavy root.

use super::{
    DocumentSession, DocumentSessionError, PersistentId, Point3V1, PreparedSessionTransitionV1,
    RevisionState, SessionOperationError, SessionOperationResultV1, WavyInsertionV1,
};

/// A one-use, revision-bound prepared Wavy insertion.
pub struct PendingCreateWavy {
    identifier: PersistentId,
    transition: PreparedSessionTransitionV1,
}

impl std::fmt::Debug for PendingCreateWavy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateWavy")
            .field("identifier", &self.identifier)
            .field("is_resolved", &self.transition.is_consumed_v1())
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
        let (identifier, effects) =
            self.reserve_generated_ids_for_transition_v1(|sequences, indexed| {
                sequences.reserve_presentation(indexed)
            })?;
        let candidate = self
            .current_state_v1()
            .document()
            .with_insert_wavy(&identifier, &insertion)
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
        Ok(PendingCreateWavy {
            identifier,
            transition,
        })
    }

    /// Accept one prepared Wavy insertion exactly once.
    pub fn commit_create_wavy(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateWavy,
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

//! Authenticated clipboard Cut through the generic admitted transition boundary.

use crate::{DocumentClipboardCutErrorV1, DocumentClipboardCutPlanV1};

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentSession, DocumentSessionError, RevisionState,
    SessionOperationResultV1, SessionTransitionEffectsV1,
};

impl DocumentSession {
    /// Apply one exact immutable Cut plan as one renderer-admitted history transition.
    pub fn cut_document_clipboard_v1(
        &mut self,
        expected_revision: u64,
        expected_digest: &[u8; 32],
        plan: &DocumentClipboardCutPlanV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let current = self.current_state_v1();
        if current.digest() != expected_digest {
            return Err(DocumentClipboardCutErrorV1::DigestMismatch.into());
        }
        if plan.source_revision() != expected_revision || plan.source_digest() != current.digest() {
            return Err(DocumentClipboardCutErrorV1::PlanProvenanceMismatch.into());
        }
        let source_revision = current.revision();
        let source_digest = *current.digest();
        let revision = current
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate =
            super::super::clipboard_cut_v1::compose_document_clipboard_cut_candidate_v1(
                current.document(),
                plan,
            )?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let mut transition = self.prepare_changed_session_transition_v1(
            source_revision,
            source_digest,
            state,
            SessionTransitionEffectsV1::none(),
        )?;
        self.commit_session_operation_transition_v1(&mut transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))
    }
}

fn map_transition_refusal(
    session: &DocumentSession,
    expected_revision: u64,
    refusal: AdmittedSessionTransitionRefusalV1,
) -> DocumentSessionError {
    match refusal {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            DocumentSessionError::PreparedOperationForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Consumed
        | AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            DocumentSessionError::PreparedOperationConsumed
        }
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            DocumentSessionError::RevisionConflict {
                expected: expected_revision,
                actual: session.current_revision_v1(),
            }
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            DocumentSessionError::RendererAdmission
        }
    }
}

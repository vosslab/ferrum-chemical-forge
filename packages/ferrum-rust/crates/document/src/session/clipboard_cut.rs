//! Immediate authenticated transaction ownership for native clipboard Cut.

use crate::{DocumentClipboardCutErrorV1, DocumentClipboardCutPlanV1};

use super::{
    DocumentSession, DocumentSessionError, RevisionState, SessionDocumentObservationV1,
    SessionOperationError, SessionOperationResultV1,
};

impl DocumentSession {
    /// Apply one exact Cut plan as one authenticated history transition.
    pub fn cut_document_clipboard_v1(
        &mut self,
        expected_revision: u64,
        expected_digest: &[u8; 32],
        plan: &DocumentClipboardCutPlanV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let current = self.history.current();
        if current.digest() != expected_digest {
            return Err(DocumentClipboardCutErrorV1::DigestMismatch.into());
        }
        if plan.source_revision() != expected_revision || plan.source_digest() != current.digest() {
            return Err(DocumentClipboardCutErrorV1::PlanProvenanceMismatch.into());
        }
        let candidate =
            super::super::clipboard_cut_v1::compose_document_clipboard_cut_candidate_v1(
                current.document(),
                plan,
            )?;
        let revision = current
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        let observation = SessionDocumentObservationV1::from_state(candidate.document(), snapshot)
            .map_err(DocumentSessionError::Projection)?;
        self.history
            .try_reserve_append()
            .map_err(|_| SessionOperationError::HistoryResourceExhausted)?;
        self.history.append_reserved(candidate);
        Ok(SessionOperationResultV1::new(observation))
    }
}

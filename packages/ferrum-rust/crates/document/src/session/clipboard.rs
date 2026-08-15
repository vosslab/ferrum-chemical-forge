//! Immediate authenticated transaction ownership for native clipboard Paste.

use crate::{
    DocumentClipboardPasteErrorV1, DocumentClipboardPastePlanV1, DocumentClipboardPastedRootV1,
};

use super::{
    DocumentSession, DocumentSessionError, RevisionState, SessionDocumentObservationV1,
    SessionOperationError, SessionOperationResultV1,
};

/// Exact authoritative outcome of one accepted native Paste.
#[derive(Debug)]
pub struct DocumentClipboardPasteResultV1 {
    operation: SessionOperationResultV1,
    pasted_roots: Vec<DocumentClipboardPastedRootV1>,
}

impl DocumentClipboardPasteResultV1 {
    /// Return the complete post-Paste observation.
    #[must_use]
    pub fn operation_result(&self) -> &SessionOperationResultV1 {
        &self.operation
    }

    /// Consume the receipt and return its authoritative observation wrapper.
    #[must_use]
    pub fn into_operation_result(self) -> SessionOperationResultV1 {
        self.operation
    }

    /// Return inserted roots in exact fragment order.
    #[must_use]
    pub fn pasted_roots(&self) -> &[DocumentClipboardPastedRootV1] {
        &self.pasted_roots
    }
}

impl DocumentSession {
    /// Insert one worker-admitted fragment as one authenticated history transition.
    pub fn paste_document_clipboard_v1(
        &mut self,
        expected_revision: u64,
        expected_digest: &[u8; 32],
        plan: &DocumentClipboardPastePlanV1,
        dx: f64,
        dy: f64,
    ) -> Result<DocumentClipboardPasteResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let current = self.history.current();
        if current.digest() != expected_digest {
            return Err(DocumentClipboardPasteErrorV1::DigestMismatch.into());
        }
        let (generated, tentative_generated_ids) = self
            .generated_ids
            .reserve_fragment_import(current.document().indexed(), plan.declared_id_count())?;
        let (candidate, pasted_roots) =
            super::super::clipboard_paste_v1::compose_clipboard_paste_candidate_v1(
                current.document(),
                plan,
                &generated,
                dx,
                dy,
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
        self.generated_ids = tentative_generated_ids;
        Ok(DocumentClipboardPasteResultV1 {
            operation: SessionOperationResultV1::new(observation),
            pasted_roots,
        })
    }
}

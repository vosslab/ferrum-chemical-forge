//! Authenticated clipboard Paste through the generic admitted transition boundary.

use crate::{
    DocumentClipboardPasteErrorV1, DocumentClipboardPastePlanV1, DocumentClipboardPastedRootV1,
};

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentSession, DocumentSessionError, RevisionState,
    SessionOperationResultV1,
};

/// Exact authoritative outcome of one accepted native Paste.
#[derive(Debug)]
pub struct DocumentClipboardPasteResultV1 {
    operation: SessionOperationResultV1,
    pasted_roots: Vec<DocumentClipboardPastedRootV1>,
}

impl DocumentClipboardPasteResultV1 {
    #[must_use]
    pub fn operation_result(&self) -> &SessionOperationResultV1 {
        &self.operation
    }

    #[must_use]
    pub fn into_operation_result(self) -> SessionOperationResultV1 {
        self.operation
    }

    #[must_use]
    pub fn pasted_roots(&self) -> &[DocumentClipboardPastedRootV1] {
        &self.pasted_roots
    }
}

impl DocumentSession {
    /// Insert one immutable clipboard plan through renderer admission.
    pub fn paste_document_clipboard_v1(
        &mut self,
        expected_revision: u64,
        expected_digest: &[u8; 32],
        plan: &DocumentClipboardPastePlanV1,
        dx: f64,
        dy: f64,
    ) -> Result<DocumentClipboardPasteResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if self.current_state_v1().digest() != expected_digest {
            return Err(DocumentClipboardPasteErrorV1::DigestMismatch.into());
        }
        let (generated, effects, source_revision, source_digest, revision) = {
            let (generated, effects) =
                self.reserve_generated_ids_for_transition_v1(|ids, indexed| {
                    ids.reserve_fragment_import(indexed, plan.declared_id_count())
                })?;
            let current = self.current_state_v1();
            (
                generated,
                effects,
                current.revision(),
                *current.digest(),
                current.next_revision(),
            )
        };
        let revision = revision.ok_or(DocumentSessionError::RevisionExhausted)?;
        let (candidate, pasted_roots) =
            super::super::clipboard_paste_v1::compose_clipboard_paste_candidate_v1(
                self.current_state_v1().document(),
                plan,
                &generated,
                dx,
                dy,
            )?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let mut transition = self.prepare_changed_session_transition_v1(
            source_revision,
            source_digest,
            state,
            effects,
        )?;
        let operation = self
            .commit_session_operation_transition_v1(&mut transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))?;
        Ok(DocumentClipboardPasteResultV1 {
            operation,
            pasted_roots,
        })
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

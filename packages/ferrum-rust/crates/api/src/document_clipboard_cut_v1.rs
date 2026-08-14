//! Product-owned preparation and immediate transaction for native Cut.

use ferrum_document::{
    DocumentClipboardCutErrorV1, DocumentClipboardCutPlanV1, DocumentClipboardSelectionV1,
    DocumentSession, DocumentSessionError, SessionDocumentObservationV1, SessionOperationResultV1,
    prepare_document_clipboard_cut_v1,
};
use thiserror::Error;

/// Prepare one source-authenticated Cut plan without borrowing the mutable session.
pub fn prepare_clipboard_cut_v1(
    observation: &SessionDocumentObservationV1,
    selection: DocumentClipboardSelectionV1,
) -> Result<DocumentClipboardCutPlanV1, DocumentClipboardCutErrorV1> {
    prepare_document_clipboard_cut_v1(observation, selection)
}

/// Failure while applying one prepared Cut to an authenticated session state.
#[derive(Debug, Error)]
pub enum DocumentClipboardCutApplyErrorV1 {
    /// The session rejected authentication, resources, candidate, or commit.
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

/// Apply one prepared Cut as one Rust-owned document transition.
pub fn apply_clipboard_cut_v1(
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &[u8; 32],
    plan: &DocumentClipboardCutPlanV1,
) -> Result<SessionOperationResultV1, DocumentClipboardCutApplyErrorV1> {
    session
        .cut_document_clipboard_v1(expected_revision, expected_digest, plan)
        .map_err(Into::into)
}

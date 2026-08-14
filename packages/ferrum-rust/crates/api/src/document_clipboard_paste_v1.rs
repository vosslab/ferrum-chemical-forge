//! Product-owned bounded profile and immediate transaction for native Paste.

use ferrum_document::{
    DocumentClipboardPasteErrorV1, DocumentClipboardPastePlanV1, DocumentClipboardPasteResultV1,
    DocumentSession, DocumentSessionError, XmlInputBudgetV1, prepare_document_clipboard_paste_v1,
};
use thiserror::Error;

use crate::{DocumentIngressFormatV1, local_cdml_ingress_format_v1};

/// Stable product profile for clipboard fragments copied from admitted local CDML.
pub const DOCUMENT_CLIPBOARD_PASTE_PROFILE_V1: &str = "ferrum-document-clipboard-paste-profile-v1";

/// The single scene-space displacement applied to every root in one Paste.
pub const DOCUMENT_CLIPBOARD_PASTE_TRANSLATION_V1: (f64, f64) = (20.0, 20.0);

/// Return the exact XML envelope already used for ordinary admitted local CDML.
///
/// Native Copy guarantees its fragment is no larger than its admitted source.
/// Reusing that source profile avoids an arbitrary smaller clipboard-only cutoff;
/// the closed Paste root grammar supplies the additional semantic restriction.
#[must_use]
pub const fn document_clipboard_paste_budget_v1() -> XmlInputBudgetV1 {
    match local_cdml_ingress_format_v1() {
        DocumentIngressFormatV1::Cdml(budget) => budget.xml,
        DocumentIngressFormatV1::Cdsvg(_) => {
            unreachable!()
        }
    }
}

/// Prepare one external clipboard string without borrowing a document session.
pub fn prepare_clipboard_paste_v1(
    source: &str,
) -> Result<DocumentClipboardPastePlanV1, DocumentClipboardPasteErrorV1> {
    prepare_document_clipboard_paste_v1(source, document_clipboard_paste_budget_v1())
}

/// Failure while applying one prepared fragment to an authenticated session state.
#[derive(Debug, Error)]
pub enum DocumentClipboardPasteApplyErrorV1 {
    /// The session rejected authentication, resources, candidate, or commit.
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

/// Apply one prepared plan with the product V1 displacement as one Rust transaction.
pub fn apply_clipboard_paste_v1(
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &[u8; 32],
    plan: &DocumentClipboardPastePlanV1,
) -> Result<DocumentClipboardPasteResultV1, DocumentClipboardPasteApplyErrorV1> {
    let (dx, dy) = DOCUMENT_CLIPBOARD_PASTE_TRANSLATION_V1;
    session
        .paste_document_clipboard_v1(expected_revision, expected_digest, plan, dx, dy)
        .map_err(Into::into)
}

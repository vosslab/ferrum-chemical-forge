//! Product-owned bounded profile and immediate transaction for native user templates.

use ferrum_document::{
    DocumentSession, DocumentSessionError, DocumentUserTemplateErrorV1, DocumentUserTemplatePlanV1,
    DocumentUserTemplateResultV1, XmlInputBudgetV1, prepare_document_user_template_v1,
};
use ferrum_geometry::Point2;
use thiserror::Error;

use crate::{DocumentIngressFormatV1, local_cdml_ingress_format_v1};

/// Stable product profile for complete saved-template CDML documents.
pub const DOCUMENT_USER_TEMPLATE_PROFILE_V1: &str = "ferrum-document-user-template-profile-v1";

/// Return the ordinary local-CDML envelope used for saved user templates.
///
/// Templates are complete CDML documents with a stricter semantic root grammar.
/// Reusing the source profile avoids an arbitrary template-only byte threshold;
/// template admission adds the one-molecule and placement constraints.
#[must_use]
pub const fn document_user_template_budget_v1() -> XmlInputBudgetV1 {
    match local_cdml_ingress_format_v1() {
        DocumentIngressFormatV1::Cdml(budget) => budget.xml,
        DocumentIngressFormatV1::Cdsvg(_) => unreachable!(),
    }
}

/// Inspect one complete external template without borrowing a document session.
pub fn prepare_user_template_v1(
    source: &str,
) -> Result<DocumentUserTemplatePlanV1, DocumentUserTemplateErrorV1> {
    prepare_document_user_template_v1(source, document_user_template_budget_v1())
}

/// Failure while applying one prepared template to an authenticated session state.
#[derive(Debug, Error)]
pub enum DocumentUserTemplateApplyErrorV1 {
    /// The session rejected authentication, resources, candidate, or commit.
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

/// Place one admitted template at an exact finite scene anchor as one Rust transaction.
pub fn apply_user_template_v1(
    session: &mut DocumentSession,
    expected_revision: u64,
    expected_digest: &[u8; 32],
    plan: &DocumentUserTemplatePlanV1,
    anchor: Point2,
) -> Result<DocumentUserTemplateResultV1, DocumentUserTemplateApplyErrorV1> {
    session
        .insert_document_user_template_v1(expected_revision, expected_digest, plan, anchor)
        .map_err(Into::into)
}

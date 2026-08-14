//! Authoritative whole-document SVG composition for admitted sessions.

use ferrum_document::DocumentSession;
use ferrum_render::{
    DocumentRenderArtifactV1, SvgDocumentV1, SvgOutputBudgetV1, SvgRenderError,
    render_document_plan_to_svg_with_budget_v1,
};
use thiserror::Error;

use crate::{CompleteDocumentRenderPlanErrorV1, compose_complete_document_render_plan_v1};

/// Render one exact admitted session revision into a complete bounded SVG artifact.
///
/// A normal artifact is refused when the current renderer names any excluded
/// root. Callers therefore never publish a visually partial document without a
/// separate future exclusion-report contract.
pub fn render_document_session_to_svg_v1(
    session: &DocumentSession,
    expected_revision: u64,
    output_budget: SvgOutputBudgetV1,
) -> Result<DocumentRenderArtifactV1<SvgDocumentV1>, DocumentSvgArtifactErrorV1> {
    let plan = compose_complete_document_render_plan_v1(session, expected_revision)?;
    render_document_plan_to_svg_with_budget_v1(&plan, output_budget).map_err(Into::into)
}

/// Failure before a complete whole-document SVG can be published.
#[derive(Debug, Error)]
pub enum DocumentSvgArtifactErrorV1 {
    /// Observation, composition, or complete-root admission failed.
    #[error(transparent)]
    Plan(#[from] CompleteDocumentRenderPlanErrorV1),
    /// The native SVG sink rejected the complete plan or output policy.
    #[error(transparent)]
    Render(#[from] SvgRenderError),
}

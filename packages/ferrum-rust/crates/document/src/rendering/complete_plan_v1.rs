//! Shared admission of one complete whole-document render plan.

use crate::DocumentSession;
use crate::{DocumentRenderObservationErrorV1, observe_document_render_v1};
use ferrum_render::{DocumentRenderOutcomeV1, DocumentRenderPlanV1};
use ferrum_render::{DocumentRenderPlanCompositionError, compose_document_render_plan_v1};
use thiserror::Error;

/// Observe and compose one exact session revision only when every root can render.
///
/// Native artifact backends share this boundary so SVG, PDF, and PNG cannot drift
/// into different partial-document policies.
pub fn compose_complete_document_render_plan_v1(
    session: &DocumentSession,
    expected_revision: u64,
) -> Result<DocumentRenderPlanV1, CompleteDocumentRenderPlanErrorV1> {
    let observation = observe_document_render_v1(session, expected_revision)?;
    let plan = compose_document_render_plan_v1(observation.resolved())?;
    if plan
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(CompleteDocumentRenderPlanErrorV1::ExcludedRoots);
    }
    Ok(plan)
}

/// Failure before any native artifact backend receives a document plan.
#[derive(Debug, Error)]
pub enum CompleteDocumentRenderPlanErrorV1 {
    /// The requested revision could not produce an authoritative observation.
    #[error(transparent)]
    Observation(#[from] DocumentRenderObservationErrorV1),
    /// The observation could not form one authenticated whole-page plan.
    #[error(transparent)]
    Composition(#[from] DocumentRenderPlanCompositionError),
    /// One or more source roots would be absent from a normal complete artifact.
    #[error("the render plan excluded one or more document roots")]
    ExcludedRoots,
}

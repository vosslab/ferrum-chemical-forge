//! Authoritative whole-document PNG composition for admitted sessions.

use ferrum_document::DocumentSession;
use ferrum_render::{
    DocumentRenderArtifactV1, PngDocumentV1, PngRenderError, PngRenderRequestV1,
    render_document_plan_to_png_v1,
};
use thiserror::Error;

use crate::{CompleteDocumentRenderPlanErrorV1, compose_complete_document_render_plan_v1};

/// Render one exact admitted session revision through the native raster PNG sink.
pub fn render_document_session_to_png_v1(
    session: &DocumentSession,
    expected_revision: u64,
    request: PngRenderRequestV1,
) -> Result<DocumentRenderArtifactV1<PngDocumentV1>, DocumentPngArtifactErrorV1> {
    let plan = compose_complete_document_render_plan_v1(session, expected_revision)?;
    render_document_plan_to_png_v1(&plan, request).map_err(Into::into)
}

/// Failure before a complete whole-document PNG can be published.
#[derive(Debug, Error)]
pub enum DocumentPngArtifactErrorV1 {
    /// Observation, composition, or complete-root admission failed.
    #[error(transparent)]
    Plan(#[from] CompleteDocumentRenderPlanErrorV1),
    /// The native raster PNG sink rejected the complete plan or output policy.
    #[error(transparent)]
    Render(#[from] PngRenderError),
}

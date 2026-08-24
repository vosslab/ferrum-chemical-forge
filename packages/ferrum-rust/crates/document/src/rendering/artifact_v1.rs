//! Complete, session-owned native artifact convenience operations.

use crate::{
    CompleteDocumentRenderPlanErrorV1, DocumentSession, compose_complete_document_render_plan_v1,
};
use ferrum_render::{
    DocumentRenderArtifactV1, PdfDocumentV1, PdfRenderError, PdfRenderRequestV1, PngDocumentV1,
    PngRenderError, PngRenderRequestV1, SvgDocumentV1, SvgOutputBudgetV1, SvgRenderError,
    render_document_plan_to_pdf_v1, render_document_plan_to_png_v1,
    render_document_plan_to_svg_with_budget_v1,
};
use thiserror::Error;

/// Render one exact session revision into a complete bounded SVG artifact.
pub fn render_document_session_to_svg_v1(
    session: &DocumentSession,
    expected_revision: u64,
    output_budget: SvgOutputBudgetV1,
) -> Result<DocumentRenderArtifactV1<SvgDocumentV1>, DocumentSvgArtifactErrorV1> {
    let plan = compose_complete_document_render_plan_v1(session, expected_revision)?;
    render_document_plan_to_svg_with_budget_v1(&plan, output_budget).map_err(Into::into)
}

/// Render one exact session revision through the native raster PNG sink.
pub fn render_document_session_to_png_v1(
    session: &DocumentSession,
    expected_revision: u64,
    request: PngRenderRequestV1,
) -> Result<DocumentRenderArtifactV1<PngDocumentV1>, DocumentPngArtifactErrorV1> {
    let plan = compose_complete_document_render_plan_v1(session, expected_revision)?;
    render_document_plan_to_png_v1(&plan, request).map_err(Into::into)
}

/// Render one exact session revision through the native vector PDF sink.
pub fn render_document_session_to_pdf_v1(
    session: &DocumentSession,
    expected_revision: u64,
    request: PdfRenderRequestV1,
) -> Result<DocumentRenderArtifactV1<PdfDocumentV1>, DocumentPdfArtifactErrorV1> {
    let plan = compose_complete_document_render_plan_v1(session, expected_revision)?;
    render_document_plan_to_pdf_v1(&plan, request).map_err(Into::into)
}

/// Failure before a complete whole-document SVG can be published.
#[derive(Debug, Error)]
pub enum DocumentSvgArtifactErrorV1 {
    #[error(transparent)]
    Plan(#[from] CompleteDocumentRenderPlanErrorV1),
    #[error(transparent)]
    Render(#[from] SvgRenderError),
}

/// Failure before a complete whole-document PNG can be published.
#[derive(Debug, Error)]
pub enum DocumentPngArtifactErrorV1 {
    #[error(transparent)]
    Plan(#[from] CompleteDocumentRenderPlanErrorV1),
    #[error(transparent)]
    Render(#[from] PngRenderError),
}

/// Failure before a complete whole-document PDF can be published.
#[derive(Debug, Error)]
pub enum DocumentPdfArtifactErrorV1 {
    #[error(transparent)]
    Plan(#[from] CompleteDocumentRenderPlanErrorV1),
    #[error(transparent)]
    Render(#[from] PdfRenderError),
}

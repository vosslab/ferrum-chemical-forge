use std::num::NonZeroU32;

use ferrum_document::DocumentSession;
use ferrum_render::SvgOutputBudgetV1;
use ferrum_render::{PngBackgroundV1, PngPixelSizeV1, PngRenderError, PngRenderRequestV1, Rgb24};

use crate::{
    CompleteDocumentRenderPlanErrorV1, DocumentPdfArtifactErrorV1, DocumentPngArtifactErrorV1,
    DocumentSvgArtifactErrorV1, local_pdf_render_request_v1, local_png_render_request_v1,
    render_document_session_to_pdf_v1, render_document_session_to_png_v1,
    render_document_session_to_svg_v1,
};

const CDML: &str = concat!(
    "<cdml><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom>",
    "</molecule></cdml>",
);

fn png_request(raw_bytes: usize) -> PngRenderRequestV1 {
    local_png_render_request_v1(
        PngPixelSizeV1::new(
            NonZeroU32::new(16).expect("nonzero width"),
            NonZeroU32::new(12).expect("nonzero height"),
        ),
        PngBackgroundV1::Opaque(Rgb24::new("ffffff").expect("white")),
        raw_bytes,
        64 * 1024,
    )
}

#[test]
fn native_pdf_and_png_share_one_complete_plan_report_without_mutation() {
    let session = DocumentSession::load(CDML).expect("session");
    let before = session.snapshot().expect("before snapshot");
    let pdf = render_document_session_to_pdf_v1(
        &session,
        0,
        local_pdf_render_request_v1(64 * 1024, 1024, 64 * 1024).expect("PDF request"),
    )
    .expect("native PDF");
    let png = render_document_session_to_png_v1(&session, 0, png_request(16 * 12 * 4))
        .expect("native PNG");

    assert!(pdf.artifact().as_bytes().starts_with(b"%PDF-"));
    assert!(png.artifact().as_bytes().starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(pdf.report(), png.report());
    assert!(pdf.report().exclusions().is_empty());
    assert_eq!(session.snapshot().expect("after snapshot"), before);
}

#[test]
fn every_native_artifact_refuses_an_excluded_root_before_backend_selection() {
    let session = DocumentSession::load(
        "<cdml><arrow id=\"bad\" type=\"normal\"><point x=\"0\" y=\"0\"/></arrow></cdml>",
    )
    .expect("session");

    let pdf_error = render_document_session_to_pdf_v1(
        &session,
        0,
        local_pdf_render_request_v1(64 * 1024, 1024, 64 * 1024).expect("PDF request"),
    )
    .expect_err("incomplete plan must not reach PDF");
    assert!(matches!(
        pdf_error,
        DocumentPdfArtifactErrorV1::Plan(CompleteDocumentRenderPlanErrorV1::ExcludedRoots)
    ));

    let png_error = render_document_session_to_png_v1(&session, 0, png_request(16 * 12 * 4))
        .expect_err("incomplete plan must not reach PNG");
    assert!(matches!(
        png_error,
        DocumentPngArtifactErrorV1::Plan(CompleteDocumentRenderPlanErrorV1::ExcludedRoots)
    ));

    let svg_error = render_document_session_to_svg_v1(
        &session,
        0,
        SvgOutputBudgetV1::new(64 * 1024).expect("SVG budget"),
    )
    .expect_err("incomplete plan must not reach SVG");
    assert!(matches!(
        svg_error,
        DocumentSvgArtifactErrorV1::Plan(CompleteDocumentRenderPlanErrorV1::ExcludedRoots)
    ));
}

#[test]
fn png_raw_allocation_refusal_preserves_the_session() {
    let session = DocumentSession::load(CDML).expect("session");
    let before = session.snapshot().expect("before snapshot");

    let error = render_document_session_to_png_v1(&session, 0, png_request((16 * 12 * 4) - 1))
        .expect_err("raw cap must reject before pixmap allocation");
    assert!(matches!(
        error,
        DocumentPngArtifactErrorV1::Render(PngRenderError::RasterAllocationLimit { .. })
    ));
    assert_eq!(session.snapshot().expect("after snapshot"), before);
}

//! Thin opaque adapters for renderer-preflighted multi-point path authoring.

use ferrum_document::{
    DocumentFenceV1, DocumentSession, PresentationGesturePoint2V1, PresentationPathKindV1,
    SessionOperationTransitionRequestV1,
};
use ferrum_document_render::{PresentationPathOverlayV1, PresentationPathRenderGestureV1};
pub use ferrum_document_render::{
    PresentationPathProgressV1, PresentationPathRenderCategoryV1, PresentationPathRenderErrorV1,
    PresentationPathRenderRecoveryV1,
};

#[derive(Debug)]
pub struct ApiPresentationPathGestureV1(PresentationPathRenderGestureV1);
#[derive(Clone, Debug)]
pub struct ApiPresentationPathOverlayV1(PresentationPathOverlayV1);

impl ApiPresentationPathOverlayV1 {
    #[must_use]
    pub fn presentation(&self) -> &PresentationPathOverlayV1 {
        &self.0
    }
}

pub fn begin_api_presentation_path_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    kind: PresentationPathKindV1,
) -> Result<ApiPresentationPathGestureV1, PresentationPathRenderErrorV1> {
    ferrum_document_render::begin_presentation_path_gesture_v1(session, fence, kind)
        .map(ApiPresentationPathGestureV1)
}
/// Add one accepted scene point to the opaque incremental candidate.
pub fn add_api_presentation_path_gesture_point_v1(
    session: &DocumentSession,
    gesture: &mut ApiPresentationPathGestureV1,
    point: PresentationGesturePoint2V1,
) -> Result<PresentationPathProgressV1, PresentationPathRenderErrorV1> {
    ferrum_document_render::add_presentation_path_gesture_point_v1(session, &mut gesture.0, point)
}

/// Return the immutable Rust-issued overlay for an optional hover point.
pub fn preview_incremental_api_presentation_path_gesture_v1(
    session: &DocumentSession,
    gesture: &ApiPresentationPathGestureV1,
    hover: Option<PresentationGesturePoint2V1>,
) -> Result<ApiPresentationPathOverlayV1, PresentationPathRenderErrorV1> {
    ferrum_document_render::preview_incremental_presentation_path_gesture_v1(
        session, &gesture.0, hover,
    )
    .map(ApiPresentationPathOverlayV1)
}
/// Resolve a complete Rust-issued incremental overlay into the canonical transition request.
pub fn resolve_incremental_api_presentation_path_gesture_v1(
    session: &DocumentSession,
    gesture: ApiPresentationPathGestureV1,
    overlay: ApiPresentationPathOverlayV1,
) -> Result<SessionOperationTransitionRequestV1, PresentationPathRenderErrorV1> {
    ferrum_document_render::resolve_incremental_presentation_path_gesture_v1(
        session, gesture.0, overlay.0,
    )
}

/// Cancel an opaque candidate before any document mutation.
pub fn cancel_api_presentation_path_gesture_v1(
    session: &DocumentSession,
    gesture: &ApiPresentationPathGestureV1,
) -> Result<(), PresentationPathRenderErrorV1> {
    ferrum_document_render::cancel_presentation_path_gesture_v1(session, &gesture.0)
}

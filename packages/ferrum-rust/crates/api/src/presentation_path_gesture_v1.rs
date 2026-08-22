//! Thin opaque adapters for renderer-preflighted multi-point path authoring.

use ferrum_document::{DocumentFenceV1, DocumentSession, PresentationGesturePoint2V1, PresentationPathKindV1};
pub use ferrum_document_render::{
    CommittedPresentationPathV1, PresentationPathRenderCategoryV1, PresentationPathRenderErrorV1,
    PresentationPathRenderRecoveryV1,
};
use ferrum_document_render::{
    PresentationPathPreviewV1, PresentationPathRenderGestureV1, PreparedPresentationPathV1,
};

#[derive(Clone, Debug)]
pub struct ApiPresentationPathGestureV1(PresentationPathRenderGestureV1);
#[derive(Clone, Debug)]
pub struct ApiPresentationPathPreviewV1(PresentationPathPreviewV1);
#[derive(Debug)]
pub struct ApiPresentationPathPreparedV1(PreparedPresentationPathV1);

impl ApiPresentationPathPreviewV1 {
    #[must_use]
    pub fn path(&self) -> &ferrum_document::PresentationPathGestureV1 { self.0.path() }
    #[must_use]
    pub fn appearance(&self) -> &ferrum_document_render::PresentationPathAppearanceV1 { self.0.appearance() }
}

pub fn begin_api_presentation_path_gesture_v1(session: &DocumentSession, fence: DocumentFenceV1, kind: PresentationPathKindV1) -> Result<ApiPresentationPathGestureV1, PresentationPathRenderErrorV1> {
    ferrum_document_render::begin_presentation_path_gesture_v1(session, fence, kind).map(ApiPresentationPathGestureV1)
}
pub fn preview_api_presentation_path_gesture_v1(session: &DocumentSession, gesture: &ApiPresentationPathGestureV1, points: Vec<PresentationGesturePoint2V1>) -> Result<ApiPresentationPathPreviewV1, PresentationPathRenderErrorV1> {
    ferrum_document_render::preview_presentation_path_gesture_v1(session, &gesture.0, points).map(ApiPresentationPathPreviewV1)
}
pub fn prepare_api_presentation_path_gesture_v1(session: &mut DocumentSession, gesture: &ApiPresentationPathGestureV1, preview: &ApiPresentationPathPreviewV1) -> Result<ApiPresentationPathPreparedV1, PresentationPathRenderErrorV1> {
    ferrum_document_render::prepare_presentation_path_gesture_v1(session, &gesture.0, &preview.0).map(ApiPresentationPathPreparedV1)
}
pub fn commit_api_presentation_path_gesture_v1(session: &mut DocumentSession, prepared: &mut ApiPresentationPathPreparedV1) -> Result<CommittedPresentationPathV1, PresentationPathRenderErrorV1> {
    ferrum_document_render::commit_presentation_path_gesture_v1(session, &mut prepared.0)
}

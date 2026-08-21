//! Thin safe vector-gesture adapters for request-owned API callers.
//!
//! The renderer bridge owns complete renderer admission. This module carries
//! no candidate state, preflight receipt, nonce, or commit authority of its own.

use ferrum_document::{DocumentFenceV1, DocumentSession, PresentationGesturePoint2V1};
pub use ferrum_document_render::{
    CommittedPresentationVectorV1, PresentationVectorGestureCategoryV1,
    PresentationVectorGestureErrorV1, PresentationVectorGestureRecoveryV1,
    PresentationVectorKindV1, PresentationVectorOverlayV1,
};
use ferrum_document_render::{
    PreparedPresentationVectorV1, PresentationVectorGestureV1, PresentationVectorPreviewV1,
};

#[derive(Clone, Debug)]
pub struct ApiPresentationVectorGestureV1(PresentationVectorGestureV1);

#[derive(Clone, Debug)]
pub struct ApiPresentationVectorPreviewV1(PresentationVectorPreviewV1);

#[derive(Debug)]
pub struct ApiPresentationVectorPreparedV1(PreparedPresentationVectorV1);

impl ApiPresentationVectorPreviewV1 {
    #[must_use]
    pub fn overlay(&self) -> &PresentationVectorOverlayV1 {
        self.0.overlay()
    }
}

pub fn begin_api_presentation_vector_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    kind: PresentationVectorKindV1,
    start: PresentationGesturePoint2V1,
) -> Result<ApiPresentationVectorGestureV1, PresentationVectorGestureErrorV1> {
    ferrum_document_render::begin_presentation_vector_gesture_v1(session, fence, kind, start)
        .map(ApiPresentationVectorGestureV1)
}

pub fn preview_api_presentation_vector_gesture_v1(
    session: &DocumentSession,
    gesture: &ApiPresentationVectorGestureV1,
    end: PresentationGesturePoint2V1,
) -> Result<ApiPresentationVectorPreviewV1, PresentationVectorGestureErrorV1> {
    ferrum_document_render::preview_presentation_vector_gesture_v1(session, &gesture.0, end)
        .map(ApiPresentationVectorPreviewV1)
}

pub fn prepare_api_presentation_vector_gesture_v1(
    session: &mut DocumentSession,
    gesture: &ApiPresentationVectorGestureV1,
    preview: &ApiPresentationVectorPreviewV1,
) -> Result<ApiPresentationVectorPreparedV1, PresentationVectorGestureErrorV1> {
    ferrum_document_render::prepare_presentation_vector_gesture_v1(session, &gesture.0, &preview.0)
        .map(ApiPresentationVectorPreparedV1)
}

pub fn commit_api_presentation_vector_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut ApiPresentationVectorPreparedV1,
) -> Result<CommittedPresentationVectorV1, PresentationVectorGestureErrorV1> {
    ferrum_document_render::commit_presentation_vector_gesture_v1(session, &mut prepared.0)
}

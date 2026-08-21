//! Renderer-backed opaque direct Plus placement.
//!
//! The document crate owns the canonical one-root transaction. This API facade
//! owns the companion preview and publishes only verified renderer output.

use ferrum_document::{
    CommittedPresentationGestureV1, DocumentFenceV1, DocumentSession,
    PresentationCreationGestureV1, PresentationCreationPreviewV1, PresentationGestureErrorV1,
    PresentationGestureKindV1, PresentationGesturePoint2V1, PresentationGestureSnapPolicyV1,
    PresentationGestureStyleV1,
};
use ferrum_render::document_observation_from_accepted_operation_v1;

#[derive(Clone, Debug)]
pub struct ApiPlusGestureV1 {
    document: PresentationCreationGestureV1,
    start: PresentationGesturePoint2V1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ApiPlusOverlayV1 {
    origin_x: f64,
    origin_y: f64,
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
    text: String,
    font_size: f64,
    color: String,
    background: Option<String>,
}

impl ApiPlusOverlayV1 {
    #[must_use]
    pub const fn origin_x(&self) -> f64 {
        self.origin_x
    }
    #[must_use]
    pub const fn origin_y(&self) -> f64 {
        self.origin_y
    }
    #[must_use]
    pub const fn left(&self) -> f64 {
        self.left
    }
    #[must_use]
    pub const fn top(&self) -> f64 {
        self.top
    }
    #[must_use]
    pub const fn right(&self) -> f64 {
        self.right
    }
    #[must_use]
    pub const fn bottom(&self) -> f64 {
        self.bottom
    }
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
    #[must_use]
    pub const fn font_size(&self) -> f64 {
        self.font_size
    }
    #[must_use]
    pub fn color(&self) -> &str {
        &self.color
    }
    #[must_use]
    pub fn background(&self) -> Option<&str> {
        self.background.as_deref()
    }
}

#[derive(Clone, Debug)]
pub struct ApiPlusPreviewV1 {
    document: PresentationCreationPreviewV1,
    overlay: ApiPlusOverlayV1,
}
impl ApiPlusPreviewV1 {
    #[must_use]
    pub fn overlay(&self) -> &ApiPlusOverlayV1 {
        &self.overlay
    }
}

pub fn begin_api_plus_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
) -> Result<ApiPlusGestureV1, PresentationGestureErrorV1> {
    let document = session.begin_presentation_creation_gesture_v1(
        fence,
        PresentationGestureKindV1::Plus,
        start,
        PresentationGestureStyleV1::Plus,
        PresentationGestureSnapPolicyV1::free(),
    )?;
    Ok(ApiPlusGestureV1 { document, start })
}

pub fn preview_api_plus_gesture_v1(
    session: &DocumentSession,
    gesture: &ApiPlusGestureV1,
) -> Result<ApiPlusPreviewV1, PresentationGestureErrorV1> {
    // Validate the authoritative live transaction before any detached work.
    // A Plus document preview has no geometry or appearance to publish.
    let document =
        session.preview_presentation_creation_gesture_v1(&gesture.document, gesture.start)?;
    let snapshot = session
        .snapshot()
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
    let mut detached = DocumentSession::load(snapshot.cdml())
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
    let detached_snapshot = detached
        .snapshot()
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
    let detached_gesture = detached.begin_presentation_creation_gesture_v1(
        DocumentFenceV1::new(detached_snapshot.revision(), *detached_snapshot.digest()),
        PresentationGestureKindV1::Plus,
        gesture.start,
        PresentationGestureStyleV1::Plus,
        PresentationGestureSnapPolicyV1::free(),
    )?;
    let detached_preview =
        detached.preview_presentation_creation_gesture_v1(&detached_gesture, gesture.start)?;
    let committed =
        detached.commit_presentation_creation_gesture_v1(&detached_gesture, &detached_preview)?;
    let observation =
        document_observation_from_accepted_operation_v1(committed.result().observation())
            .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
    let plus = observation
        .plus_renders()
        .last()
        .ok_or(PresentationGestureErrorV1::SessionConflict)?;
    let operation = plus.operation();
    let bounds = plus.bounds();
    let anchor = plus.anchor();
    Ok(ApiPlusPreviewV1 {
        document,
        overlay: ApiPlusOverlayV1 {
            origin_x: operation.origin().x(),
            origin_y: operation.origin().y(),
            left: anchor.x() + bounds.left(),
            top: anchor.y() + bounds.top(),
            right: anchor.x() + bounds.right(),
            bottom: anchor.y() + bounds.bottom(),
            text: operation.runs().iter().map(|run| run.text()).collect(),
            font_size: operation.size().get(),
            color: operation.paint().color().as_str().to_owned(),
            background: plus
                .background()
                .map(|paint| paint.color().as_str().to_owned()),
        },
    })
}

pub fn commit_api_plus_gesture_v1(
    session: &mut DocumentSession,
    gesture: &ApiPlusGestureV1,
    preview: &ApiPlusPreviewV1,
) -> Result<CommittedPresentationGestureV1, PresentationGestureErrorV1> {
    session.commit_presentation_creation_gesture_v1(&gesture.document, &preview.document)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renderer_issued_preview_equals_the_committed_standard_plus() {
        let mut session =
            DocumentSession::load("<cdml><standard font_size='18' line_color='#123456'/></cdml>")
                .expect("fixture");
        let snapshot = session.snapshot().expect("snapshot");
        let gesture = begin_api_plus_gesture_v1(
            &session,
            DocumentFenceV1::new(snapshot.revision(), *snapshot.digest()),
            PresentationGesturePoint2V1::new(72.0, 36.0).expect("point"),
        )
        .expect("gesture");
        let preview = preview_api_plus_gesture_v1(&session, &gesture).expect("preview");
        let committed =
            commit_api_plus_gesture_v1(&mut session, &gesture, &preview).expect("commit");
        let observation =
            document_observation_from_accepted_operation_v1(committed.result().observation())
                .expect("render");
        let rendered = observation.plus_renders().last().expect("Plus");
        let bounds = rendered.bounds();
        let anchor = rendered.anchor();
        assert_eq!(
            preview.overlay().color(),
            rendered.operation().paint().color().as_str()
        );
        assert_eq!(
            preview.overlay().font_size(),
            rendered.operation().size().get()
        );
        assert_eq!(preview.overlay().left(), anchor.x() + bounds.left());
        assert_eq!(preview.overlay().bottom(), anchor.y() + bounds.bottom());
    }

    #[test]
    fn generic_plus_persists_without_document_preview_geometry() {
        let session = DocumentSession::load("<cdml/>").expect("fixture");
        let snapshot = session.snapshot().expect("snapshot");
        let gesture = session
            .begin_presentation_creation_gesture_v1(
                DocumentFenceV1::new(snapshot.revision(), *snapshot.digest()),
                ferrum_document::PresentationGestureKindV1::Plus,
                PresentationGesturePoint2V1::new(1.0, 1.0).expect("point"),
                ferrum_document::PresentationGestureStyleV1::normal(false, false),
                ferrum_document::PresentationGestureSnapPolicyV1::free(),
            )
            .expect("generic Plus transaction");
        let preview = session
            .preview_presentation_creation_gesture_v1(
                &gesture,
                PresentationGesturePoint2V1::new(1.0, 1.0).expect("point"),
            )
            .expect("opaque Plus preview");
        assert!(preview.overlay().is_none());
    }
}

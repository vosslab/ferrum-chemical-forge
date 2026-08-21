//! API-owned text placement facade.  It intentionally transports only opaque
//! document transactions; rendering remains sourced from accepted observations.

use ferrum_document::{
    CommittedTextPlacementV1, DocumentFenceV1, DocumentSession, PresentationGesturePoint2V1,
    TextPlacementContentV1, TextPlacementErrorV1, TextPlacementGestureV1, TextPlacementPreviewV1,
};
use ferrum_render::{DocumentTextRenderV1, document_observation_from_accepted_operation_v1};

#[derive(Clone, Debug)]
pub struct ApiTextPlacementGestureV1 {
    document: TextPlacementGestureV1,
}
#[derive(Clone, Debug)]
pub struct ApiTextPlacementPreviewV1 {
    document: TextPlacementPreviewV1,
    overlay: DocumentTextRenderV1,
}
#[derive(Clone, Debug)]
pub struct ApiTextPlacementDefaultsV1 {
    runs: Vec<ferrum_document::AuthoredTextRunV1>,
    font_size: f64,
    color: String,
}
impl ApiTextPlacementPreviewV1 {
    #[must_use]
    pub fn overlay(&self) -> &DocumentTextRenderV1 {
        &self.overlay
    }
}
impl ApiTextPlacementDefaultsV1 {
    #[must_use]
    pub fn runs(&self) -> &[ferrum_document::AuthoredTextRunV1] {
        &self.runs
    }
    #[must_use]
    pub const fn font_size(&self) -> f64 {
        self.font_size
    }
    #[must_use]
    pub fn color(&self) -> &str {
        &self.color
    }
}
pub fn begin_api_text_placement_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    anchor: PresentationGesturePoint2V1,
) -> Result<ApiTextPlacementGestureV1, TextPlacementErrorV1> {
    session
        .begin_text_placement_gesture_v1(fence, anchor)
        .map(|document| ApiTextPlacementGestureV1 { document })
}
pub fn preview_api_text_placement_gesture_v1(
    session: &DocumentSession,
    gesture: &ApiTextPlacementGestureV1,
    content: TextPlacementContentV1,
) -> Result<ApiTextPlacementPreviewV1, TextPlacementErrorV1> {
    let document = session.preview_text_placement_gesture_v1(&gesture.document, content.clone())?;
    let overlay = render_candidate(session, &gesture.document, content)?;
    Ok(ApiTextPlacementPreviewV1 { document, overlay })
}

pub fn text_placement_defaults_v1(
    session: &DocumentSession,
    gesture: &ApiTextPlacementGestureV1,
) -> Result<ApiTextPlacementDefaultsV1, TextPlacementErrorV1> {
    let run = ferrum_document::AuthoredTextRunV1::new("Text", Vec::new())
        .expect("fixed default is valid");
    let content = TextPlacementContentV1::new(vec![run.clone()], None, None)?;
    // Validate the live capability before detached renderer work.
    session.preview_text_placement_gesture_v1(&gesture.document, content.clone())?;
    let overlay = render_candidate(session, &gesture.document, content)?;
    Ok(ApiTextPlacementDefaultsV1 {
        runs: vec![run],
        font_size: overlay.operation().size().get(),
        color: format!("#{}", overlay.operation().paint().color().as_str()),
    })
}
pub fn commit_api_text_placement_gesture_v1(
    session: &mut DocumentSession,
    gesture: &ApiTextPlacementGestureV1,
    preview: &ApiTextPlacementPreviewV1,
) -> Result<CommittedTextPlacementV1, TextPlacementErrorV1> {
    session.commit_text_placement_gesture_v1(&gesture.document, &preview.document)
}

fn render_candidate(
    session: &DocumentSession,
    gesture: &TextPlacementGestureV1,
    content: TextPlacementContentV1,
) -> Result<DocumentTextRenderV1, TextPlacementErrorV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
    let mut detached = DocumentSession::load(snapshot.cdml())
        .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
    let detached_snapshot = detached
        .snapshot()
        .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
    let detached_gesture = detached.begin_text_placement_gesture_v1(
        DocumentFenceV1::new(detached_snapshot.revision(), *detached_snapshot.digest()),
        gesture.anchor(),
    )?;
    let detached_preview =
        detached.preview_text_placement_gesture_v1(&detached_gesture, content)?;
    let committed =
        detached.commit_text_placement_gesture_v1(&detached_gesture, &detached_preview)?;
    let observation =
        document_observation_from_accepted_operation_v1(committed.result().observation())
            .map_err(|_| TextPlacementErrorV1::RenderPreparation)?;
    observation
        .text_renders()
        .last()
        .cloned()
        .ok_or(TextPlacementErrorV1::UnrenderableStandard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::TextEditStyleV1;

    #[test]
    fn renderer_preview_matches_the_committed_text_render() {
        let mut session =
            DocumentSession::load("<cdml><standard font_size='18' line_color='#123456'/></cdml>")
                .expect("session");
        let snapshot = session.snapshot().expect("snapshot");
        let gesture = begin_api_text_placement_gesture_v1(
            &session,
            DocumentFenceV1::new(snapshot.revision(), *snapshot.digest()),
            PresentationGesturePoint2V1::new(10.0, 20.0).expect("point"),
        )
        .expect("gesture");
        let content = TextPlacementContentV1::new(
            vec![
                ferrum_document::TextEditRunV1::new("H", vec![]).expect("run"),
                ferrum_document::TextEditRunV1::new("2", vec![TextEditStyleV1::Subscript])
                    .expect("run"),
                ferrum_document::TextEditRunV1::new("O", vec![]).expect("run"),
            ],
            None,
            None,
        )
        .expect("content");
        let preview =
            preview_api_text_placement_gesture_v1(&session, &gesture, content).expect("preview");
        let commit =
            commit_api_text_placement_gesture_v1(&mut session, &gesture, &preview).expect("commit");
        let observation =
            document_observation_from_accepted_operation_v1(commit.result().observation())
                .expect("render");
        assert_eq!(
            preview.overlay(),
            observation.text_renders().last().expect("Text")
        );
        assert!(matches!(
            text_placement_defaults_v1(&session, &gesture),
            Err(TextPlacementErrorV1::StaleSnapshot)
        ));
    }
}

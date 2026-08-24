use super::*;

impl DocumentSession {
    pub fn begin_text_placement_gesture_v1(
        &self,
        fence: DocumentFenceV1,
        anchor: PresentationGesturePoint2V1,
    ) -> Result<crate::TextPlacementGestureV1, crate::TextPlacementErrorV1> {
        crate::text_placement_gesture_v1::begin(&self.authoring_capability_issuer, fence, anchor)
    }

    pub fn prepare_text_placement_gesture_v1(
        &mut self,
        gesture: &crate::TextPlacementGestureV1,
        content: crate::TextPlacementContentV1,
    ) -> Result<PendingTextPlacementV1, crate::TextPlacementErrorV1> {
        text_placement::prepare(self, gesture, content)
    }

    pub fn commit_text_placement_gesture_v1(
        &mut self,
        pending: &mut PendingTextPlacementV1,
    ) -> Result<crate::CommittedTextPlacementV1, crate::TextPlacementErrorV1> {
        text_placement::commit(self, pending)
    }

    pub fn begin_presentation_creation_gesture_v1(
        &self,
        fence: DocumentFenceV1,
        kind: PresentationGestureKindV1,
        start: PresentationGesturePoint2V1,
        style: PresentationGestureStyleV1,
        snap: PresentationGestureSnapPolicyV1,
    ) -> Result<PresentationCreationGestureV1, PresentationGestureErrorV1> {
        presentation_gesture::begin(self, fence, kind, start, style, snap)
    }

    pub fn preview_presentation_creation_gesture_v1(
        &self,
        gesture: &PresentationCreationGestureV1,
        end: PresentationGesturePoint2V1,
    ) -> Result<PresentationCreationPreviewV1, PresentationGestureErrorV1> {
        presentation_gesture::preview(self, gesture, end)
    }

    pub fn resolve_presentation_creation_gesture_v1(
        &self,
        gesture: &PresentationCreationGestureV1,
        preview: &PresentationCreationPreviewV1,
    ) -> Result<SessionOperationTransitionRequestV1, PresentationGestureErrorV1> {
        presentation_gesture::resolve(self, gesture, preview)
    }
}

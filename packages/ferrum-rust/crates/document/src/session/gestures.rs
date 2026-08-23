use super::*;

impl DocumentSession {
    pub fn begin_text_placement_gesture_v1(
        &self,
        fence: DocumentFenceV1,
        anchor: PresentationGesturePoint2V1,
    ) -> Result<crate::TextPlacementGestureV1, crate::TextPlacementErrorV1> {
        crate::text_placement_gesture_v1::begin(&self.authoring_capability_issuer, fence, anchor)
    }

    pub fn preview_text_placement_gesture_v1(
        &self,
        gesture: &crate::TextPlacementGestureV1,
        content: crate::TextPlacementContentV1,
    ) -> Result<crate::TextPlacementPreviewV1, crate::TextPlacementErrorV1> {
        crate::text_placement_gesture_v1::preview(
            &self.authoring_capability_issuer,
            self.history.current().revision(),
            *self.history.current().digest(),
            gesture,
            content,
        )
    }

    pub fn commit_text_placement_gesture_v1(
        &mut self,
        gesture: &crate::TextPlacementGestureV1,
        preview: &crate::TextPlacementPreviewV1,
    ) -> Result<crate::CommittedTextPlacementV1, crate::TextPlacementErrorV1> {
        use crate::text_placement_gesture_v1::{
            CommittedTextPlacementV1, TextPlacementErrorV1, belongs_to,
        };
        if !belongs_to(&self.authoring_capability_issuer, gesture)
            || !belongs_to(&self.authoring_capability_issuer, &preview.gesture)
        {
            return Err(TextPlacementErrorV1::ForeignSession);
        }
        if gesture.capability != preview.gesture.capability {
            return Err(TextPlacementErrorV1::MismatchedPreview);
        }
        let claim = gesture
            .capability
            .claim_for_commit(&self.authoring_capability_issuer)
            .map_err(|error| match error {
                crate::AuthoringCapabilityAccessErrorV1::ForeignSession => {
                    TextPlacementErrorV1::ForeignSession
                }
                crate::AuthoringCapabilityAccessErrorV1::Replayed => {
                    TextPlacementErrorV1::ReplayedGesture
                }
            })?;
        if gesture.fence.revision() != self.history.current().revision()
            || gesture.fence.digest() != *self.history.current().digest()
        {
            return Err(TextPlacementErrorV1::StaleSnapshot);
        }
        let (identifier, next_ids) = self
            .generated_ids
            .reserve_presentation(self.history.current().document().indexed())
            .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_authored_text_v1(
                &identifier,
                gesture.anchor,
                preview.content.runs(),
                preview.content.font_size(),
                preview.content.color(),
            )
            .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(TextPlacementErrorV1::SessionConflict)?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
        self.generated_ids = next_ids;
        self.history.append(state);
        let result = match self.operation_result() {
            Ok(result) => result,
            Err(_) => {
                claim.consume();
                return Err(TextPlacementErrorV1::SessionConflict);
            }
        };
        claim.consume();
        Ok(CommittedTextPlacementV1::new(identifier, result))
    }

    pub fn begin_presentation_creation_gesture_v1(
        &self,
        fence: DocumentFenceV1,
        kind: PresentationGestureKindV1,
        start: PresentationGesturePoint2V1,
        style: PresentationGestureStyleV1,
        snap: PresentationGestureSnapPolicyV1,
    ) -> Result<PresentationCreationGestureV1, PresentationGestureErrorV1> {
        self.require_presentation_fence(fence)?;
        if !matches!(
            (kind, style),
            (
                PresentationGestureKindV1::StraightNormalArrow,
                PresentationGestureStyleV1::Normal(_)
            ) | (
                PresentationGestureKindV1::StraightEquilibriumArrow,
                PresentationGestureStyleV1::Equilibrium
            ) | (
                PresentationGestureKindV1::Plus,
                PresentationGestureStyleV1::Plus
            )
        ) {
            return Err(PresentationGestureErrorV1::InvalidGestureStyle);
        }
        Ok(PresentationCreationGestureV1 {
            capability: self.authoring_capability_issuer.issue(),
            fence,
            kind,
            start,
            style,
            snap,
        })
    }
    pub fn preview_presentation_creation_gesture_v1(
        &self,
        gesture: &PresentationCreationGestureV1,
        end: PresentationGesturePoint2V1,
    ) -> Result<PresentationCreationPreviewV1, PresentationGestureErrorV1> {
        self.require_presentation_capability(gesture)?;
        self.require_presentation_fence(gesture.fence)?;
        presentation_creation_gesture_v1::preview(gesture.clone(), end)
    }
    pub fn commit_presentation_creation_gesture_v1(
        &mut self,
        gesture: &PresentationCreationGestureV1,
        preview: &PresentationCreationPreviewV1,
    ) -> Result<CommittedPresentationGestureV1, PresentationGestureErrorV1> {
        self.require_presentation_origin(gesture)?;
        self.require_presentation_origin(&preview.gesture)?;
        if gesture.capability != preview.gesture.capability {
            return Err(PresentationGestureErrorV1::PreviewMismatch);
        }
        let claim = gesture
            .capability
            .claim_for_commit(&self.authoring_capability_issuer)
            .map_err(|error| match error {
                crate::AuthoringCapabilityAccessErrorV1::ForeignSession => {
                    PresentationGestureErrorV1::ForeignSession
                }
                crate::AuthoringCapabilityAccessErrorV1::Replayed => {
                    PresentationGestureErrorV1::ReplayedGesture
                }
            })?;
        self.require_presentation_fence(gesture.fence)?;
        let (identifier, next_ids) = self
            .generated_ids
            .reserve_presentation(self.history.current().document().indexed())
            .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
        let candidate = match (gesture.kind, gesture.style) {
            (
                PresentationGestureKindV1::StraightNormalArrow,
                PresentationGestureStyleV1::Normal(style),
            ) => self
                .history
                .current()
                .document()
                .with_insert_straight_normal_arrow(
                    &identifier,
                    gesture.start,
                    preview.end,
                    style.start_head(),
                    style.end_head(),
                ),
            (
                PresentationGestureKindV1::StraightEquilibriumArrow,
                PresentationGestureStyleV1::Equilibrium,
            ) => self
                .history
                .current()
                .document()
                .with_insert_straight_equilibrium_arrow(&identifier, gesture.start, preview.end),
            (PresentationGestureKindV1::Plus, PresentationGestureStyleV1::Plus) => self
                .history
                .current()
                .document()
                .with_insert_standard_plus(&identifier, gesture.start),
            _ => return Err(PresentationGestureErrorV1::InvalidGestureStyle),
        }
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(PresentationGestureErrorV1::SessionConflict)?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
        self.generated_ids = next_ids;
        self.history.append(state);
        let result = match self.operation_result() {
            Ok(result) => result,
            Err(_) => {
                claim.consume();
                return Err(PresentationGestureErrorV1::SessionConflict);
            }
        };
        claim.consume();
        Ok(CommittedPresentationGestureV1::new(
            gesture.kind,
            identifier,
            result,
        ))
    }
    fn require_presentation_origin(
        &self,
        gesture: &PresentationCreationGestureV1,
    ) -> Result<(), PresentationGestureErrorV1> {
        if gesture
            .capability
            .belongs_to(&self.authoring_capability_issuer)
        {
            Ok(())
        } else {
            Err(PresentationGestureErrorV1::ForeignSession)
        }
    }
    fn require_presentation_capability(
        &self,
        gesture: &PresentationCreationGestureV1,
    ) -> Result<(), PresentationGestureErrorV1> {
        let claim = gesture
            .capability
            .claim_for_commit(&self.authoring_capability_issuer)
            .map_err(|error| match error {
                crate::AuthoringCapabilityAccessErrorV1::ForeignSession => {
                    PresentationGestureErrorV1::ForeignSession
                }
                crate::AuthoringCapabilityAccessErrorV1::Replayed => {
                    PresentationGestureErrorV1::ReplayedGesture
                }
            })?;
        drop(claim);
        Ok(())
    }
    fn require_presentation_fence(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<(), PresentationGestureErrorV1> {
        if self.history.current().revision() != fence.revision() {
            return Err(PresentationGestureErrorV1::StaleRevision);
        }
        if *self.history.current().digest() != fence.digest() {
            return Err(PresentationGestureErrorV1::StaleDigest);
        }
        Ok(())
    }
}

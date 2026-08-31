//! Opaque, revision-fenced standalone Text authoring transaction.

use crate::{
    AuthoredTextRunV1, AuthoredTextStyleV1, AuthoringCapabilityIssuerV1, AuthoringCapabilityV1,
    DocumentFenceV1, PresentationGesturePoint2V1, Rgb24V1, normalize_authored_text_runs_v1,
};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct TextPlacementGestureV1 {
    pub(crate) capability: AuthoringCapabilityV1,
    pub(crate) fence: DocumentFenceV1,
    pub(crate) anchor: PresentationGesturePoint2V1,
}
impl TextPlacementGestureV1 {
    #[must_use]
    pub const fn anchor(&self) -> PresentationGesturePoint2V1 {
        self.anchor
    }

    pub(crate) fn same_preparation_gesture(&self, other: &Self) -> bool {
        self.capability.same_capability(&other.capability)
            && self.fence == other.fence
            && self.anchor == other.anchor
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct TextPlacementContentV1 {
    runs: Vec<AuthoredTextRunV1>,
    font_size: Option<u16>,
    color: Option<Rgb24V1>,
}
impl TextPlacementContentV1 {
    pub fn new(
        mut runs: Vec<AuthoredTextRunV1>,
        font_size: Option<u16>,
        color: Option<Rgb24V1>,
    ) -> Result<Self, TextPlacementErrorV1> {
        normalize_authored_text_runs_v1(&mut runs)
            .map_err(|_| TextPlacementErrorV1::BlankContent)?;
        if runs.iter().any(|run| {
            run.styles().iter().any(|style| {
                matches!(
                    style,
                    AuthoredTextStyleV1::Bold | AuthoredTextStyleV1::Italic
                )
            })
        }) {
            return Err(TextPlacementErrorV1::UnsupportedStyle);
        }
        if font_size.is_some_and(|value| !(4..=144).contains(&value)) {
            return Err(TextPlacementErrorV1::InvalidFontOverride);
        }
        Ok(Self {
            runs,
            font_size,
            color,
        })
    }
    pub fn runs(&self) -> &[AuthoredTextRunV1] {
        &self.runs
    }
    pub fn font_size(&self) -> Option<u16> {
        self.font_size
    }
    pub fn color(&self) -> Option<&Rgb24V1> {
        self.color.as_ref()
    }
}
#[derive(Clone, Debug)]
pub struct CommittedTextPlacementV1 {
    document_object_id: crate::DocumentObjectIdV1,
    result: crate::SessionOperationResultV1,
}
impl CommittedTextPlacementV1 {
    pub(crate) fn new(
        document_object_id: crate::DocumentObjectIdV1,
        result: crate::SessionOperationResultV1,
    ) -> Self {
        Self {
            document_object_id,
            result,
        }
    }
    pub fn document_object_id(&self) -> &crate::DocumentObjectIdV1 {
        &self.document_object_id
    }
    pub fn result(&self) -> &crate::SessionOperationResultV1 {
        &self.result
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextPlacementErrorCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    Consumed,
    InvalidAnchor,
    BlankContent,
    UnsupportedStyle,
    InvalidFontOverride,
    UnrenderableStandard,
    RenderPreparation,
    SessionConflict,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextPlacementRecoveryV1 {
    RestartTool,
    ChooseAnotherLocation,
    CorrectText,
    RepairDrawingStandard,
    RecoverCanvas,
    RefreshThenRetry,
}
#[derive(Clone, Debug, Error, PartialEq)]
pub enum TextPlacementErrorV1 {
    #[error("Text placement snapshot is stale")]
    StaleSnapshot,
    #[error("Text placement belongs to another session")]
    ForeignSession,
    #[error("Text placement preview does not match the gesture")]
    MismatchedPreview,
    #[error("Text placement gesture was already committed")]
    Consumed,
    #[error("Text placement anchor is not finite")]
    InvalidAnchor,
    #[error("Text content cannot be blank")]
    BlankContent,
    #[error("Bold and italic text require a renderer face and are unavailable")]
    UnsupportedStyle,
    #[error("Text font override is invalid")]
    InvalidFontOverride,
    #[error("The drawing standard cannot render authored Text")]
    UnrenderableStandard,
    #[error("The renderer could not prepare the Text preview")]
    RenderPreparation,
    #[error("Text placement could not be prepared by the document session")]
    SessionConflict,
}
impl TextPlacementErrorV1 {
    pub const fn category(&self) -> TextPlacementErrorCategoryV1 {
        match self {
            Self::StaleSnapshot => TextPlacementErrorCategoryV1::StaleSnapshot,
            Self::ForeignSession => TextPlacementErrorCategoryV1::ForeignSession,
            Self::MismatchedPreview => TextPlacementErrorCategoryV1::MismatchedPreview,
            Self::Consumed => TextPlacementErrorCategoryV1::Consumed,
            Self::InvalidAnchor => TextPlacementErrorCategoryV1::InvalidAnchor,
            Self::BlankContent => TextPlacementErrorCategoryV1::BlankContent,
            Self::UnsupportedStyle => TextPlacementErrorCategoryV1::UnsupportedStyle,
            Self::InvalidFontOverride => TextPlacementErrorCategoryV1::InvalidFontOverride,
            Self::UnrenderableStandard => TextPlacementErrorCategoryV1::UnrenderableStandard,
            Self::RenderPreparation => TextPlacementErrorCategoryV1::RenderPreparation,
            Self::SessionConflict => TextPlacementErrorCategoryV1::SessionConflict,
        }
    }
    pub const fn recovery(&self) -> TextPlacementRecoveryV1 {
        match self {
            Self::InvalidAnchor => TextPlacementRecoveryV1::ChooseAnotherLocation,
            Self::BlankContent | Self::UnsupportedStyle | Self::InvalidFontOverride => {
                TextPlacementRecoveryV1::CorrectText
            }
            Self::UnrenderableStandard => TextPlacementRecoveryV1::RepairDrawingStandard,
            Self::RenderPreparation => TextPlacementRecoveryV1::RecoverCanvas,
            Self::SessionConflict => TextPlacementRecoveryV1::RefreshThenRetry,
            _ => TextPlacementRecoveryV1::RestartTool,
        }
    }
}
pub(crate) fn begin(
    issuer: &AuthoringCapabilityIssuerV1,
    fence: DocumentFenceV1,
    anchor: PresentationGesturePoint2V1,
) -> Result<TextPlacementGestureV1, TextPlacementErrorV1> {
    if !(anchor.x().is_finite() && anchor.y().is_finite()) {
        return Err(TextPlacementErrorV1::InvalidAnchor);
    }
    Ok(TextPlacementGestureV1 {
        capability: issuer.issue(),
        fence,
        anchor,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::DocumentSession;

    fn content(styles: Vec<AuthoredTextStyleV1>) -> TextPlacementContentV1 {
        TextPlacementContentV1::new(
            vec![AuthoredTextRunV1::new("H2O", styles).expect("run")],
            None,
            None,
        )
        .expect("content")
    }

    #[test]
    fn placement_commits_one_canonical_text_and_replays_never_mutate() {
        let mut session = DocumentSession::load(
            "<cdml xmlns=\"urn:ferrum:cdml\" version='26.07'><standard font_family=\"Atkinson Hyperlegible Next\"/></cdml>",
        )
        .expect("canonical bundled face remains renderable");
        let snapshot = session.snapshot().expect("snapshot");
        let gesture = session
            .begin_text_placement_gesture_v1(
                DocumentFenceV1::new(snapshot.revision(), *snapshot.digest()),
                PresentationGesturePoint2V1::new(72.0, 36.0).expect("point"),
            )
            .expect("gesture");
        let mut preview = session
            .prepare_text_placement_gesture_v1(
                &gesture,
                content(vec![AuthoredTextStyleV1::Subscript]),
            )
            .expect("preview");
        let commit = session
            .commit_text_placement_gesture_v1(&mut preview)
            .expect("commit");
        assert_eq!(commit.result().observation().snapshot().revision(), 1);
        let cdml = commit.result().observation().snapshot().cdml();
        assert!(
            cdml.contains("<ftext>&lt;sub&gt;H2O&lt;/sub&gt;</ftext>"),
            "{cdml}"
        );
        assert!(matches!(
            session.commit_text_placement_gesture_v1(&mut preview),
            Err(TextPlacementErrorV1::Consumed)
        ));
        assert_eq!(session.snapshot().expect("snapshot").revision(), 1);
    }

    #[test]
    fn unsupported_face_style_is_rejected_without_a_candidate() {
        assert!(matches!(
            TextPlacementContentV1::new(
                vec![AuthoredTextRunV1::new("x", vec![AuthoredTextStyleV1::Bold]).expect("run")],
                None,
                None
            ),
            Err(TextPlacementErrorV1::UnsupportedStyle)
        ));
    }

    #[test]
    fn unknown_persisted_standard_refuses_text_preview_without_mutation() {
        let mut session = DocumentSession::load(
            "<cdml xmlns=\"urn:ferrum:cdml\"><standard font_family=\"Legacy Serif\"/></cdml>",
        )
        .expect("unknown persisted standard is retained for repair");
        let before = session.snapshot().expect("snapshot");
        let gesture = session
            .begin_text_placement_gesture_v1(
                DocumentFenceV1::new(before.revision(), *before.digest()),
                PresentationGesturePoint2V1::new(1.0, 2.0).expect("point"),
            )
            .expect("gesture");

        assert!(matches!(
            session.prepare_text_placement_gesture_v1(&gesture, content(Vec::new())),
            Err(TextPlacementErrorV1::UnrenderableStandard)
        ));
        assert_eq!(session.snapshot().expect("snapshot"), before);
    }

    #[test]
    fn placement_normalizes_adjacent_runs_but_retains_style_boundaries() {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("session");
        let snapshot = session.snapshot().expect("snapshot");
        let gesture = session
            .begin_text_placement_gesture_v1(
                DocumentFenceV1::new(snapshot.revision(), *snapshot.digest()),
                PresentationGesturePoint2V1::new(1.0, 2.0).expect("point"),
            )
            .expect("gesture");
        let content = TextPlacementContentV1::new(
            vec![
                AuthoredTextRunV1::new("H", vec![]).expect("run"),
                AuthoredTextRunV1::new("2", vec![AuthoredTextStyleV1::Subscript]).expect("run"),
                AuthoredTextRunV1::new("O", vec![AuthoredTextStyleV1::Subscript]).expect("run"),
                AuthoredTextRunV1::new("!", vec![]).expect("run"),
            ],
            None,
            None,
        )
        .expect("content");
        assert_eq!(content.runs().len(), 3);
        assert_eq!(content.runs()[1].text(), "2O");
        let mut preview = session
            .prepare_text_placement_gesture_v1(&gesture, content)
            .expect("preview");
        let commit = session
            .commit_text_placement_gesture_v1(&mut preview)
            .expect("commit");
        assert!(
            commit
                .result()
                .observation()
                .snapshot()
                .cdml()
                .contains("<ftext>H&lt;sub&gt;2O&lt;/sub&gt;!</ftext>")
        );
    }

    #[test]
    fn render_refusals_have_non_retryable_recovery() {
        assert_eq!(
            TextPlacementErrorV1::UnrenderableStandard.category(),
            TextPlacementErrorCategoryV1::UnrenderableStandard
        );
        assert_eq!(
            TextPlacementErrorV1::UnrenderableStandard.recovery(),
            TextPlacementRecoveryV1::RepairDrawingStandard
        );
        assert_eq!(
            TextPlacementErrorV1::RenderPreparation.category(),
            TextPlacementErrorCategoryV1::RenderPreparation
        );
        assert_eq!(
            TextPlacementErrorV1::RenderPreparation.recovery(),
            TextPlacementRecoveryV1::RecoverCanvas
        );
    }
}

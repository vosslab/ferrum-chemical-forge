//! Rust-owned, revision-fenced straight normal-arrow authoring geometry.

use crate::{
    ArrowHeadShapeV1, ArrowProjectionKindV1, AuthoringCapabilityV1, DocumentFenceV1, Point3V1,
    PositiveFiniteV1, PresentationArrowPreviewRequestV1, PresentationFactProvenanceV1,
    PresentationStrokeV1, Rgb24V1,
};
use ferrum_render::{
    PresentationPreviewRenderPlanV1, RenderPoint, lower_arrow_preview_v1,
    lower_standard_plus_preview_v1,
};
use thiserror::Error;

pub const ARROW_MINIMUM_LENGTH_PT_V1: f64 = 2.0;
pub const ARROW_MAXIMUM_LENGTH_PT_V1: f64 = 20_000.0;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationGestureKindV1 {
    StraightNormalArrow,
    StraightEquilibriumArrow,
    Plus,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresentationGesturePoint2V1 {
    x: f64,
    y: f64,
}
impl PresentationGesturePoint2V1 {
    pub fn new(x: f64, y: f64) -> Result<Self, PresentationGestureErrorV1> {
        (x.is_finite() && y.is_finite())
            .then_some(Self { x, y })
            .ok_or(PresentationGestureErrorV1::NonFinitePoint)
    }
    #[must_use]
    pub const fn x(self) -> f64 {
        self.x
    }
    #[must_use]
    pub const fn y(self) -> f64 {
        self.y
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArrowGestureStyleV1 {
    start_head: bool,
    end_head: bool,
}
/// Style facts owned by the specific presentation gesture kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationGestureStyleV1 {
    Normal(ArrowGestureStyleV1),
    Equilibrium,
    Plus,
}
impl PresentationGestureStyleV1 {
    #[must_use]
    pub const fn normal(start_head: bool, end_head: bool) -> Self {
        Self::Normal(ArrowGestureStyleV1::new(start_head, end_head))
    }
}
impl ArrowGestureStyleV1 {
    #[must_use]
    pub const fn new(start_head: bool, end_head: bool) -> Self {
        Self {
            start_head,
            end_head,
        }
    }
    #[must_use]
    pub const fn start_head(self) -> bool {
        self.start_head
    }
    #[must_use]
    pub const fn end_head(self) -> bool {
        self.end_head
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PresentationGestureSnapPolicyV1 {
    angle_increment_degrees: Option<u16>,
    fixed_length_pt: Option<u16>,
}
impl PresentationGestureSnapPolicyV1 {
    pub fn new(
        angle: Option<u16>,
        length: Option<u16>,
    ) -> Result<Self, PresentationGestureErrorV1> {
        if !matches!(angle, None | Some(15 | 30 | 45)) || length == Some(0) {
            return Err(PresentationGestureErrorV1::InvalidSnapPolicy);
        }
        Ok(Self {
            angle_increment_degrees: angle,
            fixed_length_pt: length,
        })
    }
    #[must_use]
    pub const fn free() -> Self {
        Self {
            angle_increment_degrees: None,
            fixed_length_pt: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct PresentationCreationGestureV1 {
    pub(crate) capability: AuthoringCapabilityV1,
    pub(crate) fence: DocumentFenceV1,
    pub(crate) kind: PresentationGestureKindV1,
    pub(crate) start: PresentationGesturePoint2V1,
    pub(crate) style: PresentationGestureStyleV1,
    pub(crate) snap: PresentationGestureSnapPolicyV1,
}
impl PresentationCreationGestureV1 {
    pub(crate) fn same_gesture(&self, other: &Self) -> bool {
        self.capability.same_capability(&other.capability)
            && self.fence == other.fence
            && self.kind == other.kind
            && self.start == other.start
            && self.style == other.style
            && self.snap == other.snap
    }
}
#[derive(Clone, Debug)]
pub struct PresentationCreationPreviewV1 {
    pub(crate) gesture: PresentationCreationGestureV1,
    pub(crate) end: PresentationGesturePoint2V1,
    plan: PresentationPreviewRenderPlanV1,
}
impl PresentationCreationPreviewV1 {
    /// Return the immutable renderer plan for this presentation preview.
    #[must_use]
    pub const fn plan(&self) -> &PresentationPreviewRenderPlanV1 {
        &self.plan
    }

    pub(crate) fn matches_gesture(&self, gesture: &PresentationCreationGestureV1) -> bool {
        self.gesture.same_gesture(gesture)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationGestureCategoryV1 {
    StaleRevision,
    StaleDigest,
    ForeignSession,
    Consumed,
    PreviewMismatch,
    NonFinitePoint,
    CollapsedEndpoint,
    BelowMinimumLength,
    ExceedsGeometryLimit,
    InvalidSnapPolicy,
    InvalidGestureStyle,
    SessionConflict,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationGestureRecoveryV1 {
    RefreshAndRestart,
    AdjustEndpoint,
    ChangeToolOrStyle,
    RefreshAndReport,
}
#[derive(Clone, Debug, Error, PartialEq)]
pub enum PresentationGestureErrorV1 {
    #[error("presentation gesture revision is stale")]
    StaleRevision,
    #[error("presentation gesture digest is stale")]
    StaleDigest,
    #[error("presentation gesture belongs to a different document session")]
    ForeignSession,
    #[error("presentation gesture was already redeemed")]
    Consumed,
    #[error("presentation preview belongs to a different gesture")]
    PreviewMismatch,
    #[error("presentation gesture point is not finite")]
    NonFinitePoint,
    #[error("presentation gesture endpoint collapsed onto its start")]
    CollapsedEndpoint,
    #[error("presentation gesture is below the minimum authored arrow length")]
    BelowMinimumLength,
    #[error("presentation gesture exceeds the geometry limit")]
    ExceedsGeometryLimit,
    #[error("presentation gesture snapping policy is invalid")]
    InvalidSnapPolicy,
    #[error("presentation gesture style does not belong to its kind")]
    InvalidGestureStyle,
    #[error("presentation gesture commit was rejected by the document session")]
    SessionConflict,
}
impl PresentationGestureErrorV1 {
    #[must_use]
    pub const fn category(&self) -> PresentationGestureCategoryV1 {
        match self {
            Self::StaleRevision => PresentationGestureCategoryV1::StaleRevision,
            Self::StaleDigest => PresentationGestureCategoryV1::StaleDigest,
            Self::ForeignSession => PresentationGestureCategoryV1::ForeignSession,
            Self::Consumed => PresentationGestureCategoryV1::Consumed,
            Self::PreviewMismatch => PresentationGestureCategoryV1::PreviewMismatch,
            Self::NonFinitePoint => PresentationGestureCategoryV1::NonFinitePoint,
            Self::CollapsedEndpoint => PresentationGestureCategoryV1::CollapsedEndpoint,
            Self::BelowMinimumLength => PresentationGestureCategoryV1::BelowMinimumLength,
            Self::ExceedsGeometryLimit => PresentationGestureCategoryV1::ExceedsGeometryLimit,
            Self::InvalidSnapPolicy => PresentationGestureCategoryV1::InvalidSnapPolicy,
            Self::InvalidGestureStyle => PresentationGestureCategoryV1::InvalidGestureStyle,
            Self::SessionConflict => PresentationGestureCategoryV1::SessionConflict,
        }
    }
    #[must_use]
    pub const fn recovery(&self) -> PresentationGestureRecoveryV1 {
        match self {
            Self::StaleRevision
            | Self::StaleDigest
            | Self::ForeignSession
            | Self::Consumed
            | Self::PreviewMismatch => PresentationGestureRecoveryV1::RefreshAndRestart,
            Self::NonFinitePoint
            | Self::CollapsedEndpoint
            | Self::BelowMinimumLength
            | Self::ExceedsGeometryLimit => PresentationGestureRecoveryV1::AdjustEndpoint,
            Self::InvalidSnapPolicy | Self::InvalidGestureStyle => {
                PresentationGestureRecoveryV1::ChangeToolOrStyle
            }
            Self::SessionConflict => PresentationGestureRecoveryV1::RefreshAndReport,
        }
    }
}
pub(crate) fn preview(
    gesture: PresentationCreationGestureV1,
    raw_end: PresentationGesturePoint2V1,
) -> Result<PresentationCreationPreviewV1, PresentationGestureErrorV1> {
    if gesture.kind == PresentationGestureKindV1::Plus {
        let plan = lower_standard_plus_preview_v1(
            RenderPoint::new(gesture.start.x(), gesture.start.y())
                .expect("finite gesture anchors form a render point"),
        )
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
        return Ok(PresentationCreationPreviewV1 {
            end: gesture.start,
            plan,
            gesture,
        });
    }
    let end = snap(gesture.start, raw_end, gesture.snap);
    let dx = end.x - gesture.start.x;
    let dy = end.y - gesture.start.y;
    let length = dx.hypot(dy);
    if length == 0.0 {
        return Err(PresentationGestureErrorV1::CollapsedEndpoint);
    }
    if length < ARROW_MINIMUM_LENGTH_PT_V1 {
        return Err(PresentationGestureErrorV1::BelowMinimumLength);
    }
    if length > ARROW_MAXIMUM_LENGTH_PT_V1 {
        return Err(PresentationGestureErrorV1::ExceedsGeometryLimit);
    }
    let kind = match gesture.style {
        PresentationGestureStyleV1::Normal(style)
            if gesture.kind == PresentationGestureKindV1::StraightNormalArrow =>
        {
            ArrowProjectionKindV1::Normal {
                head_shape: ArrowHeadShapeV1::default_authored(),
                start_head: style.start_head(),
                end_head: style.end_head(),
            }
        }
        PresentationGestureStyleV1::Equilibrium
            if gesture.kind == PresentationGestureKindV1::StraightEquilibriumArrow =>
        {
            ArrowProjectionKindV1::Equilibrium
        }
        _ => return Err(PresentationGestureErrorV1::InvalidGestureStyle),
    };
    let request = PresentationArrowPreviewRequestV1::new(
        vec![point3(gesture.start), point3(end)],
        kind,
        builtin_stroke(),
    )
    .map_err(|_| PresentationGestureErrorV1::InvalidGestureStyle)?;
    let plan = lower_arrow_preview_v1(&request)
        .map_err(|_| PresentationGestureErrorV1::BelowMinimumLength)?;
    Ok(PresentationCreationPreviewV1 { plan, gesture, end })
}

fn point3(point: PresentationGesturePoint2V1) -> Point3V1 {
    Point3V1::new(point.x(), point.y(), 0.0).expect("gesture points are finite")
}

fn builtin_stroke() -> PresentationStrokeV1 {
    PresentationStrokeV1::new(
        Rgb24V1::new("#000000").expect("closed builtin arrow color is valid"),
        PresentationFactProvenanceV1::Builtin,
        PositiveFiniteV1::new(1.0).expect("closed builtin arrow width is positive"),
        PresentationFactProvenanceV1::Builtin,
    )
    .expect("closed builtin arrow stroke is coherent")
}
fn snap(
    start: PresentationGesturePoint2V1,
    raw: PresentationGesturePoint2V1,
    policy: PresentationGestureSnapPolicyV1,
) -> PresentationGesturePoint2V1 {
    let dx = raw.x - start.x;
    let dy = raw.y - start.y;
    let mut length = dx.hypot(dy);
    if length == 0.0 {
        return raw;
    }
    let mut angle = dy.atan2(dx);
    if let Some(deg) = policy.angle_increment_degrees {
        let step = f64::from(deg).to_radians();
        angle = (angle / step).round() * step
    }
    if let Some(fixed) = policy.fixed_length_pt {
        length = f64::from(fixed)
    }
    point(
        start.x + length * angle.cos(),
        start.y + length * angle.sin(),
    )
}
const fn point(x: f64, y: f64) -> PresentationGesturePoint2V1 {
    PresentationGesturePoint2V1 { x, y }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snap_and_renderer_plan_are_backend_owned() {
        let issuer = crate::AuthoringCapabilityIssuerV1::new();
        let g = PresentationCreationGestureV1 {
            capability: issuer.issue(),
            fence: DocumentFenceV1::new(0, [0; 32]),
            kind: PresentationGestureKindV1::StraightNormalArrow,
            start: PresentationGesturePoint2V1::new(0.0, 0.0).unwrap(),
            style: PresentationGestureStyleV1::normal(false, true),
            snap: PresentationGestureSnapPolicyV1::new(Some(45), Some(20)).unwrap(),
        };
        let p = preview(g, PresentationGesturePoint2V1::new(8.0, 9.0).unwrap()).unwrap();
        let plan = p.plan();
        let root = plan.roots().first().expect("preview has one root");
        assert_eq!(plan.roots().len(), 1);
        assert_eq!(
            root.vector()
                .expect("preview root is vector")
                .operations()
                .len(),
            2
        );
        assert!(root.bounds().right() > 14.142);
    }
}

#[cfg(test)]
mod session_tests {
    use super::*;
    use crate::{
        CreatedPresentationRootKindV1, DocumentSession, SessionOperationOutcomeV1,
        SessionOperationResultV1,
    };

    fn commit(
        session: &mut DocumentSession,
        gesture: &PresentationCreationGestureV1,
        preview: &PresentationCreationPreviewV1,
    ) -> SessionOperationResultV1 {
        let request = session
            .resolve_presentation_creation_gesture_v1(gesture, preview)
            .expect("generic request resolves");
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("generic transition prepares");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("generic transition commits")
    }

    #[test]
    fn presentation_creation_gesture_commits_canonical_arrow_once() {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("session");
        let before = session.snapshot().expect("before");
        let gesture = session
            .begin_presentation_creation_gesture_v1(
                DocumentFenceV1::new(before.revision(), *before.digest()),
                PresentationGestureKindV1::StraightNormalArrow,
                PresentationGesturePoint2V1::new(0.0, 0.0).expect("start"),
                PresentationGestureStyleV1::normal(false, true),
                PresentationGestureSnapPolicyV1::free(),
            )
            .expect("begin");
        let preview = session
            .preview_presentation_creation_gesture_v1(
                &gesture,
                PresentationGesturePoint2V1::new(72.0, 0.0).expect("end"),
            )
            .expect("preview");
        assert_eq!(session.snapshot().expect("pure preview"), before);
        let plan = preview.plan();
        let root = plan.roots().first().expect("preview has one root");
        assert_eq!(
            root.vector()
                .expect("preview root is vector")
                .operations()
                .len(),
            2
        );
        assert_eq!(root.bounds().right(), 72.0);
        let committed = commit(&mut session, &gesture, &preview);
        assert_eq!(committed.observation().snapshot().revision(), 1);
        assert!(matches!(
            committed.outcome(),
            SessionOperationOutcomeV1::CreatedPresentationRootV1(outcome)
                if outcome.kind() == CreatedPresentationRootKindV1::StraightNormalArrow
        ));
        let cdml = committed.observation().snapshot().cdml();
        assert!(cdml.contains("width=\"1.0\""));
        assert!(cdml.contains("color=\"#000000\""));
        assert_eq!(
            session
                .undo(1)
                .expect("undo")
                .observation()
                .snapshot()
                .revision(),
            2
        );
    }

    #[test]
    fn presentation_creation_gesture_commits_standard_plus_once() {
        let mut session = DocumentSession::load(
            "<cdml xmlns=\"urn:ferrum:cdml\"><standard font_size=\"18\" font_color=\"#123456\"/></cdml>",
        )
        .expect("session");
        let before = session.snapshot().expect("before");
        let gesture = session
            .begin_presentation_creation_gesture_v1(
                DocumentFenceV1::new(before.revision(), *before.digest()),
                PresentationGestureKindV1::Plus,
                PresentationGesturePoint2V1::new(72.0, 36.0).expect("anchor"),
                PresentationGestureStyleV1::Plus,
                PresentationGestureSnapPolicyV1::free(),
            )
            .expect("begin");
        let preview = session
            .preview_presentation_creation_gesture_v1(
                &gesture,
                PresentationGesturePoint2V1::new(0.0, 0.0).expect("ignored endpoint"),
            )
            .expect("preview");
        assert_eq!(session.snapshot().expect("pure preview"), before);
        assert!(!preview.plan().roots().is_empty());
        let committed = commit(&mut session, &gesture, &preview);
        assert!(matches!(
            committed.outcome(),
            SessionOperationOutcomeV1::CreatedPresentationRootV1(outcome)
                if outcome.kind() == CreatedPresentationRootKindV1::Plus
        ));
        let cdml = committed.observation().snapshot().cdml();
        assert!(cdml.contains("<plus"));
        let plus = cdml.split("<plus").nth(1).expect("inserted plus source");
        assert!(
            !plus
                .split("</plus>")
                .next()
                .expect("plus closes")
                .contains("font_size")
        );
        assert_eq!(
            session
                .undo(1)
                .expect("undo")
                .observation()
                .snapshot()
                .revision(),
            2
        );
    }

    #[test]
    fn equilibrium_gesture_commits_one_typed_root_with_renderer_lanes() {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("session");
        let before = session.snapshot().expect("before");
        let gesture = session
            .begin_presentation_creation_gesture_v1(
                DocumentFenceV1::new(before.revision(), *before.digest()),
                PresentationGestureKindV1::StraightEquilibriumArrow,
                PresentationGesturePoint2V1::new(0.0, 0.0).expect("start"),
                PresentationGestureStyleV1::Equilibrium,
                PresentationGestureSnapPolicyV1::free(),
            )
            .expect("begin");
        let preview = session
            .preview_presentation_creation_gesture_v1(
                &gesture,
                PresentationGesturePoint2V1::new(72.0, 0.0).expect("end"),
            )
            .expect("preview");
        let plan = preview.plan();
        let root = plan.roots().first().expect("preview has one root");
        assert_eq!(
            root.vector()
                .expect("preview root is vector")
                .operations()
                .len(),
            3
        );
        assert!(root.bounds().bottom() > root.bounds().top());
        let committed = commit(&mut session, &gesture, &preview);
        let cdml = committed.observation().snapshot().cdml();
        assert_eq!(cdml.matches("<arrow").count(), 1);
        assert!(cdml.contains("type=\"equilibrium\""));
        assert!(!cdml.contains(" start="));
        assert!(!cdml.contains(" end="));
        assert!(matches!(
            committed.outcome(),
            SessionOperationOutcomeV1::CreatedPresentationRootV1(outcome)
                if outcome.kind() == CreatedPresentationRootKindV1::StraightEquilibriumArrow
        ));
    }

    #[test]
    fn equilibrium_gesture_below_its_fixed_span_is_non_mutating() {
        let session = DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("session");
        let before = session.snapshot().expect("before");
        let gesture = session
            .begin_presentation_creation_gesture_v1(
                DocumentFenceV1::new(before.revision(), *before.digest()),
                PresentationGestureKindV1::StraightEquilibriumArrow,
                PresentationGesturePoint2V1::new(0.0, 0.0).expect("start"),
                PresentationGestureStyleV1::Equilibrium,
                PresentationGestureSnapPolicyV1::free(),
            )
            .expect("begin");
        assert!(matches!(
            session.preview_presentation_creation_gesture_v1(
                &gesture,
                PresentationGesturePoint2V1::new(19.0, 0.0).expect("end"),
            ),
            Err(PresentationGestureErrorV1::BelowMinimumLength)
        ));
        assert_eq!(session.snapshot().expect("unchanged"), before);
    }

    #[test]
    fn presentation_creation_gesture_rejects_foreign_mixed_and_stale_without_mutation() {
        let first = DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("first");
        let second = DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("second");
        let first_snapshot = first.snapshot().expect("first snapshot");
        let second_snapshot = second.snapshot().expect("second snapshot");
        let first_gesture = first
            .begin_presentation_creation_gesture_v1(
                DocumentFenceV1::new(first_snapshot.revision(), *first_snapshot.digest()),
                PresentationGestureKindV1::StraightNormalArrow,
                PresentationGesturePoint2V1::new(0.0, 0.0).unwrap(),
                PresentationGestureStyleV1::normal(false, true),
                PresentationGestureSnapPolicyV1::free(),
            )
            .unwrap();
        let _second_gesture = second
            .begin_presentation_creation_gesture_v1(
                DocumentFenceV1::new(second_snapshot.revision(), *second_snapshot.digest()),
                PresentationGestureKindV1::StraightNormalArrow,
                PresentationGesturePoint2V1::new(0.0, 0.0).unwrap(),
                PresentationGestureStyleV1::normal(false, true),
                PresentationGestureSnapPolicyV1::free(),
            )
            .unwrap();
        let first_preview = first
            .preview_presentation_creation_gesture_v1(
                &first_gesture,
                PresentationGesturePoint2V1::new(10.0, 0.0).unwrap(),
            )
            .unwrap();
        assert!(matches!(
            second.preview_presentation_creation_gesture_v1(
                &first_gesture,
                PresentationGesturePoint2V1::new(10.0, 0.0).unwrap()
            ),
            Err(PresentationGestureErrorV1::ForeignSession)
        ));
        assert!(matches!(
            second.resolve_presentation_creation_gesture_v1(&first_gesture, &first_preview),
            Err(PresentationGestureErrorV1::ForeignSession)
        ));
        assert_eq!(second.snapshot().expect("unchanged"), second_snapshot);
    }
}

#[cfg(test)]
mod replay_tests {
    use super::*;
    use crate::{DocumentSession, SessionOperationResultV1};

    fn commit(
        session: &mut DocumentSession,
        gesture: &PresentationCreationGestureV1,
        preview: &PresentationCreationPreviewV1,
    ) -> SessionOperationResultV1 {
        let request = session
            .resolve_presentation_creation_gesture_v1(gesture, preview)
            .expect("generic request resolves");
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("generic transition prepares");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("generic transition commits")
    }

    #[test]
    fn presentation_creation_gesture_replay_is_terminal_and_non_mutating() {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("session");
        let snapshot = session.snapshot().expect("snapshot");
        let gesture = session
            .begin_presentation_creation_gesture_v1(
                DocumentFenceV1::new(snapshot.revision(), *snapshot.digest()),
                PresentationGestureKindV1::StraightNormalArrow,
                PresentationGesturePoint2V1::new(0.0, 0.0).expect("start"),
                PresentationGestureStyleV1::normal(false, true),
                PresentationGestureSnapPolicyV1::free(),
            )
            .expect("begin");
        let preview = session
            .preview_presentation_creation_gesture_v1(
                &gesture,
                PresentationGesturePoint2V1::new(10.0, 0.0).expect("end"),
            )
            .expect("preview");
        let _committed = commit(&mut session, &gesture, &preview);
        let after = session.snapshot().expect("after commit");
        assert!(matches!(
            session.resolve_presentation_creation_gesture_v1(&gesture, &preview),
            Err(PresentationGestureErrorV1::Consumed)
        ));
        assert_eq!(session.snapshot().expect("replay does not mutate"), after);
    }
}

//! Renderer-preflighted, Rust-owned quadratic equilibrium-arrow authoring.

use ferrum_document::{
    ArrowProjectionKindV1, AuthoringCapabilityV1, CreateCurvedEquilibriumArrowV1, DocumentFenceV1,
    DocumentSession, Point3V1, PositiveFiniteV1, PresentationArrowPreviewRequestV1,
    PresentationFactProvenanceV1, PresentationGesturePoint2V1, PresentationStrokeV1, Rgb24V1,
    SessionOperation, SessionOperationTransitionRequestV1, SessionOperationV1,
    TransitionAuthorizationV1,
};
use ferrum_render::{PresentationPreviewRenderPlanV1, lower_arrow_preview_v1};
use thiserror::Error;

const MAXIMUM_EXTENT_PT: f64 = 20_000.0;

#[derive(Debug)]
/// Opaque, session-fenced input for one curved equilibrium-arrow lifecycle.
pub struct CurvedEquilibriumArrowGestureV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
}

#[derive(Clone, Debug)]
/// Rust-issued preview that remains bound to its originating gesture and session.
pub struct CurvedEquilibriumArrowPreviewV1 {
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
    plan: PresentationPreviewRenderPlanV1,
}

impl CurvedEquilibriumArrowPreviewV1 {
    #[must_use]
    pub const fn plan(&self) -> &PresentationPreviewRenderPlanV1 {
        &self.plan
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurvedEquilibriumArrowGestureCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    Consumed,
    InvalidPoint,
    CollapsedSpan,
    ControlTooNearChord,
    ExceedsGeometryLimit,
    RenderPreparation,
    SessionConflict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurvedEquilibriumArrowGestureRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
    DocumentUnchanged,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum CurvedEquilibriumArrowGestureErrorV1 {
    #[error("curved equilibrium-arrow snapshot is stale")]
    StaleSnapshot,
    #[error("curved equilibrium-arrow gesture belongs to another document session")]
    ForeignSession,
    #[error("curved equilibrium-arrow preview belongs to a different gesture")]
    MismatchedPreview,
    #[error("curved equilibrium-arrow receipt was already consumed")]
    Consumed,
    #[error("curved equilibrium-arrow point is invalid")]
    InvalidPoint,
    #[error("curved equilibrium-arrow start and end are too close")]
    CollapsedSpan,
    #[error("curved equilibrium-arrow control point is too close to its chord")]
    ControlTooNearChord,
    #[error("curved equilibrium-arrow exceeds the geometry limit")]
    ExceedsGeometryLimit,
    #[error("curved equilibrium-arrow candidate failed renderer preflight")]
    RenderPreparation,
    #[error("curved equilibrium-arrow session transaction failed")]
    SessionConflict,
}

impl CurvedEquilibriumArrowGestureErrorV1 {
    #[must_use]
    pub const fn category(&self) -> CurvedEquilibriumArrowGestureCategoryV1 {
        match self {
            Self::StaleSnapshot => CurvedEquilibriumArrowGestureCategoryV1::StaleSnapshot,
            Self::ForeignSession => CurvedEquilibriumArrowGestureCategoryV1::ForeignSession,
            Self::MismatchedPreview => CurvedEquilibriumArrowGestureCategoryV1::MismatchedPreview,
            Self::Consumed => CurvedEquilibriumArrowGestureCategoryV1::Consumed,
            Self::InvalidPoint => CurvedEquilibriumArrowGestureCategoryV1::InvalidPoint,
            Self::CollapsedSpan => CurvedEquilibriumArrowGestureCategoryV1::CollapsedSpan,
            Self::ControlTooNearChord => {
                CurvedEquilibriumArrowGestureCategoryV1::ControlTooNearChord
            }
            Self::ExceedsGeometryLimit => {
                CurvedEquilibriumArrowGestureCategoryV1::ExceedsGeometryLimit
            }
            Self::RenderPreparation => CurvedEquilibriumArrowGestureCategoryV1::RenderPreparation,
            Self::SessionConflict => CurvedEquilibriumArrowGestureCategoryV1::SessionConflict,
        }
    }
    #[must_use]
    pub const fn recovery(&self) -> CurvedEquilibriumArrowGestureRecoveryV1 {
        match self {
            Self::StaleSnapshot
            | Self::ForeignSession
            | Self::MismatchedPreview
            | Self::Consumed
            | Self::SessionConflict => CurvedEquilibriumArrowGestureRecoveryV1::RefreshAndRestart,
            Self::CollapsedSpan | Self::ControlTooNearChord | Self::ExceedsGeometryLimit => {
                CurvedEquilibriumArrowGestureRecoveryV1::ChangeGeometry
            }
            Self::InvalidPoint | Self::RenderPreparation => {
                CurvedEquilibriumArrowGestureRecoveryV1::DocumentUnchanged
            }
        }
    }
}

/// Begin a session-fenced gesture from one current document snapshot.
///
/// The returned opaque handle is accepted only by the issuing session.
pub fn begin_curved_equilibrium_arrow_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
) -> Result<CurvedEquilibriumArrowGestureV1, CurvedEquilibriumArrowGestureErrorV1> {
    require_fence(session, fence)?;
    Ok(CurvedEquilibriumArrowGestureV1 {
        capability: session.issue_authoring_capability_v1(),
        fence,
        start,
        control,
    })
}

pub fn preview_curved_equilibrium_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: &CurvedEquilibriumArrowGestureV1,
    end: PresentationGesturePoint2V1,
) -> Result<CurvedEquilibriumArrowPreviewV1, CurvedEquilibriumArrowGestureErrorV1> {
    require_fence(session, gesture.fence)?;
    let plan = preview_plan(gesture.start, gesture.control, end)?;
    Ok(CurvedEquilibriumArrowPreviewV1 {
        fence: gesture.fence,
        start: gesture.start,
        control: gesture.control,
        end,
        plan,
    })
}

/// Resolve a Rust-issued preview into a generic document-transition request without mutation.
///
/// Generic document APIs own preparation and commit.
pub fn resolve_curved_equilibrium_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: CurvedEquilibriumArrowGestureV1,
    preview: CurvedEquilibriumArrowPreviewV1,
) -> Result<SessionOperationTransitionRequestV1, CurvedEquilibriumArrowGestureErrorV1> {
    if gesture.fence != preview.fence
        || gesture.start != preview.start
        || gesture.control != preview.control
    {
        return Err(CurvedEquilibriumArrowGestureErrorV1::MismatchedPreview);
    }
    require_fence(session, gesture.fence)?;
    Ok(SessionOperationTransitionRequestV1::new(
        gesture.fence.revision(),
        SessionOperation::V1(SessionOperationV1::CreateCurvedEquilibriumArrowV1(
            CreateCurvedEquilibriumArrowV1::new(gesture.start, gesture.control, preview.end),
        )),
        TransitionAuthorizationV1::authoring_capability(gesture.capability),
    ))
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), CurvedEquilibriumArrowGestureErrorV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| CurvedEquilibriumArrowGestureErrorV1::SessionConflict)?;
    (snapshot.revision() == fence.revision() && snapshot.digest() == &fence.digest())
        .then_some(())
        .ok_or(CurvedEquilibriumArrowGestureErrorV1::StaleSnapshot)
}

fn preview_plan(
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
) -> Result<PresentationPreviewRenderPlanV1, CurvedEquilibriumArrowGestureErrorV1> {
    if [start, control, end]
        .into_iter()
        .any(|point| point.x().abs() > MAXIMUM_EXTENT_PT || point.y().abs() > MAXIMUM_EXTENT_PT)
    {
        return Err(CurvedEquilibriumArrowGestureErrorV1::ExceedsGeometryLimit);
    }
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let span = dx.hypot(dy);
    if !span.is_finite() || span < 2.0 {
        return Err(CurvedEquilibriumArrowGestureErrorV1::CollapsedSpan);
    }
    let control_distance =
        ((control.x() - start.x()) * dy - (control.y() - start.y()) * dx).abs() / span;
    if !control_distance.is_finite() || control_distance < 1.0 {
        return Err(CurvedEquilibriumArrowGestureErrorV1::ControlTooNearChord);
    }
    let request = PresentationArrowPreviewRequestV1::new(
        vec![point3(start), point3(control), point3(end)],
        ArrowProjectionKindV1::CurvedEquilibrium,
        builtin_stroke(),
    )
    .map_err(|_| CurvedEquilibriumArrowGestureErrorV1::InvalidPoint)?;
    lower_arrow_preview_v1(&request)
        .map_err(|_| CurvedEquilibriumArrowGestureErrorV1::ControlTooNearChord)
}

fn point3(point: PresentationGesturePoint2V1) -> Point3V1 {
    Point3V1::new(point.x(), point.y(), 0.0).expect("validated finite geometry")
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

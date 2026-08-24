//! Renderer-preflighted, Rust-owned quadratic electron-arrow authoring.

use ferrum_document::{
    ArrowProjectionKindV1, AuthoringCapabilityV1, CreateCurvedTerminalArrowV1,
    CurvedTerminalArrowKindV1, DocumentFenceV1, DocumentSession, Point3V1, PositiveFiniteV1,
    PresentationArrowPreviewRequestV1, PresentationFactProvenanceV1, PresentationGesturePoint2V1,
    PresentationStrokeV1, Rgb24V1, SessionOperation, SessionOperationTransitionRequestV1,
    SessionOperationV1, TransitionAuthorizationV1,
};
use ferrum_render::{PresentationRenderPlanV1, lower_arrow_preview_v1};
use thiserror::Error;

const MINIMUM_SPAN_PT: f64 = 2.0;
const MINIMUM_CONTROL_DISTANCE_PT: f64 = 1.0;
const MAXIMUM_EXTENT_PT: f64 = 20_000.0;

#[derive(Debug)]
pub struct CurvedElectronArrowGestureV1 {
    kind: CurvedTerminalArrowKindV1,
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
}

#[derive(Clone, Debug)]
pub struct CurvedElectronArrowPreviewV1 {
    kind: CurvedTerminalArrowKindV1,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
    plan: PresentationRenderPlanV1,
}

impl CurvedElectronArrowPreviewV1 {
    #[must_use]
    pub const fn plan(&self) -> &PresentationRenderPlanV1 {
        &self.plan
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurvedElectronArrowGestureCategoryV1 {
    ForeignSession,
    StaleSnapshot,
    MismatchedPreview,
    ReplayedGesture,
    InvalidPoint,
    CollapsedSpan,
    ControlTooNearChord,
    ExceedsGeometryLimit,
    RenderPreparation,
    SessionConflict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurvedElectronArrowGestureRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
    DocumentUnchanged,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum CurvedElectronArrowGestureErrorV1 {
    #[error("curved terminal-arrow belongs to a different document session")]
    ForeignSession,
    #[error("curved terminal-arrow snapshot is stale")]
    StaleSnapshot,
    #[error("curved terminal-arrow preview belongs to a different gesture")]
    MismatchedPreview,
    #[error("curved terminal-arrow receipt was already consumed")]
    ReplayedGesture,
    #[error("curved terminal-arrow point is invalid")]
    InvalidPoint,
    #[error("curved terminal-arrow start and end are too close")]
    CollapsedSpan,
    #[error("curved terminal-arrow control point is too close to its chord")]
    ControlTooNearChord,
    #[error("curved terminal-arrow exceeds the geometry limit")]
    ExceedsGeometryLimit,
    #[error("curved terminal-arrow candidate failed renderer preflight")]
    RenderPreparation,
    #[error("curved terminal-arrow session transaction failed")]
    SessionConflict,
}

impl CurvedElectronArrowGestureErrorV1 {
    #[must_use]
    pub const fn category(&self) -> CurvedElectronArrowGestureCategoryV1 {
        match self {
            Self::ForeignSession => CurvedElectronArrowGestureCategoryV1::ForeignSession,
            Self::StaleSnapshot => CurvedElectronArrowGestureCategoryV1::StaleSnapshot,
            Self::MismatchedPreview => CurvedElectronArrowGestureCategoryV1::MismatchedPreview,
            Self::ReplayedGesture => CurvedElectronArrowGestureCategoryV1::ReplayedGesture,
            Self::InvalidPoint => CurvedElectronArrowGestureCategoryV1::InvalidPoint,
            Self::CollapsedSpan => CurvedElectronArrowGestureCategoryV1::CollapsedSpan,
            Self::ControlTooNearChord => CurvedElectronArrowGestureCategoryV1::ControlTooNearChord,
            Self::ExceedsGeometryLimit => {
                CurvedElectronArrowGestureCategoryV1::ExceedsGeometryLimit
            }
            Self::RenderPreparation => CurvedElectronArrowGestureCategoryV1::RenderPreparation,
            Self::SessionConflict => CurvedElectronArrowGestureCategoryV1::SessionConflict,
        }
    }
    #[must_use]
    pub const fn recovery(&self) -> CurvedElectronArrowGestureRecoveryV1 {
        match self {
            Self::ForeignSession
            | Self::StaleSnapshot
            | Self::MismatchedPreview
            | Self::ReplayedGesture
            | Self::SessionConflict => CurvedElectronArrowGestureRecoveryV1::RefreshAndRestart,
            Self::CollapsedSpan | Self::ControlTooNearChord | Self::ExceedsGeometryLimit => {
                CurvedElectronArrowGestureRecoveryV1::ChangeGeometry
            }
            Self::InvalidPoint | Self::RenderPreparation => {
                CurvedElectronArrowGestureRecoveryV1::DocumentUnchanged
            }
        }
    }
}

pub fn begin_curved_electron_arrow_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
) -> Result<CurvedElectronArrowGestureV1, CurvedElectronArrowGestureErrorV1> {
    begin_curved_terminal_arrow_gesture_v1(
        session,
        fence,
        start,
        control,
        CurvedTerminalArrowKindV1::Electron,
    )
}

fn begin_curved_terminal_arrow_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    kind: CurvedTerminalArrowKindV1,
) -> Result<CurvedElectronArrowGestureV1, CurvedElectronArrowGestureErrorV1> {
    require_fence(session, fence)?;
    Ok(CurvedElectronArrowGestureV1 {
        kind,
        capability: session.issue_authoring_capability_v1(),
        fence,
        start,
        control,
    })
}

pub fn preview_curved_electron_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: &CurvedElectronArrowGestureV1,
    end: PresentationGesturePoint2V1,
) -> Result<CurvedElectronArrowPreviewV1, CurvedElectronArrowGestureErrorV1> {
    require_fence(session, gesture.fence)?;
    let plan = preview_plan(gesture.kind, gesture.start, gesture.control, end)?;
    Ok(CurvedElectronArrowPreviewV1 {
        kind: gesture.kind,
        fence: gesture.fence,
        start: gesture.start,
        control: gesture.control,
        end,
        plan,
    })
}

pub fn resolve_curved_electron_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: CurvedElectronArrowGestureV1,
    preview: CurvedElectronArrowPreviewV1,
) -> Result<SessionOperationTransitionRequestV1, CurvedElectronArrowGestureErrorV1> {
    if gesture.kind != preview.kind
        || gesture.fence != preview.fence
        || gesture.start != preview.start
        || gesture.control != preview.control
    {
        return Err(CurvedElectronArrowGestureErrorV1::MismatchedPreview);
    }
    require_fence(session, gesture.fence)?;
    Ok(SessionOperationTransitionRequestV1::new(
        gesture.fence.revision(),
        SessionOperation::V1(SessionOperationV1::CreateCurvedTerminalArrowV1(
            CreateCurvedTerminalArrowV1::new(
                gesture.kind,
                gesture.start,
                gesture.control,
                preview.end,
            ),
        )),
        TransitionAuthorizationV1::authoring_capability(gesture.capability),
    ))
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), CurvedElectronArrowGestureErrorV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| CurvedElectronArrowGestureErrorV1::SessionConflict)?;
    (snapshot.revision() == fence.revision() && snapshot.digest() == &fence.digest())
        .then_some(())
        .ok_or(CurvedElectronArrowGestureErrorV1::StaleSnapshot)
}

fn preview_plan(
    kind: CurvedTerminalArrowKindV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
) -> Result<PresentationRenderPlanV1, CurvedElectronArrowGestureErrorV1> {
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let span = dx.hypot(dy);
    if !span.is_finite() || span < MINIMUM_SPAN_PT {
        return Err(CurvedElectronArrowGestureErrorV1::CollapsedSpan);
    }
    if [start, control, end]
        .into_iter()
        .any(|point| point.x().abs() > MAXIMUM_EXTENT_PT || point.y().abs() > MAXIMUM_EXTENT_PT)
    {
        return Err(CurvedElectronArrowGestureErrorV1::ExceedsGeometryLimit);
    }
    let distance = ((control.x() - start.x()) * dy - (control.y() - start.y()) * dx).abs() / span;
    if !distance.is_finite() || distance < MINIMUM_CONTROL_DISTANCE_PT {
        return Err(CurvedElectronArrowGestureErrorV1::ControlTooNearChord);
    }
    let tangent_x = end.x() - control.x();
    let tangent_y = end.y() - control.y();
    let tangent = tangent_x.hypot(tangent_y);
    if tangent < MINIMUM_CONTROL_DISTANCE_PT {
        return Err(CurvedElectronArrowGestureErrorV1::ControlTooNearChord);
    }
    let request = PresentationArrowPreviewRequestV1::new(
        vec![point3(start), point3(control), point3(end)],
        ArrowProjectionKindV1::CurvedTerminal {
            terminal_kind: kind,
        },
        builtin_stroke(),
    )
    .map_err(|_| CurvedElectronArrowGestureErrorV1::InvalidPoint)?;
    lower_arrow_preview_v1(&request)
        .map_err(|_| CurvedElectronArrowGestureErrorV1::ControlTooNearChord)
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

/// Closed retro-arrow aliases retain the trusted opaque lifecycle while the
/// document-owned policy selects the persisted `type="retro"` grammar.
pub type CurvedRetroArrowGestureV1 = CurvedElectronArrowGestureV1;
pub type CurvedRetroArrowPreviewV1 = CurvedElectronArrowPreviewV1;
pub type CurvedRetroArrowGestureCategoryV1 = CurvedElectronArrowGestureCategoryV1;
pub type CurvedRetroArrowGestureRecoveryV1 = CurvedElectronArrowGestureRecoveryV1;
pub type CurvedRetroArrowGestureErrorV1 = CurvedElectronArrowGestureErrorV1;

pub fn begin_curved_retro_arrow_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
) -> Result<CurvedRetroArrowGestureV1, CurvedRetroArrowGestureErrorV1> {
    begin_curved_terminal_arrow_gesture_v1(
        session,
        fence,
        start,
        control,
        CurvedTerminalArrowKindV1::Retro,
    )
}

pub fn preview_curved_retro_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: &CurvedRetroArrowGestureV1,
    end: PresentationGesturePoint2V1,
) -> Result<CurvedRetroArrowPreviewV1, CurvedRetroArrowGestureErrorV1> {
    preview_curved_electron_arrow_gesture_v1(session, gesture, end)
}

pub fn resolve_curved_retro_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: CurvedRetroArrowGestureV1,
    preview: CurvedRetroArrowPreviewV1,
) -> Result<SessionOperationTransitionRequestV1, CurvedRetroArrowGestureErrorV1> {
    resolve_curved_electron_arrow_gesture_v1(session, gesture, preview)
}

/// Closed curved-normal-reaction-arrow aliases retain the shared opaque lifecycle.
pub type CurvedNormalReactionArrowGestureV1 = CurvedElectronArrowGestureV1;
pub type CurvedNormalReactionArrowPreviewV1 = CurvedElectronArrowPreviewV1;
pub type CurvedNormalReactionArrowGestureCategoryV1 = CurvedElectronArrowGestureCategoryV1;
pub type CurvedNormalReactionArrowGestureRecoveryV1 = CurvedElectronArrowGestureRecoveryV1;
pub type CurvedNormalReactionArrowGestureErrorV1 = CurvedElectronArrowGestureErrorV1;

pub fn begin_curved_normal_reaction_arrow_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
) -> Result<CurvedNormalReactionArrowGestureV1, CurvedNormalReactionArrowGestureErrorV1> {
    begin_curved_terminal_arrow_gesture_v1(
        session,
        fence,
        start,
        control,
        CurvedTerminalArrowKindV1::Normal,
    )
}

pub fn preview_curved_normal_reaction_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: &CurvedNormalReactionArrowGestureV1,
    end: PresentationGesturePoint2V1,
) -> Result<CurvedNormalReactionArrowPreviewV1, CurvedNormalReactionArrowGestureErrorV1> {
    preview_curved_electron_arrow_gesture_v1(session, gesture, end)
}

pub fn resolve_curved_normal_reaction_arrow_gesture_v1(
    session: &DocumentSession,
    gesture: CurvedNormalReactionArrowGestureV1,
    preview: CurvedNormalReactionArrowPreviewV1,
) -> Result<SessionOperationTransitionRequestV1, CurvedNormalReactionArrowGestureErrorV1> {
    resolve_curved_electron_arrow_gesture_v1(session, gesture, preview)
}

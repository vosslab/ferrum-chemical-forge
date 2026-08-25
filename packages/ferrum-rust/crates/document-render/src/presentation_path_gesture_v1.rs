//! Renderer-preflighted multi-point Polyline and Polygon authoring.

use ferrum_document::{
    AuthoringCapabilityV1, CreatePresentationPathV1, DocumentFenceV1, DocumentSession,
    GeometricLineWidthV1, PRESENTATION_PATH_MAXIMUM_POINTS_V1, PresentationAppearanceV1,
    PresentationGesturePoint2V1, PresentationPathGestureErrorV1, PresentationPathGestureV1,
    PresentationPathKindV1, Rgb24V1, SessionOperation, SessionOperationTransitionRequestV1,
    SessionOperationV1, TransitionAuthorizationV1, TransparentOrRgb24V1,
};
use thiserror::Error;

use super::require_fence;

#[derive(Clone, Debug, PartialEq)]
pub struct PresentationPathAppearanceV1 {
    stroke_color: Rgb24V1,
    stroke_width: GeometricLineWidthV1,
    fill_color: Option<Rgb24V1>,
}
impl PresentationPathAppearanceV1 {
    #[must_use]
    pub fn stroke_color(&self) -> &str {
        self.stroke_color.as_str()
    }
    #[must_use]
    pub fn stroke_width(&self) -> f64 {
        self.stroke_width.value()
    }
    #[must_use]
    pub fn fill_color(&self) -> Option<&str> {
        self.fill_color.as_ref().map(Rgb24V1::as_str)
    }
}

#[derive(Debug)]
/// Opaque, session-fenced candidate for one incremental presentation path.
pub struct PresentationPathRenderGestureV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    kind: PresentationPathKindV1,
    appearance: PresentationPathAppearanceV1,
    points: Vec<PresentationGesturePoint2V1>,
}

impl PresentationPathRenderGestureV1 {
    /// Return accepted vertices in their exact authored order.
    #[must_use]
    pub fn points(&self) -> &[PresentationGesturePoint2V1] {
        &self.points
    }
}

/// Rust-derived progress for one opaque incremental path candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PresentationPathProgressV1 {
    accepted_points: usize,
    minimum_points: usize,
}

impl PresentationPathProgressV1 {
    #[must_use]
    pub const fn accepted_points(self) -> usize {
        self.accepted_points
    }

    #[must_use]
    pub const fn minimum_points(self) -> usize {
        self.minimum_points
    }

    #[must_use]
    pub const fn can_prepare(self) -> bool {
        self.accepted_points >= self.minimum_points
    }
}

/// Immutable Rust-issued display state for an incremental path candidate.
#[derive(Clone, Debug)]
pub struct PresentationPathOverlayV1 {
    fence: DocumentFenceV1,
    kind: PresentationPathKindV1,
    appearance: PresentationPathAppearanceV1,
    points: Vec<PresentationGesturePoint2V1>,
    hover: Option<PresentationGesturePoint2V1>,
    path: Option<PresentationPathGestureV1>,
}

impl PresentationPathOverlayV1 {
    /// Return the immutable appearance issued by Rust for this display state.
    #[must_use]
    pub const fn appearance(&self) -> &PresentationPathAppearanceV1 {
        &self.appearance
    }

    #[must_use]
    pub fn accepted_points(&self) -> &[PresentationGesturePoint2V1] {
        &self.points
    }

    #[must_use]
    pub const fn hover(&self) -> Option<PresentationGesturePoint2V1> {
        self.hover
    }

    #[must_use]
    pub fn path(&self) -> Option<&PresentationPathGestureV1> {
        self.path.as_ref()
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationPathRenderCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    ReplayedGesture,
    Cancelled,
    InvalidGeometry,
    RenderPreparation,
    SessionConflict,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationPathRenderRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
    ReduceRequest,
    DocumentUnchanged,
}
#[derive(Clone, Debug, Error, PartialEq)]
pub enum PresentationPathRenderErrorV1 {
    #[error("presentation path gesture snapshot is stale")]
    StaleSnapshot,
    #[error("presentation path gesture belongs to another document session")]
    ForeignSession,
    #[error("presentation path preview belongs to another gesture")]
    MismatchedPreview,
    #[error("presentation path gesture was already committed")]
    ReplayedGesture,
    #[error("presentation path gesture was cancelled without changing the document")]
    Cancelled,
    #[error("{0}")]
    InvalidGeometry(PresentationPathGestureErrorV1),
    #[error("presentation path candidate could not be rendered for preview")]
    RenderPreparation,
    #[error("presentation path commit was rejected by the document session")]
    SessionConflict,
}
impl PresentationPathRenderErrorV1 {
    #[must_use]
    pub const fn category(&self) -> PresentationPathRenderCategoryV1 {
        match self {
            Self::StaleSnapshot => PresentationPathRenderCategoryV1::StaleSnapshot,
            Self::ForeignSession => PresentationPathRenderCategoryV1::ForeignSession,
            Self::MismatchedPreview => PresentationPathRenderCategoryV1::MismatchedPreview,
            Self::ReplayedGesture => PresentationPathRenderCategoryV1::ReplayedGesture,
            Self::Cancelled => PresentationPathRenderCategoryV1::Cancelled,
            Self::InvalidGeometry(_) => PresentationPathRenderCategoryV1::InvalidGeometry,
            Self::RenderPreparation => PresentationPathRenderCategoryV1::RenderPreparation,
            Self::SessionConflict => PresentationPathRenderCategoryV1::SessionConflict,
        }
    }
    #[must_use]
    pub const fn recovery(&self) -> PresentationPathRenderRecoveryV1 {
        match self {
            Self::StaleSnapshot
            | Self::ForeignSession
            | Self::MismatchedPreview
            | Self::ReplayedGesture
            | Self::SessionConflict => PresentationPathRenderRecoveryV1::RefreshAndRestart,
            Self::Cancelled => PresentationPathRenderRecoveryV1::DocumentUnchanged,
            Self::InvalidGeometry(PresentationPathGestureErrorV1::ResourceExhausted) => {
                PresentationPathRenderRecoveryV1::ReduceRequest
            }
            Self::InvalidGeometry(_) => PresentationPathRenderRecoveryV1::ChangeGeometry,
            Self::RenderPreparation => PresentationPathRenderRecoveryV1::DocumentUnchanged,
        }
    }
}

/// Begin a session-fenced incremental path from one current document snapshot.
///
/// The returned opaque handle is accepted only by the issuing session.
pub fn begin_presentation_path_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    kind: PresentationPathKindV1,
) -> Result<PresentationPathRenderGestureV1, PresentationPathRenderErrorV1> {
    require_fence(session, fence).map_err(|_| PresentationPathRenderErrorV1::StaleSnapshot)?;
    Ok(PresentationPathRenderGestureV1 {
        capability: session.issue_authoring_capability_v1(),
        fence,
        kind,
        appearance: appearance(session, fence)?,
        points: Vec::new(),
    })
}

/// Add exactly one finite scene point to an opaque candidate.
pub fn add_presentation_path_gesture_point_v1(
    session: &DocumentSession,
    gesture: &mut PresentationPathRenderGestureV1,
    point: PresentationGesturePoint2V1,
) -> Result<PresentationPathProgressV1, PresentationPathRenderErrorV1> {
    require_fence(session, gesture.fence)
        .map_err(|_| PresentationPathRenderErrorV1::StaleSnapshot)?;
    if gesture.points.len() >= PRESENTATION_PATH_MAXIMUM_POINTS_V1 {
        return Err(PresentationPathRenderErrorV1::InvalidGeometry(
            PresentationPathGestureErrorV1::ResourceExhausted,
        ));
    }
    if gesture.points.contains(&point) {
        return Err(PresentationPathRenderErrorV1::InvalidGeometry(
            PresentationPathGestureErrorV1::DegenerateGeometry,
        ));
    }
    gesture.points.push(point);
    Ok(progress(gesture.kind, gesture.points.len()))
}

/// Derive immutable display geometry from accepted vertices and one optional hover point.
pub fn preview_incremental_presentation_path_gesture_v1(
    session: &DocumentSession,
    gesture: &PresentationPathRenderGestureV1,
    hover: Option<PresentationGesturePoint2V1>,
) -> Result<PresentationPathOverlayV1, PresentationPathRenderErrorV1> {
    require_fence(session, gesture.fence)
        .map_err(|_| PresentationPathRenderErrorV1::StaleSnapshot)?;
    if hover.is_some_and(|point| gesture.points.contains(&point)) {
        return Err(PresentationPathRenderErrorV1::InvalidGeometry(
            PresentationPathGestureErrorV1::DegenerateGeometry,
        ));
    }
    // Hover is display-only. Persistent candidates contain accepted points only.
    let path = (gesture.points.len() >= minimum_points(gesture.kind))
        .then(|| PresentationPathGestureV1::new(gesture.kind, gesture.points.clone()))
        .transpose()
        .map_err(PresentationPathRenderErrorV1::InvalidGeometry)?;
    Ok(PresentationPathOverlayV1 {
        fence: gesture.fence,
        kind: gesture.kind,
        appearance: gesture.appearance.clone(),
        points: gesture.points.clone(),
        hover,
        path,
    })
}

/// Cancel before preparation by dropping the move-only gesture; this compatibility
/// boundary reports that no document transition occurred.
pub fn cancel_presentation_path_gesture_v1(
    session: &DocumentSession,
    gesture: &PresentationPathRenderGestureV1,
) -> Result<(), PresentationPathRenderErrorV1> {
    require_fence(session, gesture.fence)
        .map_err(|_| PresentationPathRenderErrorV1::StaleSnapshot)?;
    Err(PresentationPathRenderErrorV1::Cancelled)
}

/// Resolve a Rust-issued overlay into a generic document-transition request without mutation.
///
/// Generic document APIs own preparation and commit.
pub fn resolve_incremental_presentation_path_gesture_v1(
    session: &DocumentSession,
    gesture: PresentationPathRenderGestureV1,
    overlay: PresentationPathOverlayV1,
) -> Result<SessionOperationTransitionRequestV1, PresentationPathRenderErrorV1> {
    if gesture.fence != overlay.fence
        || gesture.kind != overlay.kind
        || gesture.appearance != overlay.appearance
        || gesture.points != overlay.points
    {
        return Err(PresentationPathRenderErrorV1::MismatchedPreview);
    }
    require_fence(session, gesture.fence)
        .map_err(|_| PresentationPathRenderErrorV1::StaleSnapshot)?;
    let path = overlay
        .path
        .as_ref()
        .ok_or(PresentationPathRenderErrorV1::InvalidGeometry(
            PresentationPathGestureErrorV1::InsufficientPoints,
        ))?;
    resolve_path(gesture, path)
}

fn resolve_path(
    gesture: PresentationPathRenderGestureV1,
    path: &PresentationPathGestureV1,
) -> Result<SessionOperationTransitionRequestV1, PresentationPathRenderErrorV1> {
    Ok(SessionOperationTransitionRequestV1::new(
        gesture.fence.revision(),
        SessionOperation::V1(SessionOperationV1::CreatePresentationPathV1(
            CreatePresentationPathV1::new(
                path.clone(),
                PresentationAppearanceV1::new(
                    gesture.appearance.stroke_color,
                    gesture.appearance.stroke_width,
                    gesture.appearance.fill_color,
                ),
            ),
        )),
        TransitionAuthorizationV1::authoring_capability(gesture.capability),
    ))
}

const fn minimum_points(kind: PresentationPathKindV1) -> usize {
    match kind {
        PresentationPathKindV1::Polyline => 2,
        PresentationPathKindV1::Polygon => 3,
    }
}

const fn progress(
    kind: PresentationPathKindV1,
    accepted_points: usize,
) -> PresentationPathProgressV1 {
    PresentationPathProgressV1 {
        accepted_points,
        minimum_points: minimum_points(kind),
    }
}

fn appearance(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<PresentationPathAppearanceV1, PresentationPathRenderErrorV1> {
    let observation = session
        .observe(fence.revision())
        .map_err(|_| PresentationPathRenderErrorV1::RenderPreparation)?;
    let standard = observation.projection().drawing_standard();
    Ok(PresentationPathAppearanceV1 {
        stroke_color: standard
            .and_then(|value| value.line_color())
            .cloned()
            .unwrap_or_else(|| Rgb24V1::new("#000000").expect("closed built-in colour")),
        stroke_width: GeometricLineWidthV1::new(
            standard
                .and_then(|value| value.line_width())
                .map_or(1.0, |value| value.value()),
        )
        .ok_or(PresentationPathRenderErrorV1::RenderPreparation)?,
        fill_color: standard
            .and_then(|value| value.area_color())
            .and_then(|value| match value {
                TransparentOrRgb24V1::Transparent => None,
                TransparentOrRgb24V1::Rgb24(color) => Some(color.clone()),
            }),
    })
}

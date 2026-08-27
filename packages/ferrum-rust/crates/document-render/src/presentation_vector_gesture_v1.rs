//! Renderer-owned vector geometry and document-owned vector transition admission.

use ferrum_document::{
    AuthoringCapabilityV1, CreatePresentationVectorV1, DocumentFenceV1, DocumentSession,
    GeometricLineWidthV1, PresentationAppearanceV1, PresentationGesturePoint2V1,
    PresentationVectorCreateKindV1, Rgb24V1, SessionOperation, SessionOperationTransitionRequestV1,
    SessionOperationV1, TransitionAuthorizationV1, TransparentOrRgb24V1,
};
use ferrum_render::{RenderPaintV3, Rgb24};
use thiserror::Error;

use super::require_fence;

pub const PRESENTATION_VECTOR_MAXIMUM_EXTENT_PT_V1: f64 = 20_000.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PresentationVectorKindV1 {
    Line,
    Rectangle,
    Square,
    Oval,
    Circle,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PresentationVectorAppearanceV1 {
    stroke_paint: RenderPaintV3,
    stroke_width: GeometricLineWidthV1,
    fill_paint: Option<RenderPaintV3>,
}
impl PresentationVectorAppearanceV1 {
    #[must_use]
    pub const fn stroke_paint(&self) -> &RenderPaintV3 {
        &self.stroke_paint
    }
    #[must_use]
    pub fn stroke_width(&self) -> f64 {
        self.stroke_width.value()
    }
    #[must_use]
    pub fn fill_paint(&self) -> Option<&RenderPaintV3> {
        self.fill_paint.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PresentationVectorOverlayV1 {
    Line {
        start: PresentationGesturePoint2V1,
        end: PresentationGesturePoint2V1,
        appearance: PresentationVectorAppearanceV1,
    },
    Box {
        kind: PresentationVectorKindV1,
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        appearance: PresentationVectorAppearanceV1,
    },
}
impl PresentationVectorOverlayV1 {
    #[must_use]
    pub fn appearance(&self) -> &PresentationVectorAppearanceV1 {
        match self {
            Self::Line { appearance, .. } | Self::Box { appearance, .. } => appearance,
        }
    }
}

#[derive(Debug)]
pub struct PresentationVectorGestureV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    kind: PresentationVectorKindV1,
    start: PresentationGesturePoint2V1,
    appearance: PresentationVectorAppearanceV1,
}
#[derive(Clone, Debug)]
pub struct PresentationVectorPreviewV1 {
    fence: DocumentFenceV1,
    kind: PresentationVectorKindV1,
    start: PresentationGesturePoint2V1,
    appearance: PresentationVectorAppearanceV1,
    end: PresentationGesturePoint2V1,
    overlay: PresentationVectorOverlayV1,
}
impl PresentationVectorPreviewV1 {
    #[must_use]
    pub const fn overlay(&self) -> &PresentationVectorOverlayV1 {
        &self.overlay
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PresentationVectorGestureCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    Consumed,
    InvalidPoint,
    DegenerateGeometry,
    UnsupportedKind,
    UnrenderableStandard,
    RenderPreparation,
    SessionConflict,
    ResourceExhausted,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PresentationVectorGestureRecoveryV1 {
    DocumentUnchanged,
    RefreshAndRestart,
    ChangeGeometry,
    ChooseSupportedAppearance,
    ReduceRequest,
}
#[derive(Clone, Debug, Error, PartialEq)]
#[non_exhaustive]
pub enum PresentationVectorGestureErrorV1 {
    #[error("presentation vector gesture snapshot is stale")]
    StaleSnapshot,
    #[error("presentation vector gesture belongs to another document session")]
    ForeignSession,
    #[error("presentation vector preview belongs to another gesture")]
    MismatchedPreview,
    #[error("presentation vector gesture was already committed")]
    Consumed,
    #[error("presentation vector point is not finite")]
    InvalidPoint,
    #[error("presentation vector requires nonzero finite geometry within the V1 extent")]
    DegenerateGeometry,
    #[error("presentation vector kind is not supported by this V1 gesture")]
    UnsupportedKind,
    #[error("drawing standard cannot produce a trustworthy vector appearance")]
    UnrenderableStandard,
    #[error("presentation vector candidate could not be rendered for preview")]
    RenderPreparation,
    #[error("presentation vector commit was rejected by the document session")]
    SessionConflict,
    #[error("presentation vector request exceeds an allocation bound")]
    ResourceExhausted,
}
impl PresentationVectorGestureErrorV1 {
    #[must_use]
    pub const fn category(&self) -> PresentationVectorGestureCategoryV1 {
        match self {
            Self::StaleSnapshot => PresentationVectorGestureCategoryV1::StaleSnapshot,
            Self::ForeignSession => PresentationVectorGestureCategoryV1::ForeignSession,
            Self::MismatchedPreview => PresentationVectorGestureCategoryV1::MismatchedPreview,
            Self::Consumed => PresentationVectorGestureCategoryV1::Consumed,
            Self::InvalidPoint => PresentationVectorGestureCategoryV1::InvalidPoint,
            Self::DegenerateGeometry => PresentationVectorGestureCategoryV1::DegenerateGeometry,
            Self::UnsupportedKind => PresentationVectorGestureCategoryV1::UnsupportedKind,
            Self::UnrenderableStandard => PresentationVectorGestureCategoryV1::UnrenderableStandard,
            Self::RenderPreparation => PresentationVectorGestureCategoryV1::RenderPreparation,
            Self::SessionConflict => PresentationVectorGestureCategoryV1::SessionConflict,
            Self::ResourceExhausted => PresentationVectorGestureCategoryV1::ResourceExhausted,
        }
    }
    #[must_use]
    pub const fn recovery(&self) -> PresentationVectorGestureRecoveryV1 {
        match self {
            Self::StaleSnapshot
            | Self::ForeignSession
            | Self::MismatchedPreview
            | Self::Consumed
            | Self::SessionConflict => PresentationVectorGestureRecoveryV1::RefreshAndRestart,
            Self::InvalidPoint | Self::RenderPreparation => {
                PresentationVectorGestureRecoveryV1::DocumentUnchanged
            }
            Self::DegenerateGeometry => PresentationVectorGestureRecoveryV1::ChangeGeometry,
            Self::UnsupportedKind | Self::UnrenderableStandard => {
                PresentationVectorGestureRecoveryV1::ChooseSupportedAppearance
            }
            Self::ResourceExhausted => PresentationVectorGestureRecoveryV1::ReduceRequest,
        }
    }
}

pub fn begin_presentation_vector_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    kind: PresentationVectorKindV1,
    start: PresentationGesturePoint2V1,
) -> Result<PresentationVectorGestureV1, PresentationVectorGestureErrorV1> {
    require_fence(session, fence).map_err(|_| PresentationVectorGestureErrorV1::StaleSnapshot)?;
    Ok(PresentationVectorGestureV1 {
        capability: session.issue_authoring_capability_v1(),
        fence,
        kind,
        start,
        appearance: resolve_appearance(session, fence)?,
    })
}

fn resolve_appearance(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<PresentationVectorAppearanceV1, PresentationVectorGestureErrorV1> {
    let observation = session
        .observe(fence.revision())
        .map_err(|_| PresentationVectorGestureErrorV1::UnrenderableStandard)?;
    let standard = observation.projection().drawing_standard();
    let stroke_paint = standard
        .and_then(|value| value.line_color())
        .map(authored_paint)
        .unwrap_or_else(RenderPaintV3::document_foreground);
    let stroke_width = standard
        .and_then(|value| value.line_width())
        .map_or(1.0, |value| value.value());
    let stroke_width = GeometricLineWidthV1::new(stroke_width)
        .ok_or(PresentationVectorGestureErrorV1::UnrenderableStandard)?;
    let fill_paint = standard
        .and_then(|value| value.area_color())
        .and_then(|value| match value {
            TransparentOrRgb24V1::Transparent => None,
            TransparentOrRgb24V1::Rgb24(color) => Some(authored_paint(color)),
        });
    Ok(PresentationVectorAppearanceV1 {
        stroke_paint,
        stroke_width,
        fill_paint,
    })
}
pub fn preview_presentation_vector_gesture_v1(
    session: &DocumentSession,
    gesture: &PresentationVectorGestureV1,
    raw_end: PresentationGesturePoint2V1,
) -> Result<PresentationVectorPreviewV1, PresentationVectorGestureErrorV1> {
    require_fence(session, gesture.fence)
        .map_err(|_| PresentationVectorGestureErrorV1::StaleSnapshot)?;
    let mut dx = raw_end.x() - gesture.start.x();
    let mut dy = raw_end.y() - gesture.start.y();
    if !dx.is_finite()
        || !dy.is_finite()
        || dx.abs() > PRESENTATION_VECTOR_MAXIMUM_EXTENT_PT_V1
        || dy.abs() > PRESENTATION_VECTOR_MAXIMUM_EXTENT_PT_V1
    {
        return Err(PresentationVectorGestureErrorV1::DegenerateGeometry);
    }
    if matches!(
        gesture.kind,
        PresentationVectorKindV1::Square | PresentationVectorKindV1::Circle
    ) {
        let side = dx.abs().min(dy.abs());
        dx = dx.signum() * side;
        dy = dy.signum() * side;
    }
    if (dx == 0.0 && dy == 0.0)
        || (!matches!(gesture.kind, PresentationVectorKindV1::Line) && (dx == 0.0 || dy == 0.0))
    {
        return Err(PresentationVectorGestureErrorV1::DegenerateGeometry);
    }
    let end = PresentationGesturePoint2V1::new(gesture.start.x() + dx, gesture.start.y() + dy)
        .map_err(|_| PresentationVectorGestureErrorV1::InvalidPoint)?;
    let overlay = if gesture.kind == PresentationVectorKindV1::Line {
        PresentationVectorOverlayV1::Line {
            start: gesture.start,
            end,
            appearance: gesture.appearance.clone(),
        }
    } else {
        PresentationVectorOverlayV1::Box {
            kind: gesture.kind,
            left: gesture.start.x().min(end.x()),
            top: gesture.start.y().min(end.y()),
            right: gesture.start.x().max(end.x()),
            bottom: gesture.start.y().max(end.y()),
            appearance: gesture.appearance.clone(),
        }
    };
    Ok(PresentationVectorPreviewV1 {
        fence: gesture.fence,
        kind: gesture.kind,
        start: gesture.start,
        appearance: gesture.appearance.clone(),
        end,
        overlay,
    })
}
pub fn resolve_presentation_vector_gesture_v1(
    session: &DocumentSession,
    gesture: PresentationVectorGestureV1,
    preview: PresentationVectorPreviewV1,
) -> Result<SessionOperationTransitionRequestV1, PresentationVectorGestureErrorV1> {
    if gesture.fence != preview.fence
        || gesture.kind != preview.kind
        || gesture.start != preview.start
        || gesture.appearance != preview.appearance
    {
        return Err(PresentationVectorGestureErrorV1::MismatchedPreview);
    }
    require_fence(session, gesture.fence)
        .map_err(|_| PresentationVectorGestureErrorV1::StaleSnapshot)?;
    Ok(SessionOperationTransitionRequestV1::new(
        gesture.fence.revision(),
        SessionOperation::V1(SessionOperationV1::CreatePresentationVectorV1(
            CreatePresentationVectorV1::new(
                vector_kind(gesture.kind),
                gesture.start,
                preview.end,
                PresentationAppearanceV1::new(
                    document_rgb(&gesture.appearance.stroke_paint),
                    gesture.appearance.stroke_width,
                    gesture.appearance.fill_paint.as_ref().map(document_rgb),
                ),
            ),
        )),
        TransitionAuthorizationV1::authoring_capability(gesture.capability),
    ))
}

fn vector_kind(kind: PresentationVectorKindV1) -> PresentationVectorCreateKindV1 {
    match kind {
        PresentationVectorKindV1::Line => PresentationVectorCreateKindV1::Line,
        PresentationVectorKindV1::Rectangle => PresentationVectorCreateKindV1::Rectangle,
        PresentationVectorKindV1::Square => PresentationVectorCreateKindV1::Square,
        PresentationVectorKindV1::Oval => PresentationVectorCreateKindV1::Oval,
        PresentationVectorKindV1::Circle => PresentationVectorCreateKindV1::Circle,
    }
}

fn authored_paint(value: &Rgb24V1) -> RenderPaintV3 {
    let rgb = value
        .as_str()
        .strip_prefix('#')
        .expect("document RGB is hash-prefixed");
    RenderPaintV3::authored_rgb24(Rgb24::new(rgb).expect("validated document RGB"))
}

fn document_rgb(value: &RenderPaintV3) -> Rgb24V1 {
    Rgb24V1::new(format!("#{}", value.export_rgb().as_str()))
        .expect("resolved render RGB is valid document RGB")
}

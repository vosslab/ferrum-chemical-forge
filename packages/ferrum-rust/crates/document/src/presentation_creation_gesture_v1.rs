//! Rust-owned, revision-fenced straight normal-arrow authoring geometry.

use crate::{
    DocumentFenceV1, PersistentId, Point3V1, PresentationRecordKindV1, PresentationRootSelectorV1,
    SessionOperationResultV1,
};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

pub const ARROW_MINIMUM_LENGTH_PT_V1: f64 = 2.0;
pub const ARROW_MAXIMUM_LENGTH_PT_V1: f64 = 20_000.0;
pub const ARROW_DEFAULT_WIDTH_V1: f64 = 1.0;
pub const ARROW_DEFAULT_COLOR_V1: &str = "#000000";
const HEAD_LENGTH: f64 = 8.0;
const HEAD_HALF_WIDTH: f64 = 3.0;
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PresentationGestureSessionOriginV1(u64);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PresentationGestureCapabilityV1 {
    origin: PresentationGestureSessionOriginV1,
    nonce: u64,
}
impl PresentationGestureSessionOriginV1 {
    pub(crate) fn issue() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
    pub(crate) fn issue_gesture(self) -> PresentationGestureCapabilityV1 {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        PresentationGestureCapabilityV1 {
            origin: self,
            nonce: NEXT.fetch_add(1, Ordering::Relaxed),
        }
    }
}
impl PresentationGestureCapabilityV1 {
    pub(crate) fn belongs_to(self, origin: PresentationGestureSessionOriginV1) -> bool {
        self.origin == origin
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationCreationGestureV1 {
    pub(crate) capability: PresentationGestureCapabilityV1,
    pub(crate) fence: DocumentFenceV1,
    pub(crate) kind: PresentationGestureKindV1,
    pub(crate) start: PresentationGesturePoint2V1,
    pub(crate) style: PresentationGestureStyleV1,
    pub(crate) snap: PresentationGestureSnapPolicyV1,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresentationGestureBoundsV1 {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}
impl PresentationGestureBoundsV1 {
    #[must_use]
    pub const fn left(self) -> f64 {
        self.left
    }
    #[must_use]
    pub const fn top(self) -> f64 {
        self.top
    }
    #[must_use]
    pub const fn right(self) -> f64 {
        self.right
    }
    #[must_use]
    pub const fn bottom(self) -> f64 {
        self.bottom
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationGestureOverlayV1 {
    kind: PresentationGestureKindV1,
    start: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
    geometry: PresentationGestureOverlayGeometryV1,
    bounds: PresentationGestureBoundsV1,
    width: f64,
    color: String,
}
/// Closed, Rust-issued display geometry for an Arrow creation preview.
#[derive(Clone, Debug, PartialEq)]
pub enum PresentationGestureOverlayGeometryV1 {
    Normal {
        axis: [PresentationGesturePoint2V1; 2],
        heads: Vec<[PresentationGesturePoint2V1; 3]>,
    },
    Equilibrium {
        axes: [[PresentationGesturePoint2V1; 2]; 2],
        heads: [[PresentationGesturePoint2V1; 4]; 2],
    },
}
impl PresentationGestureOverlayV1 {
    #[must_use]
    pub const fn kind(&self) -> PresentationGestureKindV1 {
        self.kind
    }
    #[must_use]
    pub const fn start(&self) -> PresentationGesturePoint2V1 {
        self.start
    }
    #[must_use]
    pub const fn end(&self) -> PresentationGesturePoint2V1 {
        self.end
    }
    #[must_use]
    pub const fn geometry(&self) -> &PresentationGestureOverlayGeometryV1 {
        &self.geometry
    }

    #[must_use]
    pub const fn bounds(&self) -> PresentationGestureBoundsV1 {
        self.bounds
    }
    #[must_use]
    pub const fn width(&self) -> f64 {
        self.width
    }
    #[must_use]
    pub fn color(&self) -> &str {
        &self.color
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationCreationPreviewV1 {
    pub(crate) gesture: PresentationCreationGestureV1,
    pub(crate) end: PresentationGesturePoint2V1,
    overlay: Option<PresentationGestureOverlayV1>,
}
impl PresentationCreationPreviewV1 {
    /// Return Arrow-only disposable geometry, when this gesture has any.
    ///
    /// Standard Plus placement deliberately has no document-layer geometry.
    /// The API facade alone derives its preview from the verified renderer.
    #[must_use]
    pub fn overlay(&self) -> Option<&PresentationGestureOverlayV1> {
        self.overlay.as_ref()
    }
}
#[derive(Clone, Debug)]
pub struct CommittedPresentationGestureV1 {
    root: PresentationRootSelectorV1,
    result: SessionOperationResultV1,
}
impl CommittedPresentationGestureV1 {
    #[must_use]
    pub fn root(&self) -> &PresentationRootSelectorV1 {
        &self.root
    }
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
    pub(crate) fn new(
        kind: PresentationGestureKindV1,
        id: PersistentId,
        result: SessionOperationResultV1,
    ) -> Self {
        Self {
            root: PresentationRootSelectorV1::new(
                id.as_str(),
                match kind {
                    PresentationGestureKindV1::StraightNormalArrow => {
                        PresentationRecordKindV1::Arrow
                    }
                    PresentationGestureKindV1::StraightEquilibriumArrow => {
                        PresentationRecordKindV1::Arrow
                    }
                    PresentationGestureKindV1::Plus => PresentationRecordKindV1::Plus,
                },
            )
            .expect("generated ID valid"),
            result,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationGestureCategoryV1 {
    StaleRevision,
    StaleDigest,
    ForeignSession,
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
        return Ok(PresentationCreationPreviewV1 {
            end: gesture.start,
            overlay: None,
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
    let minimum_length = match gesture.kind {
        PresentationGestureKindV1::StraightEquilibriumArrow => {
            crate::equilibrium_arrow_geometry_v1::EQUILIBRIUM_MINIMUM_LENGTH_PT_V1
        }
        _ => ARROW_MINIMUM_LENGTH_PT_V1,
    };
    if length < minimum_length {
        return Err(PresentationGestureErrorV1::BelowMinimumLength);
    }
    if length > ARROW_MAXIMUM_LENGTH_PT_V1 {
        return Err(PresentationGestureErrorV1::ExceedsGeometryLimit);
    }
    if gesture.kind == PresentationGestureKindV1::StraightEquilibriumArrow {
        let source_start =
            Point3V1::new(gesture.start.x, gesture.start.y, 0.0).expect("finite gesture point");
        let source_end = Point3V1::new(end.x, end.y, 0.0).expect("finite gesture point");
        let issued = crate::equilibrium_arrow_geometry_v1::geometry(source_start, source_end)
            .map_err(|_| PresentationGestureErrorV1::BelowMinimumLength)?;
        let point2 = |point: Point3V1| PresentationGesturePoint2V1 {
            x: point.x(),
            y: point.y(),
        };
        let axes = issued.axes.map(|axis| axis.map(point2));
        let heads = issued.heads.map(|head| head.map(point2));
        let all = axes
            .into_iter()
            .flatten()
            .chain(heads.into_iter().flatten())
            .collect::<Vec<_>>();
        let half = ARROW_DEFAULT_WIDTH_V1 / 2.0;
        let bounds = PresentationGestureBoundsV1 {
            left: all.iter().map(|p| p.x).fold(f64::INFINITY, f64::min) - half,
            top: all.iter().map(|p| p.y).fold(f64::INFINITY, f64::min) - half,
            right: all.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + half,
            bottom: all.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max) + half,
        };
        return Ok(PresentationCreationPreviewV1 {
            overlay: Some(PresentationGestureOverlayV1 {
                kind: gesture.kind,
                start: gesture.start,
                end,
                geometry: PresentationGestureOverlayGeometryV1::Equilibrium { axes, heads },
                bounds,
                width: ARROW_DEFAULT_WIDTH_V1,
                color: ARROW_DEFAULT_COLOR_V1.to_owned(),
            }),
            gesture,
            end,
        });
    }
    let PresentationGestureStyleV1::Normal(style) = gesture.style else {
        return Err(PresentationGestureErrorV1::InvalidGestureStyle);
    };
    let ux = dx / length;
    let uy = dy / length;
    let px = -uy;
    let py = ux;
    let mut heads = Vec::new();
    let mut axis_start = gesture.start;
    let mut axis_end = end;
    if style.start_head {
        axis_start = point(
            gesture.start.x + ux * HEAD_LENGTH,
            gesture.start.y + uy * HEAD_LENGTH,
        );
        heads.extend([
            gesture.start,
            point(
                gesture.start.x + ux * HEAD_LENGTH + px * HEAD_HALF_WIDTH,
                gesture.start.y + uy * HEAD_LENGTH + py * HEAD_HALF_WIDTH,
            ),
            point(
                gesture.start.x + ux * HEAD_LENGTH - px * HEAD_HALF_WIDTH,
                gesture.start.y + uy * HEAD_LENGTH - py * HEAD_HALF_WIDTH,
            ),
        ]);
    }
    if style.end_head {
        axis_end = point(end.x - ux * HEAD_LENGTH, end.y - uy * HEAD_LENGTH);
        heads.extend([
            end,
            point(
                end.x - ux * HEAD_LENGTH + px * HEAD_HALF_WIDTH,
                end.y - uy * HEAD_LENGTH + py * HEAD_HALF_WIDTH,
            ),
            point(
                end.x - ux * HEAD_LENGTH - px * HEAD_HALF_WIDTH,
                end.y - uy * HEAD_LENGTH - py * HEAD_HALF_WIDTH,
            ),
        ]);
    }
    let points = std::iter::once(gesture.start)
        .chain(std::iter::once(end))
        .chain(heads.iter().copied())
        .collect::<Vec<_>>();
    let half = ARROW_DEFAULT_WIDTH_V1 / 2.0;
    let bounds = PresentationGestureBoundsV1 {
        left: points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min) - half,
        top: points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min) - half,
        right: points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max) + half,
        bottom: points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max) + half,
    };
    Ok(PresentationCreationPreviewV1 {
        overlay: Some(PresentationGestureOverlayV1 {
            kind: PresentationGestureKindV1::StraightNormalArrow,
            start: gesture.start,
            end,
            geometry: PresentationGestureOverlayGeometryV1::Normal {
                axis: [axis_start, axis_end],
                heads: heads
                    .chunks_exact(3)
                    .map(|head| [head[0], head[1], head[2]])
                    .collect(),
            },
            bounds,
            width: ARROW_DEFAULT_WIDTH_V1,
            color: ARROW_DEFAULT_COLOR_V1.to_owned(),
        }),
        gesture,
        end,
    })
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
    fn snap_and_overlay_are_backend_owned() {
        let g = PresentationCreationGestureV1 {
            capability: PresentationGestureSessionOriginV1::issue().issue_gesture(),
            fence: DocumentFenceV1::new(0, [0; 32]),
            kind: PresentationGestureKindV1::StraightNormalArrow,
            start: PresentationGesturePoint2V1::new(0.0, 0.0).unwrap(),
            style: PresentationGestureStyleV1::normal(false, true),
            snap: PresentationGestureSnapPolicyV1::new(Some(45), Some(20)).unwrap(),
        };
        let p = preview(g, PresentationGesturePoint2V1::new(8.0, 9.0).unwrap()).unwrap();
        let overlay = p.overlay().expect("Arrow owns geometry");
        assert!((overlay.end().x() - 14.142).abs() < 0.01);
        let PresentationGestureOverlayGeometryV1::Normal { heads, .. } = overlay.geometry() else {
            panic!("normal gesture must issue normal preview geometry");
        };
        assert_eq!(heads.len(), 1);
        assert_eq!(overlay.color(), ARROW_DEFAULT_COLOR_V1);
        assert!(overlay.bounds().right() > overlay.end().x());
    }
}

#[cfg(test)]
mod session_tests {
    use super::*;
    use crate::DocumentSession;

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
        let overlay = preview.overlay().expect("Arrow owns geometry");
        assert_eq!(overlay.color(), "#000000");
        assert_eq!(overlay.width(), 1.0);
        let PresentationGestureOverlayGeometryV1::Normal { heads, .. } = overlay.geometry() else {
            panic!("normal gesture must issue normal preview geometry");
        };
        assert_eq!(heads.len(), 1);
        let committed = session
            .commit_presentation_creation_gesture_v1(&gesture, &preview)
            .expect("commit");
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
        let cdml = committed.result().observation().snapshot().cdml();
        assert!(cdml.contains("width=\"1.0\""));
        assert!(cdml.contains("color=\"#000000\""));
        assert!(cdml.contains("x=\"2.540cm\""));
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
        assert!(preview.overlay().is_none());
        let committed = session
            .commit_presentation_creation_gesture_v1(&gesture, &preview)
            .expect("commit");
        assert_eq!(committed.root().kind(), PresentationRecordKindV1::Plus);
        let cdml = committed.result().observation().snapshot().cdml();
        assert!(cdml.contains("<plus"));
        assert!(cdml.contains("x=\"2.540cm\""));
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
    fn equilibrium_gesture_commits_one_typed_root_with_two_issued_shafts() {
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
        let overlay = preview.overlay().expect("equilibrium owns geometry");
        let PresentationGestureOverlayGeometryV1::Equilibrium { axes, heads } = overlay.geometry()
        else {
            panic!("equilibrium preview must not be normal-arrow shaped");
        };
        assert_eq!(axes.len(), 2);
        assert_eq!(heads.len(), 2);
        assert_eq!(heads[0].len(), 4);
        assert_eq!(heads[1].len(), 4);
        assert_ne!(axes[0][0].y(), axes[1][0].y());
        let committed = session
            .commit_presentation_creation_gesture_v1(&gesture, &preview)
            .expect("commit");
        let cdml = committed.result().observation().snapshot().cdml();
        assert_eq!(cdml.matches("<arrow").count(), 1);
        assert!(cdml.contains("type=\"equilibrium\""));
        assert!(!cdml.contains(" start="));
        assert!(!cdml.contains(" end="));
        assert_eq!(committed.root().kind(), PresentationRecordKindV1::Arrow);
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
        assert_eq!(
            session.preview_presentation_creation_gesture_v1(
                &gesture,
                PresentationGesturePoint2V1::new(19.0, 0.0).expect("end"),
            ),
            Err(PresentationGestureErrorV1::BelowMinimumLength),
        );
        assert_eq!(session.snapshot().expect("unchanged"), before);
    }

    #[test]
    fn presentation_creation_gesture_rejects_foreign_mixed_and_stale_without_mutation() {
        let first = DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("first");
        let mut second =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("second");
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
        let second_gesture = second
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
        assert_eq!(
            second.preview_presentation_creation_gesture_v1(
                &first_gesture,
                PresentationGesturePoint2V1::new(10.0, 0.0).unwrap()
            ),
            Err(PresentationGestureErrorV1::ForeignSession)
        );
        assert!(matches!(
            second.commit_presentation_creation_gesture_v1(&second_gesture, &first_preview),
            Err(PresentationGestureErrorV1::ForeignSession)
        ));
        assert_eq!(second.snapshot().expect("unchanged"), second_snapshot);
    }
}

#[cfg(test)]
mod replay_tests {
    use super::*;
    use crate::DocumentSession;

    #[test]
    fn presentation_creation_gesture_replay_is_stale_and_non_mutating() {
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
        session
            .commit_presentation_creation_gesture_v1(&gesture, &preview)
            .expect("commit");
        let after = session.snapshot().expect("after commit");
        assert!(matches!(
            session.commit_presentation_creation_gesture_v1(&gesture, &preview),
            Err(PresentationGestureErrorV1::StaleRevision)
        ));
        assert_eq!(session.snapshot().expect("replay does not mutate"), after);
    }
}

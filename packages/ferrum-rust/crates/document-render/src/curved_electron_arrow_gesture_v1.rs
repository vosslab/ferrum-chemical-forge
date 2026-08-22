//! Renderer-preflighted, Rust-owned quadratic electron-arrow authoring.

use std::sync::atomic::{AtomicU64, Ordering};

use ferrum_document::{
    DocumentFenceV1, DocumentSession, Point3V1, PresentationGesturePoint2V1,
    PresentationRecordKindV1, PresentationRootSelectorV1, SessionOperationResultV1,
    electron_arrow_geometry_v1,
};
use ferrum_render::{
    DocumentRenderOutcomeV1, compose_document_render_plan_v1,
    document_observation_from_accepted_operation_v1,
};
use thiserror::Error;

const MINIMUM_SPAN_PT: f64 = 2.0;
const MINIMUM_CONTROL_DISTANCE_PT: f64 = 1.0;
const MAXIMUM_EXTENT_PT: f64 = 20_000.0;

#[derive(Clone, Debug)]
pub struct CurvedElectronArrowGestureV1 {
    fence: DocumentFenceV1,
    nonce: u64,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
}

#[derive(Clone, Debug)]
pub struct CurvedElectronArrowPreviewV1 {
    gesture: CurvedElectronArrowGestureV1,
    end: PresentationGesturePoint2V1,
    overlay: CurvedElectronArrowOverlayV1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurvedElectronArrowOverlayV1 {
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
    cubic_control_1: PresentationGesturePoint2V1,
    cubic_control_2: PresentationGesturePoint2V1,
    head: [PresentationGesturePoint2V1; 4],
}

impl CurvedElectronArrowOverlayV1 {
    #[must_use]
    pub const fn start(&self) -> PresentationGesturePoint2V1 {
        self.start
    }
    #[must_use]
    pub const fn control(&self) -> PresentationGesturePoint2V1 {
        self.control
    }
    #[must_use]
    pub const fn end(&self) -> PresentationGesturePoint2V1 {
        self.end
    }
    #[must_use]
    pub const fn cubic_control_1(&self) -> PresentationGesturePoint2V1 {
        self.cubic_control_1
    }
    #[must_use]
    pub const fn cubic_control_2(&self) -> PresentationGesturePoint2V1 {
        self.cubic_control_2
    }
    #[must_use]
    pub const fn head(&self) -> &[PresentationGesturePoint2V1; 4] {
        &self.head
    }
}

impl CurvedElectronArrowPreviewV1 {
    #[must_use]
    pub const fn overlay(&self) -> &CurvedElectronArrowOverlayV1 {
        &self.overlay
    }
}

#[derive(Debug)]
pub struct PreparedCurvedElectronArrowV1 {
    receipt: Option<CurvedElectronArrowReceiptV1>,
    identifier: String,
}

#[derive(Debug)]
struct CurvedElectronArrowReceiptV1 {
    source_fence: DocumentFenceV1,
    candidate: String,
    candidate_digest: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct CommittedCurvedElectronArrowV1 {
    root: PresentationRootSelectorV1,
    result: SessionOperationResultV1,
}

impl CommittedCurvedElectronArrowV1 {
    #[must_use]
    pub const fn root(&self) -> &PresentationRootSelectorV1 {
        &self.root
    }
    #[must_use]
    pub const fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurvedElectronArrowGestureCategoryV1 {
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
    #[error("curved electron-arrow snapshot is stale")]
    StaleSnapshot,
    #[error("curved electron-arrow preview belongs to a different gesture")]
    MismatchedPreview,
    #[error("curved electron-arrow receipt was already consumed")]
    ReplayedGesture,
    #[error("curved electron-arrow point is invalid")]
    InvalidPoint,
    #[error("curved electron-arrow start and end are too close")]
    CollapsedSpan,
    #[error("curved electron-arrow control point is too close to its chord")]
    ControlTooNearChord,
    #[error("curved electron-arrow exceeds the geometry limit")]
    ExceedsGeometryLimit,
    #[error("curved electron-arrow candidate failed renderer preflight")]
    RenderPreparation,
    #[error("curved electron-arrow session transaction failed")]
    SessionConflict,
}

impl CurvedElectronArrowGestureErrorV1 {
    #[must_use]
    pub const fn category(&self) -> CurvedElectronArrowGestureCategoryV1 {
        match self {
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
            Self::StaleSnapshot
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
    require_fence(session, fence)?;
    static NEXT: AtomicU64 = AtomicU64::new(1);
    Ok(CurvedElectronArrowGestureV1 {
        fence,
        nonce: NEXT.fetch_add(1, Ordering::Relaxed),
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
    let overlay = geometry(gesture.start, gesture.control, end)?;
    Ok(CurvedElectronArrowPreviewV1 {
        gesture: gesture.clone(),
        end,
        overlay,
    })
}

pub fn prepare_curved_electron_arrow_gesture_v1(
    session: &mut DocumentSession,
    gesture: &CurvedElectronArrowGestureV1,
    preview: &CurvedElectronArrowPreviewV1,
) -> Result<PreparedCurvedElectronArrowV1, CurvedElectronArrowGestureErrorV1> {
    if gesture.nonce != preview.gesture.nonce {
        return Err(CurvedElectronArrowGestureErrorV1::MismatchedPreview);
    }
    require_fence(session, gesture.fence)?;
    let identifier = next_identifier(session);
    let source = session
        .snapshot()
        .map_err(|_| CurvedElectronArrowGestureErrorV1::SessionConflict)?
        .cdml()
        .to_owned();
    let candidate = append_arrow(
        &source,
        &identifier,
        gesture.start,
        gesture.control,
        preview.end,
    )?;
    ferrum_render_contract::preflight_complete_document_v1(&candidate)
        .map_err(|_| CurvedElectronArrowGestureErrorV1::RenderPreparation)?;
    let candidate_session = DocumentSession::load(&candidate)
        .map_err(|_| CurvedElectronArrowGestureErrorV1::RenderPreparation)?;
    let observation = candidate_session
        .observe(0)
        .map_err(|_| CurvedElectronArrowGestureErrorV1::RenderPreparation)?;
    let render_observation = document_observation_from_accepted_operation_v1(&observation)
        .map_err(|_| CurvedElectronArrowGestureErrorV1::RenderPreparation)?;
    let plan = compose_document_render_plan_v1(&render_observation)
        .map_err(|_| CurvedElectronArrowGestureErrorV1::RenderPreparation)?;
    if plan
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(CurvedElectronArrowGestureErrorV1::RenderPreparation);
    }
    let digest = *candidate_session
        .snapshot()
        .map_err(|_| CurvedElectronArrowGestureErrorV1::RenderPreparation)?
        .digest();
    Ok(PreparedCurvedElectronArrowV1 {
        receipt: Some(CurvedElectronArrowReceiptV1 {
            source_fence: gesture.fence,
            candidate,
            candidate_digest: digest,
        }),
        identifier,
    })
}

pub fn commit_curved_electron_arrow_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedCurvedElectronArrowV1,
) -> Result<CommittedCurvedElectronArrowV1, CurvedElectronArrowGestureErrorV1> {
    let receipt = prepared
        .receipt
        .take()
        .ok_or(CurvedElectronArrowGestureErrorV1::ReplayedGesture)?;
    require_fence(session, receipt.source_fence)?;
    let candidate = DocumentSession::load(&receipt.candidate)
        .map_err(|_| CurvedElectronArrowGestureErrorV1::RenderPreparation)?;
    if *candidate
        .snapshot()
        .map_err(|_| CurvedElectronArrowGestureErrorV1::RenderPreparation)?
        .digest()
        != receipt.candidate_digest
    {
        return Err(CurvedElectronArrowGestureErrorV1::RenderPreparation);
    }
    let result = session
        .commit_complete_cdml_transaction_v1(receipt.source_fence, &receipt.candidate)
        .map_err(|_| CurvedElectronArrowGestureErrorV1::SessionConflict)?;
    let root =
        PresentationRootSelectorV1::new(&prepared.identifier, PresentationRecordKindV1::Arrow)
            .expect("generated electron arrow identifier is valid");
    Ok(CommittedCurvedElectronArrowV1 { root, result })
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

fn geometry(
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
) -> Result<CurvedElectronArrowOverlayV1, CurvedElectronArrowGestureErrorV1> {
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
    let issued = electron_arrow_geometry_v1(point3(start), point3(control), point3(end))
        .map_err(|_| CurvedElectronArrowGestureErrorV1::ControlTooNearChord)?;
    let [_, cubic_control_1, cubic_control_2, _] = *issued.cubic_axis();
    let [tip, left, inner, right] = *issued.head();
    Ok(CurvedElectronArrowOverlayV1 {
        start,
        control,
        end,
        cubic_control_1: point2(cubic_control_1),
        cubic_control_2: point2(cubic_control_2),
        head: [point2(tip), point2(left), point2(inner), point2(right)],
    })
}

fn point3(point: PresentationGesturePoint2V1) -> Point3V1 {
    Point3V1::new(point.x(), point.y(), 0.0).expect("validated finite geometry")
}

fn point2(point: Point3V1) -> PresentationGesturePoint2V1 {
    PresentationGesturePoint2V1::new(point.x(), point.y()).expect("issued finite geometry")
}

fn next_identifier(session: &DocumentSession) -> String {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    loop {
        let identifier = format!("electron-arrow-{}", NEXT.fetch_add(1, Ordering::Relaxed));
        if !session.contains_durable_id_v1(&identifier) {
            return identifier;
        }
    }
}

fn append_arrow(
    source: &str,
    id: &str,
    start: PresentationGesturePoint2V1,
    control: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
) -> Result<String, CurvedElectronArrowGestureErrorV1> {
    let arrow = format!(
        "<arrow id=\"{id}\" type=\"electron\" width=\"1.0\" color=\"#000000\"><point x=\"{}\" y=\"{}\" z=\"0\"/><point x=\"{}\" y=\"{}\" z=\"0\"/><point x=\"{}\" y=\"{}\" z=\"0\"/></arrow>",
        start.x(),
        start.y(),
        control.x(),
        control.y(),
        end.x(),
        end.y()
    );
    if let Some(close) = source.rfind("</cdml") {
        return Ok(format!("{}{}{}", &source[..close], arrow, &source[close..]));
    }
    let close = source
        .rfind("/>")
        .filter(|index| source[index + 2..].trim().is_empty())
        .ok_or(CurvedElectronArrowGestureErrorV1::RenderPreparation)?;
    Ok(format!("{}>{}</cdml>", &source[..close], arrow))
}

#[cfg(test)]
mod tests {
    use super::*;
    const EMPTY: &str = r#"<cdml xmlns="urn:ferrum:cdml" version="26.07"/>"#;
    fn point(x: f64, y: f64) -> PresentationGesturePoint2V1 {
        PresentationGesturePoint2V1::new(x, y).expect("point")
    }
    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().unwrap();
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }
    #[test]
    fn quadratic_arrow_preflights_and_commits_one_semantic_root() {
        let mut session = DocumentSession::load(EMPTY).unwrap();
        let gesture = begin_curved_electron_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(30.0, 20.0),
        )
        .unwrap();
        let preview =
            preview_curved_electron_arrow_gesture_v1(&session, &gesture, point(60.0, 0.0)).unwrap();
        let mut prepared =
            prepare_curved_electron_arrow_gesture_v1(&mut session, &gesture, &preview).unwrap();
        let committed =
            commit_curved_electron_arrow_gesture_v1(&mut session, &mut prepared).unwrap();
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
        assert!(
            committed
                .result()
                .observation()
                .snapshot()
                .cdml()
                .contains("type=\"electron\"")
        );
    }

    #[test]
    fn preview_and_committed_projection_issue_the_same_electron_geometry() {
        let mut session = DocumentSession::load(EMPTY).unwrap();
        let gesture = begin_curved_electron_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(30.0, 20.0),
        )
        .unwrap();
        let preview =
            preview_curved_electron_arrow_gesture_v1(&session, &gesture, point(60.0, 0.0)).unwrap();
        let mut prepared =
            prepare_curved_electron_arrow_gesture_v1(&mut session, &gesture, &preview).unwrap();
        let committed =
            commit_curved_electron_arrow_gesture_v1(&mut session, &mut prepared).unwrap();
        let stack = committed
            .result()
            .observation()
            .projection()
            .presentation_stack();
        let [ferrum_document::PresentationRootProjectionV1::Arrow { arrow }] = stack.roots() else {
            panic!("expected electron arrow");
        };
        let ferrum_document::ArrowDisplayGeometryV1::Electron {
            axis_path, head, ..
        } = arrow.geometry()
        else {
            panic!("expected electron geometry");
        };
        assert_eq!(
            axis_path
                .points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect::<Vec<_>>(),
            [
                preview.overlay().start(),
                preview.overlay().cubic_control_1(),
                preview.overlay().cubic_control_2(),
                preview.overlay().end()
            ]
            .map(|point| (point.x(), point.y()))
            .to_vec(),
        );
        assert_eq!(
            head.points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect::<Vec<_>>(),
            preview
                .overlay()
                .head()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn invalid_geometry_and_replayed_receipts_leave_the_document_unchanged() {
        let mut session = DocumentSession::load(EMPTY).unwrap();
        let gesture = begin_curved_electron_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(20.0, 0.0),
        )
        .unwrap();
        assert!(matches!(
            preview_curved_electron_arrow_gesture_v1(&session, &gesture, point(40.0, 0.0)),
            Err(CurvedElectronArrowGestureErrorV1::ControlTooNearChord)
        ));
        assert_eq!(session.snapshot().unwrap().revision(), 0);
        let gesture = begin_curved_electron_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(20.0, 10.0),
        )
        .unwrap();
        let preview =
            preview_curved_electron_arrow_gesture_v1(&session, &gesture, point(40.0, 0.0)).unwrap();
        let mut prepared =
            prepare_curved_electron_arrow_gesture_v1(&mut session, &gesture, &preview).unwrap();
        commit_curved_electron_arrow_gesture_v1(&mut session, &mut prepared).unwrap();
        let revision = session.snapshot().unwrap().revision();
        assert!(matches!(
            commit_curved_electron_arrow_gesture_v1(&mut session, &mut prepared),
            Err(CurvedElectronArrowGestureErrorV1::ReplayedGesture)
        ));
        assert_eq!(session.snapshot().unwrap().revision(), revision);
    }
}

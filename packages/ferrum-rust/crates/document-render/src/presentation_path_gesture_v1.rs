//! Renderer-preflighted multi-point Polyline and Polygon authoring.

use std::sync::atomic::{AtomicU64, Ordering};

use ferrum_document::{
    DocumentFenceV1, DocumentSession, PresentationPathGestureErrorV1, PresentationPathGestureV1,
    PresentationPathKindV1, PresentationRecordKindV1, PresentationRootSelectorV1,
    SessionOperationResultV1, TransparentOrRgb24V1,
};
use ferrum_render::{
    compose_document_render_plan_v1, document_observation_from_accepted_operation_v1,
    DocumentRenderOutcomeV1, DocumentRenderPlanV1,
};
use thiserror::Error;

use super::{consume, is_consumed, origin, require_fence, BridgeSessionOriginV1};

#[derive(Clone, Debug, PartialEq)]
pub struct PresentationPathAppearanceV1 {
    stroke_color: String,
    stroke_width: f64,
    fill_color: Option<String>,
}
impl PresentationPathAppearanceV1 {
    #[must_use]
    pub fn stroke_color(&self) -> &str {
        &self.stroke_color
    }
    #[must_use]
    pub const fn stroke_width(&self) -> f64 {
        self.stroke_width
    }
    #[must_use]
    pub fn fill_color(&self) -> Option<&str> {
        self.fill_color.as_deref()
    }
}

#[derive(Clone, Debug)]
pub struct PresentationPathRenderGestureV1 {
    origin: BridgeSessionOriginV1,
    nonce: u64,
    fence: DocumentFenceV1,
    kind: PresentationPathKindV1,
    appearance: PresentationPathAppearanceV1,
}
#[derive(Clone, Debug)]
pub struct PresentationPathPreviewV1 {
    gesture: PresentationPathRenderGestureV1,
    path: PresentationPathGestureV1,
}
impl PresentationPathPreviewV1 {
    #[must_use]
    pub fn path(&self) -> &PresentationPathGestureV1 {
        &self.path
    }
    #[must_use]
    pub fn appearance(&self) -> &PresentationPathAppearanceV1 {
        &self.gesture.appearance
    }
}
#[derive(Debug)]
pub struct PreparedPresentationPathV1 {
    receipt: Option<PathRendererReceiptV1>,
    identifier: String,
}
#[derive(Clone, Debug)]
pub struct CommittedPresentationPathV1 {
    root: PresentationRootSelectorV1,
    result: SessionOperationResultV1,
}
impl CommittedPresentationPathV1 {
    #[must_use]
    pub fn root(&self) -> &PresentationRootSelectorV1 {
        &self.root
    }
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationPathRenderCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    ReplayedGesture,
    InvalidGeometry,
    RenderPreparation,
    SessionConflict,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationPathRenderRecoveryV1 {
    RefreshAndRestart,
    ChangeGeometry,
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
            Self::InvalidGeometry(_) => PresentationPathRenderRecoveryV1::ChangeGeometry,
            Self::RenderPreparation => PresentationPathRenderRecoveryV1::DocumentUnchanged,
        }
    }
}

#[derive(Debug)]
struct PathRendererReceiptV1 {
    origin: BridgeSessionOriginV1,
    nonce: u64,
    source_fence: DocumentFenceV1,
    candidate_digest: [u8; 32],
    root_identifier: String,
    root_kind: PresentationRecordKindV1,
    candidate: String,
    contract: ferrum_render_contract::PreflightedDocumentRenderV1,
    plan: DocumentRenderPlanV1,
}

pub fn begin_presentation_path_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    kind: PresentationPathKindV1,
) -> Result<PresentationPathRenderGestureV1, PresentationPathRenderErrorV1> {
    require_fence(session, fence).map_err(|_| PresentationPathRenderErrorV1::StaleSnapshot)?;
    static NEXT: AtomicU64 = AtomicU64::new(1);
    Ok(PresentationPathRenderGestureV1 {
        origin: origin(session),
        nonce: NEXT.fetch_add(1, Ordering::Relaxed),
        fence,
        kind,
        appearance: appearance(session, fence)?,
    })
}

pub fn preview_presentation_path_gesture_v1(
    session: &DocumentSession,
    gesture: &PresentationPathRenderGestureV1,
    points: Vec<ferrum_document::PresentationGesturePoint2V1>,
) -> Result<PresentationPathPreviewV1, PresentationPathRenderErrorV1> {
    if gesture.origin != origin(session) {
        return Err(PresentationPathRenderErrorV1::ForeignSession);
    }
    require_fence(session, gesture.fence)
        .map_err(|_| PresentationPathRenderErrorV1::StaleSnapshot)?;
    let path = PresentationPathGestureV1::new(gesture.kind, points)
        .map_err(PresentationPathRenderErrorV1::InvalidGeometry)?;
    Ok(PresentationPathPreviewV1 {
        gesture: gesture.clone(),
        path,
    })
}

pub fn prepare_presentation_path_gesture_v1(
    session: &mut DocumentSession,
    gesture: &PresentationPathRenderGestureV1,
    preview: &PresentationPathPreviewV1,
) -> Result<PreparedPresentationPathV1, PresentationPathRenderErrorV1> {
    if gesture.origin != origin(session) || preview.gesture.origin != origin(session) {
        return Err(PresentationPathRenderErrorV1::ForeignSession);
    }
    if gesture.nonce != preview.gesture.nonce {
        return Err(PresentationPathRenderErrorV1::MismatchedPreview);
    }
    if is_consumed(gesture.origin, gesture.nonce) {
        return Err(PresentationPathRenderErrorV1::ReplayedGesture);
    }
    require_fence(session, gesture.fence)
        .map_err(|_| PresentationPathRenderErrorV1::StaleSnapshot)?;
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let identifier = loop {
        let id = format!(
            "presentation-path-{}",
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        );
        if !session.contains_durable_id_v1(&id) {
            break id;
        }
    };
    let source = session
        .snapshot()
        .map_err(|_| PresentationPathRenderErrorV1::SessionConflict)?
        .cdml()
        .to_owned();
    let candidate = insert_path(&source, &identifier, preview.path(), preview.appearance())?;
    let contract = ferrum_render_contract::preflight_complete_document_v1(&candidate)
        .map_err(|_| PresentationPathRenderErrorV1::RenderPreparation)?;
    let candidate_session = DocumentSession::load(&candidate)
        .map_err(|_| PresentationPathRenderErrorV1::RenderPreparation)?;
    let observation = candidate_session
        .observe(0)
        .map_err(|_| PresentationPathRenderErrorV1::RenderPreparation)?;
    let render_observation = document_observation_from_accepted_operation_v1(&observation)
        .map_err(|_| PresentationPathRenderErrorV1::RenderPreparation)?;
    let plan = compose_document_render_plan_v1(&render_observation)
        .map_err(|_| PresentationPathRenderErrorV1::RenderPreparation)?;
    if plan
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(PresentationPathRenderErrorV1::RenderPreparation);
    }
    let digest = *candidate_session
        .snapshot()
        .map_err(|_| PresentationPathRenderErrorV1::RenderPreparation)?
        .digest();
    let root_kind = match preview.path().kind() {
        PresentationPathKindV1::Polyline => PresentationRecordKindV1::Polyline,
        PresentationPathKindV1::Polygon => PresentationRecordKindV1::Polygon,
    };
    Ok(PreparedPresentationPathV1 {
        receipt: Some(PathRendererReceiptV1 {
            origin: gesture.origin,
            nonce: gesture.nonce,
            source_fence: gesture.fence,
            candidate_digest: digest,
            root_identifier: identifier.clone(),
            root_kind,
            candidate,
            contract,
            plan,
        }),
        identifier,
    })
}

pub fn commit_presentation_path_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedPresentationPathV1,
) -> Result<CommittedPresentationPathV1, PresentationPathRenderErrorV1> {
    let receipt = prepared
        .receipt
        .take()
        .ok_or(PresentationPathRenderErrorV1::ReplayedGesture)?;
    if receipt.origin != origin(session) {
        prepared.receipt = Some(receipt);
        return Err(PresentationPathRenderErrorV1::ForeignSession);
    }
    if is_consumed(receipt.origin, receipt.nonce) {
        return Err(PresentationPathRenderErrorV1::ReplayedGesture);
    }
    require_fence(session, receipt.source_fence)
        .map_err(|_| PresentationPathRenderErrorV1::StaleSnapshot)?;
    if receipt.root_identifier != prepared.identifier
        || receipt.contract.source() != receipt.candidate
        || receipt
            .plan
            .outcomes()
            .iter()
            .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(PresentationPathRenderErrorV1::RenderPreparation);
    }
    let candidate = DocumentSession::load(&receipt.candidate)
        .map_err(|_| PresentationPathRenderErrorV1::RenderPreparation)?;
    if *candidate
        .snapshot()
        .map_err(|_| PresentationPathRenderErrorV1::RenderPreparation)?
        .digest()
        != receipt.candidate_digest
    {
        return Err(PresentationPathRenderErrorV1::RenderPreparation);
    }
    let result = session
        .commit_complete_cdml_transaction_v1(receipt.source_fence, &receipt.candidate)
        .map_err(|_| PresentationPathRenderErrorV1::SessionConflict)?;
    consume(receipt.origin, receipt.nonce);
    let root = PresentationRootSelectorV1::new(&receipt.root_identifier, receipt.root_kind)
        .expect("prepared receipt contains a valid generated selector");
    Ok(CommittedPresentationPathV1 { root, result })
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
            .map_or("#000000", |value| value.as_str())
            .to_owned(),
        stroke_width: standard
            .and_then(|value| value.line_width())
            .map_or(1.0, |value| value.value()),
        fill_color: standard
            .and_then(|value| value.area_color())
            .and_then(|value| match value {
                TransparentOrRgb24V1::Transparent => None,
                TransparentOrRgb24V1::Rgb24(color) => Some(color.as_str().to_owned()),
            }),
    })
}

fn insert_path(
    source: &str,
    id: &str,
    path: &PresentationPathGestureV1,
    appearance: &PresentationPathAppearanceV1,
) -> Result<String, PresentationPathRenderErrorV1> {
    let points = path
        .points()
        .iter()
        .map(|point| format!("<point x=\"{}\" y=\"{}\" z=\"0\"/>", point.x(), point.y()))
        .collect::<String>();
    let geometry = match path.kind() {
        PresentationPathKindV1::Polyline => format!("<polyline id=\"{id}\" spline=\"0\" line_color=\"{}\" width=\"{}\">{points}</polyline>", appearance.stroke_color, appearance.stroke_width),
        PresentationPathKindV1::Polygon => match appearance.fill_color.as_deref() {
            Some(fill_color) => format!("<polygon id=\"{id}\" line_color=\"{}\" width=\"{}\" area_color=\"{fill_color}\">{points}</polygon>", appearance.stroke_color, appearance.stroke_width),
            None => format!("<polygon id=\"{id}\" line_color=\"{}\" width=\"{}\">{points}</polygon>", appearance.stroke_color, appearance.stroke_width),
        },
    };
    if let Some(close) = source.rfind("</cdml") {
        return Ok(format!(
            "{}{}{}",
            &source[..close],
            geometry,
            &source[close..]
        ));
    }
    let self_close = source
        .rfind("/>")
        .filter(|index| source[index + 2..].trim().is_empty())
        .ok_or(PresentationPathRenderErrorV1::RenderPreparation)?;
    Ok(format!("{}>{}</cdml>", &source[..self_close], geometry))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }
    fn point(x: f64, y: f64) -> ferrum_document::PresentationGesturePoint2V1 {
        ferrum_document::PresentationGesturePoint2V1::new(x, y).expect("point")
    }
    #[test]
    fn path_preflight_commits_one_canonical_polygon() {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"/>")
                .expect("session");
        let gesture = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            PresentationPathKindV1::Polygon,
        )
        .expect("gesture");
        let preview = preview_presentation_path_gesture_v1(
            &session,
            &gesture,
            vec![point(0.0, 0.0), point(20.0, 0.0), point(0.0, 10.0)],
        )
        .expect("preview");
        let mut prepared = prepare_presentation_path_gesture_v1(&mut session, &gesture, &preview)
            .expect("prepared");
        let commit =
            commit_presentation_path_gesture_v1(&mut session, &mut prepared).expect("commit");
        assert_eq!(commit.result().observation().snapshot().revision(), 1);
        assert_eq!(commit.root().kind(), PresentationRecordKindV1::Polygon);
    }

    #[test]
    fn polyline_receipt_selects_the_new_polyline_after_a_polygon() {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"/>")
                .expect("session");
        let polygon = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            PresentationPathKindV1::Polygon,
        )
        .expect("polygon gesture");
        let polygon_preview = preview_presentation_path_gesture_v1(
            &session,
            &polygon,
            vec![point(0.0, 0.0), point(20.0, 0.0), point(0.0, 10.0)],
        )
        .expect("polygon preview");
        let mut polygon_prepared =
            prepare_presentation_path_gesture_v1(&mut session, &polygon, &polygon_preview)
                .expect("polygon prepared");
        commit_presentation_path_gesture_v1(&mut session, &mut polygon_prepared)
            .expect("polygon commit");

        let polyline = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            PresentationPathKindV1::Polyline,
        )
        .expect("polyline gesture");
        let polyline_preview = preview_presentation_path_gesture_v1(
            &session,
            &polyline,
            vec![point(30.0, 0.0), point(50.0, 10.0)],
        )
        .expect("polyline preview");
        let mut polyline_prepared =
            prepare_presentation_path_gesture_v1(&mut session, &polyline, &polyline_preview)
                .expect("polyline prepared");
        let commit = commit_presentation_path_gesture_v1(&mut session, &mut polyline_prepared)
            .expect("polyline commit");

        assert_eq!(commit.root().kind(), PresentationRecordKindV1::Polyline);
        assert!(commit
            .result()
            .observation()
            .snapshot()
            .cdml()
            .contains(&format!(
                "<polyline id=\"{}\"",
                commit.root().presentation_id().as_str()
            )));
    }
}

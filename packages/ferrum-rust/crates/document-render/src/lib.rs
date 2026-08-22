//! Renderer-preflighted ownership for vector authoring transactions.
//!
//! `ferrum-document` owns generic CDML state transitions. This crate owns every
//! vector-specific capability, candidate, renderer admission proof, and receipt,
//! so a vector gesture cannot reach a generic commit without complete rendering.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use ferrum_document::{
    DocumentFenceV1, DocumentSession, PresentationGesturePoint2V1, PresentationRecordKindV1,
    PresentationRootSelectorV1, SessionOperationResultV1, TransparentOrRgb24V1,
};
use ferrum_render::{
    DocumentRenderOutcomeV1, DocumentRenderPlanV1, compose_document_render_plan_v1,
    document_observation_from_accepted_operation_v1,
};
use thiserror::Error;

mod catalog_placement_v2;
mod curved_electron_arrow_gesture_v1;
mod reaction_gesture_v1;
mod reaction_lifecycle_v1;
mod reaction_observation_v1;
mod reaction_translation_v1;
mod presentation_path_gesture_v1;
mod render_interaction_v1;

pub use catalog_placement_v2::{
    CatalogPlacementCategoryV2, CatalogPlacementErrorV2, CatalogPlacementGestureV2,
    CatalogPlacementPreviewV2, CatalogPlacementRecoveryV2, CommittedCatalogPlacementV2,
    PreparedCatalogPlacementV2, begin_catalog_placement_v2, cancel_catalog_placement_gesture_v2,
    commit_catalog_placement_v2, prepare_catalog_placement_v2, preview_catalog_placement_v2,
    release_catalog_placement_preview_v2,
};
pub use curved_electron_arrow_gesture_v1::{
    CommittedCurvedElectronArrowV1, CurvedElectronArrowGestureCategoryV1,
    CurvedElectronArrowGestureErrorV1, CurvedElectronArrowGestureRecoveryV1,
    CurvedElectronArrowGestureV1, CurvedElectronArrowOverlayV1,
    CurvedElectronArrowPreviewV1, PreparedCurvedElectronArrowV1,
    begin_curved_electron_arrow_gesture_v1, commit_curved_electron_arrow_gesture_v1,
    prepare_curved_electron_arrow_gesture_v1, preview_curved_electron_arrow_gesture_v1,
};
pub use reaction_gesture_v1::{
    CommittedReactionV1, PreparedReactionV1, ReactionCreateRequestV1, ReactionGestureCategoryV1,
    ReactionGestureErrorV1, ReactionGestureRecoveryV1, ReactionGestureV1,
    begin_reaction_gesture_v1, commit_reaction_gesture_v1, prepare_reaction_gesture_v1,
};
pub use reaction_lifecycle_v1::{
    CommittedReactionLifecycleV1, PreparedReactionLifecycleV1, ReactionLifecycleGestureV1,
    ReactionMembershipPatchRequestV1, begin_reaction_definition_delete_v1,
    begin_reaction_membership_patch_v1, commit_reaction_lifecycle_v1,
    prepare_reaction_lifecycle_v1,
};
pub use reaction_observation_v1::{
    ReactionDefinitionDispositionV1, ReactionListObservationV1, ReactionMemberObservationV1,
    ReactionObservationV1, ReactionSelectionV1,
};
pub use reaction_translation_v1::{
    CommittedReactionTranslationV1, PreparedReactionTranslationV1, ReactionTranslationGestureV1,
    ReactionTranslationPreviewV1, begin_reaction_translation_v1, commit_reaction_translation_v1,
    prepare_reaction_translation_v1, preview_reaction_translation_v1,
};
pub use presentation_path_gesture_v1::{
    CommittedPresentationPathV1, PresentationPathAppearanceV1, PresentationPathPreviewV1,
    PresentationPathRenderCategoryV1, PresentationPathRenderErrorV1, PresentationPathRenderGestureV1,
    PresentationPathRenderRecoveryV1, PreparedPresentationPathV1, begin_presentation_path_gesture_v1,
    commit_presentation_path_gesture_v1, prepare_presentation_path_gesture_v1,
    preview_presentation_path_gesture_v1,
};
pub use render_interaction_v1::{
    CommittedRenderInteractionTranslationV1, CommittedStructureDeletionV1,
    ReactionAuthoringChoiceAvailabilityV1, ReactionAuthoringChoiceKindV1,
    ReactionAuthoringChoiceV1, ReactionAuthoringChoicesV1, ReactionAuthoringExclusionReasonV1,
    ReactionAuthoringExclusionRecoveryV1, ReactionAuthoringExclusionV1, RenderInteractionAxisV1,
    RenderInteractionBoundsV1, RenderInteractionErrorV1, RenderInteractionExclusionReasonV1,
    RenderInteractionExclusionV1, RenderInteractionGridSnapPolicyV1, RenderInteractionModifierV1,
    RenderInteractionObservationV1, RenderInteractionQueryV1, RenderInteractionRootV1,
    RenderInteractionSelectionV1, RenderInteractionSessionV1, RenderInteractionSnapV1,
    RenderInteractionTranslationGestureV1, RenderInteractionTranslationPreviewV1,
    StructureInteractionObservationV1, StructureInteractionQueryV1,
    StructureInteractionSelectionV1, StructureInteractionTargetV1, StructureTargetKindV1,
};

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
    stroke_color: String,
    stroke_width: f64,
    fill_color: Option<String>,
}
impl PresentationVectorAppearanceV1 {
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

#[derive(Clone, Debug)]
pub struct PresentationVectorGestureV1 {
    origin: BridgeSessionOriginV1,
    nonce: u64,
    fence: DocumentFenceV1,
    kind: PresentationVectorKindV1,
    start: PresentationGesturePoint2V1,
    appearance: PresentationVectorAppearanceV1,
}
#[derive(Clone, Debug)]
pub struct PresentationVectorPreviewV1 {
    gesture: PresentationVectorGestureV1,
    end: PresentationGesturePoint2V1,
    overlay: PresentationVectorOverlayV1,
}
impl PresentationVectorPreviewV1 {
    #[must_use]
    pub const fn overlay(&self) -> &PresentationVectorOverlayV1 {
        &self.overlay
    }
}
#[derive(Debug)]
pub struct PreparedPresentationVectorV1 {
    receipt: Option<RendererPreflightReceiptV1>,
    kind: PresentationVectorKindV1,
    identifier: String,
}
#[derive(Clone, Debug)]
pub struct CommittedPresentationVectorV1 {
    root: PresentationRootSelectorV1,
    result: SessionOperationResultV1,
}
impl CommittedPresentationVectorV1 {
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
#[non_exhaustive]
pub enum PresentationVectorGestureCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    ReplayedGesture,
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
    ReplayedGesture,
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
            Self::ReplayedGesture => PresentationVectorGestureCategoryV1::ReplayedGesture,
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
            | Self::ReplayedGesture
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct BridgeSessionOriginV1(u64);

/// Nonconstructible proof that one exact candidate completed both preflight
/// stages with no renderer exclusion. It remains entirely bridge-private.
#[derive(Debug)]
struct RendererPreflightReceiptV1 {
    origin: BridgeSessionOriginV1,
    nonce: u64,
    source_fence: DocumentFenceV1,
    candidate_revision: u64,
    candidate_digest: [u8; 32],
    root_identifier: String,
    candidate: String,
    contract: ferrum_render_contract::PreflightedDocumentRenderV1,
    plan: DocumentRenderPlanV1,
}

#[must_use]
fn origin(session: &DocumentSession) -> BridgeSessionOriginV1 {
    BridgeSessionOriginV1(session.bridge_session_origin_v1())
}

fn consumed() -> &'static Mutex<HashSet<(BridgeSessionOriginV1, u64)>> {
    static CONSUMED: OnceLock<Mutex<HashSet<(BridgeSessionOriginV1, u64)>>> = OnceLock::new();
    CONSUMED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn is_consumed(origin: BridgeSessionOriginV1, nonce: u64) -> bool {
    consumed()
        .lock()
        .expect("bridge consumed-capability lock is not poisoned")
        .contains(&(origin, nonce))
}

fn consume(origin: BridgeSessionOriginV1, nonce: u64) {
    consumed()
        .lock()
        .expect("bridge consumed-capability lock is not poisoned")
        .insert((origin, nonce));
}
fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), PresentationVectorGestureErrorV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| PresentationVectorGestureErrorV1::SessionConflict)?;
    (snapshot.revision() == fence.revision() && snapshot.digest() == &fence.digest())
        .then_some(())
        .ok_or(PresentationVectorGestureErrorV1::StaleSnapshot)
}

pub fn begin_presentation_vector_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    kind: PresentationVectorKindV1,
    start: PresentationGesturePoint2V1,
) -> Result<PresentationVectorGestureV1, PresentationVectorGestureErrorV1> {
    require_fence(session, fence)?;
    static NEXT: AtomicU64 = AtomicU64::new(1);
    Ok(PresentationVectorGestureV1 {
        origin: origin(session),
        nonce: NEXT.fetch_add(1, Ordering::Relaxed),
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
    let stroke_color = standard
        .and_then(|value| value.line_color())
        .map_or("#000000", |value| value.as_str())
        .to_owned();
    let stroke_width = standard
        .and_then(|value| value.line_width())
        .map_or(1.0, |value| value.value());
    let fill_color = standard
        .and_then(|value| value.area_color())
        .and_then(|value| match value {
            TransparentOrRgb24V1::Transparent => None,
            TransparentOrRgb24V1::Rgb24(color) => Some(color.as_str().to_owned()),
        });
    Ok(PresentationVectorAppearanceV1 {
        stroke_color,
        stroke_width,
        fill_color,
    })
}
pub fn preview_presentation_vector_gesture_v1(
    session: &DocumentSession,
    gesture: &PresentationVectorGestureV1,
    raw_end: PresentationGesturePoint2V1,
) -> Result<PresentationVectorPreviewV1, PresentationVectorGestureErrorV1> {
    if gesture.origin != origin(session) {
        return Err(PresentationVectorGestureErrorV1::ForeignSession);
    }
    require_fence(session, gesture.fence)?;
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
        gesture: gesture.clone(),
        end,
        overlay,
    })
}
pub fn prepare_presentation_vector_gesture_v1(
    session: &mut DocumentSession,
    gesture: &PresentationVectorGestureV1,
    preview: &PresentationVectorPreviewV1,
) -> Result<PreparedPresentationVectorV1, PresentationVectorGestureErrorV1> {
    if gesture.origin != origin(session) || preview.gesture.origin != origin(session) {
        return Err(PresentationVectorGestureErrorV1::ForeignSession);
    }
    if gesture.nonce != preview.gesture.nonce {
        return Err(PresentationVectorGestureErrorV1::MismatchedPreview);
    }
    if is_consumed(gesture.origin, gesture.nonce) {
        return Err(PresentationVectorGestureErrorV1::ReplayedGesture);
    }
    require_fence(session, gesture.fence)?;
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let source = session
        .snapshot()
        .map_err(|_| PresentationVectorGestureErrorV1::SessionConflict)?
        .cdml()
        .to_owned();
    let identifier = loop {
        let id = format!(
            "presentation-vector-{}",
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        );
        if !session.contains_durable_id_v1(&id) {
            break id;
        }
    };
    let candidate = insert_vector(
        &source,
        &identifier,
        gesture.kind,
        gesture.start,
        preview.end,
        &gesture.appearance,
    )?;
    let contract = ferrum_render_contract::preflight_complete_document_v1(&candidate)
        .map_err(|_| PresentationVectorGestureErrorV1::RenderPreparation)?;
    let candidate_session = DocumentSession::load(&candidate)
        .map_err(|_| PresentationVectorGestureErrorV1::RenderPreparation)?;
    let observation = candidate_session
        .observe(0)
        .map_err(|_| PresentationVectorGestureErrorV1::RenderPreparation)?;
    let render_observation = document_observation_from_accepted_operation_v1(&observation)
        .map_err(|_| PresentationVectorGestureErrorV1::RenderPreparation)?;
    let plan = compose_document_render_plan_v1(&render_observation)
        .map_err(|_| PresentationVectorGestureErrorV1::RenderPreparation)?;
    if plan
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(PresentationVectorGestureErrorV1::RenderPreparation);
    }
    let candidate_snapshot = candidate_session
        .snapshot()
        .map_err(|_| PresentationVectorGestureErrorV1::RenderPreparation)?;
    let candidate_revision = gesture
        .fence
        .revision()
        .checked_add(1)
        .ok_or(PresentationVectorGestureErrorV1::SessionConflict)?;
    Ok(PreparedPresentationVectorV1 {
        receipt: Some(RendererPreflightReceiptV1 {
            origin: gesture.origin,
            nonce: gesture.nonce,
            source_fence: gesture.fence,
            candidate_revision,
            candidate_digest: *candidate_snapshot.digest(),
            root_identifier: identifier.clone(),
            candidate,
            contract,
            plan,
        }),
        kind: gesture.kind,
        identifier,
    })
}
pub fn commit_presentation_vector_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedPresentationVectorV1,
) -> Result<CommittedPresentationVectorV1, PresentationVectorGestureErrorV1> {
    let receipt = prepared
        .receipt
        .take()
        .ok_or(PresentationVectorGestureErrorV1::ReplayedGesture)?;
    if receipt.origin != origin(session) {
        prepared.receipt = Some(receipt);
        return Err(PresentationVectorGestureErrorV1::ForeignSession);
    }
    if is_consumed(receipt.origin, receipt.nonce) {
        return Err(PresentationVectorGestureErrorV1::ReplayedGesture);
    }
    require_fence(session, receipt.source_fence)?;
    if receipt.root_identifier != prepared.identifier
        || receipt.candidate_revision != receipt.source_fence.revision().saturating_add(1)
        || receipt.contract.source() != receipt.candidate
        || receipt
            .plan
            .outcomes()
            .iter()
            .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(PresentationVectorGestureErrorV1::RenderPreparation);
    }
    let candidate_session = DocumentSession::load(&receipt.candidate)
        .map_err(|_| PresentationVectorGestureErrorV1::RenderPreparation)?;
    let candidate_snapshot = candidate_session
        .snapshot()
        .map_err(|_| PresentationVectorGestureErrorV1::RenderPreparation)?;
    if *candidate_snapshot.digest() != receipt.candidate_digest {
        return Err(PresentationVectorGestureErrorV1::RenderPreparation);
    }
    let result = session
        .commit_complete_cdml_transaction_v1(receipt.source_fence, &receipt.candidate)
        .map_err(|_| PresentationVectorGestureErrorV1::SessionConflict)?;
    consume(receipt.origin, receipt.nonce);
    let kind = match prepared.kind {
        PresentationVectorKindV1::Line => PresentationRecordKindV1::Polyline,
        PresentationVectorKindV1::Rectangle => PresentationRecordKindV1::Rectangle,
        PresentationVectorKindV1::Square => PresentationRecordKindV1::Square,
        PresentationVectorKindV1::Oval => PresentationRecordKindV1::Oval,
        PresentationVectorKindV1::Circle => PresentationRecordKindV1::Circle,
    };
    let root = PresentationRootSelectorV1::new(&prepared.identifier, kind)
        .expect("bridge generated a valid identifier");
    Ok(CommittedPresentationVectorV1 { root, result })
}

fn insert_vector(
    source: &str,
    id: &str,
    kind: PresentationVectorKindV1,
    start: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
    appearance: &PresentationVectorAppearanceV1,
) -> Result<String, PresentationVectorGestureErrorV1> {
    let geometry = match kind {
        PresentationVectorKindV1::Line => format!(
            "<polyline id=\"{id}\" spline=\"0\" line_color=\"{}\" width=\"{}\"><point x=\"{}\" y=\"{}\" z=\"0\"/><point x=\"{}\" y=\"{}\" z=\"0\"/></polyline>",
            appearance.stroke_color,
            appearance.stroke_width,
            start.x(),
            start.y(),
            end.x(),
            end.y()
        ),
        PresentationVectorKindV1::Rectangle => shape("rect", id, start, end, appearance),
        PresentationVectorKindV1::Square => shape("square", id, start, end, appearance),
        PresentationVectorKindV1::Oval => shape("oval", id, start, end, appearance),
        PresentationVectorKindV1::Circle => shape("circle", id, start, end, appearance),
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
        .ok_or(PresentationVectorGestureErrorV1::RenderPreparation)?;
    Ok(format!("{}>{}</cdml>", &source[..self_close], geometry))
}
fn shape(
    tag: &str,
    id: &str,
    start: PresentationGesturePoint2V1,
    end: PresentationGesturePoint2V1,
    appearance: &PresentationVectorAppearanceV1,
) -> String {
    format!(
        "<{tag} id=\"{id}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" line_color=\"{}\" width=\"{}\" area_color=\"{}\"/>",
        start.x(),
        start.y(),
        end.x(),
        end.y(),
        appearance.stroke_color,
        appearance.stroke_width,
        appearance.fill_color.as_deref().unwrap_or("none")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const EMPTY: &str = r#"<cdml xmlns="urn:ferrum:cdml" version="26.07"/>"#;

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    #[test]
    fn bridge_preflights_and_commits_once() {
        let mut session = DocumentSession::load(EMPTY).expect("session");
        let gesture = begin_presentation_vector_gesture_v1(
            &session,
            fence(&session),
            PresentationVectorKindV1::Rectangle,
            PresentationGesturePoint2V1::new(1.0, 2.0).expect("point"),
        )
        .expect("gesture");
        let preview = preview_presentation_vector_gesture_v1(
            &session,
            &gesture,
            PresentationGesturePoint2V1::new(8.0, 9.0).expect("point"),
        )
        .expect("preview");
        let mut prepared = prepare_presentation_vector_gesture_v1(&mut session, &gesture, &preview)
            .expect("prepare");
        let mut duplicate =
            prepare_presentation_vector_gesture_v1(&mut session, &gesture, &preview)
                .expect("independent prepared receipt");
        assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
        let committed =
            commit_presentation_vector_gesture_v1(&mut session, &mut prepared).expect("commit");
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
        assert!(matches!(
            commit_presentation_vector_gesture_v1(&mut session, &mut prepared),
            Err(PresentationVectorGestureErrorV1::ReplayedGesture)
        ));
        assert!(matches!(
            commit_presentation_vector_gesture_v1(&mut session, &mut duplicate),
            Err(PresentationVectorGestureErrorV1::ReplayedGesture)
        ));
    }

    #[test]
    fn bridge_rejects_foreign_handles_without_mutation() {
        let first = DocumentSession::load(EMPTY).expect("first");
        let second = DocumentSession::load(EMPTY).expect("second");
        let gesture = begin_presentation_vector_gesture_v1(
            &first,
            fence(&first),
            PresentationVectorKindV1::Line,
            PresentationGesturePoint2V1::new(1.0, 2.0).expect("point"),
        )
        .expect("gesture");
        assert!(matches!(
            preview_presentation_vector_gesture_v1(
                &second,
                &gesture,
                PresentationGesturePoint2V1::new(8.0, 9.0).expect("point"),
            ),
            Err(PresentationVectorGestureErrorV1::ForeignSession)
        ));
        assert_eq!(second.snapshot().expect("snapshot").revision(), 0);
    }

    #[test]
    fn bridge_origin_survives_a_session_move() {
        let session = DocumentSession::load(EMPTY).expect("session");
        let gesture = begin_presentation_vector_gesture_v1(
            &session,
            fence(&session),
            PresentationVectorKindV1::Line,
            PresentationGesturePoint2V1::new(1.0, 2.0).expect("point"),
        )
        .expect("gesture");
        let moved = Box::new(session);
        assert!(
            preview_presentation_vector_gesture_v1(
                moved.as_ref(),
                &gesture,
                PresentationGesturePoint2V1::new(8.0, 9.0).expect("point"),
            )
            .is_ok()
        );
    }

    #[test]
    fn prepared_receipt_refuses_a_changed_session_without_mutation() {
        let mut session = DocumentSession::load(EMPTY).expect("session");
        let expected = fence(&session);
        let gesture = begin_presentation_vector_gesture_v1(
            &session,
            expected,
            PresentationVectorKindV1::Line,
            PresentationGesturePoint2V1::new(1.0, 2.0).expect("point"),
        )
        .expect("gesture");
        let preview = preview_presentation_vector_gesture_v1(
            &session,
            &gesture,
            PresentationGesturePoint2V1::new(8.0, 9.0).expect("point"),
        )
        .expect("preview");
        let mut prepared = prepare_presentation_vector_gesture_v1(&mut session, &gesture, &preview)
            .expect("prepare");
        let source = session.snapshot().expect("snapshot").cdml().to_owned();
        session
            .commit_complete_cdml_transaction_v1(expected, &source)
            .expect("generic transition");
        assert!(matches!(
            commit_presentation_vector_gesture_v1(&mut session, &mut prepared),
            Err(PresentationVectorGestureErrorV1::StaleSnapshot)
        ));
        assert_eq!(session.snapshot().expect("snapshot").revision(), 1);
    }

    #[test]
    fn bridge_refuses_compositor_exclusion_without_mutation() {
        // A known Plus root with an authored face passes complete-CDML preflight,
        // but V1 cannot provide a verified layout for that face. The compositor
        // therefore emits an explicit exclusion.
        let source = r#"<cdml xmlns="urn:ferrum:cdml"><plus id="bad"><point x="1" y="2"/><font family="Arial"/></plus></cdml>"#;
        let mut session = DocumentSession::load(source).expect("session");
        let gesture = begin_presentation_vector_gesture_v1(
            &session,
            fence(&session),
            PresentationVectorKindV1::Line,
            PresentationGesturePoint2V1::new(1.0, 2.0).expect("point"),
        )
        .expect("gesture");
        let preview = preview_presentation_vector_gesture_v1(
            &session,
            &gesture,
            PresentationGesturePoint2V1::new(8.0, 9.0).expect("point"),
        )
        .expect("preview");
        assert!(matches!(
            prepare_presentation_vector_gesture_v1(&mut session, &gesture, &preview),
            Err(PresentationVectorGestureErrorV1::RenderPreparation)
        ));
        assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
    }
}

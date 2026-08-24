//! Renderer-preflighted ownership for vector authoring transactions.
//!
//! `ferrum-document` owns visual transaction admission and CDML state transitions.
//! This crate owns vector-specific gesture capability and preview interpretation.

use ferrum_document::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityIssuerV1, AuthoringCapabilityV1,
    DocumentFenceV1, DocumentSession, DocumentSessionError, GeometricLineWidthV1,
    PendingCreatePresentationV1,
    PresentationAppearanceV1, PresentationCreateErrorV1, PresentationCreateRequestV1,
    PresentationGesturePoint2V1, PresentationRootSelectorV1, PresentationVectorCreateKindV1,
    Rgb24V1, SessionOperationResultV1, TransparentOrRgb24V1, TypedDocumentError,
};
use thiserror::Error;

mod catalog_placement_v2;
mod compact_group_materialization_v1;
#[cfg(test)]
mod compact_group_materialization_v1_tests;
mod compact_group_placement_v1;
mod curved_electron_arrow_gesture_v1;
mod curved_equilibrium_arrow_gesture_v1;
mod direct_bond_admission_v3;
mod direct_bond_explicit_v1;
mod direct_bond_pointer_v3;
mod direct_bond_probe_resolution_v3;
mod direct_bond_v3_lifecycle;
mod hydrogen_materialization_v1;
mod presentation_path_gesture_v1;
mod reaction_gesture_v1;
mod reaction_lifecycle_v1;
mod reaction_observation_v1;
mod reaction_translation_v1;
mod render_interaction_v1;

pub use catalog_placement_v2::{
    CatalogPlacementCategoryV2, CatalogPlacementErrorV2, CatalogPlacementGestureV2,
    CatalogPlacementPreviewV2, CatalogPlacementRecoveryV2, CommittedCatalogPlacementV2,
    PreparedCatalogPlacementV2, begin_catalog_placement_v2, cancel_catalog_placement_gesture_v2,
    commit_catalog_placement_v2, prepare_catalog_placement_v2, preview_catalog_placement_v2,
    release_catalog_placement_preview_v2,
};
pub use compact_group_materialization_v1::{
    CommittedCompactGroupMaterializationV1, CompactGroupMaterializationErrorV1,
    PreparedCompactGroupMaterializationV1, commit_compact_group_materialization_v1,
    prepare_compact_group_materialization_v1,
};
pub use compact_group_placement_v1::{
    CompactGroupPlacementErrorV1, PreparedCompactGroupPlacementV1,
    commit_compact_group_placement_v1, prepare_compact_group_placement_v1,
};
pub use curved_electron_arrow_gesture_v1::{
    CommittedCurvedElectronArrowV1, CommittedCurvedNormalReactionArrowV1,
    CommittedCurvedRetroArrowV1, CurvedElectronArrowGestureCategoryV1,
    CurvedElectronArrowGestureErrorV1, CurvedElectronArrowGestureRecoveryV1,
    CurvedElectronArrowGestureV1, CurvedElectronArrowPreviewV1,
    CurvedNormalReactionArrowGestureCategoryV1, CurvedNormalReactionArrowGestureErrorV1,
    CurvedNormalReactionArrowGestureRecoveryV1, CurvedNormalReactionArrowGestureV1,
    CurvedNormalReactionArrowPreviewV1, CurvedRetroArrowGestureCategoryV1,
    CurvedRetroArrowGestureErrorV1, CurvedRetroArrowGestureRecoveryV1, CurvedRetroArrowGestureV1,
    CurvedRetroArrowPreviewV1, PreparedCurvedElectronArrowV1, PreparedCurvedNormalReactionArrowV1,
    PreparedCurvedRetroArrowV1, begin_curved_electron_arrow_gesture_v1,
    begin_curved_normal_reaction_arrow_gesture_v1, begin_curved_retro_arrow_gesture_v1,
    commit_curved_electron_arrow_gesture_v1, commit_curved_normal_reaction_arrow_gesture_v1,
    commit_curved_retro_arrow_gesture_v1, prepare_curved_electron_arrow_gesture_v1,
    prepare_curved_normal_reaction_arrow_gesture_v1, prepare_curved_retro_arrow_gesture_v1,
    preview_curved_electron_arrow_gesture_v1, preview_curved_normal_reaction_arrow_gesture_v1,
    preview_curved_retro_arrow_gesture_v1,
};
pub use curved_equilibrium_arrow_gesture_v1::{
    CommittedCurvedEquilibriumArrowV1, CurvedEquilibriumArrowGestureCategoryV1,
    CurvedEquilibriumArrowGestureErrorV1, CurvedEquilibriumArrowGestureRecoveryV1,
    CurvedEquilibriumArrowGestureV1, CurvedEquilibriumArrowPreviewV1,
    PreparedCurvedEquilibriumArrowV1, begin_curved_equilibrium_arrow_gesture_v1,
    commit_curved_equilibrium_arrow_gesture_v1, prepare_curved_equilibrium_arrow_gesture_v1,
    preview_curved_equilibrium_arrow_gesture_v1,
};
pub use direct_bond_admission_v3::{
    admit_direct_bond_candidate_v3, begin_direct_bond_gesture_v3, commit_direct_bond_admission_v3,
};
pub use direct_bond_explicit_v1::{DirectBondExplicitErrorV1, author_direct_bond_explicit_v1};
pub use direct_bond_pointer_v3::{
    CommittedDirectBondGestureV3, DirectBondAdmissionCategoryV3, DirectBondAdmissionErrorV3,
    DirectBondAdmissionRecoveryV3, DirectBondAdmissionRefusalV3, DirectBondAdmissionV3,
    DirectBondGestureV3, DirectBondOverlayV3, DirectBondPointerHitStateV3,
    DirectBondPointerProbeCategoryV3, DirectBondPointerProbeErrorV3,
    DirectBondPointerProbeRecoveryV3, DirectBondPointerProbeV3, DirectBondViewportToSceneV3,
};
pub use direct_bond_v3_lifecycle::{
    CommittedDirectBondGesture, DirectBondCommitCategoryV1, DirectBondCommitError,
    DirectBondCommitRecoveryV1,
};
pub use hydrogen_materialization_v1::{
    CommittedHydrogenMaterializationV1, HydrogenMaterializationErrorV1,
    PreparedHydrogenMaterializationV1, commit_hydrogen_materialization_v1,
    prepare_hydrogen_materialization_v1,
};
pub use presentation_path_gesture_v1::{
    CommittedPresentationPathV1, PreparedPresentationPathV1, PresentationPathAppearanceV1,
    PresentationPathOverlayV1, PresentationPathProgressV1, PresentationPathRenderCategoryV1,
    PresentationPathRenderErrorV1, PresentationPathRenderGestureV1,
    PresentationPathRenderRecoveryV1, add_presentation_path_gesture_point_v1,
    begin_presentation_path_gesture_v1, cancel_presentation_path_gesture_v1,
    commit_presentation_path_gesture_v1, prepare_incremental_presentation_path_gesture_v1,
    preview_incremental_presentation_path_gesture_v1,
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
    stroke_color: Rgb24V1,
    stroke_width: GeometricLineWidthV1,
    fill_color: Option<Rgb24V1>,
}
impl PresentationVectorAppearanceV1 {
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
    capability: AuthoringCapabilityV1,
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
    receipt: Option<PendingCreatePresentationV1>,
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

fn authoring_issuer(session: &DocumentSession) -> AuthoringCapabilityIssuerV1 {
    session.authoring_capability_issuer_v1()
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
    Ok(PresentationVectorGestureV1 {
        capability: authoring_issuer(session).issue(),
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
        .cloned()
        .unwrap_or_else(|| Rgb24V1::new("#000000").expect("closed built-in colour"));
    let stroke_width = standard
        .and_then(|value| value.line_width())
        .map_or(1.0, |value| value.value());
    let stroke_width = GeometricLineWidthV1::new(stroke_width)
        .ok_or(PresentationVectorGestureErrorV1::UnrenderableStandard)?;
    let fill_color = standard
        .and_then(|value| value.area_color())
        .and_then(|value| match value {
            TransparentOrRgb24V1::Transparent => None,
            TransparentOrRgb24V1::Rgb24(color) => Some(color.clone()),
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
    if !gesture.capability.belongs_to(&authoring_issuer(session)) {
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
    let issuer = authoring_issuer(session);
    if !gesture.capability.belongs_to(&issuer) || !preview.gesture.capability.belongs_to(&issuer) {
        return Err(PresentationVectorGestureErrorV1::ForeignSession);
    }
    if !gesture
        .capability
        .same_capability(&preview.gesture.capability)
    {
        return Err(PresentationVectorGestureErrorV1::MismatchedPreview);
    }
    match gesture.capability.claim_for_commit(&issuer) {
        Ok(claim) => drop(claim),
        Err(AuthoringCapabilityAccessErrorV1::ForeignSession) => {
            return Err(PresentationVectorGestureErrorV1::ForeignSession);
        }
        Err(AuthoringCapabilityAccessErrorV1::Replayed) => {
            return Err(PresentationVectorGestureErrorV1::ReplayedGesture);
        }
    }
    require_fence(session, gesture.fence)?;
    let pending = session
        .prepare_create_presentation_v1(
            &gesture.capability,
            gesture.fence,
            PresentationCreateRequestV1::Vector {
                kind: vector_kind(gesture.kind),
                start: gesture.start,
                end: preview.end,
                appearance: PresentationAppearanceV1::new(
                    gesture.appearance.stroke_color.clone(),
                    gesture.appearance.stroke_width,
                    gesture.appearance.fill_color.clone(),
                ),
            },
        )
        .map_err(map_presentation_create_error)?;
    let identifier = pending.identifier().as_str().to_owned();
    Ok(PreparedPresentationVectorV1 {
        receipt: Some(pending),
        identifier,
    })
}
pub fn commit_presentation_vector_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedPresentationVectorV1,
) -> Result<CommittedPresentationVectorV1, PresentationVectorGestureErrorV1> {
    let mut pending = prepared
        .receipt
        .take()
        .ok_or(PresentationVectorGestureErrorV1::ReplayedGesture)?;
    if pending.identifier().as_str() != prepared.identifier {
        return Err(PresentationVectorGestureErrorV1::RenderPreparation);
    }
    let result = (|| {
        session
            .commit_create_presentation_v1(&mut pending)
            .map_err(map_presentation_create_error)
    })();
    match result {
        Ok(result) => {
            let root = PresentationRootSelectorV1::new(&prepared.identifier, pending.root_kind())
                .expect("bridge generated a valid identifier");
            Ok(CommittedPresentationVectorV1 { root, result })
        }
        Err(error) => {
            prepared.receipt = Some(pending);
            Err(error)
        }
    }
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

fn map_presentation_create_error(
    error: PresentationCreateErrorV1,
) -> PresentationVectorGestureErrorV1 {
    match error {
        PresentationCreateErrorV1::ForeignSession => {
            PresentationVectorGestureErrorV1::ForeignSession
        }
        PresentationCreateErrorV1::StaleSnapshot => PresentationVectorGestureErrorV1::StaleSnapshot,
        PresentationCreateErrorV1::Replayed => PresentationVectorGestureErrorV1::ReplayedGesture,
        PresentationCreateErrorV1::SessionConflict => {
            PresentationVectorGestureErrorV1::SessionConflict
        }
        PresentationCreateErrorV1::RendererAdmission => {
            PresentationVectorGestureErrorV1::RenderPreparation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::{
        DirectBondEndpointIntent, DirectBondPoint2V1, DirectBondSnapPolicyV1, DocumentBondOrderV1,
        DocumentBondPresentationV1,
    };

    const EMPTY: &str = r#"<cdml xmlns="urn:ferrum:cdml" version="26.07"/>"#;

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    fn point(x: f64, y: f64) -> PresentationGesturePoint2V1 {
        PresentationGesturePoint2V1::new(x, y).expect("finite test point")
    }

    fn commit_terminal_arrow(
        session: &mut DocumentSession,
        begin: fn(
            &DocumentSession,
            DocumentFenceV1,
            PresentationGesturePoint2V1,
            PresentationGesturePoint2V1,
        )
            -> Result<CurvedElectronArrowGestureV1, CurvedElectronArrowGestureErrorV1>,
        preview: fn(
            &DocumentSession,
            &CurvedElectronArrowGestureV1,
            PresentationGesturePoint2V1,
        )
            -> Result<CurvedElectronArrowPreviewV1, CurvedElectronArrowGestureErrorV1>,
        prepare: fn(
            &mut DocumentSession,
            &CurvedElectronArrowGestureV1,
            &CurvedElectronArrowPreviewV1,
        )
            -> Result<PreparedCurvedElectronArrowV1, CurvedElectronArrowGestureErrorV1>,
        commit: fn(
            &mut DocumentSession,
            &mut PreparedCurvedElectronArrowV1,
        )
            -> Result<CommittedCurvedElectronArrowV1, CurvedElectronArrowGestureErrorV1>,
    ) {
        let gesture = begin(session, fence(session), point(0.0, 0.0), point(20.0, 20.0))
            .expect("terminal gesture");
        let issued = preview(session, &gesture, point(40.0, 0.0)).expect("terminal preview");
        let mut receipt = prepare(session, &gesture, &issued).expect("terminal receipt");
        commit(session, &mut receipt).expect("terminal commit");
    }

    #[test]
    fn shared_capabilities_do_not_replay_across_authoring_families() {
        let mut session = DocumentSession::load(EMPTY).expect("session");

        let equilibrium_fence = fence(&session);
        let equilibrium = begin_curved_equilibrium_arrow_gesture_v1(
            &mut session,
            equilibrium_fence,
            point(0.0, 0.0),
            point(40.0, 20.0),
        )
        .expect("equilibrium gesture");
        let equilibrium_preview =
            preview_curved_equilibrium_arrow_gesture_v1(&session, &equilibrium, point(80.0, 0.0))
                .expect("equilibrium preview");
        let mut equilibrium_receipt = prepare_curved_equilibrium_arrow_gesture_v1(
            &mut session,
            &equilibrium,
            &equilibrium_preview,
        )
        .expect("equilibrium receipt");
        commit_curved_equilibrium_arrow_gesture_v1(&mut session, &mut equilibrium_receipt)
            .expect("equilibrium commit");

        commit_terminal_arrow(
            &mut session,
            begin_curved_electron_arrow_gesture_v1,
            preview_curved_electron_arrow_gesture_v1,
            prepare_curved_electron_arrow_gesture_v1,
            commit_curved_electron_arrow_gesture_v1,
        );
        commit_terminal_arrow(
            &mut session,
            begin_curved_retro_arrow_gesture_v1,
            preview_curved_retro_arrow_gesture_v1,
            prepare_curved_retro_arrow_gesture_v1,
            commit_curved_retro_arrow_gesture_v1,
        );
        commit_terminal_arrow(
            &mut session,
            begin_curved_normal_reaction_arrow_gesture_v1,
            preview_curved_normal_reaction_arrow_gesture_v1,
            prepare_curved_normal_reaction_arrow_gesture_v1,
            commit_curved_normal_reaction_arrow_gesture_v1,
        );

        let direct_bond = direct_bond_v3_lifecycle::begin_direct_bond_v3_lifecycle(
            &session,
            fence(&session),
            DirectBondEndpointIntent::NewAtomAt {
                raw_point: DirectBondPoint2V1::new(0.0, 90.0).expect("direct-bond point"),
            },
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        )
        .expect("direct-bond gesture");
        let mut direct_admission = direct_bond_v3_lifecycle::admit_direct_bond_candidate(
            &mut session,
            &direct_bond,
            DirectBondEndpointIntent::NewAtomAt {
                raw_point: DirectBondPoint2V1::new(40.0, 90.0).expect("direct-bond point"),
            },
        )
        .expect("direct-bond admission");
        let direct_committed = direct_bond_v3_lifecycle::commit_direct_bond_admission(
            &mut session,
            &mut direct_admission,
        )
        .expect("direct-bond commit");

        let vector = begin_presentation_vector_gesture_v1(
            &session,
            fence(&session),
            PresentationVectorKindV1::Line,
            point(0.0, 30.0),
        )
        .expect("vector gesture");
        let vector_preview =
            preview_presentation_vector_gesture_v1(&session, &vector, point(40.0, 30.0))
                .expect("vector preview");
        let mut vector_receipt =
            prepare_presentation_vector_gesture_v1(&mut session, &vector, &vector_preview)
                .expect("vector receipt");
        commit_presentation_vector_gesture_v1(&mut session, &mut vector_receipt)
            .expect("vector commit");

        let mut path = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            ferrum_document::PresentationPathKindV1::Polyline,
        )
        .expect("path gesture");
        for vertex in [point(0.0, 60.0), point(40.0, 60.0)] {
            add_presentation_path_gesture_point_v1(&session, &mut path, vertex)
                .expect("path vertex");
        }
        let path_preview = preview_incremental_presentation_path_gesture_v1(&session, &path, None)
            .expect("path preview");
        let mut path_receipt =
            prepare_incremental_presentation_path_gesture_v1(&mut session, &path, &path_preview)
                .expect("path receipt");
        commit_presentation_path_gesture_v1(&mut session, &mut path_receipt).expect("path commit");

        let source = session.snapshot().expect("snapshot").cdml().to_owned();
        assert!(source.contains("type=\"curved-equilibrium\""));
        assert!(source.contains("<polyline"));
        assert!(source.contains(direct_committed.bond().as_str()));
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
        let mut generic_transition = session
            .prepare_complete_cdml_mutation_v1(expected, &source)
            .expect("prepare generic transition");
        session
            .commit_complete_cdml_mutation_v1(&mut generic_transition)
            .expect("generic transition");
        assert!(matches!(
            commit_presentation_vector_gesture_v1(&mut session, &mut prepared),
            Err(PresentationVectorGestureErrorV1::StaleSnapshot)
        ));
        assert_eq!(session.snapshot().expect("snapshot").revision(), 1);
    }

    #[test]
    fn unsupported_text_face_is_refused_before_vector_gesture_session_exists() {
        let source = r#"<cdml xmlns="urn:ferrum:cdml"><plus id="bad"><point x="1" y="2"/><font family="Arial"/></plus></cdml>"#;
        assert!(matches!(
            DocumentSession::load(source),
            Err(DocumentSessionError::Load(TypedDocumentError::UnsupportedTextFace {
                root_id,
                family,
            })) if root_id == "bad" && family == "Arial"
        ));
    }
}

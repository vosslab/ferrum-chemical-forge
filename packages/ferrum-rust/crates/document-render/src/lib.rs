//! Renderer-preflighted ownership for vector authoring transactions.
//!
//! `ferrum-document` owns visual transaction admission and CDML state transitions.
//! This crate owns vector-specific gesture capability and preview interpretation.

#[cfg(test)]
use ferrum_document::PresentationGesturePoint2V1;
use ferrum_document::{DocumentFenceV1, DocumentSession};

mod curved_electron_arrow_gesture_v1;
mod curved_equilibrium_arrow_gesture_v1;
mod direct_bond_admission_v3;
mod direct_bond_pointer_v3;
mod direct_bond_probe_resolution_v3;
mod direct_bond_v3_lifecycle;
mod presentation_path_gesture_v1;
mod presentation_vector_gesture_v1;
mod reaction_gesture_v1;
mod reaction_lifecycle_v1;
mod reaction_observation_v1;
mod reaction_translation_v1;
mod render_interaction_v1;

fn require_fence(session: &DocumentSession, fence: DocumentFenceV1) -> Result<(), ()> {
    let snapshot = session.snapshot().map_err(|_| ())?;
    (snapshot.revision() == fence.revision() && snapshot.digest() == &fence.digest())
        .then_some(())
        .ok_or(())
}

pub use curved_electron_arrow_gesture_v1::{
    CurvedElectronArrowGestureCategoryV1, CurvedElectronArrowGestureErrorV1,
    CurvedElectronArrowGestureRecoveryV1, CurvedElectronArrowGestureV1,
    CurvedElectronArrowPreviewV1, CurvedNormalReactionArrowGestureCategoryV1,
    CurvedNormalReactionArrowGestureErrorV1, CurvedNormalReactionArrowGestureRecoveryV1,
    CurvedNormalReactionArrowGestureV1, CurvedNormalReactionArrowPreviewV1,
    CurvedRetroArrowGestureCategoryV1, CurvedRetroArrowGestureErrorV1,
    CurvedRetroArrowGestureRecoveryV1, CurvedRetroArrowGestureV1, CurvedRetroArrowPreviewV1,
    begin_curved_electron_arrow_gesture_v1, begin_curved_normal_reaction_arrow_gesture_v1,
    begin_curved_retro_arrow_gesture_v1, preview_curved_electron_arrow_gesture_v1,
    preview_curved_normal_reaction_arrow_gesture_v1, preview_curved_retro_arrow_gesture_v1,
    resolve_curved_electron_arrow_gesture_v1, resolve_curved_normal_reaction_arrow_gesture_v1,
    resolve_curved_retro_arrow_gesture_v1,
};
pub use curved_equilibrium_arrow_gesture_v1::{
    CurvedEquilibriumArrowGestureCategoryV1, CurvedEquilibriumArrowGestureErrorV1,
    CurvedEquilibriumArrowGestureRecoveryV1, CurvedEquilibriumArrowGestureV1,
    CurvedEquilibriumArrowPreviewV1, begin_curved_equilibrium_arrow_gesture_v1,
    preview_curved_equilibrium_arrow_gesture_v1, resolve_curved_equilibrium_arrow_gesture_v1,
};
pub use direct_bond_admission_v3::{begin_direct_bond_gesture_v3, resolve_direct_bond_end_v3};
pub use direct_bond_pointer_v3::{
    DirectBondAdmissionCategoryV3, DirectBondAdmissionErrorV3, DirectBondAdmissionRecoveryV3,
    DirectBondAdmissionRefusalV3, DirectBondGestureV3, DirectBondPointerHitStateV3,
    DirectBondPointerProbeCategoryV3, DirectBondPointerProbeErrorV3,
    DirectBondPointerProbeRecoveryV3, DirectBondPointerProbeV3, DirectBondViewportToSceneV3,
};
pub use presentation_path_gesture_v1::{
    PresentationPathAppearanceV1, PresentationPathOverlayV1, PresentationPathProgressV1,
    PresentationPathRenderCategoryV1, PresentationPathRenderErrorV1,
    PresentationPathRenderGestureV1, PresentationPathRenderRecoveryV1,
    add_presentation_path_gesture_point_v1, begin_presentation_path_gesture_v1,
    cancel_presentation_path_gesture_v1, preview_incremental_presentation_path_gesture_v1,
    resolve_incremental_presentation_path_gesture_v1,
};
pub use presentation_vector_gesture_v1::{
    PRESENTATION_VECTOR_MAXIMUM_EXTENT_PT_V1, PresentationVectorAppearanceV1,
    PresentationVectorGestureCategoryV1, PresentationVectorGestureErrorV1,
    PresentationVectorGestureRecoveryV1, PresentationVectorGestureV1, PresentationVectorKindV1,
    PresentationVectorOverlayV1, PresentationVectorPreviewV1, begin_presentation_vector_gesture_v1,
    preview_presentation_vector_gesture_v1, resolve_presentation_vector_gesture_v1,
};
pub use reaction_gesture_v1::{
    ReactionCreateRequestV1, ReactionGestureCategoryV1, ReactionGestureErrorV1,
    ReactionGestureRecoveryV1, ReactionGestureV1, begin_reaction_gesture_v1,
    resolve_reaction_gesture_v1,
};
pub use reaction_lifecycle_v1::{
    ReactionLifecycleGestureV1, ReactionMembershipPatchRequestV1,
    begin_reaction_definition_delete_v1, begin_reaction_membership_patch_v1,
    resolve_reaction_lifecycle_v1,
};
pub use reaction_observation_v1::{
    ReactionDefinitionDispositionV1, ReactionListObservationV1, ReactionMemberObservationV1,
    ReactionObservationV1, ReactionSelectionV1,
};
pub use reaction_translation_v1::{
    ReactionTranslationGestureV1, ReactionTranslationPreviewV1, begin_reaction_translation_v1,
    preview_reaction_translation_v1, resolve_reaction_translation_v1,
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

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::{
        PresentationPathKindV1, SessionOperationOutcomeV1, SessionOperationTransitionRequestV1,
    };

    const EMPTY: &str = r#"<cdml xmlns="urn:ferrum:cdml" version="26.07"/>"#;

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    fn point(x: f64, y: f64) -> PresentationGesturePoint2V1 {
        PresentationGesturePoint2V1::new(x, y).expect("finite point")
    }

    fn commit(
        session: &mut DocumentSession,
        request: SessionOperationTransitionRequestV1,
    ) -> ferrum_document::SessionOperationResultV1 {
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("generic preparation");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("accepted generic commit")
    }

    #[test]
    fn renderer_gestures_resolve_to_generic_authoring_transitions() {
        let mut session = DocumentSession::load(EMPTY).expect("session");
        let terminal = begin_curved_electron_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 0.0),
            point(20.0, 20.0),
        )
        .expect("terminal gesture");
        let terminal_preview =
            preview_curved_electron_arrow_gesture_v1(&session, &terminal, point(40.0, 0.0))
                .expect("terminal preview");
        let terminal_request =
            resolve_curved_electron_arrow_gesture_v1(&session, terminal, terminal_preview)
                .expect("terminal request");
        let terminal_result = commit(&mut session, terminal_request);
        assert!(matches!(
            terminal_result.outcome(),
            SessionOperationOutcomeV1::CreatedPresentationRootV1(_)
        ));

        let equilibrium = begin_curved_equilibrium_arrow_gesture_v1(
            &session,
            fence(&session),
            point(0.0, 20.0),
            point(50.0, 40.0),
        )
        .expect("equilibrium gesture");
        let equilibrium_preview =
            preview_curved_equilibrium_arrow_gesture_v1(&session, &equilibrium, point(100.0, 20.0))
                .expect("equilibrium preview");
        let equilibrium_request =
            resolve_curved_equilibrium_arrow_gesture_v1(&session, equilibrium, equilibrium_preview)
                .expect("equilibrium request");
        let equilibrium_result = commit(&mut session, equilibrium_request);
        assert!(matches!(
            equilibrium_result.outcome(),
            SessionOperationOutcomeV1::CreatedPresentationRootV1(_)
        ));

        let vector = begin_presentation_vector_gesture_v1(
            &session,
            fence(&session),
            PresentationVectorKindV1::Rectangle,
            point(0.0, 50.0),
        )
        .expect("vector gesture");
        let vector_preview =
            preview_presentation_vector_gesture_v1(&session, &vector, point(20.0, 70.0))
                .expect("vector preview");
        let vector_request =
            resolve_presentation_vector_gesture_v1(&session, vector, vector_preview)
                .expect("vector request");
        let vector_result = commit(&mut session, vector_request);
        assert!(matches!(
            vector_result.outcome(),
            SessionOperationOutcomeV1::CreatedPresentationRootV1(_)
        ));

        let mut path = begin_presentation_path_gesture_v1(
            &session,
            fence(&session),
            PresentationPathKindV1::Polyline,
        )
        .expect("path gesture");
        add_presentation_path_gesture_point_v1(&session, &mut path, point(0.0, 90.0))
            .expect("first path point");
        add_presentation_path_gesture_point_v1(&session, &mut path, point(30.0, 90.0))
            .expect("second path point");
        let path_preview = preview_incremental_presentation_path_gesture_v1(&session, &path, None)
            .expect("path preview");
        let path_request =
            resolve_incremental_presentation_path_gesture_v1(&session, path, path_preview)
                .expect("path request");
        let path_result = commit(&mut session, path_request);
        assert!(matches!(
            path_result.outcome(),
            SessionOperationOutcomeV1::CreatedPresentationRootV1(_)
        ));
        assert_eq!(path_result.observation().snapshot().revision(), 4);
    }
}

//! API facade for semantic reaction gestures and generic transition requests.

use crate::{ReactionSelectionV1, RenderInteractionSessionV1};
use ferrum_document::{DocumentFenceV1, DocumentSession, SessionOperationTransitionRequestV1};
use ferrum_document_render::ReactionTranslationGestureV1;
use ferrum_document_render::RenderInteractionSnapV1;
pub use ferrum_document_render::{ReactionCreateRequestV1, ReactionMembershipPatchRequestV1};
pub use ferrum_document_render::{
    ReactionGestureCategoryV1, ReactionGestureErrorV1, ReactionGestureRecoveryV1,
};
use ferrum_document_render::{ReactionGestureV1, ReactionLifecycleGestureV1};

/// Opaque semantic create-reaction gesture before generic request resolution.
#[derive(Debug)]
pub struct ApiReactionGestureV1(ReactionGestureV1);
/// Opaque semantic reaction-lifecycle gesture before generic request resolution.
#[derive(Debug)]
pub struct ApiReactionLifecycleGestureV1(ReactionLifecycleGestureV1);
/// Opaque semantic reaction-translation gesture before generic request resolution.
#[derive(Debug)]
pub struct ApiReactionTranslationGestureV1(ReactionTranslationGestureV1);

pub fn begin_api_reaction_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    request: ReactionCreateRequestV1,
) -> Result<ApiReactionGestureV1, ReactionGestureErrorV1> {
    ferrum_document_render::begin_reaction_gesture_v1(session, fence, request)
        .map(ApiReactionGestureV1)
}

pub fn resolve_api_reaction_gesture_v1(
    session: &DocumentSession,
    gesture: ApiReactionGestureV1,
) -> Result<SessionOperationTransitionRequestV1, ReactionGestureErrorV1> {
    ferrum_document_render::resolve_reaction_gesture_v1(session, gesture.0)
}

pub fn begin_api_reaction_membership_patch_v1(
    session: &RenderInteractionSessionV1,
    selection: &ReactionSelectionV1,
    request: ReactionMembershipPatchRequestV1,
) -> Result<ApiReactionLifecycleGestureV1, ReactionGestureErrorV1> {
    ferrum_document_render::begin_reaction_membership_patch_v1(session, selection, request)
        .map(ApiReactionLifecycleGestureV1)
}

pub fn begin_api_reaction_definition_delete_v1(
    session: &RenderInteractionSessionV1,
    selection: &ReactionSelectionV1,
) -> Result<ApiReactionLifecycleGestureV1, ReactionGestureErrorV1> {
    ferrum_document_render::begin_reaction_definition_delete_v1(session, selection)
        .map(ApiReactionLifecycleGestureV1)
}

pub fn resolve_api_reaction_lifecycle_v1(
    session: &RenderInteractionSessionV1,
    gesture: ApiReactionLifecycleGestureV1,
) -> Result<SessionOperationTransitionRequestV1, ReactionGestureErrorV1> {
    ferrum_document_render::resolve_reaction_lifecycle_v1(session, gesture.0)
}

pub fn begin_api_reaction_translation_v1(
    session: &RenderInteractionSessionV1,
    selection: &ReactionSelectionV1,
    press_x: f64,
    press_y: f64,
    snap: RenderInteractionSnapV1,
) -> Result<ApiReactionTranslationGestureV1, ReactionGestureErrorV1> {
    ferrum_document_render::begin_reaction_translation_v1(
        session, selection, press_x, press_y, snap,
    )
    .map(ApiReactionTranslationGestureV1)
}

pub fn resolve_api_reaction_translation_v1(
    session: &RenderInteractionSessionV1,
    gesture: ApiReactionTranslationGestureV1,
    pointer_x: f64,
    pointer_y: f64,
) -> Result<SessionOperationTransitionRequestV1, ReactionGestureErrorV1> {
    ferrum_document_render::resolve_reaction_translation_v1(
        session, gesture.0, pointer_x, pointer_y,
    )
}

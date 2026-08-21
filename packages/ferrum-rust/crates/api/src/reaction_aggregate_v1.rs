//! API facade for renderer-preflighted reaction authoring.

use crate::{ReactionSelectionV1, RenderInteractionSessionV1};
use ferrum_document::{DocumentFenceV1, DocumentSession};
use ferrum_document_render::RenderInteractionSnapV1;
pub use ferrum_document_render::{CommittedReactionLifecycleV1, ReactionMembershipPatchRequestV1};
pub use ferrum_document_render::{
    CommittedReactionV1, ReactionCreateRequestV1, ReactionGestureCategoryV1,
    ReactionGestureErrorV1, ReactionGestureRecoveryV1,
};
use ferrum_document_render::{PreparedReactionLifecycleV1, ReactionLifecycleGestureV1};
use ferrum_document_render::{
    PreparedReactionTranslationV1, ReactionTranslationGestureV1, ReactionTranslationPreviewV1,
};
use ferrum_document_render::{PreparedReactionV1, ReactionGestureV1};

#[derive(Debug)]
pub struct ApiReactionGestureV1(ReactionGestureV1);
pub struct ApiPreparedReactionV1(PreparedReactionV1);
#[derive(Debug)]
pub struct ApiReactionLifecycleGestureV1(ReactionLifecycleGestureV1);
pub struct ApiPreparedReactionLifecycleV1(PreparedReactionLifecycleV1);
#[derive(Debug)]
pub struct ApiReactionTranslationGestureV1(ReactionTranslationGestureV1);
#[derive(Debug)]
pub struct ApiReactionTranslationPreviewV1(ReactionTranslationPreviewV1);
pub struct ApiPreparedReactionTranslationV1(PreparedReactionTranslationV1);
impl std::fmt::Debug for ApiPreparedReactionV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Keep the public facade's diagnostics capability-safe even if the
        // render bridge later gains private receipt fields.
        formatter
            .debug_tuple("ApiPreparedReactionV1")
            .field(&self.0)
            .finish()
    }
}

pub fn begin_api_reaction_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    request: ReactionCreateRequestV1,
) -> Result<ApiReactionGestureV1, ReactionGestureErrorV1> {
    ferrum_document_render::begin_reaction_gesture_v1(session, fence, request)
        .map(ApiReactionGestureV1)
}
pub fn prepare_api_reaction_gesture_v1(
    session: &mut DocumentSession,
    gesture: &ApiReactionGestureV1,
) -> Result<ApiPreparedReactionV1, ReactionGestureErrorV1> {
    ferrum_document_render::prepare_reaction_gesture_v1(session, &gesture.0)
        .map(ApiPreparedReactionV1)
}
pub fn commit_api_reaction_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut ApiPreparedReactionV1,
) -> Result<CommittedReactionV1, ReactionGestureErrorV1> {
    ferrum_document_render::commit_reaction_gesture_v1(session, &mut prepared.0)
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
pub fn prepare_api_reaction_lifecycle_v1(
    session: &mut RenderInteractionSessionV1,
    gesture: &ApiReactionLifecycleGestureV1,
) -> Result<ApiPreparedReactionLifecycleV1, ReactionGestureErrorV1> {
    ferrum_document_render::prepare_reaction_lifecycle_v1(session, &gesture.0)
        .map(ApiPreparedReactionLifecycleV1)
}
pub fn commit_api_reaction_lifecycle_v1(
    session: &mut RenderInteractionSessionV1,
    prepared: &mut ApiPreparedReactionLifecycleV1,
) -> Result<CommittedReactionLifecycleV1, ReactionGestureErrorV1> {
    ferrum_document_render::commit_reaction_lifecycle_v1(session, &mut prepared.0)
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
pub fn preview_api_reaction_translation_v1(
    session: &RenderInteractionSessionV1,
    gesture: &ApiReactionTranslationGestureV1,
    pointer_x: f64,
    pointer_y: f64,
) -> Result<ApiReactionTranslationPreviewV1, ReactionGestureErrorV1> {
    ferrum_document_render::preview_reaction_translation_v1(
        session, &gesture.0, pointer_x, pointer_y,
    )
    .map(ApiReactionTranslationPreviewV1)
}
pub fn prepare_api_reaction_translation_v1(
    session: &mut RenderInteractionSessionV1,
    gesture: &ApiReactionTranslationGestureV1,
    preview: &ApiReactionTranslationPreviewV1,
) -> Result<ApiPreparedReactionTranslationV1, ReactionGestureErrorV1> {
    ferrum_document_render::prepare_reaction_translation_v1(session, &gesture.0, &preview.0)
        .map(ApiPreparedReactionTranslationV1)
}
pub fn commit_api_reaction_translation_v1(
    session: &mut RenderInteractionSessionV1,
    prepared: &mut ApiPreparedReactionTranslationV1,
) -> Result<ferrum_document_render::CommittedReactionTranslationV1, ReactionGestureErrorV1> {
    ferrum_document_render::commit_reaction_translation_v1(session, &mut prepared.0)
}

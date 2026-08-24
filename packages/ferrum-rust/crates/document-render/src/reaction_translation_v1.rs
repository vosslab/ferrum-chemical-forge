//! Renderer-admitted translation of one complete strict reaction aggregate.

#[cfg(test)]
use ferrum_document::DocumentSession;
use ferrum_document::{
    AuthoringCapabilityV1, DocumentFenceV1, SessionOperation, SessionOperationTransitionRequestV1,
    SessionOperationV1, TopLevelRootSelectorV1, TopLevelRootTranslationV1,
};

use crate::reaction_observation_v1::selected_reaction_member_ids_v1;
use crate::{
    ReactionGestureErrorV1, ReactionSelectionV1, RenderInteractionErrorV1,
    RenderInteractionSessionV1, RenderInteractionSnapV1, RenderInteractionTranslationGestureV1,
    RenderInteractionTranslationPreviewV1,
};

#[derive(Debug)]
pub struct ReactionTranslationGestureV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    reaction_id: String,
    membership_digest: String,
    targets: Vec<TopLevelRootSelectorV1>,
    translation: RenderInteractionTranslationGestureV1,
}
#[derive(Clone, Debug)]
pub struct ReactionTranslationPreviewV1 {
    preview: RenderInteractionTranslationPreviewV1,
}
impl ReactionTranslationPreviewV1 {
    #[must_use]
    pub const fn dx(&self) -> f64 {
        self.preview.dx()
    }
    #[must_use]
    pub const fn dy(&self) -> f64 {
        self.preview.dy()
    }
}
fn selection_error(error: RenderInteractionErrorV1) -> ReactionGestureErrorV1 {
    match error {
        RenderInteractionErrorV1::ForeignSession => ReactionGestureErrorV1::ForeignSession,
        RenderInteractionErrorV1::StaleRevision | RenderInteractionErrorV1::StaleDigest => {
            ReactionGestureErrorV1::StaleSnapshot
        }
        RenderInteractionErrorV1::DisplayOnly => {
            ReactionGestureErrorV1::LegacyDefinitionNotEditable
        }
        RenderInteractionErrorV1::SelectionChanged => ReactionGestureErrorV1::MembershipChanged,
        RenderInteractionErrorV1::SessionConflict => ReactionGestureErrorV1::SessionConflict,
        _ => ReactionGestureErrorV1::RendererExclusion,
    }
}
fn require_definition(
    session: &RenderInteractionSessionV1,
    reaction_id: &str,
    membership_digest: &str,
) -> Result<(), ReactionGestureErrorV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| ReactionGestureErrorV1::SessionConflict)?;
    let definitions = ferrum_document::inspect_direct_reactions_v1(snapshot.cdml())
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    let definition = definitions
        .into_iter()
        .find(|value| value.identifier() == Some(reaction_id))
        .ok_or(ReactionGestureErrorV1::MissingReaction)?;
    if !definition.is_strict() {
        return Err(ReactionGestureErrorV1::LegacyDefinitionNotEditable);
    }
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    reaction_id.hash(&mut hasher);
    for member in definition.members() {
        member.role().hash(&mut hasher);
        member.identifier().hash(&mut hasher);
    }
    (format!("{:016x}", hasher.finish()) == membership_digest)
        .then_some(())
        .ok_or(ReactionGestureErrorV1::MembershipChanged)
}

pub fn begin_reaction_translation_v1(
    session: &RenderInteractionSessionV1,
    selection: &ReactionSelectionV1,
    press_x: f64,
    press_y: f64,
    snap: RenderInteractionSnapV1,
) -> Result<ReactionTranslationGestureV1, ReactionGestureErrorV1> {
    session
        .validate_reaction_selection_v1(selection)
        .map_err(selection_error)?;
    let ids =
        selected_reaction_member_ids_v1(session, session.render_interaction_origin_v1(), selection)
            .map_err(selection_error)?;
    let observation = session
        .observe_render_interaction_v1(selection.fence())
        .map_err(selection_error)?;
    let mut selected = None;
    for identifier in ids {
        selected = Some(
            session
                .select_render_interaction_roots_v1(
                    &observation,
                    selected.as_ref(),
                    crate::RenderInteractionQueryV1::Root {
                        identifier,
                        modifier: if selected.is_some() {
                            crate::RenderInteractionModifierV1::Toggle
                        } else {
                            crate::RenderInteractionModifierV1::Replace
                        },
                    },
                )
                .map_err(selection_error)?,
        );
    }
    let selected = selected.ok_or(ReactionGestureErrorV1::MembershipChanged)?;
    let targets = selected
        .roots()
        .iter()
        .map(|root| {
            TopLevelRootSelectorV1::new(root.identifier().to_owned(), root.kind())
                .map_err(|_| ReactionGestureErrorV1::MembershipChanged)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let translation = session
        .begin_render_interaction_translation_v1(&selected, press_x, press_y, snap)
        .map_err(selection_error)?;
    Ok(ReactionTranslationGestureV1 {
        capability: session.issue_authoring_capability_v1(),
        fence: selection.fence(),
        reaction_id: selection.reaction_id().to_owned(),
        membership_digest: selection.membership_digest().to_owned(),
        targets,
        translation,
    })
}
pub fn preview_reaction_translation_v1(
    session: &RenderInteractionSessionV1,
    gesture: &ReactionTranslationGestureV1,
    pointer_x: f64,
    pointer_y: f64,
) -> Result<ReactionTranslationPreviewV1, ReactionGestureErrorV1> {
    require_definition(session, &gesture.reaction_id, &gesture.membership_digest)?;
    session
        .preview_render_interaction_translation_v1(&gesture.translation, pointer_x, pointer_y)
        .map(|preview| ReactionTranslationPreviewV1 { preview })
        .map_err(selection_error)
}
/// Resolve one translation gesture into the opaque generic session request.
pub fn resolve_reaction_translation_v1(
    session: &RenderInteractionSessionV1,
    gesture: ReactionTranslationGestureV1,
    pointer_x: f64,
    pointer_y: f64,
) -> Result<SessionOperationTransitionRequestV1, ReactionGestureErrorV1> {
    require_definition(session, &gesture.reaction_id, &gesture.membership_digest)?;
    let preview = session
        .preview_render_interaction_translation_v1(&gesture.translation, pointer_x, pointer_y)
        .map_err(selection_error)?;
    let transform =
        TopLevelRootTranslationV1::new(gesture.targets.clone(), preview.dx(), preview.dy())
            .map_err(|_| ReactionGestureErrorV1::MembershipChanged)?;
    Ok(SessionOperationTransitionRequestV1::new(
        gesture.fence.revision(),
        SessionOperation::V1(SessionOperationV1::TranslateTopLevelRootsV1(transform)),
        ferrum_document::TransitionAuthorizationV1::authoring_capability(gesture.capability),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::AdmittedSessionTransitionRefusalV1;

    const SOURCE: &str = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"left\"><atom id=\"la\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"right\"><atom id=\"ra\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule>",
        "<arrow id=\"a\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow>",
        "<reaction id=\"r\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction></cdml>"
    );

    fn selection(session: &RenderInteractionSessionV1) -> ReactionSelectionV1 {
        let snapshot = session.snapshot().expect("snapshot");
        let list = session
            .observe_reaction_list_v1(DocumentFenceV1::new(
                snapshot.revision(),
                *snapshot.digest(),
            ))
            .expect("list");
        session.select_reaction_v1(&list, "r").expect("selection")
    }

    #[test]
    fn translates_all_members_once_and_undo_restores_the_complete_aggregate() {
        let mut session =
            RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
        let selected = selection(&session);
        let gesture = begin_reaction_translation_v1(
            &session,
            &selected,
            0.0,
            0.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin");
        let preview =
            preview_reaction_translation_v1(&session, &gesture, 10.0, 5.0).expect("preview");
        assert_eq!((preview.dx(), preview.dy()), (10.0, 5.0));
        let request =
            resolve_reaction_translation_v1(&session, gesture, preview.dx(), preview.dy())
                .expect("resolve");
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("prepare");
        let committed = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("commit");
        let cdml = committed.observation().snapshot().cdml();
        assert_ne!(cdml, SOURCE);
        assert!(cdml.contains("<reaction id=\"r\""));
        assert!(matches!(
            session.commit_session_operation_transition_v1(&mut prepared),
            Err(AdmittedSessionTransitionRefusalV1::Replayed)
        ));
        let undone = session.undo(1).expect("undo");
        assert_eq!(undone.observation().snapshot().cdml(), SOURCE);
    }

    #[test]
    fn foreign_translation_refusal_does_not_mutate_either_session() {
        let owner = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("owner"));
        let foreign =
            RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("foreign"));
        let selected = selection(&owner);
        let gesture = begin_reaction_translation_v1(
            &owner,
            &selected,
            0.0,
            0.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin");
        let before = foreign.snapshot().expect("snapshot");
        assert!(matches!(
            preview_reaction_translation_v1(&foreign, &gesture, 5.0, 0.0),
            Err(ReactionGestureErrorV1::ForeignSession)
        ));
        assert_eq!(foreign.snapshot().expect("snapshot"), before);
    }
}

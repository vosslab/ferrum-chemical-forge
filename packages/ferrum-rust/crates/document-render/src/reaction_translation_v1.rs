//! Renderer-admitted translation of one complete durable reaction aggregate.

use ferrum_document::{
    AuthoringCapabilityV1, DocumentFenceV1, DocumentReactionMemberSelectionV1,
    ReactionMemberSelectionRefusalV1, SessionOperation, SessionOperationTransitionRequestV1,
    SessionOperationV1, TopLevelRootSelectorV1, TopLevelRootTranslationV1,
};

use crate::{
    RenderInteractionErrorV1, RenderInteractionSessionV1, RenderInteractionSnapV1,
    RenderInteractionTranslationGestureV1, RenderInteractionTranslationPreviewV1,
};

#[derive(Debug)]
pub struct ReactionTranslationGestureV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    selection: DocumentReactionMemberSelectionV1,
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

fn validate_document_selection(
    session: &RenderInteractionSessionV1,
    selection: &DocumentReactionMemberSelectionV1,
) -> Result<(), RenderInteractionErrorV1> {
    session
        .resolve_reaction_member_root_selectors_v1(selection)
        .map(drop)
        .map_err(document_selection_error_v1)
}

fn document_selection_error_v1(
    error: ReactionMemberSelectionRefusalV1,
) -> RenderInteractionErrorV1 {
    match error {
        ReactionMemberSelectionRefusalV1::ForeignSession => {
            RenderInteractionErrorV1::ForeignSession
        }
        ReactionMemberSelectionRefusalV1::StaleRevision => RenderInteractionErrorV1::StaleRevision,
        ReactionMemberSelectionRefusalV1::StaleDigest => RenderInteractionErrorV1::StaleDigest,
        ReactionMemberSelectionRefusalV1::UnknownReaction
        | ReactionMemberSelectionRefusalV1::WrongReactionKind
        | ReactionMemberSelectionRefusalV1::UnresolvedReaction
        | ReactionMemberSelectionRefusalV1::UnresolvedMember
        | ReactionMemberSelectionRefusalV1::MembershipMismatch => {
            RenderInteractionErrorV1::SelectionChanged
        }
    }
}

pub fn begin_reaction_translation_v1(
    session: &RenderInteractionSessionV1,
    selection: DocumentReactionMemberSelectionV1,
    press_x: f64,
    press_y: f64,
    snap: RenderInteractionSnapV1,
) -> Result<ReactionTranslationGestureV1, RenderInteractionErrorV1> {
    let member_object_ids = session
        .resolve_reaction_member_root_selectors_v1(&selection)
        .map_err(document_selection_error_v1)?;
    let snapshot = session
        .snapshot()
        .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
    let fence = DocumentFenceV1::new(snapshot.revision(), *snapshot.digest());
    let observation = session.observe_render_interaction_v1(fence)?;
    let mut selected = None;
    for document_object_id in member_object_ids {
        selected = Some(session.select_render_interaction_roots_v1(
            &observation,
            selected.as_ref(),
            crate::RenderInteractionQueryV1::Root {
                document_object_id: document_object_id.clone(),
                modifier: if selected.is_some() {
                    crate::RenderInteractionModifierV1::Toggle
                } else {
                    crate::RenderInteractionModifierV1::Replace
                },
            },
        )?);
    }
    let selected = selected.ok_or(RenderInteractionErrorV1::SelectionChanged)?;
    let targets = selected
        .roots()
        .iter()
        .map(|root| TopLevelRootSelectorV1::new(root.document_object_id().clone(), root.kind()))
        .collect();
    let translation =
        session.begin_render_interaction_translation_v1(&selected, press_x, press_y, snap)?;
    Ok(ReactionTranslationGestureV1 {
        capability: session.issue_authoring_capability_v1(),
        fence,
        selection,
        targets,
        translation,
    })
}

pub fn preview_reaction_translation_v1(
    session: &RenderInteractionSessionV1,
    gesture: &ReactionTranslationGestureV1,
    pointer_x: f64,
    pointer_y: f64,
) -> Result<ReactionTranslationPreviewV1, RenderInteractionErrorV1> {
    validate_document_selection(session, &gesture.selection)?;
    session
        .preview_render_interaction_translation_v1(&gesture.translation, pointer_x, pointer_y)
        .map(|preview| ReactionTranslationPreviewV1 { preview })
}

pub fn resolve_reaction_translation_v1(
    session: &RenderInteractionSessionV1,
    gesture: ReactionTranslationGestureV1,
    pointer_x: f64,
    pointer_y: f64,
) -> Result<SessionOperationTransitionRequestV1, RenderInteractionErrorV1> {
    validate_document_selection(session, &gesture.selection)?;
    let preview = session.preview_render_interaction_translation_v1(
        &gesture.translation,
        pointer_x,
        pointer_y,
    )?;
    let transform =
        TopLevelRootTranslationV1::new(gesture.targets.clone(), preview.dx(), preview.dy())
            .map_err(|_| RenderInteractionErrorV1::SelectionChanged)?;
    Ok(SessionOperationTransitionRequestV1::new(
        gesture.fence.revision(),
        SessionOperation::V1(SessionOperationV1::TranslateTopLevelRootsV1(transform)),
        ferrum_document::TransitionAuthorizationV1::authoring_capability(gesture.capability),
    ))
}

#[cfg(test)]
mod tests {
    use ferrum_document::{AdmittedSessionTransitionRefusalV1, DocumentSession};

    use super::*;

    const SOURCE: &str = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:object=\"urn:ferrum:document-object:v1\"><molecule id=\"left\"><atom id=\"la\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"right\"><atom id=\"ra\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule>",
        "<arrow id=\"a\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow>",
        "<reaction id=\"r\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000001\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction></cdml>"
    );

    #[test]
    fn translates_durable_members_and_undo_restores_the_aggregate() {
        let mut session =
            RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
        let list = session.observe_reaction_list_v1().expect("list");
        let reaction = list.reactions().first().expect("reaction");
        let selection = session
            .select_reaction_members_v1(
                reaction
                    .selection_observation()
                    .expect("strict reaction selection"),
            )
            .expect("selection");
        let gesture = begin_reaction_translation_v1(
            &session,
            selection,
            0.0,
            0.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin");
        let preview =
            preview_reaction_translation_v1(&session, &gesture, 10.0, 5.0).expect("preview");
        let request =
            resolve_reaction_translation_v1(&session, gesture, preview.dx(), preview.dy())
                .expect("resolve");
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("prepare");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("commit");
        assert!(matches!(
            session.commit_session_operation_transition_v1(&mut prepared),
            Err(AdmittedSessionTransitionRefusalV1::Replayed)
        ));
        let _undone = session.undo(1).expect("undo");
        assert!(
            session
                .observe_reaction_list_v1()
                .expect("reaction list after undo")
                .reactions()
                .first()
                .expect("reaction")
                .selection_observation()
                .is_some()
        );
    }
}

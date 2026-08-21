//! Renderer-admitted translation of one complete strict reaction aggregate.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use ferrum_document::{
    DocumentFenceV1, DocumentSession, SessionOperation, SessionOperationResultV1,
    SessionOperationV1, TopLevelRootSelectorV1, TopLevelTransformModeV1, TopLevelTransformV1,
};
use ferrum_render::{
    DocumentRenderOutcomeV1, DocumentRenderPlanV1, compose_document_render_plan_v1,
    document_observation_from_accepted_operation_v1,
};

use crate::reaction_gesture_v1::ReactionSessionOriginV1;
use crate::reaction_observation_v1::selected_reaction_member_ids_v1;
use crate::{
    ReactionGestureErrorV1, ReactionSelectionV1, RenderInteractionErrorV1,
    RenderInteractionSessionV1, RenderInteractionSnapV1, RenderInteractionTranslationGestureV1,
    RenderInteractionTranslationPreviewV1,
};

#[derive(Debug)]
pub struct ReactionTranslationGestureV1 {
    origin: ReactionSessionOriginV1,
    nonce: u64,
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
pub struct PreparedReactionTranslationV1 {
    receipt: Option<ReactionTranslationReceiptV1>,
    reaction_id: String,
}
impl std::fmt::Debug for PreparedReactionTranslationV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedReactionTranslationV1")
            .field("reaction_id", &self.reaction_id)
            .field(
                "state",
                &if self.receipt.is_some() {
                    "prepared"
                } else {
                    "consumed"
                },
            )
            .finish()
    }
}
#[derive(Clone, Debug)]
pub struct CommittedReactionTranslationV1 {
    reaction_id: String,
    result: SessionOperationResultV1,
}
impl CommittedReactionTranslationV1 {
    #[must_use]
    pub fn reaction_id(&self) -> &str {
        &self.reaction_id
    }
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
}
struct ReactionTranslationReceiptV1 {
    origin: ReactionSessionOriginV1,
    nonce: u64,
    fence: DocumentFenceV1,
    reaction_id: String,
    membership_digest: String,
    candidate: String,
    candidate_digest: [u8; 32],
    contract: ferrum_render_contract::PreflightedDocumentRenderV1,
    plan: DocumentRenderPlanV1,
}

fn origin(session: &RenderInteractionSessionV1) -> ReactionSessionOriginV1 {
    ReactionSessionOriginV1(session.render_interaction_origin_v1())
}
fn consumed() -> &'static Mutex<HashSet<(ReactionSessionOriginV1, u64)>> {
    static VALUE: OnceLock<Mutex<HashSet<(ReactionSessionOriginV1, u64)>>> = OnceLock::new();
    VALUE.get_or_init(|| Mutex::new(HashSet::new()))
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
    static NEXT: AtomicU64 = AtomicU64::new(1);
    Ok(ReactionTranslationGestureV1 {
        origin: origin(session),
        nonce: NEXT.fetch_add(1, Ordering::Relaxed),
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
    if gesture.origin != origin(session) {
        return Err(ReactionGestureErrorV1::ForeignSession);
    }
    require_definition(session, &gesture.reaction_id, &gesture.membership_digest)?;
    session
        .preview_render_interaction_translation_v1(&gesture.translation, pointer_x, pointer_y)
        .map(|preview| ReactionTranslationPreviewV1 { preview })
        .map_err(selection_error)
}
pub fn prepare_reaction_translation_v1(
    session: &mut RenderInteractionSessionV1,
    gesture: &ReactionTranslationGestureV1,
    preview: &ReactionTranslationPreviewV1,
) -> Result<PreparedReactionTranslationV1, ReactionGestureErrorV1> {
    if gesture.origin != origin(session) {
        return Err(ReactionGestureErrorV1::ForeignSession);
    }
    if consumed()
        .lock()
        .expect("reaction translation receipt lock")
        .contains(&(gesture.origin, gesture.nonce))
    {
        return Err(ReactionGestureErrorV1::ReplayedGesture);
    }
    require_definition(session, &gesture.reaction_id, &gesture.membership_digest)?;
    session
        .validate_render_interaction_translation_preview_v1(&gesture.translation, &preview.preview)
        .map_err(selection_error)?;
    let source = session
        .snapshot()
        .map_err(|_| ReactionGestureErrorV1::SessionConflict)?
        .cdml()
        .to_owned();
    let transform = TopLevelTransformV1::new(
        gesture.targets.clone(),
        TopLevelTransformModeV1::Translate {
            dx: preview.dx(),
            dy: preview.dy(),
        },
    )
    .map_err(|_| ReactionGestureErrorV1::MembershipChanged)?;
    let mut candidate_session =
        DocumentSession::load(&source).map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    candidate_session
        .submit(
            0,
            SessionOperation::V1(SessionOperationV1::TransformTopLevelRoots { transform }),
        )
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    let candidate_snapshot = candidate_session
        .snapshot()
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    let candidate = candidate_snapshot.cdml().to_owned();
    let contract = ferrum_render_contract::preflight_complete_document_v1(&candidate)
        .map_err(|_| ReactionGestureErrorV1::UnrenderableDocument)?;
    let observation = candidate_session
        .observe(candidate_snapshot.revision())
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    let rendering = document_observation_from_accepted_operation_v1(&observation)
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    let plan = compose_document_render_plan_v1(&rendering)
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    if plan
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(ReactionGestureErrorV1::RendererExclusion);
    }
    Ok(PreparedReactionTranslationV1 {
        reaction_id: gesture.reaction_id.clone(),
        receipt: Some(ReactionTranslationReceiptV1 {
            origin: gesture.origin,
            nonce: gesture.nonce,
            fence: gesture.fence,
            reaction_id: gesture.reaction_id.clone(),
            membership_digest: gesture.membership_digest.clone(),
            candidate,
            candidate_digest: *candidate_snapshot.digest(),
            contract,
            plan,
        }),
    })
}
pub fn commit_reaction_translation_v1(
    session: &mut RenderInteractionSessionV1,
    prepared: &mut PreparedReactionTranslationV1,
) -> Result<CommittedReactionTranslationV1, ReactionGestureErrorV1> {
    let receipt = prepared
        .receipt
        .as_ref()
        .ok_or(ReactionGestureErrorV1::ReplayedGesture)?;
    if receipt.origin != origin(session) {
        return Err(ReactionGestureErrorV1::ForeignSession);
    }
    if consumed()
        .lock()
        .expect("reaction translation receipt lock")
        .contains(&(receipt.origin, receipt.nonce))
    {
        return Err(ReactionGestureErrorV1::ReplayedGesture);
    }
    require_definition(session, &receipt.reaction_id, &receipt.membership_digest)?;
    if receipt.contract.source() != receipt.candidate
        || receipt
            .plan
            .outcomes()
            .iter()
            .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(ReactionGestureErrorV1::RendererExclusion);
    }
    let candidate = DocumentSession::load(&receipt.candidate)
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    if *candidate
        .snapshot()
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?
        .digest()
        != receipt.candidate_digest
    {
        return Err(ReactionGestureErrorV1::RenderPreparation);
    }
    let result = session
        .commit_complete_cdml_transaction_v1(receipt.fence, &receipt.candidate)
        .map_err(|_| ReactionGestureErrorV1::SessionConflict)?;
    let used = prepared
        .receipt
        .take()
        .expect("successful translation retains receipt");
    consumed()
        .lock()
        .expect("reaction translation receipt lock")
        .insert((used.origin, used.nonce));
    Ok(CommittedReactionTranslationV1 {
        reaction_id: prepared.reaction_id.clone(),
        result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = concat!(
        "<cdml><molecule id=\"left\"><atom id=\"la\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
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
        let mut prepared =
            prepare_reaction_translation_v1(&mut session, &gesture, &preview).expect("prepare");
        let committed =
            commit_reaction_translation_v1(&mut session, &mut prepared).expect("commit");
        let cdml = committed.result().observation().snapshot().cdml();
        assert_ne!(cdml, SOURCE);
        assert!(cdml.contains("<reaction id=\"r\""));
        assert!(matches!(
            commit_reaction_translation_v1(&mut session, &mut prepared),
            Err(ReactionGestureErrorV1::ReplayedGesture)
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

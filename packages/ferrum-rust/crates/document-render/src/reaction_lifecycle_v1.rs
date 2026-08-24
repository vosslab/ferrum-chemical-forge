//! Renderer-admitted whole-reaction lifecycle transactions.
//!
//! This module has no XML-facing public API.  A caller can only name a frozen
//! reaction selector and provide complete typed role lists; the detached CDML,
//! render proof, and one-use receipt remain private to this bridge.

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use ferrum_document::{
    AuthoringCapabilityV1, DeleteReactionV1, DirectReactionMemberV1, DirectReactionRoleV1,
    DocumentFenceV1, DocumentSession, ReplaceReactionMembersV1, SessionOperation,
    SessionOperationTransitionRequestV1, SessionOperationV1, inspect_direct_reactions_v1,
};

use crate::reaction_gesture_v1::map_document_operation_error_v1;
use crate::{
    ReactionGestureErrorV1, ReactionSelectionV1, RenderInteractionErrorV1,
    RenderInteractionSessionV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReactionMembershipPatchRequestV1 {
    expected_revision: u64,
    reactants: Vec<String>,
    products: Vec<String>,
    arrow: String,
    conditions: Vec<String>,
    pluses: Vec<String>,
}
impl ReactionMembershipPatchRequestV1 {
    pub fn new(
        expected_revision: u64,
        reactants: Vec<String>,
        products: Vec<String>,
        arrow: String,
        conditions: Vec<String>,
        pluses: Vec<String>,
    ) -> Result<Self, ReactionGestureErrorV1> {
        let value = Self {
            expected_revision,
            reactants,
            products,
            arrow,
            conditions,
            pluses,
        };
        value.validate()?;
        Ok(value)
    }
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
    fn roles(&self) -> Vec<(DirectReactionRoleV1, String)> {
        let mut values = Vec::new();
        values.extend(
            self.reactants
                .iter()
                .cloned()
                .map(|id| (DirectReactionRoleV1::Reactant, id)),
        );
        values.extend(
            self.products
                .iter()
                .cloned()
                .map(|id| (DirectReactionRoleV1::Product, id)),
        );
        values.push((DirectReactionRoleV1::Arrow, self.arrow.clone()));
        values.extend(
            self.conditions
                .iter()
                .cloned()
                .map(|id| (DirectReactionRoleV1::Condition, id)),
        );
        values.extend(
            self.pluses
                .iter()
                .cloned()
                .map(|id| (DirectReactionRoleV1::Plus, id)),
        );
        values
    }
    fn validate(&self) -> Result<(), ReactionGestureErrorV1> {
        let values = self.roles();
        if self.reactants.is_empty()
            || self.products.is_empty()
            || values.iter().any(|(_, id)| id.trim().is_empty())
        {
            return Err(ReactionGestureErrorV1::InvalidRequest);
        }
        if values
            .iter()
            .map(|(_, id)| id)
            .collect::<HashSet<_>>()
            .len()
            != values.len()
        {
            return Err(ReactionGestureErrorV1::DuplicateTarget);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ReactionLifecycleGestureV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    reaction_id: String,
    membership_digest: String,
    operation: ReactionLifecycleOperationV1,
}
#[derive(Clone, Debug)]
enum ReactionLifecycleOperationV1 {
    Patch(ReactionMembershipPatchRequestV1),
    DeleteDefinition,
}
fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), ReactionGestureErrorV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| ReactionGestureErrorV1::SessionConflict)?;
    (snapshot.revision() == fence.revision() && snapshot.digest() == &fence.digest())
        .then_some(())
        .ok_or(ReactionGestureErrorV1::StaleSnapshot)
}
fn digest(reaction_id: &str, members: &[DirectReactionMemberV1]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    reaction_id.hash(&mut hasher);
    for member in members {
        member.role().hash(&mut hasher);
        member.identifier().hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}
fn validate_definition(
    source: &str,
    reaction_id: &str,
    expected: &str,
) -> Result<(), ReactionGestureErrorV1> {
    let definition = inspect_direct_reactions_v1(source)
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?
        .into_iter()
        .find(|value| value.identifier() == Some(reaction_id))
        .ok_or(ReactionGestureErrorV1::MissingReaction)?;
    if !definition.is_strict() {
        return Err(ReactionGestureErrorV1::LegacyDefinitionNotEditable);
    }
    (digest(reaction_id, definition.members()) == expected)
        .then_some(())
        .ok_or(ReactionGestureErrorV1::MembershipChanged)
}

fn lifecycle_selection_error(error: RenderInteractionErrorV1) -> ReactionGestureErrorV1 {
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
fn begin(
    session: &RenderInteractionSessionV1,
    selection: &ReactionSelectionV1,
    operation: ReactionLifecycleOperationV1,
) -> Result<ReactionLifecycleGestureV1, ReactionGestureErrorV1> {
    session
        .validate_reaction_selection_v1(selection)
        .map_err(lifecycle_selection_error)?;
    let fence = selection.fence();
    if let ReactionLifecycleOperationV1::Patch(request) = &operation
        && request.expected_revision() != fence.revision()
    {
        return Err(ReactionGestureErrorV1::StaleSnapshot);
    }
    Ok(ReactionLifecycleGestureV1 {
        capability: session.issue_authoring_capability_v1(),
        fence,
        reaction_id: selection.reaction_id().to_owned(),
        membership_digest: selection.membership_digest().to_owned(),
        operation,
    })
}
pub fn begin_reaction_membership_patch_v1(
    session: &RenderInteractionSessionV1,
    selection: &ReactionSelectionV1,
    request: ReactionMembershipPatchRequestV1,
) -> Result<ReactionLifecycleGestureV1, ReactionGestureErrorV1> {
    begin(
        session,
        selection,
        ReactionLifecycleOperationV1::Patch(request),
    )
}
pub fn begin_reaction_definition_delete_v1(
    session: &RenderInteractionSessionV1,
    selection: &ReactionSelectionV1,
) -> Result<ReactionLifecycleGestureV1, ReactionGestureErrorV1> {
    begin(
        session,
        selection,
        ReactionLifecycleOperationV1::DeleteDefinition,
    )
}
/// Consume one lifecycle gesture into the opaque generic session request.
pub fn resolve_reaction_lifecycle_v1(
    session: &RenderInteractionSessionV1,
    gesture: ReactionLifecycleGestureV1,
) -> Result<SessionOperationTransitionRequestV1, ReactionGestureErrorV1> {
    require_fence(session, gesture.fence)?;
    let source = session
        .snapshot()
        .map_err(|_| ReactionGestureErrorV1::SessionConflict)?
        .cdml()
        .to_owned();
    validate_definition(&source, &gesture.reaction_id, &gesture.membership_digest)?;
    let operation = match &gesture.operation {
        ReactionLifecycleOperationV1::Patch(request) => {
            ReplaceReactionMembersV1::new(gesture.reaction_id.clone(), request.roles())
                .map(SessionOperationV1::ReplaceReactionMembersV1)
                .map_err(|error| {
                    map_document_operation_error_v1(
                        ferrum_document::DocumentSessionError::Operation(error.into()),
                    )
                })?
        }
        ReactionLifecycleOperationV1::DeleteDefinition => {
            DeleteReactionV1::new(gesture.reaction_id.clone())
                .map(SessionOperationV1::DeleteReactionV1)
                .map_err(|error| {
                    map_document_operation_error_v1(
                        ferrum_document::DocumentSessionError::Operation(error.into()),
                    )
                })?
        }
    };
    Ok(SessionOperationTransitionRequestV1::new(
        gesture.fence.revision(),
        SessionOperation::V1(operation),
        ferrum_document::TransitionAuthorizationV1::authoring_capability(gesture.capability),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::{AdmittedSessionTransitionRefusalV1, SessionOperationOutcomeV1};

    const SOURCE: &str = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"left\"><atom id=\"la\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"right\"><atom id=\"ra\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"third\"><atom id=\"ta\" name=\"N\"><point x=\"140\" y=\"0\"/></atom></molecule>",
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
            .expect("reaction list");
        session.select_reaction_v1(&list, "r").expect("selection")
    }

    #[test]
    fn patch_replaces_all_members_in_one_renderer_admitted_transaction() {
        let mut session =
            RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
        let selection = selection(&session);
        let request = ReactionMembershipPatchRequestV1::new(
            0,
            vec!["left".into()],
            vec!["third".into()],
            "a".into(),
            vec![],
            vec![],
        )
        .expect("request");
        let gesture =
            begin_reaction_membership_patch_v1(&session, &selection, request).expect("begin");
        let request = resolve_reaction_lifecycle_v1(&session, gesture).expect("resolve");
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("prepare");
        let committed = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("commit");
        assert!(matches!(
            committed.outcome(),
            SessionOperationOutcomeV1::ReactionMembershipReplacedV1(outcome)
                if outcome.reaction_id() == "r"
        ));
        assert_eq!(committed.observation().snapshot().revision(), 1);
        assert!(
            committed
                .observation()
                .snapshot()
                .cdml()
                .contains("product idref=\"third\"")
        );
        assert!(matches!(
            session.commit_session_operation_transition_v1(&mut prepared),
            Err(AdmittedSessionTransitionRefusalV1::Replayed)
        ));
    }

    #[test]
    fn definition_delete_preserves_members_in_one_transaction() {
        let mut session =
            RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
        let selection = selection(&session);
        let gesture = begin_reaction_definition_delete_v1(&session, &selection).expect("begin");
        let request = resolve_reaction_lifecycle_v1(&session, gesture).expect("resolve");
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("prepare");
        let committed = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("commit");
        assert!(matches!(
            committed.outcome(),
            SessionOperationOutcomeV1::ReactionDefinitionDeletedV1(outcome)
                if outcome.reaction_id() == "r"
        ));
        let cdml = committed.observation().snapshot().cdml();
        assert!(!cdml.contains("<reaction"));
        assert!(cdml.contains("molecule id=\"left\""));
        assert!(cdml.contains("arrow id=\"a\""));
        let undone = session.undo(1).expect("undo definition deletion");
        assert!(
            undone
                .observation()
                .snapshot()
                .cdml()
                .contains("<reaction id=\"r\"")
        );
    }

    #[test]
    fn lifecycle_selection_refuses_foreign_and_stale_capabilities_without_mutation() {
        let mut owner =
            RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("owner load"));
        let foreign =
            RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("foreign load"));
        let foreign_selection = selection(&owner);
        let request = ReactionMembershipPatchRequestV1::new(
            0,
            vec!["left".into()],
            vec!["third".into()],
            "a".into(),
            vec![],
            vec![],
        )
        .expect("request");
        let foreign_before = foreign.snapshot().expect("foreign snapshot");
        assert!(matches!(
            begin_reaction_membership_patch_v1(&foreign, &foreign_selection, request),
            Err(ReactionGestureErrorV1::ForeignSession)
        ));
        assert_eq!(
            foreign.snapshot().expect("foreign unchanged").digest(),
            foreign_before.digest()
        );

        let committed_selection = selection(&owner);
        let committed_request = ReactionMembershipPatchRequestV1::new(
            0,
            vec!["left".into()],
            vec!["third".into()],
            "a".into(),
            vec![],
            vec![],
        )
        .expect("committed request");
        let gesture =
            begin_reaction_membership_patch_v1(&owner, &committed_selection, committed_request)
                .expect("begin current selection");
        let request = resolve_reaction_lifecycle_v1(&owner, gesture).expect("resolve");
        let mut prepared = owner
            .prepare_session_operation_transition_v1(request)
            .expect("prepare");
        owner
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("commit");
        let owner_before = owner.snapshot().expect("owner snapshot");
        let stale_request = ReactionMembershipPatchRequestV1::new(
            0,
            vec!["left".into()],
            vec!["right".into()],
            "a".into(),
            vec![],
            vec![],
        )
        .expect("stale request");
        assert!(matches!(
            begin_reaction_membership_patch_v1(&owner, &foreign_selection, stale_request),
            Err(ReactionGestureErrorV1::StaleSnapshot)
        ));
        assert_eq!(
            owner.snapshot().expect("owner unchanged").digest(),
            owner_before.digest()
        );
    }

    #[test]
    fn lifecycle_definition_refusals_are_closed_and_precise() {
        assert!(matches!(
            validate_definition(SOURCE, "missing", "unused"),
            Err(ReactionGestureErrorV1::MissingReaction)
        ));
        assert!(matches!(
            validate_definition(SOURCE, "r", "incorrect-membership"),
            Err(ReactionGestureErrorV1::MembershipChanged)
        ));
        let legacy = SOURCE.replace(
            "<reaction id=\"r\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction>",
            "<reaction id=\"r\"><reactant idref=\"left\"/></reaction>",
        );
        assert!(matches!(
            validate_definition(&legacy, "r", "unused"),
            Err(ReactionGestureErrorV1::LegacyDefinitionNotEditable)
        ));
    }

    #[test]
    fn failed_foreign_commit_keeps_receipt_retryable_without_mutation() {
        let mut owner =
            RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("owner load"));
        let mut foreign =
            RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("foreign load"));
        let selected = selection(&owner);
        let request = ReactionMembershipPatchRequestV1::new(
            0,
            vec!["left".into()],
            vec!["third".into()],
            "a".into(),
            vec![],
            vec![],
        )
        .expect("request");
        let gesture =
            begin_reaction_membership_patch_v1(&owner, &selected, request).expect("begin");
        let request = resolve_reaction_lifecycle_v1(&owner, gesture).expect("resolve");
        let mut prepared = owner
            .prepare_session_operation_transition_v1(request)
            .expect("prepare");
        let foreign_before = foreign.snapshot().expect("foreign snapshot");
        assert!(matches!(
            foreign.commit_session_operation_transition_v1(&mut prepared),
            Err(AdmittedSessionTransitionRefusalV1::ForeignSession)
        ));
        assert_eq!(
            foreign.snapshot().expect("foreign unchanged").digest(),
            foreign_before.digest()
        );
        let committed = owner
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("owner retry");
        assert_eq!(committed.observation().snapshot().revision(), 1);
    }
}

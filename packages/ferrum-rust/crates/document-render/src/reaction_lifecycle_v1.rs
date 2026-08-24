//! Renderer-admitted whole-reaction lifecycle transactions.
//!
//! This module has no XML-facing public API.  A caller can only name a frozen
//! reaction selector and provide complete typed role lists; the detached CDML,
//! render proof, and one-use receipt remain private to this bridge.

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use ferrum_document::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityV1, DirectCdmlRootKindV1,
    DirectCdmlSemanticIndexV1, DirectReactionMemberV1, DirectReactionRoleV1, DocumentFenceV1,
    DocumentSession, PendingCompleteCdmlMutationV1, SessionOperationResultV1,
    delete_direct_cdml_reaction_definition_v1, inspect_direct_reactions_v1,
    replace_direct_cdml_reaction_members_v1,
};

use crate::reaction_gesture_v1::map_complete_cdml_mutation_refusal_v1;
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
pub struct PreparedReactionLifecycleV1 {
    pending: Option<PendingCompleteCdmlMutationV1>,
    capability: AuthoringCapabilityV1,
    reaction_id: String,
    membership_digest: String,
}
impl std::fmt::Debug for PreparedReactionLifecycleV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedReactionLifecycleV1")
            .field("reaction_id", &self.reaction_id)
            .field(
                "state",
                &if self.pending.is_some() {
                    "prepared"
                } else {
                    "consumed"
                },
            )
            .finish()
    }
}
#[derive(Clone, Debug)]
pub struct CommittedReactionLifecycleV1 {
    reaction_id: String,
    result: SessionOperationResultV1,
}
impl CommittedReactionLifecycleV1 {
    #[must_use]
    pub fn reaction_id(&self) -> &str {
        &self.reaction_id
    }
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
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
fn compile_patch(
    source: &str,
    reaction_id: &str,
    request: &ReactionMembershipPatchRequestV1,
) -> Result<String, ReactionGestureErrorV1> {
    request.validate()?;
    let index = DirectCdmlSemanticIndexV1::parse(source)
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    for (role, id) in request.roles() {
        let target = index
            .roots()
            .iter()
            .find(|root| root.identifier() == Some(id.as_str()))
            .ok_or(ReactionGestureErrorV1::MissingTarget)?;
        let kind = match role {
            DirectReactionRoleV1::Reactant | DirectReactionRoleV1::Product => {
                DirectCdmlRootKindV1::Molecule
            }
            DirectReactionRoleV1::Arrow => DirectCdmlRootKindV1::Arrow,
            DirectReactionRoleV1::Condition => DirectCdmlRootKindV1::Text,
            DirectReactionRoleV1::Plus => DirectCdmlRootKindV1::Plus,
        };
        if target.kind() != kind {
            return Err(ReactionGestureErrorV1::WrongTargetKind);
        }
        if index.roots().iter().any(|root| {
            root.kind() == DirectCdmlRootKindV1::Reaction
                && root.identifier() != Some(reaction_id)
                && root.reaction_members().iter().any(|member| member == &id)
        }) {
            return Err(ReactionGestureErrorV1::CrossReactionReuse);
        }
    }
    replace_direct_cdml_reaction_members_v1(source, reaction_id, &request.roles())
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)
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
        capability: session.authoring_capability_issuer_v1().issue(),
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
pub fn prepare_reaction_lifecycle_v1(
    session: &mut RenderInteractionSessionV1,
    gesture: &ReactionLifecycleGestureV1,
) -> Result<PreparedReactionLifecycleV1, ReactionGestureErrorV1> {
    if !gesture
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
    {
        return Err(ReactionGestureErrorV1::ForeignSession);
    }
    match gesture
        .capability
        .claim_for_commit(&session.authoring_capability_issuer_v1())
    {
        Ok(claim) => drop(claim),
        Err(AuthoringCapabilityAccessErrorV1::ForeignSession) => {
            return Err(ReactionGestureErrorV1::ForeignSession);
        }
        Err(AuthoringCapabilityAccessErrorV1::Replayed) => {
            return Err(ReactionGestureErrorV1::ReplayedGesture);
        }
    }
    require_fence(session, gesture.fence)?;
    let source = session
        .snapshot()
        .map_err(|_| ReactionGestureErrorV1::SessionConflict)?
        .cdml()
        .to_owned();
    validate_definition(&source, &gesture.reaction_id, &gesture.membership_digest)?;
    let candidate = match &gesture.operation {
        ReactionLifecycleOperationV1::Patch(request) => {
            compile_patch(&source, &gesture.reaction_id, request)?
        }
        ReactionLifecycleOperationV1::DeleteDefinition => {
            delete_direct_cdml_reaction_definition_v1(&source, &gesture.reaction_id)
                .map_err(|_| ReactionGestureErrorV1::InvalidRequest)?
        }
    };
    if candidate == source {
        return Err(ReactionGestureErrorV1::InvalidRequest);
    }
    let pending = session
        .prepare_complete_cdml_mutation_v1(gesture.fence, &candidate)
        .map_err(map_complete_cdml_mutation_refusal_v1)?;
    Ok(PreparedReactionLifecycleV1 {
        reaction_id: gesture.reaction_id.clone(),
        membership_digest: gesture.membership_digest.clone(),
        pending: Some(pending),
        capability: gesture.capability.clone(),
    })
}
pub fn commit_reaction_lifecycle_v1(
    session: &mut RenderInteractionSessionV1,
    prepared: &mut PreparedReactionLifecycleV1,
) -> Result<CommittedReactionLifecycleV1, ReactionGestureErrorV1> {
    let mut pending = prepared
        .pending
        .take()
        .ok_or(ReactionGestureErrorV1::ReplayedGesture)?;
    if !prepared
        .capability
        .belongs_to(&session.authoring_capability_issuer_v1())
    {
        prepared.pending = Some(pending);
        return Err(ReactionGestureErrorV1::ForeignSession);
    }
    let claim = match prepared
        .capability
        .claim_for_commit(&session.authoring_capability_issuer_v1())
    {
        Ok(claim) => claim,
        Err(AuthoringCapabilityAccessErrorV1::ForeignSession) => {
            unreachable!("owner checked above")
        }
        Err(AuthoringCapabilityAccessErrorV1::Replayed) => {
            prepared.pending = Some(pending);
            return Err(ReactionGestureErrorV1::ReplayedGesture);
        }
    };
    let result = (|| {
        let source = session
            .snapshot()
            .map_err(|_| ReactionGestureErrorV1::SessionConflict)?
            .cdml()
            .to_owned();
        validate_definition(&source, &prepared.reaction_id, &prepared.membership_digest)?;
        session
            .commit_complete_cdml_mutation_v1(&mut pending)
            .map_err(map_complete_cdml_mutation_refusal_v1)
    })();
    match result {
        Ok(result) => {
            claim.consume();
            Ok(CommittedReactionLifecycleV1 {
                reaction_id: prepared.reaction_id.clone(),
                result,
            })
        }
        Err(error) => {
            prepared.pending = Some(pending);
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut prepared = prepare_reaction_lifecycle_v1(&mut session, &gesture).expect("prepare");
        let committed = commit_reaction_lifecycle_v1(&mut session, &mut prepared).expect("commit");
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
        assert!(
            committed
                .result()
                .observation()
                .snapshot()
                .cdml()
                .contains("product idref=\"third\"")
        );
        assert!(matches!(
            commit_reaction_lifecycle_v1(&mut session, &mut prepared),
            Err(ReactionGestureErrorV1::ReplayedGesture)
        ));
    }

    #[test]
    fn definition_delete_preserves_members_in_one_transaction() {
        let mut session =
            RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
        let selection = selection(&session);
        let gesture = begin_reaction_definition_delete_v1(&session, &selection).expect("begin");
        let mut prepared = prepare_reaction_lifecycle_v1(&mut session, &gesture).expect("prepare");
        let committed = commit_reaction_lifecycle_v1(&mut session, &mut prepared).expect("commit");
        let cdml = committed.result().observation().snapshot().cdml();
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
        let mut prepared = prepare_reaction_lifecycle_v1(&mut owner, &gesture).expect("prepare");
        commit_reaction_lifecycle_v1(&mut owner, &mut prepared).expect("commit");
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
        let mut prepared = prepare_reaction_lifecycle_v1(&mut owner, &gesture).expect("prepare");
        let foreign_before = foreign.snapshot().expect("foreign snapshot");
        assert!(matches!(
            commit_reaction_lifecycle_v1(&mut foreign, &mut prepared),
            Err(ReactionGestureErrorV1::ForeignSession)
        ));
        assert_eq!(
            foreign.snapshot().expect("foreign unchanged").digest(),
            foreign_before.digest()
        );
        let committed =
            commit_reaction_lifecycle_v1(&mut owner, &mut prepared).expect("owner retry");
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
    }
}

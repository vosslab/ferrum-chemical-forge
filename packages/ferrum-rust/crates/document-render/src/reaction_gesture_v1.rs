//! Route validation and semantic request construction for reaction creation.
//!
//! The renderer consumes route gestures into opaque generic transition requests.
//! The document owns preparation, post-preparation capability, and commit.

use std::collections::HashSet;

use ferrum_document::{
    AuthoringCapabilityV1, CreateReactionV1, DirectReactionRoleV1, DocumentFenceV1,
    DocumentSession, DocumentSessionError, ReactionOperationRefusalV1, SessionOperation,
    SessionOperationError, SessionOperationTransitionRequestV1, SessionOperationV1,
};
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReactionCreateRequestV1 {
    expected_revision: u64,
    reactants: Vec<String>,
    products: Vec<String>,
    arrow: String,
    conditions: Vec<String>,
    pluses: Vec<String>,
}
impl ReactionCreateRequestV1 {
    pub fn new(
        expected_revision: u64,
        reactants: Vec<String>,
        products: Vec<String>,
        arrow: String,
        conditions: Vec<String>,
        pluses: Vec<String>,
    ) -> Result<Self, ReactionGestureErrorV1> {
        let request = Self {
            expected_revision,
            reactants,
            products,
            arrow,
            conditions,
            pluses,
        };
        request.validate_syntax()?;
        Ok(request)
    }
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
    fn validate_syntax(&self) -> Result<(), ReactionGestureErrorV1> {
        let all = self
            .reactants
            .iter()
            .chain(&self.products)
            .chain(std::iter::once(&self.arrow))
            .chain(&self.conditions)
            .chain(&self.pluses);
        if self.reactants.is_empty()
            || self.products.is_empty()
            || all.clone().any(|id| id.trim().is_empty())
        {
            return Err(ReactionGestureErrorV1::InvalidRequest);
        }
        let values = all.collect::<Vec<_>>();
        if values.iter().collect::<HashSet<_>>().len() != values.len() {
            return Err(ReactionGestureErrorV1::DuplicateTarget);
        }
        Ok(())
    }
}

/// Opaque session-bound authoring capability. It deliberately has no public
/// fields, candidate conversion, serialization, clone, or dereference route.
#[derive(Debug)]
pub struct ReactionGestureV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    request: ReactionCreateRequestV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReactionGestureCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    ReplayedGesture,
    InvalidRequest,
    MissingTarget,
    WrongTargetKind,
    DuplicateTarget,
    CrossReactionReuse,
    UnrenderableDocument,
    RenderPreparation,
    SessionConflict,
    MissingReaction,
    LegacyDefinitionNotEditable,
    MembershipChanged,
    RendererExclusion,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReactionGestureRecoveryV1 {
    DocumentUnchanged,
    RefreshAndRestart,
    CorrectSelectors,
    ChooseRenderableMembers,
    RepairLegacyDefinition,
}
#[derive(Clone, Debug, Error, PartialEq)]
#[non_exhaustive]
pub enum ReactionGestureErrorV1 {
    #[error("reaction gesture snapshot is stale")]
    StaleSnapshot,
    #[error("reaction gesture belongs to another document session")]
    ForeignSession,
    #[error("reaction gesture was already committed")]
    ReplayedGesture,
    #[error("reaction membership request is invalid")]
    InvalidRequest,
    #[error("reaction member is missing")]
    MissingTarget,
    #[error("reaction member has the wrong direct-root kind")]
    WrongTargetKind,
    #[error("reaction member occurs more than once or has conflicting roles")]
    DuplicateTarget,
    #[error("reaction member already belongs to another reaction")]
    CrossReactionReuse,
    #[error("reaction candidate cannot be rendered completely")]
    UnrenderableDocument,
    #[error("reaction candidate renderer preflight failed")]
    RenderPreparation,
    #[error("reaction commit was rejected by the document session")]
    SessionConflict,
    #[error("selected reaction definition is missing")]
    MissingReaction,
    #[error("legacy reaction definition is display-only and cannot be edited")]
    LegacyDefinitionNotEditable,
    #[error("selected reaction membership changed")]
    MembershipChanged,
    #[error("reaction lifecycle candidate has a renderer exclusion")]
    RendererExclusion,
}
impl ReactionGestureErrorV1 {
    #[must_use]
    pub const fn category(&self) -> ReactionGestureCategoryV1 {
        match self {
            Self::StaleSnapshot => ReactionGestureCategoryV1::StaleSnapshot,
            Self::ForeignSession => ReactionGestureCategoryV1::ForeignSession,
            Self::ReplayedGesture => ReactionGestureCategoryV1::ReplayedGesture,
            Self::InvalidRequest => ReactionGestureCategoryV1::InvalidRequest,
            Self::MissingTarget => ReactionGestureCategoryV1::MissingTarget,
            Self::WrongTargetKind => ReactionGestureCategoryV1::WrongTargetKind,
            Self::DuplicateTarget => ReactionGestureCategoryV1::DuplicateTarget,
            Self::CrossReactionReuse => ReactionGestureCategoryV1::CrossReactionReuse,
            Self::UnrenderableDocument => ReactionGestureCategoryV1::UnrenderableDocument,
            Self::RenderPreparation => ReactionGestureCategoryV1::RenderPreparation,
            Self::SessionConflict => ReactionGestureCategoryV1::SessionConflict,
            Self::MissingReaction => ReactionGestureCategoryV1::MissingReaction,
            Self::LegacyDefinitionNotEditable => {
                ReactionGestureCategoryV1::LegacyDefinitionNotEditable
            }
            Self::MembershipChanged => ReactionGestureCategoryV1::MembershipChanged,
            Self::RendererExclusion => ReactionGestureCategoryV1::RendererExclusion,
        }
    }
    #[must_use]
    pub const fn recovery(&self) -> ReactionGestureRecoveryV1 {
        match self {
            Self::StaleSnapshot
            | Self::ForeignSession
            | Self::ReplayedGesture
            | Self::SessionConflict => ReactionGestureRecoveryV1::RefreshAndRestart,
            Self::InvalidRequest
            | Self::MissingTarget
            | Self::WrongTargetKind
            | Self::DuplicateTarget
            | Self::CrossReactionReuse => ReactionGestureRecoveryV1::CorrectSelectors,
            Self::UnrenderableDocument | Self::RenderPreparation => {
                ReactionGestureRecoveryV1::ChooseRenderableMembers
            }
            Self::MissingReaction | Self::MembershipChanged => {
                ReactionGestureRecoveryV1::RefreshAndRestart
            }
            Self::LegacyDefinitionNotEditable => ReactionGestureRecoveryV1::RepairLegacyDefinition,
            Self::RendererExclusion => ReactionGestureRecoveryV1::ChooseRenderableMembers,
        }
    }
}

pub(crate) fn map_document_operation_error_v1(
    error: DocumentSessionError,
) -> ReactionGestureErrorV1 {
    match error {
        DocumentSessionError::RevisionConflict { .. } => ReactionGestureErrorV1::StaleSnapshot,
        DocumentSessionError::RendererAdmission => ReactionGestureErrorV1::UnrenderableDocument,
        DocumentSessionError::Operation(SessionOperationError::Reaction(refusal)) => {
            match refusal {
                ReactionOperationRefusalV1::MissingRequiredMembers
                | ReactionOperationRefusalV1::EmptyMemberIdentifier
                | ReactionOperationRefusalV1::InvalidDefinition => {
                    ReactionGestureErrorV1::InvalidRequest
                }
                ReactionOperationRefusalV1::DuplicateMember => {
                    ReactionGestureErrorV1::DuplicateTarget
                }
                ReactionOperationRefusalV1::MissingMember => ReactionGestureErrorV1::MissingTarget,
                ReactionOperationRefusalV1::WrongMemberKind => {
                    ReactionGestureErrorV1::WrongTargetKind
                }
                ReactionOperationRefusalV1::CrossReactionReuse => {
                    ReactionGestureErrorV1::CrossReactionReuse
                }
            }
        }
        _ => ReactionGestureErrorV1::SessionConflict,
    }
}

fn create_members(request: &ReactionCreateRequestV1) -> Vec<(DirectReactionRoleV1, String)> {
    let roles = [
        (DirectReactionRoleV1::Reactant, request.reactants.as_slice()),
        (DirectReactionRoleV1::Product, request.products.as_slice()),
        (
            DirectReactionRoleV1::Arrow,
            std::slice::from_ref(&request.arrow),
        ),
        (
            DirectReactionRoleV1::Condition,
            request.conditions.as_slice(),
        ),
        (DirectReactionRoleV1::Plus, request.pluses.as_slice()),
    ];
    roles
        .into_iter()
        .flat_map(|(role, identifiers)| identifiers.iter().cloned().map(move |id| (role, id)))
        .collect()
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

pub fn begin_reaction_gesture_v1(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    request: ReactionCreateRequestV1,
) -> Result<ReactionGestureV1, ReactionGestureErrorV1> {
    require_fence(session, fence)?;
    if request.expected_revision() != fence.revision() {
        return Err(ReactionGestureErrorV1::StaleSnapshot);
    }
    Ok(ReactionGestureV1 {
        capability: session.issue_authoring_capability_v1(),
        fence,
        request,
    })
}
/// Consume one reaction route gesture into the opaque generic session request.
///
/// The route owns only its transient selector validation and semantic operation
/// construction. Document preparation creates the sole post-preparation value.
pub fn resolve_reaction_gesture_v1(
    session: &DocumentSession,
    gesture: ReactionGestureV1,
) -> Result<SessionOperationTransitionRequestV1, ReactionGestureErrorV1> {
    require_fence(session, gesture.fence)?;
    let operation = CreateReactionV1::new(create_members(&gesture.request)).map_err(|error| {
        map_document_operation_error_v1(DocumentSessionError::Operation(error.into()))
    })?;
    Ok(SessionOperationTransitionRequestV1::new(
        gesture.fence.revision(),
        SessionOperation::V1(SessionOperationV1::CreateReactionV1(operation)),
        ferrum_document::TransitionAuthorizationV1::authoring_capability(gesture.capability),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::{
        DocumentSessionError, PresentationRecordKindV1, PresentationRootDeletionV1,
        SessionOperation, SessionOperationError, SessionOperationOutcomeV1, SessionOperationV1,
        TypedDocumentError,
    };
    const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"product\"><atom id=\"product-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"arrow\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow></cdml>";
    fn request() -> ReactionCreateRequestV1 {
        ReactionCreateRequestV1::new(
            0,
            vec!["left".into()],
            vec!["product".into()],
            "arrow".into(),
            vec![],
            vec![],
        )
        .expect("fixture request")
    }
    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }
    #[test]
    fn bridge_commits_once() {
        let mut session = DocumentSession::load(SOURCE).expect("load");
        let gesture =
            begin_reaction_gesture_v1(&session, fence(&session), request()).expect("begin");
        let request = resolve_reaction_gesture_v1(&session, gesture).expect("resolve");
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("prepare");
        let committed = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("commit");
        assert!(matches!(
            committed.outcome(),
            SessionOperationOutcomeV1::ReactionCreatedV1(outcome) if outcome.reaction_id() == "rxn-1"
        ));
        assert!(
            committed
                .observation()
                .snapshot()
                .cdml()
                .contains("<reaction id=\"rxn-1\"")
        );
        assert!(matches!(
            session.commit_session_operation_transition_v1(&mut prepared),
            Err(ferrum_document::AdmittedSessionTransitionRefusalV1::Replayed)
        ));
    }
    #[test]
    fn bridge_authored_reaction_refuses_referenced_arrow_deletion_atomically() {
        let mut session = DocumentSession::load(SOURCE).expect("load");
        let gesture =
            begin_reaction_gesture_v1(&session, fence(&session), request()).expect("begin");
        let request = resolve_reaction_gesture_v1(&session, gesture).expect("resolve");
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("prepare");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("commit");
        let before = session.snapshot().expect("snapshot");
        let deletion = PresentationRootDeletionV1::new("arrow", PresentationRecordKindV1::Arrow)
            .expect("selector");
        assert!(matches!(
            session.apply_document_operation_v1(
                1,
                SessionOperation::V1(SessionOperationV1::DeletePresentationRoot { deletion }),
            ),
            Err(DocumentSessionError::Operation(
                SessionOperationError::Candidate(
                    TypedDocumentError::ReactionReferencedPresentationDeletion(_)
                )
            ))
        ));
        assert_eq!(session.snapshot().expect("snapshot"), before);
    }
    #[test]
    fn unrenderable_candidate_does_not_mutate() {
        let source = format!(
            "{}<polygon id=\"legacy\"/></cdml>",
            &SOURCE[..SOURCE.len() - 7]
        );
        let mut session = DocumentSession::load(&source).expect("load");
        let before = session.snapshot().expect("before");
        let gesture =
            begin_reaction_gesture_v1(&session, fence(&session), request()).expect("begin");
        assert!(matches!(
            session
                .prepare_session_operation_transition_v1(
                    resolve_reaction_gesture_v1(&session, gesture).expect("resolve"),
                )
                .map_err(map_document_operation_error_v1),
            Err(ReactionGestureErrorV1::UnrenderableDocument
                | ReactionGestureErrorV1::RenderPreparation)
        ));
        assert_eq!(session.snapshot().expect("after"), before);
    }
    #[test]
    fn bridge_accepts_prefixed_cdml_and_appends_one_core_reaction() {
        let source = concat!(
            "<c:cdml xmlns:c=\"urn:ferrum:cdml\">",
            "<c:molecule id=\"left\"><c:atom id=\"left-a\" name=\"C\"><c:point x=\"0\" y=\"0\"/></c:atom></c:molecule>",
            "<c:molecule id=\"product\"><c:atom id=\"product-a\" name=\"O\"><c:point x=\"100\" y=\"0\"/></c:atom></c:molecule>",
            "<c:arrow id=\"arrow\"><c:point x=\"25\" y=\"0\"/><c:point x=\"75\" y=\"0\"/></c:arrow></c:cdml>"
        );
        let mut session = DocumentSession::load(source).expect("load");
        let gesture =
            begin_reaction_gesture_v1(&session, fence(&session), request()).expect("begin");
        let request = resolve_reaction_gesture_v1(&session, gesture).expect("resolve");
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("prepare");
        let committed = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("commit");
        assert!(matches!(
            committed.outcome(),
            SessionOperationOutcomeV1::ReactionCreatedV1(outcome) if outcome.reaction_id() == "rxn-1"
        ));
        assert!(
            committed
                .observation()
                .snapshot()
                .cdml()
                .contains("reaction")
        );
    }
    #[test]
    fn foreign_and_nested_reaction_lookalikes_never_enter_authoring_semantics() {
        let source = concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\"><v:molecule id=\"left\"/>",
            "<molecule id=\"product\"><atom id=\"product-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom><v:reaction id=\"nested\"><v:arrow idref=\"arrow\"/></v:reaction></molecule>",
            "<arrow id=\"arrow\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow>",
            "<v:reaction id=\"foreign\"><v:reactant idref=\"product\"/></v:reaction></cdml>"
        );
        let mut session = DocumentSession::load(source).expect("load");
        let before = session.snapshot().expect("before");
        let gesture =
            begin_reaction_gesture_v1(&session, fence(&session), request()).expect("begin");
        assert!(matches!(
            session
                .prepare_session_operation_transition_v1(
                    resolve_reaction_gesture_v1(&session, gesture).expect("resolve"),
                )
                .map_err(map_document_operation_error_v1),
            Err(ReactionGestureErrorV1::MissingTarget | ReactionGestureErrorV1::WrongTargetKind)
        ));
        assert_eq!(session.snapshot().expect("after"), before);
    }
}

//! Renderer-admitted reaction transaction bridge.
//!
//! Reaction authoring is intentionally owned here, above the generic document
//! session. The document crate accepts compatibility CDML, protects recognized
//! reaction references during deletion, and commits complete CDML only. It has
//! no reaction candidate, request, or reaction-specific commit capability.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use ferrum_document::{
    DirectCdmlRootKindV1, DirectCdmlSemanticIndexV1, DirectReactionRoleV1, DocumentFenceV1,
    DocumentSession, SessionOperationResultV1, append_direct_cdml_reaction_v1,
};
use ferrum_render::{
    DocumentRenderOutcomeV1, DocumentRenderPlanV1, compose_document_render_plan_v1,
    document_observation_from_accepted_operation_v1,
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ReactionSessionOriginV1(pub(crate) u64);
/// Opaque session-bound authoring capability. It deliberately has no public
/// fields, candidate conversion, serialization, clone, or dereference route.
#[derive(Debug)]
pub struct ReactionGestureV1 {
    origin: ReactionSessionOriginV1,
    nonce: u64,
    fence: DocumentFenceV1,
    request: ReactionCreateRequestV1,
}
/// Opaque, one-use renderer receipt. Candidate CDML, render-contract proof,
/// and complete plan remain bridge-private.
pub struct PreparedReactionV1 {
    receipt: Option<ReactionRendererReceiptV1>,
    reaction_id: String,
}
impl std::fmt::Debug for PreparedReactionV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PreparedReactionV1")
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
pub struct CommittedReactionV1 {
    reaction_id: String,
    result: SessionOperationResultV1,
}
impl CommittedReactionV1 {
    #[must_use]
    pub fn reaction_id(&self) -> &str {
        &self.reaction_id
    }
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
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

struct ReactionRendererReceiptV1 {
    origin: ReactionSessionOriginV1,
    nonce: u64,
    source_fence: DocumentFenceV1,
    candidate_revision: u64,
    candidate_digest: [u8; 32],
    reaction_id: String,
    candidate: String,
    contract: ferrum_render_contract::PreflightedDocumentRenderV1,
    plan: DocumentRenderPlanV1,
}
impl std::fmt::Debug for ReactionRendererReceiptV1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReactionRendererReceiptV1")
            .field("reaction_id", &self.reaction_id)
            .field("candidate_revision", &self.candidate_revision)
            .finish()
    }
}

fn compile_candidate(
    source: &str,
    request: &ReactionCreateRequestV1,
) -> Result<(String, String), ReactionGestureErrorV1> {
    request.validate_syntax()?;
    let index = DirectCdmlSemanticIndexV1::parse(source)
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    let target = |id: &str, expected: DirectCdmlRootKindV1| -> Result<(), ReactionGestureErrorV1> {
        let root = index
            .roots()
            .iter()
            .find(|root| root.identifier() == Some(id))
            .ok_or(ReactionGestureErrorV1::MissingTarget)?;
        (root.kind() == expected)
            .then_some(())
            .ok_or(ReactionGestureErrorV1::WrongTargetKind)
    };
    for id in &request.reactants {
        target(id, DirectCdmlRootKindV1::Molecule)?;
    }
    for id in &request.products {
        target(id, DirectCdmlRootKindV1::Molecule)?;
    }
    target(&request.arrow, DirectCdmlRootKindV1::Arrow)?;
    for id in &request.conditions {
        target(id, DirectCdmlRootKindV1::Text)?;
    }
    for id in &request.pluses {
        target(id, DirectCdmlRootKindV1::Plus)?;
    }
    let all = request
        .reactants
        .iter()
        .chain(&request.products)
        .chain(std::iter::once(&request.arrow))
        .chain(&request.conditions)
        .chain(&request.pluses);
    if all.clone().any(|id| {
        index.roots().iter().any(|root| {
            root.kind() == DirectCdmlRootKindV1::Reaction
                && root.reaction_members().iter().any(|member| member == id)
        })
    }) {
        return Err(ReactionGestureErrorV1::CrossReactionReuse);
    }
    let reaction_id = (1_u64..)
        .map(|number| format!("rxn-{number}"))
        .find(|id| !index.reserves_identifier(id))
        .ok_or(ReactionGestureErrorV1::SessionConflict)?;
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
    let mut members = Vec::new();
    for (role, values) in roles {
        for value in values {
            members.push((role, value.to_owned()));
        }
    }
    let candidate = append_direct_cdml_reaction_v1(source, &reaction_id, &members)
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    Ok((candidate, reaction_id))
}
fn origin(session: &DocumentSession) -> ReactionSessionOriginV1 {
    ReactionSessionOriginV1(session.bridge_session_origin_v1())
}
fn consumed() -> &'static Mutex<HashSet<(ReactionSessionOriginV1, u64)>> {
    static VALUE: OnceLock<Mutex<HashSet<(ReactionSessionOriginV1, u64)>>> = OnceLock::new();
    VALUE.get_or_init(|| Mutex::new(HashSet::new()))
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
    static NEXT: AtomicU64 = AtomicU64::new(1);
    Ok(ReactionGestureV1 {
        origin: origin(session),
        nonce: NEXT.fetch_add(1, Ordering::Relaxed),
        fence,
        request,
    })
}
pub fn prepare_reaction_gesture_v1(
    session: &mut DocumentSession,
    gesture: &ReactionGestureV1,
) -> Result<PreparedReactionV1, ReactionGestureErrorV1> {
    if gesture.origin != origin(session) {
        return Err(ReactionGestureErrorV1::ForeignSession);
    }
    if consumed()
        .lock()
        .expect("reaction receipt lock is not poisoned")
        .contains(&(gesture.origin, gesture.nonce))
    {
        return Err(ReactionGestureErrorV1::ReplayedGesture);
    }
    require_fence(session, gesture.fence)?;
    let source = session
        .snapshot()
        .map_err(|_| ReactionGestureErrorV1::SessionConflict)?
        .cdml()
        .to_owned();
    let (candidate, reaction_id) = compile_candidate(&source, &gesture.request)?;
    let contract = ferrum_render_contract::preflight_complete_document_v1(&candidate)
        .map_err(|_| ReactionGestureErrorV1::UnrenderableDocument)?;
    let candidate_session =
        DocumentSession::load(&candidate).map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    let observation = candidate_session
        .observe(0)
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    let render_observation = document_observation_from_accepted_operation_v1(&observation)
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    let plan = compose_document_render_plan_v1(&render_observation)
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    if plan
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(ReactionGestureErrorV1::RenderPreparation);
    }
    let snapshot = candidate_session
        .snapshot()
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    Ok(PreparedReactionV1 {
        reaction_id: reaction_id.clone(),
        receipt: Some(ReactionRendererReceiptV1 {
            origin: gesture.origin,
            nonce: gesture.nonce,
            source_fence: gesture.fence,
            candidate_revision: gesture
                .fence
                .revision()
                .checked_add(1)
                .ok_or(ReactionGestureErrorV1::SessionConflict)?,
            candidate_digest: *snapshot.digest(),
            reaction_id,
            candidate,
            contract,
            plan,
        }),
    })
}
pub fn commit_reaction_gesture_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedReactionV1,
) -> Result<CommittedReactionV1, ReactionGestureErrorV1> {
    let receipt = prepared
        .receipt
        .as_ref()
        .ok_or(ReactionGestureErrorV1::ReplayedGesture)?;
    if receipt.origin != origin(session) {
        return Err(ReactionGestureErrorV1::ForeignSession);
    }
    if consumed()
        .lock()
        .expect("reaction receipt lock is not poisoned")
        .contains(&(receipt.origin, receipt.nonce))
    {
        return Err(ReactionGestureErrorV1::ReplayedGesture);
    }
    require_fence(session, receipt.source_fence)?;
    if receipt.reaction_id != prepared.reaction_id
        || receipt.candidate_revision != receipt.source_fence.revision().saturating_add(1)
        || receipt.contract.source() != receipt.candidate
        || receipt
            .plan
            .outcomes()
            .iter()
            .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(ReactionGestureErrorV1::RenderPreparation);
    }
    let candidate_session = DocumentSession::load(&receipt.candidate)
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    let snapshot = candidate_session
        .snapshot()
        .map_err(|_| ReactionGestureErrorV1::RenderPreparation)?;
    if *snapshot.digest() != receipt.candidate_digest {
        return Err(ReactionGestureErrorV1::RenderPreparation);
    }
    let result = session
        .commit_complete_cdml_transaction_v1(receipt.source_fence, &receipt.candidate)
        .map_err(|_| ReactionGestureErrorV1::SessionConflict)?;
    let consumed_receipt = prepared
        .receipt
        .take()
        .expect("successful commit retains receipt");
    consumed()
        .lock()
        .expect("reaction receipt lock is not poisoned")
        .insert((consumed_receipt.origin, consumed_receipt.nonce));
    Ok(CommittedReactionV1 {
        reaction_id: prepared.reaction_id.clone(),
        result,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::{
        DocumentSessionError, PresentationRecordKindV1, PresentationRootDeletionV1,
        SessionOperation, SessionOperationError, SessionOperationV1, TypedDocumentError,
    };
    const SOURCE: &str = "<cdml><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"product\"><atom id=\"product-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"arrow\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow></cdml>";
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
        let mut prepared = prepare_reaction_gesture_v1(&mut session, &gesture).expect("prepare");
        let committed = commit_reaction_gesture_v1(&mut session, &mut prepared).expect("commit");
        assert_eq!(committed.reaction_id(), "rxn-1");
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
        assert!(matches!(
            commit_reaction_gesture_v1(&mut session, &mut prepared),
            Err(ReactionGestureErrorV1::ReplayedGesture)
        ));
    }
    #[test]
    fn bridge_authored_reaction_refuses_referenced_arrow_deletion_atomically() {
        let mut session = DocumentSession::load(SOURCE).expect("load");
        let gesture =
            begin_reaction_gesture_v1(&session, fence(&session), request()).expect("begin");
        let mut prepared = prepare_reaction_gesture_v1(&mut session, &gesture).expect("prepare");
        commit_reaction_gesture_v1(&mut session, &mut prepared).expect("commit");
        let before = session.snapshot().expect("snapshot");
        let deletion = PresentationRootDeletionV1::new("arrow", PresentationRecordKindV1::Arrow)
            .expect("selector");
        assert!(matches!(
            session.submit(
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
            prepare_reaction_gesture_v1(&mut session, &gesture),
            Err(ReactionGestureErrorV1::UnrenderableDocument
                | ReactionGestureErrorV1::RenderPreparation)
        ));
        assert_eq!(session.snapshot().expect("after"), before);
    }
    #[test]
    fn bridge_accepts_prefixed_cdml_and_appends_one_core_reaction() {
        let source = concat!(
            "<c:cdml xmlns:c=\"http://www.freesoftware.fsf.org/bkchem/cdml\">",
            "<c:molecule id=\"left\"><c:atom id=\"left-a\" name=\"C\"><c:point x=\"0\" y=\"0\"/></c:atom></c:molecule>",
            "<c:molecule id=\"product\"><c:atom id=\"product-a\" name=\"O\"><c:point x=\"100\" y=\"0\"/></c:atom></c:molecule>",
            "<c:arrow id=\"arrow\"><c:point x=\"25\" y=\"0\"/><c:point x=\"75\" y=\"0\"/></c:arrow></c:cdml>"
        );
        let mut session = DocumentSession::load(source).expect("load");
        let gesture =
            begin_reaction_gesture_v1(&session, fence(&session), request()).expect("begin");
        let mut prepared = prepare_reaction_gesture_v1(&mut session, &gesture).expect("prepare");
        let committed = commit_reaction_gesture_v1(&mut session, &mut prepared).expect("commit");
        assert_eq!(committed.reaction_id(), "rxn-1");
        assert!(
            committed
                .result()
                .observation()
                .snapshot()
                .cdml()
                .contains("reaction")
        );
    }
    #[test]
    fn foreign_and_nested_reaction_lookalikes_never_enter_authoring_semantics() {
        let source = concat!(
            "<cdml xmlns:v=\"urn:vendor\"><v:molecule id=\"left\"/>",
            "<molecule id=\"product\"><atom id=\"product-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom><v:reaction id=\"nested\"><v:arrow idref=\"arrow\"/></v:reaction></molecule>",
            "<arrow id=\"arrow\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow>",
            "<v:reaction id=\"foreign\"><v:reactant idref=\"product\"/></v:reaction></cdml>"
        );
        let mut session = DocumentSession::load(source).expect("load");
        let before = session.snapshot().expect("before");
        let gesture =
            begin_reaction_gesture_v1(&session, fence(&session), request()).expect("begin");
        assert!(matches!(
            prepare_reaction_gesture_v1(&mut session, &gesture),
            Err(ReactionGestureErrorV1::MissingTarget | ReactionGestureErrorV1::WrongTargetKind)
        ));
        assert_eq!(session.snapshot().expect("after"), before);
    }
}

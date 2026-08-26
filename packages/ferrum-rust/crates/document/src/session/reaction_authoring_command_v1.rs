//! Durable, document-owned commands for creating and changing direct reactions.
//!
//! Public callers name only durable document object IDs.  This module validates
//! those IDs at the live-session boundary and keeps the corresponding direct
//! CDML identifiers inside the crate-only lowering payloads.

use std::fmt;

use thiserror::Error;

use crate::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityV1, CreateReactionV1, DeleteReactionV1,
    DirectReactionRoleV1, DocumentFenceV1, DocumentObjectIdV1, ReactionOperationRefusalV1,
    ReplaceReactionMembersV1, SessionOperation, SessionOperationTransitionRequestV1,
    SessionOperationV1, TransitionAuthorizationV1, TypedClass,
    session_operation::{validate_reaction_members, validate_reaction_members_against_document},
};

use super::{DocumentReactionMemberSelectionV1, DocumentSession, ReactionMemberSelectionRefusalV1};

/// Complete role-separated durable targets for one direct reaction definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentReactionMemberTargetsV1 {
    reactants: Vec<DocumentObjectIdV1>,
    arrow: DocumentObjectIdV1,
    products: Vec<DocumentObjectIdV1>,
    conditions: Vec<DocumentObjectIdV1>,
    pluses: Vec<DocumentObjectIdV1>,
}

impl DocumentReactionMemberTargetsV1 {
    /// Validate a complete durable direct-reaction membership set.
    pub fn new(
        reactants: Vec<DocumentObjectIdV1>,
        arrow: DocumentObjectIdV1,
        products: Vec<DocumentObjectIdV1>,
        conditions: Vec<DocumentObjectIdV1>,
        pluses: Vec<DocumentObjectIdV1>,
    ) -> Result<Self, ReactionOperationRefusalV1> {
        let targets = Self {
            reactants,
            arrow,
            products,
            conditions,
            pluses,
        };
        validate_reaction_members(&targets.durable_members())?;
        Ok(targets)
    }

    /// Return the nonempty durable reactant targets in authored role order.
    #[must_use]
    pub fn reactants(&self) -> &[DocumentObjectIdV1] {
        &self.reactants
    }

    /// Return the single durable arrow target.
    #[must_use]
    pub fn arrow(&self) -> &DocumentObjectIdV1 {
        &self.arrow
    }

    /// Return the nonempty durable product targets in authored role order.
    #[must_use]
    pub fn products(&self) -> &[DocumentObjectIdV1] {
        &self.products
    }

    /// Return optional durable condition-text targets in authored role order.
    #[must_use]
    pub fn conditions(&self) -> &[DocumentObjectIdV1] {
        &self.conditions
    }

    /// Return optional durable Plus targets in authored role order.
    #[must_use]
    pub fn pluses(&self) -> &[DocumentObjectIdV1] {
        &self.pluses
    }

    fn durable_members(&self) -> Vec<(DirectReactionRoleV1, String)> {
        let mut members = Vec::with_capacity(
            self.reactants.len()
                + self.products.len()
                + self.conditions.len()
                + self.pluses.len()
                + 1,
        );
        members.extend(
            self.reactants
                .iter()
                .map(|id| (DirectReactionRoleV1::Reactant, id.as_str().to_owned())),
        );
        members.push((DirectReactionRoleV1::Arrow, self.arrow.as_str().to_owned()));
        members.extend(
            self.products
                .iter()
                .map(|id| (DirectReactionRoleV1::Product, id.as_str().to_owned())),
        );
        members.extend(
            self.conditions
                .iter()
                .map(|id| (DirectReactionRoleV1::Condition, id.as_str().to_owned())),
        );
        members.extend(
            self.pluses
                .iter()
                .map(|id| (DirectReactionRoleV1::Plus, id.as_str().to_owned())),
        );
        members
    }

    fn role_targets(&self) -> impl Iterator<Item = (DirectReactionRoleV1, &DocumentObjectIdV1)> {
        self.reactants
            .iter()
            .map(|id| (DirectReactionRoleV1::Reactant, id))
            .chain(std::iter::once((DirectReactionRoleV1::Arrow, &self.arrow)))
            .chain(
                self.products
                    .iter()
                    .map(|id| (DirectReactionRoleV1::Product, id)),
            )
            .chain(
                self.conditions
                    .iter()
                    .map(|id| (DirectReactionRoleV1::Condition, id)),
            )
            .chain(
                self.pluses
                    .iter()
                    .map(|id| (DirectReactionRoleV1::Plus, id)),
            )
    }
}

/// Stable category for one opaque reaction-authoring command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentReactionAuthoringCommandKindV1 {
    /// Add one complete strict direct reaction.
    Create,
    /// Replace every member of one selected strict direct reaction.
    ReplaceMembers,
    /// Remove one selected strict direct reaction definition.
    Delete,
}

/// Closed refusal surface for durable reaction authoring commands.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ReactionAuthoringCommandRefusalV1 {
    /// The complete role-separated target set violates the direct reaction contract.
    #[error(transparent)]
    InvalidMembers(#[from] ReactionOperationRefusalV1),
    /// The selected reaction is no longer a valid complete session capability.
    #[error(transparent)]
    InvalidSelection(#[from] ReactionMemberSelectionRefusalV1),
    /// The authoring receipt belongs to a different live document session.
    #[error("reaction authoring command belongs to a different document session")]
    ForeignSession,
    /// The authoring receipt was claimed or terminally consumed.
    #[error("reaction authoring command was already redeemed")]
    Consumed,
    /// The session revision no longer matches the command fence.
    #[error("reaction authoring command was prepared at a stale document revision")]
    StaleRevision,
    /// The retained document content no longer matches the command fence.
    #[error("reaction authoring command was prepared from different document content")]
    StaleDigest,
}

/// Opaque one-use command for one durable direct-reaction creation.
pub struct DocumentCreateReactionCommandV1 {
    command: ReactionAuthoringCommandCoreV1,
}

/// Opaque one-use command for replacing durable direct-reaction members.
pub struct DocumentReplaceReactionMembersCommandV1 {
    command: ReactionAuthoringCommandCoreV1,
}

/// Opaque one-use command for deleting one durable direct-reaction definition.
pub struct DocumentDeleteReactionCommandV1 {
    command: ReactionAuthoringCommandCoreV1,
}

impl DocumentCreateReactionCommandV1 {
    /// Return the stable category without exposing source IDs or CDML references.
    #[must_use]
    pub const fn kind(&self) -> DocumentReactionAuthoringCommandKindV1 {
        DocumentReactionAuthoringCommandKindV1::Create
    }
}

impl DocumentReplaceReactionMembersCommandV1 {
    /// Return the stable category without exposing source IDs or CDML references.
    #[must_use]
    pub const fn kind(&self) -> DocumentReactionAuthoringCommandKindV1 {
        DocumentReactionAuthoringCommandKindV1::ReplaceMembers
    }
}

impl DocumentDeleteReactionCommandV1 {
    /// Return the stable category without exposing source IDs or CDML references.
    #[must_use]
    pub const fn kind(&self) -> DocumentReactionAuthoringCommandKindV1 {
        DocumentReactionAuthoringCommandKindV1::Delete
    }
}

impl fmt::Debug for DocumentCreateReactionCommandV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.command
            .fmt("DocumentCreateReactionCommandV1", formatter)
    }
}

impl fmt::Debug for DocumentReplaceReactionMembersCommandV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.command
            .fmt("DocumentReplaceReactionMembersCommandV1", formatter)
    }
}

impl fmt::Debug for DocumentDeleteReactionCommandV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.command
            .fmt("DocumentDeleteReactionCommandV1", formatter)
    }
}

enum ReactionAuthoringCommandPayloadV1 {
    Create(CreateReactionV1),
    Replace {
        request: ReplaceReactionMembersV1,
        selection: DocumentReactionMemberSelectionV1,
    },
    Delete {
        request: DeleteReactionV1,
        selection: DocumentReactionMemberSelectionV1,
    },
}

struct ReactionAuthoringCommandCoreV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    payload: ReactionAuthoringCommandPayloadV1,
}

impl ReactionAuthoringCommandCoreV1 {
    fn fmt(&self, type_name: &str, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let category = match self.payload {
            ReactionAuthoringCommandPayloadV1::Create(_) => {
                DocumentReactionAuthoringCommandKindV1::Create
            }
            ReactionAuthoringCommandPayloadV1::Replace { .. } => {
                DocumentReactionAuthoringCommandKindV1::ReplaceMembers
            }
            ReactionAuthoringCommandPayloadV1::Delete { .. } => {
                DocumentReactionAuthoringCommandKindV1::Delete
            }
        };
        formatter
            .debug_struct(type_name)
            .field("category", &category)
            .field("revision", &self.fence.revision())
            .finish()
    }
}

impl DocumentSession {
    /// Begin one durable reaction creation without changing the retained document.
    pub fn begin_create_reaction_v1(
        &self,
        capability: AuthoringCapabilityV1,
        fence: DocumentFenceV1,
        targets: DocumentReactionMemberTargetsV1,
    ) -> Result<DocumentCreateReactionCommandV1, ReactionAuthoringCommandRefusalV1> {
        self.validate_reaction_authoring_capability_v1(&capability)?;
        self.require_reaction_authoring_fence_v1(fence)?;
        let members = self.lower_reaction_targets_v1(&targets, None)?;
        let request = CreateReactionV1::new(members)?;
        Ok(DocumentCreateReactionCommandV1 {
            command: ReactionAuthoringCommandCoreV1 {
                capability,
                fence,
                payload: ReactionAuthoringCommandPayloadV1::Create(request),
            },
        })
    }

    /// Begin one complete durable membership replacement without changing the document.
    pub fn begin_replace_reaction_members_v1(
        &self,
        capability: AuthoringCapabilityV1,
        selection: DocumentReactionMemberSelectionV1,
        targets: DocumentReactionMemberTargetsV1,
    ) -> Result<DocumentReplaceReactionMembersCommandV1, ReactionAuthoringCommandRefusalV1> {
        self.validate_reaction_authoring_capability_v1(&capability)?;
        self.validate_reaction_selection_v1(&selection)?;
        let reaction_id = self.lower_selected_reaction_v1(&selection)?;
        let members = self.lower_reaction_targets_v1(&targets, Some(reaction_id.as_str()))?;
        let request = ReplaceReactionMembersV1::new(reaction_id, members)?;
        Ok(DocumentReplaceReactionMembersCommandV1 {
            command: ReactionAuthoringCommandCoreV1 {
                capability,
                fence: DocumentFenceV1::new(self.current_revision_v1(), self.current_digest_v1()),
                payload: ReactionAuthoringCommandPayloadV1::Replace { request, selection },
            },
        })
    }

    /// Begin one durable reaction-definition deletion without changing the document.
    pub fn begin_delete_reaction_v1(
        &self,
        capability: AuthoringCapabilityV1,
        selection: DocumentReactionMemberSelectionV1,
    ) -> Result<DocumentDeleteReactionCommandV1, ReactionAuthoringCommandRefusalV1> {
        self.validate_reaction_authoring_capability_v1(&capability)?;
        self.validate_reaction_selection_v1(&selection)?;
        let reaction_id = self.lower_selected_reaction_v1(&selection)?;
        let request = DeleteReactionV1::new(reaction_id)?;
        Ok(DocumentDeleteReactionCommandV1 {
            command: ReactionAuthoringCommandCoreV1 {
                capability,
                fence: DocumentFenceV1::new(self.current_revision_v1(), self.current_digest_v1()),
                payload: ReactionAuthoringCommandPayloadV1::Delete { request, selection },
            },
        })
    }

    /// Resolve one creation command into the existing generic renderer-admission request.
    pub fn resolve_create_reaction_command_v1(
        &self,
        command: DocumentCreateReactionCommandV1,
    ) -> Result<SessionOperationTransitionRequestV1, ReactionAuthoringCommandRefusalV1> {
        self.resolve_reaction_authoring_command_v1(command.command)
    }

    /// Resolve one replacement command into the existing generic renderer-admission request.
    pub fn resolve_replace_reaction_members_command_v1(
        &self,
        command: DocumentReplaceReactionMembersCommandV1,
    ) -> Result<SessionOperationTransitionRequestV1, ReactionAuthoringCommandRefusalV1> {
        self.resolve_reaction_authoring_command_v1(command.command)
    }

    /// Resolve one deletion command into the existing generic renderer-admission request.
    pub fn resolve_delete_reaction_command_v1(
        &self,
        command: DocumentDeleteReactionCommandV1,
    ) -> Result<SessionOperationTransitionRequestV1, ReactionAuthoringCommandRefusalV1> {
        self.resolve_reaction_authoring_command_v1(command.command)
    }

    fn resolve_reaction_authoring_command_v1(
        &self,
        command: ReactionAuthoringCommandCoreV1,
    ) -> Result<SessionOperationTransitionRequestV1, ReactionAuthoringCommandRefusalV1> {
        self.validate_reaction_authoring_capability_v1(&command.capability)?;
        self.require_reaction_authoring_fence_v1(command.fence)?;
        let operation = match command.payload {
            ReactionAuthoringCommandPayloadV1::Create(request) => {
                SessionOperation::V1(SessionOperationV1::CreateReactionV1(request))
            }
            ReactionAuthoringCommandPayloadV1::Replace { request, selection } => {
                self.validate_reaction_selection_v1(&selection)?;
                SessionOperation::V1(SessionOperationV1::ReplaceReactionMembersV1(request))
            }
            ReactionAuthoringCommandPayloadV1::Delete { request, selection } => {
                self.validate_reaction_selection_v1(&selection)?;
                SessionOperation::V1(SessionOperationV1::DeleteReactionV1(request))
            }
        };
        Ok(SessionOperationTransitionRequestV1::new(
            command.fence.revision(),
            operation,
            TransitionAuthorizationV1::authoring_capability(command.capability),
        ))
    }

    fn validate_reaction_authoring_capability_v1(
        &self,
        capability: &AuthoringCapabilityV1,
    ) -> Result<(), ReactionAuthoringCommandRefusalV1> {
        capability
            .claim_for_commit(&self.authoring_capability_issuer_v1())
            .map(drop)
            .map_err(|error| match error {
                AuthoringCapabilityAccessErrorV1::ForeignSession => {
                    ReactionAuthoringCommandRefusalV1::ForeignSession
                }
                AuthoringCapabilityAccessErrorV1::Consumed => {
                    ReactionAuthoringCommandRefusalV1::Consumed
                }
            })
    }

    fn require_reaction_authoring_fence_v1(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<(), ReactionAuthoringCommandRefusalV1> {
        if self.current_revision_v1() != fence.revision() {
            return Err(ReactionAuthoringCommandRefusalV1::StaleRevision);
        }
        if self.current_digest_v1() != fence.digest() {
            return Err(ReactionAuthoringCommandRefusalV1::StaleDigest);
        }
        Ok(())
    }

    fn lower_reaction_targets_v1(
        &self,
        targets: &DocumentReactionMemberTargetsV1,
        excluded_reaction: Option<&str>,
    ) -> Result<Vec<(DirectReactionRoleV1, String)>, ReactionAuthoringCommandRefusalV1> {
        let members = targets
            .role_targets()
            .map(|(role, object_id)| self.lower_reaction_member_v1(role, object_id))
            .collect::<Result<Vec<_>, _>>()?;
        validate_reaction_members(&members)?;
        validate_reaction_members_against_document(
            self.current_document_v1(),
            &members,
            excluded_reaction,
        )?;
        Ok(members)
    }

    fn lower_reaction_member_v1(
        &self,
        role: DirectReactionRoleV1,
        object_id: &DocumentObjectIdV1,
    ) -> Result<(DirectReactionRoleV1, String), ReactionAuthoringCommandRefusalV1> {
        let document = self.current_document_v1();
        let record = document
            .resolve_document_object_id(object_id)
            .ok_or(ReactionOperationRefusalV1::MissingMember)?;
        if record.path().components().len() != 1
            || !reaction_role_matches_class(role, record.class())
        {
            return Err(ReactionOperationRefusalV1::WrongMemberKind.into());
        }
        let source_id = document
            .source_id_for_document_object_id_v1(object_id)
            .ok_or(ReactionOperationRefusalV1::MissingMember)?;
        Ok((role, source_id.as_str().to_owned()))
    }

    fn lower_selected_reaction_v1(
        &self,
        selection: &DocumentReactionMemberSelectionV1,
    ) -> Result<String, ReactionAuthoringCommandRefusalV1> {
        let document = self.current_document_v1();
        let record = document
            .resolve_document_object_id(selection.reaction_object_id_v1())
            .ok_or(ReactionMemberSelectionRefusalV1::UnknownReaction)?;
        if record.path().components().len() != 1 || record.class() != TypedClass::Reaction {
            return Err(ReactionMemberSelectionRefusalV1::WrongReactionKind.into());
        }
        let source_id = document
            .source_id_for_document_object_id_v1(selection.reaction_object_id_v1())
            .ok_or(ReactionMemberSelectionRefusalV1::UnresolvedReaction)?;
        Ok(source_id.as_str().to_owned())
    }
}

fn reaction_role_matches_class(role: DirectReactionRoleV1, class: TypedClass) -> bool {
    match role {
        DirectReactionRoleV1::Reactant | DirectReactionRoleV1::Product => {
            class == TypedClass::Molecule
        }
        DirectReactionRoleV1::Arrow => class == TypedClass::CanvasArrow,
        DirectReactionRoleV1::Condition => class == TypedClass::CanvasText,
        DirectReactionRoleV1::Plus => class == TypedClass::CanvasPlus,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PersistentId;

    const CREATE_SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"left\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><arrow id=\"arrow\"><point x=\"0\" y=\"10\"/><point x=\"20\" y=\"10\"/></arrow><molecule id=\"right\"><atom id=\"b\" name=\"C\"><point x=\"20\" y=\"0\"/></atom></molecule></cdml>";
    const REACTION_SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:object=\"urn:ferrum:document-object:v1\"><molecule id=\"left\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><arrow id=\"arrow\"><point x=\"0\" y=\"10\"/><point x=\"20\" y=\"10\"/></arrow><molecule id=\"right\"><atom id=\"b\" name=\"C\"><point x=\"20\" y=\"0\"/></atom></molecule><molecule id=\"new-left\"><atom id=\"c\" name=\"C\"><point x=\"40\" y=\"0\"/></atom></molecule><arrow id=\"new-arrow\"><point x=\"40\" y=\"10\"/><point x=\"60\" y=\"10\"/></arrow><molecule id=\"new-right\"><atom id=\"d\" name=\"C\"><point x=\"60\" y=\"0\"/></atom></molecule><reaction id=\"reaction\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000001\"><reactant idref=\"left\"/><arrow idref=\"arrow\"/><product idref=\"right\"/></reaction></cdml>";

    fn object(session: &DocumentSession, source: &str) -> DocumentObjectIdV1 {
        session
            .current_document_v1()
            .document_object_id_for_source_id_v1(&PersistentId::new(source).expect("source ID"))
            .expect("durable object ID")
    }

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    fn targets(
        session: &DocumentSession,
        left: &str,
        arrow: &str,
        right: &str,
    ) -> DocumentReactionMemberTargetsV1 {
        DocumentReactionMemberTargetsV1::new(
            vec![object(session, left)],
            object(session, arrow),
            vec![object(session, right)],
            Vec::new(),
            Vec::new(),
        )
        .expect("target contract")
    }

    fn selection(session: &DocumentSession) -> DocumentReactionMemberSelectionV1 {
        let observation = session
            .observe_reaction_members_v1(&object(session, "reaction"))
            .expect("reaction observation");
        session
            .select_reaction_members_v1(&observation)
            .expect("reaction selection")
    }

    fn prepare_and_commit(
        session: &mut DocumentSession,
        request: SessionOperationTransitionRequestV1,
    ) {
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("generic transition");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("generic commit");
    }

    #[test]
    fn creates_replaces_and_deletes_only_through_generic_transition_requests() {
        let mut create_session = DocumentSession::load(CREATE_SOURCE).expect("create session");
        let before_create = create_session.snapshot().expect("before create");
        let create = create_session
            .begin_create_reaction_v1(
                create_session.issue_authoring_capability_v1(),
                fence(&create_session),
                targets(&create_session, "left", "arrow", "right"),
            )
            .expect("create command");
        assert_eq!(
            create_session.snapshot().expect("begin is inert"),
            before_create
        );
        let create_request = create_session
            .resolve_create_reaction_command_v1(create)
            .expect("create request");
        prepare_and_commit(&mut create_session, create_request);
        assert!(
            create_session
                .snapshot()
                .expect("created snapshot")
                .cdml()
                .contains("<reaction")
        );

        let mut replace_session = DocumentSession::load(REACTION_SOURCE).expect("replace session");
        let replace = replace_session
            .begin_replace_reaction_members_v1(
                replace_session.issue_authoring_capability_v1(),
                selection(&replace_session),
                targets(&replace_session, "new-left", "new-arrow", "new-right"),
            )
            .expect("replace command");
        let replace_request = replace_session
            .resolve_replace_reaction_members_command_v1(replace)
            .expect("replace request");
        prepare_and_commit(&mut replace_session, replace_request);

        let delete_selection = selection(&replace_session);
        let delete = replace_session
            .begin_delete_reaction_v1(
                replace_session.issue_authoring_capability_v1(),
                delete_selection,
            )
            .expect("delete command");
        let delete_request = replace_session
            .resolve_delete_reaction_command_v1(delete)
            .expect("delete request");
        prepare_and_commit(&mut replace_session, delete_request);
        assert!(
            replace_session
                .observe_reaction_list_v1()
                .expect("list")
                .reactions()
                .is_empty()
        );
    }

    #[test]
    fn refuses_role_mismatch_duplicates_cross_reaction_stale_and_foreign_without_mutation() {
        let session = DocumentSession::load(REACTION_SOURCE).expect("session");
        let before = session.snapshot().expect("before");
        let wrong_role = DocumentReactionMemberTargetsV1::new(
            vec![object(&session, "new-arrow")],
            object(&session, "arrow"),
            vec![object(&session, "new-right")],
            Vec::new(),
            Vec::new(),
        )
        .expect("shape permits durable IDs");
        assert!(matches!(
            session.begin_create_reaction_v1(
                session.issue_authoring_capability_v1(),
                fence(&session),
                wrong_role,
            ),
            Err(ReactionAuthoringCommandRefusalV1::InvalidMembers(
                ReactionOperationRefusalV1::WrongMemberKind
            ))
        ));
        assert!(matches!(
            DocumentReactionMemberTargetsV1::new(
                vec![object(&session, "new-left"), object(&session, "new-left")],
                object(&session, "new-arrow"),
                vec![object(&session, "new-right")],
                Vec::new(),
                Vec::new(),
            ),
            Err(ReactionOperationRefusalV1::DuplicateMember)
        ));
        assert!(matches!(
            session.begin_create_reaction_v1(
                session.issue_authoring_capability_v1(),
                fence(&session),
                targets(&session, "left", "new-arrow", "new-right"),
            ),
            Err(ReactionAuthoringCommandRefusalV1::InvalidMembers(
                ReactionOperationRefusalV1::CrossReactionReuse
            ))
        ));
        assert!(matches!(
            session.begin_create_reaction_v1(
                session.issue_authoring_capability_v1(),
                DocumentFenceV1::new(fence(&session).revision() + 1, fence(&session).digest()),
                targets(&session, "new-left", "new-arrow", "new-right"),
            ),
            Err(ReactionAuthoringCommandRefusalV1::StaleRevision)
        ));
        let foreign = DocumentSession::load(REACTION_SOURCE).expect("foreign");
        assert!(matches!(
            foreign.begin_delete_reaction_v1(
                foreign.issue_authoring_capability_v1(),
                selection(&session)
            ),
            Err(ReactionAuthoringCommandRefusalV1::InvalidSelection(
                ReactionMemberSelectionRefusalV1::ForeignSession
            ))
        ));
        assert_eq!(session.snapshot().expect("refused begin is inert"), before);
    }

    #[test]
    fn consumed_authoring_receipts_refuse_command_replay_before_mutation() {
        let mut session = DocumentSession::load(CREATE_SOURCE).expect("session");
        let capability = session.issue_authoring_capability_v1();
        let first = session
            .begin_create_reaction_v1(
                capability.clone(),
                fence(&session),
                targets(&session, "left", "arrow", "right"),
            )
            .expect("first command");
        let second = session
            .begin_create_reaction_v1(
                capability,
                fence(&session),
                targets(&session, "left", "arrow", "right"),
            )
            .expect("second command before redemption");
        let request = session
            .resolve_create_reaction_command_v1(first)
            .expect("first request");
        prepare_and_commit(&mut session, request);
        assert!(matches!(
            session.resolve_create_reaction_command_v1(second),
            Err(ReactionAuthoringCommandRefusalV1::Consumed)
        ));
        assert!(
            session
                .snapshot()
                .expect("committed snapshot")
                .cdml()
                .contains("<reaction")
        );
    }
}

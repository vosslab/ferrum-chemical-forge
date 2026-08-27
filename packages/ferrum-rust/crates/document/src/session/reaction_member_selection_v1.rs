//! Durable, session-affine reaction-member selection.
//!
//! Direct CDML source identifiers remain wholly inside this module. Callers see
//! only document-owned object IDs and cannot serialize or forge a selection.

use thiserror::Error;

use ferrum_document_projection::{
    DocumentDirectRootKindV1, DocumentProjectionV1, PresentationRecordKindV1,
};

use crate::direct_cdml_semantic_index_v1::{
    DirectReactionDurableMemberV1, DirectReactionDurableV1,
};
use crate::{
    DirectCdmlSemanticIndexV1, DirectReactionRoleV1, DocumentObjectIdV1,
    ReactionDefinitionDiagnosticV1, TypedClass, TypedDocument,
};

use super::DocumentSession;

/// One durable reaction member observed from the retained direct reaction root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentReactionMemberObservationV1 {
    role: DirectReactionRoleV1,
    object_id: DocumentObjectIdV1,
    role_ordinal: u32,
    source_order: u32,
}

impl DocumentReactionMemberObservationV1 {
    /// Return the direct-reaction role for this member.
    #[must_use]
    pub const fn role(&self) -> DirectReactionRoleV1 {
        self.role
    }

    /// Return the member's durable document-owned ID.
    #[must_use]
    pub fn object_id(&self) -> &DocumentObjectIdV1 {
        &self.object_id
    }

    /// Return the ordinal among members of the same reaction role.
    #[must_use]
    pub const fn role_ordinal(&self) -> u32 {
        self.role_ordinal
    }
}

/// Durable observation of one direct reaction and its ordered member facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentReactionSelectionObservationV1 {
    reaction_object_id: DocumentObjectIdV1,
    members: Vec<DocumentReactionMemberObservationV1>,
}

impl DocumentReactionSelectionObservationV1 {
    /// Return the reaction root's durable document-owned ID.
    #[must_use]
    pub fn reaction_object_id(&self) -> &DocumentObjectIdV1 {
        &self.reaction_object_id
    }

    /// Return reaction members in canonical direct-document order.
    #[must_use]
    pub fn members(&self) -> &[DocumentReactionMemberObservationV1] {
        &self.members
    }
}

/// Whether one direct reaction can issue an exact selection observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentReactionListDispositionV1 {
    /// The direct reaction has complete, role-correct durable member facts.
    Strict,
    /// The direct reaction remains visible for diagnostics but cannot be selected.
    DisplayOnly,
}

/// One renderer-admitted durable reaction member for a reaction-list UI.
///
/// A list member carries only the direct-root paint position used by the
/// renderer. Source-relative semantic ordering remains private to the
/// selection observation that fences authoring commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentReactionListMemberV1 {
    role: DirectReactionRoleV1,
    object_id: DocumentObjectIdV1,
    role_ordinal: u32,
    document_paint_order: u32,
}

impl DocumentReactionListMemberV1 {
    /// Return the direct-reaction role for this renderer-admitted member.
    #[must_use]
    pub const fn role(&self) -> DirectReactionRoleV1 {
        self.role
    }

    /// Return the member's durable document-owned ID.
    #[must_use]
    pub fn object_id(&self) -> &DocumentObjectIdV1 {
        &self.object_id
    }

    /// Return the ordinal among members of the same reaction role.
    #[must_use]
    pub const fn role_ordinal(&self) -> u32 {
        self.role_ordinal
    }

    /// Return the member's exact document-wide direct-root paint position.
    #[must_use]
    pub const fn document_paint_order(&self) -> u32 {
        self.document_paint_order
    }
}

/// One document-owned direct reaction fact for a reaction-list UI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentReactionListReactionV1 {
    reaction_object_id: DocumentObjectIdV1,
    disposition: DocumentReactionListDispositionV1,
    diagnostics: Vec<ReactionDefinitionDiagnosticV1>,
    members: Vec<DocumentReactionListMemberV1>,
    selection_observation: Option<DocumentReactionSelectionObservationV1>,
}

impl DocumentReactionListReactionV1 {
    #[must_use]
    pub fn reaction_object_id(&self) -> &DocumentObjectIdV1 {
        &self.reaction_object_id
    }

    #[must_use]
    pub const fn disposition(&self) -> DocumentReactionListDispositionV1 {
        self.disposition
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[ReactionDefinitionDiagnosticV1] {
        &self.diagnostics
    }

    #[must_use]
    pub fn members(&self) -> &[DocumentReactionListMemberV1] {
        &self.members
    }

    #[must_use]
    pub fn selection_observation(&self) -> Option<&DocumentReactionSelectionObservationV1> {
        self.selection_observation.as_ref()
    }
}

/// Ordered document-owned facts for every directly persisted reaction root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentReactionListObservationV1 {
    reactions: Vec<DocumentReactionListReactionV1>,
}

impl DocumentReactionListObservationV1 {
    #[must_use]
    pub fn reactions(&self) -> &[DocumentReactionListReactionV1] {
        &self.reactions
    }
}

/// Opaque selection capability valid only for its issuing live document session.
///
/// Its fields intentionally remain private: callers cannot serialize source
/// identifiers, replace a fence, or forge ordered membership facts.
#[derive(Clone, Debug)]
pub struct DocumentReactionMemberSelectionV1 {
    issuer: crate::AuthoringCapabilityIssuerV1,
    revision: u64,
    digest: [u8; 32],
    reaction_object_id: DocumentObjectIdV1,
    members: Vec<DocumentReactionMemberObservationV1>,
}

impl DocumentReactionMemberSelectionV1 {
    #[must_use]
    pub(super) fn reaction_object_id_v1(&self) -> &DocumentObjectIdV1 {
        &self.reaction_object_id
    }
}

/// Closed refusal surface for reaction-member observation and selection validation.
#[derive(Debug, Error)]
pub enum ReactionMemberSelectionRefusalV1 {
    /// A retained document identity was malformed or missing while resolving reaction facts.
    #[error("reaction selection encountered an invalid retained document identity: {0}")]
    InvalidIdentity(#[source] crate::ProjectionError),
    /// The durable reaction ID no longer resolves in the retained document.
    #[error("reaction selection names an unknown durable document object")]
    UnknownReaction,
    /// The durable reaction ID resolves, but not to one direct reaction root.
    #[error("reaction selection names a non-reaction or non-direct reaction root")]
    WrongReactionKind,
    /// A direct reaction root has no usable source identity or unique semantic definition.
    #[error("reaction selection cannot resolve one direct reaction definition")]
    UnresolvedReaction,
    /// A reaction member target is absent, ambiguous, non-direct, or role-incompatible.
    #[error("reaction selection contains an unresolved or role-incompatible member")]
    UnresolvedMember,
    /// The selection was issued by another live document session.
    #[error("reaction selection belongs to a different document session")]
    ForeignSession,
    /// The authoritative session revision differs from the selection fence.
    #[error("reaction selection was prepared at a stale document revision")]
    StaleRevision,
    /// The authoritative retained-document digest differs from the selection fence.
    #[error("reaction selection was prepared from different document content")]
    StaleDigest,
    /// The complete ordered role/member facts no longer match the retained reaction.
    #[error("reaction selection membership no longer matches the retained reaction")]
    MembershipMismatch,
}

impl DocumentSession {
    /// Observe direct reaction roots for presentation without exposing CDML source IDs.
    pub fn observe_reaction_list_v1(
        &self,
    ) -> Result<DocumentReactionListObservationV1, ReactionMemberSelectionRefusalV1> {
        let observation = self
            .observe(self.current_revision_v1())
            .map_err(|_| ReactionMemberSelectionRefusalV1::UnresolvedReaction)?;
        let document = self.current_document_v1();
        build_reaction_list_observation_v1(document, observation.projection())
    }
    /// Observe one direct reaction using durable document-owned IDs only.
    pub fn observe_reaction_members_v1(
        &self,
        reaction_object_id: &DocumentObjectIdV1,
    ) -> Result<DocumentReactionSelectionObservationV1, ReactionMemberSelectionRefusalV1> {
        observe_reaction(self.current_document_v1(), reaction_object_id)
    }

    /// Fence one observed direct reaction as an opaque session-affine capability.
    pub fn select_reaction_members_v1(
        &self,
        observation: &DocumentReactionSelectionObservationV1,
    ) -> Result<DocumentReactionMemberSelectionV1, ReactionMemberSelectionRefusalV1> {
        let current = self.observe_reaction_members_v1(observation.reaction_object_id())?;
        if current.members != observation.members {
            return Err(ReactionMemberSelectionRefusalV1::MembershipMismatch);
        }
        Ok(DocumentReactionMemberSelectionV1 {
            issuer: self.authoring_capability_issuer.clone(),
            revision: self.current_revision_v1(),
            digest: self.current_digest_v1(),
            reaction_object_id: current.reaction_object_id,
            members: current.members,
        })
    }

    /// Revalidate a complete reaction-member capability against the live document.
    pub fn validate_reaction_selection_v1(
        &self,
        selection: &DocumentReactionMemberSelectionV1,
    ) -> Result<(), ReactionMemberSelectionRefusalV1> {
        if !self
            .authoring_capability_issuer
            .same_issuer(&selection.issuer)
        {
            return Err(ReactionMemberSelectionRefusalV1::ForeignSession);
        }
        if self.current_revision_v1() != selection.revision {
            return Err(ReactionMemberSelectionRefusalV1::StaleRevision);
        }
        if self.current_digest_v1() != selection.digest {
            return Err(ReactionMemberSelectionRefusalV1::StaleDigest);
        }
        let current = self.observe_reaction_members_v1(&selection.reaction_object_id)?;
        if current.members != selection.members {
            return Err(ReactionMemberSelectionRefusalV1::MembershipMismatch);
        }
        Ok(())
    }

    /// Resolve a session-affine reaction selection into renderer-safe durable roots.
    ///
    /// The returned IDs retain the exact observed reaction-member order. The
    /// selection remains opaque: issuer, revision, digest, and complete member
    /// facts are revalidated before any durable target leaves the session.
    pub fn resolve_reaction_member_root_selectors_v1(
        &self,
        selection: &DocumentReactionMemberSelectionV1,
    ) -> Result<Vec<DocumentObjectIdV1>, ReactionMemberSelectionRefusalV1> {
        self.validate_reaction_selection_v1(selection)?;
        Ok(selection
            .members
            .iter()
            .map(|member| member.object_id.clone())
            .collect())
    }
}

fn build_reaction_list_observation_v1(
    document: &TypedDocument,
    projection: &DocumentProjectionV1,
) -> Result<DocumentReactionListObservationV1, ReactionMemberSelectionRefusalV1> {
    let index = DirectCdmlSemanticIndexV1::from_document(document);
    let durable = index
        .bind_durable_reactions_v1(document)
        .map_err(ReactionMemberSelectionRefusalV1::InvalidIdentity)?;
    let reactions = durable
        .durable_reactions_v1()
        .iter()
        .map(|reaction| list_reaction(document, projection, reaction))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();
    Ok(DocumentReactionListObservationV1 { reactions })
}

fn list_reaction(
    document: &TypedDocument,
    projection: &DocumentProjectionV1,
    reaction: &DirectReactionDurableV1,
) -> Result<Option<DocumentReactionListReactionV1>, ReactionMemberSelectionRefusalV1> {
    let selection_observation = if reaction.is_strict() {
        match observe_durable_reaction(document, reaction) {
            Ok(selection) => Some(selection),
            Err(ReactionMemberSelectionRefusalV1::InvalidIdentity(error)) => {
                return Err(ReactionMemberSelectionRefusalV1::InvalidIdentity(error));
            }
            Err(_) => None,
        }
    } else {
        None
    };
    let (members, selection_observation) = selection_observation
        .and_then(|selection| {
            reaction_list_members(projection, &selection).map(|members| (members, selection))
        })
        .map_or_else(
            || (Vec::new(), None),
            |(members, selection)| (members, Some(selection)),
        );
    let disposition = if selection_observation.is_some() {
        DocumentReactionListDispositionV1::Strict
    } else {
        DocumentReactionListDispositionV1::DisplayOnly
    };
    Ok(Some(DocumentReactionListReactionV1 {
        reaction_object_id: reaction.reaction_object_id().clone(),
        disposition,
        diagnostics: reaction.diagnostics().to_vec(),
        members,
        selection_observation,
    }))
}

fn reaction_list_members(
    projection: &DocumentProjectionV1,
    selection: &DocumentReactionSelectionObservationV1,
) -> Option<Vec<DocumentReactionListMemberV1>> {
    selection
        .members()
        .iter()
        .map(|member| reaction_list_member(projection, member))
        .collect()
}

fn reaction_list_member(
    projection: &DocumentProjectionV1,
    member: &DocumentReactionMemberObservationV1,
) -> Option<DocumentReactionListMemberV1> {
    let direct_root = projection
        .direct_roots()
        .iter()
        .find(|root| root.document_object_id() == member.object_id())?;
    if !direct_root_matches_role(member.role(), direct_root.kind()) {
        return None;
    }
    Some(DocumentReactionListMemberV1 {
        role: member.role(),
        object_id: member.object_id().clone(),
        role_ordinal: member.role_ordinal(),
        document_paint_order: direct_root.paint_order(),
    })
}

fn direct_root_matches_role(role: DirectReactionRoleV1, kind: DocumentDirectRootKindV1) -> bool {
    match role {
        DirectReactionRoleV1::Reactant | DirectReactionRoleV1::Product => {
            kind == DocumentDirectRootKindV1::Molecule
        }
        DirectReactionRoleV1::Arrow => {
            kind == DocumentDirectRootKindV1::Presentation(PresentationRecordKindV1::Arrow)
        }
        DirectReactionRoleV1::Condition => {
            kind == DocumentDirectRootKindV1::Presentation(PresentationRecordKindV1::Text)
        }
        DirectReactionRoleV1::Plus => {
            kind == DocumentDirectRootKindV1::Presentation(PresentationRecordKindV1::Plus)
        }
    }
}

fn observe_reaction(
    document: &TypedDocument,
    reaction_object_id: &DocumentObjectIdV1,
) -> Result<DocumentReactionSelectionObservationV1, ReactionMemberSelectionRefusalV1> {
    let reaction_record = document
        .resolve_document_object_id(reaction_object_id)
        .map_err(ReactionMemberSelectionRefusalV1::InvalidIdentity)?
        .ok_or(ReactionMemberSelectionRefusalV1::UnknownReaction)?;
    if reaction_record.class() != TypedClass::Reaction
        || reaction_record.path().components().len() != 1
    {
        return Err(ReactionMemberSelectionRefusalV1::WrongReactionKind);
    }
    let index = DirectCdmlSemanticIndexV1::from_document(document);
    let durable = index
        .bind_durable_reactions_v1(document)
        .map_err(ReactionMemberSelectionRefusalV1::InvalidIdentity)?;
    let reaction = durable
        .durable_reaction_v1(reaction_object_id)
        .ok_or(ReactionMemberSelectionRefusalV1::UnresolvedReaction)?;
    observe_durable_reaction(document, reaction)
}

fn observe_durable_reaction(
    document: &TypedDocument,
    reaction: &DirectReactionDurableV1,
) -> Result<DocumentReactionSelectionObservationV1, ReactionMemberSelectionRefusalV1> {
    if !reaction.is_strict() {
        if reaction.diagnostics().iter().any(|diagnostic| {
            matches!(
                diagnostic,
                ReactionDefinitionDiagnosticV1::MissingIdref
                    | ReactionDefinitionDiagnosticV1::EmptyIdref
                    | ReactionDefinitionDiagnosticV1::MissingTarget
                    | ReactionDefinitionDiagnosticV1::WrongTargetKind
            )
        }) {
            return Err(ReactionMemberSelectionRefusalV1::UnresolvedMember);
        }
        return Err(ReactionMemberSelectionRefusalV1::UnresolvedReaction);
    }
    let members = reaction
        .members()
        .iter()
        .map(|member| observe_member(document, member))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DocumentReactionSelectionObservationV1 {
        reaction_object_id: reaction.reaction_object_id().clone(),
        members,
    })
}

fn observe_member(
    document: &TypedDocument,
    member: &DirectReactionDurableMemberV1,
) -> Result<DocumentReactionMemberObservationV1, ReactionMemberSelectionRefusalV1> {
    let record = document
        .resolve_document_object_id(member.member_object_id())
        .map_err(ReactionMemberSelectionRefusalV1::InvalidIdentity)?
        .ok_or(ReactionMemberSelectionRefusalV1::UnresolvedMember)?;
    if record.path().components().len() != 1 || !role_matches_class(member.role(), record.class()) {
        return Err(ReactionMemberSelectionRefusalV1::UnresolvedMember);
    }
    Ok(DocumentReactionMemberObservationV1 {
        role: member.role(),
        object_id: member.member_object_id().clone(),
        role_ordinal: member.role_ordinal(),
        source_order: member.source_order(),
    })
}

fn role_matches_class(role: DirectReactionRoleV1, class: TypedClass) -> bool {
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
    use super::super::DocumentReactionMemberTargetsV1;
    use super::*;
    use crate::PersistentId;

    const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:object=\"urn:ferrum:document-object:v1\"><molecule id=\"left\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><arrow id=\"arrow\" type=\"normal\"><point x=\"5\" y=\"0\"/><point x=\"15\" y=\"0\"/></arrow><molecule id=\"right\"><atom id=\"b\" name=\"C\"><point x=\"20\" y=\"0\"/></atom></molecule><reaction id=\"reaction\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000001\"><reactant idref=\"left\"/><arrow idref=\"arrow\"/><product idref=\"right\"/></reaction></cdml>";

    fn object(session: &DocumentSession, source: &str) -> DocumentObjectIdV1 {
        session
            .current_document_v1()
            .document_object_id_for_source_id_v1(&PersistentId::new(source).expect("source ID"))
            .expect("durable identity projection must succeed")
            .expect("durable object ID")
    }

    #[test]
    fn selects_and_validates_ordered_durable_reaction_members() {
        let session = DocumentSession::load(SOURCE).expect("session");
        let observation = session
            .observe_reaction_members_v1(&object(&session, "reaction"))
            .expect("observation");
        assert_eq!(observation.members().len(), 3);
        assert_eq!(
            observation.members()[0].object_id(),
            &object(&session, "left")
        );
        let selection = session
            .select_reaction_members_v1(&observation)
            .expect("selection");
        assert!(session.validate_reaction_selection_v1(&selection).is_ok());
    }

    #[test]
    fn resolves_a_validated_selection_as_exact_ordered_durable_roots() {
        let session = DocumentSession::load(SOURCE).expect("session");
        let observation = session
            .observe_reaction_members_v1(&object(&session, "reaction"))
            .expect("observation");
        let selection = session
            .select_reaction_members_v1(&observation)
            .expect("selection");

        assert_eq!(
            session
                .resolve_reaction_member_root_selectors_v1(&selection)
                .expect("resolved roots"),
            vec![
                object(&session, "left"),
                object(&session, "arrow"),
                object(&session, "right"),
            ]
        );

        let mut stale = selection;
        stale.revision += 1;
        assert!(matches!(
            session.resolve_reaction_member_root_selectors_v1(&stale),
            Err(ReactionMemberSelectionRefusalV1::StaleRevision)
        ));
    }

    #[test]
    fn refuses_foreign_reopened_stale_and_wrong_kind_capabilities() {
        let session = DocumentSession::load(SOURCE).expect("session");
        let observation = session
            .observe_reaction_members_v1(&object(&session, "reaction"))
            .expect("observation");
        let selection = session
            .select_reaction_members_v1(&observation)
            .expect("selection");
        let other = DocumentSession::load(SOURCE).expect("other session");
        assert!(matches!(
            other.validate_reaction_selection_v1(&selection),
            Err(ReactionMemberSelectionRefusalV1::ForeignSession)
        ));
        let mut stale = selection.clone();
        stale.revision += 1;
        assert!(matches!(
            session.validate_reaction_selection_v1(&stale),
            Err(ReactionMemberSelectionRefusalV1::StaleRevision)
        ));
        assert!(matches!(
            session.observe_reaction_members_v1(&object(&session, "left")),
            Err(ReactionMemberSelectionRefusalV1::WrongReactionKind)
        ));
    }

    #[test]
    fn refuses_unresolved_members_without_exposing_source_lookup() {
        let source = SOURCE.replace("idref=\"right\"", "idref=\"missing\"");
        let session = DocumentSession::load(&source).expect("session");
        assert!(matches!(
            session.observe_reaction_members_v1(&object(&session, "reaction")),
            Err(ReactionMemberSelectionRefusalV1::UnresolvedMember)
        ));
    }

    #[test]
    fn list_hides_members_when_a_semantic_member_is_not_renderer_admitted() {
        let source = SOURCE.replace(
            "<arrow id=\"arrow\" type=\"normal\"><point x=\"5\" y=\"0\"/><point x=\"15\" y=\"0\"/></arrow>",
            "<arrow id=\"arrow\"/>",
        );
        let session = DocumentSession::load(&source).expect("session");
        let list = session.observe_reaction_list_v1().expect("reaction list");
        let reaction = list.reactions().first().expect("reaction");

        assert_eq!(
            reaction.disposition(),
            DocumentReactionListDispositionV1::DisplayOnly
        );
        assert!(reaction.members().is_empty());
        assert!(reaction.selection_observation().is_none());
    }

    #[test]
    fn list_observation_preserves_reaction_sequence_and_exposes_member_paint_order() {
        let source = SOURCE.replace(
            "</cdml>",
            "<molecule id=\"display-left\"><atom id=\"display-a\" name=\"C\"><point x=\"40\" y=\"0\"/></atom></molecule><arrow id=\"display-arrow\"/><molecule id=\"display-right\"><atom id=\"display-b\" name=\"C\"><point x=\"60\" y=\"0\"/></atom></molecule><reaction id=\"display\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000002\"><reactant idref=\"display-left\"/><product idref=\"display-right\"/></reaction></cdml>",
        );
        let session = DocumentSession::load(&source).expect("session");
        let list = session.observe_reaction_list_v1().expect("reaction list");
        assert_eq!(list.reactions().len(), 2);
        assert_eq!(
            list.reactions()[0].reaction_object_id(),
            &object(&session, "reaction")
        );
        assert_eq!(
            list.reactions()[1].reaction_object_id(),
            &object(&session, "display")
        );
        assert_eq!(
            list.reactions()[0].disposition(),
            DocumentReactionListDispositionV1::Strict
        );
        assert_eq!(list.reactions()[0].members().len(), 3);
        assert_eq!(
            list.reactions()[0]
                .members()
                .iter()
                .map(DocumentReactionListMemberV1::document_paint_order)
                .collect::<Vec<_>>(),
            vec![0, 1, 2]
        );
        assert!(list.reactions()[0].selection_observation().is_some());
        assert_eq!(
            list.reactions()[1].disposition(),
            DocumentReactionListDispositionV1::DisplayOnly
        );
        assert!(list.reactions()[1].members().is_empty());
        assert!(list.reactions()[1].selection_observation().is_none());
        assert!(!list.reactions()[1].diagnostics().is_empty());
    }

    #[test]
    fn durable_observation_preserves_complete_role_order_and_reloads_identity_facts() {
        let source = SOURCE.replace(
            "<reaction id=\"reaction\"",
            "<text id=\"condition\"><point x=\"10\" y=\"-5\"/></text><plus id=\"plus\"><point x=\"10\" y=\"5\"/></plus><reaction id=\"reaction\"",
        ).replace(
            "<product idref=\"right\"/>",
            "<product idref=\"right\"/><condition idref=\"condition\"/><plus idref=\"plus\"/>",
        );
        let session = DocumentSession::load(&source).expect("session");
        let observation = session
            .observe_reaction_members_v1(&object(&session, "reaction"))
            .expect("durable observation");
        assert_eq!(
            observation
                .members()
                .iter()
                .map(DocumentReactionMemberObservationV1::role)
                .collect::<Vec<_>>(),
            vec![
                DirectReactionRoleV1::Reactant,
                DirectReactionRoleV1::Arrow,
                DirectReactionRoleV1::Product,
                DirectReactionRoleV1::Condition,
                DirectReactionRoleV1::Plus,
            ]
        );
        let reloaded = DocumentSession::load(session.snapshot().expect("snapshot").cdml())
            .expect("reloaded session");
        let reobserved = reloaded
            .observe_reaction_members_v1(&object(&reloaded, "reaction"))
            .expect("reloaded observation");
        assert_eq!(
            reobserved
                .members()
                .iter()
                .map(|member| (
                    member.role(),
                    member.object_id().clone(),
                    member.role_ordinal()
                ))
                .collect::<Vec<_>>(),
            observation
                .members()
                .iter()
                .map(|member| (
                    member.role(),
                    member.object_id().clone(),
                    member.role_ordinal()
                ))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn wrong_role_member_remains_display_only_and_refuses_selection() {
        let source = SOURCE.replace("<arrow idref=\"arrow\"/>", "<arrow idref=\"left\"/>");
        let session = DocumentSession::load(&source).expect("session");
        assert!(matches!(
            session.observe_reaction_members_v1(&object(&session, "reaction")),
            Err(ReactionMemberSelectionRefusalV1::UnresolvedMember)
        ));
        let list = session.observe_reaction_list_v1().expect("reaction list");
        let reaction = list.reactions().first().expect("reaction");
        assert_eq!(
            reaction.disposition(),
            DocumentReactionListDispositionV1::DisplayOnly
        );
        assert!(!reaction.diagnostics().is_empty());
    }

    #[test]
    fn mutation_undo_redo_stales_prior_selection_and_reobserves_durable_members() {
        let source = SOURCE.replace(
            "</cdml>",
            "<molecule id=\"new-left\"><atom id=\"c\" name=\"C\"><point x=\"40\" y=\"0\"/></atom></molecule><arrow id=\"new-arrow\"><point x=\"45\" y=\"0\"/><point x=\"55\" y=\"0\"/></arrow><molecule id=\"new-right\"><atom id=\"d\" name=\"C\"><point x=\"60\" y=\"0\"/></atom></molecule></cdml>",
        );
        let mut session = DocumentSession::load(&source).expect("session");
        let initial = session
            .observe_reaction_members_v1(&object(&session, "reaction"))
            .expect("initial observation");
        let prior_selection = session
            .select_reaction_members_v1(&initial)
            .expect("prior selection");
        let targets = DocumentReactionMemberTargetsV1::new(
            vec![object(&session, "new-left")],
            object(&session, "new-arrow"),
            vec![object(&session, "new-right")],
            Vec::new(),
            Vec::new(),
        )
        .expect("replacement targets");
        let command = session
            .begin_replace_reaction_members_v1(
                session.issue_authoring_capability_v1(),
                prior_selection.clone(),
                targets,
            )
            .expect("replace command");
        let request = session
            .resolve_replace_reaction_members_command_v1(command)
            .expect("replace request");
        let mut prepared = session
            .prepare_session_operation_transition_v1(request)
            .expect("prepared replacement");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("committed replacement");
        assert!(matches!(
            session.validate_reaction_selection_v1(&prior_selection),
            Err(ReactionMemberSelectionRefusalV1::StaleRevision)
        ));
        session
            .undo(session.snapshot().expect("replacement snapshot").revision())
            .expect("undo replacement");
        let undone = session
            .observe_reaction_members_v1(&object(&session, "reaction"))
            .expect("undo observation");
        assert_eq!(undone, initial);
        session
            .redo(session.snapshot().expect("undo snapshot").revision())
            .expect("redo replacement");
        let redone = session
            .observe_reaction_members_v1(&object(&session, "reaction"))
            .expect("redo observation");
        assert_eq!(
            redone
                .members()
                .iter()
                .map(|member| member.object_id().clone())
                .collect::<Vec<_>>(),
            vec![
                object(&session, "new-left"),
                object(&session, "new-arrow"),
                object(&session, "new-right"),
            ]
        );
    }

    #[test]
    fn corrupted_reaction_identity_refuses_list_and_single_observation() {
        let session = DocumentSession::load(SOURCE).expect("session");
        let reaction_object_id = object(&session, "reaction");
        let projection = session.observe(0).expect("projection");
        let mut document = session
            .current_document_v1()
            .detached_candidate()
            .expect("detached document");
        document.corrupt_direct_document_object_id_for_test("reaction");

        assert!(matches!(
            build_reaction_list_observation_v1(&document, projection.projection()),
            Err(ReactionMemberSelectionRefusalV1::InvalidIdentity(_))
        ));
        assert!(matches!(
            observe_reaction(&document, &reaction_object_id),
            Err(ReactionMemberSelectionRefusalV1::InvalidIdentity(_))
        ));
    }

    #[test]
    fn corrupted_member_identity_refuses_list_and_single_observation() {
        let session = DocumentSession::load(SOURCE).expect("session");
        let reaction_object_id = object(&session, "reaction");
        let projection = session.observe(0).expect("projection");
        let mut document = session
            .current_document_v1()
            .detached_candidate()
            .expect("detached document");
        document.corrupt_direct_document_object_id_for_test("left");

        assert!(matches!(
            build_reaction_list_observation_v1(&document, projection.projection()),
            Err(ReactionMemberSelectionRefusalV1::InvalidIdentity(_))
        ));
        assert!(matches!(
            observe_reaction(&document, &reaction_object_id),
            Err(ReactionMemberSelectionRefusalV1::InvalidIdentity(_))
        ));
    }
}

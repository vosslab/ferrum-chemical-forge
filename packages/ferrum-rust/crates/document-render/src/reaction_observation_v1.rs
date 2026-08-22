//! Immutable, renderer-backed reaction lifecycle observations.
//!
//! This module combines document-owned direct-CDML semantics with the exact
//! admitted root observations issued by the render interaction session. It
//! deliberately exposes facts only: no CDML, DOM handles, render plan, or
//! mutable root collection crosses this boundary.

use std::collections::{HashMap, hash_map::DefaultHasher};
use std::hash::{Hash, Hasher};

use ferrum_document::{
    DirectReactionMemberV1, DirectReactionRoleV1, DocumentFenceV1, DocumentSession,
    ReactionDefinitionDiagnosticV1, inspect_direct_reactions_v1,
};

use crate::{
    RenderInteractionBoundsV1, RenderInteractionErrorV1, RenderInteractionObservationV1,
    RenderInteractionRootV1,
};

/// Closed disposition of a retained direct reaction definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReactionDefinitionDispositionV1 {
    Strict,
    DisplayOnly,
}

/// One immutable role member fact. The ID remains a selector only when a
/// separately issued `ReactionSelectionV1` is accepted by a later operation.
#[derive(Clone, Debug, PartialEq)]
pub struct ReactionMemberObservationV1 {
    identifier: String,
    role: DirectReactionRoleV1,
    role_ordinal: u32,
    source_order: u32,
    bounds: Option<RenderInteractionBoundsV1>,
}
impl ReactionMemberObservationV1 {
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    #[must_use]
    pub const fn role(&self) -> DirectReactionRoleV1 {
        self.role
    }
    #[must_use]
    pub const fn role_ordinal(&self) -> u32 {
        self.role_ordinal
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    #[must_use]
    pub const fn bounds(&self) -> Option<RenderInteractionBoundsV1> {
        self.bounds
    }
}

/// One preserved reaction record in direct document source order.
#[derive(Clone, Debug, PartialEq)]
pub struct ReactionObservationV1 {
    reaction_id: String,
    source_order: u32,
    disposition: ReactionDefinitionDispositionV1,
    diagnostics: Vec<ReactionDefinitionDiagnosticV1>,
    membership_digest: String,
    members: Vec<ReactionMemberObservationV1>,
    union_bounds: Option<RenderInteractionBoundsV1>,
}
impl ReactionObservationV1 {
    #[must_use]
    pub fn reaction_id(&self) -> &str {
        &self.reaction_id
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    #[must_use]
    pub const fn disposition(&self) -> ReactionDefinitionDispositionV1 {
        self.disposition
    }
    #[must_use]
    pub fn diagnostics(&self) -> &[ReactionDefinitionDiagnosticV1] {
        &self.diagnostics
    }
    #[must_use]
    pub fn membership_digest(&self) -> &str {
        &self.membership_digest
    }
    #[must_use]
    pub fn members(&self) -> &[ReactionMemberObservationV1] {
        &self.members
    }
    #[must_use]
    pub const fn union_bounds(&self) -> Option<RenderInteractionBoundsV1> {
        self.union_bounds
    }
}

/// Exact session/fence-bound reaction list issued by one render observation.
#[derive(Clone, Debug)]
pub struct ReactionListObservationV1 {
    origin: u64,
    capability: u64,
    fence: DocumentFenceV1,
    reactions: Vec<ReactionObservationV1>,
}
impl ReactionListObservationV1 {
    #[must_use]
    pub const fn fence(&self) -> DocumentFenceV1 {
        self.fence
    }
    #[must_use]
    pub fn reactions(&self) -> &[ReactionObservationV1] {
        &self.reactions
    }
}

/// Opaque authority to operate on one frozen strict, renderer-admitted reaction.
/// It has no public constructor and intentionally offers no root-set conversion.
#[derive(Debug)]
pub struct ReactionSelectionV1 {
    origin: u64,
    capability: u64,
    fence: DocumentFenceV1,
    reaction_id: String,
    membership_digest: String,
}
impl ReactionSelectionV1 {
    #[must_use]
    pub fn reaction_id(&self) -> &str {
        &self.reaction_id
    }
    pub(crate) fn membership_digest(&self) -> &str {
        &self.membership_digest
    }
    pub(crate) const fn fence(&self) -> DocumentFenceV1 {
        self.fence
    }
}

/// Observe all direct semantic reaction records without mutating the session.
pub(crate) fn observe_reaction_list_v1(
    session: &DocumentSession,
    origin: u64,
    rendered: &RenderInteractionObservationV1,
) -> Result<ReactionListObservationV1, RenderInteractionErrorV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
    let definitions = inspect_direct_reactions_v1(snapshot.cdml())
        .map_err(|_| RenderInteractionErrorV1::Observation)?;
    let roots = rendered
        .roots()
        .iter()
        .map(|root| (root.identifier(), root))
        .collect::<HashMap<_, _>>();
    let reactions = definitions
        .into_iter()
        .map(|definition| {
            let members = definition
                .members()
                .iter()
                .map(|member| member_observation(member, &roots))
                .collect::<Vec<_>>();
            let all_rendered = members.iter().all(|member| member.bounds().is_some());
            let mut diagnostics = definition.diagnostics().to_vec();
            if !all_rendered {
                diagnostics.push(ReactionDefinitionDiagnosticV1::UnrenderableMember);
                diagnostics.sort();
                diagnostics.dedup();
            }
            let reaction_id = definition
                .identifier()
                .filter(|value| !value.trim().is_empty());
            let strict = reaction_id.is_some() && diagnostics.is_empty();
            let union_bounds = strict.then(|| union_bounds(&members)).flatten();
            ReactionObservationV1 {
                reaction_id: reaction_id.unwrap_or_default().to_owned(),
                source_order: definition.source_order(),
                disposition: if strict {
                    ReactionDefinitionDispositionV1::Strict
                } else {
                    ReactionDefinitionDispositionV1::DisplayOnly
                },
                diagnostics,
                membership_digest: reaction_id.map_or_else(String::new, |value| {
                    membership_digest(value, definition.members())
                }),
                members,
                union_bounds,
            }
        })
        .collect();
    Ok(ReactionListObservationV1 {
        origin,
        capability: rendered.capability(),
        fence: rendered.fence(),
        reactions,
    })
}

pub(crate) fn validate_reaction_list_v1(
    session: &DocumentSession,
    origin: u64,
    list: &ReactionListObservationV1,
) -> Result<(), RenderInteractionErrorV1> {
    if list.origin != origin {
        return Err(RenderInteractionErrorV1::ForeignSession);
    }
    if list.capability == 0 {
        return Err(RenderInteractionErrorV1::SelectionChanged);
    }
    let snapshot = session
        .snapshot()
        .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
    if snapshot.revision() != list.fence.revision() {
        return Err(RenderInteractionErrorV1::StaleRevision);
    }
    if snapshot.digest() != &list.fence.digest() {
        return Err(RenderInteractionErrorV1::StaleDigest);
    }
    Ok(())
}

pub(crate) fn select_reaction_v1(
    session: &DocumentSession,
    origin: u64,
    list: &ReactionListObservationV1,
    reaction_id: &str,
) -> Result<ReactionSelectionV1, RenderInteractionErrorV1> {
    validate_reaction_list_v1(session, origin, list)?;
    let reaction = list
        .reactions
        .iter()
        .find(|value| value.reaction_id() == reaction_id)
        .ok_or(RenderInteractionErrorV1::SelectionChanged)?;
    if reaction.disposition() != ReactionDefinitionDispositionV1::Strict
        || reaction.union_bounds().is_none()
    {
        return Err(RenderInteractionErrorV1::DisplayOnly);
    }
    Ok(ReactionSelectionV1 {
        origin,
        capability: list.capability,
        fence: list.fence,
        reaction_id: reaction.reaction_id.clone(),
        membership_digest: reaction.membership_digest.clone(),
    })
}

pub(crate) fn validate_reaction_selection_v1(
    session: &DocumentSession,
    origin: u64,
    selection: &ReactionSelectionV1,
) -> Result<(), RenderInteractionErrorV1> {
    if selection.origin != origin {
        return Err(RenderInteractionErrorV1::ForeignSession);
    }
    if selection.capability == 0 {
        return Err(RenderInteractionErrorV1::SelectionChanged);
    }
    let snapshot = session
        .snapshot()
        .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
    if snapshot.revision() != selection.fence.revision()
        || snapshot.digest() != &selection.fence.digest()
    {
        return Err(RenderInteractionErrorV1::StaleRevision);
    }
    let definitions = inspect_direct_reactions_v1(snapshot.cdml())
        .map_err(|_| RenderInteractionErrorV1::Observation)?;
    let definition = definitions
        .into_iter()
        .find(|value| value.identifier() == Some(selection.reaction_id.as_str()))
        .ok_or(RenderInteractionErrorV1::SelectionChanged)?;
    (definition.is_strict()
        && membership_digest(selection.reaction_id(), definition.members())
            == selection.membership_digest)
        .then_some(())
        .ok_or(RenderInteractionErrorV1::SelectionChanged)
}

/// Resolve the frozen strict selection to its complete direct-member IDs.
///
/// This stays crate-private so callers cannot turn a reaction selection into a
/// general mutable root collection.  Gesture owners use it only to derive the
/// exact complete-root transaction candidate.
pub(crate) fn selected_reaction_member_ids_v1(
    session: &DocumentSession,
    origin: u64,
    selection: &ReactionSelectionV1,
) -> Result<Vec<String>, RenderInteractionErrorV1> {
    validate_reaction_selection_v1(session, origin, selection)?;
    let snapshot = session
        .snapshot()
        .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
    let definition = inspect_direct_reactions_v1(snapshot.cdml())
        .map_err(|_| RenderInteractionErrorV1::Observation)?
        .into_iter()
        .find(|value| value.identifier() == Some(selection.reaction_id.as_str()))
        .ok_or(RenderInteractionErrorV1::SelectionChanged)?;
    Ok(definition
        .members()
        .iter()
        .map(|member| member.identifier().to_owned())
        .collect())
}

fn member_observation(
    member: &DirectReactionMemberV1,
    roots: &HashMap<&str, &RenderInteractionRootV1>,
) -> ReactionMemberObservationV1 {
    ReactionMemberObservationV1 {
        identifier: member.identifier().to_owned(),
        role: member.role(),
        role_ordinal: member.role_ordinal(),
        source_order: member.source_order(),
        bounds: roots.get(member.identifier()).map(|root| root.bounds()),
    }
}

fn union_bounds(members: &[ReactionMemberObservationV1]) -> Option<RenderInteractionBoundsV1> {
    members
        .iter()
        .filter_map(|member| member.bounds())
        .reduce(RenderInteractionBoundsV1::union)
}

fn membership_digest(reaction_id: &str, members: &[DirectReactionMemberV1]) -> String {
    let mut hasher = DefaultHasher::new();
    reaction_id.hash(&mut hasher);
    for member in members {
        member.role().hash(&mut hasher);
        member.identifier().hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use ferrum_document::DocumentSession;

    use super::*;
    use crate::RenderInteractionSessionV1;

    const SOURCE: &str = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"right\"><atom id=\"right-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule>",
        "<arrow id=\"a\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow>",
        "<reaction id=\"r\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction></cdml>"
    );

    fn fence(session: &RenderInteractionSessionV1) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    #[test]
    fn list_is_renderer_backed_and_selection_is_session_bound() {
        let session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
        let list = session
            .observe_reaction_list_v1(fence(&session))
            .expect("list");
        let reaction = &list.reactions()[0];
        assert_eq!(reaction.reaction_id(), "r");
        assert_eq!(
            reaction.disposition(),
            ReactionDefinitionDispositionV1::Strict
        );
        assert_eq!(reaction.members().len(), 3);
        assert!(reaction.union_bounds().is_some());
        let selection = session.select_reaction_v1(&list, "r").expect("selection");
        session
            .validate_reaction_selection_v1(&selection)
            .expect("valid");
        let foreign = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
        assert!(matches!(
            foreign.validate_reaction_selection_v1(&selection),
            Err(RenderInteractionErrorV1::ForeignSession)
        ));
    }

    #[test]
    fn malformed_definition_is_observed_but_never_selectable() {
        let source = SOURCE.replace("<reaction id=\"r\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction>", "<reaction id=\"r\"><reactant idref=\"left\"/></reaction>");
        let session =
            RenderInteractionSessionV1::new(DocumentSession::load(&source).expect("load"));
        let list = session
            .observe_reaction_list_v1(fence(&session))
            .expect("list");
        assert_eq!(
            list.reactions()[0].disposition(),
            ReactionDefinitionDispositionV1::DisplayOnly
        );
        assert!(matches!(
            session.select_reaction_v1(&list, "r"),
            Err(RenderInteractionErrorV1::DisplayOnly)
        ));
    }
}

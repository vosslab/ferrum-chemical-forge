//! Namespace-aware semantic facts for direct CDML roots.
//!
//! Compatibility XML remains retained without interpretation. This module is
//! the shared classifier for direct-root reaction semantics.

use std::collections::{HashMap, HashSet};

use thiserror::Error;
use xot::Xot;

use crate::projection_identity_v1::projection_document_object_id_from_record_v1;

use super::{
    CDML_NAMESPACE, DocumentObjectIdV1, PersistentId, TypedClass, TypedDocument,
    TypedDocumentError, TypedRecord, XmlSerializationError, element_name, ferrum_cdml_element_name,
    is_ferrum_cdml_name,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DirectCdmlRootKindV1 {
    Molecule,
    Arrow,
    Text,
    Plus,
    Reaction,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectCdmlRootV1 {
    kind: DirectCdmlRootKindV1,
    identifier: Option<String>,
    reaction_members: Vec<String>,
}

impl DirectCdmlRootV1 {
    #[must_use]
    pub const fn kind(&self) -> DirectCdmlRootKindV1 {
        self.kind
    }

    #[must_use]
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }

    #[must_use]
    pub fn reaction_members(&self) -> &[String] {
        &self.reaction_members
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectCdmlSemanticIndexV1 {
    roots: Vec<DirectCdmlRootV1>,
    reserved_ids: HashSet<String>,
    reactions: Vec<DirectReactionSemanticsEntryV1>,
}

impl DirectCdmlSemanticIndexV1 {
    /// Parse retained CDML and classify only direct semantic roots.
    pub fn parse(source: &str) -> Result<Self, DirectCdmlSemanticErrorV1> {
        let document = TypedDocument::parse(source)?;
        Ok(Self::from_document(&document))
    }

    pub(crate) fn from_document(document: &TypedDocument) -> Self {
        let semantics = DirectReactionSemanticsV1::from_typed_document_v1(document);
        let roots = semantics
            .direct_roots
            .iter()
            .map(DirectCdmlRootV1::from_semantics)
            .collect();
        let reserved_ids = document
            .indexed()
            .persistent_ids()
            .map(|identifier| identifier.as_str().to_owned())
            .collect();
        Self {
            roots,
            reserved_ids,
            reactions: semantics.reactions,
        }
    }

    #[must_use]
    pub fn roots(&self) -> &[DirectCdmlRootV1] {
        &self.roots
    }

    #[must_use]
    pub fn reserves_identifier(&self, identifier: &str) -> bool {
        self.reserved_ids.contains(identifier)
    }

    pub(crate) fn bind_durable_reactions_v1(
        &self,
        document: &TypedDocument,
    ) -> Result<DirectReactionDurableIndexV1, crate::ProjectionError> {
        bind_durable_reactions_v1(&self.reactions, document)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DirectReactionRoleV1 {
    Reactant,
    Product,
    Arrow,
    Condition,
    Plus,
}

impl DirectReactionRoleV1 {
    #[must_use]
    pub const fn local_name(self) -> &'static str {
        match self {
            Self::Reactant => "reactant",
            Self::Product => "product",
            Self::Arrow => "arrow",
            Self::Condition => "condition",
            Self::Plus => "plus",
        }
    }
}

/// One direct, recognized reaction reference.  This is a semantic fact, not a
/// mutable DOM handle and not permission to alter its target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectReactionMemberV1 {
    role: DirectReactionRoleV1,
    identifier: String,
    role_ordinal: u32,
    source_order: u32,
}

impl DirectReactionMemberV1 {
    #[must_use]
    pub const fn role(&self) -> DirectReactionRoleV1 {
        self.role
    }
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    #[must_use]
    pub const fn role_ordinal(&self) -> u32 {
        self.role_ordinal
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
}

/// Closed compatibility diagnostic for a direct reaction definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReactionDefinitionDiagnosticV1 {
    MissingReactionId,
    EmptyReactionId,
    UnknownRoleChild,
    MissingIdref,
    EmptyIdref,
    MissingReactants,
    MissingProducts,
    MissingArrow,
    MultipleArrows,
    MissingTarget,
    UnrenderableMember,
    WrongTargetKind,
    DuplicateTarget,
    CrossReactionReuse,
}

/// Immutable interpretation of one direct core reaction root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReactionDefinitionV1 {
    identifier: Option<String>,
    source_order: u32,
    members: Vec<DirectReactionMemberV1>,
    diagnostics: Vec<ReactionDefinitionDiagnosticV1>,
}

/// Shared retained-tree interpretation for direct reaction inspection and durable binding.
struct DirectReactionSemanticsV1 {
    direct_roots: Vec<DirectRootSemanticsV1>,
    reactions: Vec<DirectReactionSemanticsEntryV1>,
}

struct DirectRootSemanticsV1 {
    kind: DirectCdmlRootKindV1,
    identifier: Option<String>,
    root_source_order: u32,
    reaction: Option<ReactionDefinitionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DirectReactionSemanticsEntryV1 {
    reaction_source_id: PersistentId,
    members: Vec<DirectReactionMemberV1>,
    diagnostics: Vec<ReactionDefinitionDiagnosticV1>,
}

/// Crate-private durable identity binding for one source-semantic reaction index.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DirectReactionDurableIndexV1 {
    durable_reactions: Vec<DirectReactionDurableV1>,
    durable_reaction_by_object_id: HashMap<DocumentObjectIdV1, usize>,
}

impl DirectReactionDurableIndexV1 {
    pub(crate) fn durable_reaction_v1(
        &self,
        reaction_object_id: &DocumentObjectIdV1,
    ) -> Option<&DirectReactionDurableV1> {
        self.durable_reaction_by_object_id
            .get(reaction_object_id)
            .map(|index| &self.durable_reactions[*index])
    }

    pub(crate) fn durable_reactions_v1(&self) -> &[DirectReactionDurableV1] {
        &self.durable_reactions
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DirectReactionDurableV1 {
    reaction_object_id: DocumentObjectIdV1,
    members: Vec<DirectReactionDurableMemberV1>,
    diagnostics: Vec<ReactionDefinitionDiagnosticV1>,
}

impl DirectReactionDurableV1 {
    #[must_use]
    pub(crate) fn reaction_object_id(&self) -> &DocumentObjectIdV1 {
        &self.reaction_object_id
    }

    #[must_use]
    pub(crate) fn members(&self) -> &[DirectReactionDurableMemberV1] {
        &self.members
    }

    #[must_use]
    pub(crate) fn diagnostics(&self) -> &[ReactionDefinitionDiagnosticV1] {
        &self.diagnostics
    }

    #[must_use]
    pub(crate) const fn is_strict(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DirectReactionDurableMemberV1 {
    role: DirectReactionRoleV1,
    role_ordinal: u32,
    source_order: u32,
    member_object_id: DocumentObjectIdV1,
}

impl DirectReactionDurableMemberV1 {
    #[must_use]
    pub(crate) const fn role(&self) -> DirectReactionRoleV1 {
        self.role
    }

    #[must_use]
    pub(crate) const fn role_ordinal(&self) -> u32 {
        self.role_ordinal
    }

    #[must_use]
    pub(crate) const fn source_order(&self) -> u32 {
        self.source_order
    }

    #[must_use]
    pub(crate) fn member_object_id(&self) -> &DocumentObjectIdV1 {
        &self.member_object_id
    }
}

impl ReactionDefinitionV1 {
    #[must_use]
    pub fn identifier(&self) -> Option<&str> {
        self.identifier.as_deref()
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    #[must_use]
    pub fn members(&self) -> &[DirectReactionMemberV1] {
        &self.members
    }
    #[must_use]
    pub fn diagnostics(&self) -> &[ReactionDefinitionDiagnosticV1] {
        &self.diagnostics
    }
    #[must_use]
    pub const fn is_strict(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Parse the direct core reaction roots from compatible CDML without changing it.
/// Foreign-namespace and nested reaction lookalikes remain opaque preservation
/// content and therefore never appear in this result.
pub fn inspect_direct_reactions_v1(
    source: &str,
) -> Result<Vec<ReactionDefinitionV1>, DirectCdmlSemanticErrorV1> {
    let document = TypedDocument::parse(source)?;
    Ok(DirectReactionSemanticsV1::from_typed_document_v1(&document).definitions())
}

impl DirectReactionSemanticsV1 {
    fn from_typed_document_v1(document: &TypedDocument) -> Self {
        let tree = &document.indexed().xml.tree;
        let root = tree
            .document_element(document.indexed().xml.document)
            .expect("a parsed CDML document has a root");
        let mut direct_roots = tree
            .children(root)
            .enumerate()
            .filter_map(|(source_order, node)| {
                DirectRootSemanticsV1::from_node(tree, node, source_order as u32)
            })
            .collect::<Vec<_>>();
        let mut target_kinds = HashMap::<String, Vec<DirectCdmlRootKindV1>>::new();
        for item in &direct_roots {
            if let Some(identifier) = item.identifier.as_ref() {
                target_kinds
                    .entry(identifier.clone())
                    .or_default()
                    .push(item.kind);
            }
        }
        let mut definitions = direct_roots
            .iter()
            .filter_map(|item| item.reaction.clone())
            .collect::<Vec<_>>();
        let mut uses = HashMap::<String, usize>::new();
        for definition in &definitions {
            for member in definition.members() {
                *uses.entry(member.identifier().to_owned()).or_default() += 1;
            }
        }
        for definition in &mut definitions {
            let mut diagnostics = definition.diagnostics.clone();
            let mut seen = HashSet::new();
            let mut reactants = 0usize;
            let mut products = 0usize;
            let mut arrows = 0usize;
            for member in &definition.members {
                match member.role {
                    DirectReactionRoleV1::Reactant => reactants += 1,
                    DirectReactionRoleV1::Product => products += 1,
                    DirectReactionRoleV1::Arrow => arrows += 1,
                    DirectReactionRoleV1::Condition | DirectReactionRoleV1::Plus => {}
                }
                if !seen.insert(member.identifier.clone()) {
                    diagnostics.push(ReactionDefinitionDiagnosticV1::DuplicateTarget);
                }
                if uses[member.identifier()] > 1 {
                    diagnostics.push(ReactionDefinitionDiagnosticV1::CrossReactionReuse);
                }
                match target_kinds.get(member.identifier()) {
                    None => diagnostics.push(ReactionDefinitionDiagnosticV1::MissingTarget),
                    Some(kinds)
                        if kinds.len() != 1 || !role_accepts_kind(member.role, kinds[0]) =>
                    {
                        diagnostics.push(ReactionDefinitionDiagnosticV1::WrongTargetKind)
                    }
                    Some(_) => {}
                }
            }
            if reactants == 0 {
                diagnostics.push(ReactionDefinitionDiagnosticV1::MissingReactants);
            }
            if products == 0 {
                diagnostics.push(ReactionDefinitionDiagnosticV1::MissingProducts);
            }
            if arrows == 0 {
                diagnostics.push(ReactionDefinitionDiagnosticV1::MissingArrow);
            }
            if arrows > 1 {
                diagnostics.push(ReactionDefinitionDiagnosticV1::MultipleArrows);
            }
            diagnostics.sort();
            diagnostics.dedup();
            definition.diagnostics = diagnostics;
        }
        let mut definition_by_source_order = definitions
            .into_iter()
            .map(|definition| (definition.source_order, definition))
            .collect::<HashMap<_, _>>();
        for root in &mut direct_roots {
            if root.reaction.is_some() {
                root.reaction = definition_by_source_order.remove(&root.root_source_order);
            }
        }
        let reactions = direct_roots
            .iter()
            .filter_map(|root| {
                let definition = root.reaction.as_ref()?;
                let source_id = PersistentId::new(definition.identifier.clone()?).ok()?;
                Some(DirectReactionSemanticsEntryV1 {
                    reaction_source_id: source_id,
                    members: definition.members.clone(),
                    diagnostics: definition.diagnostics.clone(),
                })
            })
            .collect();
        Self {
            direct_roots,
            reactions,
        }
    }

    fn definitions(&self) -> Vec<ReactionDefinitionV1> {
        self.direct_roots
            .iter()
            .filter_map(|root| root.reaction.clone())
            .collect()
    }
}

fn bind_durable_reactions_v1(
    reactions: &[DirectReactionSemanticsEntryV1],
    document: &TypedDocument,
) -> Result<DirectReactionDurableIndexV1, crate::ProjectionError> {
    let mut direct_records = HashMap::<&str, Option<&TypedRecord>>::new();
    for child in document.root().typed_children() {
        let record = child.record();
        if record.path().components().len() == 1 {
            if let Some(identifier) = record.attribute("id") {
                direct_records
                    .entry(identifier)
                    .and_modify(|entry| *entry = None)
                    .or_insert(Some(record));
            }
        }
    }
    let durable_reactions = reactions
        .iter()
        .filter_map(|reaction| {
            let reaction_record = direct_records
                .get(reaction.reaction_source_id.as_str())
                .and_then(|record| *record)
                .filter(|record| record.class() == TypedClass::Reaction)?;
            Some((|| {
                let reaction_object_id =
                    projection_document_object_id_from_record_v1(reaction_record)?;
                let members = reaction
                    .members
                    .iter()
                    .filter_map(|member| {
                        reaction_record.typed_children().iter().find(|child| {
                            child.position() == member.source_order
                                && reaction_role_matches_class(member.role, child.record().class())
                                && child.record().attribute("idref") == Some(member.identifier())
                        })?;
                        let target = direct_records
                            .get(member.identifier())
                            .and_then(|record| *record)
                            .filter(|record| {
                                role_accepts_typed_class(member.role, record.class())
                            })?;
                        Some(projection_document_object_id_from_record_v1(target).map(
                            |member_object_id| DirectReactionDurableMemberV1 {
                                role: member.role,
                                role_ordinal: member.role_ordinal,
                                source_order: member.source_order,
                                member_object_id,
                            },
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(DirectReactionDurableV1 {
                    reaction_object_id,
                    members,
                    diagnostics: reaction.diagnostics.clone(),
                })
            })())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let durable_reaction_by_object_id = durable_reactions
        .iter()
        .enumerate()
        .map(|(index, reaction)| (reaction.reaction_object_id.clone(), index))
        .collect();
    Ok(DirectReactionDurableIndexV1 {
        durable_reactions,
        durable_reaction_by_object_id,
    })
}

impl Ord for ReactionDefinitionDiagnosticV1 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}
impl PartialOrd for ReactionDefinitionDiagnosticV1 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl DirectCdmlRootV1 {
    fn from_semantics(root: &DirectRootSemanticsV1) -> Self {
        Self {
            kind: root.kind,
            identifier: root.identifier.clone(),
            reaction_members: root
                .reaction
                .as_ref()
                .map(|reaction| {
                    reaction
                        .members
                        .iter()
                        .map(|member| member.identifier.clone())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

impl DirectRootSemanticsV1 {
    fn from_node(tree: &Xot, node: xot::Node, source_order: u32) -> Option<Self> {
        let (local, namespace) = element_name(tree, node)?;
        let core = namespace == CDML_NAMESPACE;
        let kind = if core {
            match local.as_str() {
                "molecule" => DirectCdmlRootKindV1::Molecule,
                "arrow" => DirectCdmlRootKindV1::Arrow,
                "text" => DirectCdmlRootKindV1::Text,
                "plus" => DirectCdmlRootKindV1::Plus,
                "reaction" => DirectCdmlRootKindV1::Reaction,
                _ => DirectCdmlRootKindV1::Other,
            }
        } else {
            DirectCdmlRootKindV1::Other
        };
        let identifier = attribute(tree, node, "id").map(str::to_owned);
        let reaction = (kind == DirectCdmlRootKindV1::Reaction)
            .then(|| parse_reaction(tree, node, identifier.clone(), source_order));
        Some(Self {
            kind,
            identifier,
            root_source_order: source_order,
            reaction,
        })
    }
}

fn parse_reaction(
    tree: &Xot,
    reaction: xot::Node,
    identifier: Option<String>,
    source_order: u32,
) -> ReactionDefinitionV1 {
    let mut diagnostics = match identifier.as_deref() {
        None => vec![ReactionDefinitionDiagnosticV1::MissingReactionId],
        Some(value) if value.trim().is_empty() => {
            vec![ReactionDefinitionDiagnosticV1::EmptyReactionId]
        }
        Some(_) => Vec::new(),
    };
    let mut members = Vec::new();
    let mut role_counts = HashMap::<DirectReactionRoleV1, u32>::new();
    for (source_order, child) in tree.children(reaction).enumerate() {
        let Some((local, namespace)) = element_name(tree, child) else {
            continue;
        };
        if !(namespace == CDML_NAMESPACE) {
            diagnostics.push(ReactionDefinitionDiagnosticV1::UnknownRoleChild);
            continue;
        }
        let role = match local.as_str() {
            "reactant" => DirectReactionRoleV1::Reactant,
            "product" => DirectReactionRoleV1::Product,
            "arrow" => DirectReactionRoleV1::Arrow,
            "condition" => DirectReactionRoleV1::Condition,
            "plus" => DirectReactionRoleV1::Plus,
            _ => {
                diagnostics.push(ReactionDefinitionDiagnosticV1::UnknownRoleChild);
                continue;
            }
        };
        let Some(idref) = attribute(tree, child, "idref") else {
            diagnostics.push(ReactionDefinitionDiagnosticV1::MissingIdref);
            continue;
        };
        if idref.trim().is_empty() {
            diagnostics.push(ReactionDefinitionDiagnosticV1::EmptyIdref);
            continue;
        }
        let role_ordinal = role_counts.entry(role).or_default();
        members.push(DirectReactionMemberV1 {
            role,
            identifier: idref.to_owned(),
            role_ordinal: *role_ordinal,
            source_order: source_order as u32,
        });
        *role_ordinal += 1;
    }
    ReactionDefinitionV1 {
        identifier,
        source_order,
        members,
        diagnostics,
    }
}

fn role_accepts_kind(role: DirectReactionRoleV1, kind: DirectCdmlRootKindV1) -> bool {
    matches!(
        (role, kind),
        (
            DirectReactionRoleV1::Reactant | DirectReactionRoleV1::Product,
            DirectCdmlRootKindV1::Molecule
        ) | (DirectReactionRoleV1::Arrow, DirectCdmlRootKindV1::Arrow)
            | (DirectReactionRoleV1::Condition, DirectCdmlRootKindV1::Text)
            | (DirectReactionRoleV1::Plus, DirectCdmlRootKindV1::Plus)
    )
}

fn reaction_role_matches_class(role: DirectReactionRoleV1, class: TypedClass) -> bool {
    matches!(
        (role, class),
        (DirectReactionRoleV1::Reactant, TypedClass::ReactionReactant)
            | (DirectReactionRoleV1::Product, TypedClass::ReactionProduct)
            | (DirectReactionRoleV1::Arrow, TypedClass::ReactionArrow)
            | (
                DirectReactionRoleV1::Condition,
                TypedClass::ReactionCondition
            )
            | (DirectReactionRoleV1::Plus, TypedClass::ReactionPlus)
    )
}

fn role_accepts_typed_class(role: DirectReactionRoleV1, class: TypedClass) -> bool {
    matches!(
        (role, class),
        (
            DirectReactionRoleV1::Reactant | DirectReactionRoleV1::Product,
            TypedClass::Molecule
        ) | (DirectReactionRoleV1::Arrow, TypedClass::CanvasArrow)
            | (DirectReactionRoleV1::Condition, TypedClass::CanvasText)
            | (DirectReactionRoleV1::Plus, TypedClass::CanvasPlus)
    )
}

#[derive(Debug, Error)]
pub enum DirectCdmlSemanticErrorV1 {
    #[error(transparent)]
    Document(#[from] TypedDocumentError),
    #[error(transparent)]
    Serialization(#[from] XmlSerializationError),
    #[error("direct reaction definition is not strict and cannot be rewritten")]
    InvalidReactionDefinition,
}

/// Append one complete direct reaction using the CDML root namespace.
///
/// The retained XML tree determines the insertion point, preserving every
/// existing direct-child ordering and opaque namespace.
pub(crate) fn append_direct_cdml_reaction_v1(
    source: &str,
    reaction_id: &str,
    roles: &[(DirectReactionRoleV1, String)],
) -> Result<String, DirectCdmlSemanticErrorV1> {
    let document = TypedDocument::parse(source)?;
    let mut candidate = document.detached_candidate()?;
    let indexed = candidate.detached_indexed_mut();
    let tree = &mut indexed.xml.tree;
    let root = tree
        .document_element(indexed.xml.document)
        .expect("a parsed CDML document has a root");
    let reaction_name = ferrum_cdml_element_name(tree, "reaction");
    let reaction = tree.new_element(reaction_name);
    let id = tree.add_name("id");
    tree.set_attribute(reaction, id, reaction_id);
    for (role, identifier) in roles {
        let child_name = ferrum_cdml_element_name(tree, role.local_name());
        let child = tree.new_element(child_name);
        let idref = tree.add_name("idref");
        tree.set_attribute(child, idref, identifier);
        tree.append(reaction, child)
            .map_err(TypedDocumentError::Mutation)?;
    }
    tree.append(root, reaction)
        .map_err(TypedDocumentError::Mutation)?;
    Ok(TypedDocument::parse(&candidate.to_xml()?)?.to_xml()?)
}

/// Replace only the recognized role children of one strict direct reaction.
///
/// Compatibility records are deliberately not normalized: callers must first
/// prove strictness through `inspect_direct_reactions_v1`.  Unknown children
/// and foreign namespaces therefore never cross this edit seam.
pub(crate) fn replace_direct_cdml_reaction_members_v1(
    source: &str,
    reaction_id: &str,
    roles: &[(DirectReactionRoleV1, String)],
) -> Result<String, DirectCdmlSemanticErrorV1> {
    let definitions = inspect_direct_reactions_v1(source)?;
    if !definitions
        .iter()
        .any(|definition| definition.identifier() == Some(reaction_id) && definition.is_strict())
    {
        return Err(DirectCdmlSemanticErrorV1::InvalidReactionDefinition);
    }
    let document = TypedDocument::parse(source)?;
    let mut candidate = document.detached_candidate()?;
    let indexed = candidate.detached_indexed_mut();
    let tree = &mut indexed.xml.tree;
    let root = tree
        .document_element(indexed.xml.document)
        .expect("a parsed CDML document has a root");
    let reaction = tree
        .children(root)
        .find(|child| {
            element_name(tree, *child).is_some_and(|(local, namespace)| {
                is_ferrum_cdml_name(&local, &namespace, "reaction")
                    && attribute(tree, *child, "id") == Some(reaction_id)
            })
        })
        .ok_or(DirectCdmlSemanticErrorV1::InvalidReactionDefinition)?;
    let children = tree.children(reaction).collect::<Vec<_>>();
    for child in children {
        if is_reaction_role(tree, child) {
            tree.remove(child).map_err(TypedDocumentError::Mutation)?;
        }
    }
    for (role, identifier) in roles {
        let child_name = ferrum_cdml_element_name(tree, role.local_name());
        let child = tree.new_element(child_name);
        let idref = tree.add_name("idref");
        tree.set_attribute(child, idref, identifier);
        tree.append(reaction, child)
            .map_err(TypedDocumentError::Mutation)?;
    }
    Ok(TypedDocument::parse(&candidate.to_xml()?)?.to_xml()?)
}

/// Remove exactly one strict direct reaction definition, preserving all roots
/// it references. This is intentionally distinct from aggregate deletion.
pub(crate) fn delete_direct_cdml_reaction_definition_v1(
    source: &str,
    reaction_id: &str,
) -> Result<String, DirectCdmlSemanticErrorV1> {
    let definitions = inspect_direct_reactions_v1(source)?;
    if !definitions
        .iter()
        .any(|definition| definition.identifier() == Some(reaction_id) && definition.is_strict())
    {
        return Err(DirectCdmlSemanticErrorV1::InvalidReactionDefinition);
    }
    let document = TypedDocument::parse(source)?;
    let mut candidate = document.detached_candidate()?;
    let indexed = candidate.detached_indexed_mut();
    let tree = &mut indexed.xml.tree;
    let root = tree
        .document_element(indexed.xml.document)
        .expect("a parsed CDML document has a root");
    let reaction = tree
        .children(root)
        .find(|child| {
            element_name(tree, *child).is_some_and(|(local, namespace)| {
                is_ferrum_cdml_name(&local, &namespace, "reaction")
                    && attribute(tree, *child, "id") == Some(reaction_id)
            })
        })
        .ok_or(DirectCdmlSemanticErrorV1::InvalidReactionDefinition)?;
    tree.remove(reaction)
        .map_err(TypedDocumentError::Mutation)?;
    Ok(TypedDocument::parse(&candidate.to_xml()?)?.to_xml()?)
}

fn is_reaction_role(tree: &Xot, node: xot::Node) -> bool {
    element_name(tree, node).is_some_and(|(local, namespace)| {
        matches!(
            local.as_str(),
            "reactant" | "product" | "arrow" | "condition" | "plus"
        ) && namespace == CDML_NAMESPACE
    })
}

fn attribute<'a>(tree: &'a Xot, node: xot::Node, expected: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local, namespace) = tree.name_ns_str(name);
        (local == expected && namespace.is_empty()).then_some(value.as_str())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_ignores_foreign_and_nested_lookalikes_but_reserves_literal_ids() {
        let index = DirectCdmlSemanticIndexV1::parse(concat!(
            "<c:cdml xmlns:c=\"urn:ferrum:cdml\" ",
            "xmlns:v=\"urn:vendor\"><v:molecule id=\"foreign-molecule\"/>",
            "<c:molecule id=\"core\"><c:atom id=\"core-atom\" name=\"C\"><c:point x=\"0\" y=\"0\"/></c:atom><v:reaction id=\"nested\"/></c:molecule>",
            "<c:reaction id=\"r\"><c:arrow idref=\"arrow\"/>",
            "<v:plus idref=\"foreign\"/></c:reaction></c:cdml>"
        ))
        .expect("fixture parses");
        assert_eq!(index.roots()[0].kind(), DirectCdmlRootKindV1::Other);
        assert_eq!(index.roots()[1].kind(), DirectCdmlRootKindV1::Molecule);
        assert_eq!(index.roots()[2].reaction_members(), ["arrow"]);
        assert!(index.reserves_identifier("foreign-molecule"));
        assert!(index.reserves_identifier("nested"));
    }

    #[test]
    fn typed_append_uses_the_root_namespace_and_retains_direct_child_order() {
        let source = concat!(
            "<c:cdml xmlns:c=\"urn:ferrum:cdml\" ",
            "xmlns:v=\"urn:vendor\"><v:note id=\"opaque\"/>",
            "<c:molecule id=\"m\"><c:atom id=\"m-atom\" name=\"C\"><c:point x=\"0\" y=\"0\"/></c:atom></c:molecule></c:cdml>"
        );
        let candidate = append_direct_cdml_reaction_v1(
            source,
            "rxn-1",
            &[(DirectReactionRoleV1::Reactant, "m".to_owned())],
        )
        .expect("append succeeds");
        let index = DirectCdmlSemanticIndexV1::parse(&candidate).expect("candidate parses");
        assert_eq!(index.roots()[0].identifier(), Some("opaque"));
        assert_eq!(index.roots()[1].identifier(), Some("m"));
        assert_eq!(index.roots()[2].kind(), DirectCdmlRootKindV1::Reaction);
        assert_eq!(index.roots()[2].reaction_members(), ["m"]);
    }

    #[test]
    fn reaction_definition_is_namespace_aware_and_preserves_member_order() {
        let definitions = inspect_direct_reactions_v1(concat!(
            "<c:cdml xmlns:c=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\">",
            "<c:molecule id=\"left\"><c:atom id=\"left-atom\" name=\"C\"><c:point x=\"0\" y=\"0\"/></c:atom></c:molecule><c:molecule id=\"right\"><c:atom id=\"right-atom\" name=\"O\"><c:point x=\"1\" y=\"0\"/></c:atom></c:molecule><c:arrow id=\"a\"/>",
            "<c:reaction id=\"r\"><c:reactant idref=\"left\"/><c:product idref=\"right\"/><c:arrow idref=\"a\"/></c:reaction>",
            "<v:reaction id=\"foreign\"><v:reactant idref=\"left\"/></v:reaction>",
            "<c:molecule id=\"nested\"><c:atom id=\"nested-atom\" name=\"N\"><c:point x=\"2\" y=\"0\"/></c:atom><c:reaction id=\"nested-r\"/></c:molecule></c:cdml>"
        )).expect("fixture parses");
        assert_eq!(definitions.len(), 1);
        let definition = &definitions[0];
        assert_eq!(definition.identifier(), Some("r"));
        assert!(definition.is_strict());
        assert_eq!(
            definition
                .members()
                .iter()
                .map(DirectReactionMemberV1::identifier)
                .collect::<Vec<_>>(),
            ["left", "right", "a"]
        );
        assert_eq!(definition.members()[0].role_ordinal(), 0);
    }

    #[test]
    fn malformed_direct_reaction_is_retained_as_display_only_definition() {
        let definitions = inspect_direct_reactions_v1("<cdml xmlns=\"urn:ferrum:cdml\"><reaction id=\"r\"><reactant idref=\"missing\"/><arrow idref=\"a\"/><arrow idref=\"a\"/></reaction><arrow id=\"a\"/></cdml>").expect("fixture parses");
        assert_eq!(definitions.len(), 1);
        assert!(!definitions[0].is_strict());
        assert!(
            definitions[0]
                .diagnostics()
                .contains(&ReactionDefinitionDiagnosticV1::MissingProducts)
        );
        assert!(
            definitions[0]
                .diagnostics()
                .contains(&ReactionDefinitionDiagnosticV1::MultipleArrows)
        );
    }

    #[test]
    fn durable_reaction_binds_direct_typed_member_identities_in_role_order() {
        let document = TypedDocument::parse(concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\">",
            "<molecule id=\"left\"><atom id=\"left-atom\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
            "<molecule id=\"right\"><atom id=\"right-atom\" name=\"O\"><point x=\"1\" y=\"0\"/></atom></molecule>",
            "<molecule id=\"product\"><atom id=\"product-atom\" name=\"N\"><point x=\"2\" y=\"0\"/></atom></molecule>",
            "<arrow id=\"arrow\"/><text id=\"condition\"/><plus id=\"plus\"/>",
            "<reaction id=\"reaction\"><reactant idref=\"left\"/><reactant idref=\"right\"/><product idref=\"product\"/><arrow idref=\"arrow\"/><condition idref=\"condition\"/><plus idref=\"plus\"/></reaction>",
            "</cdml>"
        ))
        .expect("fixture parses");
        let index = DirectCdmlSemanticIndexV1::from_document(&document);
        let durable = index
            .bind_durable_reactions_v1(&document)
            .expect("durable identities bind");
        let reaction = durable
            .durable_reactions_v1()
            .first()
            .expect("one direct reaction has durable semantics");

        assert!(reaction.is_strict());
        assert_eq!(
            reaction
                .members()
                .iter()
                .map(|member| (member.role(), member.role_ordinal(), member.source_order()))
                .collect::<Vec<_>>(),
            [
                (DirectReactionRoleV1::Reactant, 0, 0),
                (DirectReactionRoleV1::Reactant, 1, 1),
                (DirectReactionRoleV1::Product, 0, 2),
                (DirectReactionRoleV1::Arrow, 0, 3),
                (DirectReactionRoleV1::Condition, 0, 4),
                (DirectReactionRoleV1::Plus, 0, 5),
            ]
        );
        assert_eq!(
            document
                .resolve_document_object_id(reaction.reaction_object_id())
                .expect("durable reaction lookup succeeds")
                .expect("durable reaction resolves")
                .class(),
            TypedClass::Reaction
        );
        for member in reaction.members() {
            assert!(
                document
                    .resolve_document_object_id(member.member_object_id())
                    .expect("durable member lookup succeeds")
                    .is_some()
            );
        }
        assert_eq!(
            durable.durable_reaction_v1(reaction.reaction_object_id()),
            Some(reaction)
        );
    }

    #[test]
    fn durable_reactions_retain_diagnostics_but_exclude_invalid_members() {
        let document = TypedDocument::parse(concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"a\"/>",
            "<reaction id=\"r\"><reactant idref=\"missing\"/><arrow idref=\"a\"/><arrow idref=\"a\"/></reaction></cdml>"
        ))
        .expect("fixture parses");
        let index = DirectCdmlSemanticIndexV1::from_document(&document);
        let durable = index
            .bind_durable_reactions_v1(&document)
            .expect("durable identities bind");
        let reaction = durable
            .durable_reactions_v1()
            .first()
            .expect("display-only reaction retains its durable root");

        assert!(!reaction.is_strict());
        assert!(
            reaction
                .diagnostics()
                .contains(&ReactionDefinitionDiagnosticV1::MissingProducts)
        );
        assert!(
            reaction
                .diagnostics()
                .contains(&ReactionDefinitionDiagnosticV1::MultipleArrows)
        );
        assert_eq!(reaction.members().len(), 2);
        assert!(
            reaction
                .members()
                .iter()
                .all(|member| member.role() == DirectReactionRoleV1::Arrow)
        );
    }

    #[test]
    fn durable_reactions_exclude_foreign_and_nested_lookalikes() {
        let document = TypedDocument::parse(concat!(
            "<c:cdml xmlns:c=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\">",
            "<c:molecule id=\"m\"><c:atom id=\"atom\" name=\"C\"><c:point x=\"0\" y=\"0\"/></c:atom><c:reaction id=\"nested\"/></c:molecule>",
            "<c:arrow id=\"a\"/><c:reaction id=\"direct\"><c:reactant idref=\"m\"/><c:product idref=\"m\"/><c:arrow idref=\"a\"/></c:reaction>",
            "<v:reaction id=\"foreign\"><v:reactant idref=\"m\"/></v:reaction></c:cdml>"
        ))
        .expect("fixture parses");
        let index = DirectCdmlSemanticIndexV1::from_document(&document);
        let durable = index
            .bind_durable_reactions_v1(&document)
            .expect("durable identities bind");
        let definitions = inspect_direct_reactions_v1(
            &document
                .to_xml()
                .expect("fixture serializes for inspection"),
        )
        .expect("serialized fixture parses");

        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].identifier(), Some("direct"));
        assert_eq!(durable.durable_reactions_v1().len(), 1);
    }
}

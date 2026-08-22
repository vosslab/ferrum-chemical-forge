//! Namespace-aware semantic facts for direct CDML roots.
//!
//! Compatibility XML remains retained without interpretation. This module is
//! the shared classifier for direct-root reaction semantics.

use std::collections::{HashMap, HashSet};

use thiserror::Error;
use xot::Xot;

use super::{
    CDML_NAMESPACE, TypedDocument, TypedDocumentError, XmlSerializationError, element_name,
    ferrum_cdml_element_name, is_ferrum_cdml_name,
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
}

impl DirectCdmlSemanticIndexV1 {
    /// Parse retained CDML and classify only direct semantic roots.
    pub fn parse(source: &str) -> Result<Self, DirectCdmlSemanticErrorV1> {
        let document = TypedDocument::parse(source)?;
        Ok(Self::from_document(&document))
    }

    pub(crate) fn from_document(document: &TypedDocument) -> Self {
        let tree = &document.indexed().xml.tree;
        let root = tree
            .document_element(document.indexed().xml.document)
            .expect("a parsed CDML document has a root");
        let roots = tree
            .children(root)
            .filter_map(|node| direct_root(tree, node))
            .collect();
        let reserved_ids = document
            .indexed()
            .persistent_ids()
            .map(|identifier| identifier.as_str().to_owned())
            .collect();
        Self {
            roots,
            reserved_ids,
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
    let tree = &document.indexed().xml.tree;
    let root = tree
        .document_element(document.indexed().xml.document)
        .expect("a parsed CDML document has a root");
    let direct = tree
        .children(root)
        .enumerate()
        .filter_map(|(source_order, node)| direct_root_node(tree, node, source_order as u32))
        .collect::<Vec<_>>();
    let mut target_kinds = HashMap::<String, Vec<DirectCdmlRootKindV1>>::new();
    for item in &direct {
        if let Some(identifier) = item.identifier.as_ref() {
            target_kinds
                .entry(identifier.clone())
                .or_default()
                .push(item.kind);
        }
    }
    let mut definitions = direct
        .into_iter()
        .filter_map(|item| item.reaction)
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
                Some(kinds) if kinds.len() != 1 || !role_accepts_kind(member.role, kinds[0]) => {
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
    Ok(definitions)
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

struct DirectRootParseV1 {
    kind: DirectCdmlRootKindV1,
    identifier: Option<String>,
    reaction: Option<ReactionDefinitionV1>,
}

fn direct_root_node(tree: &Xot, node: xot::Node, source_order: u32) -> Option<DirectRootParseV1> {
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
    Some(DirectRootParseV1 {
        kind,
        identifier,
        reaction,
    })
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
pub fn append_direct_cdml_reaction_v1(
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
pub fn replace_direct_cdml_reaction_members_v1(
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
pub fn delete_direct_cdml_reaction_definition_v1(
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

fn direct_root(tree: &Xot, node: xot::Node) -> Option<DirectCdmlRootV1> {
    let (local, namespace) = element_name(tree, node)?;
    let kind = if namespace == CDML_NAMESPACE {
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
    let reaction_members = if kind == DirectCdmlRootKindV1::Reaction {
        tree.children(node)
            .filter(|child| is_reaction_role(tree, *child))
            .filter_map(|child| attribute(tree, child, "idref").map(str::to_owned))
            .collect()
    } else {
        Vec::new()
    };
    Some(DirectCdmlRootV1 {
        kind,
        identifier,
        reaction_members,
    })
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
            "<c:molecule id=\"core\"><v:reaction id=\"nested\"/></c:molecule>",
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
            "<c:molecule id=\"m\"/></c:cdml>"
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
            "<c:molecule id=\"left\"/><c:molecule id=\"right\"/><c:arrow id=\"a\"/>",
            "<c:reaction id=\"r\"><c:reactant idref=\"left\"/><c:product idref=\"right\"/><c:arrow idref=\"a\"/></c:reaction>",
            "<v:reaction id=\"foreign\"><v:reactant idref=\"left\"/></v:reaction>",
            "<c:molecule id=\"nested\"><c:reaction id=\"nested-r\"/></c:molecule></c:cdml>"
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
}

//! Content-independent typed recognition over the structurally retained CDML tree.

use std::collections::BTreeMap;

use xot::xmlname::NameStrInfo;
use xot::{Node, Value, Xot};

use super::identity_index::ProvisionalToken;
use super::typed_schema::typed_attribute_names;
use super::{
    CDML_NAMESPACE, ElementPath, IndexedDocument, IndexedDocumentError, PersistentId, Point3V1,
    TypedDiagnostic, TypedDiagnosticKind, TypedDocumentError, element_name,
};

pub use super::typed_class::TypedClass;

/// An expanded XML name independent of any source prefix spelling.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ExpandedName {
    namespace: String,
    local_name: String,
}

impl ExpandedName {
    /// Return the namespace URI, or an empty string for no namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Return the local part of the XML name.
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }
}

/// One prefix-to-namespace binding visible on an element.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct NamespaceBinding {
    prefix: String,
    namespace: String,
}

impl NamespaceBinding {
    /// Return the prefix, or an empty string for the default namespace.
    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Return the namespace URI.
    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }
}

/// An attribute outside the named schema for its recognized record class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownAttribute {
    qualified_name: String,
    expanded_name: ExpandedName,
    value: String,
    namespace_context: Vec<NamespaceBinding>,
}

impl UnknownAttribute {
    /// Return the QName selected by the retained namespace context.
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        &self.qualified_name
    }

    /// Return the namespace URI and local name.
    #[must_use]
    pub fn expanded_name(&self) -> &ExpandedName {
        &self.expanded_name
    }

    /// Return the uninterpreted source value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Return every prefix binding in scope where the attribute occurred.
    #[must_use]
    pub fn namespace_context(&self) -> &[NamespaceBinding] {
        &self.namespace_context
    }
}

/// A direct text segment carried as typed content by a text-bearing class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedText {
    position: u32,
    value: String,
}

impl TypedText {
    /// Return the position in the complete direct-child sequence.
    #[must_use]
    pub fn position(&self) -> u32 {
        self.position
    }

    /// Return the exact parsed character data.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// One recognized child and its position among all direct child nodes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedChild {
    position: u32,
    record: TypedRecord,
}

impl TypedChild {
    /// Return the position in the complete direct-child sequence.
    #[must_use]
    pub fn position(&self) -> u32 {
        self.position
    }

    /// Return the child's typed payload.
    #[must_use]
    pub fn record(&self) -> &TypedRecord {
        &self.record
    }
}

/// Structurally retained content that the parent context does not type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnrecognizedNode {
    /// An entire element subtree retained as opaque XML.
    Element {
        /// Expanded element name.
        name: ExpandedName,
        /// Self-contained structural serialization of the subtree.
        xml: String,
    },
    /// Parsed character data.
    Text(String),
    /// XML comment text.
    Comment(String),
    /// Processing instruction target and optional data.
    ProcessingInstruction {
        /// Expanded target name.
        target: ExpandedName,
        /// Instruction data.
        data: Option<String>,
    },
}

/// One opaque direct child and its place in mixed content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnrecognizedChild {
    position: u32,
    node: UnrecognizedNode,
}

impl UnrecognizedChild {
    /// Return the position in the complete direct-child sequence.
    #[must_use]
    pub fn position(&self) -> u32 {
        self.position
    }

    /// Return the retained node payload.
    #[must_use]
    pub fn node(&self) -> &UnrecognizedNode {
        &self.node
    }
}

/// One content-independent typed CDML element projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedRecord {
    class: TypedClass,
    path: ElementPath,
    typed_attributes: BTreeMap<String, String>,
    unknown_attributes: Vec<UnknownAttribute>,
    typed_children: Vec<TypedChild>,
    typed_text: Vec<TypedText>,
    unrecognized_children: Vec<UnrecognizedChild>,
    diagnostics: Vec<TypedDiagnostic>,
}

impl TypedRecord {
    /// Return the context-qualified record class.
    #[must_use]
    pub fn class(&self) -> TypedClass {
        self.class
    }

    /// Return the stable element path.
    #[must_use]
    pub fn path(&self) -> &ElementPath {
        &self.path
    }

    /// Return one typed attribute's exact lexical value.
    #[must_use]
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.typed_attributes.get(name).map(String::as_str)
    }

    /// Return every present typed field in deterministic name order.
    #[must_use]
    pub fn typed_attributes(&self) -> &BTreeMap<String, String> {
        &self.typed_attributes
    }

    /// Return attributes not assigned as named fields for this class.
    #[must_use]
    pub fn unknown_attributes(&self) -> &[UnknownAttribute] {
        &self.unknown_attributes
    }

    /// Return recognized direct children in complete source order.
    #[must_use]
    pub fn typed_children(&self) -> &[TypedChild] {
        &self.typed_children
    }

    /// Iterate recognized direct children of one class.
    pub fn children_of(&self, class: TypedClass) -> impl Iterator<Item = &TypedRecord> {
        self.typed_children
            .iter()
            .filter(move |child| child.record.class == class)
            .map(|child| &child.record)
    }

    /// Return typed direct character-data segments in mixed-content order.
    #[must_use]
    pub fn typed_text(&self) -> &[TypedText] {
        &self.typed_text
    }

    /// Concatenate typed direct character data without interpreting markup.
    #[must_use]
    pub fn text_content(&self) -> String {
        self.typed_text
            .iter()
            .map(|text| text.value.as_str())
            .collect()
    }

    /// Return every direct node that this class did not recognize.
    #[must_use]
    pub fn unrecognized_children(&self) -> &[UnrecognizedChild] {
        &self.unrecognized_children
    }

    /// Return non-demoting cardinality diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[TypedDiagnostic] {
        &self.diagnostics
    }
}

/// A single retained CDML tree plus its immutable typed record overlay.
#[derive(Debug)]
pub struct TypedDocument {
    indexed: IndexedDocument,
    root: TypedRecord,
}

impl TypedDocument {
    /// Parse CDML once, establish persistent identity, then project recognized classes.
    pub fn parse(source: &str) -> Result<Self, TypedDocumentError> {
        let indexed = IndexedDocument::parse(source)?;
        Self::from_indexed(indexed)
    }

    /// Admit CDML under an explicit caller-owned XML budget, then type one retained tree.
    ///
    /// This preserves the existing DTD and entity protections while rejecting an
    /// over-budget source before `xot` retains it. The caller selects the budget;
    /// this document layer deliberately provides no default policy.
    pub fn parse_with_budget(
        source: &str,
        budget: super::XmlInputBudgetV1,
    ) -> Result<Self, TypedDocumentError> {
        let xml = super::XmlDocument::parse_with_budget(source, budget)?;
        let indexed = IndexedDocument::from_xml(xml).map_err(IndexedDocumentError::from)?;
        Self::from_indexed(indexed)
    }

    pub(crate) fn from_indexed(mut indexed: IndexedDocument) -> Result<Self, TypedDocumentError> {
        let tree = &indexed.xml.tree;
        let root_node = tree
            .document_element(indexed.xml.document)
            .expect("an indexed XML document has a document element");
        let root = project_record(tree, root_node, TypedClass::Cdml, Vec::new())?;
        validate_canonical_values(&root)?;
        canonicalize_presentation_face_aliases(&mut indexed);
        let tree = &indexed.xml.tree;
        let root_node = tree
            .document_element(indexed.xml.document)
            .expect("an indexed XML document has a document element");
        let root = project_record(tree, root_node, TypedClass::Cdml, Vec::new())?;
        Ok(Self { indexed, root })
    }

    /// Return the typed root payload.
    #[must_use]
    pub fn root(&self) -> &TypedRecord {
        &self.root
    }

    /// Return the identity and ordering index over the same retained tree.
    #[must_use]
    pub fn indexed(&self) -> &IndexedDocument {
        &self.indexed
    }

    /// Serialize the single retained tree under the structural-fidelity contract.
    pub fn to_xml(&self) -> Result<String, super::XmlSerializationError> {
        self.indexed.xml.to_xml()
    }

    pub(crate) fn detached_candidate(&self) -> Result<Self, TypedDocumentError> {
        Self::parse(&self.to_xml()?)
    }

    pub(crate) fn detached_indexed_mut(&mut self) -> &mut IndexedDocument {
        &mut self.indexed
    }

    /// Return a detached document with one typed atom's element spelling replaced.
    ///
    /// This deliberately narrow primitive is owned by the typed CDML layer rather
    /// than accepting caller-authored XML. It re-parses after mutation so the
    /// identity index and typed overlay always describe the resulting retained tree.
    pub(crate) fn with_atom_element(
        &self,
        identifier: &PersistentId,
        element: &str,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let source = self.to_xml()?;
        let mut candidate = Self::parse(&source)?;
        let id_name = candidate.indexed.xml.tree.add_name("id");
        let element_name = candidate.indexed.xml.tree.add_name("name");
        let root = candidate
            .indexed
            .xml
            .tree
            .document_element(candidate.indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let target = candidate.indexed.xml.tree.descendants(root).find(|node| {
            let Some((local_name, namespace)) =
                super::element_name(&candidate.indexed.xml.tree, *node)
            else {
                return false;
            };
            local_name == "atom"
                && (namespace == CDML_NAMESPACE)
                && candidate.indexed.xml.tree.get_attribute(*node, id_name)
                    == Some(identifier.as_str())
        });
        let Some(target) = target else {
            return Ok(None);
        };
        if candidate
            .indexed
            .xml
            .tree
            .get_attribute(target, element_name)
            == Some(element)
        {
            return Ok(Some(candidate));
        }
        candidate
            .indexed
            .xml
            .tree
            .set_attribute(target, element_name, element);
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }

    /// Build a detached, fully indexed candidate containing one new typed atom.
    ///
    /// The caller supplies validated persistent identities rather than XML. The
    /// candidate is reparsed after mutation, preserving the single-tree contract and
    /// rejecting a duplicate identity before any session token is issued.
    pub(crate) fn with_insert_atom(
        &self,
        molecule_id: &PersistentId,
        atom_id: &PersistentId,
        element: &str,
        position: Point3V1,
    ) -> Result<Self, TypedDocumentError> {
        if element.trim().is_empty()
            || element
                .chars()
                .any(|character| !character.is_ascii_alphabetic())
        {
            return Err(TypedDocumentError::InvalidAtomElement);
        }
        if self.indexed.resolve_id(atom_id).is_some() {
            return Err(TypedDocumentError::DuplicateAtomId(atom_id.clone()));
        }
        let source = self.to_xml()?;
        let mut candidate = Self::parse(&source)?;
        let id_name = candidate.indexed.xml.tree.add_name("id");
        let element_name = candidate.indexed.xml.tree.add_name("name");
        let root = candidate
            .indexed
            .xml
            .tree
            .document_element(candidate.indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let molecule = candidate
            .indexed
            .xml
            .tree
            .descendants(root)
            .find_map(|node| {
                let (local_name, namespace) =
                    super::element_name(&candidate.indexed.xml.tree, node)?;
                (local_name == "molecule"
                    && (namespace == CDML_NAMESPACE)
                    && candidate.indexed.xml.tree.get_attribute(node, id_name)
                        == Some(molecule_id.as_str()))
                .then_some((node, namespace))
            });
        let Some((molecule, molecule_namespace)) = molecule else {
            return Err(TypedDocumentError::UnknownMolecule(molecule_id.clone()));
        };
        let atom_name = if molecule_namespace.is_empty() {
            candidate.indexed.xml.tree.add_name("atom")
        } else {
            let namespace = candidate
                .indexed
                .xml
                .tree
                .add_namespace(&molecule_namespace);
            candidate.indexed.xml.tree.add_name_ns("atom", namespace)
        };
        let atom = candidate.indexed.xml.tree.new_element(atom_name);
        candidate
            .indexed
            .xml
            .tree
            .set_attribute(atom, id_name, atom_id.as_str());
        candidate
            .indexed
            .xml
            .tree
            .set_attribute(atom, element_name, element);
        let point_name = if molecule_namespace.is_empty() {
            candidate.indexed.xml.tree.add_name("point")
        } else {
            let namespace = candidate
                .indexed
                .xml
                .tree
                .add_namespace(&molecule_namespace);
            candidate.indexed.xml.tree.add_name_ns("point", namespace)
        };
        let point = candidate.indexed.xml.tree.new_element(point_name);
        let x_name = candidate.indexed.xml.tree.add_name("x");
        let y_name = candidate.indexed.xml.tree.add_name("y");
        let z_name = candidate.indexed.xml.tree.add_name("z");
        candidate
            .indexed
            .xml
            .tree
            .set_attribute(point, x_name, position.x().to_string());
        candidate
            .indexed
            .xml
            .tree
            .set_attribute(point, y_name, position.y().to_string());
        candidate
            .indexed
            .xml
            .tree
            .set_attribute(point, z_name, position.z().to_string());
        candidate
            .indexed
            .xml
            .tree
            .append(atom, point)
            .map_err(TypedDocumentError::Mutation)?;
        candidate
            .indexed
            .xml
            .tree
            .append(molecule, atom)
            .map_err(TypedDocumentError::Mutation)?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized)
    }

    pub(crate) fn try_issue_provisional_token(
        &mut self,
    ) -> Result<ProvisionalToken, TypedDocumentError> {
        self.indexed
            .try_issue_provisional_token()
            .map_err(IndexedDocumentError::from)
            .map_err(TypedDocumentError::from)
    }

    pub(crate) fn consume_provisional_token(
        &mut self,
        token: &ProvisionalToken,
    ) -> Result<(), TypedDocumentError> {
        self.indexed
            .consume_provisional_token(token)
            .map_err(IndexedDocumentError::from)
            .map_err(TypedDocumentError::from)
    }

    pub(crate) fn verify_provisional_token(
        &self,
        token: &ProvisionalToken,
    ) -> Result<(), TypedDocumentError> {
        self.indexed
            .verify_provisional_token(token)
            .map_err(IndexedDocumentError::from)
            .map_err(TypedDocumentError::from)
    }
}

/// Serialize every admitted presentation-face alias as the one canonical CDML spelling.
fn canonicalize_presentation_face_aliases(indexed: &mut IndexedDocument) {
    let tree = &mut indexed.xml.tree;
    let root = tree
        .document_element(indexed.xml.document)
        .expect("an indexed XML document has a document element");
    let family_name = tree.add_name("family");
    let roots = tree.children(root).collect::<Vec<_>>();
    for presentation in roots {
        let is_presentation = element_name(tree, presentation).is_some_and(|(local, namespace)| {
            namespace == CDML_NAMESPACE && matches!(local.as_str(), "text" | "plus")
        });
        if !is_presentation {
            continue;
        }
        let fonts = tree.children(presentation).collect::<Vec<_>>();
        for font in fonts {
            let is_font = element_name(tree, font)
                .is_some_and(|(local, namespace)| namespace == CDML_NAMESPACE && local == "font");
            if !is_font {
                continue;
            }
            let Some(family) = tree.get_attribute(font, family_name) else {
                continue;
            };
            if let Some(face) = super::PresentationFontFaceV1::from_cdml_family(family) {
                tree.set_attribute(font, family_name, face.cdml_family());
            }
        }
    }
}

/// Enforce facts whose meaning is shared by every typed-document consumer.
///
/// This runs after structural projection so raw XML indexing remains lossless, while
/// every typed document, detached candidate, and reconstructed revision shares the
/// same semantic admission boundary.
fn validate_canonical_values(root: &TypedRecord) -> Result<(), TypedDocumentError> {
    validate_record_values(root)?;
    validate_presentation_faces(root)?;
    for child in root.children_of(TypedClass::Molecule) {
        let molecule = child;
        if molecule.typed_children().iter().any(|child| {
            matches!(
                child.record().class(),
                TypedClass::Atom
                    | TypedClass::CompactGroup
                    | TypedClass::MoleculeText
                    | TypedClass::Query
            )
        }) {
            continue;
        }
        return Err(TypedDocumentError::EmptyDirectMolecule {
            molecule_id: typed_record_id(molecule),
        });
    }
    Ok(())
}

/// Admit presentation faces before a document, session, or candidate can exist.
fn validate_presentation_faces(root: &TypedRecord) -> Result<(), TypedDocumentError> {
    for root in root.typed_children() {
        if !matches!(
            root.record().class(),
            TypedClass::CanvasText | TypedClass::CanvasPlus
        ) {
            continue;
        }
        for font in root.record().children_of(TypedClass::Font) {
            let Some(family) = font.attribute("family") else {
                continue;
            };
            if super::PresentationFontFaceV1::from_cdml_family(family).is_none() {
                return Err(TypedDocumentError::UnsupportedTextFace {
                    root_id: typed_record_id(root.record()),
                    family: family.to_owned(),
                });
            }
        }
    }
    Ok(())
}

fn validate_record_values(record: &TypedRecord) -> Result<(), TypedDocumentError> {
    if record.class() == TypedClass::Atom {
        validate_atom_multiplicity(record)?;
    }
    for child in record.typed_children() {
        validate_record_values(child.record())?;
    }
    Ok(())
}

fn validate_atom_multiplicity(record: &TypedRecord) -> Result<(), TypedDocumentError> {
    let Some(value) = record.attribute("multiplicity") else {
        return Ok(());
    };
    let is_positive_u16 = value.bytes().all(|byte| byte.is_ascii_digit())
        && value.parse::<u16>().is_ok_and(|parsed| parsed > 0);
    if is_positive_u16 {
        return Ok(());
    }
    Err(TypedDocumentError::InvalidAtomMultiplicity {
        atom_id: typed_record_id(record),
        value: value.to_owned(),
    })
}

fn typed_record_id(record: &TypedRecord) -> String {
    record
        .attribute("id")
        .unwrap_or("<unidentified>")
        .to_owned()
}

fn project_record(
    tree: &Xot,
    node: Node,
    class: TypedClass,
    path: Vec<u32>,
) -> Result<TypedRecord, TypedDocumentError> {
    let (typed_attributes, unknown_attributes) = project_attributes(tree, node, class)?;
    let mut typed_children = Vec::new();
    let mut typed_text = Vec::new();
    let mut unrecognized_children = Vec::new();
    let mut diagnostics = Vec::new();
    let mut counts = BTreeMap::<TypedClass, u32>::new();
    let mut element_index = 0_u32;

    for (position, child) in tree.children(node).enumerate() {
        let position = u32::try_from(position).expect("XML child count fits u32");
        if let Some((local_name, namespace)) = super::element_name(tree, child) {
            let mut child_path = path.clone();
            child_path.push(element_index);
            element_index += 1;
            let candidate = if namespace == CDML_NAMESPACE {
                child_class(class, &local_name)
            } else {
                None
            };
            if let Some(child_class) = candidate {
                let count = counts.entry(child_class).or_default();
                let (_, maximum) = child_cardinality(class, child_class);
                if maximum.is_some_and(|maximum| *count >= maximum) {
                    diagnostics.push(TypedDiagnostic {
                        kind: TypedDiagnosticKind::ExcessChild,
                        child_class,
                        message: format!(
                            "{} has more than {maximum:?} {} child",
                            class.name(),
                            child_class.name()
                        ),
                    });
                    unrecognized_children.push(unrecognized_child(tree, child, position)?);
                } else {
                    let record = project_record(tree, child, child_class, child_path)?;
                    typed_children.push(TypedChild { position, record });
                    *count += 1;
                }
            } else {
                unrecognized_children.push(unrecognized_child(tree, child, position)?);
            }
            continue;
        }

        if let Some(text) = tree.text_str(child) {
            if class_carries_text(class) {
                typed_text.push(TypedText {
                    position,
                    value: text.to_owned(),
                });
            } else {
                unrecognized_children.push(UnrecognizedChild {
                    position,
                    node: UnrecognizedNode::Text(text.to_owned()),
                });
            }
            continue;
        }
        unrecognized_children.push(unrecognized_child(tree, child, position)?);
    }

    for child_class in permitted_children(class) {
        let (minimum, _) = child_cardinality(class, *child_class);
        let actual = counts.get(child_class).copied().unwrap_or(0);
        if actual < minimum {
            diagnostics.push(TypedDiagnostic {
                kind: TypedDiagnosticKind::MissingChild,
                child_class: *child_class,
                message: format!(
                    "{} requires at least {minimum} {} child",
                    class.name(),
                    child_class.name()
                ),
            });
        }
    }

    validate_compact_group_content(class, &typed_children, &unrecognized_children)?;

    Ok(TypedRecord {
        class,
        path: ElementPath(path),
        typed_attributes,
        unknown_attributes,
        typed_children,
        typed_text,
        unrecognized_children,
        diagnostics,
    })
}

fn validate_compact_group_content(
    class: TypedClass,
    typed_children: &[TypedChild],
    unrecognized_children: &[UnrecognizedChild],
) -> Result<(), TypedDocumentError> {
    if class != TypedClass::CompactGroup {
        return Ok(());
    }
    let has_exactly_one_anchor = matches!(typed_children, [child]
        if child.record().class() == TypedClass::Point);
    let has_only_whitespace = unrecognized_children.iter().all(
        |child| matches!(child.node(), UnrecognizedNode::Text(text) if text.trim().is_empty()),
    );
    if has_exactly_one_anchor && has_only_whitespace {
        return Ok(());
    }
    Err(TypedDocumentError::UndeclaredCompactGroupContent)
}

fn project_attributes(
    tree: &Xot,
    node: Node,
    class: TypedClass,
) -> Result<(BTreeMap<String, String>, Vec<UnknownAttribute>), TypedDocumentError> {
    let mut typed = BTreeMap::new();
    let mut unknown = Vec::new();
    let context = namespace_context(tree, node);
    for (name, value) in tree.attributes(node).iter() {
        let (local_name, namespace) = tree.name_ns_str(name);
        if namespace.is_empty() && typed_attribute_names(class).contains(&local_name) {
            typed.insert(local_name.to_owned(), value.clone());
            continue;
        }
        let reference = tree
            .name_ref(name, node)
            .map_err(TypedDocumentError::AttributeName)?;
        if class == TypedClass::CompactGroup {
            return Err(TypedDocumentError::UndeclaredCompactGroupAttribute {
                attribute: reference.full_name().into_owned(),
            });
        }
        unknown.push(UnknownAttribute {
            qualified_name: reference.full_name().into_owned(),
            expanded_name: ExpandedName {
                namespace: namespace.to_owned(),
                local_name: local_name.to_owned(),
            },
            value: value.clone(),
            namespace_context: context.clone(),
        });
    }
    unknown.sort_by(|left, right| {
        (
            &left.expanded_name.namespace,
            &left.expanded_name.local_name,
            &left.qualified_name,
        )
            .cmp(&(
                &right.expanded_name.namespace,
                &right.expanded_name.local_name,
                &right.qualified_name,
            ))
    });
    Ok((typed, unknown))
}

fn namespace_context(tree: &Xot, node: Node) -> Vec<NamespaceBinding> {
    let mut context = tree
        .namespaces_in_scope(node)
        .map(|(prefix, namespace)| NamespaceBinding {
            prefix: tree.prefix_str(prefix).to_owned(),
            namespace: tree.namespace_str(namespace).to_owned(),
        })
        .collect::<Vec<_>>();
    context.sort();
    context
}

fn unrecognized_child(
    tree: &Xot,
    node: Node,
    position: u32,
) -> Result<UnrecognizedChild, TypedDocumentError> {
    let retained = match tree.value(node) {
        Value::Element(element) => {
            let (local_name, namespace) = tree.name_ns_str(element.name());
            UnrecognizedNode::Element {
                name: ExpandedName {
                    namespace: namespace.to_owned(),
                    local_name: local_name.to_owned(),
                },
                xml: tree
                    .to_string(node)
                    .map_err(TypedDocumentError::OpaqueSnapshot)?,
            }
        }
        Value::Text(text) => UnrecognizedNode::Text(text.get().to_owned()),
        Value::Comment(comment) => UnrecognizedNode::Comment(comment.get().to_owned()),
        Value::ProcessingInstruction(instruction) => {
            let (local_name, namespace) = tree.name_ns_str(instruction.target());
            UnrecognizedNode::ProcessingInstruction {
                target: ExpandedName {
                    namespace: namespace.to_owned(),
                    local_name: local_name.to_owned(),
                },
                data: instruction.data().map(str::to_owned),
            }
        }
        Value::Document | Value::Attribute(_) | Value::Namespace(_) => {
            unreachable!("only normal XML children are projected")
        }
    };
    Ok(UnrecognizedChild {
        position,
        node: retained,
    })
}

fn class_carries_text(class: TypedClass) -> bool {
    matches!(
        class,
        TypedClass::AuthorProgram
            | TypedClass::Author
            | TypedClass::Note
            | TypedClass::FragmentName
            | TypedClass::FormattedText
    )
}

fn child_class(parent: TypedClass, local_name: &str) -> Option<TypedClass> {
    permitted_children(parent)
        .iter()
        .copied()
        .find(|class| child_local_name(*class) == local_name)
}

fn child_local_name(class: TypedClass) -> &'static str {
    match class {
        TypedClass::Cdml => "cdml",
        TypedClass::Info => "info",
        TypedClass::AuthorProgram => "author_program",
        TypedClass::Author => "author",
        TypedClass::Note => "note",
        TypedClass::Metadata => "metadata",
        TypedClass::MetadataDocument => "doc",
        TypedClass::Standard => "standard",
        TypedClass::StandardBond | TypedClass::Bond | TypedClass::FragmentBond => "bond",
        TypedClass::StandardArrow | TypedClass::CanvasArrow | TypedClass::ReactionArrow => "arrow",
        TypedClass::StandardAtom | TypedClass::Atom => "atom",
        TypedClass::CompactGroup => "compact-group",
        TypedClass::Paper => "paper",
        TypedClass::Viewport => "viewport",
        TypedClass::Molecule => "molecule",
        TypedClass::CanvasPlus | TypedClass::ReactionPlus => "plus",
        TypedClass::CanvasText | TypedClass::MoleculeText => "text",
        TypedClass::Rectangle => "rect",
        TypedClass::Square => "square",
        TypedClass::Oval => "oval",
        TypedClass::Circle => "circle",
        TypedClass::Polygon => "polygon",
        TypedClass::Polyline => "polyline",
        TypedClass::Reaction => "reaction",
        TypedClass::ReactionReactant => "reactant",
        TypedClass::ReactionProduct => "product",
        TypedClass::ReactionCondition => "condition",
        TypedClass::ExternalData => "external-data",
        TypedClass::Group => "group",
        TypedClass::Query => "query",
        TypedClass::Template => "template",
        TypedClass::Fragment => "fragment",
        TypedClass::DisplayForm => "display-form",
        TypedClass::UserData => "user-data",
        TypedClass::FragmentName => "name",
        TypedClass::FragmentVertex => "vertex",
        TypedClass::FragmentProperty => "property",
        TypedClass::Point => "point",
        TypedClass::Font => "font",
        TypedClass::FormattedText => "ftext",
        TypedClass::Mark => "mark",
    }
}

fn permitted_children(class: TypedClass) -> &'static [TypedClass] {
    use TypedClass as C;
    match class {
        C::Cdml => &[
            C::Info,
            C::Metadata,
            C::Standard,
            C::Paper,
            C::Viewport,
            C::Molecule,
            C::CanvasArrow,
            C::CanvasPlus,
            C::CanvasText,
            C::Rectangle,
            C::Square,
            C::Oval,
            C::Circle,
            C::Polygon,
            C::Polyline,
            C::Reaction,
            C::ExternalData,
        ],
        C::Info => &[C::AuthorProgram, C::Author, C::Note],
        C::Metadata => &[C::MetadataDocument],
        C::Standard => &[C::StandardBond, C::StandardArrow, C::StandardAtom],
        C::Molecule => &[
            C::Template,
            C::Atom,
            C::CompactGroup,
            C::Group,
            C::MoleculeText,
            C::Query,
            C::Bond,
            C::Fragment,
            C::DisplayForm,
            C::UserData,
        ],
        C::CanvasArrow | C::Polygon | C::Polyline => &[C::Point],
        C::CanvasPlus => &[C::Point, C::Font],
        C::CanvasText | C::MoleculeText => &[C::Font, C::Point, C::FormattedText],
        C::Reaction => &[
            C::ReactionReactant,
            C::ReactionProduct,
            C::ReactionArrow,
            C::ReactionCondition,
            C::ReactionPlus,
        ],
        C::Atom => &[C::Point, C::Font, C::FormattedText, C::Mark],
        C::CompactGroup | C::Group | C::Query => &[C::Point],
        C::Fragment => &[
            C::FragmentName,
            C::FragmentBond,
            C::FragmentVertex,
            C::FragmentProperty,
        ],
        _ => &[],
    }
}

fn child_cardinality(parent: TypedClass, child: TypedClass) -> (u32, Option<u32>) {
    use TypedClass as C;
    match (parent, child) {
        (C::CanvasArrow, C::Point) | (C::Polyline, C::Point) => (2, None),
        (C::Polygon, C::Point) => (3, None),
        (C::Atom | C::CompactGroup | C::Group | C::MoleculeText | C::Query, C::Point)
        | (C::CanvasPlus | C::CanvasText, C::Point) => (1, Some(1)),
        (C::Atom | C::MoleculeText | C::CanvasPlus | C::CanvasText, C::Font)
        | (C::Atom | C::MoleculeText | C::CanvasText, C::FormattedText)
        | (C::Fragment, C::FragmentName)
        | (C::Metadata, C::MetadataDocument)
        | (C::Standard, C::StandardBond | C::StandardArrow | C::StandardAtom)
        | (C::Info, C::AuthorProgram | C::Author | C::Note) => (0, Some(1)),
        _ => (0, None),
    }
}

//! Content-independent typed recognition over the structurally retained CDML tree.

use std::collections::BTreeMap;

use thiserror::Error;
use xot::xmlname::NameStrInfo;
use xot::{Node, Value, Xot};

use super::{CDML_NAMESPACE, ElementPath, IndexedDocument, IndexedDocumentError};

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

/// Every CDML record class assigned a typed representation by the context table.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TypedClass {
    /// The document root.
    Cdml,
    /// Root information container.
    Info,
    /// Authoring program information.
    AuthorProgram,
    /// Author text.
    Author,
    /// Document note text.
    Note,
    /// Metadata container.
    Metadata,
    /// Metadata discovery document.
    MetadataDocument,
    /// Drawing defaults container.
    Standard,
    /// Default bond settings.
    StandardBond,
    /// Default arrow settings.
    StandardArrow,
    /// Default atom settings.
    StandardAtom,
    /// Paper settings.
    Paper,
    /// Canvas viewport.
    Viewport,
    /// Chemical molecule.
    Molecule,
    /// Direct-root arrow.
    CanvasArrow,
    /// Direct-root plus sign.
    CanvasPlus,
    /// Direct-root text label.
    CanvasText,
    /// Rectangle vector graphic.
    Rectangle,
    /// Square vector graphic.
    Square,
    /// Oval vector graphic.
    Oval,
    /// Circle vector graphic.
    Circle,
    /// Polygon vector graphic.
    Polygon,
    /// Polyline vector graphic, including persisted bracket artwork.
    Polyline,
    /// Reaction container.
    Reaction,
    /// Reaction reactant role.
    ReactionReactant,
    /// Reaction product role.
    ReactionProduct,
    /// Reaction arrow role.
    ReactionArrow,
    /// Reaction condition role.
    ReactionCondition,
    /// Reaction plus role.
    ReactionPlus,
    /// Root-level opaque external payload container.
    ExternalData,
    /// Molecule-local atom.
    Atom,
    /// Molecule-local group vertex.
    Group,
    /// Molecule-local text vertex.
    MoleculeText,
    /// Molecule-local query vertex.
    Query,
    /// Molecule-local bond.
    Bond,
    /// Molecule attachment template.
    Template,
    /// Molecule fragment.
    Fragment,
    /// Molecule-local opaque display payload container.
    DisplayForm,
    /// Molecule-local opaque user payload container.
    UserData,
    /// Fragment name text.
    FragmentName,
    /// Fragment bond reference.
    FragmentBond,
    /// Fragment vertex reference.
    FragmentVertex,
    /// Fragment property.
    FragmentProperty,
    /// Shared point record.
    Point,
    /// Shared font record.
    Font,
    /// Shared formatted-text payload.
    FormattedText,
    /// Atom mark.
    Mark,
}

impl TypedClass {
    /// Return the context-qualified stable class name used in diagnostics.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Cdml => "cdml",
            Self::Info => "cdml/info",
            Self::AuthorProgram => "info/author_program",
            Self::Author => "info/author",
            Self::Note => "info/note",
            Self::Metadata => "cdml/metadata",
            Self::MetadataDocument => "metadata/doc",
            Self::Standard => "cdml/standard",
            Self::StandardBond => "standard/bond",
            Self::StandardArrow => "standard/arrow",
            Self::StandardAtom => "standard/atom",
            Self::Paper => "cdml/paper",
            Self::Viewport => "cdml/viewport",
            Self::Molecule => "cdml/molecule",
            Self::CanvasArrow => "cdml/arrow",
            Self::CanvasPlus => "cdml/plus",
            Self::CanvasText => "cdml/text",
            Self::Rectangle => "cdml/rect",
            Self::Square => "cdml/square",
            Self::Oval => "cdml/oval",
            Self::Circle => "cdml/circle",
            Self::Polygon => "cdml/polygon",
            Self::Polyline => "cdml/polyline",
            Self::Reaction => "cdml/reaction",
            Self::ReactionReactant => "reaction/reactant",
            Self::ReactionProduct => "reaction/product",
            Self::ReactionArrow => "reaction/arrow",
            Self::ReactionCondition => "reaction/condition",
            Self::ReactionPlus => "reaction/plus",
            Self::ExternalData => "cdml/external-data",
            Self::Atom => "molecule/atom",
            Self::Group => "molecule/group",
            Self::MoleculeText => "molecule/text",
            Self::Query => "molecule/query",
            Self::Bond => "molecule/bond",
            Self::Template => "molecule/template",
            Self::Fragment => "molecule/fragment",
            Self::DisplayForm => "molecule/display-form",
            Self::UserData => "molecule/user-data",
            Self::FragmentName => "fragment/name",
            Self::FragmentBond => "fragment/bond",
            Self::FragmentVertex => "fragment/vertex",
            Self::FragmentProperty => "fragment/property",
            Self::Point => "shared/point",
            Self::Font => "shared/font",
            Self::FormattedText => "shared/ftext",
            Self::Mark => "atom/mark",
        }
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

/// A non-demoting structural problem found on a recognized record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypedDiagnosticKind {
    /// A required child slot was absent.
    MissingChild,
    /// A child exceeded the class's maximum cardinality and stayed opaque.
    ExcessChild,
}

/// One diagnostic attached to a typed record without changing its class.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedDiagnostic {
    kind: TypedDiagnosticKind,
    child_class: TypedClass,
    message: String,
}

impl TypedDiagnostic {
    /// Return the stable problem category.
    #[must_use]
    pub fn kind(&self) -> TypedDiagnosticKind {
        self.kind
    }

    /// Return the child slot involved.
    #[must_use]
    pub fn child_class(&self) -> TypedClass {
        self.child_class
    }

    /// Return the human-readable diagnostic.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
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

/// Parse or typed-projection failure.
#[derive(Debug, Error)]
pub enum TypedDocumentError {
    /// XML parsing or document identity validation failed.
    #[error(transparent)]
    Indexed(#[from] IndexedDocumentError),
    /// An opaque subtree could not be snapshotted structurally.
    #[error("cannot retain an opaque CDML subtree: {0}")]
    OpaqueSnapshot(#[source] xot::Error),
    /// A namespaced unknown attribute had no usable in-scope prefix.
    #[error("cannot retain an unknown CDML attribute name: {0}")]
    AttributeName(#[source] xot::Error),
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
            let candidate = if namespace.is_empty() || namespace == CDML_NAMESPACE {
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
        C::Group | C::Query => &[C::Point],
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
        (C::Atom | C::Group | C::MoleculeText | C::Query, C::Point)
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

fn typed_attribute_names(class: TypedClass) -> &'static [&'static str] {
    use TypedClass as C;
    match class {
        C::Cdml => &["version", "type"],
        C::AuthorProgram => &["version"],
        C::MetadataDocument => &["href"],
        C::Standard => &[
            "line_width",
            "font_size",
            "font_family",
            "line_color",
            "area_color",
            "paper_type",
            "paper_orientation",
            "paper_crop_svg",
            "paper_crop_margin",
        ],
        C::StandardBond => &[
            "length",
            "width",
            "wedge-width",
            "double-ratio",
            "min_wedge_angle",
        ],
        C::StandardArrow => &["length"],
        C::StandardAtom => &["show_hydrogens"],
        C::Paper => &[
            "id",
            "type",
            "orientation",
            "crop_svg",
            "crop_margin",
            "use_real_minus",
            "replace_minus",
            "size_x",
            "size_y",
        ],
        C::Viewport => &["viewport", "id"],
        C::Molecule => &["id", "name"],
        C::CanvasArrow => &[
            "id", "type", "start", "end", "width", "spline", "shape", "color",
        ],
        C::CanvasPlus => &["id", "font_size", "color", "background-color"],
        C::CanvasText => &["id", "background-color"],
        C::Rectangle | C::Square | C::Oval | C::Circle => &[
            "id",
            "x1",
            "y1",
            "x2",
            "y2",
            "area_color",
            "line_color",
            "width",
        ],
        C::Polygon => &["id", "area_color", "line_color", "width"],
        C::Polyline => &["id", "line_color", "width", "spline"],
        C::Reaction => &["id"],
        C::ReactionReactant
        | C::ReactionProduct
        | C::ReactionArrow
        | C::ReactionCondition
        | C::ReactionPlus => &["idref"],
        C::Atom => &[
            "id",
            "name",
            "charge",
            "pos",
            "show",
            "hydrogens",
            "show_number",
            "number",
            "background-color",
            "multiplicity",
            "valency",
            "free_sites",
            "isotope",
            "explicit_hydrogens",
        ],
        C::Group => &[
            "id",
            "name",
            "group-type",
            "pos",
            "background-color",
            "show_number",
            "number",
        ],
        C::MoleculeText => &["id", "pos", "background-color", "show_number", "number"],
        C::Query => &[
            "id",
            "name",
            "pos",
            "background-color",
            "show_number",
            "number",
            "free_sites",
        ],
        C::Bond => &[
            "id",
            "type",
            "start",
            "end",
            "line_width",
            "bond_width",
            "wedge_width",
            "double_ratio",
            "center",
            "auto_sign",
            "equithick",
            "simple_double",
            "color",
            "wavy_style",
            "haworth_position",
        ],
        C::Template => &["atom", "bond_first", "bond_second"],
        C::Fragment => &["id", "type"],
        C::FragmentBond | C::FragmentVertex => &["id"],
        C::FragmentProperty => &["name", "value", "type"],
        C::Point => &["x", "y", "z"],
        C::Font => &["size", "family", "color"],
        C::Mark => &[
            "type",
            "x",
            "y",
            "auto",
            "size",
            "line_width",
            "draw_circle",
            "text",
            "refname",
        ],
        C::Info
        | C::Author
        | C::Note
        | C::Metadata
        | C::ExternalData
        | C::DisplayForm
        | C::UserData
        | C::FragmentName
        | C::FormattedText => &[],
    }
}

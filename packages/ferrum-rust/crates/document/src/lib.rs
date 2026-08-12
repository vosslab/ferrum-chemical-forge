//! CDML document storage and structural preservation services.
//!
//! One `xot` tree owns structural preservation. A document index records persistent
//! identity and direct-child order over that tree; a content-independent typed overlay
//! projects molecule persistence facts into `ferrum-core`. Reference validation,
//! mutation transactions, and the session API remain later work. The stored tree
//! retains XML structure, not original source spelling.

mod core_projection;
mod typed;

pub use core_projection::{CoreProjection, CoreProjectionError};
pub use typed::{
    ExpandedName, NamespaceBinding, TypedChild, TypedClass, TypedDiagnostic, TypedDiagnosticKind,
    TypedDocument, TypedDocumentError, TypedRecord, TypedText, UnknownAttribute, UnrecognizedChild,
    UnrecognizedNode,
};

#[cfg(test)]
mod typed_tests;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use thiserror::Error;
use xot::Node;
use xot::Xot;

const CDML_NAMESPACE: &str = "http://www.freesoftware.fsf.org/bkchem/cdml";
static NEXT_DOCUMENT_INSTANCE: AtomicU64 = AtomicU64::new(0);

/// A nonblank persistent XML `id` exactly as it appeared in the source document.
///
/// This is deliberately distinct from a provisional token. It may be used to query
/// the document-wide index, but it does not assert a typed CDML record class.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PersistentId(String);

impl PersistentId {
    /// Validate and retain exact persistent ID source text.
    pub fn new(value: impl Into<String>) -> Result<Self, DocumentIdentityError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(DocumentIdentityError::BlankPersistentId);
        }
        Ok(Self(value))
    }

    /// Return the exact source spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PersistentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A one-use correlation value that has not become a persistent XML identity.
///
/// A token is only issued by an [`IndexedDocument`]. It has a different Rust type
/// from [`PersistentId`], so callers cannot pass a token to [`IndexedDocument::resolve_id`].
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ProvisionalToken {
    document_instance: u64,
    sequence: u64,
    spelling: String,
}

impl ProvisionalToken {
    /// Return the opaque correlation spelling for transport or diagnostics.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.spelling
    }

    /// Return the document-local issuance sequence.
    ///
    /// Distinct documents both begin at zero. A private document-instance component
    /// prevents same-sequence tokens from crossing document boundaries.
    #[must_use]
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

/// Stable position of a direct child element in the source `<cdml>` sequence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceOrder(u32);

impl SourceOrder {
    /// Return the zero-based direct-child position.
    #[must_use]
    pub fn value(self) -> u32 {
        self.0
    }
}

/// A structural path to an XML element, expressed as child indexes from the root.
///
/// The path is a diagnostic and indexing identity only. It carries no CDML semantics.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ElementPath(Vec<u32>);

impl ElementPath {
    /// Return the root-relative child-index components.
    #[must_use]
    pub fn components(&self) -> &[u32] {
        &self.0
    }
}

impl fmt::Display for ElementPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return formatter.write_str("/");
        }
        for component in &self.0 {
            write!(formatter, "/{component}")?;
        }
        Ok(())
    }
}

/// Index entry for a persistent XML `id`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedId {
    identifier: PersistentId,
    path: ElementPath,
    source_order: Option<SourceOrder>,
}

impl ResolvedId {
    /// Return the persistent ID resolved by this entry.
    #[must_use]
    pub fn identifier(&self) -> &PersistentId {
        &self.identifier
    }

    /// Return the source-stable structural location.
    #[must_use]
    pub fn path(&self) -> &ElementPath {
        &self.path
    }

    /// Return the enclosing direct-child order, if this is below a direct child.
    #[must_use]
    pub fn source_order(&self) -> Option<SourceOrder> {
        self.source_order
    }
}

/// An ordered direct child of the CDML root.
///
/// The identity index intentionally does not classify XML into typed record variants.
/// This record fixes the persistent direct-child sequence for later interpretation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentRecord {
    source_order: SourceOrder,
    identifier: Option<PersistentId>,
    path: ElementPath,
}

impl DocumentRecord {
    /// Return its canonical direct-child order.
    #[must_use]
    pub fn source_order(&self) -> SourceOrder {
        self.source_order
    }

    /// Return its direct-child ID, if one is present.
    #[must_use]
    pub fn identifier(&self) -> Option<&PersistentId> {
        self.identifier.as_ref()
    }

    /// Return its root-relative XML element path.
    #[must_use]
    pub fn path(&self) -> &ElementPath {
        &self.path
    }
}

/// Identity and ordering failures found while indexing opaque XML.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DocumentIdentityError {
    /// The root is not a CDML element in the canonical or legacy namespace form.
    #[error("expected a CDML root element, found {root_name} in namespace {namespace:?}")]
    NotCdmlRoot {
        /// Root local name.
        root_name: String,
        /// Root namespace URI.
        namespace: String,
    },
    /// An XML `id` cannot be used as a persistent document identity.
    #[error("persistent XML id must contain at least one non-whitespace character")]
    BlankPersistentId,
    /// More than one element reserved the same document-wide persistent ID.
    #[error(
        "duplicate persistent XML id {identifier:?}: first at {first}, duplicate at {duplicate}"
    )]
    DuplicatePersistentId {
        /// Colliding exact ID text.
        identifier: PersistentId,
        /// First structural occurrence.
        first: ElementPath,
        /// Later structural occurrence.
        duplicate: ElementPath,
    },
    /// A caller attempted to consume a token that this document never issued.
    #[error("provisional token {token:?} was not issued by this document")]
    UnknownProvisionalToken {
        /// Rejected token spelling.
        token: String,
    },
    /// A caller attempted to consume an already-consumed token.
    #[error("provisional token {token:?} was already consumed")]
    ConsumedProvisionalToken {
        /// Rejected token spelling.
        token: String,
    },
}

/// Parse or identity-index failures for [`IndexedDocument`].
#[derive(Debug, Error)]
pub enum IndexedDocumentError {
    /// The XML parser rejected the supplied source.
    #[error("XML parse error: {0}")]
    Xml(#[from] xot::ParseError),
    /// The opaque XML could not establish a safe identity index.
    #[error(transparent)]
    Identity(#[from] DocumentIdentityError),
}

/// Structural XML serialization failed.
///
/// This wrapper keeps the XML parser implementation out of downstream public error
/// contracts while retaining the original error as a source.
#[derive(Debug, Error)]
#[error("XML serialization error: {0}")]
pub struct XmlSerializationError(#[from] xot::Error);

/// An XML document retained as an opaque, editable tree.
///
/// The tree carries every element, attribute, namespace identity, child order, text,
/// comment, and processing instruction accepted by `xot`. The API intentionally does
/// not expose typed CDML records or a wire format.
#[derive(Debug)]
pub struct XmlDocument {
    tree: Xot,
    document: Node,
}

impl XmlDocument {
    /// Parse one well-formed XML 1.0 document into opaque storage.
    ///
    /// `xot` does not support DTDs, so this entry point neither expands external
    /// entities nor resolves network resources. Callers must provide decoded text.
    pub fn parse(source: &str) -> Result<Self, xot::ParseError> {
        let mut tree = Xot::new();
        let document = tree.parse(source)?;
        Ok(Self { tree, document })
    }

    /// Serialize the retained XML tree.
    ///
    /// The result is structurally equivalent XML. XML declaration spelling, CDATA
    /// boundaries, entity spelling, prefixes, and attribute order are lexical details
    /// that a tree parser may normalize.
    pub fn to_xml(&self) -> Result<String, XmlSerializationError> {
        self.tree
            .to_string(self.document)
            .map_err(XmlSerializationError::from)
    }
}

/// An opaque CDML document accompanied by its stable identity and order index.
///
/// Parsing never rewrites XML attributes or text. The index reserves declaration
/// `id` attributes and every unqualified `id` inside foreign or otherwise opaque
/// subtrees, so later persistent identity allocation cannot collide with unknown
/// content. The two context-defined fragment `id` reference fields are not indexed as
/// declarations. Other references such as `idref`, `start`, and free text are never
/// interpreted here.
#[derive(Debug)]
pub struct IndexedDocument {
    xml: XmlDocument,
    records: Vec<DocumentRecord>,
    id_index: BTreeMap<PersistentId, ResolvedId>,
    issued_tokens: BTreeSet<ProvisionalToken>,
    consumed_tokens: BTreeSet<ProvisionalToken>,
    document_instance: u64,
    next_token: u64,
}

impl IndexedDocument {
    /// Parse opaque CDML, validate document-wide persistent ID uniqueness, and index
    /// direct-child source order.
    pub fn parse(source: &str) -> Result<Self, IndexedDocumentError> {
        let xml = XmlDocument::parse(source)?;
        Self::from_xml(xml).map_err(IndexedDocumentError::from)
    }

    fn from_xml(xml: XmlDocument) -> Result<Self, DocumentIdentityError> {
        let root = xml
            .tree
            .document_element(xml.document)
            .expect("a parsed XML document has a document element");
        let (root_name, namespace) =
            element_name(&xml.tree, root).expect("a document element is always an XML element");
        if root_name != "cdml" || (!namespace.is_empty() && namespace != CDML_NAMESPACE) {
            return Err(DocumentIdentityError::NotCdmlRoot {
                root_name,
                namespace,
            });
        }

        let mut id_index = BTreeMap::new();
        let mut records = Vec::new();
        let mut root_path = Vec::new();
        index_element(
            &xml.tree,
            root,
            None,
            &mut root_path,
            None,
            &mut id_index,
            &mut records,
        )?;
        Ok(Self {
            xml,
            records,
            id_index,
            issued_tokens: BTreeSet::new(),
            consumed_tokens: BTreeSet::new(),
            document_instance: NEXT_DOCUMENT_INSTANCE.fetch_add(1, Ordering::Relaxed),
            next_token: 0,
        })
    }

    /// Return opaque XML storage for structural serialization.
    #[must_use]
    pub fn xml(&self) -> &XmlDocument {
        &self.xml
    }

    /// Return direct-root records in their canonical source order.
    #[must_use]
    pub fn records(&self) -> &[DocumentRecord] {
        &self.records
    }

    /// Resolve one validated persistent XML ID.
    #[must_use]
    pub fn resolve_id(&self, identifier: &PersistentId) -> Option<&ResolvedId> {
        self.id_index.get(identifier)
    }

    /// Return the number of reserved persistent IDs, including opaque content.
    #[must_use]
    pub fn persistent_id_count(&self) -> usize {
        self.id_index.len()
    }

    /// Issue a fresh, document-local provisional token.
    ///
    /// The token is not an XML `id`, is never inserted into [`Self::id_index`], and
    /// can only be consumed once by this document. Issuance is deterministic in one
    /// process: every document starts at sequence zero but has a process-local instance
    /// component. Tokens have no persisted-document meaning.
    pub fn issue_provisional_token(&mut self) -> ProvisionalToken {
        let sequence = self.next_token;
        let token = ProvisionalToken {
            document_instance: self.document_instance,
            sequence,
            spelling: format!("ferrum-provisional-{}-{sequence}", self.document_instance),
        };
        self.next_token += 1;
        let inserted = self.issued_tokens.insert(token.clone());
        debug_assert!(inserted, "monotonic provisional tokens are unique");
        token
    }

    /// Consume a token exactly once after its candidate has been accepted.
    pub fn consume_provisional_token(
        &mut self,
        token: ProvisionalToken,
    ) -> Result<(), DocumentIdentityError> {
        if token.document_instance != self.document_instance {
            return Err(DocumentIdentityError::UnknownProvisionalToken {
                token: token.spelling,
            });
        }
        if self.consumed_tokens.contains(&token) {
            return Err(DocumentIdentityError::ConsumedProvisionalToken {
                token: token.spelling,
            });
        }
        if !self.issued_tokens.contains(&token) {
            return Err(DocumentIdentityError::UnknownProvisionalToken {
                token: token.spelling,
            });
        }
        let inserted = self.consumed_tokens.insert(token);
        debug_assert!(inserted, "the consumed-token check established uniqueness");
        Ok(())
    }
}

fn element_name(tree: &Xot, node: Node) -> Option<(String, String)> {
    let element = tree.element(node)?;
    let (local_name, namespace) = tree.name_ns_str(element.name());
    Some((local_name.to_string(), namespace.to_string()))
}

fn persistent_id(
    tree: &Xot,
    node: Node,
    parent: Option<Node>,
) -> Result<Option<PersistentId>, DocumentIdentityError> {
    if is_fragment_id_reference(tree, node, parent) {
        return Ok(None);
    }
    let identifier = tree.attributes(node).iter().find_map(|(name, value)| {
        let (local_name, namespace) = tree.name_ns_str(name);
        (local_name == "id" && namespace.is_empty()).then(|| value.clone())
    });
    identifier.map(PersistentId::new).transpose()
}

fn is_fragment_id_reference(tree: &Xot, node: Node, parent: Option<Node>) -> bool {
    let Some(parent) = parent else {
        return false;
    };
    let Some((local_name, namespace)) = element_name(tree, node) else {
        return false;
    };
    let Some((parent_name, parent_namespace)) = element_name(tree, parent) else {
        return false;
    };
    let core_namespace = |namespace: &str| namespace.is_empty() || namespace == CDML_NAMESPACE;
    core_namespace(&namespace)
        && core_namespace(&parent_namespace)
        && parent_name == "fragment"
        && matches!(local_name.as_str(), "bond" | "vertex")
}

fn index_element(
    tree: &Xot,
    node: Node,
    parent: Option<Node>,
    path: &mut Vec<u32>,
    source_order: Option<SourceOrder>,
    id_index: &mut BTreeMap<PersistentId, ResolvedId>,
    records: &mut Vec<DocumentRecord>,
) -> Result<(), DocumentIdentityError> {
    let element_path = ElementPath(path.clone());
    if let Some(identifier) = persistent_id(tree, node, parent)? {
        let resolved = ResolvedId {
            identifier: identifier.clone(),
            path: element_path.clone(),
            source_order,
        };
        if let Some(first) = id_index.get(&identifier) {
            return Err(DocumentIdentityError::DuplicatePersistentId {
                identifier,
                first: first.path.clone(),
                duplicate: element_path,
            });
        }
        id_index.insert(identifier, resolved);
    }

    let mut element_child_index = 0_u32;
    for child in tree.children(node) {
        if tree.element(child).is_none() {
            continue;
        }
        path.push(element_child_index);
        let child_order = if path.len() == 1 {
            Some(SourceOrder(element_child_index))
        } else {
            source_order
        };
        if path.len() == 1 {
            records.push(DocumentRecord {
                source_order: child_order.expect("direct child receives source order"),
                identifier: persistent_id(tree, child, Some(node))?,
                path: ElementPath(path.clone()),
            });
        }
        index_element(
            tree,
            child,
            Some(node),
            path,
            child_order,
            id_index,
            records,
        )?;
        path.pop();
        element_child_index += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use xot::Node;
    use xot::Value;
    use xot::Xot;

    use super::{DocumentIdentityError, IndexedDocument, PersistentId, XmlDocument};

    #[derive(Debug, PartialEq, Eq)]
    enum Shape {
        Document(Vec<Shape>),
        Element {
            namespace: String,
            local_name: String,
            attributes: Vec<(String, String, String)>,
            children: Vec<Shape>,
        },
        Text(String),
        Comment(String),
        ProcessingInstruction {
            target: String,
            data: Option<String>,
        },
    }

    fn structural_shape(tree: &Xot, node: Node) -> Shape {
        match tree.value(node) {
            Value::Document => Shape::Document(
                tree.children(node)
                    .map(|child| structural_shape(tree, child))
                    .collect(),
            ),
            Value::Element(element) => {
                let (local_name, namespace) = tree.name_ns_str(element.name());
                let mut attributes = tree
                    .attributes(node)
                    .iter()
                    .map(|(name, value)| {
                        let (local_name, namespace) = tree.name_ns_str(name);
                        (namespace.to_string(), local_name.to_string(), value.clone())
                    })
                    .collect::<Vec<_>>();
                attributes.sort();
                Shape::Element {
                    namespace: namespace.to_string(),
                    local_name: local_name.to_string(),
                    attributes,
                    children: tree
                        .children(node)
                        .map(|child| structural_shape(tree, child))
                        .collect(),
                }
            }
            Value::Text(text) => Shape::Text(text.get().to_string()),
            Value::Comment(comment) => Shape::Comment(comment.get().to_string()),
            Value::ProcessingInstruction(instruction) => {
                let (local_name, namespace) = tree.name_ns_str(instruction.target());
                let target = format!("{{{namespace}}}{local_name}");
                Shape::ProcessingInstruction {
                    target,
                    data: instruction.data().map(str::to_string),
                }
            }
            Value::Attribute(_) | Value::Namespace(_) => {
                panic!("attribute and namespace nodes are not normal children")
            }
        }
    }

    fn parsed_shape(source: &str) -> Shape {
        let mut tree = Xot::new();
        let document = tree.parse(source).expect("inline XML must parse");
        structural_shape(&tree, document)
    }

    #[test]
    fn opaque_xml_round_trip_preserves_structure_across_namespace_spellings() {
        let source = r#"<?before retain?>
<cdml xmlns="urn:cdml" xmlns:vendor="urn:vendor" xmlns:alt="urn:vendor"
      xmlns:q="urn:qname" z="last" vendor:state="literal" a="first">
  before<vendor:extension q:kind="q:literal" alt:flag="yes">inside<q:item/>after</vendor:extension>
  <unknown xmlns="urn:other" vendor:label="keep"/>tail
  <!-- retained --><?inside remain?>
</cdml>"#;

        let document = XmlDocument::parse(source).expect("inline XML must parse");
        let serialized = document.to_xml().expect("stored XML must serialize");

        assert_eq!(parsed_shape(source), parsed_shape(&serialized));
        assert!(serialized.contains("q:literal"));
    }

    #[test]
    fn dtd_input_is_rejected_without_external_entity_resolution() {
        let source = r#"<!DOCTYPE cdml [<!ENTITY external SYSTEM "https://example.invalid/entity">]>
<cdml>&external;</cdml>"#;

        assert!(XmlDocument::parse(source).is_err());
    }

    #[test]
    fn indexed_document_round_trip_retains_direct_source_order_and_identity_paths() {
        let source = r#"<cdml xmlns="http://www.freesoftware.fsf.org/bkchem/cdml">
  <info id="header"/>
  <molecule id="m1"><atom id="a1"/></molecule>
  <arrow id="arrow-1"/>
</cdml>"#;

        let document = IndexedDocument::parse(source).expect("valid CDML index");
        let before = document
            .records()
            .iter()
            .map(|record| {
                (
                    record.source_order().value(),
                    record.identifier().map(|identifier| identifier.as_str()),
                    record.path().components().to_vec(),
                )
            })
            .collect::<Vec<_>>();
        let serialized = document.xml().to_xml().expect("stored XML serializes");
        let reparsed = IndexedDocument::parse(&serialized).expect("serialized CDML indexes");
        let after = reparsed
            .records()
            .iter()
            .map(|record| {
                (
                    record.source_order().value(),
                    record.identifier().map(|identifier| identifier.as_str()),
                    record.path().components().to_vec(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(before, after);
        let atom = PersistentId::new("a1").expect("nonblank id");
        assert_eq!(
            document.resolve_id(&atom).unwrap().path().components(),
            &[1, 0]
        );
        assert_eq!(
            document
                .resolve_id(&atom)
                .unwrap()
                .source_order()
                .unwrap()
                .value(),
            1
        );
    }

    #[test]
    fn duplicate_and_blank_persistent_ids_fail_with_locations() {
        let duplicate = r#"<cdml><molecule id="same"/><arrow id="same"/></cdml>"#;
        let duplicate_error = IndexedDocument::parse(duplicate).expect_err("duplicate id fails");
        assert!(matches!(
            duplicate_error,
            super::IndexedDocumentError::Identity(DocumentIdentityError::DuplicatePersistentId {
                ref first,
                ref duplicate,
                ..
            }) if first.components() == [0] && duplicate.components() == [1]
        ));

        let blank = r#"<cdml><molecule id=" "/></cdml>"#;
        assert!(matches!(
            IndexedDocument::parse(blank),
            Err(super::IndexedDocumentError::Identity(
                DocumentIdentityError::BlankPersistentId
            ))
        ));
    }

    #[test]
    fn provisional_tokens_are_distinct_from_persistent_ids_and_consumed_once() {
        let mut document =
            IndexedDocument::parse("<cdml><molecule id=\"m1\"/></cdml>").expect("valid CDML index");
        let token = document.issue_provisional_token();
        let replay = token.clone();
        assert_eq!(token.sequence(), 0);
        document
            .consume_provisional_token(token)
            .expect("first consumption succeeds");
        assert!(matches!(
            document.consume_provisional_token(replay),
            Err(DocumentIdentityError::ConsumedProvisionalToken { .. })
        ));

        let foreign = super::ProvisionalToken {
            document_instance: u64::MAX,
            sequence: 0,
            spelling: "foreign".to_string(),
        };
        assert!(matches!(
            document.consume_provisional_token(foreign),
            Err(DocumentIdentityError::UnknownProvisionalToken { .. })
        ));
        let durable = PersistentId::new("m1").expect("nonblank persistent id");
        assert!(document.resolve_id(&durable).is_some());
    }

    #[test]
    fn provisional_tokens_with_matching_sequences_cannot_cross_documents() {
        let mut first = IndexedDocument::parse("<cdml/>").expect("first valid CDML index");
        let mut second = IndexedDocument::parse("<cdml/>").expect("second valid CDML index");
        let first_token = first.issue_provisional_token();
        let second_token = second.issue_provisional_token();

        assert_eq!(first_token.sequence(), 0);
        assert_eq!(second_token.sequence(), 0);
        assert_ne!(first_token.as_str(), second_token.as_str());
        assert!(matches!(
            second.consume_provisional_token(first_token.clone()),
            Err(DocumentIdentityError::UnknownProvisionalToken { .. })
        ));
        second
            .consume_provisional_token(second_token)
            .expect("own token is consumable");
        first
            .consume_provisional_token(first_token)
            .expect("own token remains consumable after foreign rejection");
    }

    #[test]
    fn root_id_reserves_the_document_wide_collision_name() {
        let root_identifier = PersistentId::new("document").expect("nonblank id");
        let indexed = IndexedDocument::parse("<cdml id=\"document\"><molecule/></cdml>")
            .expect("root id indexes");
        assert_eq!(
            indexed
                .resolve_id(&root_identifier)
                .unwrap()
                .path()
                .components(),
            &[]
        );
        assert_eq!(
            indexed.resolve_id(&root_identifier).unwrap().source_order(),
            None
        );
        assert!(indexed.records()[0].identifier().is_none());

        let duplicate = "<cdml id=\"document\"><molecule id=\"document\"/></cdml>";
        assert!(matches!(
            IndexedDocument::parse(duplicate),
            Err(super::IndexedDocumentError::Identity(
                DocumentIdentityError::DuplicatePersistentId { ref first, ref duplicate, .. }
            )) if first.components().is_empty() && duplicate.components() == [0]
        ));
        assert!(matches!(
            IndexedDocument::parse("<cdml id=\" \"/>"),
            Err(super::IndexedDocumentError::Identity(
                DocumentIdentityError::BlankPersistentId
            ))
        ));
    }

    #[test]
    fn opaque_reference_looking_values_are_reserved_but_never_rewritten() {
        let source = r#"<cdml xmlns="http://www.freesoftware.fsf.org/bkchem/cdml"
  xmlns:v="urn:vendor"><molecule id="m1"/><v:extension id="opaque-id"
  idref="m1">reference m1 <v:item start="m1">m1</v:item></v:extension></cdml>"#;

        let document = IndexedDocument::parse(source).expect("opaque content indexes");
        let opaque_id = PersistentId::new("opaque-id").expect("nonblank id");
        assert!(document.resolve_id(&opaque_id).is_some());
        assert_eq!(document.persistent_id_count(), 2);
        let serialized = document.xml().to_xml().expect("stored XML serializes");

        assert!(serialized.contains("idref=\"m1\""));
        assert!(serialized.contains("start=\"m1\""));
        assert!(serialized.contains("reference m1"));
    }

    #[test]
    fn fragment_id_references_do_not_collide_with_their_declarations() {
        let source = r#"<cdml><molecule id="m1"><atom id="a1"/><bond id="b1"/>
<fragment id="f1"><bond id="b1"/><vertex id="a1"/></fragment></molecule></cdml>"#;
        let document = IndexedDocument::parse(source).expect("fragment references are not ids");

        assert_eq!(document.persistent_id_count(), 4);
        let bond = PersistentId::new("b1").expect("nonblank id");
        assert_eq!(
            document.resolve_id(&bond).unwrap().path().components(),
            &[0, 1]
        );
    }
}

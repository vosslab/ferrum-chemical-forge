//! Opaque XML retention and persistent identity indexing for CDML documents.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::fmt::Write;
use std::sync::atomic::{AtomicU64, Ordering};

use thiserror::Error;
use xot::{Node, Xot};

pub(crate) const CDML_NAMESPACE: &str = "http://www.freesoftware.fsf.org/bkchem/cdml";
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
#[derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct ProvisionalToken {
    document_instance: u64,
    sequence: u64,
    spelling: String,
}

impl fmt::Debug for ProvisionalToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProvisionalToken([opaque])")
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
pub struct ElementPath(pub(crate) Vec<u32>);

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
    /// Storage for a provisional-token registry could not be reserved.
    #[error("provisional token registry allocation failed")]
    ProvisionalTokenAllocationFailed,
    /// The monotonic provisional-token sequence cannot advance further.
    #[error("provisional token sequence is exhausted")]
    ProvisionalTokenExhausted,
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
    pub(crate) tree: Xot,
    pub(crate) document: Node,
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

    /// Preflight caller-supplied XML resource limits, then retain one opaque XML tree.
    ///
    /// The preflight is a non-retaining tokenizer pass. It rejects an over-budget or
    /// DTD-bearing input before `xot` allocates its retained tree. No library default
    /// is provided: each external ingress must choose and document its own policy.
    pub fn parse_with_budget(
        source: &str,
        budget: super::XmlInputBudgetV1,
    ) -> Result<Self, super::XmlInputError> {
        super::xml_input_budget_v1::preflight(source, budget)?;
        let mut tree = Xot::new();
        let document = tree.parse(source).map_err(super::XmlInputError::Xml)?;
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
    pub(crate) xml: XmlDocument,
    records: Vec<DocumentRecord>,
    id_index: BTreeMap<PersistentId, ResolvedId>,
    issued_tokens: HashSet<u64>,
    consumed_tokens: HashSet<u64>,
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

    pub(crate) fn from_xml(xml: XmlDocument) -> Result<Self, DocumentIdentityError> {
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
            issued_tokens: HashSet::new(),
            consumed_tokens: HashSet::new(),
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

    /// Fallibly reserve both token registries before issuing one token.
    ///
    /// Every issued token reserves its later consumed-set entry, so a successful
    /// `consume_provisional_token` only probes and inserts into existing capacity.
    pub(crate) fn try_issue_provisional_token(
        &mut self,
    ) -> Result<ProvisionalToken, DocumentIdentityError> {
        self.issued_tokens
            .try_reserve(1)
            .map_err(|_| DocumentIdentityError::ProvisionalTokenAllocationFailed)?;
        self.consumed_tokens
            .try_reserve(1)
            .map_err(|_| DocumentIdentityError::ProvisionalTokenAllocationFailed)?;
        let sequence = self.next_token;
        let next_token = sequence
            .checked_add(1)
            .ok_or(DocumentIdentityError::ProvisionalTokenExhausted)?;
        let mut spelling = String::new();
        spelling
            .try_reserve("ferrum-provisional-".len() + 40)
            .map_err(|_| DocumentIdentityError::ProvisionalTokenAllocationFailed)?;
        write!(
            spelling,
            "ferrum-provisional-{}-{sequence}",
            self.document_instance
        )
        .expect("writing to a reserved String is infallible");
        let token = ProvisionalToken {
            document_instance: self.document_instance,
            sequence,
            spelling,
        };
        self.next_token = next_token;
        let inserted = self.issued_tokens.insert(sequence);
        debug_assert!(inserted, "monotonic provisional tokens are unique");
        Ok(token)
    }

    /// Consume a token exactly once after its candidate has been accepted.
    pub(crate) fn consume_provisional_token(
        &mut self,
        token: &ProvisionalToken,
    ) -> Result<(), DocumentIdentityError> {
        if token.document_instance != self.document_instance {
            return Err(DocumentIdentityError::UnknownProvisionalToken {
                token: token.spelling.clone(),
            });
        }
        if self.consumed_tokens.contains(&token.sequence) {
            return Err(DocumentIdentityError::ConsumedProvisionalToken {
                token: token.spelling.clone(),
            });
        }
        if !self.issued_tokens.contains(&token.sequence) {
            return Err(DocumentIdentityError::UnknownProvisionalToken {
                token: token.spelling.clone(),
            });
        }
        let inserted = self.consumed_tokens.insert(token.sequence);
        debug_assert!(inserted, "the consumed-token check established uniqueness");
        Ok(())
    }

    /// Check whether a token belongs to this document and remains unconsumed.
    pub(crate) fn verify_provisional_token(
        &self,
        token: &ProvisionalToken,
    ) -> Result<(), DocumentIdentityError> {
        if token.document_instance != self.document_instance
            || !self.issued_tokens.contains(&token.sequence)
        {
            return Err(DocumentIdentityError::UnknownProvisionalToken {
                token: token.spelling.clone(),
            });
        }
        if self.consumed_tokens.contains(&token.sequence) {
            return Err(DocumentIdentityError::ConsumedProvisionalToken {
                token: token.spelling.clone(),
            });
        }
        Ok(())
    }
}

pub(crate) fn element_name(tree: &Xot, node: Node) -> Option<(String, String)> {
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

//! Bounded, insertion-valid CDML clipboard plans for native Paste.

use std::collections::BTreeMap;

use thiserror::Error;
use xot::Node;

use super::{
    DocumentObjectIdV1, PersistentId, TopLevelRootKindV1, TopLevelRootSelectorV1,
    TopLevelTransformModeV1, TopLevelTransformV1, TypedClass, TypedDocument, TypedDocumentError,
    UnrecognizedNode, XmlInputBudgetV1, XmlSerializationError,
};

/// Stable schema identifier for one worker-prepared native Paste plan.
pub const DOCUMENT_CLIPBOARD_PASTE_SCHEMA_V1: &str = "ferrum-document-clipboard-paste-v1";

/// One validated source root retained by a prepared Paste plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentClipboardPasteRootV1 {
    source_id: PersistentId,
    kind: TopLevelRootKindV1,
}

impl DocumentClipboardPasteRootV1 {
    /// Return the exact source-fragment root ID.
    #[must_use]
    pub fn source_id(&self) -> &PersistentId {
        &self.source_id
    }

    /// Return the closed supported root kind.
    #[must_use]
    pub const fn kind(&self) -> TopLevelRootKindV1 {
        self.kind
    }
}

/// Immutable, handle-free result of bounded clipboard-fragment admission.
#[derive(Debug)]
pub struct DocumentClipboardPastePlanV1 {
    schema: &'static str,
    canonical_fragment: String,
    roots: Vec<DocumentClipboardPasteRootV1>,
    declared_ids: Vec<PersistentId>,
}

impl DocumentClipboardPastePlanV1 {
    /// Return the closed plan schema.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Return supported source roots in exact fragment order.
    #[must_use]
    pub fn roots(&self) -> &[DocumentClipboardPasteRootV1] {
        &self.roots
    }

    /// Return the number of document-wide ID declarations that Paste must replace.
    #[must_use]
    pub fn declared_id_count(&self) -> usize {
        self.declared_ids.len()
    }
}

/// One inserted root returned after the authoritative Paste transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentClipboardPastedRootV1 {
    object_id: DocumentObjectIdV1,
    source_id: PersistentId,
    kind: TopLevelRootKindV1,
}

impl DocumentClipboardPastedRootV1 {
    /// Return the opaque durable selector for the inserted root.
    #[must_use]
    pub fn object_id(&self) -> &DocumentObjectIdV1 {
        &self.object_id
    }

    /// Return the generated persistent XML ID installed on the root.
    #[must_use]
    pub fn source_id(&self) -> &PersistentId {
        &self.source_id
    }

    /// Return the closed inserted root kind.
    #[must_use]
    pub const fn kind(&self) -> TopLevelRootKindV1 {
        self.kind
    }
}

/// Failure while admitting or composing one insertion-valid Paste candidate.
#[derive(Debug, Error)]
pub enum DocumentClipboardPasteErrorV1 {
    /// The caller's installed observation no longer matches the session state.
    #[error("clipboard Paste expected a different document digest")]
    DigestMismatch,
    /// XML admission, identity, or typed recognition failed.
    #[error("clipboard Paste fragment is invalid CDML: {0}")]
    Typed(#[from] TypedDocumentError),
    /// The fragment carried no supported direct root.
    #[error("clipboard Paste requires at least one supported direct root")]
    EmptyFragment,
    /// A direct fragment child was not in the closed Paste root grammar.
    #[error("clipboard Paste fragment has an unsupported direct-root record")]
    UnsupportedRoot,
    /// A supported direct root did not have one durable source ID.
    #[error("clipboard Paste root lacks a durable persistent ID")]
    MissingRootId,
    /// Root-level comments, processing instructions, or non-whitespace text are not Paste roots.
    #[error("clipboard Paste fragment has unsupported root-level content")]
    UnsupportedRootContent,
    /// The supplied generated-ID count disagreed with the prepared plan.
    #[error("clipboard Paste generated identity count disagrees with its prepared plan")]
    IdentityCountMismatch,
    /// A generated root identity could not be re-resolved after remapping.
    #[error("clipboard Paste generated root identity did not resolve")]
    IdentityInvariant,
    /// The translated root selector could not be constructed.
    #[error("clipboard Paste could not construct its translated root selection: {0}")]
    TransformRequest(#[from] super::TopLevelTransformV1Error),
    /// The retained XML tree refused a structural append.
    #[error("clipboard Paste could not append retained XML: {0}")]
    Mutation(#[source] xot::Error),
    /// A retained candidate could not be serialized.
    #[error("clipboard Paste could not serialize retained XML: {0}")]
    Serialize(#[from] XmlSerializationError),
    /// Re-parsing already-admitted canonical fragment XML failed unexpectedly.
    #[error("clipboard Paste could not reparse admitted XML: {0}")]
    Parse(#[source] xot::ParseError),
}

/// Admit one external CDML fragment under an explicit caller-owned budget.
pub fn prepare_document_clipboard_paste_v1(
    source: &str,
    budget: XmlInputBudgetV1,
) -> Result<DocumentClipboardPastePlanV1, DocumentClipboardPasteErrorV1> {
    let document = TypedDocument::parse_with_budget(source, budget)?;
    let roots = validate_roots(&document)?;
    let declared_ids = document
        .indexed()
        .persistent_ids()
        .cloned()
        .collect::<Vec<_>>();
    let canonical_fragment = document.to_xml()?;
    Ok(DocumentClipboardPastePlanV1 {
        schema: DOCUMENT_CLIPBOARD_PASTE_SCHEMA_V1,
        canonical_fragment,
        roots,
        declared_ids,
    })
}

fn validate_roots(
    document: &TypedDocument,
) -> Result<Vec<DocumentClipboardPasteRootV1>, DocumentClipboardPasteErrorV1> {
    for child in document.root().unrecognized_children() {
        match child.node() {
            UnrecognizedNode::Text(text) if text.trim().is_empty() => {}
            _ => return Err(DocumentClipboardPasteErrorV1::UnsupportedRootContent),
        }
    }
    let mut roots = Vec::new();
    for child in document.root().typed_children() {
        let record = child.record();
        let Some(kind) = root_kind(record.class()) else {
            return Err(DocumentClipboardPasteErrorV1::UnsupportedRoot);
        };
        let source_id = record
            .attribute("id")
            .ok_or(DocumentClipboardPasteErrorV1::MissingRootId)
            .and_then(|value| {
                PersistentId::new(value.to_owned())
                    .map_err(|_| DocumentClipboardPasteErrorV1::MissingRootId)
            })?;
        roots.push(DocumentClipboardPasteRootV1 { source_id, kind });
    }
    if roots.is_empty() {
        return Err(DocumentClipboardPasteErrorV1::EmptyFragment);
    }
    Ok(roots)
}

fn root_kind(class: TypedClass) -> Option<TopLevelRootKindV1> {
    match class {
        TypedClass::Molecule => Some(TopLevelRootKindV1::Molecule),
        TypedClass::CanvasArrow => Some(TopLevelRootKindV1::Arrow),
        TypedClass::CanvasPlus => Some(TopLevelRootKindV1::Plus),
        TypedClass::CanvasText => Some(TopLevelRootKindV1::Text),
        TypedClass::Rectangle => Some(TopLevelRootKindV1::Rectangle),
        TypedClass::Square => Some(TopLevelRootKindV1::Square),
        TypedClass::Oval => Some(TopLevelRootKindV1::Oval),
        TypedClass::Circle => Some(TopLevelRootKindV1::Circle),
        TypedClass::Polygon => Some(TopLevelRootKindV1::Polygon),
        TypedClass::Polyline => Some(TopLevelRootKindV1::Polyline),
        _ => None,
    }
}

pub(super) fn compose_clipboard_paste_candidate_v1(
    current: &TypedDocument,
    plan: &DocumentClipboardPastePlanV1,
    generated_ids: &[PersistentId],
    dx: f64,
    dy: f64,
) -> Result<(TypedDocument, Vec<DocumentClipboardPastedRootV1>), DocumentClipboardPasteErrorV1> {
    if generated_ids.len() != plan.declared_ids.len() {
        return Err(DocumentClipboardPasteErrorV1::IdentityCountMismatch);
    }
    let replacements = plan
        .declared_ids
        .iter()
        .cloned()
        .zip(generated_ids.iter().cloned())
        .collect::<BTreeMap<_, _>>();
    let mut fragment = TypedDocument::parse(&plan.canonical_fragment)?;
    remap_exact_attribute_values(&mut fragment, &replacements)?;
    let mut selectors = Vec::new();
    let mut inserted_roots = Vec::new();
    for root in &plan.roots {
        let generated = replacements
            .get(&root.source_id)
            .ok_or(DocumentClipboardPasteErrorV1::IdentityInvariant)?;
        selectors.push(TopLevelRootSelectorV1::new(
            generated.as_str().to_owned(),
            root.kind,
        )?);
        let record = fragment
            .root()
            .typed_children()
            .iter()
            .map(|child| child.record())
            .find(|record| record.attribute("id") == Some(generated.as_str()))
            .ok_or(DocumentClipboardPasteErrorV1::IdentityInvariant)?;
        let object_id = DocumentObjectIdV1::from_record(record)
            .ok_or(DocumentClipboardPasteErrorV1::IdentityInvariant)?;
        inserted_roots.push(DocumentClipboardPastedRootV1 {
            object_id,
            source_id: generated.clone(),
            kind: root.kind,
        });
    }
    let transform =
        TopLevelTransformV1::new(selectors, TopLevelTransformModeV1::Translate { dx, dy })?;
    let translated = fragment.with_top_level_transform(&transform)?;
    let candidate = append_fragment_roots(current, &translated)?;
    Ok((candidate, inserted_roots))
}

fn remap_exact_attribute_values(
    document: &mut TypedDocument,
    replacements: &BTreeMap<PersistentId, PersistentId>,
) -> Result<(), DocumentClipboardPasteErrorV1> {
    let indexed = document.detached_indexed_mut();
    let root = indexed
        .xml
        .tree
        .document_element(indexed.xml.document)
        .map_err(DocumentClipboardPasteErrorV1::Mutation)?;
    let nodes = indexed.xml.tree.descendants(root).collect::<Vec<_>>();
    for node in nodes {
        if indexed.xml.tree.element(node).is_none() {
            continue;
        }
        let changes = indexed
            .xml
            .tree
            .attributes(node)
            .iter()
            .filter_map(|(name, value)| {
                let source = PersistentId::new(value.clone()).ok()?;
                replacements
                    .get(&source)
                    .map(|replacement| (name, replacement.as_str().to_owned()))
            })
            .collect::<Vec<_>>();
        for (name, value) in changes {
            indexed.xml.tree.set_attribute(node, name, value);
        }
    }
    *document = TypedDocument::parse(&document.to_xml()?)?;
    Ok(())
}

fn append_fragment_roots(
    current: &TypedDocument,
    fragment: &TypedDocument,
) -> Result<TypedDocument, DocumentClipboardPasteErrorV1> {
    let mut candidate = current.detached_candidate()?;
    let fragment_xml = fragment.to_xml()?;
    let indexed = candidate.detached_indexed_mut();
    let parsed = indexed
        .xml
        .tree
        .parse(&fragment_xml)
        .map_err(DocumentClipboardPasteErrorV1::Parse)?;
    let fragment_root = indexed
        .xml
        .tree
        .document_element(parsed)
        .map_err(DocumentClipboardPasteErrorV1::Mutation)?;
    let destination = indexed
        .xml
        .tree
        .document_element(indexed.xml.document)
        .map_err(DocumentClipboardPasteErrorV1::Mutation)?;
    let roots = indexed
        .xml
        .tree
        .children(fragment_root)
        .filter(|node| indexed.xml.tree.element(*node).is_some())
        .collect::<Vec<Node>>();
    for root in roots {
        let cloned = indexed.xml.tree.clone_with_prefixes(root);
        indexed
            .xml
            .tree
            .append(destination, cloned)
            .map_err(DocumentClipboardPasteErrorV1::Mutation)?;
    }
    TypedDocument::parse(&candidate.to_xml()?).map_err(Into::into)
}

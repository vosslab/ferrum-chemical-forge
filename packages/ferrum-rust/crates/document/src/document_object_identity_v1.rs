//! Document-owned persistence for opaque durable object selectors.

use std::collections::HashMap;

use ferrum_document_projection::{DocumentLocationKindV1, DocumentLocationV1, DocumentObjectIdV1};
use xot::{Node, Xot};

use crate::{IndexedDocument, TypedClass, TypedDocumentError, TypedRecord};

/// Namespace reserved for Ferrum's persisted opaque document-object selectors.
pub(crate) const DOCUMENT_OBJECT_NAMESPACE_V1: &str = "urn:ferrum:document-object:v1";
const DOCUMENT_OBJECT_ATTRIBUTE_V1: &str = "id";
const DOCUMENT_OBJECT_PREFIX_V1: &str = "ferrum-object";
const MAX_DOCUMENT_OBJECT_ALLOCATION_ATTEMPTS_V1: usize = 32;

/// Allocate document-object selectors independently of CDML source identifiers.
///
/// Production uses [`OsDocumentObjectIdAllocatorV1`]. The crate-private trait
/// permits deterministic identity sequences in focused document tests only.
pub(crate) trait DocumentObjectIdAllocatorV1 {
    fn allocate(&mut self) -> Result<DocumentObjectIdV1, getrandom::Error>;
}

pub(crate) struct OsDocumentObjectIdAllocatorV1;

impl DocumentObjectIdAllocatorV1 for OsDocumentObjectIdAllocatorV1 {
    fn allocate(&mut self) -> Result<DocumentObjectIdV1, getrandom::Error> {
        let mut entropy = [0_u8; 16];
        getrandom::fill(&mut entropy)?;
        Ok(DocumentObjectIdV1::from_entropy_bytes(entropy))
    }
}

/// Normalize persisted opaque selectors before a typed document can escape.
pub(crate) fn normalize_document_object_ids_v1(
    indexed: &mut IndexedDocument,
    root: &TypedRecord,
) -> Result<(), TypedDocumentError> {
    normalize_document_object_ids_with_allocator_v1(
        indexed,
        root,
        &mut OsDocumentObjectIdAllocatorV1,
    )
}

pub(crate) fn normalize_document_object_ids_with_allocator_v1(
    indexed: &mut IndexedDocument,
    root: &TypedRecord,
    allocator: &mut impl DocumentObjectIdAllocatorV1,
) -> Result<(), TypedDocumentError> {
    let mut seen = HashMap::new();
    indexed.reset_document_object_index_v1();
    normalize_record(indexed, root, allocator, &mut seen)
}

fn normalize_record(
    indexed: &mut IndexedDocument,
    record: &TypedRecord,
    allocator: &mut impl DocumentObjectIdAllocatorV1,
    seen: &mut HashMap<DocumentObjectIdV1, DocumentLocationV1>,
) -> Result<(), TypedDocumentError> {
    if is_addressable_record(record) {
        let location = location_for_record_v1(record);
        let source = record.attribute("id").ok_or_else(|| {
            TypedDocumentError::MissingStructuralSourceId {
                location: location.clone(),
            }
        })?;
        if source.trim().is_empty() {
            return Err(TypedDocumentError::InvalidStructuralSourceId { location });
        }
        let node = node_at_path(
            &indexed.xml.tree,
            indexed.xml.document,
            record.path().components(),
        )
        .expect("typed record path remains present in its indexed XML tree");
        let identity_name =
            document_object_attribute_name(&mut indexed.xml.tree, indexed.xml.document);
        let identity = match indexed.xml.tree.get_attribute(node, identity_name) {
            Some(value) => DocumentObjectIdV1::parse(value.to_owned()).map_err(|_| {
                TypedDocumentError::InvalidPersistedDocumentObjectId {
                    location: location.clone(),
                }
            })?,
            None => allocate_unique(allocator, seen, location.clone())?,
        };
        if let Some(first) = seen.insert(identity.clone(), location.clone()) {
            return Err(TypedDocumentError::DuplicatePersistedDocumentObjectId {
                first,
                duplicate: location,
            });
        }
        indexed
            .xml
            .tree
            .set_attribute(node, identity_name, identity.as_str());
        indexed.index_document_object_id_v1(identity, record.path().clone());
    }
    for child in record.typed_children() {
        normalize_record(indexed, child.record(), allocator, seen)?;
    }
    Ok(())
}

fn allocate_unique(
    allocator: &mut impl DocumentObjectIdAllocatorV1,
    seen: &HashMap<DocumentObjectIdV1, DocumentLocationV1>,
    location: DocumentLocationV1,
) -> Result<DocumentObjectIdV1, TypedDocumentError> {
    for _ in 0..MAX_DOCUMENT_OBJECT_ALLOCATION_ATTEMPTS_V1 {
        let identity = allocator
            .allocate()
            .map_err(TypedDocumentError::DocumentObjectIdEntropy)?;
        if !seen.contains_key(&identity) {
            return Ok(identity);
        }
    }
    Err(TypedDocumentError::DocumentObjectIdAllocationExhausted { location })
}

pub(crate) fn is_document_object_attribute_v1(namespace: &str, local_name: &str) -> bool {
    namespace == DOCUMENT_OBJECT_NAMESPACE_V1 && local_name == DOCUMENT_OBJECT_ATTRIBUTE_V1
}

fn document_object_attribute_name(tree: &mut Xot, document: Node) -> xot::NameId {
    let namespace = tree.add_namespace(DOCUMENT_OBJECT_NAMESPACE_V1);
    let prefix = tree.add_prefix(DOCUMENT_OBJECT_PREFIX_V1);
    let root = tree
        .document_element(document)
        .expect("an indexed document always has a CDML root element");
    tree.set_namespace(root, prefix, namespace);
    tree.add_name_ns(DOCUMENT_OBJECT_ATTRIBUTE_V1, namespace)
}

fn node_at_path(tree: &Xot, document: Node, path: &[u32]) -> Option<Node> {
    let mut node = tree.document_element(document).ok()?;
    for component in path {
        node = tree
            .children(node)
            .filter(|child| super::element_name(tree, *child).is_some())
            .nth(*component as usize)?;
    }
    Some(node)
}

fn is_addressable_record(record: &TypedRecord) -> bool {
    match record.class() {
        TypedClass::Molecule
        | TypedClass::Atom
        | TypedClass::CompactGroup
        | TypedClass::Group
        | TypedClass::MoleculeText
        | TypedClass::Query
        | TypedClass::Bond
        | TypedClass::Fragment => true,
        TypedClass::CanvasArrow
        | TypedClass::CanvasPlus
        | TypedClass::CanvasText
        | TypedClass::Rectangle
        | TypedClass::Square
        | TypedClass::Oval
        | TypedClass::Circle
        | TypedClass::Polygon
        | TypedClass::Polyline
        | TypedClass::Reaction => record.path().components().len() == 1,
        _ => false,
    }
}

pub(crate) fn location_for_record_v1(record: &TypedRecord) -> DocumentLocationV1 {
    let path = record.path().components();
    let root_ordinal = path
        .first()
        .copied()
        .expect("addressable typed record occurs below one direct CDML root");
    let kind = if path.len() == 1
        && matches!(
            record.class(),
            TypedClass::CanvasArrow
                | TypedClass::CanvasPlus
                | TypedClass::CanvasText
                | TypedClass::Rectangle
                | TypedClass::Square
                | TypedClass::Oval
                | TypedClass::Circle
                | TypedClass::Polygon
                | TypedClass::Polyline
        ) {
        DocumentLocationKindV1::Presentation
    } else {
        DocumentLocationKindV1::Structural
    };
    DocumentLocationV1::try_new(kind, root_ordinal, path[1..].to_vec())
        .expect("addressable typed records have bounded structural paths")
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;
    use crate::{IndexedDocument, TypedDocument};

    const TWO_ROOT_DOCUMENT: &str = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\">",
        "<molecule id=\"molecule\"><atom id=\"atom\" name=\"C\">",
        "<point x=\"0\" y=\"0\"/></atom></molecule>",
        "<plus id=\"plus\"><point x=\"0\" y=\"0\"/></plus>",
        "</cdml>",
    );

    struct SequenceAllocatorV1 {
        identities: VecDeque<DocumentObjectIdV1>,
        calls: usize,
    }

    impl SequenceAllocatorV1 {
        fn new(identities: Vec<DocumentObjectIdV1>) -> Self {
            Self {
                identities: identities.into(),
                calls: 0,
            }
        }
    }

    impl DocumentObjectIdAllocatorV1 for SequenceAllocatorV1 {
        fn allocate(&mut self) -> Result<DocumentObjectIdV1, getrandom::Error> {
            self.calls += 1;
            Ok(self
                .identities
                .pop_front()
                .expect("test sequence supplies every requested identity"))
        }
    }

    fn deterministic_identity(sequence: u8) -> DocumentObjectIdV1 {
        DocumentObjectIdV1::from_entropy_bytes([sequence; 16])
    }

    fn normalize_with_allocator(
        allocator: &mut impl DocumentObjectIdAllocatorV1,
    ) -> Result<TypedDocument, TypedDocumentError> {
        let root = TypedDocument::parse(TWO_ROOT_DOCUMENT)
            .expect("fixture must establish a typed root")
            .root()
            .clone();
        let mut indexed = IndexedDocument::parse(TWO_ROOT_DOCUMENT)
            .expect("fixture must establish an indexed document");
        normalize_document_object_ids_with_allocator_v1(&mut indexed, &root, allocator)?;
        TypedDocument::from_indexed(indexed)
    }

    #[test]
    fn deterministic_allocator_retries_a_collision_before_admitting_unique_selectors() {
        let first = deterministic_identity(1);
        let second = deterministic_identity(2);
        let third = deterministic_identity(3);
        let mut allocator = SequenceAllocatorV1::new(vec![
            first.clone(),
            first.clone(),
            second.clone(),
            third.clone(),
        ]);

        let document = normalize_with_allocator(&mut allocator)
            .expect("a retry with a fresh selector must admit the candidate");

        assert_eq!(allocator.calls, 4);
        assert_eq!(
            document
                .resolve_document_object_id(&first)
                .map(TypedRecord::class),
            Some(TypedClass::Molecule)
        );
        assert_eq!(
            document
                .resolve_document_object_id(&second)
                .map(TypedRecord::class),
            Some(TypedClass::Atom)
        );
        assert_eq!(
            document
                .resolve_document_object_id(&third)
                .map(TypedRecord::class),
            Some(TypedClass::CanvasPlus)
        );
    }

    #[test]
    fn exhausted_allocator_refuses_the_candidate_without_an_admitted_document() {
        let collision = deterministic_identity(3);
        let mut allocator = SequenceAllocatorV1::new(vec![
            collision;
            MAX_DOCUMENT_OBJECT_ALLOCATION_ATTEMPTS_V1
                + 1
        ]);

        let candidate = normalize_with_allocator(&mut allocator);

        assert!(matches!(
            candidate,
            Err(TypedDocumentError::DocumentObjectIdAllocationExhausted { .. })
        ));
        assert_eq!(
            allocator.calls,
            MAX_DOCUMENT_OBJECT_ALLOCATION_ATTEMPTS_V1 + 1
        );
    }
}

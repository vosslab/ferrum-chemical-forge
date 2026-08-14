use xot::{Node, Value, Xot};

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
            Shape::ProcessingInstruction {
                target: format!("{{{namespace}}}{local_name}"),
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
  <info id="header"/><molecule id="m1"><atom id="a1"/></molecule><arrow id="arrow-1"/>
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
            ref first, ref duplicate, ..
        }) if first.components() == [0] && duplicate.components() == [1]
    ));
    assert!(matches!(
        IndexedDocument::parse(r#"<cdml><molecule id=" "/></cdml>"#),
        Err(super::IndexedDocumentError::Identity(
            DocumentIdentityError::BlankPersistentId
        ))
    ));
}

#[test]
fn provisional_tokens_are_distinct_from_persistent_ids_and_consumed_once() {
    let mut document =
        IndexedDocument::parse(r#"<cdml><molecule id="m1"/></cdml>"#).expect("valid CDML index");
    let token = document
        .try_issue_provisional_token()
        .expect("token registry reserves");
    let replay = token.clone();
    document
        .consume_provisional_token(&token)
        .expect("first consumption succeeds");
    assert!(matches!(
        document.consume_provisional_token(&replay),
        Err(DocumentIdentityError::ConsumedProvisionalToken { .. })
    ));
    let mut foreign_document = IndexedDocument::parse("<cdml/>").expect("valid foreign document");
    let foreign = foreign_document
        .try_issue_provisional_token()
        .expect("token registry reserves");
    assert!(matches!(
        document.consume_provisional_token(&foreign),
        Err(DocumentIdentityError::UnknownProvisionalToken { .. })
    ));
    let durable = PersistentId::new("m1").expect("nonblank persistent id");
    assert!(document.resolve_id(&durable).is_some());
}

#[test]
fn provisional_tokens_with_matching_sequences_cannot_cross_documents() {
    let mut first = IndexedDocument::parse("<cdml/>").expect("first valid CDML index");
    let mut second = IndexedDocument::parse("<cdml/>").expect("second valid CDML index");
    let first_token = first
        .try_issue_provisional_token()
        .expect("registry reserves");
    let second_token = second
        .try_issue_provisional_token()
        .expect("registry reserves");
    assert_ne!(first_token, second_token);
    assert!(matches!(
        second.consume_provisional_token(&first_token),
        Err(DocumentIdentityError::UnknownProvisionalToken { .. })
    ));
    second
        .consume_provisional_token(&second_token)
        .expect("own token is consumable");
    first
        .consume_provisional_token(&first_token)
        .expect("own token remains consumable");
}

#[test]
fn root_id_reserves_the_document_wide_collision_name() {
    let root_identifier = PersistentId::new("document").expect("nonblank id");
    let indexed = IndexedDocument::parse(r#"<cdml id="document"><molecule/></cdml>"#)
        .expect("root id indexes");
    assert_eq!(
        indexed
            .resolve_id(&root_identifier)
            .unwrap()
            .path()
            .components(),
        &[] as &[u32]
    );
    assert_eq!(
        indexed.resolve_id(&root_identifier).unwrap().source_order(),
        None
    );
    assert!(indexed.records()[0].identifier().is_none());
    let duplicate = r#"<cdml id="document"><molecule id="document"/></cdml>"#;
    assert!(matches!(
        IndexedDocument::parse(duplicate),
        Err(super::IndexedDocumentError::Identity(
            DocumentIdentityError::DuplicatePersistentId { ref first, ref duplicate, .. }
        )) if first.components().is_empty() && duplicate.components() == [0]
    ));
    assert!(matches!(
        IndexedDocument::parse(r#"<cdml id=" "/>"#),
        Err(super::IndexedDocumentError::Identity(
            DocumentIdentityError::BlankPersistentId
        ))
    ));
}

#[test]
fn opaque_reference_looking_values_are_reserved_but_never_rewritten() {
    let source = r#"<cdml xmlns="http://www.freesoftware.fsf.org/bkchem/cdml" xmlns:v="urn:vendor"><molecule id="m1"/><v:extension id="opaque-id" idref="m1">reference m1 <v:item start="m1">m1</v:item></v:extension></cdml>"#;
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
    let source = r#"<cdml><molecule id="m1"><atom id="a1"/><bond id="b1"/><fragment id="f1"><bond id="b1"/><vertex id="a1"/></fragment></molecule></cdml>"#;
    let document = IndexedDocument::parse(source).expect("fragment references are not ids");
    assert_eq!(document.persistent_id_count(), 4);
    let bond = PersistentId::new("b1").expect("nonblank id");
    assert_eq!(
        document.resolve_id(&bond).unwrap().path().components(),
        &[0, 1]
    );
}

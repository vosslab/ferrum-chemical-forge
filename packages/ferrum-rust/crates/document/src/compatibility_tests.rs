use super::{PersistentId, TypedClass, TypedDocument};

const CDML_NAMESPACE: &str = "http://www.freesoftware.fsf.org/bkchem/cdml";

fn structural_facts(document: &TypedDocument) -> Vec<(u32, Option<String>, Vec<u32>)> {
    document
        .indexed()
        .records()
        .iter()
        .map(|record| {
            (
                record.source_order().value(),
                record
                    .identifier()
                    .map(|identifier| identifier.as_str().to_owned()),
                record.path().components().to_vec(),
            )
        })
        .collect()
}

fn round_trip(source: &str) -> TypedDocument {
    let document = TypedDocument::parse(source).expect("defined CDML compatibility input parses");
    let facts = structural_facts(&document);
    let serialized = document.to_xml().expect("retained CDML serializes");
    let reparsed = TypedDocument::parse(&serialized).expect("serialized CDML reparses");

    assert_eq!(structural_facts(&reparsed), facts);
    reparsed
}

#[test]
fn legacy_no_namespace_cdml_preserves_structure_order_and_persistent_ids() {
    let document = round_trip(
        r#"<cdml version="26.02"><paper id="paper-1"/><molecule id="m1"><atom id="a1"/></molecule><arrow id="arrow-1"/></cdml>"#,
    );

    assert_eq!(document.root().class(), TypedClass::Cdml);
    assert_eq!(document.root().attribute("version"), Some("26.02"));
    assert_eq!(
        structural_facts(&document),
        vec![
            (0, Some("paper-1".to_owned()), vec![0]),
            (1, Some("m1".to_owned()), vec![1]),
            (2, Some("arrow-1".to_owned()), vec![2]),
        ]
    );
    let atom = PersistentId::new("a1").expect("nonblank source id");
    assert_eq!(
        document
            .indexed()
            .resolve_id(&atom)
            .expect("nested persistent id remains indexed")
            .path()
            .components(),
        &[1, 0]
    );
}

#[test]
fn unknown_root_version_spelling_survives_canonical_cdml_round_trip() {
    let document = round_trip(&format!(
        r#"<cdml xmlns="{CDML_NAMESPACE}" version="99.99"><paper id="paper-1"/></cdml>"#
    ));

    assert_eq!(document.root().class(), TypedClass::Cdml);
    assert_eq!(document.root().attribute("version"), Some("99.99"));
}

#[test]
fn alternate_canonical_prefix_uses_expanded_names_for_projection_and_structure() {
    let document = round_trip(&format!(
        r#"<c:cdml xmlns:c="{CDML_NAMESPACE}" version="26.07"><c:paper id="paper-1"/><c:molecule id="m1"><c:atom id="a1"/></c:molecule></c:cdml>"#
    ));

    assert_eq!(document.root().class(), TypedClass::Cdml);
    assert_eq!(document.root().attribute("version"), Some("26.07"));
    assert_eq!(document.root().typed_children().len(), 2);
    assert_eq!(
        document
            .root()
            .typed_children()
            .iter()
            .map(|child| child.record().class())
            .collect::<Vec<_>>(),
        vec![TypedClass::Paper, TypedClass::Molecule]
    );
}

//! Core source-identity and structural validation behavior tests.

use super::super::*;

fn id(value: &str) -> Identifier {
    Identifier::new(value).expect("test source ID is nonblank")
}

fn atom(value: &str, x: f64) -> Atom {
    Atom::new(
        id(value),
        Some("C".to_owned()),
        Position::new(x, 0.0, 0.0).expect("finite"),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("valid atom")
}

#[test]
fn source_ids_produce_deterministic_kind_scoped_identities() {
    let first = atom("a", 0.0);
    let second = atom("a", 1.0);
    let bond_id = RecordId::new(RecordKind::Bond, id("a")).expect("valid record ID");
    assert_eq!(first.identity(), second.identity());
    assert_ne!(first.identity(), &bond_id);
    assert_eq!(first.identity().source_id().as_str(), "a");
}

#[test]
fn identifier_and_serde_refuse_blank_or_legacy_identity_shapes() {
    assert!(Identifier::new(" \t ").is_err());
    assert!(
        serde_json::from_value::<RecordId>(serde_json::json!({"kind":"Atom", "source_id":""}))
            .is_err()
    );
    assert!(
        serde_json::from_value::<RecordId>(
            serde_json::json!({"kind":"Atom", "origin":{"Source":"a"}})
        )
        .is_err()
    );
}

#[test]
fn serde_refuses_spoofed_record_kind_or_source() {
    let atom = atom("a", 0.0);
    let mut wrong_source = serde_json::to_value(&atom).expect("serialize");
    wrong_source["identity"]["source_id"] = serde_json::json!("other");
    assert!(serde_json::from_value::<Atom>(wrong_source).is_err());
    let mut wrong_kind = serde_json::to_value(&atom).expect("serialize");
    wrong_kind["identity"]["kind"] = serde_json::json!("Bond");
    assert!(serde_json::from_value::<Atom>(wrong_kind).is_err());
}

#[test]
fn vertex_reference_serde_refuses_kind_spoofing() {
    let atom = atom("a", 0.0);
    let spoof = serde_json::json!({"Group": atom.identity()});
    assert!(serde_json::from_value::<VertexRef>(spoof).is_err());
}

#[test]
fn replacement_retains_source_identity_and_validates_fields() {
    let atom = atom("a", 0.0);
    let replacement = atom
        .replace_source_fields(
            Some("N".to_owned()),
            Position::new(1.0, 0.0, 0.0).expect("finite"),
            Some(1),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("valid replacement");
    assert_eq!(atom.identity(), replacement.identity());
    assert_eq!(replacement.formal_charge(), Some(1));
}

#[test]
fn molecule_rejects_duplicate_source_ids_and_unresolved_endpoints() {
    let first = atom("a", 0.0);
    let duplicate = atom("a", 1.0);
    assert!(matches!(
        Molecule::new(
            id("m"),
            None,
            vec![first, duplicate],
            vec![],
            vec![],
            vec![],
            vec![]
        ),
        Err(ModelError::DuplicateIdentity)
    ));
    let left = atom("left", 0.0);
    let outside = atom("outside", 1.0);
    let bond = Bond::new(
        id("b"),
        VertexRef::Atom(left.identity().clone()),
        VertexRef::Atom(outside.identity().clone()),
        None,
        None,
        None,
        None,
    )
    .expect("bond structure is valid");
    assert!(matches!(
        Molecule::new(
            id("m2"),
            None,
            vec![left],
            vec![],
            vec![],
            vec![],
            vec![bond]
        ),
        Err(ModelError::UnresolvedBondEndpoint)
    ));
}

#[test]
fn molecule_rejects_child_source_id_that_duplicates_its_own_source_id() {
    assert!(matches!(
        Molecule::new(
            id("molecule"),
            None,
            vec![atom("molecule", 0.0)],
            vec![],
            vec![],
            vec![],
            vec![]
        ),
        Err(ModelError::DuplicateSourceId)
    ));
}

use super::*;
use proptest::prelude::*;
fn source(value: &str) -> Identifier {
    Identifier::new(value).expect("test id is valid")
}
fn atom(id: Option<&str>, occurrence: Option<u32>, x: f64) -> Atom {
    Atom::new(
        id.map(source),
        Some("C".to_owned()),
        Position::new(x, 0.0, 0.0).expect("finite"),
        None,
        None,
        None,
        None,
        None,
        None,
        occurrence,
    )
    .expect("valid atom")
}
fn vertex(kind: RecordKind, id: Option<&str>, occurrence: Option<u32>) -> NonAtomVertex {
    NonAtomVertex::new(kind, id.map(source), occurrence).expect("valid vertex")
}
fn bond(start: VertexRef, end: VertexRef, occurrence: Option<u32>) -> Bond {
    Bond::new(None, start, end, None, None, None, None, occurrence).expect("valid bond")
}
fn molecule(
    atoms: Vec<Atom>,
    groups: Vec<NonAtomVertex>,
    texts: Vec<NonAtomVertex>,
    queries: Vec<NonAtomVertex>,
    bonds: Vec<Bond>,
) -> Result<Molecule, ModelError> {
    Molecule::new(
        Some(source("m")),
        None,
        atoms,
        groups,
        texts,
        queries,
        bonds,
        None,
    )
}

#[test]
fn serde_rejects_spoofed_source_identity() {
    let atom = atom(Some("a"), None, 0.0);
    let mut json = serde_json::to_value(&atom).expect("serialize");
    json["identity"]["origin"]["Source"] = serde_json::json!("other");
    assert!(serde_json::from_value::<Atom>(json).is_err());

    let mut json = serde_json::to_value(&atom).expect("serialize");
    json["identity"]["kind"] = serde_json::json!("Bond");
    assert!(serde_json::from_value::<Atom>(json).is_err());
}
#[test]
fn bond_rejects_every_wrong_vertex_reference_kind() {
    let atom = atom(Some("a"), None, 0.0);
    let correct = VertexRef::Atom(atom.identity().clone());
    for wrong in [
        VertexRef::Group(atom.identity().clone()),
        VertexRef::Text(atom.identity().clone()),
        VertexRef::Query(atom.identity().clone()),
    ] {
        let result = Bond::new(
            Some(source("b")),
            correct.clone(),
            wrong,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(matches!(result, Err(ModelError::VertexKindMismatch)));
    }
}
#[test]
fn canonical_encoding_separates_delimiter_inputs() {
    assert_ne!(
        LegacyFingerprint::test_encoding(RecordKind::Atom, &["a:b".to_owned(), "c".to_owned()]),
        LegacyFingerprint::test_encoding(RecordKind::Atom, &["a".to_owned(), "b:c".to_owned()])
    );
}
#[test]
fn exact_idless_duplicates_have_session_occurrences() {
    let first = atom(None, Some(0), 0.0);
    let second = atom(None, Some(1), 0.0);
    assert_ne!(first.identity(), second.identity());
    let model = molecule(vec![first, second], vec![], vec![], vec![], vec![])
        .expect("exact idless duplicates load with occurrences");
    assert_eq!(
        serde_json::from_str::<Molecule>(&serde_json::to_string(&model).expect("serialize"))
            .expect("deserialize"),
        model
    );
}
#[test]
fn nonfinite_position_and_missing_bond_facts_reject_or_preserve() {
    assert!(Position::new(f64::NAN, 0.0, 0.0).is_err());
    let left = atom(Some("a"), None, 0.0);
    let right = atom(Some("b"), None, 1.0);
    let bond = bond(
        VertexRef::Atom(left.identity().clone()),
        VertexRef::Atom(right.identity().clone()),
        Some(0),
    );
    assert!(bond.source_type().is_none());
    assert!(bond.order().is_none());
    assert!(bond.style().is_none());
}
#[test]
fn idless_bond_identity_includes_ordered_endpoint_identities() {
    let left = atom(None, Some(0), 0.0);
    let middle = atom(None, Some(0), 1.0);
    let right = atom(None, Some(0), 2.0);
    let first = bond(
        VertexRef::Atom(left.identity().clone()),
        VertexRef::Atom(middle.identity().clone()),
        Some(0),
    );
    let second = bond(
        VertexRef::Atom(left.identity().clone()),
        VertexRef::Atom(right.identity().clone()),
        Some(0),
    );
    assert_ne!(first.identity(), second.identity());
}
#[test]
fn idless_molecule_anchor_includes_sorted_child_identities() {
    let left = atom(None, Some(0), 0.0);
    let right = atom(None, Some(0), 1.0);
    let reordered = Molecule::new(
        None,
        Some("same".to_owned()),
        vec![right.clone(), left.clone()],
        vec![],
        vec![],
        vec![],
        vec![],
        Some(0),
    )
    .expect("valid idless molecule");
    let original = Molecule::new(
        None,
        Some("same".to_owned()),
        vec![left.clone(), right.clone()],
        vec![],
        vec![],
        vec![],
        vec![],
        Some(0),
    )
    .expect("valid idless molecule");
    let distinct = Molecule::new(
        None,
        Some("same".to_owned()),
        vec![left],
        vec![],
        vec![],
        vec![],
        vec![],
        Some(0),
    )
    .expect("different child set is valid");
    assert_eq!(original.identity(), reordered.identity());
    assert_ne!(original.identity(), distinct.identity());
    let replacement = original
        .replace_records(
            Some("edited".to_owned()),
            vec![right],
            vec![],
            vec![],
            vec![],
            vec![],
        )
        .expect("replacement remains valid");
    assert_eq!(original.identity(), replacement.identity());
}
#[test]
fn edited_idless_anchor_rehydrates_without_reseeding() {
    let first = atom(None, Some(0), 0.0);
    let second = atom(None, Some(0), 1.0);
    let edited_atom = first
        .replace_source_fields(
            Some("N".to_owned()),
            Position::new(2.0, 0.0, 0.0).expect("finite"),
            Some(1),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("replacement is valid");
    let edited_bond = bond(
        VertexRef::Atom(first.identity().clone()),
        VertexRef::Atom(second.identity().clone()),
        Some(0),
    )
    .replace_source_fields(
        VertexRef::Atom(first.identity().clone()),
        VertexRef::Atom(second.identity().clone()),
        Some("n1".to_owned()),
        Some(BondOrder::Single),
        Some(BondStyle::Normal),
        None,
    )
    .expect("replacement is valid");
    let original = Molecule::new(
        None,
        Some("before".to_owned()),
        vec![first, second],
        vec![],
        vec![],
        vec![],
        vec![],
        Some(0),
    )
    .expect("valid idless molecule");
    let edited_molecule = original
        .replace_records(
            Some("after".to_owned()),
            vec![edited_atom.clone()],
            vec![],
            vec![],
            vec![],
            vec![],
        )
        .expect("replacement is valid");

    let restored_atom: Atom =
        serde_json::from_str(&serde_json::to_string(&edited_atom).expect("serialize"))
            .expect("rehydrate");
    assert_eq!(restored_atom, edited_atom);
    let restored_bond: Bond =
        serde_json::from_str(&serde_json::to_string(&edited_bond).expect("serialize"))
            .expect("rehydrate");
    assert_eq!(restored_bond, edited_bond);
    let restored_molecule: Molecule =
        serde_json::from_str(&serde_json::to_string(&edited_molecule).expect("serialize"))
            .expect("rehydrate");
    assert_eq!(restored_molecule, edited_molecule);
}
#[test]
fn serde_rejects_spoofed_legacy_occurrence() {
    let atom = atom(None, Some(0), 0.0);
    let mut json = serde_json::to_value(&atom).expect("serialize");
    json["legacy_occurrence"] = serde_json::json!(1);
    assert!(serde_json::from_value::<Atom>(json).is_err());
}
#[test]
fn serde_rejects_malformed_or_wrong_kind_legacy_anchors_everywhere() {
    let first_atom = atom(None, Some(0), 0.0);
    let other = atom(None, Some(0), 1.0);
    let bond = bond(
        VertexRef::Atom(first_atom.identity().clone()),
        VertexRef::Atom(other.identity().clone()),
        Some(0),
    );
    let molecule = Molecule::new(
        None,
        None,
        vec![first_atom.clone()],
        vec![],
        vec![],
        vec![],
        vec![],
        Some(0),
    )
    .expect("valid molecule");
    for bad in [
        "wrong-version",
        "ferrum-core-legacy-v1:Bond:1:x",
        "ferrum-core-legacy-v1:Atom:1:x:garbage",
    ] {
        let mut atom_json = serde_json::to_value(&first_atom).expect("serialize");
        atom_json["identity"]["origin"]["Legacy"]["fingerprint"] = serde_json::json!(bad);
        assert!(serde_json::from_value::<Atom>(atom_json).is_err());
        let mut bond_json = serde_json::to_value(&bond).expect("serialize");
        bond_json["identity"]["origin"]["Legacy"]["fingerprint"] = serde_json::json!(bad);
        assert!(serde_json::from_value::<Bond>(bond_json).is_err());
        let mut molecule_json = serde_json::to_value(&molecule).expect("serialize");
        molecule_json["identity"]["origin"]["Legacy"]["fingerprint"] = serde_json::json!(bad);
        assert!(serde_json::from_value::<Molecule>(molecule_json).is_err());
    }
    let wrong_kind = LegacyFingerprint::test_encoding(RecordKind::Bond, &vec!["x".to_owned(); 7]).0;
    let mut atom_json = serde_json::to_value(&first_atom).expect("serialize");
    atom_json["identity"]["origin"]["Legacy"]["fingerprint"] = serde_json::json!(wrong_kind);
    assert!(serde_json::from_value::<Atom>(atom_json).is_err());
    let mut nested = serde_json::to_value(&molecule).expect("serialize");
    nested["atoms"][0]["identity"]["origin"]["Legacy"]["fingerprint"] =
        serde_json::json!("ferrum-core-legacy-v1:Bond:1:x");
    assert!(serde_json::from_value::<Molecule>(nested).is_err());
}
#[test]
fn standalone_record_and_vertex_deserialization_enforce_kinds() {
    let legacy_atom = atom(None, Some(0), 0.0).identity().clone();
    let source_atom = atom(Some("source"), None, 0.0).identity().clone();
    for identity in [&legacy_atom, &source_atom] {
        let restored: RecordId =
            serde_json::from_str(&serde_json::to_string(identity).expect("serialize"))
                .expect("valid record identity");
        assert_eq!(restored, *identity);
    }
    let valid_vertex = VertexRef::Atom(legacy_atom.clone());
    let restored: VertexRef =
        serde_json::from_str(&serde_json::to_string(&valid_vertex).expect("serialize"))
            .expect("valid vertex reference");
    assert_eq!(restored, valid_vertex);

    let wrong_fingerprint =
        LegacyFingerprint::test_encoding(RecordKind::Bond, &vec!["x".to_owned(); 7]).0;
    let wrong_record = serde_json::json!({"kind":"Atom", "origin":{"Legacy":{"fingerprint":wrong_fingerprint, "occurrence":0}}});
    assert!(serde_json::from_value::<RecordId>(wrong_record).is_err());
    for (variant, kind, fields) in [
        ("Atom", RecordKind::Bond, 7usize),
        ("Group", RecordKind::Atom, 9usize),
        ("Text", RecordKind::Atom, 9usize),
        ("Query", RecordKind::Atom, 9usize),
    ] {
        let fingerprint = LegacyFingerprint::test_encoding(kind, &vec!["x".to_owned(); fields]).0;
        let identity = serde_json::json!({"kind": format!("{kind:?}"), "origin":{"Legacy":{"fingerprint":fingerprint, "occurrence":0}}});
        assert!(
            serde_json::from_value::<VertexRef>(serde_json::json!({variant: identity})).is_err()
        );
    }
}
proptest! {
    #[test] fn nonidentical_idless_identity_is_reorder_independent(x in -9999i64..9999, y in -9999i64..9999) { prop_assume!(x != y); let first = atom(None, Some(0), x as f64); let second = atom(None, Some(0), y as f64); let forward = molecule(vec![first.clone(), second.clone()], vec![], vec![], vec![], vec![]).expect("valid"); let reverse = molecule(vec![second, first], vec![], vec![], vec![], vec![]).expect("valid"); prop_assert!(forward.atoms().iter().all(|a| reverse.atoms().iter().any(|b| a.identity() == b.identity()))); }
    #[test]
    fn carried_optional_scalars_keep_absence_distinct_from_present_default(
        charge in proptest::option::weighted(0.5, -8i32..8),
        isotope in proptest::option::weighted(0.5, 1u16..300),
        hydrogens in proptest::option::weighted(0.5, 0u16..8),
        valence in proptest::option::weighted(0.5, 0u16..8),
        multiplicity in proptest::option::weighted(0.5, 1u16..4),
        free_sites in proptest::option::weighted(0.5, 0u16..8),
        order in proptest::option::weighted(0.5, prop_oneof![
            Just(BondOrder::Single),
            Just(BondOrder::Double),
            Just(BondOrder::Triple),
            Just(BondOrder::Aromatic),
            (0u8..4).prop_map(BondOrder::Other),
        ]),
        style in proptest::option::weighted(0.5, prop_oneof![
            Just(BondStyle::Normal),
            Just(BondStyle::Wedge),
            Just(BondStyle::Hashed),
            Just(BondStyle::Other("custom".to_owned())),
        ]),
        aromatic in proptest::option::weighted(0.5, any::<bool>()),
    ) {
        let origin = Position::new(0.0, 0.0, 0.0).expect("finite");
        let carrier = Atom::new(
            None, Some("C".to_owned()), origin, charge, isotope, hydrogens,
            valence, multiplicity, free_sites, Some(0),
        ).expect("valid atom");
        let partner = atom(None, Some(0), 1.0);
        let link = Bond::new(
            None,
            VertexRef::Atom(carrier.identity().clone()),
            VertexRef::Atom(partner.identity().clone()),
            None, order, style.clone(), aromatic, Some(0),
        ).expect("valid bond");
        let model = molecule(
            vec![carrier.clone(), partner.clone()], vec![], vec![], vec![], vec![link.clone()],
        ).expect("valid molecule");
        let restored: Molecule =
            serde_json::from_str(&serde_json::to_string(&model).expect("serialize"))
                .expect("deserialize");

        // A round trip reproduces each carried option exactly, absence included.
        let restored_atom = &restored.atoms()[0];
        prop_assert_eq!(restored_atom.formal_charge(), charge);
        prop_assert_eq!(restored_atom.isotope(), isotope);
        prop_assert_eq!(restored_atom.explicit_hydrogens(), hydrogens);
        prop_assert_eq!(restored_atom.valence(), valence);
        prop_assert_eq!(restored_atom.multiplicity(), multiplicity);
        prop_assert_eq!(restored_atom.free_sites(), free_sites);
        prop_assert_eq!(restored_atom.identity(), carrier.identity());
        let restored_bond = &restored.bonds()[0];
        prop_assert_eq!(restored_bond.order(), order);
        prop_assert_eq!(restored_bond.style(), style.as_ref());
        prop_assert_eq!(restored_bond.aromatic(), aromatic);
        prop_assert_eq!(restored_bond.identity(), link.identity());

        // The same record with every absence filled by a present default is a
        // different record, and stays different across the same round trip.
        let filled_atom = Atom::new(
            None, Some("C".to_owned()), origin,
            Some(charge.unwrap_or(0)), Some(isotope.unwrap_or(0)),
            Some(hydrogens.unwrap_or(0)), Some(valence.unwrap_or(0)),
            Some(multiplicity.unwrap_or(1)), Some(free_sites.unwrap_or(0)), Some(0),
        ).expect("valid atom");
        // The atom and the bond vary in separate molecules, so a changed atom
        // identity cannot be what makes the compared bond identities differ.
        let filled_atom_model = molecule(
            vec![filled_atom, partner.clone()], vec![], vec![], vec![], vec![],
        ).expect("valid molecule");
        let restored_filled: Molecule =
            serde_json::from_str(&serde_json::to_string(&filled_atom_model).expect("serialize"))
                .expect("deserialize");
        let restored_filled_atom = &restored_filled.atoms()[0];
        prop_assert!(restored_filled_atom.formal_charge().is_some());
        prop_assert!(restored_filled_atom.isotope().is_some());
        prop_assert!(restored_filled_atom.explicit_hydrogens().is_some());
        prop_assert!(restored_filled_atom.valence().is_some());
        prop_assert!(restored_filled_atom.multiplicity().is_some());
        prop_assert!(restored_filled_atom.free_sites().is_some());

        let filled_bond = Bond::new(
            None,
            VertexRef::Atom(carrier.identity().clone()),
            VertexRef::Atom(partner.identity().clone()),
            None,
            Some(order.unwrap_or(BondOrder::Single)),
            Some(style.clone().unwrap_or(BondStyle::Normal)),
            Some(aromatic.unwrap_or(false)),
            Some(0),
        ).expect("valid bond");
        let filled_bond_model = molecule(
            vec![carrier, partner], vec![], vec![], vec![], vec![filled_bond],
        ).expect("valid molecule");
        let restored_filled_bonds: Molecule =
            serde_json::from_str(&serde_json::to_string(&filled_bond_model).expect("serialize"))
                .expect("deserialize");
        let restored_filled_bond = &restored_filled_bonds.bonds()[0];
        prop_assert!(restored_filled_bond.order().is_some());
        prop_assert!(restored_filled_bond.style().is_some());
        prop_assert!(restored_filled_bond.aromatic().is_some());
        let atom_absence = charge.is_none() || isotope.is_none() || hydrogens.is_none()
            || valence.is_none() || multiplicity.is_none() || free_sites.is_none();
        if atom_absence {
            prop_assert_ne!(restored_filled_atom.identity(), restored_atom.identity());
        }
        if order.is_none() || style.is_none() || aromatic.is_none() {
            prop_assert_ne!(restored_filled_bond.identity(), restored_bond.identity());
        }
    }
    #[test] fn endpoint_and_source_absence_round_trip(x in -9999i64..9999) { let a = atom(None, Some(0), x as f64); let g = vertex(RecordKind::Group, None, Some(0)); let q = vertex(RecordKind::Query, None, Some(0)); let b = bond(VertexRef::Group(g.identity().clone()), VertexRef::Query(q.identity().clone()), Some(0)); let model = molecule(vec![a], vec![g], vec![], vec![q], vec![b]).expect("typed endpoints resolve"); let restored: Molecule = serde_json::from_str(&serde_json::to_string(&model).expect("serialize")).expect("deserialize"); prop_assert!(restored.bonds()[0].source_id().is_none()); prop_assert!(!restored.bonds()[0].start().is_atom()); }
}

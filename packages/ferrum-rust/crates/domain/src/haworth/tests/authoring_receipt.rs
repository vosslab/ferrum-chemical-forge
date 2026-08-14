use ferrum_core::{
    Atom, Bond, BondOrder, BondStyle, Identifier, Molecule, NonAtomVertex, Position, RecordKind,
    VertexRef,
};

use crate::haworth::{
    AuthoredDirectGlycosidicHaworthBondRoleV1, DirectGlycosidicHaworthAuthoringAtomElementV1,
    DirectGlycosidicHaworthBondStyleV1, DirectGlycosidicHaworthTopologyV1, HaworthError,
    HaworthTopology, HaworthTopologyBuilder, HaworthVertex, RingForm,
    direct_glycosidic_haworth_authoring_receipt_v1,
};

fn atom(index: usize, element: &str, charge: Option<i32>) -> Atom {
    Atom::new(
        Some(Identifier::new(format!("a{index}")).expect("identifier")),
        Some(element.to_owned()),
        Position::new(index as f64, 0.0, 0.0).expect("position"),
        charge,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("atom")
}

fn bond(index: usize, start: &Atom, end: &Atom) -> Bond {
    Bond::new(
        Some(Identifier::new(format!("b{index}")).expect("identifier")),
        VertexRef::Atom(start.identity().clone()),
        VertexRef::Atom(end.identity().clone()),
        None,
        Some(BondOrder::Single),
        None,
        Some(false),
        None,
    )
    .expect("bond")
}

fn source(
    first_form: RingForm,
    second_form: RingForm,
    reverse_request: bool,
    charged: bool,
) -> (Molecule, DirectGlycosidicHaworthTopologyV1) {
    let first_count = first_form.vertex_count();
    let second_count = second_form.vertex_count();
    let bridge_index = first_count + second_count;
    let mut atoms: Vec<_> = (0..bridge_index)
        .map(|index| {
            atom(
                index,
                if index == 0 || index == first_count {
                    "O"
                } else {
                    "C"
                },
                (charged && index == 1).then_some(1),
            )
        })
        .collect();
    atoms.push(atom(bridge_index, "O", None));
    let mut bonds: Vec<_> = (0..first_count)
        .map(|index| bond(index, &atoms[index], &atoms[(index + 1) % first_count]))
        .collect();
    bonds.extend((0..second_count).map(|index| {
        let start = first_count + index;
        bond(
            first_count + index,
            &atoms[start],
            &atoms[first_count + (index + 1) % second_count],
        )
    }));
    let first_bridge = first_count + second_count;
    bonds.push(bond(first_bridge, &atoms[1], &atoms[bridge_index]));
    bonds.push(bond(
        first_bridge + 1,
        &atoms[first_count + 1],
        &atoms[bridge_index],
    ));
    let molecule = Molecule::new(
        Some(Identifier::new("closed-two-rings").expect("identifier")),
        None,
        atoms.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        bonds.clone(),
        None,
    )
    .expect("molecule");
    let ring = |offset: usize, form: RingForm| {
        let vertices: Vec<_> = atoms[offset..offset + form.vertex_count()]
            .iter()
            .map(|atom| HaworthVertex {
                atom: atom.identity().clone(),
            })
            .collect();
        HaworthTopologyBuilder::new(form, vertices[1].atom.clone(), vertices)
            .build(&molecule)
            .expect("ring")
    };
    let rings: [HaworthTopology; 2] = [ring(0, first_form), ring(first_count, second_form)];
    let bridge_bonds = [
        bonds[first_bridge].identity().clone(),
        bonds[first_bridge + 1].identity().clone(),
    ];
    let topology = if reverse_request {
        DirectGlycosidicHaworthTopologyV1::classify(
            &molecule,
            [rings[1].clone(), rings[0].clone()],
            atoms[bridge_index].identity().clone(),
            [bridge_bonds[1].clone(), bridge_bonds[0].clone()],
        )
    } else {
        DirectGlycosidicHaworthTopologyV1::classify(
            &molecule,
            rings,
            atoms[bridge_index].identity().clone(),
            bridge_bonds,
        )
    }
    .expect("topology");
    (molecule, topology)
}

fn durable(kind: RecordKind, value: &str) -> ferrum_core::RecordId {
    ferrum_core::RecordId::from_source(kind, &Identifier::new(value).expect("identifier"))
}

#[test]
fn authored_depiction_rebinds_canonical_orders_and_rejects_bad_durable_inputs() {
    let (molecule, topology) = source(RingForm::Furanose, RingForm::Pyranose, true, false);
    let receipt = direct_glycosidic_haworth_authoring_receipt_v1(&molecule, topology.clone(), 7.0)
        .expect("receipt");
    let atoms: Vec<_> = (0..receipt.atoms_in_canonical_order().len())
        .map(|index| durable(RecordKind::Atom, &format!("z{index}")))
        .collect();
    let bonds: Vec<_> = (0..receipt.bonds_in_canonical_order().len())
        .map(|index| durable(RecordKind::Bond, &format!("a{index}")))
        .collect();
    let translation = crate::haworth::HaworthPoint { x: 3.0, y: -2.0 };
    let authored = receipt
        .authored_depiction_for_durable_commit_v1(&atoms, &bonds, translation)
        .expect("authored");
    let atom_for_source: std::collections::BTreeMap<_, _> = receipt
        .atoms_in_canonical_order()
        .iter()
        .zip(&atoms)
        .map(|(fact, durable)| (fact.source_atom_identity(), durable.clone()))
        .collect();
    assert_eq!(
        authored
            .canonical_atoms()
            .iter()
            .map(|fact| fact.atom())
            .collect::<Vec<_>>(),
        atoms.iter().collect::<Vec<_>>(),
        "canonical atoms retain supplied order even when their durable IDs sort differently"
    );
    assert_eq!(
        authored
            .canonical_atoms()
            .iter()
            .map(|fact| fact.authored_child_order())
            .collect::<Vec<_>>(),
        (0..atoms.len())
            .map(|index| index as u32)
            .collect::<Vec<_>>()
    );
    for (index, (fact, durable)) in receipt
        .bonds_in_canonical_order()
        .iter()
        .zip(&bonds)
        .enumerate()
    {
        let canonical = &authored.canonical_bonds()[index];
        assert_eq!(canonical.bond(), durable);
        assert_eq!(canonical.token(), fact.token());
        assert_eq!(canonical.haworth_position(), fact.haworth_position());
        assert_eq!(
            canonical.role(),
            if fact.haworth_position().is_some() {
                AuthoredDirectGlycosidicHaworthBondRoleV1::Ring
            } else {
                AuthoredDirectGlycosidicHaworthBondRoleV1::Bridge
            }
        );
        assert_eq!(
            canonical.authored_child_order(),
            (atoms.len() + index) as u32
        );
        assert_eq!(
            canonical.endpoints().as_slice(),
            [
                atom_for_source
                    .get(&fact.endpoints()[0])
                    .expect("start")
                    .clone(),
                atom_for_source
                    .get(&fact.endpoints()[1])
                    .expect("end")
                    .clone(),
            ]
            .as_slice()
        );
    }
    assert_eq!(authored.coordinates().len(), atoms.len());
    assert_eq!(authored.rings()[0].ring_form(), RingForm::Furanose);
    assert_eq!(authored.rings()[1].ring_form(), RingForm::Pyranose);
    for (fact, durable) in receipt.atoms_in_canonical_order().iter().zip(&atoms) {
        assert_eq!(
            authored.coordinates().get(durable),
            Some(&crate::haworth::HaworthPoint {
                x: fact.local().x + translation.x,
                y: fact.local().y + translation.y,
            })
        );
    }
    let expected_bounds = receipt.bounds().map(|point| crate::haworth::HaworthPoint {
        x: point.x + translation.x,
        y: point.y + translation.y,
    });
    assert_eq!(authored.bounds(), expected_bounds);
    let first_ring_count = topology.rings()[0].topology().bond_ids().len();
    let second_ring_count = topology.rings()[1].topology().bond_ids().len();
    let ring_counts = [first_ring_count, second_ring_count];
    let mut bond_index = 0;
    for (ring_index, count) in ring_counts.into_iter().enumerate() {
        let expected_cycle = &bonds[bond_index..bond_index + count];
        assert_eq!(
            authored.rings()[ring_index].bonds_in_canonical_cycle_order(),
            expected_cycle
        );
        for (fact, durable) in receipt.bonds_in_canonical_order()[bond_index..bond_index + count]
            .iter()
            .zip(expected_cycle)
        {
            let bond = authored.ring_bonds().get(durable).expect("ring fact");
            assert_eq!(bond.bond(), durable);
            assert_eq!(
                bond.endpoints().as_slice(),
                [
                    atom_for_source
                        .get(&fact.endpoints()[0])
                        .expect("start")
                        .clone(),
                    atom_for_source
                        .get(&fact.endpoints()[1])
                        .expect("end")
                        .clone(),
                ]
                .as_slice()
            );
            assert_eq!(bond.style(), fact.token());
            assert_eq!(
                bond.haworth_position(),
                fact.haworth_position().expect("ring depth")
            );
            assert_eq!(
                bond.authored_child_order(),
                (atoms.len() + bond_index) as u32
            );
            bond_index += 1;
        }
    }
    for (fact, durable) in receipt.bonds_in_canonical_order()[bond_index..]
        .iter()
        .zip(&bonds[bond_index..])
    {
        let bond = authored.bridge_bonds().get(durable).expect("bridge fact");
        assert_eq!(bond.bond(), durable);
        assert_eq!(
            bond.endpoints().as_slice(),
            [
                atom_for_source
                    .get(&fact.endpoints()[0])
                    .expect("start")
                    .clone(),
                atom_for_source
                    .get(&fact.endpoints()[1])
                    .expect("end")
                    .clone(),
            ]
            .as_slice()
        );
        assert_eq!(
            bond.authored_child_order(),
            (atoms.len() + bond_index) as u32
        );
        bond_index += 1;
    }
    assert_eq!(bond_index, bonds.len());
    let duplicate_atoms = vec![atoms[0].clone(); atoms.len()];
    let duplicate_bonds = vec![bonds[0].clone(); bonds.len()];
    let wrong_bond_kind: Vec<_> = (0..bonds.len())
        .map(|index| durable(RecordKind::Atom, &format!("wrong-bond{index}")))
        .collect();
    let wrong_kind: Vec<_> = (0..atoms.len())
        .map(|index| durable(RecordKind::Bond, &format!("wrong{index}")))
        .collect();
    for bad_atoms in [&duplicate_atoms, &wrong_kind] {
        assert!(
            receipt
                .authored_depiction_for_durable_commit_v1(
                    bad_atoms,
                    &bonds,
                    crate::haworth::HaworthPoint { x: 0.0, y: 0.0 }
                )
                .is_err()
        );
    }
    assert!(
        receipt
            .authored_depiction_for_durable_commit_v1(
                &atoms,
                &duplicate_bonds,
                crate::haworth::HaworthPoint { x: 0.0, y: 0.0 }
            )
            .is_err()
    );
    assert!(
        receipt
            .authored_depiction_for_durable_commit_v1(
                &atoms,
                &wrong_bond_kind,
                crate::haworth::HaworthPoint { x: 0.0, y: 0.0 }
            )
            .is_err()
    );
    assert!(
        receipt
            .authored_depiction_for_durable_commit_v1(
                &atoms,
                &bonds,
                crate::haworth::HaworthPoint {
                    x: f64::NAN,
                    y: 0.0
                }
            )
            .is_err()
    );
    assert!(
        receipt
            .authored_depiction_for_durable_commit_v1(
                &atoms,
                &bonds,
                crate::haworth::HaworthPoint {
                    x: f64::INFINITY,
                    y: 0.0
                }
            )
            .is_err()
    );
    let overflow_receipt =
        direct_glycosidic_haworth_authoring_receipt_v1(&molecule, topology, f64::MAX / 8.0)
            .expect("large finite local geometry");
    assert!(
        overflow_receipt
            .authored_depiction_for_durable_commit_v1(
                &atoms,
                &bonds,
                crate::haworth::HaworthPoint {
                    x: f64::MAX,
                    y: 0.0
                }
            )
            .is_err()
    );
    assert!(
        receipt
            .authored_depiction_for_durable_commit_v1(
                &atoms[..1],
                &bonds,
                crate::haworth::HaworthPoint { x: 0.0, y: 0.0 }
            )
            .is_err()
    );
}

#[test]
fn receipt_accepts_closed_supported_pairs_with_relational_geometry() {
    for (first, second) in [
        (RingForm::Furanose, RingForm::Furanose),
        (RingForm::Furanose, RingForm::Pyranose),
        (RingForm::Pyranose, RingForm::Pyranose),
    ] {
        let (molecule, topology) = source(first, second, false, false);
        let receipt = direct_glycosidic_haworth_authoring_receipt_v1(&molecule, topology, 7.0)
            .expect("closed receipt");
        let bridge = receipt.atoms_in_canonical_order().last().expect("bridge");
        assert_eq!(
            bridge.element(),
            DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen
        );
        assert!(
            receipt
                .atoms_in_canonical_order()
                .iter()
                .all(|atom| { atom.local().x.is_finite() && atom.local().y.is_finite() })
        );
        assert!(receipt.bonds_in_canonical_order().iter().all(|bond| {
            let endpoint_elements: Vec<_> = bond
                .endpoints()
                .iter()
                .map(|endpoint| {
                    receipt
                        .atoms_in_canonical_order()
                        .iter()
                        .find(|atom| atom.source_atom_identity() == endpoint)
                        .expect("bond endpoint retained as canonical atom")
                        .element()
                })
                .collect();
            match (bond.token(), bond.haworth_position()) {
                (DirectGlycosidicHaworthBondStyleV1::Q1, Some(position))
                | (DirectGlycosidicHaworthBondStyleV1::W1, Some(position)) => {
                    position == crate::haworth::DirectGlycosidicHaworthPositionV1::Front
                }
                (DirectGlycosidicHaworthBondStyleV1::N1, Some(position)) => {
                    position == crate::haworth::DirectGlycosidicHaworthPositionV1::Back
                }
                (DirectGlycosidicHaworthBondStyleV1::N1, None) => {
                    endpoint_elements
                        == [
                            DirectGlycosidicHaworthAuthoringAtomElementV1::Carbon,
                            DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen,
                        ]
                }
                _ => false,
            }
        }));
    }
}

#[test]
fn receipt_retains_c_or_o_and_canonical_roles_without_source_order_dependence() {
    let (forward_molecule, forward_topology) =
        source(RingForm::Furanose, RingForm::Pyranose, false, false);
    let (reverse_molecule, reverse_topology) =
        source(RingForm::Furanose, RingForm::Pyranose, true, false);
    let forward =
        direct_glycosidic_haworth_authoring_receipt_v1(&forward_molecule, forward_topology, 9.0)
            .expect("forward receipt");
    let reverse =
        direct_glycosidic_haworth_authoring_receipt_v1(&reverse_molecule, reverse_topology, 9.0)
            .expect("reverse receipt");
    assert_eq!(forward, reverse);
    assert!(forward.atoms_in_canonical_order().iter().all(|fact| {
        let source = forward_molecule
            .atoms()
            .iter()
            .find(|atom| atom.identity() == fact.source_atom_identity())
            .expect("receipt atom comes from source");
        matches!(
            (source.element(), fact.element()),
            (
                Some("C"),
                DirectGlycosidicHaworthAuthoringAtomElementV1::Carbon
            ) | (
                Some("O"),
                DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen
            )
        )
    }));
}

#[test]
fn receipt_rejects_stale_classification_and_richer_source_atoms() {
    let (molecule, topology) = source(RingForm::Pyranose, RingForm::Pyranose, false, false);
    let stale = Molecule::new(
        molecule.source_id().cloned(),
        None,
        molecule.atoms().iter().cloned().rev().collect(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        molecule.bonds().to_vec(),
        None,
    )
    .expect("reordered molecule");
    assert!(matches!(
        direct_glycosidic_haworth_authoring_receipt_v1(&stale, topology.clone(), 8.0),
        Err(HaworthError::StaleTopology(_))
    ));
    let (rich_molecule, rich_topology) =
        source(RingForm::Pyranose, RingForm::Pyranose, false, true);
    assert!(matches!(
        direct_glycosidic_haworth_authoring_receipt_v1(&rich_molecule, rich_topology, 8.0),
        Err(HaworthError::UnsupportedTopology(_))
    ));
}

#[test]
fn receipt_rejects_bond_type_or_style_for_ring_and_bridge_sources() {
    for (use_bridge, use_style) in [(false, false), (true, false), (false, true), (true, true)] {
        let (molecule, topology) = source(RingForm::Furanose, RingForm::Pyranose, false, false);
        let selected = if use_bridge {
            topology.bridge().bonds()[0].clone()
        } else {
            topology.rings()[0].topology().bond_ids()[0].clone()
        };
        let bonds = molecule
            .bonds()
            .iter()
            .map(|bond| {
                if bond.identity() == &selected {
                    bond.replace_source_fields(
                        bond.start().clone(),
                        bond.end().clone(),
                        (!use_style).then_some("n1".to_owned()),
                        bond.order(),
                        use_style.then_some(BondStyle::Normal),
                        bond.aromatic(),
                    )
                    .expect("modified bond")
                } else {
                    bond.clone()
                }
            })
            .collect();
        let altered = Molecule::new(
            molecule.source_id().cloned(),
            None,
            molecule.atoms().to_vec(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            bonds,
            None,
        )
        .expect("altered molecule");
        assert!(matches!(
            direct_glycosidic_haworth_authoring_receipt_v1(&altered, topology, 8.0),
            Err(HaworthError::UnsupportedTopology(_))
        ));
    }
}

#[test]
fn receipt_rejects_closed_source_guards_and_invalid_scales() {
    let (molecule, topology) = source(RingForm::Furanose, RingForm::Pyranose, false, false);
    for guard in ["name", "group", "extra_atom", "extra_bond"] {
        let mut atoms = molecule.atoms().to_vec();
        let mut bonds = molecule.bonds().to_vec();
        let mut groups = Vec::new();
        let name = match guard {
            "name" => Some("not closed".to_owned()),
            "group" => {
                groups.push(
                    NonAtomVertex::new(
                        RecordKind::Group,
                        Some(Identifier::new("g0").expect("identifier")),
                        None,
                    )
                    .expect("group"),
                );
                None
            }
            "extra_atom" => {
                atoms.push(atom(99, "C", None));
                None
            }
            "extra_bond" => {
                bonds.push(bond(99, &atoms[0], &atoms[5]));
                None
            }
            _ => unreachable!("table lists only closed-source guards"),
        };
        let altered = Molecule::new(
            molecule.source_id().cloned(),
            name,
            atoms,
            groups,
            Vec::new(),
            Vec::new(),
            bonds,
            None,
        )
        .expect("closed-source guard molecule");
        assert!(
            direct_glycosidic_haworth_authoring_receipt_v1(&altered, topology.clone(), 8.0)
                .is_err()
        );
    }
    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(matches!(
            direct_glycosidic_haworth_authoring_receipt_v1(&molecule, topology.clone(), scale),
            Err(HaworthError::InvalidSpec(
                "scale must be finite and positive"
            ))
        ));
    }
}

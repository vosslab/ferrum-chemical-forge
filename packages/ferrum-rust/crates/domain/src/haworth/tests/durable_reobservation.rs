use ferrum_core::{Identifier, RecordId, RecordKind};

use crate::haworth::{
    AuthoredDirectGlycosidicHaworthBondRoleV1, DirectGlycosidicHaworthAuthoringAtomElementV1,
    DirectGlycosidicHaworthBondStyleV1, DirectGlycosidicHaworthPositionV1,
    DurableDirectGlycosidicHaworthAtomFactV1, DurableDirectGlycosidicHaworthBondFactV1,
    DurableDirectGlycosidicHaworthProfileV1, DurableDirectGlycosidicHaworthRingFactV1,
    HaworthPoint, RingForm, authored_direct_glycosidic_haworth_depiction_from_durable_profile_v1,
};

fn identity(kind: RecordKind, name: &str) -> RecordId {
    RecordId::from_source(kind, &Identifier::new(name).expect("identifier"))
}

fn profile(
    reverse_shoulder: bool,
    invalid_bridge: bool,
    invalid_bond_order: bool,
) -> DurableDirectGlycosidicHaworthProfileV1 {
    let forms = [RingForm::Furanose, RingForm::Pyranose];
    let ring_sizes = forms.map(RingForm::vertex_count);
    let mut atoms = Vec::new();
    let mut rings = Vec::new();
    for (ring_index, count) in ring_sizes.into_iter().enumerate() {
        let mut ring_atoms = Vec::new();
        for vertex in 0..count {
            let atom = identity(RecordKind::Atom, &format!("a{ring_index}_{vertex}"));
            ring_atoms.push(atom.clone());
            atoms.push(DurableDirectGlycosidicHaworthAtomFactV1::new(
                atom,
                if vertex == 0 {
                    DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen
                } else {
                    DirectGlycosidicHaworthAuthoringAtomElementV1::Carbon
                },
                HaworthPoint {
                    x: (ring_index * 10 + vertex) as f64,
                    y: ring_index as f64,
                },
                u32::try_from(atoms.len()).expect("small child order"),
            ));
        }
        rings.push((forms[ring_index], ring_atoms));
    }
    let exterior = identity(RecordKind::Atom, "exterior");
    atoms.push(DurableDirectGlycosidicHaworthAtomFactV1::new(
        exterior.clone(),
        DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen,
        HaworthPoint { x: 8.0, y: -2.0 },
        u32::try_from(atoms.len()).expect("small child order"),
    ));
    let mut bonds = Vec::new();
    let mut ring_facts = Vec::new();
    for (ring_index, (form, ring_atoms)) in rings.iter().enumerate() {
        let count = ring_atoms.len();
        let q_index = 1;
        let mut ring_bonds = Vec::new();
        for index in 0..count {
            let bond = identity(RecordKind::Bond, &format!("b{ring_index}_{index}"));
            ring_bonds.push(bond.clone());
            let previous = (q_index + count - 1) % count;
            let next = (q_index + 1) % count;
            let (token, position, endpoints) = if index == q_index {
                (
                    DirectGlycosidicHaworthBondStyleV1::Q1,
                    Some(DirectGlycosidicHaworthPositionV1::Front),
                    [
                        ring_atoms[index].clone(),
                        ring_atoms[(index + 1) % count].clone(),
                    ],
                )
            } else if index == previous {
                (
                    DirectGlycosidicHaworthBondStyleV1::W1,
                    Some(DirectGlycosidicHaworthPositionV1::Front),
                    [ring_atoms[index].clone(), ring_atoms[q_index].clone()],
                )
            } else if index == next {
                (
                    DirectGlycosidicHaworthBondStyleV1::W1,
                    Some(DirectGlycosidicHaworthPositionV1::Front),
                    [
                        ring_atoms[(index + 1) % count].clone(),
                        ring_atoms[index].clone(),
                    ],
                )
            } else {
                (
                    DirectGlycosidicHaworthBondStyleV1::N1,
                    Some(DirectGlycosidicHaworthPositionV1::Back),
                    [
                        ring_atoms[index].clone(),
                        ring_atoms[(index + 1) % count].clone(),
                    ],
                )
            };
            let endpoints = if reverse_shoulder && ring_index == 0 && index == next {
                [endpoints[1].clone(), endpoints[0].clone()]
            } else {
                endpoints
            };
            bonds.push(DurableDirectGlycosidicHaworthBondFactV1::new(
                bond,
                endpoints,
                AuthoredDirectGlycosidicHaworthBondRoleV1::Ring,
                token,
                position,
                if invalid_bond_order && ring_index == 1 && index == 0 {
                    0
                } else {
                    u32::try_from(atoms.len() + bonds.len()).expect("small child order")
                },
            ));
        }
        ring_facts.push(DurableDirectGlycosidicHaworthRingFactV1::new(
            *form,
            ring_atoms.clone(),
            ring_bonds,
        ));
    }
    for (ring_index, (_, ring_atoms)) in rings.iter().enumerate() {
        bonds.push(DurableDirectGlycosidicHaworthBondFactV1::new(
            identity(RecordKind::Bond, &format!("bridge{ring_index}")),
            [
                ring_atoms[1].clone(),
                if invalid_bridge && ring_index == 1 {
                    ring_atoms[0].clone()
                } else {
                    exterior.clone()
                },
            ],
            AuthoredDirectGlycosidicHaworthBondRoleV1::Bridge,
            DirectGlycosidicHaworthBondStyleV1::N1,
            None,
            u32::try_from(atoms.len() + bonds.len()).expect("small child order"),
        ));
    }
    DurableDirectGlycosidicHaworthProfileV1::new(
        atoms,
        bonds,
        [ring_facts.remove(0), ring_facts.remove(0)],
    )
}

#[test]
fn durable_profile_rebuilds_the_exact_positional_depiction() {
    let depiction = authored_direct_glycosidic_haworth_depiction_from_durable_profile_v1(profile(
        false, false, false,
    ))
    .expect("closed durable profile");

    assert_eq!(depiction.rings()[0].ring_form(), RingForm::Furanose);
    assert_eq!(depiction.rings()[1].ring_form(), RingForm::Pyranose);
    assert_eq!(
        depiction
            .canonical_atoms()
            .iter()
            .map(|atom| atom.atom().clone())
            .collect::<Vec<_>>(),
        [
            "a0_0", "a0_1", "a0_2", "a0_3", "a0_4", "a1_0", "a1_1", "a1_2", "a1_3", "a1_4", "a1_5",
            "exterior",
        ]
        .into_iter()
        .map(|name| identity(RecordKind::Atom, name))
        .collect::<Vec<_>>()
    );
    assert_eq!(
        depiction
            .canonical_atoms()
            .iter()
            .map(|atom| atom.authored_child_order())
            .collect::<Vec<_>>(),
        (0..12).collect::<Vec<_>>()
    );
    let ring_facts = |ring: usize, count: usize| {
        (0..count)
            .map(|index| {
                let previous = 0;
                let q_index = 1;
                let next = 2;
                let atom = |vertex| identity(RecordKind::Atom, &format!("a{ring}_{vertex}"));
                let (token, position, endpoints) = if index == q_index {
                    (
                        DirectGlycosidicHaworthBondStyleV1::Q1,
                        Some(DirectGlycosidicHaworthPositionV1::Front),
                        [atom(index), atom((index + 1) % count)],
                    )
                } else if index == previous {
                    (
                        DirectGlycosidicHaworthBondStyleV1::W1,
                        Some(DirectGlycosidicHaworthPositionV1::Front),
                        [atom(index), atom(q_index)],
                    )
                } else if index == next {
                    (
                        DirectGlycosidicHaworthBondStyleV1::W1,
                        Some(DirectGlycosidicHaworthPositionV1::Front),
                        [atom((index + 1) % count), atom(index)],
                    )
                } else {
                    (
                        DirectGlycosidicHaworthBondStyleV1::N1,
                        Some(DirectGlycosidicHaworthPositionV1::Back),
                        [atom(index), atom((index + 1) % count)],
                    )
                };
                (
                    identity(RecordKind::Bond, &format!("b{ring}_{index}")),
                    endpoints,
                    AuthoredDirectGlycosidicHaworthBondRoleV1::Ring,
                    token,
                    position,
                )
            })
            .collect::<Vec<_>>()
    };
    let mut expected_bond_facts = ring_facts(0, 5);
    expected_bond_facts.extend(ring_facts(1, 6));
    expected_bond_facts.extend([
        (
            identity(RecordKind::Bond, "bridge0"),
            [
                identity(RecordKind::Atom, "a0_1"),
                identity(RecordKind::Atom, "exterior"),
            ],
            AuthoredDirectGlycosidicHaworthBondRoleV1::Bridge,
            DirectGlycosidicHaworthBondStyleV1::N1,
            None,
        ),
        (
            identity(RecordKind::Bond, "bridge1"),
            [
                identity(RecordKind::Atom, "a1_1"),
                identity(RecordKind::Atom, "exterior"),
            ],
            AuthoredDirectGlycosidicHaworthBondRoleV1::Bridge,
            DirectGlycosidicHaworthBondStyleV1::N1,
            None,
        ),
    ]);
    assert_eq!(
        depiction
            .canonical_bonds()
            .iter()
            .map(|bond| {
                (
                    bond.bond().clone(),
                    bond.endpoints().clone(),
                    bond.role(),
                    bond.token(),
                    bond.haworth_position(),
                    bond.authored_child_order(),
                )
            })
            .collect::<Vec<_>>(),
        expected_bond_facts
            .into_iter()
            .zip(12..)
            .map(|((bond, endpoints, role, token, position), order)| {
                (bond, endpoints, role, token, position, order)
            })
            .collect::<Vec<_>>()
    );
    assert_eq!(
        depiction.bounds(),
        [
            HaworthPoint { x: 0.0, y: -2.0 },
            HaworthPoint { x: 15.0, y: 1.0 }
        ]
    );
}

#[test]
fn durable_profile_rejects_noncanonical_directed_or_bridge_facts() {
    for profile in [
        profile(true, false, false),
        profile(false, true, false),
        profile(false, false, true),
    ] {
        assert!(
            authored_direct_glycosidic_haworth_depiction_from_durable_profile_v1(profile).is_err(),
            "the checked durable profile must refuse changed direct facts"
        );
    }
}

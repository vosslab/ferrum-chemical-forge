use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};

use crate::haworth::{
    DirectGlycosidicHaworthTopologyV1, HaworthError, HaworthTopology, HaworthTopologyBuilder,
    HaworthVertex, RingForm,
};

fn atom(index: usize, element: &str) -> Atom {
    Atom::new(
        Some(Identifier::new(format!("a{index}")).expect("identifier")),
        Some(element.to_owned()),
        Position::new(index as f64, 0.0, 0.0).expect("position"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("atom")
}

fn bond(index: usize, start: &Atom, end: &Atom, order: BondOrder, aromatic: bool) -> Bond {
    Bond::new(
        Some(Identifier::new(format!("b{index}")).expect("identifier")),
        VertexRef::Atom(start.identity().clone()),
        VertexRef::Atom(end.identity().clone()),
        None,
        Some(order),
        None,
        Some(aromatic),
        None,
    )
    .expect("bond")
}

fn two_ring_molecule(
    bridge_element: &str,
    bridge_order: BondOrder,
    bridge_aromatic: bool,
    extra_bridge_attachment: bool,
    reverse_storage: bool,
) -> (Molecule, [Vec<HaworthVertex>; 2], Vec<Atom>, Vec<Bond>) {
    let mut atoms: Vec<_> = ["O", "C", "C", "C", "C", "C"]
        .iter()
        .chain(["O", "C", "C", "C", "C", "C"].iter())
        .enumerate()
        .map(|(index, element)| atom(index, element))
        .collect();
    atoms.push(atom(12, bridge_element));
    if extra_bridge_attachment {
        atoms.push(atom(13, "C"));
    }
    let rings = [
        atoms[..6]
            .iter()
            .map(|atom| HaworthVertex {
                atom: atom.identity().clone(),
            })
            .collect(),
        atoms[6..12]
            .iter()
            .map(|atom| HaworthVertex {
                atom: atom.identity().clone(),
            })
            .collect(),
    ];
    let mut bonds: Vec<_> = (0..6)
        .map(|index| {
            bond(
                index,
                &atoms[index],
                &atoms[(index + 1) % 6],
                BondOrder::Single,
                false,
            )
        })
        .chain((0..6).map(|index| {
            bond(
                6 + index,
                &atoms[6 + index],
                &atoms[6 + (index + 1) % 6],
                BondOrder::Single,
                false,
            )
        }))
        .collect();
    bonds.push(bond(
        12,
        &atoms[1],
        &atoms[12],
        bridge_order,
        bridge_aromatic,
    ));
    bonds.push(bond(
        13,
        &atoms[7],
        &atoms[12],
        bridge_order,
        bridge_aromatic,
    ));
    if extra_bridge_attachment {
        bonds.push(bond(14, &atoms[12], &atoms[13], BondOrder::Single, false));
    }
    let source_atoms = atoms.clone();
    let source_bonds = bonds.clone();
    if reverse_storage {
        atoms.reverse();
        bonds.reverse();
    }
    (
        Molecule::new(
            Some(Identifier::new("two-rings").expect("identifier")),
            None,
            atoms,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            bonds,
            None,
        )
        .expect("molecule"),
        rings,
        source_atoms,
        source_bonds,
    )
}

fn topologies(molecule: &Molecule, rings: [Vec<HaworthVertex>; 2]) -> [HaworthTopology; 2] {
    rings.map(|vertices| {
        HaworthTopologyBuilder::new(RingForm::Pyranose, vertices[1].atom.clone(), vertices)
            .build(molecule)
            .expect("ring topology")
    })
}

fn classify(
    molecule: &Molecule,
    rings: [Vec<HaworthVertex>; 2],
) -> Result<DirectGlycosidicHaworthTopologyV1, HaworthError> {
    let bridge = molecule
        .atoms()
        .iter()
        .find(|atom| atom.source_id().is_some_and(|id| id.as_str() == "a12"))
        .expect("bridge")
        .identity()
        .clone();
    let bridge_bonds = [
        molecule
            .bonds()
            .iter()
            .find(|bond| bond.source_id().is_some_and(|id| id.as_str() == "b12"))
            .expect("first bridge bond")
            .identity()
            .clone(),
        molecule
            .bonds()
            .iter()
            .find(|bond| bond.source_id().is_some_and(|id| id.as_str() == "b13"))
            .expect("second bridge bond")
            .identity()
            .clone(),
    ];
    DirectGlycosidicHaworthTopologyV1::classify(
        molecule,
        topologies(molecule, rings),
        bridge,
        bridge_bonds,
    )
}

#[test]
fn classifies_two_rings_and_canonicalizes_caller_order() {
    let (molecule, rings, _, _) = two_ring_molecule("O", BondOrder::Single, false, false, false);
    let forward = classify(&molecule, rings.clone()).expect("accepted direct glycosidic topology");
    let reversed = DirectGlycosidicHaworthTopologyV1::classify(
        &molecule,
        [
            topologies(&molecule, rings.clone())[1].clone(),
            topologies(&molecule, rings)[0].clone(),
        ],
        forward.bridge().atom().clone(),
        [
            forward.bridge().bonds()[1].clone(),
            forward.bridge().bonds()[0].clone(),
        ],
    )
    .expect("caller order is not semantic");

    assert_eq!(forward, reversed);
    assert_eq!(
        forward.rings()[0].attachment_atom(),
        &forward.rings()[0].topology().vertices()[5].atom
    );
    assert_eq!(
        forward.rings()[1].attachment_atom(),
        &forward.rings()[1].topology().vertices()[5].atom
    );
}

#[test]
fn records_selected_graph_source_orders_from_the_frozen_molecule() {
    let (molecule, rings, _, _) = two_ring_molecule("O", BondOrder::Single, false, false, true);
    let topology = classify(&molecule, rings).expect("accepted topology");

    for (order, atom) in molecule.atoms().iter().enumerate() {
        if topology.atom_source_orders().contains_key(atom.identity()) {
            assert_eq!(
                topology.atom_source_orders().get(atom.identity()),
                Some(&order)
            );
        }
    }
    for (order, bond) in molecule.bonds().iter().enumerate() {
        if topology.bond_source_orders().contains_key(bond.identity()) {
            assert_eq!(
                topology.bond_source_orders().get(bond.identity()),
                Some(&order)
            );
        }
    }
}

#[test]
fn rejects_nonprofile_bridge_and_inconsistent_ring_selection() {
    let (carbon_bridge, rings, _, _) =
        two_ring_molecule("C", BondOrder::Single, false, false, false);
    assert_eq!(
        classify(&carbon_bridge, rings),
        Err(HaworthError::UnsupportedTopology(
            "bridge atom must be an exterior oxygen"
        ))
    );

    let (extra_attachment, rings, _, _) =
        two_ring_molecule("O", BondOrder::Single, false, true, false);
    assert_eq!(
        classify(&extra_attachment, rings),
        Err(HaworthError::UnsupportedTopology(
            "bridge oxygen must have degree two"
        ))
    );

    let (molecule, rings, _, _) = two_ring_molecule("O", BondOrder::Single, false, false, false);
    let topology = topologies(&molecule, rings);
    let bridge = molecule.atoms()[12].identity().clone();
    let bridge_bonds = [
        molecule.bonds()[12].identity().clone(),
        molecule.bonds()[13].identity().clone(),
    ];
    assert_eq!(
        DirectGlycosidicHaworthTopologyV1::classify(
            &molecule,
            [topology[0].clone(), topology[0].clone()],
            bridge,
            bridge_bonds,
        ),
        Err(HaworthError::UnsupportedTopology(
            "direct glycosidic rings must be vertex-disjoint"
        ))
    );
}

#[test]
fn rejects_non_single_or_aromatic_bridge_bonds() {
    let (non_single, rings, _, _) = two_ring_molecule("O", BondOrder::Double, false, false, false);
    assert_eq!(
        classify(&non_single, rings),
        Err(HaworthError::UnsupportedTopology(
            "bridge bonds must be non-aromatic single bonds"
        ))
    );

    let (aromatic, rings, _, _) = two_ring_molecule("O", BondOrder::Single, true, false, false);
    assert_eq!(
        classify(&aromatic, rings),
        Err(HaworthError::UnsupportedTopology(
            "bridge bonds must be non-aromatic single bonds"
        ))
    );
}

#[test]
fn rejects_a_bridge_bond_attached_to_a_selected_ring_oxygen() {
    let (molecule, rings, _, _) = two_ring_molecule("O", BondOrder::Single, false, false, false);
    let mut bonds = molecule.bonds().to_vec();
    bonds[12] = bonds[12]
        .replace_source_fields(
            VertexRef::Atom(molecule.atoms()[0].identity().clone()),
            VertexRef::Atom(molecule.atoms()[12].identity().clone()),
            None,
            Some(BondOrder::Single),
            None,
            Some(false),
        )
        .expect("oxygen-attached bridge bond");
    let oxygen_attached = molecule
        .replace_records(
            None,
            molecule.atoms().to_vec(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            bonds,
        )
        .expect("oxygen-attached graph");

    assert_eq!(
        classify(&oxygen_attached, rings),
        Err(HaworthError::UnsupportedTopology(
            "bridge bonds must attach to selected ring carbons"
        ))
    );
}

#[test]
fn rejects_a_ring_topology_from_a_stale_graph_snapshot() {
    let (original, rings, _, _) = two_ring_molecule("O", BondOrder::Single, false, false, false);
    let stale_rings = topologies(&original, rings.clone());
    let mut bonds = original.bonds().to_vec();
    bonds[0] = bonds[0]
        .replace_source_fields(
            VertexRef::Atom(rings[0][0].atom.clone()),
            VertexRef::Atom(rings[0][2].atom.clone()),
            None,
            Some(BondOrder::Single),
            None,
            Some(false),
        )
        .expect("altered ring edge");
    let altered = original
        .replace_records(
            None,
            original.atoms().to_vec(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            bonds,
        )
        .expect("alternate graph snapshot");
    assert_eq!(
        DirectGlycosidicHaworthTopologyV1::classify(
            &altered,
            stale_rings,
            altered.atoms()[12].identity().clone(),
            [
                altered.bonds()[12].identity().clone(),
                altered.bonds()[13].identity().clone(),
            ],
        ),
        Err(HaworthError::StaleTopology(
            "selected ring does not match molecule snapshot"
        ))
    );
}

use std::collections::BTreeSet;

use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, RecordId, VertexRef};

use crate::haworth::{
    DirectGlycosidicHaworthFragmentRequestV1, DirectGlycosidicHaworthTopologyV1, HaworthError,
    HaworthTopologyBuilder, HaworthVertex, RingForm,
    assemble_direct_glycosidic_haworth_fragment_v1,
};

use super::direct_glycosidic_layout::topology as layout_topology;

fn atom(index: usize, element: &str) -> Atom {
    Atom::new(
        Identifier::new(format!("a{index}")).expect("identifier"),
        Some(element.to_owned()),
        Position::new(index as f64, 0.0, 0.0).expect("position"),
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
        Identifier::new(format!("b{index}")).expect("identifier"),
        VertexRef::Atom(start.identity().clone()),
        VertexRef::Atom(end.identity().clone()),
        None,
        Some(BondOrder::Single),
        None,
        Some(false),
    )
    .expect("bond")
}

fn topology_with_substituent() -> (DirectGlycosidicHaworthTopologyV1, RecordId, RecordId) {
    let mut atoms: Vec<_> = (0..13)
        .map(|index| {
            atom(
                index,
                if index % 6 == 0 || index == 12 {
                    "O"
                } else {
                    "C"
                },
            )
        })
        .collect();
    atoms.push(atom(13, "O"));
    let mut bonds: Vec<_> = (0..6)
        .map(|index| bond(index, &atoms[index], &atoms[(index + 1) % 6]))
        .collect();
    bonds.extend(
        (0..6).map(|index| bond(6 + index, &atoms[6 + index], &atoms[6 + (index + 1) % 6])),
    );
    bonds.push(bond(12, &atoms[1], &atoms[12]));
    bonds.push(bond(13, &atoms[7], &atoms[12]));
    bonds.push(bond(14, &atoms[2], &atoms[13]));
    let molecule = Molecule::new(
        Identifier::new("two-rings").expect("identifier"),
        None,
        atoms.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        bonds.clone(),
    )
    .expect("molecule");
    let ring = |offset: usize| {
        let vertices: Vec<_> = atoms[offset..offset + 6]
            .iter()
            .map(|atom| HaworthVertex {
                atom: atom.identity().clone(),
            })
            .collect();
        HaworthTopologyBuilder::new(RingForm::Pyranose, vertices[1].atom.clone(), vertices)
            .build(&molecule)
            .expect("ring topology")
    };
    let topology = DirectGlycosidicHaworthTopologyV1::classify(
        &molecule,
        [ring(0), ring(6)],
        atoms[12].identity().clone(),
        [bonds[12].identity().clone(), bonds[13].identity().clone()],
    )
    .expect("direct glycosidic topology");
    (
        topology,
        atoms[13].identity().clone(),
        bonds[14].identity().clone(),
    )
}

#[test]
fn assembles_exact_selected_graph_coverage_and_endpoint_geometry() {
    let (topology, substituent_atom, substituent_bond) = topology_with_substituent();
    let fragment =
        assemble_direct_glycosidic_haworth_fragment_v1(&DirectGlycosidicHaworthFragmentRequestV1 {
            topology: topology.clone(),
            scale: 10.0,
        })
        .expect("fragment");
    let expected_atoms: BTreeSet<_> = topology
        .rings()
        .iter()
        .flat_map(|ring| {
            ring.topology()
                .vertices()
                .iter()
                .map(|vertex| vertex.atom.clone())
        })
        .chain(std::iter::once(topology.bridge().atom().clone()))
        .collect();
    let expected_bonds: BTreeSet<_> = topology
        .rings()
        .iter()
        .flat_map(|ring| ring.topology().bond_ids().iter().cloned())
        .chain(topology.bridge().bonds().iter().cloned())
        .collect();
    assert_eq!(
        fragment
            .coordinates()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        expected_atoms
    );
    assert!(!fragment.coordinates().contains_key(&substituent_atom));
    assert!(!fragment.ring_edges().contains_key(&substituent_bond));
    assert!(!fragment.bridge_edges().contains_key(&substituent_bond));
    let ring_bonds: BTreeSet<_> = fragment.ring_edges().keys().cloned().collect();
    let bridge_bonds: BTreeSet<_> = fragment.bridge_edges().keys().cloned().collect();
    assert!(ring_bonds.is_disjoint(&bridge_bonds));
    assert_eq!(
        ring_bonds
            .union(&bridge_bonds)
            .cloned()
            .collect::<BTreeSet<_>>(),
        expected_bonds
    );
    for (bond, endpoints) in fragment.ring_edges() {
        assert_eq!(
            fragment.ring_geometry()[bond],
            [
                fragment.coordinates()[&endpoints[0]],
                fragment.coordinates()[&endpoints[1]]
            ]
        );
        assert!(fragment.ring_depictions().contains_key(bond));
    }
    for ring in topology.rings() {
        let bond = ring.attachment_bond();
        let endpoints = &fragment.bridge_edges()[bond];
        assert_eq!(endpoints[0], *ring.attachment_atom());
        assert_eq!(endpoints[1], *topology.bridge().atom());
        assert_eq!(
            fragment.bridge_geometry()[bond],
            [
                fragment.coordinates()[&endpoints[0]],
                fragment.coordinates()[&endpoints[1]]
            ]
        );
    }
    assert!(
        fragment
            .bounds()
            .into_iter()
            .all(|point| point.x.is_finite() && point.y.is_finite())
    );
}

#[test]
fn propagates_invalid_scale_as_a_typed_layout_error() {
    let (topology, _, _) = topology_with_substituent();
    assert_eq!(
        assemble_direct_glycosidic_haworth_fragment_v1(&DirectGlycosidicHaworthFragmentRequestV1 {
            topology,
            scale: f64::NAN,
        },),
        Err(HaworthError::InvalidSpec(
            "scale must be finite and positive"
        )),
    );
}

#[test]
fn preserves_all_supported_profile_facts_and_canonical_roles() {
    for (forms, attachments) in [
        ((RingForm::Furanose, RingForm::Furanose), (1, 3)),
        ((RingForm::Furanose, RingForm::Pyranose), (2, 5)),
        ((RingForm::Pyranose, RingForm::Pyranose), (4, 1)),
    ] {
        let fragment = |reverse_request| {
            assemble_direct_glycosidic_haworth_fragment_v1(
                &DirectGlycosidicHaworthFragmentRequestV1 {
                    topology: layout_topology(
                        forms.0,
                        forms.1,
                        attachments.0,
                        attachments.1,
                        reverse_request,
                    ),
                    scale: 8.0,
                },
            )
            .expect("fragment")
        };
        let forward = fragment(false);
        let reversed = fragment(true);
        assert_eq!(forward, reversed);
        assert!(
            forward
                .coordinates()
                .values()
                .all(|point| point.x.is_finite() && point.y.is_finite())
        );
        assert_eq!(forward.ring_edges().len(), forward.ring_geometry().len());
        assert_eq!(forward.bridge_edges().len(), 2);
    }
}

use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};

use crate::haworth::{
    DirectGlycosidicHaworthLayoutRequestV1, DirectGlycosidicHaworthTopologyV1, HaworthError,
    HaworthTopology, HaworthTopologyBuilder, HaworthVertex, RingForm,
    layout_direct_glycosidic_haworth_v1,
};

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

pub(super) fn topology(
    first_form: RingForm,
    second_form: RingForm,
    first_attachment: usize,
    second_attachment: usize,
    reverse_request: bool,
) -> DirectGlycosidicHaworthTopologyV1 {
    let first_count = first_form.vertex_count();
    let second_count = second_form.vertex_count();
    let mut atoms = Vec::new();
    for index in 0..first_count {
        atoms.push(atom(index, if index == 0 { "O" } else { "C" }));
    }
    for index in 0..second_count {
        atoms.push(atom(
            first_count + index,
            if index == 0 { "O" } else { "C" },
        ));
    }
    let bridge_index = first_count + second_count;
    atoms.push(atom(bridge_index, "O"));
    let first_vertices: Vec<_> = atoms[..first_count]
        .iter()
        .map(|atom| HaworthVertex {
            atom: atom.identity().clone(),
        })
        .collect();
    let second_vertices: Vec<_> = atoms[first_count..bridge_index]
        .iter()
        .map(|atom| HaworthVertex {
            atom: atom.identity().clone(),
        })
        .collect();
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
    let first_bridge_bond = first_count + second_count;
    let second_bridge_bond = first_bridge_bond + 1;
    bonds.push(bond(
        first_bridge_bond,
        &atoms[first_attachment],
        &atoms[bridge_index],
    ));
    bonds.push(bond(
        second_bridge_bond,
        &atoms[first_count + second_attachment],
        &atoms[bridge_index],
    ));
    let molecule = Molecule::new(
        Identifier::new("two-rings").expect("identifier"),
        None,
        atoms,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        bonds,
    )
    .expect("molecule");
    let build = |form: RingForm, vertices: Vec<HaworthVertex>| {
        HaworthTopologyBuilder::new(form, vertices[1].atom.clone(), vertices)
            .build(&molecule)
            .expect("ring topology")
    };
    let first = build(first_form, first_vertices);
    let second = build(second_form, second_vertices);
    let bridge = molecule
        .atoms()
        .iter()
        .find(|atom| atom.source_id().as_str() == format!("a{bridge_index}"))
        .expect("bridge atom")
        .identity()
        .clone();
    let bridge_bonds = [
        molecule
            .bonds()
            .iter()
            .find(|bond| bond.source_id().as_str() == format!("b{first_bridge_bond}"))
            .expect("first bridge bond")
            .identity()
            .clone(),
        molecule
            .bonds()
            .iter()
            .find(|bond| bond.source_id().as_str() == format!("b{second_bridge_bond}"))
            .expect("second bridge bond")
            .identity()
            .clone(),
    ];
    let rings: [HaworthTopology; 2] = [first, second];
    if reverse_request {
        DirectGlycosidicHaworthTopologyV1::classify(
            &molecule,
            [rings[1].clone(), rings[0].clone()],
            bridge,
            [bridge_bonds[1].clone(), bridge_bonds[0].clone()],
        )
    } else {
        DirectGlycosidicHaworthTopologyV1::classify(&molecule, rings, bridge, bridge_bonds)
    }
    .expect("direct glycosidic topology")
}

#[test]
fn layouts_supported_ring_pairs_with_identity_preserving_bridge_endpoints() {
    for (forms, attachments) in [
        ((RingForm::Furanose, RingForm::Furanose), (1, 3)),
        ((RingForm::Furanose, RingForm::Pyranose), (2, 5)),
        ((RingForm::Pyranose, RingForm::Pyranose), (4, 1)),
    ] {
        let topology = topology(forms.0, forms.1, attachments.0, attachments.1, false);
        let layout = layout_direct_glycosidic_haworth_v1(&DirectGlycosidicHaworthLayoutRequestV1 {
            topology: topology.clone(),
            scale: 12.0,
        })
        .expect("finite local layout");
        assert!(
            layout
                .depictions()
                .iter()
                .flat_map(|depiction| depiction.coordinates().values())
                .all(|point| point.x.is_finite() && point.y.is_finite())
        );
        assert!(
            layout
                .bounds()
                .iter()
                .all(|point| point.x.is_finite() && point.y.is_finite())
        );
        assert_eq!(layout.bridge_atom(), topology.bridge().atom());
        for ring in topology.rings() {
            assert_eq!(
                layout.bridge_endpoints().get(ring.attachment_bond()),
                Some(&[
                    ring.attachment_atom().clone(),
                    topology.bridge().atom().clone()
                ]),
            );
        }
        for (depiction, ring) in layout.depictions().iter().zip(topology.rings()) {
            assert_no_proper_bridge_crossing(
                depiction,
                ring.topology().vertices(),
                ring.attachment_atom(),
                layout.bridge_point(),
            );
        }
    }
}

fn assert_no_proper_bridge_crossing(
    depiction: &crate::haworth::HaworthDepiction,
    vertices: &[HaworthVertex],
    attachment: &ferrum_core::RecordId,
    bridge: crate::haworth::HaworthPoint,
) {
    let attachment_point = depiction.coordinates()[attachment];
    for index in 0..vertices.len() {
        let first = &vertices[index].atom;
        let second = &vertices[(index + 1) % vertices.len()].atom;
        if first == attachment || second == attachment {
            continue;
        }
        assert!(
            !properly_intersects(
                attachment_point,
                bridge,
                depiction.coordinates()[first],
                depiction.coordinates()[second],
            ),
            "bridge must not properly cross a nonincident ring edge",
        );
    }
}

fn properly_intersects(
    a: crate::haworth::HaworthPoint,
    b: crate::haworth::HaworthPoint,
    c: crate::haworth::HaworthPoint,
    d: crate::haworth::HaworthPoint,
) -> bool {
    let cross = |origin: crate::haworth::HaworthPoint,
                 endpoint: crate::haworth::HaworthPoint,
                 point: crate::haworth::HaworthPoint| {
        (endpoint.x - origin.x) * (point.y - origin.y)
            - (endpoint.y - origin.y) * (point.x - origin.x)
    };
    let first = cross(a, b, c);
    let second = cross(a, b, d);
    let third = cross(c, d, a);
    let fourth = cross(c, d, b);
    ((first > 0.0 && second < 0.0) || (first < 0.0 && second > 0.0))
        && ((third > 0.0 && fourth < 0.0) || (third < 0.0 && fourth > 0.0))
}

#[test]
fn caller_order_does_not_change_canonical_layout_roles() {
    let forward = topology(RingForm::Furanose, RingForm::Pyranose, 1, 4, false);
    let reversed = topology(RingForm::Furanose, RingForm::Pyranose, 1, 4, true);
    assert_eq!(forward, reversed);
    let request = |topology| DirectGlycosidicHaworthLayoutRequestV1 {
        topology,
        scale: 8.0,
    };
    let first = layout_direct_glycosidic_haworth_v1(&request(forward)).expect("forward layout");
    let second = layout_direct_glycosidic_haworth_v1(&request(reversed)).expect("reverse layout");
    assert_eq!(first.bridge_endpoints(), second.bridge_endpoints());
    for (first_ring, second_ring) in first.depictions().iter().zip(second.depictions()) {
        assert_eq!(first_ring.ring_form(), second_ring.ring_form());
        assert_eq!(
            first_ring.coordinates().keys().collect::<Vec<_>>(),
            second_ring.coordinates().keys().collect::<Vec<_>>()
        );
        assert_eq!(first_ring.bonds(), second_ring.bonds());
    }
}

#[test]
fn rejects_invalid_scale() {
    let topology = topology(RingForm::Pyranose, RingForm::Furanose, 2, 3, false);
    for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            layout_direct_glycosidic_haworth_v1(&DirectGlycosidicHaworthLayoutRequestV1 {
                topology: topology.clone(),
                scale,
            }),
            Err(HaworthError::InvalidSpec(
                "scale must be finite and positive"
            )),
        );
    }
}

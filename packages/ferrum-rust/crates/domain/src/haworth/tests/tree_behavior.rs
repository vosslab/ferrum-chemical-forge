use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};

use crate::haworth::{
    GlycosidicLink, HaworthAttachment, HaworthError, HaworthFragment, HaworthRingNode,
    HaworthTopologyBuilder, HaworthTreeRequest, HaworthVertex, RingForm, layout_tree,
};

fn restore(
    request: &HaworthTreeRequest,
    value: serde_json::Value,
) -> serde_json::Result<HaworthFragment> {
    let text = serde_json::to_string(&value)?;
    let mut deserializer = serde_json::Deserializer::from_str(&text);
    HaworthFragment::restore(request, &mut deserializer)
}

fn atom(index: usize, element: &str) -> Atom {
    Atom::new(
        Some(Identifier::new(format!("tree-a{index}")).expect("identifier")),
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

fn bond(index: usize, start: &Atom, end: &Atom) -> Bond {
    Bond::new(
        Some(Identifier::new(format!("tree-b{index}")).expect("identifier")),
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

fn tree(ring_count: usize, reversed_storage: bool) -> HaworthTreeRequest {
    let mut atoms = Vec::new();
    let mut cycles = Vec::new();
    let mut bonds = Vec::new();
    for _ring in 0..ring_count {
        let offset = atoms.len();
        for element in ["O", "C", "C", "C", "C", "C"] {
            atoms.push(atom(atoms.len(), element));
        }
        let cycle: Vec<_> = atoms[offset..offset + 6]
            .iter()
            .map(|atom| HaworthVertex {
                atom: atom.identity().clone(),
            })
            .collect();
        for edge in 0..6 {
            bonds.push(bond(
                bonds.len(),
                &atoms[offset + edge],
                &atoms[offset + (edge + 1) % 6],
            ));
        }
        cycles.push(cycle);
    }
    for ring in 1..ring_count {
        bonds.push(bond(
            bonds.len(),
            &atoms[(ring - 1) * 6 + 2],
            &atoms[ring * 6 + 1],
        ));
    }
    let molecule = Molecule::new(
        Some(Identifier::new("tree-molecule").expect("identifier")),
        None,
        atoms.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        bonds.clone(),
        None,
    )
    .expect("molecule");
    let mut nodes: Vec<_> = cycles
        .into_iter()
        .enumerate()
        .map(|(node_id, mut cycle)| {
            if reversed_storage {
                cycle.reverse();
            }
            let anomeric = atoms[node_id * 6 + 1].identity().clone();
            HaworthRingNode {
                node_id: node_id as u32,
                topology: HaworthTopologyBuilder::new(RingForm::Pyranose, anomeric, cycle)
                    .build(&molecule)
                    .expect("topology"),
            }
        })
        .collect();
    let mut links = Vec::new();
    for ring in 1..ring_count {
        let link_bond = bonds[ring_count * 6 + ring - 1].identity().clone();
        links.push(GlycosidicLink {
            bond: link_bond,
            parent: HaworthAttachment {
                node_id: (ring - 1) as u32,
                atom: atoms[(ring - 1) * 6 + 2].identity().clone(),
            },
            child: HaworthAttachment {
                node_id: ring as u32,
                atom: atoms[ring * 6 + 1].identity().clone(),
            },
        });
    }
    if reversed_storage {
        nodes.reverse();
        links.reverse();
    }
    HaworthTreeRequest {
        molecule,
        rings: nodes,
        links,
        root: 0,
        scale: 12.0,
    }
}

#[test]
fn one_two_and_three_ring_tree_layouts_are_finite_and_storage_deterministic() {
    for count in 1..=3 {
        let first = layout_tree(&tree(count, false)).expect("tree layout");
        let reordered = layout_tree(&tree(count, true)).expect("tree layout");
        assert_eq!(first, reordered);
        assert_eq!(first.coordinates().len(), count * 6);
        assert_eq!(first.links().len(), count - 1);
        assert!(
            first
                .coordinates()
                .values()
                .all(|point| point.x.is_finite() && point.y.is_finite())
        );
    }
}

#[test]
fn branch_order_is_deterministic_without_a_fixed_cardinality_api() {
    let mut request = tree(3, false);
    request.links[1].parent.node_id = 0;
    request.links[1].parent.atom = request.rings[0].topology.vertices()[3].atom.clone();
    let changed_bond = request.links[1].bond.clone();
    let mut bonds = request.molecule.bonds().to_vec();
    let bond = bonds
        .iter_mut()
        .find(|bond| bond.identity() == &changed_bond)
        .expect("declared bond exists");
    *bond = bond
        .replace_source_fields(
            VertexRef::Atom(request.links[1].parent.atom.clone()),
            VertexRef::Atom(request.links[1].child.atom.clone()),
            None,
            Some(BondOrder::Single),
            None,
            Some(false),
        )
        .expect("rewired branch bond");
    request.molecule = request
        .molecule
        .replace_records(
            None,
            request.molecule.atoms().to_vec(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            bonds,
        )
        .expect("rewired molecule");
    let first = layout_tree(&request).expect("branched tree layout");
    request.links.reverse();
    let reordered = layout_tree(&request).expect("branched tree layout");
    assert_eq!(first, reordered);
}

#[test]
fn malformed_tree_rejects_attachment_reuse_and_nonfinite_scale() {
    let mut request = tree(2, false);
    request.links.push(request.links[0].clone());
    let error = layout_tree(&request).expect_err("duplicate link rejected");
    assert_eq!(
        error,
        HaworthError::UnsupportedTopology("tree must have exactly one fewer link than rings")
    );
    let mut request = tree(1, false);
    request.scale = f64::INFINITY;
    assert_eq!(
        layout_tree(&request),
        Err(HaworthError::InvalidSpec(
            "scale must be finite and positive"
        ))
    );
}

#[test]
fn fragment_wire_revalidates_bounds_and_fingerprint() {
    let request = tree(2, false);
    let fragment = layout_tree(&request).expect("fragment");
    let value = serde_json::to_value(&fragment).expect("serialize");
    assert_eq!(
        restore(&request, value.clone()).expect("validated wire"),
        fragment
    );
    let mut invalid = value;
    invalid["graph_fingerprint"] = serde_json::json!(0);
    assert!(restore(&request, invalid).is_err());

    let mut geometry_tamper = serde_json::to_value(&fragment).expect("serialize");
    geometry_tamper["bond_geometry"][0][1][0]["x"] = serde_json::json!(999.0);
    assert!(restore(&request, geometry_tamper).is_err());

    let mut link_identity_tamper = serde_json::to_value(&fragment).expect("serialize");
    let parent = link_identity_tamper["link_topology"][0][1]["parent_atom"].clone();
    link_identity_tamper["link_topology"][0][1]["parent_atom"] =
        link_identity_tamper["link_topology"][0][1]["child_atom"].clone();
    link_identity_tamper["link_topology"][0][1]["child_atom"] = parent;
    assert!(restore(&request, link_identity_tamper).is_err());
}

#[test]
fn tree_rejects_an_undeclared_inter_ring_graph_bond() {
    let mut request = tree(2, false);
    let mut bonds = request.molecule.bonds().to_vec();
    let atoms = request.molecule.atoms().to_vec();
    bonds.push(bond(99, &atoms[3], &atoms[10]));
    request.molecule = request
        .molecule
        .replace_records(None, atoms, Vec::new(), Vec::new(), Vec::new(), bonds)
        .expect("molecule with extra link");
    assert_eq!(
        layout_tree(&request),
        Err(HaworthError::StaleTopology(
            "molecule has an undeclared inter-ring bond"
        ))
    );
}

#[test]
fn tree_rejects_a_ring_topology_from_a_different_graph_snapshot() {
    let mut request = tree(1, false);
    let mut bonds = request.molecule.bonds().to_vec();
    let atoms = request.molecule.atoms().to_vec();
    bonds[0] = bonds[0]
        .replace_source_fields(
            VertexRef::Atom(atoms[0].identity().clone()),
            VertexRef::Atom(atoms[2].identity().clone()),
            None,
            Some(BondOrder::Single),
            None,
            Some(false),
        )
        .expect("altered ring edge");
    request.molecule = request
        .molecule
        .replace_records(None, atoms, Vec::new(), Vec::new(), Vec::new(), bonds)
        .expect("alternate graph snapshot");
    assert_eq!(
        layout_tree(&request),
        Err(HaworthError::StaleTopology(
            "selected ring edge does not match molecule snapshot"
        ))
    );
}

#[test]
fn fragment_keeps_graph_source_order_for_ring_and_link_targets() {
    let request = tree(2, false);
    let fragment = layout_tree(&request).expect("fragment");
    for (source_order, bond) in request.molecule.bonds().iter().enumerate() {
        if fragment.ring_bonds().contains_key(bond.identity())
            || fragment.links().contains_key(bond.identity())
        {
            assert_eq!(
                fragment.source_orders().get(bond.identity()),
                Some(&(source_order as u32))
            );
        }
    }
}

#[test]
fn fragment_wire_rejects_recomputed_topology_tampering() {
    let request = tree(3, false);
    let fragment = layout_tree(&request).expect("fragment");
    let original = serde_json::to_value(fragment).expect("serialize");

    let mut same_ring = original.clone();
    same_ring["link_topology"][0][1]["child_ring"] =
        same_ring["link_topology"][0][1]["parent_ring"].clone();
    crate::haworth::tree::refresh_wire_fingerprint(&mut same_ring);
    assert!(restore(&request, same_ring).is_err());

    let mut reused_attachment = original.clone();
    reused_attachment["link_topology"][1][1]["parent_atom"] =
        reused_attachment["link_topology"][0][1]["parent_atom"].clone();
    crate::haworth::tree::refresh_wire_fingerprint(&mut reused_attachment);
    assert!(restore(&request, reused_attachment).is_err());

    let mut mixed_cycle = original;
    let first = mixed_cycle["ring_topology"][0]["bonds"][0].clone();
    mixed_cycle["ring_topology"][0]["bonds"][0] =
        mixed_cycle["ring_topology"][1]["bonds"][0].clone();
    mixed_cycle["ring_topology"][1]["bonds"][0] = first;
    crate::haworth::tree::refresh_wire_fingerprint(&mut mixed_cycle);
    assert!(restore(&request, mixed_cycle).is_err());

    let reversal_request = tree(2, false);
    let mut reversed_atoms =
        serde_json::to_value(layout_tree(&reversal_request).expect("fragment")).expect("serialize");
    reversed_atoms["ring_topology"][0]["atoms"]
        .as_array_mut()
        .expect("atoms")
        .reverse();
    crate::haworth::tree::refresh_wire_fingerprint(&mut reversed_atoms);
    assert!(restore(&reversal_request, reversed_atoms).is_err());

    let rotation_request = tree(2, false);
    let mut rotated =
        serde_json::to_value(layout_tree(&rotation_request).expect("fragment")).expect("serialize");
    rotated["ring_topology"][0]["atoms"]
        .as_array_mut()
        .expect("atoms")
        .rotate_left(1);
    rotated["ring_topology"][0]["bonds"]
        .as_array_mut()
        .expect("bonds")
        .rotate_left(1);
    rotated["ring_cycles"][0]
        .as_array_mut()
        .expect("cycle")
        .rotate_left(1);
    crate::haworth::tree::refresh_wire_fingerprint(&mut rotated);
    assert!(restore(&rotation_request, rotated).is_err());
}

#[test]
fn fragment_restore_rejects_a_coordinated_anchor_rotation() {
    let request = tree(2, false);
    let mut wire =
        serde_json::to_value(layout_tree(&request).expect("fragment")).expect("serialize");
    let atoms = wire["ring_topology"][0]["atoms"]
        .as_array_mut()
        .expect("atoms");
    atoms.rotate_left(1);
    let atoms = atoms.clone();
    let bonds = wire["ring_topology"][0]["bonds"]
        .as_array_mut()
        .expect("bonds");
    bonds.rotate_left(1);
    let bonds = bonds.clone();
    wire["ring_cycles"][0] = serde_json::Value::Array(bonds.clone());
    wire["ring_topology"][0]["oxygen_atom"] = atoms[0].clone();
    wire["ring_topology"][0]["anomeric_atom"] = atoms[atoms.len() - 1].clone();

    for (index, bond) in bonds.iter().enumerate() {
        let start = atoms[index].clone();
        let end = atoms[(index + 1) % atoms.len()].clone();
        let start_point = wire["coordinates"]
            .as_array()
            .expect("coordinates")
            .iter()
            .find(|entry| entry[0] == start)
            .expect("start coordinate")[1]
            .clone();
        let end_point = wire["coordinates"]
            .as_array()
            .expect("coordinates")
            .iter()
            .find(|entry| entry[0] == end)
            .expect("end coordinate")[1]
            .clone();
        wire["ring_edges"]
            .as_array_mut()
            .expect("ring edges")
            .iter_mut()
            .find(|entry| entry[0] == *bond)
            .expect("ring edge")[1] = serde_json::json!([start, end]);
        wire["bond_geometry"]
            .as_array_mut()
            .expect("bond geometry")
            .iter_mut()
            .find(|entry| entry[0] == *bond)
            .expect("bond geometry")[1] = serde_json::json!([start_point, end_point]);
    }
    crate::haworth::tree::refresh_wire_fingerprint(&mut wire);

    let error = restore(&request, wire).expect_err("rewritten canonical cycle rejected");
    assert_eq!(
        error.to_string(),
        "fragment wire does not match authoritative Haworth request"
    );
}

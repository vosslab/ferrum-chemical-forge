use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};

use crate::haworth::{
    GlycosidicLink, HaworthAttachment, HaworthError, HaworthRingNode, HaworthTopologyBuilder,
    HaworthTreeRequest, HaworthVertex, RingForm, layout_tree,
};

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
    let fragment = layout_tree(&tree(2, false)).expect("fragment");
    let value = serde_json::to_value(&fragment).expect("serialize");
    assert_eq!(
        serde_json::from_value::<crate::haworth::HaworthFragment>(value.clone())
            .expect("validated wire"),
        fragment
    );
    let mut invalid = value;
    invalid["graph_fingerprint"] = serde_json::json!(0);
    assert!(serde_json::from_value::<crate::haworth::HaworthFragment>(invalid).is_err());
}

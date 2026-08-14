use std::collections::BTreeSet;

use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};
use ferrum_domain::haworth::{
    DirectGlycosidicHaworthFragmentRequestV1, DirectGlycosidicHaworthTopologyV1,
    HaworthTopologyBuilder, HaworthVertex, RingForm,
    assemble_direct_glycosidic_haworth_fragment_v1, direct_glycosidic_haworth_depiction_spec_v1,
};
use xot::{Node, Xot};

use crate::direct_glycosidic_haworth::{
    DirectGlycosidicHaworthDrawOpV1, DirectGlycosidicHaworthPathCommandV1,
};
use crate::*;

fn atom(index: usize, element: &str) -> Atom {
    Atom::new(
        Some(Identifier::new(format!("a{index}")).expect("id")),
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
fn bond(index: usize, first: &Atom, second: &Atom) -> Bond {
    Bond::new(
        Some(Identifier::new(format!("b{index}")).expect("id")),
        VertexRef::Atom(first.identity().clone()),
        VertexRef::Atom(second.identity().clone()),
        None,
        Some(BondOrder::Single),
        None,
        Some(false),
        None,
    )
    .expect("bond")
}
fn spec(scale: f64) -> ferrum_domain::haworth::DirectGlycosidicHaworthDepictionSpecV1 {
    let atoms: Vec<_> = (0..12)
        .map(|index| atom(index, if index == 0 || index == 6 { "O" } else { "C" }))
        .chain(std::iter::once(atom(12, "O")))
        .collect();
    let bonds: Vec<_> = (0..6)
        .map(|index| bond(index, &atoms[index], &atoms[(index + 1) % 6]))
        .chain((0..6).map(|index| bond(index + 6, &atoms[index + 6], &atoms[6 + (index + 1) % 6])))
        .chain([
            bond(12, &atoms[1], &atoms[12]),
            bond(13, &atoms[7], &atoms[12]),
        ])
        .collect();
    let molecule = Molecule::new(
        Some(Identifier::new("direct").expect("id")),
        None,
        atoms.clone(),
        vec![],
        vec![],
        vec![],
        bonds.clone(),
        None,
    )
    .expect("molecule");
    let ring = |offset: usize| {
        HaworthTopologyBuilder::new(
            RingForm::Pyranose,
            atoms[offset + 1].identity().clone(),
            atoms[offset..offset + 6]
                .iter()
                .map(|atom| HaworthVertex {
                    atom: atom.identity().clone(),
                })
                .collect::<Vec<_>>(),
        )
        .build(&molecule)
        .expect("ring")
    };
    let topology = DirectGlycosidicHaworthTopologyV1::classify(
        &molecule,
        [ring(0), ring(6)],
        atoms[12].identity().clone(),
        [bonds[12].identity().clone(), bonds[13].identity().clone()],
    )
    .expect("topology");
    let fragment =
        assemble_direct_glycosidic_haworth_fragment_v1(&DirectGlycosidicHaworthFragmentRequestV1 {
            topology,
            scale,
        })
        .expect("fragment");
    direct_glycosidic_haworth_depiction_spec_v1(&fragment).expect("spec")
}
fn request_with(scale: f64) -> DirectGlycosidicHaworthRenderRequestV1 {
    DirectGlycosidicHaworthRenderRequestV1::new(
        RenderProvenance::new(RenderRevision::new(0).expect("revision"), [8; 32]),
        spec(scale),
        Paint::rgb24(Rgb24::new("102030").expect("paint")),
        PositiveFinite::new(scale / 8.0).expect("line"),
        PositiveFinite::new(scale / 2.0).expect("wedge"),
    )
}
pub(crate) fn request() -> DirectGlycosidicHaworthRenderRequestV1 {
    request_with(8.0)
}

fn geometry_values(operation: &DirectGlycosidicHaworthDrawOpV1) -> Vec<f64> {
    let mut values = Vec::new();
    match operation {
        DirectGlycosidicHaworthDrawOpV1::OrdinaryLine {
            endpoints, width, ..
        }
        | DirectGlycosidicHaworthDrawOpV1::HaworthFrontStroke {
            endpoints, width, ..
        } => {
            append_point(&mut values, endpoints[0]);
            append_point(&mut values, endpoints[1]);
            values.push(width.get());
        }
        DirectGlycosidicHaworthDrawOpV1::RoundedFrontWedge {
            tip,
            base,
            width,
            commands,
            ..
        } => {
            append_point(&mut values, *tip);
            append_point(&mut values, *base);
            values.push(width.get());
            for command in commands {
                match command {
                    DirectGlycosidicHaworthPathCommandV1::MoveTo(point)
                    | DirectGlycosidicHaworthPathCommandV1::LineTo(point) => {
                        append_point(&mut values, *point);
                    }
                    DirectGlycosidicHaworthPathCommandV1::CubicTo {
                        control_1,
                        control_2,
                        end,
                    } => {
                        append_point(&mut values, *control_1);
                        append_point(&mut values, *control_2);
                        append_point(&mut values, *end);
                    }
                    DirectGlycosidicHaworthPathCommandV1::Close => {}
                }
            }
        }
    }
    values
}

fn append_point(values: &mut Vec<f64>, point: RenderPoint) {
    values.extend([point.x(), point.y()]);
}
fn children(tree: &Xot, node: Node) -> Vec<Node> {
    tree.children(node)
        .filter(|node| tree.element(*node).is_some())
        .collect()
}
fn attr<'a>(tree: &'a Xot, node: Node, name: &str) -> Option<&'a str> {
    tree.name(name)
        .and_then(|name| tree.get_attribute(node, name))
}

#[test]
fn direct_profile_partitions_closed_targets_and_uses_semantic_tiers() {
    let input = request();
    let plan = lower_direct_glycosidic_haworth_v1(&input).expect("profile");
    let expected: BTreeSet<_> = input
        .spec()
        .ring_bonds()
        .keys()
        .chain(input.spec().bridge_bonds().keys())
        .cloned()
        .collect();
    let actual: BTreeSet<_> = plan
        .operations()
        .iter()
        .map(|op| op.bond().clone())
        .collect();
    assert_eq!(actual, expected);
    assert_eq!(plan.operations().len(), actual.len());
    let first_q = plan
        .operations()
        .iter()
        .position(|op| {
            matches!(
                op,
                DirectGlycosidicHaworthDrawOpV1::HaworthFrontStroke { .. }
            )
        })
        .expect("q tier");
    let first_w = plan
        .operations()
        .iter()
        .position(|op| {
            matches!(
                op,
                DirectGlycosidicHaworthDrawOpV1::RoundedFrontWedge { .. }
            )
        })
        .expect("w tier");
    assert!(
        plan.operations()[..first_q]
            .iter()
            .all(|op| matches!(op, DirectGlycosidicHaworthDrawOpV1::OrdinaryLine { .. }))
    );
    assert!(
        plan.operations()[first_q..first_w]
            .iter()
            .all(|op| matches!(
                op,
                DirectGlycosidicHaworthDrawOpV1::HaworthFrontStroke { .. }
            ))
    );
    assert!(plan.operations()[first_w..].iter().all(|op| matches!(
        op,
        DirectGlycosidicHaworthDrawOpV1::RoundedFrontWedge { .. }
    )));
    let q = plan
        .operations()
        .iter()
        .find(|op| {
            matches!(
                op,
                DirectGlycosidicHaworthDrawOpV1::HaworthFrontStroke { .. }
            )
        })
        .expect("q");
    assert!(matches!(
        q,
        DirectGlycosidicHaworthDrawOpV1::HaworthFrontStroke { width, .. }
            if *width == input.wedge_width()
    ));
    assert!(
        plan.operations()
            .iter()
            .filter_map(|op| match op {
                DirectGlycosidicHaworthDrawOpV1::RoundedFrontWedge { commands, .. } =>
                    Some(commands),
                _ => None,
            })
            .all(|commands| commands.iter().any(|command| matches!(
                command,
                DirectGlycosidicHaworthPathCommandV1::CubicTo { .. }
            )))
    );
    assert!(plan.operations().iter().all(|operation| {
        let source_order = input
            .spec()
            .ring_bonds()
            .get(operation.bond())
            .map(|fact| fact.source_order())
            .or_else(|| {
                input
                    .spec()
                    .bridge_bonds()
                    .get(operation.bond())
                    .map(|fact| fact.source_order())
            });
        source_order
            .and_then(|source_order| u32::try_from(source_order).ok())
            .is_some_and(|source_order| operation.source_order() == source_order)
    }));
}

#[test]
fn direct_profile_geometry_scales_with_coordinates_and_widths() {
    let first = lower_direct_glycosidic_haworth_v1(&request_with(8.0)).expect("first");
    let doubled = lower_direct_glycosidic_haworth_v1(&request_with(16.0)).expect("doubled");
    assert_eq!(first.operations().len(), doubled.operations().len());
    let compared_values = first
        .operations()
        .iter()
        .map(geometry_values)
        .map(|values| values.len())
        .sum::<usize>();
    for (left, right) in first.operations().iter().zip(doubled.operations()) {
        assert_eq!(std::mem::discriminant(left), std::mem::discriminant(right));
        let left_values = geometry_values(left);
        let right_values = geometry_values(right);
        assert_eq!(left_values.len(), right_values.len());
        for (left, right) in left_values.into_iter().zip(right_values) {
            let magnitude = left.abs().max(right.abs()).max(1.0);
            // Each compared primitive is built from a bounded profile. This allowance
            // scales machine epsilon by both coordinate magnitude and the complete
            // emitted primitive set, avoiding a scale-specific absolute threshold.
            let allowance = f64::EPSILON * magnitude * compared_values as f64;
            assert!((right - 2.0 * left).abs() <= allowance);
        }
    }
}

#[test]
fn direct_svg_keeps_round_q_and_filled_rounded_w_edges_structural() {
    let plan = lower_direct_glycosidic_haworth_v1(&request()).expect("profile");
    let document = render_direct_glycosidic_haworth_to_svg_v1(
        &plan,
        SvgViewportV1::new(-20.0, -20.0, 40.0, 40.0).expect("viewport"),
    )
    .expect("svg");
    let mut tree = Xot::new();
    let parsed = tree.parse(document.as_str()).expect("xot");
    let root = tree.document_element(parsed).expect("root");
    let elements: Vec<_> = children(&tree, root);
    let q = elements
        .iter()
        .copied()
        .find(|node| attr(&tree, *node, "data-ferrum-direct-glycosidic") == Some("q1"))
        .expect("q");
    assert_eq!(attr(&tree, q, "stroke-linecap"), Some("round"));
    let wedges: Vec<_> = elements
        .iter()
        .copied()
        .filter(|node| attr(&tree, *node, "data-ferrum-direct-glycosidic") == Some("w1"))
        .collect();
    assert!(!wedges.is_empty());
    assert!(
        wedges
            .iter()
            .all(|node| attr(&tree, *node, "fill") == Some("#102030")
                && attr(&tree, *node, "stroke") == Some("none"))
    );
}

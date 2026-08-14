use crate::repair::{
    DepictionGraph, RepairError, RepairKind, RepairRequest, plan_repair, plan_repair_with_outcome,
};

use super::fixtures::{bond, id, point, vertex};

fn assert_roundoff_close(actual: f64, expected: f64) {
    let scale = actual.abs().max(expected.abs()).max(1.0);
    assert!((actual - expected).abs() <= f64::EPSILON * 16.0 * scale);
}

#[test]
fn snap_returns_sparse_patch_without_mutating_source_graph() {
    let graph = DepictionGraph::new(
        vec![vertex("a", 0.2, 0.2), vertex("b", 0.0, 0.0)],
        vec![bond("ab", "a", "b")],
    )
    .expect("graph must validate");
    let original = graph.clone();
    let request = RepairRequest::new(
        graph,
        RepairKind::SnapToHexGrid {
            spacing: 1.0,
            origin: point(0.0, 0.0),
        },
    );
    let patch = plan_repair(&request).expect("snap must be representable");
    assert_eq!(patch.replacements().len(), 1);
    assert_eq!(
        patch.replacements().next().map(|(id, _)| id),
        Some(&id(ferrum_core::RecordKind::Atom, "a"))
    );
    assert_eq!(request.graph(), &original);
}

#[test]
fn snap_tie_break_is_deterministic_and_uses_identity_order() {
    let graph =
        DepictionGraph::new(vec![vertex("atom", 0.0, 0.5)], vec![]).expect("graph must validate");
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::SnapToHexGrid {
            spacing: 1.0,
            origin: point(0.0, 0.0),
        },
    ))
    .expect("snap must be representable");
    let (_, snapped) = patch.replacements().next().expect("coordinate should move");
    assert_eq!(snapped.expected(), point(0.0, 0.5));
    assert_eq!(snapped.replacement(), point(0.0, 0.0));
}

#[test]
fn already_snapped_graph_produces_successful_empty_patch() {
    let graph =
        DepictionGraph::new(vec![vertex("atom", 0.0, 0.0)], vec![]).expect("graph must validate");
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::SnapToHexGrid {
            spacing: 1.0,
            origin: point(0.0, 0.0),
        },
    ))
    .expect("snap must be representable");
    assert!(patch.is_empty());
}

#[test]
fn straighten_uses_durable_sorted_ids_and_returns_coordinate_patch() {
    let graph = DepictionGraph::new(
        vec![vertex("z", 0.0, 0.0), vertex("a", 1.0, 0.3)],
        vec![bond("az", "a", "z")],
    )
    .expect("graph must validate");
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::Straighten {
            minimize_rotation: false,
        },
    ))
    .expect("straightening must be representable");
    assert!(!patch.is_empty());
    assert_eq!(patch.replacements().count(), 1);
    assert_eq!(
        patch.replacements().next().map(|(id, _)| id),
        Some(&id(ferrum_core::RecordKind::Atom, "a"))
    );
}

#[test]
fn straighten_outcome_retains_full_result_before_sparse_patch_filtering() {
    let graph = DepictionGraph::new(
        vec![vertex("b", 1.0, 0.0), vertex("a", 0.0, 0.0)],
        vec![bond("ab", "a", "b")],
    )
    .expect("graph must validate");
    let request = RepairRequest::new(
        graph,
        RepairKind::Straighten {
            minimize_rotation: true,
        },
    );
    let outcome = plan_repair_with_outcome(&request).expect("straightening must be representable");
    assert_eq!(outcome.applied_rotation_radians(), Some(0.0));
    assert!(outcome.patch().is_empty());
    assert_eq!(
        outcome
            .straightened_coordinates()
            .expect("straightening retains complete coordinates")
            .map(|(atom_id, point)| (atom_id.clone(), point))
            .collect::<Vec<_>>(),
        vec![
            (id(ferrum_core::RecordKind::Atom, "a"), point(0.0, 0.0)),
            (id(ferrum_core::RecordKind::Atom, "b"), point(1.0, 0.0)),
        ]
    );
}

#[test]
fn straighten_outcome_preserves_sparse_patch_compatibility_and_omits_unrelated_metadata() {
    let graph = DepictionGraph::new(
        vec![vertex("a", 0.0, 0.0), vertex("b", 1.0, 1.0)],
        vec![bond("ab", "a", "b")],
    )
    .expect("graph must validate");
    let request = RepairRequest::new(
        graph,
        RepairKind::Straighten {
            minimize_rotation: false,
        },
    );
    let outcome = plan_repair_with_outcome(&request).expect("straightening must be representable");
    assert_eq!(
        outcome.patch(),
        &plan_repair(&request).expect("compatibility patch must plan")
    );
    assert!(
        outcome
            .applied_rotation_radians()
            .is_some_and(|angle| angle != 0.0)
    );

    let unrelated = plan_repair_with_outcome(&RepairRequest::new(
        DepictionGraph::new(vec![vertex("atom", 0.0, 0.0)], vec![]).expect("graph must validate"),
        RepairKind::SnapToHexGrid {
            spacing: 1.0,
            origin: point(0.0, 0.0),
        },
    ))
    .expect("snap must be representable");
    assert_eq!(unrelated.applied_rotation_radians(), None);
    assert!(unrelated.straightened_coordinates().is_none());
}

#[test]
fn bridged_topology_reaches_whole_depiction_straightening() {
    let graph = DepictionGraph::new(
        vec![
            vertex("left", 0.0, 0.0),
            vertex("right", 2.0, 0.0),
            vertex("upper", 1.0, 1.0),
            vertex("lower", 1.0, -1.0),
            vertex("middle", 1.0, 0.0),
        ],
        vec![
            bond("left_upper", "left", "upper"),
            bond("upper_right", "upper", "right"),
            bond("left_lower", "left", "lower"),
            bond("lower_right", "lower", "right"),
            bond("left_middle", "left", "middle"),
            bond("middle_right", "middle", "right"),
        ],
    )
    .expect("topology-independent graph validation accepts a bridged graph");
    assert!(
        plan_repair_with_outcome(&RepairRequest::new(
            graph,
            RepairKind::Straighten {
                minimize_rotation: false,
            },
        ))
        .is_ok()
    );
}

#[test]
fn terminal_straightening_moves_only_degree_one_endpoints() {
    let graph = DepictionGraph::new(
        vec![
            vertex("a", 0.0, 0.0),
            vertex("b", 1.0, 0.0),
            vertex("c", 2.0, 0.4),
        ],
        vec![bond("ab", "a", "b"), bond("bc", "b", "c")],
    )
    .expect("graph must validate");
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::StraightenTerminalBonds,
    ))
    .expect("terminal bonds must be representable");
    assert_eq!(patch.replacements().count(), 1);
    let (atom_id, replacement) = patch.replacements().next().expect("c must move");
    assert_eq!(atom_id, &id(ferrum_core::RecordKind::Atom, "c"));
    assert_eq!(replacement.expected(), point(2.0, 0.4));
    let length = 1.0_f64.hypot(0.4);
    assert_eq!(
        replacement.replacement(),
        point(1.0 + length * 3.0_f64.sqrt() / 2.0, length / 2.0)
    );
}

#[test]
fn terminal_straightening_uses_increasing_half_slot_and_lexical_two_atom_anchor() {
    let half_slot = std::f64::consts::PI / 12.0;
    let graph = DepictionGraph::new(
        vec![
            vertex("z", half_slot.cos(), half_slot.sin()),
            vertex("a", 0.0, 0.0),
        ],
        vec![bond("az", "a", "z")],
    )
    .expect("graph must validate");
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::StraightenTerminalBonds,
    ))
    .expect("terminal bond must be representable");
    assert_eq!(patch.replacements().count(), 1);
    let (atom_id, replacement) = patch.replacements().next().expect("z must move");
    assert_eq!(atom_id, &id(ferrum_core::RecordKind::Atom, "z"));
    assert_eq!(replacement.replacement(), point(3.0_f64.sqrt() / 2.0, 0.5));
}

#[test]
fn terminal_straightening_leaves_degenerate_endpoint_unchanged() {
    let graph = DepictionGraph::new(
        vec![vertex("a", 0.0, 0.0), vertex("b", 0.0, 0.0)],
        vec![bond("ab", "a", "b")],
    )
    .expect("graph must validate");
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::StraightenTerminalBonds,
    ))
    .expect("degenerate terminal vector is a no-op");
    assert!(patch.is_empty());
}

#[test]
fn length_normalization_preserves_tree_directions_from_durable_root() {
    let graph = DepictionGraph::new(
        vec![
            vertex("a", -2.0, 0.0),
            vertex("b", 0.0, 0.0),
            vertex("c", 0.0, 3.0),
        ],
        vec![bond("ab", "a", "b"), bond("bc", "b", "c")],
    )
    .expect("graph must validate");
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::NormalizeBondLengths { spacing: 1.0 },
    ))
    .expect("tree lengths must be representable");
    let replacements = patch.replacements().collect::<Vec<_>>();
    assert_eq!(replacements.len(), 2);
    assert_eq!(replacements[0].0, &id(ferrum_core::RecordKind::Atom, "a"));
    assert_eq!(replacements[0].1.replacement(), point(-1.0, 0.0));
    assert_eq!(replacements[1].0, &id(ferrum_core::RecordKind::Atom, "c"));
    assert_eq!(replacements[1].1.replacement(), point(0.0, 1.0));
}

#[test]
fn length_normalization_fixes_ring_and_uses_east_for_degenerate_pair() {
    let graph = DepictionGraph::new(
        vec![
            vertex("a", 0.0, 0.0),
            vertex("b", 2.0, 0.0),
            vertex("c", 1.0, 1.0),
            vertex("d", 4.0, 0.0),
        ],
        vec![
            bond("ab", "a", "b"),
            bond("bc", "b", "c"),
            bond("ca", "c", "a"),
            bond("bd", "b", "d"),
        ],
    )
    .expect("single-cycle graph must validate");
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::NormalizeBondLengths { spacing: 1.0 },
    ))
    .expect("ring substituent must be representable");
    assert_eq!(patch.replacements().count(), 1);
    let (atom_id, replacement) = patch.replacements().next().expect("d must move");
    assert_eq!(atom_id, &id(ferrum_core::RecordKind::Atom, "d"));
    assert_eq!(replacement.replacement(), point(3.0, 0.0));

    let degenerate = DepictionGraph::new(
        vec![vertex("z", 0.0, 0.0), vertex("a", 0.0, 0.0)],
        vec![bond("az", "a", "z")],
    )
    .expect("degenerate graph remains structurally valid");
    let patch = plan_repair(&RepairRequest::new(
        degenerate,
        RepairKind::NormalizeBondLengths { spacing: 2.0 },
    ))
    .expect("degenerate direction uses the documented east fallback");
    let (atom_id, replacement) = patch.replacements().next().expect("z must move");
    assert_eq!(atom_id, &id(ferrum_core::RecordKind::Atom, "z"));
    assert_eq!(replacement.replacement(), point(2.0, 0.0));
}

#[test]
fn ring_normalization_uses_canonical_walk_centroid_and_rigid_substituent_shift() {
    let graph = DepictionGraph::new(
        vec![
            vertex("a", 0.0, 0.0),
            vertex("b", 2.0, 0.0),
            vertex("c", 1.5, 1.0),
            vertex("d", 0.0, 1.0),
            vertex("side", -1.0, 1.0),
        ],
        vec![
            bond("ab", "a", "b"),
            bond("bc", "b", "c"),
            bond("cd", "c", "d"),
            bond("da", "d", "a"),
            bond("ds", "d", "side"),
        ],
    )
    .expect("one simple ring validates");
    let original = graph.clone();
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::NormalizeSingleRing { spacing: 2.0 },
    ))
    .expect("ring repair must be representable");
    let replacements = patch
        .replacements()
        .map(|(id, replacement)| (id.clone(), replacement))
        .collect::<std::collections::BTreeMap<_, _>>();
    let ring_ids = ["a", "b", "c", "d"].map(|value| id(ferrum_core::RecordKind::Atom, value));
    let centroid = ring_ids.iter().fold((0.0, 0.0), |(x, y), atom_id| {
        let point = replacements[atom_id].replacement();
        (x + point.x() / 4.0, y + point.y() / 4.0)
    });
    assert_roundoff_close(centroid.0, 0.875);
    assert_roundoff_close(centroid.1, 0.5);
    for index in 0..ring_ids.len() {
        let left = replacements[&ring_ids[index]].replacement();
        let right = replacements[&ring_ids[(index + 1) % ring_ids.len()]].replacement();
        assert_roundoff_close(left.distance_to(right), 2.0);
    }
    let old_d = original.coordinates()[&ring_ids[3]];
    let new_d = replacements[&ring_ids[3]].replacement();
    let old_side = original.coordinates()[&id(ferrum_core::RecordKind::Atom, "side")];
    let new_side = replacements[&id(ferrum_core::RecordKind::Atom, "side")].replacement();
    assert_eq!(
        (new_side.x() - old_side.x(), new_side.y() - old_side.y()),
        (new_d.x() - old_d.x(), new_d.y() - old_d.y())
    );
}

#[test]
fn ring_normalization_is_noop_without_cycle_and_rejects_unanchored_component() {
    let tree = DepictionGraph::new(
        vec![vertex("a", 0.0, 0.0), vertex("b", 2.0, 0.0)],
        vec![bond("ab", "a", "b")],
    )
    .expect("tree validates");
    assert!(
        plan_repair(&RepairRequest::new(
            tree,
            RepairKind::NormalizeSingleRing { spacing: 1.0 },
        ))
        .expect("ring-free repair is a no-op")
        .is_empty()
    );

    let disconnected = DepictionGraph::new(
        vec![
            vertex("a", 0.0, 0.0),
            vertex("b", 1.0, 0.0),
            vertex("c", 0.5, 1.0),
            vertex("orphan", 5.0, 5.0),
        ],
        vec![
            bond("ab", "a", "b"),
            bond("bc", "b", "c"),
            bond("ca", "c", "a"),
        ],
    )
    .expect("one cycle plus isolated atom validates structurally");
    assert!(matches!(
        plan_repair(&RepairRequest::new(
            disconnected,
            RepairKind::NormalizeSingleRing { spacing: 1.0 },
        )),
        Err(crate::repair::RepairError::UnsupportedTopology(_))
    ));
}

#[test]
fn single_ring_normalization_rejects_disconnected_multiple_cycles() {
    let graph = DepictionGraph::new(
        vec![
            vertex("a", 0.0, 0.0),
            vertex("b", 1.0, 0.0),
            vertex("c", 0.5, 1.0),
            vertex("d", 3.0, 0.0),
            vertex("e", 4.0, 0.0),
            vertex("f", 3.5, 1.0),
        ],
        vec![
            bond("ab", "a", "b"),
            bond("bc", "b", "c"),
            bond("ca", "c", "a"),
            bond("de", "d", "e"),
            bond("ef", "e", "f"),
            bond("fd", "f", "d"),
        ],
    )
    .expect("topology-independent graph validation accepts disconnected cycles");
    assert_eq!(
        plan_repair(&RepairRequest::new(
            graph,
            RepairKind::NormalizeSingleRing { spacing: 1.0 },
        )),
        Err(RepairError::UnsupportedTopology(
            "single-ring normalization supports exactly one independent cycle"
        ))
    );
}

#[test]
fn angle_normalization_assigns_contested_slots_in_authored_bond_order() {
    let graph = DepictionGraph::new(
        vec![
            vertex("root", 0.0, 0.0),
            vertex("z_first", 1.0, -0.1),
            vertex("a_second", 1.0, -0.2),
        ],
        vec![
            bond("z_first_bond", "root", "z_first"),
            bond("a_second_bond", "root", "a_second"),
        ],
    )
    .expect("source-ordered tree validates");
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::NormalizeBondAngles { spacing: 2.0 },
    ))
    .expect("two outgoing bonds have distinct slots");
    let replacements = patch
        .replacements()
        .map(|(atom_id, replacement)| (atom_id.clone(), replacement.replacement()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let first_distance = 1.0_f64.hypot(0.1);
    let second_distance = 1.0_f64.hypot(0.2);
    let first = replacements[&id(ferrum_core::RecordKind::Atom, "z_first")];
    let second = replacements[&id(ferrum_core::RecordKind::Atom, "a_second")];
    assert_roundoff_close(first.x(), first_distance);
    assert_roundoff_close(first.y(), 0.0);
    assert_roundoff_close(second.x(), second_distance / 2.0);
    assert_roundoff_close(second.y(), -second_distance * 3.0_f64.sqrt() / 2.0);
}

#[test]
fn angle_normalization_rounds_authored_half_slots_forward_and_uses_degenerate_spacing() {
    let half_slot = std::f64::consts::PI / 6.0;
    let graph = DepictionGraph::new(
        vec![
            vertex("root", 0.0, 0.0),
            vertex("half", half_slot.cos(), -half_slot.sin()),
        ],
        vec![bond("root_half", "root", "half")],
    )
    .expect("half-slot graph validates");
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::NormalizeBondAngles { spacing: 2.0 },
    ))
    .expect("represented half slot advances");
    let (_, replacement) = patch.replacements().next().expect("half must move");
    assert_roundoff_close(replacement.replacement().x(), 0.5);
    assert_roundoff_close(replacement.replacement().y(), -3.0_f64.sqrt() / 2.0);

    let degenerate = DepictionGraph::new(
        vec![vertex("root", 0.0, 0.0), vertex("child", 0.0, 0.0)],
        vec![bond("root_child", "root", "child")],
    )
    .expect("coincident atoms remain structurally valid");
    let patch = plan_repair(&RepairRequest::new(
        degenerate,
        RepairKind::NormalizeBondAngles { spacing: 2.0 },
    ))
    .expect("coincident outgoing atom uses explicit spacing");
    let (_, replacement) = patch.replacements().next().expect("child must move");
    assert_eq!(replacement.replacement(), point(2.0, 0.0));
}

#[test]
fn angle_normalization_fixes_ring_and_its_anchor_edge() {
    let graph = DepictionGraph::new(
        vec![
            vertex("a", 0.0, 0.0),
            vertex("b", 2.0, 0.0),
            vertex("c", 1.0, 1.0),
            vertex("root", -1.0, 0.0),
            vertex("child", -2.0, 0.2),
        ],
        vec![
            bond("ab", "a", "b"),
            bond("bc", "b", "c"),
            bond("ca", "c", "a"),
            bond("a_root", "a", "root"),
            bond("root_child", "root", "child"),
        ],
    )
    .expect("single ring with one anchored branch validates");
    let patch = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::NormalizeBondAngles { spacing: 1.0 },
    ))
    .expect("anchored branch has one movable outgoing bond");
    assert_eq!(patch.replacements().count(), 1);
    let (atom_id, replacement) = patch.replacements().next().expect("child must move");
    assert_eq!(atom_id, &id(ferrum_core::RecordKind::Atom, "child"));
    assert_roundoff_close(replacement.replacement().x(), -1.0 - 1.0_f64.hypot(0.2));
    assert_roundoff_close(replacement.replacement().y(), 0.0);
}

#[test]
fn angle_normalization_rejects_a_parent_without_a_free_slot() {
    let mut vertices = vec![vertex("root", 0.0, 0.0)];
    let mut bonds = Vec::new();
    for index in 0..7 {
        let child = format!("child_{index}");
        vertices.push(vertex(&child, 1.0, 0.0));
        bonds.push(bond(&format!("bond_{index}"), "root", &child));
    }
    let graph = DepictionGraph::new(vertices, bonds).expect("seven-branch tree validates");
    assert!(matches!(
        plan_repair(&RepairRequest::new(
            graph,
            RepairKind::NormalizeBondAngles { spacing: 1.0 },
        )),
        Err(crate::repair::RepairError::UnsupportedTopology(_))
    ));
}

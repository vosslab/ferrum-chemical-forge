use crate::repair::{DepictionGraph, RepairKind, RepairRequest, plan_repair};

use super::fixtures::{bond, id, point, vertex};

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

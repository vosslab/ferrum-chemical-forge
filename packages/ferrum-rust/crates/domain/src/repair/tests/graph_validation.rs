use crate::repair::{
    DepictionGraph, RepairError, RepairKind, RepairRequest, plan_repair, plan_repair_with_outcome,
};

use super::fixtures::{bond, vertex};

#[test]
fn invalid_endpoint_and_parallel_bond_are_rejected() {
    let missing = DepictionGraph::new(
        vec![vertex("a", 0.0, 0.0)],
        vec![bond("missing", "a", "not-present")],
    )
    .expect_err("missing endpoint must fail");
    assert_eq!(
        missing,
        RepairError::InvalidGraph("depiction bond endpoints must belong to the graph")
    );
    let parallel = DepictionGraph::new(
        vec![vertex("a", 0.0, 0.0), vertex("b", 1.0, 0.0)],
        vec![bond("ab-one", "a", "b"), bond("ab-two", "b", "a")],
    )
    .expect_err("parallel endpoints must fail");
    assert_eq!(
        parallel,
        RepairError::InvalidGraph("depiction graph cannot contain parallel bond endpoints")
    );
}

#[test]
fn multi_cycle_graph_reaches_whole_depiction_straightening_but_not_single_ring_normalization() {
    let graph = DepictionGraph::new(
        vec![
            vertex("a", 0.0, 0.0),
            vertex("b", 1.0, 0.0),
            vertex("c", 1.0, 1.0),
            vertex("d", 0.0, 1.0),
        ],
        vec![
            bond("ab", "a", "b"),
            bond("bc", "b", "c"),
            bond("cd", "c", "d"),
            bond("da", "d", "a"),
            bond("ac", "a", "c"),
        ],
    )
    .expect("topology-independent graph validation accepts a fused cycle graph");
    assert!(
        plan_repair_with_outcome(&RepairRequest::new(
            graph.clone(),
            RepairKind::Straighten {
                minimize_rotation: true,
            },
        ))
        .is_ok()
    );
    let error = plan_repair(&RepairRequest::new(
        graph,
        RepairKind::NormalizeSingleRing { spacing: 1.0 },
    ))
    .expect_err("single-ring normalization must not choose one cycle from a fused graph");
    assert_eq!(
        error,
        RepairError::UnsupportedTopology(
            "single-ring normalization supports exactly one independent cycle"
        )
    );
}

use crate::repair::{DepictionGraph, RepairError};

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
fn multi_cycle_topology_is_declined_before_normalization() {
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
    .expect_err("two independent cycles need a future profile");
    assert_eq!(
        graph,
        RepairError::UnsupportedTopology(
            "initial repair profile supports at most one independent cycle"
        )
    );
}

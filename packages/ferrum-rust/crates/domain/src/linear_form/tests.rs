use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_geometry::Point2;

use super::*;

fn atom(name: &str, x: f64, y: f64) -> LinearFormAtomV1 {
    LinearFormAtomV1::new(
        id(RecordKind::Atom, name),
        Point2::new(x, y).expect("finite point"),
    )
}

fn bond(name: &str, start: &str, end: &str) -> LinearFormBondV1 {
    LinearFormBondV1::new(
        id(RecordKind::Bond, name),
        id(RecordKind::Atom, start),
        id(RecordKind::Atom, end),
    )
}

fn id(kind: RecordKind, name: &str) -> RecordId {
    RecordId::from_source(kind, &Identifier::new(name).expect("identifier"))
}

fn request(
    selected: &[&str],
    atoms: Vec<LinearFormAtomV1>,
    bonds: Vec<LinearFormBondV1>,
) -> LinearFormRequestV1 {
    LinearFormRequestV1::new(
        selected
            .iter()
            .map(|name| id(RecordKind::Atom, name))
            .collect(),
        LinearFormGraphV1::new(atoms, bonds),
    )
}

#[test]
fn single_atom_keeps_its_point_and_enables_hydrogen() {
    let plan =
        plan_linear_form_v1(&request(&["a"], vec![atom("a", 3.0, 4.0)], vec![])).expect("plan");
    assert_eq!(
        plan.selected_replacements()[0].point(),
        Point2::new(3.0, 4.0).expect("finite")
    );
    assert_eq!(plan.hydrogen_visible_atoms(), plan.ordered_atoms());
    assert!(plan.ordered_bonds().is_empty());
}

#[test]
fn source_order_controls_direction_and_fixed_spacing() {
    let plan = plan_linear_form_v1(&request(
        &["z", "a"],
        vec![atom("z", 90.0, 12.0), atom("a", -3.0, -8.0)],
        vec![bond("za", "z", "a")],
    ))
    .expect("plan");
    assert_eq!(
        plan.ordered_atoms(),
        &[id(RecordKind::Atom, "z"), id(RecordKind::Atom, "a")]
    );
    assert_eq!(
        plan.selected_replacements()[1].point(),
        Point2::new(100.0, 12.0).expect("finite")
    );
    assert_eq!(plan.metadata().atom_members(), plan.ordered_atoms());
    assert_eq!(plan.metadata().bond_members(), plan.ordered_bonds());
}

#[test]
fn source_order_directs_three_atom_path_and_canonical_bond_order() {
    let plan = plan_linear_form_v1(&request(
        &["c", "a", "b"],
        vec![
            atom("c", 80.0, 5.0),
            atom("a", 0.0, 0.0),
            atom("b", 40.0, 3.0),
        ],
        vec![bond("ab", "a", "b"), bond("bc", "b", "c")],
    ))
    .expect("plan");
    assert_eq!(
        plan.ordered_atoms(),
        &[
            id(RecordKind::Atom, "c"),
            id(RecordKind::Atom, "b"),
            id(RecordKind::Atom, "a"),
        ]
    );
    assert_eq!(
        plan.ordered_bonds(),
        &[id(RecordKind::Bond, "bc"), id(RecordKind::Bond, "ab")]
    );
}

#[test]
fn rejects_parallel_fork_ring_and_disconnected_induced_selections() {
    let atoms = vec![
        atom("a", 0.0, 0.0),
        atom("b", 1.0, 0.0),
        atom("c", 2.0, 0.0),
        atom("d", 3.0, 0.0),
    ];
    for bonds in [
        vec![bond("ab1", "a", "b"), bond("ab2", "a", "b")],
        vec![
            bond("ab", "a", "b"),
            bond("ac", "a", "c"),
            bond("ad", "a", "d"),
        ],
        vec![
            bond("ab", "a", "b"),
            bond("bc", "b", "c"),
            bond("ca", "c", "a"),
        ],
        vec![bond("ab", "a", "b"), bond("cd", "c", "d")],
    ] {
        assert_eq!(
            plan_linear_form_v1(&request(&["a", "b", "c", "d"], atoms.clone(), bonds)),
            Err(LinearFormPlanErrorV1::NotSinglePath)
        );
    }
}

#[test]
fn uniquely_anchored_exterior_component_translates_with_its_path_atom() {
    let plan = plan_linear_form_v1(&request(
        &["a", "b"],
        vec![
            atom("a", 0.0, 0.0),
            atom("b", 20.0, 0.0),
            atom("x", 20.0, 5.0),
            atom("y", 23.0, 5.0),
        ],
        vec![
            bond("ab", "a", "b"),
            bond("bx", "b", "x"),
            bond("xy", "x", "y"),
        ],
    ))
    .expect("plan");
    assert_eq!(
        plan.exterior_replacements()[0].point(),
        Point2::new(10.0, 5.0).expect("finite")
    );
    assert_eq!(
        plan.exterior_replacements()[1].point(),
        Point2::new(13.0, 5.0).expect("finite")
    );
}

#[test]
fn refuses_an_exterior_bridge_with_two_path_anchors() {
    let result = plan_linear_form_v1(&request(
        &["a", "b"],
        vec![
            atom("a", 0.0, 0.0),
            atom("b", 20.0, 0.0),
            atom("x", 10.0, 5.0),
        ],
        vec![
            bond("ab", "a", "b"),
            bond("ax", "a", "x"),
            bond("xb", "x", "b"),
        ],
    ));
    assert_eq!(
        result,
        Err(LinearFormPlanErrorV1::ExteriorComponentHasMultipleAnchors)
    );
}

#[test]
fn rejects_duplicate_and_foreign_selected_atoms() {
    let graph = LinearFormGraphV1::new(vec![atom("a", 0.0, 0.0)], vec![]);
    assert_eq!(
        plan_linear_form_v1(&LinearFormRequestV1::new(
            vec![id(RecordKind::Atom, "a"), id(RecordKind::Atom, "a")],
            graph.clone()
        )),
        Err(LinearFormPlanErrorV1::DuplicateAtomId)
    );
    assert_eq!(
        plan_linear_form_v1(&LinearFormRequestV1::new(
            vec![id(RecordKind::Atom, "foreign")],
            graph
        )),
        Err(LinearFormPlanErrorV1::UnknownOrForeignAtom)
    );
}

#[test]
fn rejects_duplicate_durable_bond_ids_before_three_atom_path_planning() {
    let result = plan_linear_form_v1(&request(
        &["a", "b", "c"],
        vec![
            atom("a", 0.0, 0.0),
            atom("b", 1.0, 0.0),
            atom("c", 2.0, 0.0),
        ],
        vec![bond("b1", "a", "b"), bond("b1", "b", "c")],
    ));
    assert_eq!(result, Err(LinearFormPlanErrorV1::DuplicateBondId));
}

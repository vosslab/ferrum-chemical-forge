use super::{
    DocumentSession, DocumentSessionError, MoleculeInsertionAtomV1, MoleculeInsertionBondOrderV1,
    MoleculeInsertionBondV1, MoleculeInsertionV1, MoleculeInsertionV1Error, Point3V1,
};

const SOURCE: &str = concat!(
    "<cdml version=\"1.0\"><opaque id=\"ferrum-molecule-v1-0\"/>",
    "<opaque id=\"ferrum-atom-v1-0\"/><opaque id=\"ferrum-bond-v1-0\"/></cdml>"
);

fn atom(
    element: &str,
    x: f64,
    charge: Option<i32>,
    isotope: Option<u16>,
    hydrogens: Option<u16>,
) -> MoleculeInsertionAtomV1 {
    MoleculeInsertionAtomV1::new(
        element,
        Point3V1::new(x, 20.0, 0.0).expect("test position is finite"),
        charge,
        isotope,
        hydrogens,
    )
    .expect("test atom is valid")
}

fn carbonyl() -> MoleculeInsertionV1 {
    MoleculeInsertionV1::new(
        vec![
            atom("C", 10.0, None, None, None),
            atom("O", 30.0, Some(-1), Some(18), Some(1)),
        ],
        vec![MoleculeInsertionBondV1::new(
            0,
            1,
            MoleculeInsertionBondOrderV1::Double,
        )],
    )
    .expect("test graph is valid")
}

#[test]
fn insertion_graph_rejects_ambiguous_or_impossible_edges() {
    let atoms = vec![atom("C", 0.0, None, None, None)];
    let self_bond = MoleculeInsertionV1::new(
        atoms.clone(),
        vec![MoleculeInsertionBondV1::new(
            0,
            0,
            MoleculeInsertionBondOrderV1::Single,
        )],
    );
    assert_eq!(
        self_bond,
        Err(MoleculeInsertionV1Error::SelfBond { atom: 0 })
    );
    assert_eq!(
        MoleculeInsertionV1::new(Vec::new(), Vec::new()),
        Err(MoleculeInsertionV1Error::EmptyMolecule)
    );
}

#[test]
fn complete_insertion_allocates_collision_free_ids_and_projects_exact_facts() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture must load");
    let mut pending = session
        .prepare_create_molecule_v1(0, &carbonyl())
        .expect("complete candidate must prepare");
    assert_eq!(
        pending.molecule_identifier().as_str(),
        "ferrum-molecule-v1-1"
    );
    assert_eq!(pending.atom_identifiers()[0].as_str(), "ferrum-atom-v1-1");
    assert_eq!(pending.bond_identifiers()[0].as_str(), "ferrum-bond-v1-1");

    let accepted = session
        .commit_create_molecule(0, &mut pending)
        .expect("prepared molecule must commit");
    let observation = accepted.observation();
    let inserted = &observation.projection().molecules()[0];
    assert_eq!(inserted.source_id(), Some("ferrum-molecule-v1-1"));
    assert_eq!(inserted.atoms()[1].element(), Some("O"));
    assert_eq!(inserted.atoms()[1].formal_charge(), Some(-1));
    assert_eq!(inserted.atoms()[1].explicit_hydrogens(), Some(1));
    assert_eq!(inserted.bonds()[0].source_type(), Some("n2"));
    assert!(observation.snapshot().cdml().contains("isotope=\"18\""));
}

#[test]
fn prepared_molecule_is_owner_bound_consumed_once_and_history_restorable() {
    let mut owner =
        DocumentSession::load("<cdml version=\"1.0\"/>").expect("owner fixture must load");
    let mut foreign =
        DocumentSession::load("<cdml version=\"1.0\"/>").expect("foreign fixture must load");
    let mut pending = owner
        .prepare_create_molecule_v1(0, &carbonyl())
        .expect("candidate must prepare");
    assert!(matches!(
        foreign.commit_create_molecule(0, &mut pending),
        Err(DocumentSessionError::PreparedOperationForeignSession)
    ));
    let accepted = owner
        .commit_create_molecule(0, &mut pending)
        .expect("owner can still accept candidate");
    assert!(matches!(
        owner.commit_create_molecule(accepted.observation().snapshot().revision(), &mut pending),
        Err(DocumentSessionError::PreparedOperationConsumed)
    ));
    let undone = owner.undo(1).expect("accepted molecule must be undoable");
    assert!(undone.observation().projection().molecules().is_empty());
    let redone = owner.redo(2).expect("accepted molecule must be redoable");
    assert_eq!(
        redone.observation().projection().molecules()[0].source_id(),
        Some("ferrum-molecule-v1-0")
    );
}

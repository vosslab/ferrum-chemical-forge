use super::{
    AdmittedSessionTransitionRefusalV1, DocumentBondOrderV1, DocumentSession,
    MoleculeInsertionAtomV1, MoleculeInsertionBondV1, MoleculeInsertionV1,
    MoleculeInsertionV1Error, Point3V1, SessionOperation, SessionOperationOutcomeV1,
    SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"><opaque id=\"ferrum-molecule-v1-0\"/>",
    "<opaque id=\"ferrum-atom-v1-0\"/><opaque id=\"ferrum-bond-v1-0\"/></cdml>"
);

fn atom(element: &str, x: f64) -> MoleculeInsertionAtomV1 {
    MoleculeInsertionAtomV1::new(
        element,
        Point3V1::new(x, 20.0, 0.0).expect("finite test position"),
        None,
        None,
        None,
    )
    .expect("valid test atom")
}

fn carbonyl() -> MoleculeInsertionV1 {
    MoleculeInsertionV1::new(
        vec![atom("C", 10.0), atom("O", 30.0)],
        vec![MoleculeInsertionBondV1::new(
            0,
            1,
            DocumentBondOrderV1::Double,
        )],
    )
    .expect("valid test graph")
}

fn request(revision: u64, molecule: MoleculeInsertionV1) -> SessionOperationTransitionRequestV1 {
    SessionOperationTransitionRequestV1::new(
        revision,
        SessionOperation::V1(SessionOperationV1::InsertMoleculeV1(molecule)),
        TransitionAuthorizationV1::None,
    )
}

#[test]
fn insertion_graph_rejects_ambiguous_or_impossible_edges() {
    assert_eq!(
        MoleculeInsertionV1::new(
            vec![atom("C", 0.0)],
            vec![MoleculeInsertionBondV1::new(
                0,
                0,
                DocumentBondOrderV1::Single,
            )],
        ),
        Err(MoleculeInsertionV1Error::SelfBond { atom: 0 })
    );
    assert_eq!(
        MoleculeInsertionV1::new(Vec::new(), Vec::new()),
        Err(MoleculeInsertionV1Error::EmptyMolecule)
    );
}

#[test]
fn generic_molecule_insertion_publishes_ids_only_after_commit_and_is_one_history_step() {
    let mut session = DocumentSession::load(SOURCE).expect("source loads");
    let baseline = session.snapshot().expect("baseline snapshot");
    let mut prepared = session
        .prepare_session_operation_transition_v1(request(0, carbonyl()))
        .expect("generic transition prepares");
    assert_eq!(session.snapshot().expect("preparation is inert"), baseline);

    let accepted = session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("generic transition commits");
    let SessionOperationOutcomeV1::MoleculeInsertedV1(outcome) = accepted.outcome() else {
        panic!("commit publishes molecule insertion facts");
    };
    assert_eq!(
        outcome.molecule_identifier().as_str(),
        "ferrum-molecule-v1-1"
    );
    assert_eq!(outcome.atom_identifiers()[0].as_str(), "ferrum-atom-v1-1");
    assert_eq!(outcome.atom_identifiers()[1].as_str(), "ferrum-atom-v1-2");
    assert_eq!(outcome.bond_identifiers()[0].as_str(), "ferrum-bond-v1-1");
    assert_eq!(accepted.observation().snapshot().revision(), 1);
    assert!(
        session
            .undo(1)
            .expect("one insertion undoes")
            .observation()
            .projection()
            .molecules()
            .is_empty()
    );
}

#[test]
fn generic_molecule_transition_refusals_leave_state_and_id_allocation_unchanged() {
    let mut owner = DocumentSession::create_empty_document_v1().expect("owner creates");
    let mut foreign = DocumentSession::create_empty_document_v1().expect("foreign creates");
    let mut prepared = owner
        .prepare_session_operation_transition_v1(request(0, carbonyl()))
        .expect("transition prepares");
    let baseline = owner.snapshot().expect("baseline snapshot");
    assert_eq!(
        foreign.commit_session_operation_transition_v1(&mut prepared),
        Err(AdmittedSessionTransitionRefusalV1::ForeignSession)
    );
    assert_eq!(
        owner.snapshot().expect("foreign refusal is inert"),
        baseline
    );
    owner
        .retire_session_operation_transition_v1(&mut prepared)
        .expect("transition retires");
    assert_eq!(
        owner.commit_session_operation_transition_v1(&mut prepared),
        Err(AdmittedSessionTransitionRefusalV1::Replayed)
    );
    let mut fresh = owner
        .prepare_session_operation_transition_v1(request(0, carbonyl()))
        .expect("equivalent request prepares after retirement");
    let accepted = owner
        .commit_session_operation_transition_v1(&mut fresh)
        .expect("fresh transition commits");
    let SessionOperationOutcomeV1::MoleculeInsertedV1(outcome) = accepted.outcome() else {
        panic!("committed outcome is molecule insertion");
    };
    assert_eq!(
        outcome.molecule_identifier().as_str(),
        "ferrum-molecule-v1-0"
    );
}

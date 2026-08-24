use std::collections::HashMap;

use super::{
    DetachedRegularRingInsertionV1, DocumentSession, Point3V1, RegularRingOrientationV1,
    RegularRingSizeV1, SessionOperation, SessionOperationOutcomeV1,
    SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
};

fn ring(size: u8, center: Point3V1) -> DetachedRegularRingInsertionV1 {
    DetachedRegularRingInsertionV1::new(
        RegularRingSizeV1::new(size).expect("closed-family test size"),
        center,
        4.0,
        RegularRingOrientationV1::FlatTop,
    )
    .expect("finite positive test ring")
}

fn is_carbon_single_cycle(molecule: &super::MoleculeProjectionV1) -> bool {
    if molecule.atoms().is_empty()
        || molecule
            .atoms()
            .iter()
            .any(|atom| atom.element() != Some("C"))
        || molecule
            .bonds()
            .iter()
            .any(|bond| bond.source_type() != Some("n1"))
    {
        return false;
    }
    let mut degrees = HashMap::new();
    for atom in molecule.atoms() {
        degrees.insert(atom.source_id().expect("atom ID"), 0_u8);
    }
    for bond in molecule.bonds() {
        let start = bond.start().source_id().expect("bond start");
        let end = bond.end().source_id().expect("bond end");
        if start == end {
            return false;
        }
        *degrees.get_mut(start).expect("known start") += 1;
        *degrees.get_mut(end).expect("known end") += 1;
    }
    molecule.bonds().len() == molecule.atoms().len() && degrees.values().all(|degree| *degree == 2)
}

#[test]
fn regular_ring_geometry_is_lowered_to_the_generic_molecule_operation() {
    let ring = ring(6, Point3V1::new(13.0, -7.0, 2.0).expect("finite centre"));
    let vertices = ring.vertices().expect("ring vertices");
    let mut session = DocumentSession::create_empty_document_v1().expect("session creates");
    let mut prepared = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            0,
            SessionOperation::V1(SessionOperationV1::InsertMoleculeV1(
                ring.molecule().expect("ring molecule"),
            )),
            TransitionAuthorizationV1::None,
        ))
        .expect("generic ring transition prepares");
    let accepted = session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("ring commits");
    let SessionOperationOutcomeV1::MoleculeInsertedV1(outcome) = accepted.outcome() else {
        panic!("ring uses molecule outcome");
    };
    let molecule = accepted
        .observation()
        .projection()
        .molecules()
        .iter()
        .find(|candidate| candidate.source_id() == Some(outcome.molecule_identifier().as_str()))
        .expect("committed ring");
    assert!(is_carbon_single_cycle(molecule));
    assert_eq!(
        molecule
            .atoms()
            .iter()
            .map(|atom| atom.position())
            .collect::<Vec<_>>(),
        vertices
    );
    assert!(
        session
            .undo(1)
            .expect("ring undo")
            .observation()
            .projection()
            .molecules()
            .is_empty()
    );
}

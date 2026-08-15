use std::collections::HashMap;

use super::{
    DetachedRegularRingInsertionV1, DocumentSession, Point3V1, RegularRingOrientationV1,
    RegularRingSizeV1,
};

fn request(size: u8, center: Point3V1) -> DetachedRegularRingInsertionV1 {
    DetachedRegularRingInsertionV1::new(
        RegularRingSizeV1::new(size).expect("test size belongs to the closed family"),
        center,
        4.0,
        RegularRingOrientationV1::FlatTop,
    )
    .expect("finite positive test request")
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
    let mut degree_by_id = HashMap::new();
    for atom in molecule.atoms() {
        let Some(identifier) = atom.source_id() else {
            return false;
        };
        degree_by_id.insert(identifier, 0_u8);
    }
    for bond in molecule.bonds() {
        let (Some(start), Some(end)) = (bond.start().source_id(), bond.end().source_id()) else {
            return false;
        };
        if start == end || !degree_by_id.contains_key(start) || !degree_by_id.contains_key(end) {
            return false;
        }
        *degree_by_id
            .get_mut(start)
            .expect("target presence was established") += 1;
        *degree_by_id
            .get_mut(end)
            .expect("target presence was established") += 1;
    }
    molecule.bonds().len() == molecule.atoms().len()
        && degree_by_id.values().all(|degree| *degree == 2)
}

#[test]
fn regular_ring_geometry_has_closed_admission_and_flat_top_equal_edges() {
    assert!(RegularRingSizeV1::new(3).is_ok());
    assert!(RegularRingSizeV1::new(8).is_ok());
    assert!(RegularRingSizeV1::new(2).is_err());
    assert!(RegularRingSizeV1::new(9).is_err());
    assert!(
        DetachedRegularRingInsertionV1::new(
            RegularRingSizeV1::new(6).expect("six is admitted"),
            Point3V1::new(0.0, 0.0, 0.0).expect("finite centre"),
            f64::INFINITY,
            RegularRingOrientationV1::FlatTop,
        )
        .is_err()
    );

    let vertices = request(6, Point3V1::new(13.0, -7.0, 2.0).expect("finite centre"))
        .vertices()
        .expect("admitted geometry remains finite");
    let edge_lengths = vertices
        .iter()
        .zip(vertices.iter().cycle().skip(1))
        .map(|(start, end)| (start.x() - end.x()).hypot(start.y() - end.y()))
        .collect::<Vec<_>>();

    assert!(
        edge_lengths
            .iter()
            .all(|length| (*length - 4.0).abs() < 1.0e-10)
    );
    assert!((vertices[0].y() - vertices[5].y()).abs() < 1.0e-10);
}

#[test]
fn regular_ring_commit_is_reversible_and_reopens_as_an_ordinary_cycle() {
    let center = Point3V1::new(13.0, -7.0, 2.0).expect("finite centre");
    let ring = request(6, center);
    let vertices = ring.vertices().expect("ring vertices");
    let mut session = DocumentSession::load("<cdml/>").expect("empty source loads");
    let mut pending = session
        .prepare_create_regular_ring_v1(0, ring)
        .expect("detached ring prepares");
    let accepted = session
        .commit_create_molecule(0, &mut pending)
        .expect("prepared ring commits");
    let snapshot = accepted.observation().snapshot().clone();
    let molecule = accepted
        .observation()
        .projection()
        .molecules()
        .iter()
        .find(|candidate| candidate.source_id() == Some(pending.molecule_identifier().as_str()))
        .expect("receipt identifies the committed molecule");

    assert!(is_carbon_single_cycle(molecule));
    assert_eq!(
        molecule
            .atoms()
            .iter()
            .map(|atom| atom.position())
            .collect::<Vec<_>>(),
        vertices,
    );

    let undone = session.undo(1).expect("one accepted ring is undoable");
    let redone = session.redo(2).expect("one accepted ring is redoable");
    let reopened = DocumentSession::load(snapshot.cdml()).expect("ordinary CDML reopens");
    assert!(undone.observation().projection().molecules().is_empty());
    assert!(
        redone
            .observation()
            .projection()
            .molecules()
            .iter()
            .any(is_carbon_single_cycle)
            && reopened
                .observe(0)
                .expect("reopened projection")
                .projection()
                .molecules()
                .iter()
                .any(is_carbon_single_cycle),
    );
}

#[test]
fn stale_regular_ring_receipt_preserves_current_document() {
    let center = Point3V1::new(0.0, 0.0, 0.0).expect("finite centre");
    let mut session = DocumentSession::load("<cdml/>").expect("empty source loads");
    let mut stale = session
        .prepare_create_regular_ring_v1(0, request(6, center))
        .expect("first candidate prepares");
    let mut accepted = session
        .prepare_create_regular_ring_v1(
            0,
            request(6, Point3V1::new(20.0, 0.0, 0.0).expect("finite centre")),
        )
        .expect("second candidate prepares");
    session
        .commit_create_molecule(0, &mut accepted)
        .expect("second candidate commits");
    let before_refusal = session.snapshot().expect("current snapshot");

    assert!(session.commit_create_molecule(1, &mut stale).is_err());
    assert_eq!(
        session.snapshot().expect("unchanged snapshot"),
        before_refusal
    );
}

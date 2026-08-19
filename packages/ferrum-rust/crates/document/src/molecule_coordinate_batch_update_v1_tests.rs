use super::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, MoleculeCoordinateBatchUpdateV1,
    MoleculeCoordinateBatchUpdateV1Error, MoleculeCoordinateUpdateV1, Point3V1, SessionOperation,
    SessionOperationV1,
};

const SOURCE: &str = concat!(
    "<cdml><molecule id=\"first\">",
    "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\" z=\"1\"/></atom>",
    "<atom id=\"b\" name=\"O\"><point x=\"2\" y=\"0\" z=\"3\"/></atom>",
    "<bond id=\"ab\" type=\"n1\" start=\"a\" end=\"b\"/></molecule>",
    "<molecule id=\"second\">",
    "<atom id=\"c\" name=\"N\"><point x=\"5\" y=\"0\" z=\"7\"/></atom>",
    "<atom id=\"d\" name=\"N\"><point x=\"5\" y=\"2\" z=\"9\"/></atom>",
    "<bond id=\"cd\" type=\"n1\" start=\"c\" end=\"d\"/></molecule></cdml>"
);

fn molecule_id(session: &DocumentSession, revision: u64, index: usize) -> DocumentObjectIdV1 {
    session
        .observe(revision)
        .expect("fixture observation must project")
        .projection()
        .molecules()[index]
        .id()
        .expect("fixture molecule has a durable ID")
        .clone()
}

fn point(x: f64, y: f64, z: f64) -> Point3V1 {
    Point3V1::new(x, y, z).expect("test coordinates are finite")
}

fn update(
    revision: u64,
    digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    positions: Vec<Point3V1>,
) -> MoleculeCoordinateUpdateV1 {
    MoleculeCoordinateUpdateV1::new(revision, digest, molecule_id, positions)
        .expect("test update has positions")
}

fn batch(session: &DocumentSession) -> MoleculeCoordinateBatchUpdateV1 {
    let snapshot = session.snapshot().expect("snapshot must work");
    let first = molecule_id(session, snapshot.revision(), 0);
    let second = molecule_id(session, snapshot.revision(), 1);
    MoleculeCoordinateBatchUpdateV1::new(
        snapshot.revision(),
        *snapshot.digest(),
        vec![
            update(
                snapshot.revision(),
                *snapshot.digest(),
                first,
                vec![point(0.0, 2.0, 1.0), point(2.0, 2.0, 3.0)],
            ),
            update(
                snapshot.revision(),
                *snapshot.digest(),
                second,
                vec![point(3.0, 0.0, 7.0), point(3.0, 2.0, 9.0)],
            ),
        ],
    )
    .expect("targets are unique and share a source")
}

#[test]
fn point3_coordinate_batch_commits_all_targets_in_one_history_transition() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture must load");
    let prepared = batch(&session);

    let result = session
        .submit(
            0,
            SessionOperation::V1(SessionOperationV1::SetMoleculeAtomPositionsBatch {
                update: prepared,
            }),
        )
        .expect("batch must commit");

    assert_eq!(result.observation().snapshot().revision(), 1);
    let molecules = result.observation().projection().molecules();
    assert_eq!(molecules[0].atoms()[0].position(), point(0.0, 2.0, 1.0));
    assert_eq!(molecules[1].atoms()[0].position(), point(3.0, 0.0, 7.0));

    let first = molecules[0].atoms();
    let first_centroid = (
        (first[0].position().x() + first[1].position().x()) / 2.0,
        (first[0].position().y() + first[1].position().y()) / 2.0,
    );
    assert_eq!(first_centroid, (1.0, 2.0));
    let first_bond_length = ((first[1].position().x() - first[0].position().x()).powi(2)
        + (first[1].position().y() - first[0].position().y()).powi(2))
    .sqrt();
    assert_eq!(first_bond_length, 2.0);
    assert_eq!(
        (first[0].position().z(), first[1].position().z()),
        (1.0, 3.0)
    );
}

#[test]
fn point3_coordinate_batch_rejects_stale_and_invalid_targets_without_mutation() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture must load");
    let stale = batch(&session);
    session
        .submit(
            0,
            SessionOperation::V1(SessionOperationV1::SetAtomPosition {
                atom_id: "a".to_owned(),
                position: point(1.0, 0.0, 1.0),
            }),
        )
        .expect("intervening mutation must commit");
    let after_intervening = session.snapshot().expect("snapshot must work");
    assert!(matches!(
        session.submit(
            1,
            SessionOperation::V1(SessionOperationV1::SetMoleculeAtomPositionsBatch {
                update: stale
            })
        ),
        Err(DocumentSessionError::Operation(
            super::SessionOperationError::MoleculeCoordinateRevisionMismatch { .. }
        ))
    ));
    assert_eq!(
        session.snapshot().expect("snapshot must work"),
        after_intervening
    );

    let snapshot = session.snapshot().expect("snapshot must work");
    let first = molecule_id(&session, snapshot.revision(), 0);
    let second = molecule_id(&session, snapshot.revision(), 1);
    let invalid = MoleculeCoordinateBatchUpdateV1::new(
        snapshot.revision(),
        *snapshot.digest(),
        vec![
            update(
                snapshot.revision(),
                *snapshot.digest(),
                first,
                vec![point(0.0, 0.0, 1.0), point(2.0, 0.0, 3.0)],
            ),
            update(
                snapshot.revision(),
                *snapshot.digest(),
                second,
                vec![point(5.0, 0.0, 7.0)],
            ),
        ],
    )
    .expect("unique targets form a valid request shape");
    assert!(matches!(
        session.submit(
            snapshot.revision(),
            SessionOperation::V1(SessionOperationV1::SetMoleculeAtomPositionsBatch {
                update: invalid
            })
        ),
        Err(DocumentSessionError::Operation(
            super::SessionOperationError::Candidate(
                super::TypedDocumentError::MoleculePositionCountMismatch { .. }
            )
        ))
    ));
    assert_eq!(
        session.snapshot().expect("snapshot must work"),
        after_intervening
    );
}

#[test]
fn point3_coordinate_batch_rejects_duplicate_and_mismatched_source_entries() {
    let session = DocumentSession::load(SOURCE).expect("fixture must load");
    let snapshot = session.snapshot().expect("snapshot must work");
    let digest = *snapshot.digest();
    let first = update(
        snapshot.revision(),
        digest,
        molecule_id(&session, snapshot.revision(), 0),
        vec![point(0.0, 0.0, 0.0)],
    );
    assert_eq!(
        MoleculeCoordinateBatchUpdateV1::new(
            snapshot.revision(),
            digest,
            vec![first.clone(), first]
        ),
        Err(MoleculeCoordinateBatchUpdateV1Error::DuplicateMolecule)
    );
    assert_eq!(
        MoleculeCoordinateBatchUpdateV1::new(
            snapshot.revision() + 1,
            digest,
            vec![update(
                snapshot.revision(),
                digest,
                molecule_id(&session, snapshot.revision(), 0),
                vec![point(0.0, 0.0, 0.0)],
            )],
        ),
        Err(MoleculeCoordinateBatchUpdateV1Error::SourceMismatch)
    );
}

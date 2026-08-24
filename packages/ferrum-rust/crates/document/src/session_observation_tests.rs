use crate::{DocumentSession, DocumentSnapshot, ProjectionError, SessionDocumentObservationV1};

#[test]
fn malformed_snapshot_cdml_is_a_closed_projection_error() {
    let snapshot = DocumentSnapshot::new(7, "<cdml".to_owned(), [0; 32], false);

    assert!(matches!(
        crate::projection_adapter::document_projection_from_snapshot_v1(&snapshot),
        Err(ProjectionError::InvalidValue {
            context,
            field: "cdml",
            ..
        }) if context == "document snapshot"
    ));
}

#[test]
fn observation_pairs_projection_with_its_own_snapshot() {
    let session = DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>")
        .expect("minimal CDML source must load");
    let snapshot = session.snapshot().expect("loaded session must snapshot");

    let observation = SessionDocumentObservationV1::from_snapshot(snapshot.clone())
        .expect("authoritative snapshot must project");

    assert_eq!(observation.snapshot(), &snapshot);
    assert_eq!(observation.projection().revision(), snapshot.revision());
    assert_eq!(observation.projection().digest(), snapshot.digest());
    assert_eq!(observation.projection().is_dirty(), snapshot.is_dirty());
}

use ferrum_document::{
    DirectBondAdmissionRefusalV1, DirectBondCommitErrorV1, DirectBondEndpointIntent,
    DirectBondPoint2V1, DirectBondSnapPolicyV1, DocumentBondOrderV1, DocumentBondPresentationV1,
    DocumentFenceV1, DocumentObjectIdV1, DocumentSession,
};

const EMPTY: &str = "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"/>";

fn fence(session: &DocumentSession) -> DocumentFenceV1 {
    let snapshot = session.snapshot().expect("session snapshot");
    DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
}

fn point(x: f64, y: f64) -> DirectBondEndpointIntent {
    DirectBondEndpointIntent::NewAtomAt {
        raw_point: DirectBondPoint2V1::new(x, y).expect("finite point"),
    }
}

fn neutral_candidate(session: &DocumentSession) -> ferrum_document::DirectBondMutationCandidate {
    session
        .materialize_direct_bond_mutation(
            fence(session),
            point(0.0, 0.0),
            point(40.0, 0.0),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        )
        .expect("candidate materializes")
}

fn snapshot_state(session: &DocumentSession) -> (u64, [u8; 32], String) {
    let snapshot = session.snapshot().expect("session snapshot");
    (
        snapshot.revision(),
        *snapshot.digest(),
        snapshot.cdml().to_owned(),
    )
}

#[test]
fn external_rust_domain_consumer_commits_neutral_direct_bond_mutation() {
    let mut session = DocumentSession::load(EMPTY).expect("empty session");
    let before = snapshot_state(&session);
    let candidate = neutral_candidate(&session);

    session
        .commit_direct_bond_mutation(&candidate)
        .expect("candidate commits");

    assert_eq!(snapshot_state(&session).0, before.0 + 1);
}

#[test]
fn external_consumer_refuses_identical_foreign_session_and_retains_owner_receipt() {
    let mut session_a = DocumentSession::load(EMPTY).expect("first session loads");
    let candidate = neutral_candidate(&session_a);
    let before_a = snapshot_state(&session_a);
    let mut session_b = DocumentSession::load(EMPTY).expect("second session loads");
    let before_b = snapshot_state(&session_b);

    assert_eq!(
        session_b.commit_direct_bond_mutation(&candidate),
        Err(DirectBondCommitErrorV1::ForeignSession)
    );
    assert_eq!(snapshot_state(&session_a), before_a);
    assert_eq!(snapshot_state(&session_b), before_b);
    session_a
        .commit_direct_bond_mutation(&candidate)
        .expect("owner retains its candidate after foreign refusal");
    assert_eq!(snapshot_state(&session_a).0, before_a.0 + 1);
}

#[test]
fn external_consumer_aliases_share_one_redeemable_candidate() {
    let mut session = DocumentSession::load(EMPTY).expect("empty session");
    let candidate = neutral_candidate(&session);
    let alias = candidate.clone();
    session
        .commit_direct_bond_mutation(&candidate)
        .expect("first commit succeeds");
    let after_first_commit = snapshot_state(&session);

    assert_eq!(
        session.commit_direct_bond_mutation(&alias),
        Err(DirectBondCommitErrorV1::ReplayedReceipt)
    );
    assert_eq!(snapshot_state(&session), after_first_commit);
}

#[test]
fn external_consumer_materialization_refusals_leave_session_unchanged() {
    let session = DocumentSession::load(EMPTY).expect("empty session");
    let before = snapshot_state(&session);

    assert_eq!(
        session.materialize_direct_bond_mutation(
            fence(&session),
            point(0.0, 0.0),
            point(0.0, 0.0),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        ),
        Err(DirectBondAdmissionRefusalV1::CollapsedEndpoint)
    );
    assert_eq!(
        session.materialize_direct_bond_mutation(
            fence(&session),
            DirectBondEndpointIntent::ExistingAtom {
                atom: DocumentObjectIdV1::parse(
                    "ferrum-document-object-v1/63646d6c2f61746f6d/source/6d697373696e67",
                )
                .expect("valid opaque atom selector"),
            },
            point(40.0, 0.0),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        ),
        Err(DirectBondAdmissionRefusalV1::UnknownStartAtom)
    );
    assert_eq!(snapshot_state(&session), before);
}

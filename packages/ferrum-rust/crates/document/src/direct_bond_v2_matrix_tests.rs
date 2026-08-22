//! Matrix coverage for the public V2 direct-bond ownership contract.

use super::*;

const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"C\"><point x=\"40\" y=\"0\"/></atom></molecule></cdml>";
const BLANK_SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"/>";
const DUPLICATE_SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"C\"><point x=\"40\" y=\"0\"/></atom><bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule></cdml>";
const CROSS_MOLECULE_SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"first\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"second\"><atom id=\"b\" name=\"C\"><point x=\"40\" y=\"0\"/></atom></molecule></cdml>";
const SATURATED_CARBON_SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"C\"><point x=\"20\" y=\"0\"/></atom><atom id=\"c\" name=\"C\"><point x=\"-20\" y=\"0\"/></atom><atom id=\"d\" name=\"C\"><point x=\"0\" y=\"20\"/></atom><atom id=\"e\" name=\"C\"><point x=\"0\" y=\"-20\"/></atom><bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/><bond id=\"ac\" start=\"a\" end=\"c\" type=\"n1\"/><bond id=\"ad\" start=\"a\" end=\"d\" type=\"n1\"/><bond id=\"ae\" start=\"a\" end=\"e\" type=\"n1\"/></molecule></cdml>";

fn fence(session: &DocumentSession) -> DocumentFenceV1 {
    let snapshot = session.snapshot().expect("snapshot");
    DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
}

fn existing(session: &DocumentSession, source_id: &str) -> DirectBondEndpointIntentV2 {
    let observation = session.observe(0).expect("current observation");
    let atom = observation
        .projection()
        .molecules()
        .iter()
        .flat_map(|molecule| molecule.atoms())
        .find(|atom| atom.source_id() == Some(source_id))
        .expect("projected atom has expected source ID");
    DirectBondEndpointIntentV2::ExistingAtom {
        atom: atom.id().expect("projected atom has canonical ID").clone(),
    }
}

fn point(x: f64, y: f64) -> DirectBondEndpointIntentV2 {
    DirectBondEndpointIntentV2::NewAtomAt {
        raw_point: DirectBondPoint2V1::new(x, y).expect("finite point"),
    }
}

#[derive(Debug, PartialEq)]
enum V2Refusal {
    Begin(DirectBondGestureErrorV1),
    Admission(DirectBondAdmissionRefusalV1),
}

fn begin(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: DirectBondEndpointIntentV2,
    element: &str,
    snap: DirectBondSnapPolicyV1,
) -> Result<DirectBondGestureV2, DirectBondGestureErrorV1> {
    session.begin_direct_bond_gesture_v2(
        fence,
        start,
        DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
        element.to_owned(),
        snap,
    )
}

fn admission_refusal(
    session: &mut DocumentSession,
    gesture: &DirectBondGestureV2,
    end: DirectBondEndpointIntentV2,
) -> V2Refusal {
    let snapshot = session.snapshot().expect("snapshot before refusal");
    let token_facts = session.provisional_token_facts_for_test();
    let error = session
        .admit_direct_bond_candidate_v2(gesture, end)
        .expect_err("matrix candidate refuses");
    assert_eq!(
        session.snapshot().expect("snapshot after refusal"),
        snapshot
    );
    assert_eq!(session.provisional_token_facts_for_test(), token_facts);
    V2Refusal::Admission(error)
}

fn stale_digest_refusal() -> V2Refusal {
    let session = DocumentSession::load(BLANK_SOURCE).expect("blank session loads");
    let snapshot = session.snapshot().expect("snapshot before refusal");
    let token_facts = session.provisional_token_facts_for_test();
    let mut stale_digest = *snapshot.digest();
    stale_digest[0] ^= u8::MAX;
    let error = begin(
        &session,
        DocumentFenceV1::new(snapshot.revision(), stale_digest),
        point(0.0, 0.0),
        "C",
        DirectBondSnapPolicyV1::free(),
    )
    .expect_err("stale digest refuses before a gesture exists");
    assert_eq!(
        session.snapshot().expect("snapshot after refusal"),
        snapshot
    );
    assert_eq!(session.provisional_token_facts_for_test(), token_facts);
    V2Refusal::Begin(error)
}

fn stale_revision_refusal() -> V2Refusal {
    let mut session = DocumentSession::load(SOURCE).expect("session loads");
    let stale_start = existing(&session, "a");
    let stale_end = existing(&session, "b");
    let stale = begin(
        &session,
        fence(&session),
        stale_start,
        "C",
        DirectBondSnapPolicyV1::free(),
    )
    .expect("gesture begins");
    let update = begin(
        &session,
        fence(&session),
        existing(&session, "a"),
        "C",
        DirectBondSnapPolicyV1::free(),
    )
    .expect("current gesture begins");
    let update = session
        .admit_direct_bond_candidate_v2(&update, point(80.0, 0.0))
        .expect("current candidate admits");
    session
        .commit_direct_bond_admission_v2(&update)
        .expect("current candidate commits");
    admission_refusal(&mut session, &stale, stale_end)
}

fn unsupported_presentation_refusal() -> V2Refusal {
    let session = DocumentSession::load(BLANK_SOURCE).expect("blank session loads");
    let snapshot = session.snapshot().expect("snapshot before refusal");
    let token_facts = session.provisional_token_facts_for_test();
    let error = session
        .begin_direct_bond_gesture_v2(
            fence(&session),
            point(0.0, 0.0),
            DocumentBondPresentationV1::SolidWedge,
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        )
        .expect_err("non-normal presentation refuses before a gesture exists");
    assert_eq!(
        session.snapshot().expect("snapshot after refusal"),
        snapshot
    );
    assert_eq!(session.provisional_token_facts_for_test(), token_facts);
    V2Refusal::Begin(error)
}

fn foreign_session_refusal() -> V2Refusal {
    let owner = DocumentSession::load(SOURCE).expect("owner session loads");
    let gesture = begin(
        &owner,
        fence(&owner),
        existing(&owner, "a"),
        "C",
        DirectBondSnapPolicyV1::free(),
    )
    .expect("owner gesture begins");
    let mut target = DocumentSession::load(SOURCE).expect("target session loads");
    let end = existing(&target, "b");
    admission_refusal(&mut target, &gesture, end)
}

fn collapsed_endpoint_refusal() -> V2Refusal {
    let mut session = DocumentSession::load(BLANK_SOURCE).expect("blank session loads");
    let gesture = begin(
        &session,
        fence(&session),
        point(0.0, 0.0),
        "C",
        DirectBondSnapPolicyV1::free(),
    )
    .expect("gesture begins");
    admission_refusal(&mut session, &gesture, point(0.0, 0.0))
}

fn invalid_endpoint_input_refusal() -> V2Refusal {
    let mut session = DocumentSession::load(BLANK_SOURCE).expect("blank session loads");
    let gesture = begin(
        &session,
        fence(&session),
        point(0.0, 0.0),
        "C",
        DirectBondSnapPolicyV1::new(false, None, Some(20.0)).expect("valid snap policy"),
    )
    .expect("gesture begins");
    admission_refusal(&mut session, &gesture, point(0.0, 0.0))
}

fn duplicate_bond_refusal() -> V2Refusal {
    let mut session = DocumentSession::load(DUPLICATE_SOURCE).expect("session loads");
    let gesture = begin(
        &session,
        fence(&session),
        existing(&session, "a"),
        "C",
        DirectBondSnapPolicyV1::free(),
    )
    .expect("gesture begins");
    let end = existing(&session, "b");
    admission_refusal(&mut session, &gesture, end)
}

fn cross_molecule_refusal() -> V2Refusal {
    let mut session = DocumentSession::load(CROSS_MOLECULE_SOURCE).expect("session loads");
    let gesture = begin(
        &session,
        fence(&session),
        existing(&session, "a"),
        "C",
        DirectBondSnapPolicyV1::free(),
    )
    .expect("gesture begins");
    let end = existing(&session, "b");
    admission_refusal(&mut session, &gesture, end)
}

fn unknown_endpoint_refusal() -> V2Refusal {
    let mut session = DocumentSession::load(SOURCE).expect("session loads");
    let gesture = begin(
        &session,
        fence(&session),
        existing(&session, "a"),
        "C",
        DirectBondSnapPolicyV1::free(),
    )
    .expect("gesture begins");
    let missing = DirectBondEndpointIntentV2::ExistingAtom {
        atom: DocumentObjectIdV1::from_class_source("cdml/atom", "missing"),
    };
    admission_refusal(&mut session, &gesture, missing)
}

fn unknown_start_refusal() -> V2Refusal {
    let mut session = DocumentSession::load(SOURCE).expect("session loads");
    let missing = DirectBondEndpointIntentV2::ExistingAtom {
        atom: DocumentObjectIdV1::from_class_source("cdml/atom", "missing"),
    };
    let gesture = begin(
        &session,
        fence(&session),
        missing,
        "C",
        DirectBondSnapPolicyV1::free(),
    )
    .expect("gesture begins without resolving an endpoint");
    admission_refusal(&mut session, &gesture, point(40.0, 0.0))
}

fn unsupported_chemistry_refusal() -> V2Refusal {
    let mut session = DocumentSession::load(BLANK_SOURCE).expect("blank session loads");
    let gesture = begin(
        &session,
        fence(&session),
        point(0.0, 0.0),
        "Xx",
        DirectBondSnapPolicyV1::free(),
    )
    .expect("gesture begins");
    admission_refusal(&mut session, &gesture, point(40.0, 0.0))
}

fn exceeds_chemistry_capacity_refusal() -> V2Refusal {
    let mut session = DocumentSession::load(SATURATED_CARBON_SOURCE).expect("session loads");
    let gesture = begin(
        &session,
        fence(&session),
        existing(&session, "a"),
        "C",
        DirectBondSnapPolicyV1::free(),
    )
    .expect("gesture begins");
    admission_refusal(&mut session, &gesture, point(40.0, 40.0))
}

#[test]
fn v2_normal_orders_persist_and_history_round_trip_for_every_endpoint_form() {
    for (order, expected_type) in [
        (DocumentBondOrderV1::Single, "n1"),
        (DocumentBondOrderV1::Double, "n2"),
        (DocumentBondOrderV1::Triple, "n3"),
    ] {
        for form in [
            "existing_existing",
            "existing_new",
            "new_existing",
            "new_new",
        ] {
            let source = if form == "new_new" {
                BLANK_SOURCE
            } else {
                SOURCE
            };
            let mut session = DocumentSession::load(source).expect("session loads");
            let (start, end, created_new_atom, created_new_molecule) = match form {
                "existing_existing" => (
                    existing(&session, "a"),
                    existing(&session, "b"),
                    false,
                    false,
                ),
                "existing_new" => (existing(&session, "a"), point(80.0, 0.0), true, false),
                "new_existing" => (point(80.0, 0.0), existing(&session, "b"), true, false),
                "new_new" => (point(0.0, 0.0), point(40.0, 0.0), true, true),
                _ => unreachable!("fixed endpoint form matrix"),
            };
            let before = session.snapshot().expect("snapshot before admission");
            let gesture = session
                .begin_direct_bond_gesture_v2(
                    fence(&session),
                    start,
                    DocumentBondPresentationV1::Normal(order),
                    "C".to_owned(),
                    DirectBondSnapPolicyV1::free(),
                )
                .expect("gesture begins");
            let admission = session
                .admit_direct_bond_candidate_v2(&gesture, end)
                .unwrap_or_else(|error| {
                    panic!("{form} candidate admits without mutation: {error:?}")
                });
            assert_eq!(session.snapshot().expect("admission stays pure"), before);

            let receipt = session
                .commit_direct_bond_admission_v2(&admission)
                .expect("admission commits atomically");
            assert_eq!(receipt.created_new_atom(), created_new_atom);
            assert_eq!(receipt.created_new_molecule(), created_new_molecule);
            assert_eq!(receipt.second_created_atom().is_some(), form == "new_new");
            if matches!(form, "existing_existing" | "new_existing") {
                assert_eq!(receipt.end_atom().as_str(), "b");
            }

            let committed = session.snapshot().expect("committed snapshot");
            let reopened = DocumentSession::load(committed.cdml()).expect("CDML reopens");
            let observation = reopened.observe(0).expect("reopened observation");
            let bonds: Vec<_> = observation
                .projection()
                .molecules()
                .iter()
                .flat_map(|molecule| molecule.bonds())
                .collect();
            assert_eq!(bonds.len(), 1);
            assert_eq!(bonds[0].source_type(), Some(expected_type));

            let undone = session
                .undo(committed.revision())
                .expect("one transition undoes");
            assert_eq!(undone.observation().snapshot().cdml(), before.cdml());
            let redone = session
                .redo(undone.observation().snapshot().revision())
                .expect("one transition redoes");
            assert_eq!(redone.observation().snapshot().cdml(), committed.cdml());
        }
    }
}

#[test]
fn v2_representable_refusals_leave_document_and_identity_state_unchanged() {
    for (name, refuse, expected) in [
        (
            "stale_digest",
            stale_digest_refusal as fn() -> V2Refusal,
            V2Refusal::Begin(DirectBondGestureErrorV1::StaleDigest),
        ),
        (
            "stale_revision",
            stale_revision_refusal,
            V2Refusal::Admission(DirectBondAdmissionRefusalV1::StaleRevision),
        ),
        (
            "unsupported_presentation",
            unsupported_presentation_refusal,
            V2Refusal::Begin(DirectBondGestureErrorV1::UnsupportedPresentation),
        ),
        (
            "foreign_session",
            foreign_session_refusal,
            V2Refusal::Admission(DirectBondAdmissionRefusalV1::ForeignSession),
        ),
        (
            "collapsed_endpoint",
            collapsed_endpoint_refusal,
            V2Refusal::Admission(DirectBondAdmissionRefusalV1::CollapsedEndpoint),
        ),
        (
            "invalid_endpoint_input",
            invalid_endpoint_input_refusal,
            V2Refusal::Admission(DirectBondAdmissionRefusalV1::InvalidEndpointInput),
        ),
        (
            "duplicate_bond",
            duplicate_bond_refusal,
            V2Refusal::Admission(DirectBondAdmissionRefusalV1::DuplicateBond),
        ),
        (
            "cross_molecule",
            cross_molecule_refusal,
            V2Refusal::Admission(DirectBondAdmissionRefusalV1::CrossMolecule),
        ),
        (
            "unknown_endpoint",
            unknown_endpoint_refusal,
            V2Refusal::Admission(DirectBondAdmissionRefusalV1::UnknownEndAtom),
        ),
        (
            "unknown_start",
            unknown_start_refusal,
            V2Refusal::Admission(DirectBondAdmissionRefusalV1::UnknownStartAtom),
        ),
        (
            "unsupported_chemistry",
            unsupported_chemistry_refusal,
            V2Refusal::Admission(DirectBondAdmissionRefusalV1::UnsupportedChemistryAdmission),
        ),
        (
            "exceeds_chemistry_capacity",
            exceeds_chemistry_capacity_refusal,
            V2Refusal::Admission(DirectBondAdmissionRefusalV1::ExceedsChemistryCapacity),
        ),
    ] {
        assert_eq!(refuse(), expected, "{name} refusal contract");
    }
}

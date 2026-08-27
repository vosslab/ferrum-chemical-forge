use super::super::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, PersistentId,
    PreparedLinearFormConvertResultV1,
};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
    "</atom><atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom>",
    "<bond id=\"ab\" type=\"n1\" start=\"a\" end=\"b\"/></molecule></cdml>",
);

const TWO_MOLECULE_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
    "</atom><atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom>",
    "<bond id=\"ab\" type=\"n1\" start=\"a\" end=\"b\"/></molecule>",
    "<molecule id=\"n\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"20\"/>",
    "</atom><atom id=\"d\" name=\"O\"><point x=\"20\" y=\"20\"/></atom>",
    "<bond id=\"cd\" type=\"n1\" start=\"c\" end=\"d\"/></molecule></cdml>",
);

fn request(session: &DocumentSession, revision: u64) -> (DocumentObjectIdV1, Vec<PersistentId>) {
    let molecule = session
        .observe(revision)
        .expect("fixture must project")
        .projection()
        .molecules()[0]
        .document_object_id()
        .clone();
    (
        molecule,
        vec![
            PersistentId::new("a").unwrap(),
            PersistentId::new("b").unwrap(),
        ],
    )
}

fn pending(
    session: &mut DocumentSession,
    revision: u64,
) -> super::super::PendingLinearFormConvertV1 {
    let (molecule, atoms) = request(session, revision);
    match session
        .prepare_convert_linear_form_v1(revision, &molecule, &atoms)
        .expect("conversion must prepare")
    {
        PreparedLinearFormConvertResultV1::Pending(pending) => *pending,
        PreparedLinearFormConvertResultV1::NoChange(_) => panic!("fixture must change"),
    }
}

#[test]
fn conversion_commits_once_and_repeat_is_history_free() {
    let mut session = DocumentSession::load(SOURCE).expect("source loads");
    let mut receipt = pending(&mut session, 0);
    let fragment = receipt.fragment_id().clone();
    let changed = session
        .commit_convert_linear_form_v1(0, &mut receipt)
        .expect("prepared conversion commits");
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert!(
        changed
            .observation()
            .snapshot()
            .cdml()
            .contains(fragment.as_str())
    );
    let (molecule, atoms) = request(&session, 1);
    let no_change = session
        .prepare_convert_linear_form_v1(1, &molecule, &atoms)
        .expect("repeat classifies");
    let PreparedLinearFormConvertResultV1::NoChange(no_change) = no_change else {
        panic!("canonical repeat must not issue a receipt");
    };
    assert_eq!(no_change.observation().snapshot().revision(), 1);
    let undone = session.undo(1).expect("conversion is undoable");
    assert!(
        !undone
            .observation()
            .snapshot()
            .cdml()
            .contains(fragment.as_str())
    );
    let redone = session.redo(2).expect("conversion is redoable");
    let reopened = DocumentSession::load(redone.observation().snapshot().cdml())
        .expect("accepted conversion reopens");
    assert!(
        reopened
            .observe(0)
            .expect("reopened conversion projects")
            .snapshot()
            .cdml()
            .contains(fragment.as_str())
    );
}

#[test]
fn stale_foreign_and_consumed_receipts_do_not_accept_again() {
    let mut session = DocumentSession::load(SOURCE).expect("source loads");
    let mut receipt = pending(&mut session, 0);
    let expected_fragment = receipt.fragment_id().clone();
    let before = session.snapshot().expect("snapshot works");
    assert!(matches!(
        session.commit_convert_linear_form_v1(1, &mut receipt),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));
    assert_eq!(session.snapshot().expect("snapshot works"), before);
    assert_eq!(pending(&mut session, 0).fragment_id(), &expected_fragment);
    session
        .commit_convert_linear_form_v1(0, &mut receipt)
        .expect("receipt remains retryable after stale caller revision");
    assert!(matches!(
        session.commit_convert_linear_form_v1(1, &mut receipt),
        Err(DocumentSessionError::PreparedOperationConsumed)
    ));
    let committed = session.snapshot().expect("snapshot works");
    assert_eq!(session.snapshot().expect("snapshot works"), committed);
    let mut target = DocumentSession::load(SOURCE).expect("target source loads");
    let mut other = DocumentSession::load(SOURCE).expect("other source loads");
    let mut foreign = pending(&mut other, 0);
    let foreign_fragment = foreign.fragment_id().clone();
    let target_before = target.snapshot().expect("snapshot works");
    assert!(matches!(
        target.commit_convert_linear_form_v1(0, &mut foreign),
        Err(DocumentSessionError::PreparedOperationForeignSession)
    ));
    assert_eq!(target.snapshot().expect("snapshot works"), target_before);
    assert_eq!(pending(&mut target, 0).fragment_id(), &foreign_fragment);
}

#[test]
fn dropped_receipt_does_not_install_its_tentative_fragment_sequence() {
    let mut session = DocumentSession::load(SOURCE).expect("source loads");
    let first = pending(&mut session, 0).fragment_id().clone();
    let before = session.snapshot().expect("snapshot works");
    let mut second = pending(&mut session, 0);
    assert_eq!(second.fragment_id(), &first);
    assert_eq!(session.snapshot().expect("snapshot works"), before);
    session
        .commit_convert_linear_form_v1(0, &mut second)
        .expect("later receipt commits with same tentative ID");
}

#[test]
fn failed_preparation_leaves_the_fragment_sequence_tentative() {
    let mut session = DocumentSession::load(SOURCE).expect("source loads");
    let expected_fragment = pending(&mut session, 0).fragment_id().clone();
    let (molecule, _) = request(&session, 0);
    let before = session.snapshot().expect("snapshot works");
    assert!(matches!(
        session.prepare_convert_linear_form_v1(0, &molecule, &[]),
        Err(DocumentSessionError::Operation(
            super::super::SessionOperationError::EmptyLinearFormSelection
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot works"), before);
    assert_eq!(pending(&mut session, 0).fragment_id(), &expected_fragment);
}

#[test]
fn repair_keeps_its_existing_id_without_installing_a_fragment_sequence() {
    let mut first = DocumentSession::load(TWO_MOLECULE_SOURCE).expect("source loads");
    let mut initial = pending(&mut first, 0);
    let generated = initial.fragment_id().clone();
    let converted = first
        .commit_convert_linear_form_v1(0, &mut initial)
        .expect("first molecule converts");
    let damaged = converted
        .observation()
        .snapshot()
        .cdml()
        .replace("x=\"10\"", "x=\"11\"");
    let mut repaired = DocumentSession::load(&damaged).expect("damaged source loads");
    let (molecule, atoms) = request(&repaired, 0);
    let PreparedLinearFormConvertResultV1::Pending(mut repair) = repaired
        .prepare_convert_linear_form_v1(0, &molecule, &atoms)
        .expect("existing owner prepares a repair")
    else {
        panic!("damaged generated geometry requires repair");
    };
    assert_eq!(repair.fragment_id(), &generated);
    repaired
        .commit_convert_linear_form_v1(0, &mut repair)
        .expect("repair commits");
    let observation = repaired.observe(1).expect("repair projects");
    let molecule = observation.projection().molecules()[1]
        .document_object_id()
        .clone();
    let atoms = vec![
        PersistentId::new("c").unwrap(),
        PersistentId::new("d").unwrap(),
    ];
    let PreparedLinearFormConvertResultV1::Pending(next) = repaired
        .prepare_convert_linear_form_v1(1, &molecule, &atoms)
        .expect("second molecule prepares")
    else {
        panic!("second molecule requires a new record");
    };
    assert_eq!(next.fragment_id().as_str(), "ferrum-fragment-v1-1");
}

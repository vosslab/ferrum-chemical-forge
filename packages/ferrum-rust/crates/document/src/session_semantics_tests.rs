use super::{
    DocumentSession, DocumentSessionError, PersistentId, PublicationDurability, SaveOutcome,
    SessionOperation, SessionOperationV1,
};

const SOURCE: &str = concat!(
    "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
    "<point x=\"1\" y=\"2\"/></atom></molecule></cdml>"
);

const PREFIXED_SOURCE: &str = concat!(
    "<cdml:cdml xmlns:cdml=\"http://www.freesoftware.fsf.org/bkchem/cdml\">",
    "<cdml:molecule id=\"m\"/></cdml:cdml>"
);

fn set_atom(element: &str) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetAtomElement {
        atom_id: "a".to_owned(),
        element: element.to_owned(),
    })
}

#[test]
fn typed_operation_is_revisioned_and_noop_is_history_free() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let baseline = session.snapshot().expect("snapshot must work");
    assert_eq!(baseline.revision(), 0);
    assert!(!baseline.is_dirty());

    let no_change = session
        .submit(0, set_atom("C"))
        .expect("no-op must succeed");
    assert_eq!(no_change, baseline);

    let changed = session.submit(0, set_atom("N")).expect("edit must succeed");
    assert_eq!(changed.revision(), 1);
    assert!(changed.is_dirty());
    assert!(changed.cdml().contains("name=\"N\""));

    assert!(matches!(
        session.submit(0, set_atom("O")),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), changed);
}

#[test]
fn backend_history_navigation_publishes_monotonic_revisions() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session.submit(0, set_atom("N")).expect("edit must succeed");
    let undone = session.undo(changed.revision()).expect("undo must succeed");
    assert_eq!(undone.revision(), 2);
    assert!(undone.cdml().contains("name=\"C\""));

    let redone = session.redo(undone.revision()).expect("redo must succeed");
    assert_eq!(redone.revision(), 3);
    assert!(redone.cdml().contains("name=\"N\""));
    assert!(redone.is_dirty());
}

#[test]
fn rejected_operation_cannot_change_the_authoritative_snapshot() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    assert!(matches!(
        session.submit(0, set_atom("2")),
        Err(DocumentSessionError::Operation(_))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn confirmed_save_advances_the_baseline_without_losing_published_identity() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session.submit(0, set_atom("N")).expect("edit must succeed");
    let saved = session
        .record_save_outcome_for_test(PublicationDurability::Confirmed)
        .expect("confirmed outcome must advance baseline");
    assert_eq!(saved.outcome(), SaveOutcome::Confirmed);
    assert_eq!(saved.published_snapshot(), &changed);
    assert!(!saved.snapshot().is_dirty());
    assert_eq!(saved.snapshot().revision(), changed.revision());

    let undone = session
        .undo(saved.snapshot().revision())
        .expect("undo must succeed");
    assert!(undone.is_dirty());
    let redone = session.redo(undone.revision()).expect("redo must succeed");
    assert!(!redone.is_dirty());
}

#[test]
fn unconfirmed_replacement_keeps_the_session_dirty_for_verification_or_recovery() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session.submit(0, set_atom("N")).expect("edit must succeed");
    let outcome = session
        .record_save_outcome_for_test(PublicationDurability::DirectoryEntryUnconfirmed)
        .expect("injected outcome must report");

    assert_eq!(outcome.outcome(), SaveOutcome::DirectoryEntryUnconfirmed);
    assert_eq!(outcome.published_snapshot(), &changed);
    assert_eq!(outcome.snapshot(), &changed);
    assert!(outcome.snapshot().is_dirty());
    assert_eq!(session.snapshot().expect("snapshot must work"), changed);
}

#[test]
fn prepared_atom_creation_is_revision_bound_and_consumed_once() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let molecule = PersistentId::new("m").expect("fixture molecule ID is valid");
    let atom = PersistentId::new("created").expect("fixture atom ID is valid");
    let mut pending = session
        .prepare_create_atom(0, &molecule, atom.clone(), "O")
        .expect("candidate must prepare");
    assert_eq!(pending.identifier(), &atom);

    let accepted = session
        .commit_create_atom(0, &mut pending)
        .expect("candidate must commit once");
    assert_eq!(accepted.revision(), 1);
    assert!(accepted.cdml().contains("id=\"created\""));
    assert!(matches!(
        session.commit_create_atom(1, &mut pending),
        Err(DocumentSessionError::PreparedOperationConsumed)
    ));
}

#[test]
fn prepared_atom_creation_preserves_the_target_cdml_namespace() {
    let mut session = DocumentSession::load(PREFIXED_SOURCE).expect("source must load");
    let molecule = PersistentId::new("m").expect("fixture molecule ID is valid");
    let atom = PersistentId::new("created").expect("fixture atom ID is valid");
    let mut pending = session
        .prepare_create_atom(0, &molecule, atom, "O")
        .expect("candidate must prepare");
    let snapshot = session
        .commit_create_atom(0, &mut pending)
        .expect("candidate must commit");
    let reparsed = DocumentSession::load(snapshot.cdml()).expect("result must remain CDML");
    assert!(
        reparsed
            .snapshot()
            .expect("reparsed snapshot must work")
            .cdml()
            .contains("id=\"created\"")
    );
}

#[test]
fn stale_or_rejected_atom_creation_does_not_consume_a_candidate() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let molecule = PersistentId::new("m").expect("fixture molecule ID is valid");
    let atom = PersistentId::new("created").expect("fixture atom ID is valid");
    let mut pending = session
        .prepare_create_atom(0, &molecule, atom, "O")
        .expect("candidate must prepare");
    let changed = session.submit(0, set_atom("N")).expect("edit must succeed");

    assert!(matches!(
        session.commit_create_atom(0, &mut pending),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));
    assert!(matches!(
        session.commit_create_atom(changed.revision(), &mut pending),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));
    assert!(
        !session
            .snapshot()
            .expect("snapshot must work")
            .cdml()
            .contains("id=\"created\"")
    );
    assert!(matches!(
        session.prepare_create_atom(
            changed.revision(),
            &molecule,
            PersistentId::new("a").expect("valid ID"),
            "2"
        ),
        Err(DocumentSessionError::Operation(_))
    ));
}

#[test]
fn revision_exhaustion_is_a_typed_error_without_state_change() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    session.set_revision_for_test(u64::MAX);
    let before = session.snapshot().expect("snapshot must work");
    assert!(matches!(
        session.submit(u64::MAX, set_atom("N")),
        Err(DocumentSessionError::RevisionExhausted)
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn recovery_export_and_observation_never_commit_or_mark_the_session_saved() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session.submit(0, set_atom("N")).expect("edit must succeed");
    let observed = session
        .observe(changed.revision())
        .expect("observe must work");
    assert_eq!(observed.snapshot(), &changed);
    assert!(matches!(
        session.observe(0),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));

    let directory = std::fs::canonicalize(std::env::temp_dir())
        .expect("temporary directory must resolve")
        .join(format!("ferrum-recovery-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("directory must create");
    let output = directory.join("recovery.cdml");
    let publication = session
        .recovery_export(&output, changed.revision())
        .expect("recovery export must publish");
    assert_eq!(publication.snapshot(), &changed);
    assert_eq!(session.snapshot().expect("snapshot must work"), changed);
    std::fs::remove_dir_all(directory).expect("directory cleanup must work");
}

#[test]
fn new_edit_after_undo_discards_the_redo_branch() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let first = session.submit(0, set_atom("N")).expect("first edit");
    let second = session
        .submit(first.revision(), set_atom("O"))
        .expect("second edit");
    let undone = session.undo(second.revision()).expect("undo must work");
    let branched = session
        .submit(undone.revision(), set_atom("S"))
        .expect("branch edit");
    assert!(matches!(
        session.redo(branched.revision()),
        Err(DocumentSessionError::HistoryUnavailable)
    ));
}

#[test]
fn saved_content_stays_clean_after_its_history_entry_is_evicted() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let first = session.submit(0, set_atom("N")).expect("first edit");
    session
        .record_save_outcome_for_test(PublicationDurability::Confirmed)
        .expect("confirmed save must update baseline");

    let mut revision = first.revision();
    for index in 0..20 {
        let element = if index % 2 == 0 { "C" } else { "N" };
        revision = session
            .submit(revision, set_atom(element))
            .expect("alternating edit must succeed")
            .revision();
    }
    let snapshot = session.snapshot().expect("snapshot must work");
    assert_eq!(snapshot.revision(), revision);
    assert!(snapshot.cdml().contains("name=\"N\""));
    assert!(
        !snapshot.is_dirty(),
        "saved content must remain clean even after its old history entry is evicted"
    );
}

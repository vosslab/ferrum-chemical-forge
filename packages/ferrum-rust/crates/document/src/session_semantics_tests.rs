use super::{
    DocumentBondOrderV1, DocumentBondPresentationV1, DocumentObjectIdV1, DocumentSession,
    DocumentSessionError, Point3V1, PublicationDurability, SaveOutcome, SessionOperation,
    SessionOperationError, SessionOperationV1, TypedDocumentError,
};

mod arrow_properties;
mod atom_marks;
mod atom_number;
mod atom_properties;
mod atom_rotation;
mod bond_mutation;
mod bond_properties;
mod bracket_creation;
mod directed_bond_endpoint_reverse;
mod drawing_standard;
mod explicit_fragment;
mod geometric_properties;
mod geometry_repair;
mod linear_form_convert;
mod molecule_name;
mod paper_properties;
mod plus_properties;
mod presentation_deletion;
mod presentation_stack_reorder;
mod text_properties;
mod top_level_transform;
mod wavy_creation;
mod wavy_properties;

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
    "<point x=\"1\" y=\"2\"/></atom></molecule></cdml>"
);

const BOND_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/></atom>",
    "<atom id=\"b\" name=\"O\"><point x=\"3\" y=\"2\"/></atom>",
    "</molecule><molecule id=\"other\">",
    "<atom id=\"c\" name=\"N\"><point x=\"5\" y=\"2\"/></atom>",
    "</molecule></cdml>",
);

const MISSING_POINT_SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"/></molecule></cdml>";

const DELETE_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/></atom>",
    "<atom id=\"b\" name=\"O\"><point x=\"3\" y=\"2\"/></atom>",
    "<atom id=\"c\" name=\"N\"><point x=\"5\" y=\"2\"/></atom>",
    "<bond id=\"ab\" type=\"n1\" start=\"a\" end=\"b\"/>",
    "<bond id=\"bc\" type=\"n1\" start=\"b\" end=\"c\"/>",
    "<bond id=\"ac\" type=\"n1\" start=\"a\" end=\"c\"/>",
    "</molecule><opaque id=\"payload\" retained-bond=\"ab\">",
    "<nested start=\"a\" end=\"b\"/></opaque></cdml>",
);

fn set_atom(element: &str) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetAtomElement {
        atom_id: "a".to_owned(),
        element: element.to_owned(),
    })
}

fn move_atom(x: f64, y: f64, z: f64) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetAtomPosition {
        atom_id: "a".to_owned(),
        position: Point3V1::new(x, y, z).expect("finite test position"),
    })
}

fn delete_atom(atom_id: &str) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::DeleteAtom {
        atom_id: atom_id.to_owned(),
    })
}

fn position() -> Point3V1 {
    Point3V1::new(3.0, 4.0, 0.0).expect("finite test position")
}

fn molecule_object_id(session: &DocumentSession, revision: u64) -> DocumentObjectIdV1 {
    session
        .observe(revision)
        .expect("fixture observation must project")
        .projection()
        .molecules()[0]
        .document_object_id()
        .clone()
}

fn atom_object_ids(
    session: &DocumentSession,
    revision: u64,
) -> (DocumentObjectIdV1, DocumentObjectIdV1, DocumentObjectIdV1) {
    let observation = session.observe(revision).expect("fixture must project");
    let molecules = observation.projection().molecules();
    (
        molecules[0].atoms()[0].document_object_id().clone(),
        molecules[0].atoms()[1].document_object_id().clone(),
        molecules[1].atoms()[0].document_object_id().clone(),
    )
}

#[test]
fn typed_operation_is_revisioned_and_noop_is_history_free() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let baseline = session.snapshot().expect("snapshot must work");
    assert_eq!(baseline.revision(), 0);
    assert!(!baseline.is_dirty());

    let no_change = session
        .apply_document_operation_v1(0, set_atom("C"))
        .expect("no-op must succeed");
    assert_eq!(no_change.observation().snapshot(), &baseline);
    assert_eq!(
        no_change.observation().snapshot().revision(),
        no_change.observation().projection().revision()
    );
    assert_eq!(
        no_change.observation().snapshot().digest(),
        no_change.observation().projection().digest()
    );

    let changed = session
        .apply_document_operation_v1(0, set_atom("N"))
        .expect("edit must succeed");
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert!(changed.observation().snapshot().is_dirty());
    assert!(
        changed
            .observation()
            .snapshot()
            .cdml()
            .contains("name=\"N\"")
    );
    assert_eq!(
        changed.observation().snapshot().revision(),
        changed.observation().projection().revision()
    );
    assert_eq!(
        changed.observation().snapshot().digest(),
        changed.observation().projection().digest()
    );

    assert!(matches!(
        session.apply_document_operation_v1(0, set_atom("O")),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(
        session.snapshot().expect("snapshot must work"),
        *changed.observation().snapshot()
    );
}

#[test]
fn atom_position_operation_is_finite_history_aware_and_preserves_point_structure() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let baseline = session.snapshot().expect("snapshot must work");
    let no_change = session
        .apply_document_operation_v1(0, move_atom(1.0, 2.0, 0.0))
        .expect("same point must be a no-op");
    assert_eq!(no_change.observation().snapshot(), &baseline);

    let moved = session
        .apply_document_operation_v1(0, move_atom(7.5, 8.25, 0.0))
        .expect("finite point must move");
    let atom = &moved.observation().projection().molecules()[0].atoms()[0];
    assert_eq!(moved.observation().snapshot().revision(), 1);
    assert_eq!(atom.position(), Point3V1::new(7.5, 8.25, 0.0).unwrap());
    assert!(
        moved
            .observation()
            .snapshot()
            .cdml()
            .contains("<point x=\"7.5\" y=\"8.25\"/>")
    );

    let undone = session.undo(1).expect("move must be undoable");
    assert_eq!(
        undone.observation().projection().molecules()[0].atoms()[0].position(),
        Point3V1::new(1.0, 2.0, 0.0).unwrap()
    );
    let redone = session.redo(2).expect("move must be redoable");
    assert_eq!(
        redone.observation().projection().molecules()[0].atoms()[0].position(),
        Point3V1::new(7.5, 8.25, 0.0).unwrap()
    );
}

#[test]
fn atom_position_rejects_unknown_or_unpositioned_atoms_without_state_change() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    let unknown = SessionOperation::V1(SessionOperationV1::SetAtomPosition {
        atom_id: "missing".to_owned(),
        position: position(),
    });
    assert!(matches!(
        session.apply_document_operation_v1(0, unknown),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownAtom(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);

    let mut missing_point =
        DocumentSession::load(MISSING_POINT_SOURCE).expect("structural source must load");
    let before = missing_point.snapshot().expect("snapshot must work");
    assert!(matches!(
        missing_point.apply_document_operation_v1(0, move_atom(3.0, 4.0, 0.0)),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::MissingAtomPosition(_))
        ))
    ));
    assert_eq!(
        missing_point.snapshot().expect("snapshot must work"),
        before
    );
}

#[test]
fn atom_deletion_removes_only_incident_bonds_and_is_one_history_entry() {
    let mut session = DocumentSession::load(DELETE_SOURCE).expect("source must load");
    let deleted = session
        .apply_document_operation_v1(0, delete_atom("b"))
        .expect("durable atom must delete");
    let molecule = &deleted.observation().projection().molecules()[0];
    assert_eq!(deleted.observation().snapshot().revision(), 1);
    assert_eq!(
        molecule
            .atoms()
            .iter()
            .map(|atom| atom.source_id().expect("fixture atom is durable"))
            .collect::<Vec<_>>(),
        ["a", "c"]
    );
    assert_eq!(molecule.bonds().len(), 1);
    assert_eq!(molecule.bonds()[0].source_id(), Some("ac"));

    let undone = session.undo(1).expect("deletion must undo once");
    let restored = &undone.observation().projection().molecules()[0];
    assert_eq!(restored.atoms().len(), 3);
    assert_eq!(restored.bonds().len(), 3);
    let redone = session.redo(2).expect("deletion must redo once");
    let redone_molecule = &redone.observation().projection().molecules()[0];
    assert_eq!(redone_molecule.atoms().len(), 2);
    assert_eq!(redone_molecule.bonds().len(), 1);
}

#[test]
fn atom_deletion_rejects_unknown_identity_without_state_change() {
    let mut session = DocumentSession::load(DELETE_SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    assert!(matches!(
        session.apply_document_operation_v1(0, delete_atom("missing")),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownAtom(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn backend_history_navigation_publishes_monotonic_revisions() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    assert!(!session.can_undo());
    assert!(!session.can_redo());
    let changed = session
        .apply_document_operation_v1(0, set_atom("N"))
        .expect("edit must succeed");
    assert!(session.can_undo());
    assert!(!session.can_redo());
    let undone = session
        .undo(changed.observation().snapshot().revision())
        .expect("undo must succeed");
    assert!(!session.can_undo());
    assert!(session.can_redo());
    assert_eq!(undone.observation().snapshot().revision(), 2);
    assert!(
        undone
            .observation()
            .snapshot()
            .cdml()
            .contains("name=\"C\"")
    );
    assert_eq!(
        undone.observation().snapshot().digest(),
        undone.observation().projection().digest()
    );

    let redone = session
        .redo(undone.observation().snapshot().revision())
        .expect("redo must succeed");
    assert!(session.can_undo());
    assert!(!session.can_redo());
    assert_eq!(redone.observation().snapshot().revision(), 3);
    assert!(
        redone
            .observation()
            .snapshot()
            .cdml()
            .contains("name=\"N\"")
    );
    assert!(redone.observation().snapshot().is_dirty());
    assert_eq!(
        redone.observation().snapshot().digest(),
        redone.observation().projection().digest()
    );
}

#[test]
fn rejected_operation_cannot_change_the_authoritative_snapshot() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    assert!(matches!(
        session.apply_document_operation_v1(0, set_atom("2")),
        Err(DocumentSessionError::Operation(_))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn confirmed_save_advances_the_baseline_without_losing_published_identity() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session
        .apply_document_operation_v1(0, set_atom("N"))
        .expect("edit must succeed");
    let saved = session
        .record_save_outcome_for_test(PublicationDurability::Confirmed)
        .expect("confirmed outcome must advance baseline");
    assert_eq!(saved.outcome(), SaveOutcome::Confirmed);
    assert_eq!(saved.published_snapshot(), changed.observation().snapshot());
    assert!(!saved.snapshot().is_dirty());
    assert_eq!(
        saved.snapshot().revision(),
        changed.observation().snapshot().revision()
    );

    let undone = session
        .undo(saved.snapshot().revision())
        .expect("undo must succeed");
    assert!(undone.observation().snapshot().is_dirty());
    let redone = session
        .redo(undone.observation().snapshot().revision())
        .expect("redo must succeed");
    assert!(!redone.observation().snapshot().is_dirty());
}

#[test]
fn unconfirmed_replacement_keeps_the_session_dirty_for_verification_or_recovery() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session
        .apply_document_operation_v1(0, set_atom("N"))
        .expect("edit must succeed");
    let outcome = session
        .record_save_outcome_for_test(PublicationDurability::DirectoryEntryUnconfirmed)
        .expect("injected outcome must report");

    assert_eq!(outcome.outcome(), SaveOutcome::DirectoryEntryUnconfirmed);
    assert_eq!(
        outcome.published_snapshot(),
        changed.observation().snapshot()
    );
    assert_eq!(outcome.snapshot(), changed.observation().snapshot());
    assert!(outcome.snapshot().is_dirty());
    assert_eq!(
        session.snapshot().expect("snapshot must work"),
        *changed.observation().snapshot()
    );
}

#[test]
fn generic_atom_creation_returns_the_committed_identifier() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let molecule = molecule_object_id(&session, 0);
    let request = crate::SessionOperationTransitionRequestV1::new(
        0,
        crate::SessionOperation::V1(crate::SessionOperationV1::CreateAtomV1(
            crate::CreateAtomV1::new(molecule, "O".to_owned(), position()),
        )),
        crate::TransitionAuthorizationV1::authoring_capability(
            session.issue_authoring_capability_v1(),
        ),
    );
    let mut prepared = session
        .prepare_session_operation_transition_v1(request)
        .expect("atom prepares");
    let accepted = session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("atom commits");
    assert!(matches!(
        accepted.outcome(),
        crate::SessionOperationOutcomeV1::AtomCreatedV1(outcome) if !outcome.atom_identifier().as_str().is_empty()
    ));
    assert_eq!(accepted.observation().snapshot().revision(), 1);
}

#[test]
fn generic_bond_creation_rejects_invalid_topology_and_returns_the_committed_identifier() {
    let mut session = DocumentSession::load(BOND_SOURCE).expect("source must load");
    let (start, end, other) = atom_object_ids(&session, 0);
    for (first, second) in [(start.clone(), start.clone()), (start.clone(), other)] {
        let request = crate::SessionOperationTransitionRequestV1::new(
            0,
            crate::SessionOperation::V1(crate::SessionOperationV1::CreateBondV1(
                crate::CreateBondV1::new(first, second, DocumentBondPresentationV1::SolidWedge),
            )),
            crate::TransitionAuthorizationV1::authoring_capability(
                session.issue_authoring_capability_v1(),
            ),
        );
        assert!(
            session
                .prepare_session_operation_transition_v1(request)
                .is_err()
        );
        assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
    }
    let request = crate::SessionOperationTransitionRequestV1::new(
        0,
        crate::SessionOperation::V1(crate::SessionOperationV1::CreateBondV1(
            crate::CreateBondV1::new(start, end, DocumentBondPresentationV1::SolidWedge),
        )),
        crate::TransitionAuthorizationV1::authoring_capability(
            session.issue_authoring_capability_v1(),
        ),
    );
    let mut prepared = session
        .prepare_session_operation_transition_v1(request)
        .expect("bond prepares");
    let accepted = session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("bond commits");
    assert!(matches!(
        accepted.outcome(),
        crate::SessionOperationOutcomeV1::BondCreatedV1(outcome) if !outcome.bond_identifier().as_str().is_empty()
    ));
}

#[test]
fn revision_exhaustion_is_a_typed_error_without_state_change() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    session.set_revision_for_test(u64::MAX);
    let before = session.snapshot().expect("snapshot must work");
    assert!(matches!(
        session.apply_document_operation_v1(u64::MAX, set_atom("N")),
        Err(DocumentSessionError::RevisionExhausted)
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn recovery_export_and_observation_never_commit_or_mark_the_session_saved() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session
        .apply_document_operation_v1(0, set_atom("N"))
        .expect("edit must succeed");
    let observed = session
        .observe(changed.observation().snapshot().revision())
        .expect("observe must work");
    assert_eq!(observed.snapshot(), changed.observation().snapshot());
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
        .recovery_export(&output, changed.observation().snapshot().revision())
        .expect("recovery export must publish");
    assert_eq!(publication.snapshot(), changed.observation().snapshot());
    assert_eq!(
        session.snapshot().expect("snapshot must work"),
        *changed.observation().snapshot()
    );
    std::fs::remove_dir_all(directory).expect("directory cleanup must work");
}

#[test]
fn new_edit_after_undo_discards_the_redo_branch() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    assert!(!session.can_undo());
    assert!(!session.can_redo());
    let changed = session
        .apply_document_operation_v1(0, set_atom("N"))
        .expect("first edit");
    assert!(session.can_undo());
    assert!(!session.can_redo());
    let undone = session
        .undo(changed.observation().snapshot().revision())
        .expect("undo must work");
    assert!(!session.can_undo());
    assert!(session.can_redo());
    let branched = session
        .apply_document_operation_v1(undone.observation().snapshot().revision(), set_atom("O"))
        .expect("branch edit");
    assert!(session.can_undo());
    assert!(!session.can_redo());
    assert!(matches!(
        session.redo(branched.observation().snapshot().revision()),
        Err(DocumentSessionError::HistoryUnavailable)
    ));
}

#[test]
fn saved_content_stays_clean_after_its_history_entry_is_evicted() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let first = session
        .apply_document_operation_v1(0, set_atom("N"))
        .expect("first edit");
    session
        .record_save_outcome_for_test(PublicationDurability::Confirmed)
        .expect("confirmed save must update baseline");

    let mut revision = first.observation().snapshot().revision();
    for index in 0..20 {
        let element = if index % 2 == 0 { "C" } else { "N" };
        revision = session
            .apply_document_operation_v1(revision, set_atom(element))
            .expect("alternating edit must succeed")
            .observation()
            .snapshot()
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

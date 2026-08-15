use super::{
    DocumentBondOrderV1, DocumentBondPresentationV1, DocumentObjectIdV1, DocumentSession,
    DocumentSessionError, Point3V1, PublicationDurability, SaveOutcome, SessionOperation,
    SessionOperationError, SessionOperationV1, TypedClass, TypedDocumentError,
};

mod arrow_properties;
mod atom_marks;
mod atom_number;
mod atom_properties;
mod atom_rotation;
mod bond_mutation;
mod bond_properties;
mod bracket_creation;
mod direct_haworth_insertion;
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
mod standalone_haworth_insertion;
mod text_properties;
mod top_level_transform;
mod wavy_creation;
mod wavy_properties;

const SOURCE: &str = concat!(
    "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
    "<point x=\"1\" y=\"2\"/></atom></molecule></cdml>"
);

const PREFIXED_SOURCE: &str = concat!(
    "<cdml:cdml xmlns:cdml=\"http://www.freesoftware.fsf.org/bkchem/cdml\">",
    "<cdml:molecule id=\"m\"/></cdml:cdml>"
);

const RESERVED_GENERATED_ID_SOURCE: &str = concat!(
    "<cdml><opaque id=\"ferrum-atom-v1-0\"><retained/></opaque>",
    "<molecule id=\"m\"><atom id=\"a\" name=\"C\">",
    "<point x=\"1\" y=\"2\"/></atom></molecule></cdml>"
);

const BOND_SOURCE: &str = concat!(
    "<cdml version=\"26.08\"><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/></atom>",
    "<atom id=\"b\" name=\"O\"><point x=\"3\" y=\"2\"/></atom>",
    "</molecule><molecule id=\"other\">",
    "<atom id=\"c\" name=\"N\"><point x=\"5\" y=\"2\"/></atom>",
    "</molecule></cdml>",
);

const MISSING_POINT_SOURCE: &str =
    "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"/></molecule></cdml>";

const DELETE_SOURCE: &str = concat!(
    "<cdml><molecule id=\"m\">",
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
        .id()
        .expect("fixture molecule has a durable ID")
        .clone()
}

fn atom_object_ids(
    session: &DocumentSession,
    revision: u64,
) -> (DocumentObjectIdV1, DocumentObjectIdV1, DocumentObjectIdV1) {
    let observation = session.observe(revision).expect("fixture must project");
    let molecules = observation.projection().molecules();
    (
        molecules[0].atoms()[0]
            .id()
            .expect("first atom is durable")
            .clone(),
        molecules[0].atoms()[1]
            .id()
            .expect("second atom is durable")
            .clone(),
        molecules[1].atoms()[0]
            .id()
            .expect("other molecule atom is durable")
            .clone(),
    )
}

fn first_atom_object_id(session: &DocumentSession, revision: u64) -> DocumentObjectIdV1 {
    session
        .observe(revision)
        .expect("fixture must project")
        .projection()
        .molecules()[0]
        .atoms()[0]
        .id()
        .expect("fixture atom has a durable ID")
        .clone()
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
    assert_eq!(no_change.observation().snapshot(), &baseline);
    assert_eq!(
        no_change.observation().snapshot().revision(),
        no_change.observation().projection().revision()
    );
    assert_eq!(
        no_change.observation().snapshot().digest(),
        no_change.observation().projection().digest()
    );

    let changed = session.submit(0, set_atom("N")).expect("edit must succeed");
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
        session.submit(0, set_atom("O")),
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
        .submit(0, move_atom(1.0, 2.0, 0.0))
        .expect("same point must be a no-op");
    assert_eq!(no_change.observation().snapshot(), &baseline);

    let moved = session
        .submit(0, move_atom(7.5, 8.25, 0.0))
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
        session.submit(0, unknown),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownAtom(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);

    let mut missing_point =
        DocumentSession::load(MISSING_POINT_SOURCE).expect("structural source must load");
    let before = missing_point.snapshot().expect("snapshot must work");
    assert!(matches!(
        missing_point.submit(0, move_atom(3.0, 4.0, 0.0)),
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
        .submit(0, delete_atom("b"))
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
        session.submit(0, delete_atom("missing")),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownAtom(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn backend_history_navigation_publishes_monotonic_revisions() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session.submit(0, set_atom("N")).expect("edit must succeed");
    let undone = session
        .undo(changed.observation().snapshot().revision())
        .expect("undo must succeed");
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
    let changed = session.submit(0, set_atom("N")).expect("edit must succeed");
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
fn prepared_atom_creation_is_revision_bound_and_consumed_once() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let molecule = molecule_object_id(&session, 0);
    let mut pending = session
        .prepare_create_atom_v1(0, &molecule, "O", position())
        .expect("candidate must prepare");
    assert_eq!(pending.identifier().as_str(), "ferrum-atom-v1-0");

    let accepted = session
        .commit_create_atom(0, &mut pending)
        .expect("positioned candidate must commit");
    assert_eq!(accepted.observation().snapshot().revision(), 1);
    assert!(
        accepted
            .observation()
            .snapshot()
            .cdml()
            .contains("id=\"ferrum-atom-v1-0\"")
    );
    assert!(
        accepted
            .observation()
            .snapshot()
            .cdml()
            .contains("x=\"3\" y=\"4\" z=\"0\"")
    );
    assert!(matches!(
        session.commit_create_atom(1, &mut pending),
        Err(DocumentSessionError::PreparedOperationConsumed)
    ));
}

#[test]
fn foreign_session_rejection_preserves_the_owning_prepared_candidate() {
    let mut owner = DocumentSession::load(SOURCE).expect("owner source must load");
    let mut foreign = DocumentSession::load(SOURCE).expect("foreign source must load");
    let molecule = molecule_object_id(&owner, 0);
    let mut pending = owner
        .prepare_create_atom_v1(0, &molecule, "O", position())
        .expect("candidate must prepare");

    assert!(matches!(
        foreign.commit_create_atom(0, &mut pending),
        Err(DocumentSessionError::PreparedOperationForeignSession)
    ));
    assert!(
        !foreign
            .snapshot()
            .expect("foreign snapshot must work")
            .cdml()
            .contains("id=\"ferrum-atom-v1-0\"")
    );

    let accepted = owner
        .commit_create_atom(0, &mut pending)
        .expect("owner candidate must remain retryable");
    assert_eq!(accepted.observation().snapshot().revision(), 1);
}

#[test]
fn generated_atom_identity_skips_ids_reserved_by_opaque_content() {
    let mut session =
        DocumentSession::load(RESERVED_GENERATED_ID_SOURCE).expect("source must load");
    let molecule = molecule_object_id(&session, 0);
    let pending = session
        .prepare_create_atom_v1(0, &molecule, "O", position())
        .expect("allocator must skip the retained opaque ID");

    assert_eq!(pending.identifier().as_str(), "ferrum-atom-v1-1");
}

#[test]
fn invalid_insert_position_is_rejected_before_preparation_or_session_mutation() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let molecule = molecule_object_id(&session, 0);
    let before = session.snapshot().expect("snapshot must work");
    assert!(Point3V1::new(f64::NAN, 4.0, 0.0).is_err());
    assert!(
        session
            .prepare_create_atom_v1(0, &molecule, "O", position())
            .is_ok()
    );
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn rejected_atom_creation_preserves_the_target_cdml_namespace() {
    let mut session = DocumentSession::load(PREFIXED_SOURCE).expect("source must load");
    let molecule = molecule_object_id(&session, 0);
    let mut pending = session
        .prepare_create_atom_v1(0, &molecule, "O", position())
        .expect("candidate must prepare");
    let accepted = session.commit_create_atom(0, &mut pending).expect("commit");
    let snapshot = accepted.observation().snapshot();
    let reparsed = DocumentSession::load(snapshot.cdml()).expect("result must remain CDML");
    assert!(
        reparsed
            .snapshot()
            .expect("reparsed snapshot must work")
            .cdml()
            .contains("<cdml:cdml")
    );
}

#[test]
fn stale_or_rejected_atom_creation_does_not_consume_a_candidate() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let molecule = molecule_object_id(&session, 0);
    let mut pending = session
        .prepare_create_atom_v1(0, &molecule, "O", position())
        .expect("candidate must prepare");
    let changed = session.submit(0, set_atom("N")).expect("edit must succeed");

    assert!(matches!(
        session.commit_create_atom(0, &mut pending),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));
    assert!(matches!(
        session.commit_create_atom(changed.observation().snapshot().revision(), &mut pending),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));
    assert!(
        !session
            .snapshot()
            .expect("snapshot must work")
            .cdml()
            .contains("id=\"ferrum-atom-v1-0\"")
    );
    assert!(matches!(
        session.prepare_create_atom_v1(
            changed.observation().snapshot().revision(),
            &molecule,
            "2",
            position()
        ),
        Err(DocumentSessionError::Operation(_))
    ));
}

#[test]
fn rejected_preparation_does_not_advance_the_generated_identity() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let molecule = molecule_object_id(&session, 0);
    assert!(matches!(
        session.prepare_create_atom_v1(0, &molecule, "2", position()),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(_)
        ))
    ));

    let pending = session
        .prepare_create_atom_v1(0, &molecule, "O", position())
        .expect("valid preparation must succeed");
    assert_eq!(pending.identifier().as_str(), "ferrum-atom-v1-0");
}

#[test]
fn abandoned_preparations_never_reuse_generated_identities() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let molecule = molecule_object_id(&session, 0);
    let first = session
        .prepare_create_atom_v1(0, &molecule, "O", position())
        .expect("first preparation must succeed");
    let second = session
        .prepare_create_atom_v1(0, &molecule, "N", position())
        .expect("second preparation must succeed");

    assert_eq!(first.identifier().as_str(), "ferrum-atom-v1-0");
    assert_eq!(second.identifier().as_str(), "ferrum-atom-v1-1");
    assert_eq!(
        session.snapshot().expect("snapshot must work").revision(),
        0
    );
}

#[test]
fn create_atom_requires_a_current_durable_molecule_selector() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let observation = session.observe(0).expect("fixture must project");
    let atom = observation.projection().molecules()[0].atoms()[0]
        .id()
        .expect("fixture atom has a durable ID")
        .clone();
    assert!(matches!(
        session.prepare_create_atom_v1(0, &atom, "O", position()),
        Err(DocumentSessionError::Operation(
            SessionOperationError::InvalidCreateAtomTarget(_)
        ))
    ));

    let unknown = DocumentObjectIdV1::from_class_source(TypedClass::Molecule.name(), "missing");
    assert!(matches!(
        session.prepare_create_atom_v1(0, &unknown, "O", position()),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownDocumentObject(_)
        ))
    ));
}

#[test]
fn generated_atom_identifier_exhaustion_is_typed_and_state_preserving() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let molecule = molecule_object_id(&session, 0);
    session.set_next_generated_atom_sequence_for_test(None);
    let before = session.snapshot().expect("snapshot must work");

    assert!(matches!(
        session.prepare_create_atom_v1(0, &molecule, "O", position()),
        Err(DocumentSessionError::Operation(
            SessionOperationError::AtomIdentifierExhausted
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn prepared_bond_creation_preserves_closed_presentation_through_history() {
    let mut session = DocumentSession::load(BOND_SOURCE).expect("source must load");
    let (start, end, _) = atom_object_ids(&session, 0);
    let mut pending = session
        .prepare_create_bond_v2(0, &start, &end, DocumentBondPresentationV1::SolidWedge)
        .expect("bond candidate must prepare");
    assert_eq!(pending.identifier().as_str(), "ferrum-bond-v1-0");

    let accepted = session
        .commit_create_bond(0, &mut pending)
        .expect("bond candidate must commit");
    let accepted_snapshot = accepted.observation().snapshot();
    assert_eq!(accepted_snapshot.revision(), 1);
    let bond = &accepted.observation().projection().molecules()[0].bonds()[0];
    assert_eq!(bond.source_type(), Some("w1"));
    assert_eq!(bond.start().object_id(), Some(&start));
    assert_eq!(bond.end().object_id(), Some(&end));

    let undone = session.undo(1).expect("bond insertion must be undoable");
    assert!(!undone.observation().snapshot().cdml().contains("<bond"));
    let redone = session.redo(2).expect("bond insertion must be redoable");
    assert_eq!(
        redone.observation().projection().molecules()[0].bonds()[0].source_type(),
        Some("w1"),
    );
    assert!(matches!(
        session.commit_create_bond(3, &mut pending),
        Err(DocumentSessionError::PreparedOperationConsumed)
    ));
}

#[test]
fn bond_creation_rejects_self_cross_molecule_and_duplicate_edges_without_mutation() {
    let mut session = DocumentSession::load(BOND_SOURCE).expect("source must load");
    let (start, end, other) = atom_object_ids(&session, 0);
    let before = session.snapshot().expect("snapshot must work");
    assert!(matches!(
        session.prepare_create_bond_v2(0, &start, &start, DocumentBondPresentationV1::SolidWedge),
        Err(DocumentSessionError::Operation(
            SessionOperationError::CreateBondSelfLoop(_)
        ))
    ));
    assert!(matches!(
        session.prepare_create_bond_v2(0, &start, &other, DocumentBondPresentationV1::SolidWedge),
        Err(DocumentSessionError::Operation(
            SessionOperationError::CreateBondAcrossMolecules
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);

    let mut first = session
        .prepare_create_bond_v2(0, &start, &end, DocumentBondPresentationV1::SolidWedge)
        .expect("valid edge must prepare");
    assert_eq!(first.identifier().as_str(), "ferrum-bond-v1-0");
    session
        .commit_create_bond(0, &mut first)
        .expect("valid edge must commit");
    assert!(matches!(
        session.prepare_create_bond_v2(1, &end, &start, DocumentBondPresentationV1::HashedWedge),
        Err(DocumentSessionError::Operation(
            SessionOperationError::CreateBondDuplicate { .. }
        ))
    ));
}

#[test]
fn prepared_bond_is_foreign_safe_stale_safe_and_preserves_exact_order() {
    let mut owner = DocumentSession::load(BOND_SOURCE).expect("owner source must load");
    let mut foreign = DocumentSession::load(BOND_SOURCE).expect("foreign source must load");
    let (start, end, _) = atom_object_ids(&owner, 0);
    let mut pending = owner
        .prepare_create_bond_v2(0, &start, &end, DocumentBondPresentationV1::HashedWedge)
        .expect("candidate must prepare");
    assert!(matches!(
        foreign.commit_create_bond(0, &mut pending),
        Err(DocumentSessionError::PreparedOperationForeignSession)
    ));
    let changed = owner.submit(0, set_atom("N")).expect("edit must succeed");
    assert!(matches!(
        owner.commit_create_bond(0, &mut pending),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));
    assert!(matches!(
        owner.commit_create_bond(changed.observation().snapshot().revision(), &mut pending),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));
    assert!(
        !owner
            .snapshot()
            .expect("snapshot must work")
            .cdml()
            .contains("<bond")
    );
}

#[test]
fn generated_bond_identifier_exhaustion_is_typed_and_state_preserving() {
    let mut session = DocumentSession::load(BOND_SOURCE).expect("source must load");
    let (start, end, _) = atom_object_ids(&session, 0);
    session.set_next_generated_bond_sequence_for_test(None);
    let before = session.snapshot().expect("snapshot must work");

    assert!(matches!(
        session.prepare_create_bond_v2(
            0,
            &start,
            &end,
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Triple),
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::BondIdentifierExhausted
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn bonded_atom_creation_is_one_revision_one_history_entry_and_two_identities() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let start = first_atom_object_id(&session, 0);
    let before = session.snapshot().expect("snapshot must work");
    let mut pending = session
        .prepare_create_bonded_atom_v2(
            0,
            &start,
            "O",
            position(),
            DocumentBondPresentationV1::HashedWedge,
        )
        .expect("complete candidate must prepare");
    assert_eq!(pending.atom_identifier().as_str(), "ferrum-atom-v1-0");
    assert_eq!(pending.bond_identifier().as_str(), "ferrum-bond-v1-0");
    assert_eq!(session.snapshot().expect("snapshot must work"), before);

    let accepted = session
        .commit_create_bonded_atom(0, &mut pending)
        .expect("complete candidate must commit");
    let molecule = &accepted.observation().projection().molecules()[0];
    assert_eq!(accepted.observation().snapshot().revision(), 1);
    assert_eq!(molecule.atoms().len(), 2);
    assert_eq!(molecule.bonds().len(), 1);
    assert_eq!(molecule.atoms()[1].source_id(), Some("ferrum-atom-v1-0"));
    assert_eq!(molecule.atoms()[1].position(), position());
    assert_eq!(molecule.bonds()[0].source_id(), Some("ferrum-bond-v1-0"));
    assert_eq!(molecule.bonds()[0].source_type(), Some("h1"));
    assert_eq!(molecule.bonds()[0].start().object_id(), Some(&start));
    assert_eq!(
        molecule.bonds()[0].end().source_id(),
        Some("ferrum-atom-v1-0")
    );

    let undone = session.undo(1).expect("composite insertion must undo once");
    let undone_molecule = &undone.observation().projection().molecules()[0];
    assert_eq!(undone_molecule.atoms().len(), 1);
    assert!(undone_molecule.bonds().is_empty());
    let redone = session.redo(2).expect("composite insertion must redo once");
    let redone_molecule = &redone.observation().projection().molecules()[0];
    assert_eq!(redone_molecule.atoms().len(), 2);
    assert_eq!(redone_molecule.bonds().len(), 1);
    assert!(matches!(
        session.commit_create_bonded_atom(3, &mut pending),
        Err(DocumentSessionError::PreparedOperationConsumed)
    ));
}

#[test]
fn bonded_atom_rejection_is_identity_safe_and_foreign_candidates_remain_retryable() {
    let mut owner = DocumentSession::load(SOURCE).expect("source must load");
    let mut foreign = DocumentSession::load(SOURCE).expect("source must load");
    let start = first_atom_object_id(&owner, 0);
    let molecule = molecule_object_id(&owner, 0);
    assert!(matches!(
        owner.prepare_create_bonded_atom_v2(
            0,
            &molecule,
            "O",
            position(),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::InvalidCreateBondTarget(_)
        ))
    ));
    assert!(
        owner
            .prepare_create_bonded_atom_v2(
                0,
                &start,
                "2",
                position(),
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            )
            .is_err()
    );

    let mut pending = owner
        .prepare_create_bonded_atom_v2(
            0,
            &start,
            "N",
            position(),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Double),
        )
        .expect("valid candidate must prepare");
    assert_eq!(pending.atom_identifier().as_str(), "ferrum-atom-v1-0");
    assert_eq!(pending.bond_identifier().as_str(), "ferrum-bond-v1-0");
    assert!(matches!(
        foreign.commit_create_bonded_atom(0, &mut pending),
        Err(DocumentSessionError::PreparedOperationForeignSession)
    ));
    let accepted = owner
        .commit_create_bonded_atom(0, &mut pending)
        .expect("owner must retain its candidate");
    assert!(
        accepted
            .observation()
            .snapshot()
            .cdml()
            .contains("id=\"ferrum-bond-v1-0\" type=\"n2\"")
    );
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
    let first = session.submit(0, set_atom("N")).expect("first edit");
    let second = session
        .submit(first.observation().snapshot().revision(), set_atom("O"))
        .expect("second edit");
    let undone = session
        .undo(second.observation().snapshot().revision())
        .expect("undo must work");
    let branched = session
        .submit(undone.observation().snapshot().revision(), set_atom("S"))
        .expect("branch edit");
    assert!(matches!(
        session.redo(branched.observation().snapshot().revision()),
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

    let mut revision = first.observation().snapshot().revision();
    for index in 0..20 {
        let element = if index % 2 == 0 { "C" } else { "N" };
        revision = session
            .submit(revision, set_atom(element))
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

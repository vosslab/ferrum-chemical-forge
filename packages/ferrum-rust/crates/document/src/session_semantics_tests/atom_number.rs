use super::{DocumentSession, DocumentSessionError, SessionOperationError};
use crate::{SessionOperation, SessionOperationV1, TypedDocumentError, VisibilityV1};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/>",
    "<opaque retained=\"yes\"/></atom></molecule>",
    "<molecule id=\"other\"><atom id=\"b\" name=\"O\">",
    "<point x=\"3\" y=\"4\"/></atom></molecule></cdml>",
);

fn assign(molecule_id: &str, atom_id: &str, number: u64, show_number: bool) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetAtomNumber {
        molecule_id: molecule_id.to_owned(),
        atom_id: atom_id.to_owned(),
        number: Some(number),
        show_number: Some(show_number),
    })
}

fn clear(molecule_id: &str, atom_id: &str) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetAtomNumber {
        molecule_id: molecule_id.to_owned(),
        atom_id: atom_id.to_owned(),
        number: None,
        show_number: None,
    })
}

#[test]
fn assign_clear_undo_and_redo_preserve_the_retained_atom() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let assigned = session
        .apply_document_operation_v1(0, assign("m", "a", 11, false))
        .expect("number assignment must succeed");
    let atom = &assigned.observation().projection().molecules()[0].atoms()[0];
    assert_eq!(atom.number(), Some(11));
    assert_eq!(atom.show_number(), Some(VisibilityV1::Disabled));
    assert!(
        assigned
            .observation()
            .snapshot()
            .cdml()
            .contains("<opaque retained=\"yes\"/>")
    );

    let cleared = session
        .apply_document_operation_v1(1, clear("m", "a"))
        .expect("number clear must succeed");
    let atom = &cleared.observation().projection().molecules()[0].atoms()[0];
    assert_eq!((atom.number(), atom.show_number()), (None, None));
    assert!(!cleared.observation().snapshot().cdml().contains("number="));

    let undone = session.undo(2).expect("clear must be undoable");
    assert_eq!(
        undone.observation().projection().molecules()[0].atoms()[0].number(),
        Some(11)
    );
    let redone = session.redo(3).expect("clear must be redoable");
    assert_eq!(
        redone.observation().projection().molecules()[0].atoms()[0].number(),
        None
    );
}

#[test]
fn matching_assignment_and_empty_clear_are_history_free() {
    let source = SOURCE.replace(
        "id=\"a\" name=\"C\"",
        "id=\"a\" name=\"C\" number=\"7\" show_number=\"yes\"",
    );
    let mut session = DocumentSession::load(&source).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    let same = session
        .apply_document_operation_v1(0, assign("m", "a", 7, true))
        .expect("same pair must be accepted");
    assert_eq!(same.observation().snapshot(), &before);

    let mut empty = DocumentSession::load(SOURCE).expect("source must load");
    let before = empty.snapshot().expect("snapshot must work");
    let cleared = empty
        .apply_document_operation_v1(0, clear("m", "a"))
        .expect("empty clear must be accepted");
    assert_eq!(cleared.observation().snapshot(), &before);
}

#[test]
fn malformed_pairs_and_molecule_mismatch_leave_state_unchanged() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    for operation in [
        SessionOperation::V1(SessionOperationV1::SetAtomNumber {
            molecule_id: "m".to_owned(),
            atom_id: "a".to_owned(),
            number: Some(0),
            show_number: Some(true),
        }),
        SessionOperation::V1(SessionOperationV1::SetAtomNumber {
            molecule_id: "m".to_owned(),
            atom_id: "a".to_owned(),
            number: Some(3),
            show_number: None,
        }),
    ] {
        assert!(matches!(
            session.apply_document_operation_v1(0, operation),
            Err(DocumentSessionError::Operation(
                SessionOperationError::InvalidAtomNumberPair
            ))
        ));
        assert_eq!(session.snapshot().expect("snapshot must work"), before);
    }
    assert!(matches!(
        session.apply_document_operation_v1(0, assign("other", "a", 3, true)),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownAtom(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn direct_legacy_number_mark_is_a_typed_atomic_failure() {
    let source = SOURCE.replace(
        "<opaque retained=\"yes\"/>",
        "<mark type=\"atom_number\"/><opaque retained=\"yes\"/>",
    );
    let mut session = DocumentSession::load(&source).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    assert!(matches!(
        session.apply_document_operation_v1(0, assign("m", "a", 3, true)),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::LegacyAtomNumberMark(_))
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn alternate_canonical_prefix_is_mutated_by_expanded_name() {
    let source = concat!(
        "<c:cdml xmlns:c=\"urn:ferrum:cdml\" version=\"26.07\">",
        "<c:molecule id=\"m\"><c:atom id=\"a\" name=\"C\">",
        "<c:point x=\"1\" y=\"2\"/><f:opaque xmlns:f=\"urn:foreign\"/>",
        "</c:atom></c:molecule></c:cdml>",
    );
    let mut session = DocumentSession::load(source).expect("source must load");
    let changed = session
        .apply_document_operation_v1(0, assign("m", "a", 29, true))
        .expect("prefixed target must be editable");
    let cdml = changed.observation().snapshot().cdml();
    assert!(cdml.contains("number=\"29\" show_number=\"yes\""));
    assert!(cdml.contains("f:opaque"));
}

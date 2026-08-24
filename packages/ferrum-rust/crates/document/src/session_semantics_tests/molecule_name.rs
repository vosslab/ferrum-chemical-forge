use super::{DocumentSession, DocumentSessionError, SessionOperationError};
use crate::{DocumentObjectIdV1, SessionOperation, SessionOperationV1};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\" version=\"26.07\">",
    "<v:note id=\"before\"><v:detail>keep</v:detail></v:note>",
    "<molecule id=\"m\" name=\"old\" role=\"source\">",
    "<atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/>",
    "<v:opaque retained=\"yes\"/></atom></molecule>",
    "<molecule id=\"other\" name=\"unrelated\"><atom id=\"b\" name=\"O\">",
    "<point x=\"3\" y=\"4\"/></atom></molecule></cdml>",
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

fn set_name(molecule_id: DocumentObjectIdV1, name: Option<&str>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetMoleculeName {
        molecule_id,
        name: name.map(str::to_owned),
    })
}

#[test]
fn exact_name_clear_undo_redo_and_reopen_preserve_retained_content() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let identifier = molecule_id(&session, 0, 0);
    let spaced = session
        .apply_document_operation_v1(0, set_name(identifier.clone(), Some("  ")))
        .expect("whitespace name must persist");
    let spaced_cdml = spaced.observation().snapshot().cdml();
    assert!(spaced_cdml.contains("id=\"m\" name=\"  \" role=\"source\""));
    assert!(spaced_cdml.contains("<v:opaque retained=\"yes\"/>"));

    let cleared = session
        .apply_document_operation_v1(1, set_name(identifier, None))
        .expect("empty name intent must clear the attribute");
    assert_eq!(
        cleared.observation().projection().molecules()[0].name(),
        None
    );
    let undone = session.undo(2).expect("clear must be undoable");
    assert_eq!(
        undone.observation().projection().molecules()[0].name(),
        Some("  ")
    );
    let redone = session.redo(3).expect("clear must be redoable");
    assert_eq!(
        redone.observation().projection().molecules()[0].name(),
        None
    );
    let reopened = DocumentSession::load(redone.observation().snapshot().cdml())
        .expect("accepted snapshot must reopen");
    assert_eq!(
        reopened
            .observe(0)
            .expect("reopened state must project")
            .projection()
            .molecules()[1]
            .name(),
        Some("unrelated")
    );
}

#[test]
fn exact_same_name_and_absent_clear_are_history_free() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let identifier = molecule_id(&session, 0, 0);
    let before = session.snapshot().expect("snapshot must work");
    let same = session
        .apply_document_operation_v1(0, set_name(identifier, Some("old")))
        .expect("same name must be accepted");
    assert_eq!(same.observation().snapshot(), &before);

    let unnamed_source = SOURCE.replace(" name=\"old\"", "");
    let mut unnamed = DocumentSession::load(&unnamed_source).expect("unnamed source must load");
    let identifier = molecule_id(&unnamed, 0, 0);
    let before = unnamed.snapshot().expect("snapshot must work");
    let same = unnamed
        .apply_document_operation_v1(0, set_name(identifier, Some("")))
        .expect("absent clear must be accepted");
    assert_eq!(same.observation().snapshot(), &before);
}

#[test]
fn wrong_kind_foreign_and_invalid_name_leave_state_unchanged() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    let atom_id = session
        .observe(0)
        .expect("fixture must project")
        .projection()
        .molecules()[0]
        .atoms()[0]
        .id()
        .expect("atom is durable")
        .clone();
    let foreign = DocumentObjectIdV1::parse(
        "ferrum-document-object-v1/63646d6c2f6d6f6c6563756c65/source/6d697373696e67",
    )
    .expect("test selector is valid");
    for operation in [set_name(atom_id, Some("x")), set_name(foreign, Some("x"))] {
        assert!(matches!(
            session.apply_document_operation_v1(0, operation),
            Err(DocumentSessionError::Operation(
                SessionOperationError::UnknownMolecule
            ))
        ));
        assert_eq!(session.snapshot().expect("snapshot must work"), before);
    }
    let identifier = molecule_id(&session, 0, 0);
    assert!(matches!(
        session.apply_document_operation_v1(0, set_name(identifier, Some("bad\u{0}name"))),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn alternate_canonical_prefix_is_mutated_by_expanded_name() {
    let source = concat!(
        "<c:cdml xmlns:c=\"urn:ferrum:cdml\">",
        "<c:molecule id=\"m\" name=\"before\"><c:atom id=\"a\" name=\"C\">",
        "<c:point x=\"1\" y=\"2\"/><v:opaque xmlns:v=\"urn:vendor\"/>",
        "</c:atom></c:molecule></c:cdml>",
    );
    let mut session = DocumentSession::load(source).expect("source must load");
    let identifier = molecule_id(&session, 0, 0);
    let changed = session
        .apply_document_operation_v1(0, set_name(identifier, Some("after")))
        .expect("prefixed molecule must be editable");
    let cdml = changed.observation().snapshot().cdml();
    assert!(cdml.contains("name=\"after\""));
    assert!(cdml.contains("v:opaque"));
}

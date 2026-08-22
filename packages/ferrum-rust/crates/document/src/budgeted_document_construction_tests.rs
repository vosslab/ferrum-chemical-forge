use super::{
    DocumentObjectIdV1, DocumentSession, Point3V1, TypedDocument, TypedDocumentError,
    XmlBudgetError, XmlInputBudgetV1, XmlInputError,
};

const SOURCE: &str = r#"<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C"><point x="1" y="2"/></atom></molecule></cdml>"#;

fn admitting_budget() -> XmlInputBudgetV1 {
    XmlInputBudgetV1 {
        max_utf8_bytes: SOURCE.len(),
        max_elements: 4,
        max_depth: 4,
        max_attributes: 6,
        max_text_bytes: 0,
    }
}

fn molecule_object_id(session: &DocumentSession, revision: u64) -> DocumentObjectIdV1 {
    session
        .observe(revision)
        .expect("fixture must project")
        .projection()
        .molecules()[0]
        .id()
        .expect("fixture molecule has a durable ID")
        .clone()
}

fn assert_matching_semantics(
    admitted: &DocumentSession,
    unbounded: &DocumentSession,
    revision: u64,
    is_dirty: bool,
) {
    let admitted_snapshot = admitted.snapshot().expect("snapshot works");
    let unbounded_snapshot = unbounded.snapshot().expect("snapshot works");
    let admitted_observation = admitted.observe(revision).expect("observation works");
    let unbounded_observation = unbounded.observe(revision).expect("observation works");

    assert_eq!(admitted_snapshot.revision(), revision);
    assert_eq!(unbounded_snapshot.revision(), revision);
    assert_eq!(admitted_snapshot.is_dirty(), is_dirty);
    assert_eq!(unbounded_snapshot.is_dirty(), is_dirty);
    assert_eq!(
        admitted_observation.projection().molecules(),
        unbounded_observation.projection().molecules()
    );
    assert_eq!(
        admitted_observation.projection().issues(),
        unbounded_observation.projection().issues()
    );
}

#[test]
fn budgeted_parse_and_admitted_session_match_unbounded_load_semantics() {
    let unbounded = DocumentSession::load(SOURCE).expect("fixture must load without a budget");
    let document = TypedDocument::parse_with_budget(SOURCE, admitting_budget())
        .expect("budget admits fixture");
    let admitted = DocumentSession::from_admitted_document(document)
        .expect("admitted document starts session");

    assert_matching_semantics(&admitted, &unbounded, 0, false);
}

#[test]
fn admitted_session_matches_first_generated_commit_and_history_semantics() {
    let mut unbounded = DocumentSession::load(SOURCE).expect("fixture must load without a budget");
    let document = TypedDocument::parse_with_budget(SOURCE, admitting_budget())
        .expect("budget admits fixture");
    let mut admitted = DocumentSession::from_admitted_document(document)
        .expect("admitted document starts session");

    let unbounded_molecule = molecule_object_id(&unbounded, 0);
    let admitted_molecule = molecule_object_id(&admitted, 0);
    assert_eq!(admitted_molecule, unbounded_molecule);

    let position = Point3V1::new(3.0, 4.0, 0.0).expect("test position is finite");
    let mut unbounded_pending = unbounded
        .prepare_create_atom_v1(0, &unbounded_molecule, "O", position)
        .expect("unbounded session prepares a valid atom");
    let mut admitted_pending = admitted
        .prepare_create_atom_v1(0, &admitted_molecule, "O", position)
        .expect("admitted session prepares a valid atom");
    assert_eq!(
        admitted_pending.identifier(),
        unbounded_pending.identifier()
    );
    assert_eq!(admitted_pending.identifier().as_str(), "ferrum-atom-v1-0");

    unbounded
        .commit_create_atom(0, &mut unbounded_pending)
        .expect("unbounded prepared atom commits");
    admitted
        .commit_create_atom(0, &mut admitted_pending)
        .expect("admitted prepared atom commits");
    assert_matching_semantics(&admitted, &unbounded, 1, true);

    unbounded.undo(1).expect("unbounded edit is undoable");
    admitted.undo(1).expect("admitted edit is undoable");
    assert_matching_semantics(&admitted, &unbounded, 2, false);

    unbounded.redo(2).expect("unbounded edit is redoable");
    admitted.redo(2).expect("admitted edit is redoable");
    assert_matching_semantics(&admitted, &unbounded, 3, true);
}

#[test]
fn budgeted_parse_rejects_before_a_session_can_be_constructed() {
    let mut too_small = admitting_budget();
    too_small.max_elements = 3;
    let error = TypedDocument::parse_with_budget(SOURCE, too_small)
        .expect_err("four elements exceed the three-element budget");

    assert!(matches!(
        error,
        TypedDocumentError::XmlInput(XmlInputError::Budget(XmlBudgetError::Elements {
            limit: 3,
            actual: 4,
        }))
    ));
}

#[test]
fn budgeted_parse_preserves_dtd_rejection() {
    let source =
        "<!DOCTYPE cdml [<!ENTITY hostile 'x'>]><cdml xmlns=\"urn:ferrum:cdml\">&hostile;</cdml>";
    let budget = XmlInputBudgetV1 {
        max_utf8_bytes: source.len(),
        max_elements: 1,
        max_depth: 1,
        max_attributes: 0,
        max_text_bytes: 16,
    };

    assert!(matches!(
        TypedDocument::parse_with_budget(source, budget),
        Err(TypedDocumentError::XmlInput(XmlInputError::DtdForbidden))
    ));
}

use super::{
    DocumentSession, TypedDocument, TypedDocumentError, XmlBudgetError, XmlInputBudgetV1,
    XmlInputError,
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
    let [admitted_molecule] = admitted_observation.projection().molecules() else {
        panic!("admitted fixture must project one molecule");
    };
    let [unbounded_molecule] = unbounded_observation.projection().molecules() else {
        panic!("unbounded fixture must project one molecule");
    };
    assert_eq!(admitted_molecule.source_id(), Some("m"));
    assert_eq!(unbounded_molecule.source_id(), Some("m"));
    assert_eq!(admitted_molecule.atoms().len(), 1);
    assert_eq!(unbounded_molecule.atoms().len(), 1);
    assert_eq!(admitted_molecule.atoms()[0].source_id(), Some("a"));
    assert_eq!(unbounded_molecule.atoms()[0].source_id(), Some("a"));
    assert_ne!(admitted_molecule.id(), unbounded_molecule.id());
    assert_ne!(
        admitted_molecule.atoms()[0].id(),
        unbounded_molecule.atoms()[0].id()
    );
    assert!(admitted_observation.projection().issues().is_empty());
    assert!(unbounded_observation.projection().issues().is_empty());
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

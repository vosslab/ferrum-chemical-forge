use crate::{DocumentObjectIdV1, DocumentSession};

use super::{
    DOCUMENT_MOLECULE_INSPECTION_SCHEMA_V1, DocumentMoleculeInspectionErrorV1,
    DocumentMoleculeInspectionRequestV1, inspect_document_molecule_v1,
};

fn source(atom_facts: &str, bond_type: &str) -> String {
    format!(
        concat!(
            "<cdml version=\"1.0\"><molecule id=\"m1\" name=\"Example\">",
            "{}<bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"{}\"/>",
            "</molecule></cdml>"
        ),
        atom_facts, bond_type
    )
}

fn observation_and_request(
    source: &str,
) -> (
    crate::SessionDocumentObservationV1,
    DocumentMoleculeInspectionRequestV1,
) {
    let session = DocumentSession::load(source).expect("source must load");
    let observation = session.observe(0).expect("source must project");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("durable root")
        .clone();
    let request = DocumentMoleculeInspectionRequestV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        molecule_id,
    );
    (observation, request)
}

#[test]
fn inspection_returns_frozen_source_facts_in_lexical_element_order() {
    let atoms = concat!(
        "<atom id=\"a1\" name=\"O\" charge=\"-1\"><point x=\"10\" y=\"20\"/></atom>",
        "<atom id=\"a2\" name=\"C\" charge=\"1\"><point x=\"30\" y=\"5\"/></atom>"
    );
    let (observation, request) = observation_and_request(&source(atoms, "n1"));
    let before = observation.clone();

    let inspection = inspect_document_molecule_v1(&observation, &request).expect("inspect");

    assert_eq!(inspection.schema(), DOCUMENT_MOLECULE_INSPECTION_SCHEMA_V1);
    assert_eq!(inspection.source_revision(), 0);
    assert_eq!(inspection.molecule_id(), request.molecule_id());
    assert_eq!(inspection.source_id(), "m1");
    assert_eq!(inspection.authored_name(), Some("Example"));
    assert_eq!(inspection.atom_count(), 2);
    assert_eq!(inspection.bond_count(), 1);
    assert_eq!(
        inspection
            .element_inventory()
            .iter()
            .map(|entry| (entry.symbol(), entry.atom_count()))
            .collect::<Vec<_>>(),
        vec![("C", 1), ("O", 1)]
    );
    assert_eq!(inspection.total_formal_charge(), Some(0));
    let bounds = inspection.bounds().expect("atoms have bounds");
    assert_eq!(
        (
            bounds.min_x(),
            bounds.min_y(),
            bounds.max_x(),
            bounds.max_y()
        ),
        (10.0, 5.0, 30.0, 20.0)
    );
    assert_eq!(observation, before);
}

#[test]
fn absent_any_authored_charge_remains_unknown_and_drawing_bond_is_allowed() {
    let atoms = concat!(
        "<atom id=\"a1\" name=\"C\" charge=\"1\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"a2\" name=\"O\"><point x=\"1\" y=\"1\"/></atom>"
    );
    let (observation, request) = observation_and_request(&source(atoms, "w1"));

    let inspection =
        inspect_document_molecule_v1(&observation, &request).expect("drawing bond counts");

    assert_eq!(inspection.total_formal_charge(), None);
    assert_eq!(inspection.bond_count(), 1);
}

#[test]
fn stale_revision_and_digest_are_distinct_request_failures() {
    let atoms = concat!(
        "<atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"a2\" name=\"O\"><point x=\"1\" y=\"1\"/></atom>"
    );
    let (observation, request) = observation_and_request(&source(atoms, "n1"));
    let stale = DocumentMoleculeInspectionRequestV1::new(
        1,
        *request.expected_digest(),
        request.molecule_id().clone(),
    );
    assert!(matches!(
        inspect_document_molecule_v1(&observation, &stale),
        Err(DocumentMoleculeInspectionErrorV1::StaleObservation { .. })
    ));
    let digest = [7_u8; 32];
    let wrong_digest =
        DocumentMoleculeInspectionRequestV1::new(0, digest, request.molecule_id().clone());
    assert!(matches!(
        inspect_document_molecule_v1(&observation, &wrong_digest),
        Err(DocumentMoleculeInspectionErrorV1::DigestMismatch)
    ));
}

#[test]
fn atom_selectors_are_not_inspectable_molecule_roots() {
    let source = concat!(
        "<cdml version=\"1.0\"><molecule id=\"m1\">",
        "<atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"a2\" name=\"O\"><point x=\"1\" y=\"1\"/></atom>",
        "<bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"n1\"/>",
        "</molecule></cdml>"
    );
    let (observation, request) = observation_and_request(source);
    let atom_id = DocumentObjectIdV1::parse(
        "ferrum-document-object-v1/6d6f6c6563756c652f61746f6d/source/6131",
    )
    .expect("opaque atom key");
    let atom_request =
        DocumentMoleculeInspectionRequestV1::new(0, *request.expected_digest(), atom_id);
    assert!(matches!(
        inspect_document_molecule_v1(&observation, &atom_request),
        Err(DocumentMoleculeInspectionErrorV1::UnknownDirectMolecule { .. })
    ));
}

#[test]
fn opaque_nested_looking_and_foreign_selectors_are_not_direct_roots() {
    let nested_source = concat!(
        "<cdml version=\"1.0\"><molecule id=\"m1\">",
        "<molecule id=\"nested\">",
        "<atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "</molecule></molecule></cdml>"
    );
    let (observation, request) = observation_and_request(nested_source);
    let before = observation.clone();
    let nested_id = DocumentObjectIdV1::parse(
        "ferrum-document-object-v1/63646d6c2f6d6f6c6563756c65/source/6e6573746564",
    )
    .expect("nested durable molecule key");
    let nested_request =
        DocumentMoleculeInspectionRequestV1::new(0, *request.expected_digest(), nested_id);
    assert!(matches!(
        inspect_document_molecule_v1(&observation, &nested_request),
        Err(DocumentMoleculeInspectionErrorV1::UnknownDirectMolecule { .. })
    ));

    let foreign_source = concat!(
        "<cdml version=\"1.0\"><molecule id=\"foreign\">",
        "<atom id=\"a1\" name=\"O\"><point x=\"0\" y=\"0\"/></atom>",
        "</molecule></cdml>"
    );
    let foreign_session = DocumentSession::load(foreign_source).expect("foreign source loads");
    let foreign_observation = foreign_session.observe(0).expect("foreign source projects");
    let foreign_id = foreign_observation.projection().molecules()[0]
        .id()
        .expect("foreign durable root")
        .clone();
    let foreign_request =
        DocumentMoleculeInspectionRequestV1::new(0, *request.expected_digest(), foreign_id);
    assert!(matches!(
        inspect_document_molecule_v1(&observation, &foreign_request),
        Err(DocumentMoleculeInspectionErrorV1::UnknownDirectMolecule { .. })
    ));
    assert_eq!(observation, before);
}

#[test]
fn missing_and_invalid_elements_remain_typed_source_failures() {
    let missing = concat!(
        "<cdml version=\"1.0\"><molecule id=\"m1\">",
        "<atom id=\"a1\"><point x=\"0\" y=\"0\"/></atom>",
        "</molecule></cdml>"
    );
    let (observation, request) = observation_and_request(missing);
    assert!(matches!(
        inspect_document_molecule_v1(&observation, &request),
        Err(DocumentMoleculeInspectionErrorV1::MissingElement { atom_index: 0 })
    ));

    let invalid = concat!(
        "<cdml version=\"1.0\"><molecule id=\"m1\">",
        "<atom id=\"a1\" name=\"Xx\"><point x=\"0\" y=\"0\"/></atom>",
        "</molecule></cdml>"
    );
    let (observation, request) = observation_and_request(invalid);
    assert!(matches!(
        inspect_document_molecule_v1(&observation, &request),
        Err(DocumentMoleculeInspectionErrorV1::InvalidElement { atom_index: 0, .. })
    ));
}

#[test]
fn empty_and_unresolved_retained_graph_facts_never_make_a_receipt() {
    let empty = "<cdml version=\"1.0\"><molecule id=\"m1\"/></cdml>";
    let (observation, request) = observation_and_request(empty);
    assert!(matches!(
        inspect_document_molecule_v1(&observation, &request),
        Err(DocumentMoleculeInspectionErrorV1::EmptyMolecule)
    ));

    let unresolved = concat!(
        "<cdml version=\"1.0\"><molecule id=\"m1\">",
        "<atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<bond id=\"b1\" start=\"a1\" end=\"missing\" type=\"n1\"/>",
        "</molecule></cdml>"
    );
    let (observation, request) = observation_and_request(unresolved);
    assert!(matches!(
        inspect_document_molecule_v1(&observation, &request),
        Err(DocumentMoleculeInspectionErrorV1::CoreProjection(_))
    ));
}

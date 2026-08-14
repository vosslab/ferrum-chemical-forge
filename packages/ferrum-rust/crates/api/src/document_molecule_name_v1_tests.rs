use ferrum_document::{DocumentSession, DocumentSessionError, SessionOperationError};

use super::{
    DocumentMoleculeInspectionErrorV1, DocumentMoleculeNameErrorV1, DocumentMoleculeNameRequestV1,
    set_document_molecule_name_v1,
};

const SOURCE: &str = concat!(
    "<cdml xmlns:v=\"urn:vendor\" version=\"26.07\">",
    "<molecule id=\"m\" name=\"before\" role=\"source\">",
    "<atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/>",
    "<v:opaque retained=\"yes\"/></atom></molecule>",
    "<molecule id=\"other\"><atom id=\"b\" name=\"O\">",
    "<point x=\"3\" y=\"4\"/></atom></molecule></cdml>",
);

fn request(
    session: &DocumentSession,
    revision: u64,
    root: usize,
    name: &str,
) -> DocumentMoleculeNameRequestV1 {
    let observation = session.observe(revision).expect("fixture must project");
    DocumentMoleculeNameRequestV1::new(
        revision,
        *observation.snapshot().digest(),
        observation.projection().molecules()[root]
            .id()
            .expect("fixture root is durable")
            .clone(),
        name.to_owned(),
    )
}

#[test]
fn authenticated_name_commit_preserves_source_facts_and_returns_one_observation() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let request = request(&session, 0, 0, "Product A");
    let result = set_document_molecule_name_v1(&mut session, request)
        .expect("authenticated name must commit");
    let snapshot = result.observation().snapshot();
    assert_eq!(
        result.observation().projection().molecules()[0].name(),
        Some("Product A")
    );
    assert!(snapshot.cdml().contains("role=\"source\""));
    assert!(snapshot.cdml().contains("<v:opaque retained=\"yes\"/>"));
}

#[test]
fn digest_stale_and_nonroot_selectors_are_atomic() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let observation = session.observe(0).expect("fixture must project");
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("molecule is durable")
        .clone();
    let atom_id = observation.projection().molecules()[0].atoms()[0]
        .id()
        .expect("atom is durable")
        .clone();
    let digest = *observation.snapshot().digest();
    let before = session.snapshot().expect("snapshot must work");
    let wrong_digest = DocumentMoleculeNameRequestV1::new(0, [0_u8; 32], molecule_id, "x".into());
    assert!(matches!(
        set_document_molecule_name_v1(&mut session, wrong_digest),
        Err(DocumentMoleculeNameErrorV1::Observation(
            DocumentMoleculeInspectionErrorV1::DigestMismatch
        ))
    ));
    let nonroot = DocumentMoleculeNameRequestV1::new(0, digest, atom_id, "x".into());
    assert!(matches!(
        set_document_molecule_name_v1(&mut session, nonroot),
        Err(DocumentMoleculeNameErrorV1::Observation(
            DocumentMoleculeInspectionErrorV1::UnknownDirectMolecule { .. }
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);

    let accepted = request(&session, 0, 0, "changed");
    set_document_molecule_name_v1(&mut session, accepted).expect("fixture edit must commit");
    let stale = request_from_observation(&observation, 0, "again");
    assert!(matches!(
        set_document_molecule_name_v1(&mut session, stale),
        Err(DocumentMoleculeNameErrorV1::Session(
            DocumentSessionError::RevisionConflict { .. }
        ))
    ));
}

#[test]
fn invalid_xml_name_is_a_typed_nonmutating_session_failure() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    let invalid = request(&session, 0, 0, "bad\u{0}name");
    assert!(matches!(
        set_document_molecule_name_v1(&mut session, invalid),
        Err(DocumentMoleculeNameErrorV1::Session(
            DocumentSessionError::Operation(SessionOperationError::Candidate(_))
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

fn request_from_observation(
    observation: &ferrum_document::SessionDocumentObservationV1,
    root: usize,
    name: &str,
) -> DocumentMoleculeNameRequestV1 {
    DocumentMoleculeNameRequestV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        observation.projection().molecules()[root]
            .id()
            .expect("fixture root is durable")
            .clone(),
        name.to_owned(),
    )
}

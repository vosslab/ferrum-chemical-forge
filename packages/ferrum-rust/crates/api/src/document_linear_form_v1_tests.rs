use ferrum_document::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, PersistentId, SessionOperationError,
};

use super::{
    DocumentLinearFormErrorV1, DocumentLinearFormRequestV1, DocumentLinearFormResultV1,
    convert_document_linear_form_v1,
};

const SOURCE: &str = concat!(
    "<cdml version=\"1.0\"><molecule id=\"m\">",
    "<atom id=\"late\" name=\"C\"><point x=\"40\" y=\"5\"/></atom>",
    "<atom id=\"early\" name=\"O\"><point x=\"10\" y=\"5\"/></atom>",
    "<bond id=\"b\" start=\"late\" end=\"early\" type=\"n1\"/>",
    "</molecule></cdml>",
);

fn atom(id: &str) -> PersistentId {
    PersistentId::new(id).expect("fixture atom ID is valid")
}

fn request(
    session: &DocumentSession,
    revision: u64,
    root: DocumentObjectIdV1,
    selected_atom_ids: Vec<PersistentId>,
) -> DocumentLinearFormRequestV1 {
    let observation = session.observe(revision).expect("fixture must observe");
    DocumentLinearFormRequestV1::new(
        revision,
        *observation.snapshot().digest(),
        root,
        selected_atom_ids,
    )
}

fn root(session: &DocumentSession, revision: u64) -> DocumentObjectIdV1 {
    session
        .observe(revision)
        .expect("fixture must observe")
        .projection()
        .molecules()[0]
        .id()
        .expect("fixture root is durable")
        .clone()
}

#[test]
fn changed_result_has_authoritative_provenance_and_source_ordered_conversion() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture must load");
    let conversion = request(
        &session,
        0,
        root(&session, 0),
        vec![atom("late"), atom("early")],
    );
    let result = convert_document_linear_form_v1(&mut session, conversion)
        .expect("authenticated path must convert");

    let DocumentLinearFormResultV1::Changed(result) = result else {
        panic!("first conversion must change the source");
    };
    assert_eq!(result.observation().snapshot().revision(), 1);
    assert_eq!(
        result.observation().snapshot(),
        session
            .snapshot()
            .as_ref()
            .expect("session has current snapshot")
    );
    let cdml = result.observation().snapshot().cdml();
    assert!(
        cdml.contains("id=\"late\" name=\"C\" show_hydrogens=\"on\"><point x=\"40\" y=\"5\""),
        "{cdml}"
    );
    assert!(
        cdml.contains("id=\"early\" name=\"O\" show_hydrogens=\"on\"><point x=\"1.764cm\" y=\"5\""),
        "{cdml}"
    );
    assert!(cdml.contains("type=\"linear_form\""));
}

#[test]
fn canonical_repeat_returns_no_change_without_advancing_revision() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture must load");
    let conversion = request(
        &session,
        0,
        root(&session, 0),
        vec![atom("late"), atom("early")],
    );
    let first = convert_document_linear_form_v1(&mut session, conversion)
        .expect("fixture must convert")
        .into_operation_result();
    let repeated = DocumentLinearFormRequestV1::new(
        first.observation().snapshot().revision(),
        *first.observation().snapshot().digest(),
        root(&session, 1),
        vec![atom("late"), atom("early")],
    );

    let result = convert_document_linear_form_v1(&mut session, repeated)
        .expect("canonical source must classify as no change");
    assert!(matches!(result, DocumentLinearFormResultV1::NoChange(_)));
    assert_eq!(
        session.snapshot().expect("snapshot must work").revision(),
        1
    );
}

#[test]
fn stale_digest_and_nonroot_requests_are_atomic() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture must load");
    let before = session.snapshot().expect("snapshot must work");
    let root_id = root(&session, 0);
    let stale = DocumentLinearFormRequestV1::new(1, [0; 32], root_id.clone(), vec![atom("late")]);
    assert!(matches!(
        convert_document_linear_form_v1(&mut session, stale),
        Err(DocumentLinearFormErrorV1::Session(
            DocumentSessionError::RevisionConflict { .. }
        ))
    ));
    let wrong_digest = DocumentLinearFormRequestV1::new(0, [0; 32], root_id, vec![atom("late")]);
    assert!(matches!(
        convert_document_linear_form_v1(&mut session, wrong_digest),
        Err(DocumentLinearFormErrorV1::Observation(
            super::DocumentMoleculeInspectionErrorV1::DigestMismatch
        ))
    ));
    let nonroot = DocumentLinearFormRequestV1::new(
        0,
        *before.digest(),
        DocumentObjectIdV1::parse("ferrum-document-object-v1/61746f6d/source/6c617465")
            .expect("closed atom selector"),
        vec![atom("late")],
    );
    assert!(matches!(
        convert_document_linear_form_v1(&mut session, nonroot),
        Err(DocumentLinearFormErrorV1::Observation(
            super::DocumentMoleculeInspectionErrorV1::UnknownDirectMolecule { .. }
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn invalid_selection_and_session_refusal_are_atomic() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture must load");
    let before = session.snapshot().expect("snapshot must work");
    for selected in [
        vec![],
        vec![atom("late"), atom("late")],
        vec![atom("foreign")],
    ] {
        let conversion = request(&session, 0, root(&session, 0), selected);
        let error = convert_document_linear_form_v1(&mut session, conversion)
            .expect_err("invalid selection must be refused");
        assert!(
            matches!(
                error,
                DocumentLinearFormErrorV1::Session(DocumentSessionError::Operation(
                    SessionOperationError::EmptyLinearFormSelection
                        | SessionOperationError::LinearFormPlan(_)
                        | SessionOperationError::Candidate(_)
                ))
            ),
            "{error:?}"
        );
        assert_eq!(session.snapshot().expect("snapshot must work"), before);
    }
}

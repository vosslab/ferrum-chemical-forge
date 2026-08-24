use std::f64::consts::FRAC_PI_2;

use crate::{
    AtomRotationTargetV1, AtomRotationV1, AtomRotationV1Error, DocumentSession,
    DocumentSessionError, SessionOperation, SessionOperationError, SessionOperationV1,
    TypedDocumentError,
};

const HALF_AUTHORED_UNIT_POINTS: f64 = (0.001 * 72.0 / 2.54) / 2.0;

fn target(molecule: &str, atom: &str) -> AtomRotationTargetV1 {
    AtomRotationTargetV1::new(molecule, atom).expect("fixture target")
}

fn operation(rotation: AtomRotationV1) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::RotateAtoms { rotation })
}

fn assert_authored_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() <= HALF_AUTHORED_UNIT_POINTS);
}

#[test]
fn selected_atoms_rotate_in_one_history_entry_and_retire_only_invalid_owned_metadata() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"C\"><point x=\"10\" y=\"0\" z=\"2\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
        "<fragment id=\"owned\" type=\"linear_form\"><name>linear_form</name>",
        "<bond id=\"ab\"/><vertex id=\"a\"/><vertex id=\"b\"/>",
        "<property name=\"bond_length\" value=\"10\" type=\"IntType\"/></fragment>",
        "<fragment id=\"richer\" type=\"linear_form\" retained=\"yes\"><extension/>",
        "</fragment></molecule></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let rotation = AtomRotationV1::new(
        vec![target("m", "a"), target("m", "b")],
        0.0,
        0.0,
        FRAC_PI_2,
    )
    .expect("fixture rotation");
    let rotated = session
        .apply_document_operation_v1(0, operation(rotation))
        .expect("rotation succeeds");
    let atoms = rotated.observation().projection().molecules()[0].atoms();
    assert_authored_close(atoms[0].position().x(), 0.0);
    assert_authored_close(atoms[0].position().y(), 0.0);
    assert_authored_close(atoms[1].position().x(), 0.0);
    assert_authored_close(atoms[1].position().y(), 10.0);
    assert_eq!(atoms[1].position().z(), 2.0);
    let cdml = rotated.observation().snapshot().cdml();
    assert!(!cdml.contains("id=\"owned\""));
    assert!(cdml.contains("id=\"richer\""));
    assert!(cdml.contains("retained=\"yes\""));

    let undone = session.undo(1).expect("rotation is one history entry");
    assert_eq!(
        undone.observation().projection().molecules()[0].atoms()[1]
            .position()
            .x(),
        10.0
    );
    let zero = AtomRotationV1::new(vec![target("m", "a")], 0.0, 0.0, 0.0)
        .expect("zero rotation intent is valid");
    let unchanged = session
        .apply_document_operation_v1(2, operation(zero))
        .expect("zero rotation is accepted");
    assert_eq!(unchanged.observation().snapshot().revision(), 2);
}

#[test]
fn atom_rotation_rejects_invalid_or_unresolved_complete_intent_atomically() {
    let duplicate = vec![target("m", "a"), target("m", "a")];
    assert_eq!(
        AtomRotationV1::new(duplicate, 0.0, 0.0, 1.0),
        Err(AtomRotationV1Error::DuplicateTarget)
    );
    assert_eq!(
        AtomRotationV1::new(vec![target("m", "a")], f64::NAN, 0.0, 1.0),
        Err(AtomRotationV1Error::NonFiniteCenter)
    );

    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
        "<point x=\"1\" y=\"2\"/></atom></molecule></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let before = session.snapshot().expect("snapshot");
    let rotation = AtomRotationV1::new(vec![target("m", "a"), target("other", "a")], 0.0, 0.0, 1.0)
        .expect("structurally valid unresolved request");
    let error = session
        .apply_document_operation_v1(0, operation(rotation))
        .expect_err("later unresolved target rejects the whole request");
    assert!(matches!(
        error,
        DocumentSessionError::Operation(SessionOperationError::Candidate(
            TypedDocumentError::UnknownAtomRotationTarget { .. }
        ))
    ));
    let after = session.snapshot().expect("snapshot");
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.digest(), before.digest());
}

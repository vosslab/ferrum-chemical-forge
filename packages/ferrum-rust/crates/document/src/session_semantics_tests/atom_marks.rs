use super::{DocumentSession, DocumentSessionError, SessionOperationError};
use crate::{
    AtomMarkActionV1, AtomMarkKindV1, SessionOperation, SessionOperationV1, TypedDocumentError,
};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"1cm\" y=\"2cm\"/>",
    "<opaque retained=\"yes\"/></atom></molecule></cdml>",
);

fn operation(
    action: AtomMarkActionV1,
    kind: AtomMarkKindV1,
    matching_mark_index: Option<u32>,
) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::ApplyAtomMark {
        molecule_id: "m".to_owned(),
        atom_id: "a".to_owned(),
        action,
        kind,
        matching_mark_index,
    })
}

#[test]
fn add_and_remove_charge_mark_are_one_atomic_semantic_edit() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let added = session
        .submit(
            0,
            operation(AtomMarkActionV1::Add, AtomMarkKindV1::Plus, None),
        )
        .expect("plus mark must be accepted");
    let atom = &added.observation().projection().molecules()[0].atoms()[0];
    let [mark] = atom.marks() else {
        panic!("one supported mark must be projected");
    };
    assert_eq!(
        (atom.formal_charge(), mark.kind()),
        (Some(1), AtomMarkKindV1::Plus)
    );
    assert_eq!(mark.same_type_ordinal(), 0);
    // Each authored axis is rounded to 0.001 cm. Two half-quanta produce at
    // most sqrt(2) * 0.0005 cm of radial error from the requested 12 points.
    let radial_error_bound = f64::sqrt(2.0) * 0.0005 * 72.0 / 2.54;
    assert!((mark.radial_offset() - 12.0).abs() <= radial_error_bound);
    assert!(
        added
            .observation()
            .snapshot()
            .cdml()
            .contains("<opaque retained=\"yes\"/>")
    );

    let removed = session
        .submit(
            1,
            operation(AtomMarkActionV1::Remove, AtomMarkKindV1::Plus, None),
        )
        .expect("plus removal must be accepted");
    let atom = &removed.observation().projection().molecules()[0].atoms()[0];
    assert_eq!((atom.formal_charge(), atom.marks().len()), (None, 0));
    let undone = session.undo(2).expect("removal must be undoable");
    assert_eq!(
        undone.observation().projection().molecules()[0].atoms()[0]
            .marks()
            .len(),
        1,
    );
}

#[test]
fn same_type_ordinal_removes_only_the_selected_duplicate() {
    let source = SOURCE
        .replace(
            "<opaque retained=\"yes\"/>",
            concat!(
                "<mark type=\"radical\" x=\"1cm\" y=\"2.4cm\" size=\"4\"/>",
                "<opaque retained=\"yes\"/>",
                "<mark type=\"radical\" x=\"1cm\" y=\"2.5cm\" size=\"4\"/>",
            ),
        )
        .replace("name=\"C\"", "name=\"C\" multiplicity=\"3\"");
    let mut session = DocumentSession::load(&source).expect("source must load");
    let removed = session
        .submit(
            0,
            operation(AtomMarkActionV1::Remove, AtomMarkKindV1::Radical, Some(1)),
        )
        .expect("selected duplicate must be removed");
    let atom = &removed.observation().projection().molecules()[0].atoms()[0];
    assert_eq!(atom.multiplicity(), Some(2));
    assert_eq!(atom.marks().len(), 1);
    assert_eq!(atom.marks()[0].same_type_ordinal(), 0);
    assert!(
        removed
            .observation()
            .snapshot()
            .cdml()
            .contains("y=\"2.4cm\"")
    );
    assert!(
        !removed
            .observation()
            .snapshot()
            .cdml()
            .contains("y=\"2.5cm\"")
    );
}

#[test]
fn missing_remove_is_history_free_but_bad_selector_is_atomic() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    let unchanged = session
        .submit(
            0,
            operation(AtomMarkActionV1::Remove, AtomMarkKindV1::Electronpair, None),
        )
        .expect("missing unselected removal is a successful no-op");
    assert_eq!(unchanged.observation().snapshot(), &before);
    assert!(matches!(
        session.submit(
            0,
            operation(
                AtomMarkActionV1::Remove,
                AtomMarkKindV1::Electronpair,
                Some(0),
            ),
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::AtomMarkIndexOutOfRange { .. })
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn invalid_geometry_and_scalar_bounds_never_change_authoritative_state() {
    for (source, kind) in [
        (
            SOURCE.replace("<point x=\"1cm\" y=\"2cm\"/>", "<point x=\"1cm\"/>"),
            AtomMarkKindV1::PzOrbital,
        ),
        (
            SOURCE.replace("name=\"C\"", "name=\"C\" charge=\"9\""),
            AtomMarkKindV1::Plus,
        ),
    ] {
        let mut session = DocumentSession::load(&source).expect("source must load");
        let before = session.snapshot().expect("snapshot must work");
        assert!(matches!(
            session.submit(0, operation(AtomMarkActionV1::Add, kind, None)),
            Err(DocumentSessionError::Operation(
                SessionOperationError::Candidate(
                    TypedDocumentError::InvalidAtomMarkPoint(_)
                        | TypedDocumentError::AtomMarkScalarOutOfRange { .. }
                )
            ))
        ));
        assert_eq!(session.snapshot().expect("snapshot must work"), before);
    }
}

#[test]
fn charge_mark_preserves_unrelated_legacy_multiplicity_and_returns_an_observation() {
    let source = SOURCE.replace("name=\"C\"", "name=\"C\" multiplicity=\"legacy\"");
    let mut session = DocumentSession::load(&source).expect("source must load");
    let added = session
        .submit(
            0,
            operation(AtomMarkActionV1::Add, AtomMarkKindV1::Plus, None),
        )
        .expect("plus addresses charge without normalizing multiplicity");
    let observation = added.observation();
    let atom = &observation.projection().molecules()[0].atoms()[0];

    assert_eq!((atom.formal_charge(), atom.multiplicity()), (Some(1), None));
    assert_eq!(atom.marks()[0].kind(), AtomMarkKindV1::Plus);
    assert!(
        observation
            .snapshot()
            .cdml()
            .contains("multiplicity=\"legacy\"")
    );
    assert!(observation.projection().issues().iter().any(|issue| {
        issue.code() == crate::ProjectionIssueCodeV1::InvalidPresentationFact
            && issue.detail().contains("multiplicity")
    }));
}

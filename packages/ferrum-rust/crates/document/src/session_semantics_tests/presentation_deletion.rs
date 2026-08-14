//! Atomic durable direct-root presentation deletion behavior.

use super::{DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError};
use crate::{
    PresentationRecordKindV1, PresentationRootDeletionSetV1, PresentationRootDeletionV1,
    PresentationRootProjectionV1, SessionOperationV1, TypedDocumentError,
};

const SOURCE: &str = concat!(
    "<c:cdml xmlns:c=\"http://www.freesoftware.fsf.org/bkchem/cdml\" ",
    "xmlns:v=\"urn:vendor\"><c:text id=\"t\" keep=\"yes\"><c:point x=\"1\" y=\"2\"/>",
    "<c:ftext>label</c:ftext><v:inside/></c:text><v:opaque retained-id=\"t\"/>",
    "<c:plus id=\"p\"><c:point x=\"3\" y=\"4\"/></c:plus><v:tail/></c:cdml>",
);

const BRACKET_SOURCE: &str = concat!(
    "<cdml><polyline id=\"left\" bracket_pair=\"left\" bracket_side=\"left\" spline=\"yes\">",
    "<point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/><point x=\"1\" y=\"2\"/>",
    "<point x=\"0\" y=\"3\"/></polyline><polyline id=\"right\" bracket_pair=\"left\" ",
    "bracket_side=\"right\" spline=\"yes\"><point x=\"4\" y=\"0\"/>",
    "<point x=\"3\" y=\"1\"/><point x=\"3\" y=\"2\"/><point x=\"4\" y=\"3\"/>",
    "</polyline></cdml>",
);

fn deletion(identifier: &str, kind: PresentationRecordKindV1) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::DeletePresentationRoot {
        deletion: PresentationRootDeletionV1::new(identifier, kind).unwrap(),
    })
}

fn deletion_set(targets: Vec<PresentationRootDeletionV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::DeletePresentationRoots {
        deletions: PresentationRootDeletionSetV1::new(targets).unwrap(),
    })
}

#[test]
fn presentation_deletion_removes_exact_typed_root_preserves_opaque_content_and_follows_history() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session
        .submit(0, deletion("t", PresentationRecordKindV1::Text))
        .expect("typed Text deletion must commit");
    assert_eq!(changed.observation().snapshot().revision(), 1);
    let [PresentationRootProjectionV1::Plus { plus }] = changed
        .observation()
        .projection()
        .presentation_stack()
        .roots()
    else {
        panic!("only the retained Plus should remain projected");
    };
    assert_eq!(plus.target().source_id(), Some("p"));
    let cdml = changed.observation().snapshot().cdml();
    assert!(!cdml.contains("<c:text"));
    assert!(cdml.contains("<v:opaque retained-id=\"t\""));
    assert!(cdml.contains("<v:tail"));

    let undone = session.undo(1).expect("deletion must undo");
    assert!(
        undone
            .observation()
            .projection()
            .presentation_stack()
            .roots()
            .iter()
            .any(|root| matches!(root, PresentationRootProjectionV1::Text { .. }))
    );
    let redone = session.redo(2).expect("deletion must redo");
    assert!(
        redone
            .observation()
            .projection()
            .presentation_stack()
            .roots()
            .iter()
            .all(|root| !matches!(root, PresentationRootProjectionV1::Text { .. }))
    );
}

#[test]
fn presentation_deletion_rejects_wrong_kind_bracket_member_and_stale_intent_atomically() {
    let mut wrong_kind = DocumentSession::load(SOURCE).expect("source must load");
    let before = wrong_kind.snapshot().unwrap();
    assert!(matches!(
        wrong_kind.submit(0, deletion("t", PresentationRecordKindV1::Plus)),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownPresentationRoot(_)
        ))
    ));
    assert_eq!(wrong_kind.snapshot().unwrap(), before);

    let mut bracket = DocumentSession::load(BRACKET_SOURCE).expect("bracket source must load");
    let before = bracket.snapshot().unwrap();
    assert!(matches!(
        bracket.submit(0, deletion("left", PresentationRecordKindV1::Polyline)),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::PresentationRootIsBracketMember(
                _
            ))
        ))
    ));
    assert_eq!(bracket.snapshot().unwrap(), before);
    assert!(matches!(
        bracket.submit(
            0,
            deletion_set(vec![
                PresentationRootDeletionV1::new("left", PresentationRecordKindV1::Polyline,)
                    .unwrap(),
            ]),
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::PartialBracketDeletion(_))
        ))
    ));
    assert_eq!(bracket.snapshot().unwrap(), before);

    let mut stale = DocumentSession::load(SOURCE).expect("source must load");
    stale
        .submit(0, deletion("p", PresentationRecordKindV1::Plus))
        .expect("first deletion must commit");
    let before = stale.snapshot().unwrap();
    assert!(matches!(
        stale.submit(0, deletion("t", PresentationRecordKindV1::Text)),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(stale.snapshot().unwrap(), before);
}

#[test]
fn complete_bracket_pair_deletion_is_one_atomic_history_entry() {
    assert!(PresentationRootDeletionSetV1::new(Vec::new()).is_err());
    assert!(
        PresentationRootDeletionSetV1::new(vec![
            PresentationRootDeletionV1::new("left", PresentationRecordKindV1::Polyline).unwrap(),
            PresentationRootDeletionV1::new("left", PresentationRecordKindV1::Polyline).unwrap(),
        ])
        .is_err()
    );
    let mut session = DocumentSession::load(BRACKET_SOURCE).expect("bracket source must load");
    let changed = session
        .submit(
            0,
            deletion_set(vec![
                PresentationRootDeletionV1::new("left", PresentationRecordKindV1::Polyline)
                    .unwrap(),
                PresentationRootDeletionV1::new("right", PresentationRecordKindV1::Polyline)
                    .unwrap(),
            ]),
        )
        .expect("complete bracket pair deletes");
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert!(
        changed
            .observation()
            .projection()
            .presentation_stack()
            .roots()
            .is_empty()
    );
    let restored = session.undo(1).expect("pair deletion undoes");
    assert_eq!(restored.observation().snapshot().revision(), 2);
    let [pair] = restored
        .observation()
        .projection()
        .presentation_stack()
        .bracket_pairs()
    else {
        panic!("undo must restore the authoritative bracket pair");
    };
    assert_eq!(pair.member_ids(), &["left".to_owned(), "right".to_owned()]);
}

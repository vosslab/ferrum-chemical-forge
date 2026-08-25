//! Atomic durable direct-root presentation deletion behavior.

use super::{DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError};
use crate::{
    DocumentObjectIdV1, PresentationRecordKindV1, PresentationRootDeletionSetV1,
    PresentationRootDeletionV1, PresentationRootProjectionV1, SessionOperationV1,
    TypedDocumentError,
};

const SOURCE: &str = concat!(
    "<c:cdml xmlns:c=\"urn:ferrum:cdml\" ",
    "xmlns:v=\"urn:vendor\"><c:text id=\"t\" keep=\"yes\"><c:point x=\"1\" y=\"2\"/>",
    "<c:ftext>label</c:ftext><v:inside/></c:text><v:opaque retained-id=\"t\"/>",
    "<c:plus id=\"p\"><c:point x=\"3\" y=\"4\"/></c:plus><v:tail/></c:cdml>",
);

const BRACKET_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><polyline id=\"left\" bracket_pair=\"left\" bracket_side=\"left\" spline=\"yes\">",
    "<point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/><point x=\"1\" y=\"2\"/>",
    "<point x=\"0\" y=\"3\"/></polyline><polyline id=\"right\" bracket_pair=\"left\" ",
    "bracket_side=\"right\" spline=\"yes\"><point x=\"4\" y=\"0\"/>",
    "<point x=\"3\" y=\"1\"/><point x=\"3\" y=\"2\"/><point x=\"4\" y=\"3\"/>",
    "</polyline></cdml>",
);

const REACTION_PRESENTATION_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"a\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"0\"/></arrow>",
    "<text id=\"t\"><point x=\"0\" y=\"10\"/><ftext>conditions</ftext></text>",
    "<plus id=\"p\"><point x=\"20\" y=\"0\"/></plus>",
    "<arrow id=\"free-a\"><point x=\"0\" y=\"20\"/><point x=\"10\" y=\"20\"/></arrow>",
    "<text id=\"free-t\"><point x=\"0\" y=\"30\"/><ftext>free</ftext></text>",
    "<plus id=\"free-p\"><point x=\"20\" y=\"20\"/></plus>",
    "<reaction id=\"r\"><arrow idref=\"a\"/><condition idref=\"t\"/><plus idref=\"p\"/></reaction></cdml>"
);

fn deletion(
    document_object_id: DocumentObjectIdV1,
    kind: PresentationRecordKindV1,
) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::DeletePresentationRoot {
        deletion: PresentationRootDeletionV1::new(document_object_id, kind),
    })
}

fn deletion_set(targets: Vec<PresentationRootDeletionV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::DeletePresentationRoots {
        deletions: PresentationRootDeletionSetV1::new(targets)
            .expect("test deletion targets must be nonempty and unique"),
    })
}

fn presentation_root_id(
    session: &DocumentSession,
    kind: PresentationRecordKindV1,
    occurrence: usize,
) -> DocumentObjectIdV1 {
    session
        .observe(0)
        .expect("fixture must project")
        .projection()
        .presentation_stack()
        .entries()
        .iter()
        .filter(|entry| entry.root().target().record_kind() == kind)
        .nth(occurrence)
        .map(|entry| entry.root().target().document_object_id().clone())
        .expect("fixture must project the requested durable presentation root")
}

fn bracket_member_ids(session: &DocumentSession) -> [DocumentObjectIdV1; 2] {
    let observation = session.observe(0).expect("fixture must project");
    let [pair] = observation
        .projection()
        .presentation_stack()
        .bracket_pairs()
    else {
        panic!("fixture must project one bracket pair");
    };
    let [left, right] = pair.members();
    [left.clone(), right.clone()]
}

fn invalid_document_object_id() -> DocumentObjectIdV1 {
    DocumentObjectIdV1::parse("ferrum-document-object-v1/00000000000000000000000000000000")
        .expect("fixed opaque test selector is syntactically durable")
}

#[test]
fn presentation_deletion_removes_exact_typed_root_preserves_opaque_content_and_follows_history() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let text_id = presentation_root_id(&session, PresentationRecordKindV1::Text, 0);
    let plus_id = presentation_root_id(&session, PresentationRecordKindV1::Plus, 0);
    let changed = session
        .apply_document_operation_v1(0, deletion(text_id, PresentationRecordKindV1::Text))
        .expect("typed Text deletion must commit");
    assert_eq!(changed.observation().snapshot().revision(), 1);
    let [entry] = changed
        .observation()
        .projection()
        .presentation_stack()
        .entries()
    else {
        panic!("only the retained Plus should remain projected");
    };
    let PresentationRootProjectionV1::Plus { plus } = entry.root() else {
        panic!("only the retained Plus should remain projected");
    };
    assert_eq!(plus.target().document_object_id(), &plus_id);
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
            .entries()
            .iter()
            .any(|entry| matches!(entry.root(), PresentationRootProjectionV1::Text { .. }))
    );
    let redone = session.redo(2).expect("deletion must redo");
    assert!(
        redone
            .observation()
            .projection()
            .presentation_stack()
            .entries()
            .iter()
            .all(|entry| !matches!(entry.root(), PresentationRootProjectionV1::Text { .. }))
    );
}

#[test]
fn presentation_deletion_rejects_wrong_kind_bracket_member_and_stale_intent_atomically() {
    let mut wrong_kind = DocumentSession::load(SOURCE).expect("source must load");
    let text_id = presentation_root_id(&wrong_kind, PresentationRecordKindV1::Text, 0);
    let before = wrong_kind.snapshot().unwrap();
    assert!(matches!(
        wrong_kind
            .apply_document_operation_v1(0, deletion(text_id, PresentationRecordKindV1::Plus)),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownPresentationRoot(_)
        ))
    ));
    assert_eq!(wrong_kind.snapshot().unwrap(), before);

    let mut bracket = DocumentSession::load(BRACKET_SOURCE).expect("bracket source must load");
    let [left_id, _] = bracket_member_ids(&bracket);
    let before = bracket.snapshot().unwrap();
    assert!(matches!(
        bracket.apply_document_operation_v1(
            0,
            deletion(left_id.clone(), PresentationRecordKindV1::Polyline)
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::PresentationRootIsBracketMember(
                _
            ))
        ))
    ));
    assert_eq!(bracket.snapshot().unwrap(), before);
    assert!(matches!(
        bracket.apply_document_operation_v1(
            0,
            deletion_set(vec![PresentationRootDeletionV1::new(
                left_id,
                PresentationRecordKindV1::Polyline
            ),]),
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::PartialBracketDeletion(_))
        ))
    ));
    assert_eq!(bracket.snapshot().unwrap(), before);

    let mut stale = DocumentSession::load(SOURCE).expect("source must load");
    let plus_id = presentation_root_id(&stale, PresentationRecordKindV1::Plus, 0);
    let text_id = presentation_root_id(&stale, PresentationRecordKindV1::Text, 0);
    stale
        .apply_document_operation_v1(0, deletion(plus_id, PresentationRecordKindV1::Plus))
        .expect("first deletion must commit");
    let before = stale.snapshot().unwrap();
    assert!(matches!(
        stale.apply_document_operation_v1(0, deletion(text_id, PresentationRecordKindV1::Text)),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(stale.snapshot().unwrap(), before);
}

#[test]
fn compatibility_reaction_references_refuse_single_and_batch_deletion_without_mutation() {
    let protected = [
        PresentationRecordKindV1::Arrow,
        PresentationRecordKindV1::Text,
        PresentationRecordKindV1::Plus,
    ];
    for kind in protected {
        let mut session =
            DocumentSession::load(REACTION_PRESENTATION_SOURCE).expect("fixture loads");
        let document_object_id = presentation_root_id(&session, kind, 0);
        let before = session.snapshot().expect("snapshot works");
        assert!(matches!(
            session.apply_document_operation_v1(0, deletion(document_object_id, kind)),
            Err(DocumentSessionError::Operation(
                SessionOperationError::Candidate(
                    TypedDocumentError::ReactionReferencedPresentationDeletion(_)
                )
            ))
        ));
        assert_eq!(session.snapshot().expect("snapshot works"), before);
    }

    let mut mixed = DocumentSession::load(REACTION_PRESENTATION_SOURCE).expect("fixture loads");
    let referenced_text_id = presentation_root_id(&mixed, PresentationRecordKindV1::Text, 0);
    let free_arrow_id = presentation_root_id(&mixed, PresentationRecordKindV1::Arrow, 1);
    let free_text_id = presentation_root_id(&mixed, PresentationRecordKindV1::Text, 1);
    let free_plus_id = presentation_root_id(&mixed, PresentationRecordKindV1::Plus, 1);
    let before = mixed.snapshot().expect("snapshot works");
    assert!(matches!(
        mixed.apply_document_operation_v1(
            0,
            deletion_set(vec![
                PresentationRootDeletionV1::new(
                    free_arrow_id.clone(),
                    PresentationRecordKindV1::Arrow
                ),
                PresentationRootDeletionV1::new(referenced_text_id, PresentationRecordKindV1::Text),
                PresentationRootDeletionV1::new(
                    free_plus_id.clone(),
                    PresentationRecordKindV1::Plus
                ),
            ]),
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(
                TypedDocumentError::ReactionReferencedPresentationDeletion(_)
            )
        ))
    ));
    assert_eq!(mixed.snapshot().expect("snapshot works"), before);

    let changed = mixed
        .apply_document_operation_v1(
            0,
            deletion_set(vec![
                PresentationRootDeletionV1::new(free_arrow_id, PresentationRecordKindV1::Arrow),
                PresentationRootDeletionV1::new(free_text_id, PresentationRecordKindV1::Text),
                PresentationRootDeletionV1::new(free_plus_id, PresentationRecordKindV1::Plus),
            ]),
        )
        .expect("unreferenced multi-delete commits after rejected batch");
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert!(changed.observation().snapshot().cdml().contains("id=\"a\""));
    assert!(
        !changed
            .observation()
            .snapshot()
            .cdml()
            .contains("id=\"free-a\"")
    );
}

#[test]
fn complete_bracket_pair_deletion_is_one_atomic_history_entry() {
    assert!(PresentationRootDeletionSetV1::new(Vec::new()).is_err());
    let invalid_id = invalid_document_object_id();
    assert!(
        PresentationRootDeletionSetV1::new(vec![
            PresentationRootDeletionV1::new(invalid_id.clone(), PresentationRecordKindV1::Polyline),
            PresentationRootDeletionV1::new(invalid_id, PresentationRecordKindV1::Polyline),
        ])
        .is_err()
    );
    let mut session = DocumentSession::load(BRACKET_SOURCE).expect("bracket source must load");
    let [left_id, right_id] = bracket_member_ids(&session);
    let changed = session
        .apply_document_operation_v1(
            0,
            deletion_set(vec![
                PresentationRootDeletionV1::new(
                    left_id.clone(),
                    PresentationRecordKindV1::Polyline,
                ),
                PresentationRootDeletionV1::new(
                    right_id.clone(),
                    PresentationRecordKindV1::Polyline,
                ),
            ]),
        )
        .expect("complete bracket pair deletes");
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert!(
        changed
            .observation()
            .projection()
            .presentation_stack()
            .entries()
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
    assert_eq!(pair.members(), &[left_id, right_id]);
}

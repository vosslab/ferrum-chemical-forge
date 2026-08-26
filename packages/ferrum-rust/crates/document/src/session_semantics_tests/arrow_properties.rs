//! Atomic durable direct-root Arrow properties behavior.

use super::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1, TypedDocumentError,
};
use crate::{
    ArrowLineWidthV1, ArrowPropertiesPatchV1, ArrowPropertiesPatchV1Error, ArrowPropertyChangeV1,
    DocumentObjectIdV1, PresentationRootProjectionV1, Rgb24V1,
};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\"><arrow id=\"a\" type=\"normal\" start=\"false\" ",
    "end=\"true\" spline=\"0\" width=\"1px\" color=\"#000\" keep=\"yes\">",
    "<point x=\"0\" y=\"0\"/><v:opaque/><point x=\"40\" y=\"0\"/>",
    "</arrow><v:root/></cdml>"
);

fn patch(session: &DocumentSession, changes: Vec<ArrowPropertyChangeV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetArrowProperties {
        patch: ArrowPropertiesPatchV1::new(arrow_object_id(session), changes)
            .expect("valid Arrow patch"),
    })
}

fn test_object_id() -> DocumentObjectIdV1 {
    DocumentObjectIdV1::from_entropy_bytes([0; 16])
}

fn arrow_object_id(session: &DocumentSession) -> DocumentObjectIdV1 {
    let revision = session.snapshot().expect("snapshot").revision();
    let observation = session.observe(revision).expect("observation");
    observation
        .projection()
        .presentation_stack()
        .entries()
        .iter()
        .find_map(|entry| match entry.root() {
            PresentationRootProjectionV1::Arrow { .. } => {
                Some(entry.root().target().document_object_id().clone())
            }
            _ => None,
        })
        .expect("expected direct-root Arrow")
}

fn arrow(observation: &crate::SessionDocumentObservationV1) -> &crate::ArrowProjectionV1 {
    observation
        .projection()
        .presentation_stack()
        .entries()
        .iter()
        .find_map(|entry| match entry.root() {
            PresentationRootProjectionV1::Arrow { arrow } => Some(arrow),
            _ => None,
        })
        .expect("expected direct-root Arrow")
}

fn normal_head_flags(observation: &crate::SessionDocumentObservationV1) -> (bool, bool) {
    let crate::ArrowProjectionKindV1::Normal {
        start_head,
        end_head,
        ..
    } = arrow(observation).kind()
    else {
        panic!("normal Arrow properties require normal semantic policy");
    };
    (*start_head, *end_head)
}

#[test]
fn arrow_properties_commit_once_preserve_extensions_and_follow_history() {
    let changes = vec![
        ArrowPropertyChangeV1::StartHead(true),
        ArrowPropertyChangeV1::EndHead(false),
        ArrowPropertyChangeV1::Spline(false),
        ArrowPropertyChangeV1::LineWidth(ArrowLineWidthV1::new(2.5).unwrap()),
        ArrowPropertyChangeV1::Color(Rgb24V1::new("#AbC").unwrap()),
    ];
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session
        .apply_document_operation_v1(0, patch(&session, changes))
        .expect("patch must commit");
    let projected = arrow(changed.observation());
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert_eq!(normal_head_flags(changed.observation()), (true, false));
    assert_eq!(projected.stroke().width().value(), 2.5);
    assert_eq!(projected.stroke().color().as_str(), "#aabbcc");
    let cdml = changed.observation().snapshot().cdml();
    assert!(cdml.contains("keep=\"yes\""));
    assert!(cdml.contains("<v:opaque"));
    assert!(cdml.contains("<v:root"));

    let undone = session.undo(1).expect("one patch must undo once");
    assert!(!normal_head_flags(undone.observation()).0);
    let redone = session.redo(2).expect("one patch must redo once");
    assert!(normal_head_flags(redone.observation()).0);
}

#[test]
fn arrow_properties_compare_historical_spellings_without_normalizing_them() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let result = session
        .apply_document_operation_v1(
            0,
            patch(
                &session,
                vec![
                    ArrowPropertyChangeV1::StartHead(false),
                    ArrowPropertyChangeV1::EndHead(true),
                    ArrowPropertyChangeV1::Spline(false),
                    ArrowPropertyChangeV1::LineWidth(ArrowLineWidthV1::new(1.0).unwrap()),
                    ArrowPropertyChangeV1::Color(Rgb24V1::new("#000000").unwrap()),
                ],
            ),
        )
        .expect("semantic equal patch must be accepted");
    assert_eq!(result.observation().snapshot().revision(), 0);
    assert!(
        result
            .observation()
            .snapshot()
            .cdml()
            .contains("start=\"false\"")
    );
    assert!(
        result
            .observation()
            .snapshot()
            .cdml()
            .contains("width=\"1px\"")
    );
}

#[test]
fn arrow_properties_reject_invalid_intent_structure_target_and_stale_revision() {
    assert_eq!(
        ArrowPropertiesPatchV1::new(
            test_object_id(),
            vec![
                ArrowPropertyChangeV1::StartHead(true),
                ArrowPropertyChangeV1::StartHead(false),
            ],
        ),
        Err(ArrowPropertiesPatchV1Error::DuplicateChange {
            property: "start head"
        })
    );
    assert!(ArrowLineWidthV1::new(0.09).is_none());
    assert!(ArrowLineWidthV1::new(20.1).is_none());

    let malformed = SOURCE.replace("<v:opaque/>", "<bogus/><v:opaque/>");
    let mut malformed = DocumentSession::load(&malformed).expect("retained source loads");
    let before = malformed.snapshot().expect("snapshot");
    assert!(matches!(
        malformed.apply_document_operation_v1(
            0,
            patch(&malformed, vec![ArrowPropertyChangeV1::StartHead(true)])
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::InvalidArrowStructure(_))
        ))
    ));
    assert_eq!(malformed.snapshot().expect("snapshot"), before);

    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot");
    let unknown = ArrowPropertiesPatchV1::new(
        test_object_id(),
        vec![ArrowPropertyChangeV1::StartHead(true)],
    )
    .unwrap();
    assert!(matches!(
        session.apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::SetArrowProperties { patch: unknown })
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownArrow(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot"), before);

    let cross_kind_source = SOURCE.replace(
        "<v:root/>",
        "<text id=\"t\"><point x=\"0\" y=\"0\"/><ftext>x</ftext></text><v:root/>",
    );
    let mut cross_kind = DocumentSession::load(&cross_kind_source).expect("source loads");
    let arrow_id = arrow_object_id(&cross_kind);
    let revision = cross_kind.snapshot().expect("snapshot").revision();
    let foreign_id = cross_kind
        .observe(revision)
        .expect("observation")
        .projection()
        .presentation_stack()
        .entries()
        .iter()
        .find_map(|entry| match entry.root() {
            PresentationRootProjectionV1::Text { .. } => {
                Some(entry.root().target().document_object_id().clone())
            }
            _ => None,
        })
        .expect("Text root has a durable ID");
    assert_ne!(foreign_id, arrow_id);
    let foreign =
        ArrowPropertiesPatchV1::new(foreign_id, vec![ArrowPropertyChangeV1::StartHead(true)])
            .expect("valid cross-kind patch");
    let before = cross_kind.snapshot().expect("snapshot");
    assert!(matches!(
        cross_kind.apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::SetArrowProperties { patch: foreign })
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownArrow(_)
        ))
    ));
    assert_eq!(cross_kind.snapshot().expect("snapshot"), before);
    session
        .apply_document_operation_v1(
            0,
            patch(&session, vec![ArrowPropertyChangeV1::StartHead(true)]),
        )
        .expect("initial patch");
    let before = session.snapshot().expect("snapshot");
    assert!(matches!(
        session.apply_document_operation_v1(
            0,
            patch(&session, vec![ArrowPropertyChangeV1::EndHead(false)])
        ),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(session.snapshot().expect("snapshot"), before);
}

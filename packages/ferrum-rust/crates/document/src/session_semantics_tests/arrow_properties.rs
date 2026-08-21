//! Atomic durable direct-root Arrow properties behavior.

use super::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1, TypedDocumentError,
};
use crate::{
    ArrowLineWidthV1, ArrowPropertiesPatchV1, ArrowPropertiesPatchV1Error, ArrowPropertyChangeV1,
    PresentationRootProjectionV1, Rgb24V1,
};

const SOURCE: &str = concat!(
    "<cdml xmlns:v=\"urn:vendor\"><arrow id=\"a\" type=\"normal\" start=\"false\" ",
    "end=\"true\" spline=\"0\" width=\"1px\" color=\"#000\" keep=\"yes\">",
    "<point x=\"0\" y=\"0\"/><v:opaque/><point x=\"40\" y=\"0\"/>",
    "</arrow><v:root/></cdml>"
);

fn patch(changes: Vec<ArrowPropertyChangeV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetArrowProperties {
        patch: ArrowPropertiesPatchV1::new("a", changes).expect("valid Arrow patch"),
    })
}

fn arrow(observation: &crate::SessionDocumentObservationV1) -> &crate::ArrowProjectionV1 {
    let [PresentationRootProjectionV1::Arrow { arrow }] =
        observation.projection().presentation_stack().roots()
    else {
        panic!("expected one normal Arrow");
    };
    arrow
}

fn normal_head_flags(observation: &crate::SessionDocumentObservationV1) -> (bool, bool) {
    let crate::ArrowDisplayGeometryV1::Normal {
        start_head,
        end_head,
        ..
    } = arrow(observation).geometry()
    else {
        panic!("normal Arrow properties require normal display geometry");
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
        .submit(0, patch(changes))
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
    assert_eq!(normal_head_flags(undone.observation()).0, false);
    let redone = session.redo(2).expect("one patch must redo once");
    assert_eq!(normal_head_flags(redone.observation()).0, true);
}

#[test]
fn arrow_properties_compare_historical_spellings_without_normalizing_them() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let result = session
        .submit(
            0,
            patch(vec![
                ArrowPropertyChangeV1::StartHead(false),
                ArrowPropertyChangeV1::EndHead(true),
                ArrowPropertyChangeV1::Spline(false),
                ArrowPropertyChangeV1::LineWidth(ArrowLineWidthV1::new(1.0).unwrap()),
                ArrowPropertyChangeV1::Color(Rgb24V1::new("#000000").unwrap()),
            ]),
        )
        .expect("semantic equal patch must be accepted");
    assert_eq!(result.observation().snapshot().revision(), 0);
    assert!(result
        .observation()
        .snapshot()
        .cdml()
        .contains("start=\"false\""));
    assert!(result
        .observation()
        .snapshot()
        .cdml()
        .contains("width=\"1px\""));
}

#[test]
fn arrow_properties_reject_invalid_intent_structure_target_and_stale_revision() {
    assert_eq!(
        ArrowPropertiesPatchV1::new(
            "a",
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
        malformed.submit(0, patch(vec![ArrowPropertyChangeV1::StartHead(true)])),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::InvalidArrowStructure(_))
        ))
    ));
    assert_eq!(malformed.snapshot().expect("snapshot"), before);

    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let unknown =
        ArrowPropertiesPatchV1::new("missing", vec![ArrowPropertyChangeV1::StartHead(true)])
            .unwrap();
    assert!(matches!(
        session.submit(
            0,
            SessionOperation::V1(SessionOperationV1::SetArrowProperties { patch: unknown })
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownArrow(_)
        ))
    ));
    session
        .submit(0, patch(vec![ArrowPropertyChangeV1::StartHead(true)]))
        .expect("initial patch");
    let before = session.snapshot().expect("snapshot");
    assert!(matches!(
        session.submit(0, patch(vec![ArrowPropertyChangeV1::EndHead(false)])),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(session.snapshot().expect("snapshot"), before);
}

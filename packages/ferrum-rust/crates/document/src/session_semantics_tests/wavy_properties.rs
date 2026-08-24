//! Atomic durable Wavy appearance behavior.

use super::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1,
};
use crate::{
    GeometricLineWidthV1, PresentationRecordKindV1, PresentationRootProjectionV1, Rgb24V1,
    WavyPropertiesPatchV1, WavyPropertiesPatchV1Error, WavyPropertyChangeV1,
};

const SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\"><standard line_color=\"#123456\" line_width=\"1\"/>",
    "<polyline id=\"wave\" style=\"wavy\" color=\"#ABC\" keep=\"yes\">",
    "<point x=\"0\" y=\"0\"/><point x=\"3\" y=\"2\"/>",
    "<point x=\"6\" y=\"0\"/><v:opaque/><!--keep--></polyline>",
    "<polyline id=\"ordinary\"><point x=\"0\" y=\"0\"/>",
    "<point x=\"1\" y=\"1\"/></polyline></cdml>"
);

fn patch(identifier: &str, changes: Vec<WavyPropertyChangeV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetWavyProperties {
        patch: WavyPropertiesPatchV1::new(identifier, changes).expect("valid Wavy patch"),
    })
}

#[test]
fn authored_wavy_path_and_appearance_commit_preserve_and_follow_history() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.observe(0).expect("source observation");
    let [PresentationRootProjectionV1::Wavy { polyline }, ..] =
        before.projection().presentation_stack().roots()
    else {
        panic!("expected a distinct Wavy projection root");
    };
    assert_eq!(
        polyline.target().record_kind(),
        PresentationRecordKindV1::Polyline
    );
    assert_eq!(polyline.stroke().color().as_str(), "#aabbcc");
    assert_eq!(
        polyline
            .path()
            .points()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect::<Vec<_>>(),
        vec![(0.0, 0.0), (3.0, 2.0), (6.0, 0.0)]
    );

    let changed = session
        .apply_document_operation_v1(
            0,
            patch(
                "wave",
                vec![
                    WavyPropertyChangeV1::LineWidth(GeometricLineWidthV1::new(2.5).unwrap()),
                    WavyPropertyChangeV1::LineColor(Rgb24V1::new("#445566").unwrap()),
                ],
            ),
        )
        .expect("Wavy patch must commit");
    let [PresentationRootProjectionV1::Wavy { polyline }, ..] = changed
        .observation()
        .projection()
        .presentation_stack()
        .roots()
    else {
        panic!("expected updated Wavy root");
    };
    assert_eq!(polyline.stroke().width().value(), 2.5);
    assert_eq!(polyline.stroke().color().as_str(), "#445566");
    let cdml = changed.observation().snapshot().cdml();
    assert!(cdml.contains("keep=\"yes\""));
    assert!(cdml.contains("v:opaque"));
    assert_eq!(
        session
            .undo(1)
            .expect("undo")
            .observation()
            .snapshot()
            .revision(),
        2
    );
    assert_eq!(
        session
            .redo(2)
            .expect("redo")
            .observation()
            .snapshot()
            .revision(),
        3
    );
}

#[test]
fn duplicate_unknown_ordinary_and_stale_wavy_intent_are_atomic() {
    assert_eq!(
        WavyPropertiesPatchV1::new(
            "wave",
            vec![
                WavyPropertyChangeV1::LineColor(Rgb24V1::new("#010203").unwrap()),
                WavyPropertyChangeV1::LineColor(Rgb24V1::new("#040506").unwrap()),
            ],
        ),
        Err(WavyPropertiesPatchV1Error::DuplicateChange {
            property: "line color"
        })
    );
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot");
    for identifier in ["ordinary", "missing"] {
        assert!(matches!(
            session.apply_document_operation_v1(
                0,
                patch(
                    identifier,
                    vec![WavyPropertyChangeV1::LineColor(
                        Rgb24V1::new("#010203").unwrap()
                    )]
                )
            ),
            Err(DocumentSessionError::Operation(
                SessionOperationError::UnknownWavy(_)
            ))
        ));
        assert_eq!(session.snapshot().expect("snapshot"), before);
    }
    session
        .apply_document_operation_v1(
            0,
            patch(
                "wave",
                vec![WavyPropertyChangeV1::LineColor(
                    Rgb24V1::new("#010203").unwrap(),
                )],
            ),
        )
        .expect("valid patch");
    let accepted = session.snapshot().expect("snapshot");
    assert!(matches!(
        session.apply_document_operation_v1(0, patch("wave", Vec::new())),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(session.snapshot().expect("snapshot"), accepted);
}

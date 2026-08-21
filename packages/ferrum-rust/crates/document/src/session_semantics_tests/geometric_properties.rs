//! Atomic durable geometric presentation appearance behavior.

use super::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1, TypedDocumentError,
};
use crate::{
    GeometricLineWidthV1, GeometricPropertiesPatchV1, GeometricPropertiesPatchV1Error,
    GeometricPropertyChangeV1, PresentationRootProjectionV1, Rgb24V1,
};

const SOURCE: &str = concat!(
    "<cdml xmlns:v=\"urn:vendor\"><standard line_color=\"#123456\" ",
    "area_color=\"#abcdef\" line_width=\"1\"/>",
    "<rect id=\"shape\" x1=\"1\" y1=\"2\" x2=\"3\" y2=\"4\" ",
    "color=\"#ABC\" background-color=\"#dEf\" keep=\"yes\">",
    "<v:opaque/><!--keep--><?keep value?></rect>",
    "<polyline id=\"line\" color=\"#246\"><point x=\"0\" y=\"0\"/>",
    "<point x=\"5\" y=\"6\"/></polyline>",
    "<polyline id=\"wave\" style=\"wavy\"><point x=\"0\" y=\"0\"/>",
    "<point x=\"2\" y=\"2\"/></polyline></cdml>"
);

fn patch(identifier: &str, changes: Vec<GeometricPropertyChangeV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetGeometricProperties {
        patch: GeometricPropertiesPatchV1::new(identifier, changes)
            .expect("valid geometric properties patch"),
    })
}

#[test]
fn closed_shape_appearance_commits_once_preserves_content_and_follows_history() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session
        .submit(
            0,
            patch(
                "shape",
                vec![
                    GeometricPropertyChangeV1::LineWidth(GeometricLineWidthV1::new(2.5).unwrap()),
                    GeometricPropertyChangeV1::StrokeColor(Rgb24V1::new("#445566").unwrap()),
                    GeometricPropertyChangeV1::FillColor(None),
                ],
            ),
        )
        .expect("shape patch must commit");
    let [PresentationRootProjectionV1::Rectangle { shape }, ..] = changed
        .observation()
        .projection()
        .presentation_stack()
        .roots()
    else {
        panic!("expected the rectangle first");
    };
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert_eq!(shape.stroke().width().value(), 2.5);
    assert_eq!(shape.stroke().color().as_str(), "#445566");
    assert!(shape.fill().color().is_none());
    let cdml = changed.observation().snapshot().cdml();
    assert!(cdml.contains("keep=\"yes\""));
    assert!(cdml.contains("v:opaque"));
    assert!(cdml.contains("background-color=\"#dEf\""));

    let undone = session.undo(1).expect("one patch must undo once");
    let [PresentationRootProjectionV1::Rectangle { shape }, ..] = undone
        .observation()
        .projection()
        .presentation_stack()
        .roots()
    else {
        panic!("expected restored rectangle");
    };
    assert_eq!(shape.stroke().color().as_str(), "#aabbcc");
    assert_eq!(shape.fill().color().unwrap().as_str(), "#ddeeff");
    session.redo(2).expect("one patch must redo once");
}

#[test]
fn legacy_color_and_fill_spellings_compare_semantically_without_rewriting() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let result = session
        .submit(
            0,
            patch(
                "shape",
                vec![
                    GeometricPropertyChangeV1::LineWidth(GeometricLineWidthV1::new(1.0).unwrap()),
                    GeometricPropertyChangeV1::StrokeColor(Rgb24V1::new("#aabbcc").unwrap()),
                    GeometricPropertyChangeV1::FillColor(Some(Rgb24V1::new("#ddeeff").unwrap())),
                ],
            ),
        )
        .expect("semantic equal patch must be accepted");
    assert_eq!(result.observation().snapshot().revision(), 0);
    assert!(result
        .observation()
        .snapshot()
        .cdml()
        .contains("color=\"#ABC\""));
    assert!(result
        .observation()
        .snapshot()
        .cdml()
        .contains("background-color=\"#dEf\""));
}

#[test]
fn invalid_inapplicable_specialized_unknown_and_stale_intent_are_atomic() {
    assert_eq!(
        GeometricPropertiesPatchV1::new(
            "shape",
            vec![
                GeometricPropertyChangeV1::FillColor(None),
                GeometricPropertyChangeV1::FillColor(None),
            ],
        ),
        Err(GeometricPropertiesPatchV1Error::DuplicateChange {
            property: "fill color"
        })
    );
    assert!(GeometricLineWidthV1::new(0.09).is_none());

    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot");
    for (identifier, change, expected) in [
        (
            "line",
            GeometricPropertyChangeV1::FillColor(None),
            "inapplicable",
        ),
        (
            "wave",
            GeometricPropertyChangeV1::LineWidth(GeometricLineWidthV1::new(2.0).unwrap()),
            "specialized",
        ),
    ] {
        let error = session
            .submit(0, patch(identifier, vec![change]))
            .expect_err("invalid intent must fail");
        match (expected, error) {
            (
                "inapplicable",
                DocumentSessionError::Operation(SessionOperationError::Candidate(
                    TypedDocumentError::InapplicableGeometricProperty(_),
                )),
            )
            | (
                "specialized",
                DocumentSessionError::Operation(SessionOperationError::Candidate(
                    TypedDocumentError::SpecializedGeometricTarget(_),
                )),
            ) => {}
            (_, other) => panic!("unexpected error: {other}"),
        }
        assert_eq!(session.snapshot().expect("snapshot"), before);
    }

    assert!(matches!(
        session.submit(
            0,
            patch(
                "missing",
                vec![GeometricPropertyChangeV1::StrokeColor(
                    Rgb24V1::new("#010203").unwrap()
                )]
            )
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownGeometricPresentation(_)
        ))
    ));
    session
        .submit(
            0,
            patch(
                "line",
                vec![GeometricPropertyChangeV1::StrokeColor(
                    Rgb24V1::new("#010203").unwrap(),
                )],
            ),
        )
        .expect("ordinary polyline stroke must commit");
    let accepted = session.snapshot().expect("snapshot");
    assert!(matches!(
        session.submit(
            0,
            patch("shape", vec![GeometricPropertyChangeV1::FillColor(None)])
        ),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(session.snapshot().expect("snapshot"), accepted);
}

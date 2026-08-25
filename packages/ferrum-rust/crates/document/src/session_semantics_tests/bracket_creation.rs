use super::super::{
    BracketPropertiesPatchV1, BracketPropertiesPatchV1Error, BracketPropertyChangeV1,
    BracketStyleV1, DocumentObjectIdV1, DocumentSession, DocumentSessionError,
    GeometricLineWidthV1, PresentationRootProjectionV1, Rgb24V1, SessionOperation,
    SessionOperationV1,
};
use ferrum_document_projection::PresentationBracketStyleV1;

fn properties(
    members: [DocumentObjectIdV1; 2],
    changes: Vec<BracketPropertyChangeV1>,
) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetBracketProperties {
        patch: BracketPropertiesPatchV1::new(members, changes)
            .expect("valid bracket-properties patch"),
    })
}

#[test]
fn rectangular_bracket_creation_owns_pair_identity_geometry_standard_and_history() {
    let source = concat!(
        "<c:cdml xmlns:c=\"urn:ferrum:cdml\" ",
        "xmlns:v=\"urn:vendor\"><c:standard line_width=\"2\" ",
        "line_color=\"#123\"/><v:opaque id=\"ferrum-presentation-v1-0\">",
        "<v:keep/></v:opaque></c:cdml>",
    );
    let mut session = DocumentSession::load(source).expect("source must load");
    let mut pending = session
        .prepare_create_bracket_v1(0, BracketStyleV1::Rectangular, 0.0, 10.0, 100.0, 210.0)
        .expect("finite normalized rectangle must prepare");
    assert_eq!(
        pending.pair_identifier().as_str(),
        "ferrum-presentation-v1-1"
    );
    assert_eq!(pending.left_identifier(), pending.pair_identifier());
    assert_eq!(
        pending.right_identifier().as_str(),
        "ferrum-presentation-v1-2"
    );
    assert_eq!(session.snapshot().expect("snapshot").revision(), 0);

    let result = session
        .commit_create_bracket(0, &mut pending)
        .expect("prepared pair must commit");
    let stack = result.observation().projection().presentation_stack();
    let [left_entry, right_entry] = stack.entries() else {
        panic!("expected two rectangular bracket sides");
    };
    let PresentationRootProjectionV1::Polyline { polyline: left } = left_entry.root() else {
        panic!("expected left rectangular bracket side");
    };
    let PresentationRootProjectionV1::Polyline { polyline: right } = right_entry.root() else {
        panic!("expected right rectangular bracket side");
    };
    let [pair] = stack.bracket_pairs() else {
        panic!("expected one exact bracket relationship");
    };
    assert_eq!(
        pair.members(),
        &[
            left_entry.root().target().document_object_id().clone(),
            right_entry.root().target().document_object_id().clone(),
        ]
    );
    assert_eq!(pair.style(), PresentationBracketStyleV1::Rectangular);
    assert_eq!(pair.line_width().unwrap().value(), 2.0);
    assert_eq!(pair.line_color().unwrap().as_str(), "#112233");
    assert_eq!(
        result
            .observation()
            .projection()
            .direct_roots()
            .iter()
            .map(|root| root.document_object_id())
            .collect::<Vec<_>>(),
        vec![
            left_entry.root().target().document_object_id(),
            right_entry.root().target().document_object_id(),
        ]
    );
    let inset = 0.05 * 100.0_f64.hypot(200.0);
    assert_eq!(
        left.path()
            .points()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect::<Vec<_>>(),
        vec![(inset, 10.0), (0.0, 10.0), (0.0, 210.0), (inset, 210.0)]
    );
    assert_eq!(right.stroke().width().value(), 2.0);
    assert!(stack.issues().is_empty());
    let cdml = result.observation().snapshot().cdml();
    assert!(cdml.contains("bracket_pair=\"ferrum-presentation-v1-1\""));
    assert!(cdml.contains("bracket_side=\"left\""));
    assert!(cdml.contains("urn:vendor"));
    assert!(cdml.contains("keep"));
    session.undo(1).expect("pair creation must undo atomically");
    assert!(
        session
            .observe(2)
            .expect("undo observation")
            .projection()
            .presentation_stack()
            .bracket_pairs()
            .is_empty()
    );
    session.redo(2).expect("pair creation must redo atomically");
    assert_eq!(
        session
            .observe(3)
            .expect("redo observation")
            .projection()
            .presentation_stack()
            .bracket_pairs()
            .len(),
        1
    );
}

#[test]
fn round_projection_is_explicit_and_invalid_or_stale_requests_do_not_mutate() {
    let mut session =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("source must load");
    let before = session.snapshot().expect("snapshot");
    for bounds in [
        (0.0, 0.0, f64::INFINITY, 10.0),
        (1.0, 0.0, 1.0, 10.0),
        (2.0, 0.0, 1.0, 10.0),
    ] {
        assert!(
            session
                .prepare_create_bracket_v1(
                    0,
                    BracketStyleV1::Rectangular,
                    bounds.0,
                    bounds.1,
                    bounds.2,
                    bounds.3,
                )
                .is_err()
        );
    }
    assert_eq!(session.snapshot().expect("snapshot"), before);

    let mut pending = session
        .prepare_create_bracket_v1(0, BracketStyleV1::Round, 0.0, 0.0, 80.0, 120.0)
        .expect("round pair geometry must prepare");
    assert_eq!(
        pending.left_identifier().as_str(),
        "ferrum-presentation-v1-0"
    );
    assert!(matches!(
        session.commit_create_bracket(1, &mut pending),
        Err(DocumentSessionError::RevisionConflict {
            expected: 1,
            actual: 0
        })
    ));
    let result = session
        .commit_create_bracket(0, &mut pending)
        .expect("pending pair remains usable after a stale caller revision");
    let stack = result.observation().projection().presentation_stack();
    let [pair] = stack.bracket_pairs() else {
        panic!("expected one observed round relationship");
    };
    assert_eq!(pair.style(), PresentationBracketStyleV1::Round);
    let [left_entry, right_entry] = stack.entries() else {
        panic!("expected two explicit round bracket spline sides");
    };
    let PresentationRootProjectionV1::RoundBracket { polyline: left } = left_entry.root() else {
        panic!("expected left explicit round bracket spline side");
    };
    let PresentationRootProjectionV1::RoundBracket { polyline: right } = right_entry.root() else {
        panic!("expected right explicit round bracket spline side");
    };
    assert_eq!(left.path().points().len(), 4);
    assert_eq!(right.path().points().len(), 4);
    assert!(stack.issues().is_empty());
}

#[test]
fn common_bracket_appearance_commits_once_preserves_content_and_follows_history() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\"><polyline id=\"left\" bracket_pair=\"left\" ",
        "bracket_side=\"left\" spline=\"no\" width=\"1.0\" color=\"#ABC\" keep=\"yes\">",
        "<point x=\"1\" y=\"0\"/><point x=\"0\" y=\"0\"/>",
        "<point x=\"0\" y=\"10\"/><point x=\"1\" y=\"10\"/><v:opaque/>",
        "</polyline><polyline id=\"right\" bracket_pair=\"left\" bracket_side=\"right\" ",
        "spline=\"no\" width=\"1\" line_color=\"#aabbcc\">",
        "<point x=\"9\" y=\"0\"/><point x=\"10\" y=\"0\"/>",
        "<point x=\"10\" y=\"10\"/><point x=\"9\" y=\"10\"/>",
        "</polyline></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("source must load");
    let members = {
        let observation = session.observe(0).expect("observation");
        let [pair] = observation
            .projection()
            .presentation_stack()
            .bracket_pairs()
        else {
            panic!("expected one observed bracket relationship");
        };
        pair.members().clone()
    };
    let changed = session
        .apply_document_operation_v1(
            0,
            properties(
                members.clone(),
                vec![
                    BracketPropertyChangeV1::LineWidth(GeometricLineWidthV1::new(2.5).unwrap()),
                    BracketPropertyChangeV1::LineColor(Rgb24V1::new("#445566").unwrap()),
                ],
            ),
        )
        .expect("common appearance patch must commit");
    let [pair] = changed
        .observation()
        .projection()
        .presentation_stack()
        .bracket_pairs()
    else {
        panic!("expected one retained bracket pair");
    };
    assert_eq!(pair.line_width().unwrap().value(), 2.5);
    assert_eq!(pair.line_color().unwrap().as_str(), "#445566");
    let cdml = changed.observation().snapshot().cdml();
    assert_eq!(cdml.matches("width=\"2.5\"").count(), 2);
    assert_eq!(cdml.matches("line_color=\"#445566\"").count(), 2);
    assert!(cdml.contains("keep=\"yes\""));
    assert!(cdml.contains("v:opaque"));

    let no_change = session
        .apply_document_operation_v1(
            1,
            properties(
                members,
                vec![BracketPropertyChangeV1::LineWidth(
                    GeometricLineWidthV1::new(2.5).unwrap(),
                )],
            ),
        )
        .expect("semantic equality must be accepted without history");
    assert_eq!(no_change.observation().snapshot().revision(), 1);
    session
        .undo(1)
        .expect("pair appearance must undo atomically");
    session
        .redo(2)
        .expect("pair appearance must redo atomically");
    assert_eq!(session.snapshot().expect("snapshot").revision(), 3);
}

#[test]
fn duplicate_malformed_unknown_and_stale_bracket_properties_are_atomic() {
    assert_eq!(
        BracketPropertiesPatchV1::new(
            [
                DocumentObjectIdV1::from_entropy_bytes([0x10; 16]),
                DocumentObjectIdV1::from_entropy_bytes([0x20; 16]),
            ],
            vec![
                BracketPropertyChangeV1::LineColor(Rgb24V1::new("#010203").unwrap()),
                BracketPropertyChangeV1::LineColor(Rgb24V1::new("#040506").unwrap()),
            ],
        ),
        Err(BracketPropertiesPatchV1Error::DuplicateChange {
            property: "line color"
        })
    );
    let malformed = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><polyline id=\"left\" bracket_pair=\"left\" bracket_side=\"left\" ",
        "spline=\"no\"><point x=\"0\" y=\"0\"/></polyline>",
        "<polyline id=\"right\" bracket_pair=\"left\" bracket_side=\"right\" ",
        "spline=\"no\"><point x=\"1\" y=\"0\"/></polyline></cdml>",
    );
    let mut session = DocumentSession::load(malformed).expect("source must load");
    let before = session.snapshot().expect("snapshot");
    for members in [
        [
            DocumentObjectIdV1::from_entropy_bytes([0x10; 16]),
            DocumentObjectIdV1::from_entropy_bytes([0x20; 16]),
        ],
        [
            DocumentObjectIdV1::from_entropy_bytes([0x30; 16]),
            DocumentObjectIdV1::from_entropy_bytes([0x40; 16]),
        ],
    ] {
        assert!(
            session
                .apply_document_operation_v1(
                    0,
                    properties(
                        members,
                        vec![BracketPropertyChangeV1::LineColor(
                            Rgb24V1::new("#010203").unwrap(),
                        )],
                    ),
                )
                .is_err()
        );
        assert_eq!(session.snapshot().expect("snapshot"), before);
    }

    let mut valid =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("source must load");
    let mut pending = valid
        .prepare_create_bracket_v1(0, BracketStyleV1::Rectangular, 0.0, 0.0, 10.0, 10.0)
        .expect("pair must prepare");
    valid
        .commit_create_bracket(0, &mut pending)
        .expect("pair must commit");
    let members = {
        let observation = valid.observe(1).expect("observation");
        let [pair] = observation
            .projection()
            .presentation_stack()
            .bracket_pairs()
        else {
            panic!("expected one observed bracket relationship");
        };
        pair.members().clone()
    };
    let accepted = valid.snapshot().expect("snapshot");
    assert!(matches!(
        valid.apply_document_operation_v1(
            0,
            properties(
                members,
                vec![BracketPropertyChangeV1::LineColor(
                    Rgb24V1::new("#010203").unwrap(),
                )],
            ),
        ),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(valid.snapshot().expect("snapshot"), accepted);
}

#[test]
fn bracket_properties_reject_members_from_different_authoritative_pairs() {
    let mut session =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("source must load");
    let mut first = session
        .prepare_create_bracket_v1(0, BracketStyleV1::Rectangular, 0.0, 0.0, 10.0, 10.0)
        .expect("first pair must prepare");
    session
        .commit_create_bracket(0, &mut first)
        .expect("first pair must commit");
    let mut second = session
        .prepare_create_bracket_v1(1, BracketStyleV1::Round, 20.0, 0.0, 30.0, 10.0)
        .expect("second pair must prepare");
    session
        .commit_create_bracket(1, &mut second)
        .expect("second pair must commit");

    let foreign_members = {
        let observation = session.observe(2).expect("observation");
        let [first_pair, second_pair] = observation
            .projection()
            .presentation_stack()
            .bracket_pairs()
        else {
            panic!("expected two authoritative bracket pairs");
        };
        [
            first_pair.members()[0].clone(),
            second_pair.members()[1].clone(),
        ]
    };
    let before = session.snapshot().expect("snapshot");

    assert!(
        session
            .apply_document_operation_v1(
                2,
                properties(
                    foreign_members,
                    vec![BracketPropertyChangeV1::LineColor(
                        Rgb24V1::new("#010203").unwrap(),
                    )],
                ),
            )
            .is_err()
    );
    assert_eq!(session.snapshot().expect("snapshot"), before);
}

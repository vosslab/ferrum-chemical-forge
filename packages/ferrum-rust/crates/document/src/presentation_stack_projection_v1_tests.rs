use super::{
    DocumentSession, PRESENTATION_STACK_PROJECTION_SCHEMA_V1, PresentationFactProvenanceV1,
    PresentationProjectionIssueCodeV1, PresentationRootProjectionV1, PresentationTextStyleV1,
};

fn observed(source: &str) -> super::SessionDocumentObservationV1 {
    let session = DocumentSession::load(source).unwrap();
    session.observe(0).unwrap()
}

#[test]
fn drawing_only_document_projects_one_direct_root_multisegment_polyline() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><polyline id=\"line\" spline=\"no\" line_color=\"#AbC\" width=\"2px\"><point x=\"1cm\" y=\"2\"/><point x=\"3\" y=\"4\"/><point x=\"5\" y=\"6\"/><point x=\"7\" y=\"8\"/></polyline></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    assert_eq!(stack.schema(), PRESENTATION_STACK_PROJECTION_SCHEMA_V1);
    assert_eq!(stack.revision(), observation.snapshot().revision());
    assert_eq!(stack.digest(), observation.snapshot().digest());
    let [PresentationRootProjectionV1::Polyline { polyline }] = stack.roots() else {
        panic!("expected one polyline");
    };
    assert_eq!(polyline.target().source_id(), Some("line"));
    assert!(polyline.target().id().is_some());
    let [first, second, third, last] = polyline.path().points() else {
        panic!("expected every authored polyline point");
    };
    assert_eq!(first.x(), 72.0 / 2.54);
    assert_eq!((second.x(), third.x(), last.x()), (3.0, 5.0, 7.0));
    assert_eq!(polyline.stroke().color().as_str(), "#aabbcc");
    assert_eq!(polyline.stroke().width().value(), 2.0);
    assert_eq!(
        polyline.stroke().color_provenance(),
        PresentationFactProvenanceV1::Root
    );
    assert_eq!(
        polyline.stroke().width_provenance(),
        PresentationFactProvenanceV1::Root
    );
    assert!(stack.issues().is_empty());
}

#[test]
fn normal_arrow_projects_authored_head_policy_without_display_geometry() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"a\" type=\"normal\" start=\"no\" end=\"yes\" spline=\"no\" width=\"2\" color=\"#123456\" shape=\"(8,10,3)\"><point x=\"0\" y=\"0\"/><point x=\"40\" y=\"0\"/></arrow></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    let [PresentationRootProjectionV1::Arrow { arrow }] = stack.roots() else {
        panic!("expected one supported normal arrow");
    };
    assert_eq!(arrow.target().source_id(), Some("a"));
    assert_eq!(arrow.source_path().points()[1].x(), 40.0);
    let crate::ArrowProjectionKindV1::Normal {
        head_shape,
        start_head,
        end_head,
    } = arrow.kind()
    else {
        panic!("normal source must retain normal semantic policy");
    };
    assert!(!start_head && *end_head);
    assert_eq!(head_shape.total_length(), 10.0);
    assert_eq!(arrow.stroke().color().as_str(), "#123456");
    assert_eq!(arrow.stroke().width().value(), 2.0);
}

#[test]
fn curved_terminal_arrows_preserve_their_closed_semantic_identity() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"electron\" type=\"electron\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"10\"/><point x=\"20\" y=\"0\"/></arrow><arrow id=\"retro\" type=\"retro\"><point x=\"30\" y=\"0\"/><point x=\"40\" y=\"10\"/><point x=\"50\" y=\"0\"/></arrow><arrow id=\"normal\" type=\"curved-normal\"><point x=\"60\" y=\"0\"/><point x=\"70\" y=\"10\"/><point x=\"80\" y=\"0\"/></arrow></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    let kinds: Vec<_> = stack
        .roots()
        .iter()
        .map(|root| {
            let crate::PresentationRootProjectionV1::Arrow { arrow } = root else {
                panic!("expected terminal arrow root");
            };
            let crate::ArrowProjectionKindV1::CurvedTerminal { terminal_kind } = arrow.kind()
            else {
                panic!("expected shared curved terminal policy");
            };
            *terminal_kind
        })
        .collect();
    assert_eq!(
        kinds,
        vec![
            crate::CurvedTerminalArrowKindV1::Electron,
            crate::CurvedTerminalArrowKindV1::Retro,
            crate::CurvedTerminalArrowKindV1::Normal,
        ]
    );
}

#[test]
fn electron_arrows_refuse_normal_head_facts_and_non_quadratic_cardinality() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"head\" type=\"electron\" end=\"no\"><point x=\"0\" y=\"0\"/><point x=\"20\" y=\"10\"/><point x=\"40\" y=\"0\"/></arrow><arrow id=\"short\" type=\"electron\"><point x=\"0\" y=\"0\"/><point x=\"40\" y=\"0\"/></arrow></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    assert!(stack.roots().is_empty());
    assert_eq!(
        stack
            .issues()
            .iter()
            .map(|issue| issue.code())
            .collect::<Vec<_>>(),
        vec![
            PresentationProjectionIssueCodeV1::InvalidArrowFact,
            PresentationProjectionIssueCodeV1::InvalidArrowGeometry,
        ]
    );
    assert!(
        stack.issues()[0]
            .detail()
            .contains("no normal-arrow head facts")
    );
    assert!(stack.issues()[1].detail().contains("exactly three points"));
}

#[test]
fn curved_equilibrium_arrow_has_a_direct_closed_three_point_policy() {
    let accepted = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"accepted\" type=\"curved-equilibrium\" width=\"2\" color=\"#123456\"><point x=\"0\" y=\"0\"/><point x=\"40\" y=\"20\"/><point x=\"80\" y=\"0\"/></arrow></cdml>",
    );
    let stack = accepted.projection().presentation_stack();
    let [PresentationRootProjectionV1::Arrow { arrow }] = stack.roots() else {
        panic!("expected the direct curved-equilibrium root");
    };
    let crate::ArrowProjectionKindV1::CurvedEquilibrium = arrow.kind() else {
        panic!("expected named curved-equilibrium policy");
    };
    assert_eq!(arrow.source_path().points().len(), 3);
    assert!(stack.issues().is_empty());

    let rejected = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"equilibrium2\" type=\"equilibrium2\"><point x=\"0\" y=\"0\"/><point x=\"40\" y=\"20\"/><point x=\"80\" y=\"0\"/></arrow><arrow id=\"facts\" type=\"curved-equilibrium\" spline=\"no\" start=\"yes\" end=\"yes\" shape=\"(8,10,3)\" properties=\"x\" association=\"reaction\" factory=\"generic\"><point x=\"0\" y=\"0\"/><point x=\"40\" y=\"20\"/><point x=\"80\" y=\"0\"/></arrow></cdml>",
    );
    let rejected_stack = rejected.projection().presentation_stack();
    assert!(rejected_stack.roots().is_empty());
    assert_eq!(
        rejected_stack
            .issues()
            .iter()
            .map(|issue| issue.code())
            .collect::<Vec<_>>(),
        vec![
            PresentationProjectionIssueCodeV1::UnsupportedArrowType,
            PresentationProjectionIssueCodeV1::InvalidArrowFact,
        ]
    );

    let rejected_geometry = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"reverse\" type=\"curved-equilibrium\"><point x=\"0\" y=\"0\"/><point x=\"-10\" y=\"5\"/><point x=\"80\" y=\"0\"/></arrow><arrow id=\"cusp\" type=\"curved-equilibrium\"><point x=\"0\" y=\"0\"/><point x=\"0\" y=\"0\"/><point x=\"80\" y=\"0\"/></arrow></cdml>",
    );
    let rejected_geometry_stack = rejected_geometry.projection().presentation_stack();
    assert!(rejected_geometry_stack.roots().is_empty());
    assert_eq!(
        rejected_geometry_stack
            .issues()
            .iter()
            .map(|issue| issue.code())
            .collect::<Vec<_>>(),
        vec![
            PresentationProjectionIssueCodeV1::InvalidArrowGeometry,
            PresentationProjectionIssueCodeV1::InvalidArrowGeometry,
        ]
    );
    assert!(
        rejected_geometry_stack
            .issues()
            .iter()
            .all(|issue| issue.detail().contains("endpoint tangents"))
    );
}

#[test]
fn plus_projects_anchor_and_resolved_appearance_without_font_layout() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><standard font_size=\"12\" line_color=\"#123456\"/><plus id=\"p\" font_size=\"18\" color=\"#AbC\" background-color=\"#fedcba\"><point x=\"1cm\" y=\"2\"/></plus></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    let [PresentationRootProjectionV1::Plus { plus }] = stack.roots() else {
        panic!("expected one plus source projection");
    };
    assert_eq!(plus.target().source_id(), Some("p"));
    assert_eq!((plus.anchor().x(), plus.anchor().y()), (72.0 / 2.54, 2.0));
    assert_eq!(plus.font().font_face().id(), "telex_regular_v1");
    assert_eq!(
        plus.font().font_face_provenance(),
        PresentationFactProvenanceV1::Builtin
    );
    assert_eq!(plus.font().size().value(), 18.0);
    assert_eq!(
        plus.font().size_provenance(),
        PresentationFactProvenanceV1::Root
    );
    assert_eq!(plus.font().color().as_str(), "#aabbcc");
    assert_eq!(plus.background().color().unwrap().as_str(), "#fedcba");
    assert!(stack.issues().is_empty());
}

#[test]
fn text_projects_normalized_rich_runs_multiline_content_and_resolved_appearance() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><standard font_size=\"11\" line_color=\"#123456\"/><text id=\"label\" background-color=\"#AbC\"><point x=\"1cm\" y=\"2\"/><font size=\"18\" color=\"#fedcba\"/><ftext>Hello &lt;b&gt;bold &lt;sub&gt;2&lt;/sub&gt;&lt;/b&gt;\nnext &amp;amp; end</ftext></text></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    let [PresentationRootProjectionV1::Text { text }] = stack.roots() else {
        panic!("expected one Text source projection");
    };
    assert_eq!(text.target().source_id(), Some("label"));
    assert_eq!((text.anchor().x(), text.anchor().y()), (72.0 / 2.54, 2.0));
    assert_eq!(text.font().size().value(), 18.0);
    assert_eq!(
        text.font().size_provenance(),
        PresentationFactProvenanceV1::Root
    );
    assert_eq!(text.font().color().as_str(), "#fedcba");
    assert_eq!(text.background().color().unwrap().as_str(), "#aabbcc");
    assert_eq!(
        text.runs()
            .iter()
            .map(|run| (run.text(), run.styles()))
            .collect::<Vec<_>>(),
        vec![
            ("Hello ", &[][..]),
            ("bold ", &[PresentationTextStyleV1::Bold][..]),
            (
                "2",
                &[
                    PresentationTextStyleV1::Bold,
                    PresentationTextStyleV1::Subscript,
                ][..],
            ),
            ("\nnext & end", &[][..]),
        ]
    );
    assert!(stack.issues().is_empty());
}

#[test]
fn unsupported_formatted_text_is_a_targeted_issue_without_a_fallback_root() {
    for authored in [
        "&lt;u&gt;unknown&lt;/u&gt;",
        "&lt;b&gt;&lt;b&gt;duplicate&lt;/b&gt;&lt;/b&gt;",
        "&lt;sub&gt;&lt;sup&gt;conflict&lt;/sup&gt;&lt;/sub&gt;",
        "&lt;!--comment--&gt;",
        "&amp;custom;",
        "&lt;!DOCTYPE ftext-root&gt;unsafe",
    ] {
        let observation = observed(&format!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><text id=\"bad\"><point x=\"0\" y=\"0\"/><ftext>{authored}</ftext></text></cdml>",
        ));
        let stack = observation.projection().presentation_stack();
        assert!(stack.roots().is_empty());
        let [issue] = stack.issues() else {
            panic!("expected one Text content issue");
        };
        assert_eq!(
            issue.code(),
            PresentationProjectionIssueCodeV1::InvalidTextContent
        );
        assert_eq!(issue.target().source_id(), Some("bad"));
    }
}

#[test]
fn vector_shapes_preserve_kind_geometry_appearance_and_root_order() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><standard line_color=\"#123456\" line_width=\"3\" area_color=\"#abc\"/><rect id=\"r\" x1=\"10\" y1=\"8\" x2=\"2\" y2=\"4\" area_color=\"none\"/><square id=\"s\" x1=\"1\" y1=\"2\" x2=\"5\" y2=\"6\" area_color=\"blue\"/><oval id=\"o\" x1=\"0\" y1=\"0\" x2=\"7\" y2=\"3\" area_color=\"#010203\"/><circle id=\"c\" x1=\"0\" y1=\"0\" x2=\"4\" y2=\"6\" area_color=\"\"/><polygon id=\"p\" line_color=\"#fedcba\" width=\"2\"><point x=\"0\" y=\"0\"/><point x=\"5\" y=\"1\"/><point x=\"2\" y=\"7\"/></polygon></cdml>",
    );
    let roots = observation.projection().presentation_stack().roots();
    let [
        PresentationRootProjectionV1::Rectangle { shape: rectangle },
        PresentationRootProjectionV1::Square { shape: square },
        PresentationRootProjectionV1::Oval { shape: oval },
        PresentationRootProjectionV1::Circle { shape: circle },
        PresentationRootProjectionV1::Polygon { polygon },
    ] = roots
    else {
        panic!("expected every closed vector shape in source order");
    };
    let bounds = rectangle.bounds();
    assert_eq!(
        (bounds.left(), bounds.top(), bounds.right(), bounds.bottom()),
        (2.0, 4.0, 10.0, 8.0)
    );
    assert!(rectangle.fill().color().is_none());
    assert_eq!(
        rectangle.fill().color_provenance(),
        PresentationFactProvenanceV1::Root
    );
    assert_eq!(square.fill().color().unwrap().as_str(), "#aabbcc");
    assert_eq!(square.stroke().color().as_str(), "#123456");
    assert_eq!(square.stroke().width().value(), 3.0);
    assert_eq!(oval.fill().color().unwrap().as_str(), "#010203");
    assert!(circle.fill().color().is_none());
    assert_eq!(circle.bounds().bottom(), 6.0);
    assert_eq!(polygon.target().source_order(), 5);
    assert_eq!(polygon.stroke().color().as_str(), "#fedcba");
    assert_eq!(
        polygon
            .path()
            .points()
            .iter()
            .map(|point| (point.x(), point.y()))
            .collect::<Vec<_>>(),
        vec![(0.0, 0.0), (5.0, 1.0), (2.0, 7.0)]
    );
    let issues = observation.projection().presentation_stack().issues();
    assert_eq!(issues.len(), 1);
    assert_eq!(
        issues[0].code(),
        PresentationProjectionIssueCodeV1::InvalidFillFact
    );
    assert_eq!(issues[0].target().source_id(), Some("s"));
}

#[test]
fn zero_extent_box_shapes_are_targeted_projection_issues_without_roots() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><rect id=\"rect-width\" x1=\"1\" y1=\"0\" x2=\"1\" y2=\"2\"/><square id=\"square-height\" x1=\"0\" y1=\"2\" x2=\"2\" y2=\"2\"/><oval id=\"oval-width\" x1=\"3\" y1=\"0\" x2=\"3\" y2=\"2\"/><circle id=\"circle-height\" x1=\"0\" y1=\"4\" x2=\"2\" y2=\"4\"/></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    assert!(stack.roots().is_empty());
    assert_eq!(
        stack
            .issues()
            .iter()
            .map(|issue| (issue.target().source_id(), issue.code()))
            .collect::<Vec<_>>(),
        vec![
            (
                Some("rect-width"),
                PresentationProjectionIssueCodeV1::InvalidShapeGeometry,
            ),
            (
                Some("square-height"),
                PresentationProjectionIssueCodeV1::InvalidShapeGeometry,
            ),
            (
                Some("oval-width"),
                PresentationProjectionIssueCodeV1::InvalidShapeGeometry,
            ),
            (
                Some("circle-height"),
                PresentationProjectionIssueCodeV1::InvalidShapeGeometry,
            ),
        ]
    );
}

#[test]
fn root_interleave_and_standard_precedence_remain_explicit() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><polyline id=\"first\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline><molecule><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><standard line_color=\"#123456\" line_width=\"3\"/><polyline id=\"last\" line_color=\"#abcdef\" width=\"4\"><point x=\"2\" y=\"2\"/><point x=\"3\" y=\"3\"/></polyline></cdml>",
    );
    let roots = observation.projection().presentation_stack().roots();
    let [
        PresentationRootProjectionV1::Polyline { polyline: first },
        PresentationRootProjectionV1::Polyline { polyline: last },
    ] = roots
    else {
        panic!("expected two polylines");
    };
    assert_eq!(first.target().source_order(), 0);
    assert_eq!(last.target().source_order(), 3);
    assert_eq!(first.stroke().color().as_str(), "#123456");
    assert_eq!(first.stroke().width().value(), 3.0);
    assert_eq!(
        first.stroke().color_provenance(),
        PresentationFactProvenanceV1::Standard
    );
    assert_eq!(
        first.stroke().width_provenance(),
        PresentationFactProvenanceV1::Standard
    );
    assert_eq!(
        last.stroke().color_provenance(),
        PresentationFactProvenanceV1::Root
    );
    assert_eq!(
        last.stroke().width_provenance(),
        PresentationFactProvenanceV1::Root
    );
}

#[test]
fn alternate_prefix_and_opaque_payload_are_preserved_without_changing_projection() {
    let source = "<c:cdml xmlns:c=\"urn:ferrum:cdml\"><c:polyline id=\"line\" keep=\"yes\"><c:point x=\"0\" y=\"0\"/><c:point x=\"1\" y=\"1\"/><foreign xmlns=\"urn:opaque\"/></c:polyline></c:cdml>";
    let session = DocumentSession::load(source).unwrap();
    let snapshot = session.snapshot().unwrap();
    assert!(snapshot.cdml().contains("urn:opaque"));
    let projection = session
        .observe(0)
        .unwrap()
        .projection()
        .presentation_stack()
        .clone();
    assert_eq!(projection.roots().len(), 1);
    assert_eq!(projection.roots()[0].target().source_id(), Some("line"));
}

#[test]
fn invalid_geometry_is_a_targeted_display_only_issue_without_fallback() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><polyline id=\"bad\"><point x=\"NaN\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline><polyline id=\"short\"><point x=\"0\" y=\"0\"/></polyline><polyline id=\"wave\" style=\"wavy\" spline=\"yes\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline><rect id=\"partial\" x1=\"0\" y1=\"0\" x2=\"1\"/><polygon id=\"triangle\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polygon><arrow id=\"retro\" type=\"retro\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"0\"/></arrow><arrow id=\"curve\" spline=\"yes\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"0\"/></arrow><arrow id=\"shape\" shape=\"(10,8,3)\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"0\"/></arrow><arrow id=\"zero\"><point x=\"0\" y=\"0\"/><point x=\"0\" y=\"0\"/></arrow></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    assert!(stack.roots().is_empty());
    assert_eq!(
        stack
            .issues()
            .iter()
            .map(|issue| issue.code())
            .collect::<Vec<_>>(),
        vec![
            PresentationProjectionIssueCodeV1::InvalidPolylineGeometry,
            PresentationProjectionIssueCodeV1::InvalidPolylineGeometry,
            PresentationProjectionIssueCodeV1::UnsupportedSpline,
            PresentationProjectionIssueCodeV1::InvalidShapeGeometry,
            PresentationProjectionIssueCodeV1::InvalidPolygonGeometry,
            PresentationProjectionIssueCodeV1::InvalidArrowGeometry,
            PresentationProjectionIssueCodeV1::UnsupportedArrowSpline,
            PresentationProjectionIssueCodeV1::InvalidArrowFact,
            PresentationProjectionIssueCodeV1::InvalidArrowGeometry,
        ]
    );
    assert_eq!(stack.issues()[0].target().source_id(), Some("bad"));
    assert_eq!(stack.issues()[1].target().source_id(), Some("short"));
    assert_eq!(stack.issues()[2].target().source_id(), Some("wave"));
    assert_eq!(stack.issues()[3].target().source_id(), Some("partial"));
    assert_eq!(stack.issues()[4].target().source_id(), Some("triangle"));
    assert_eq!(stack.issues()[5].target().source_id(), Some("retro"));
    assert_eq!(stack.issues()[6].target().source_id(), Some("curve"));
    assert_eq!(stack.issues()[7].target().source_id(), Some("shape"));
    assert_eq!(stack.issues()[8].target().source_id(), Some("zero"));
}

#[test]
fn idless_identical_roots_get_unique_non_operation_keys_and_closed_wire_names() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><polyline><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline><polyline><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    assert!(
        stack
            .roots()
            .iter()
            .all(|root| root.target().id().is_none())
    );
    assert_ne!(
        stack.roots()[0].target().projection_key().as_str(),
        stack.roots()[1].target().projection_key().as_str()
    );
}

#[test]
fn stale_observations_cannot_be_requested_after_a_session_change() {
    let mut session = DocumentSession::load(
        "<cdml xmlns=\"urn:ferrum:cdml\"><polyline id=\"line\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .unwrap();
    let before = session.observe(0).unwrap();
    session
        .apply_document_operation_v1(
            0,
            super::SessionOperation::V1(super::SessionOperationV1::SetAtomElement {
                atom_id: "a".to_owned(),
                element: "N".to_owned(),
            }),
        )
        .unwrap();
    assert!(matches!(
        session.observe(0),
        Err(super::DocumentSessionError::RevisionConflict { .. })
    ));
    let after = session.observe(1).unwrap();
    assert_eq!(before.projection().presentation_stack().revision(), 0);
    assert_eq!(after.projection().presentation_stack().revision(), 1);
    assert_ne!(
        before.projection().presentation_stack().digest(),
        after.projection().presentation_stack().digest()
    );
}

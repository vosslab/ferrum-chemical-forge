use super::{
    DocumentSession, PRESENTATION_STACK_PROJECTION_SCHEMA_V1, PresentationFactProvenanceV1,
    PresentationProjectionIssueCodeV1, PresentationRecordKindV1, PresentationRootProjectionV1,
    PresentationTargetV1, PresentationTextStyleV1,
};
use ferrum_document_projection::{DocumentDirectRootKindV1, DocumentDirectRootV1};

fn assert_durable_target(target: &PresentationTargetV1, expected_kind: PresentationRecordKindV1) {
    assert!(
        target
            .document_object_id()
            .as_str()
            .starts_with("ferrum-document-object-v1/")
    );
    assert_eq!(target.record_kind(), expected_kind);
}

fn direct_root_for_target<'a>(
    observation: &'a super::SessionDocumentObservationV1,
    target: &PresentationTargetV1,
) -> &'a DocumentDirectRootV1 {
    observation
        .projection()
        .direct_roots()
        .iter()
        .find(|root| root.document_object_id() == target.document_object_id())
        .expect("expected document direct root for presentation target")
}

fn assert_presentation_direct_root(
    observation: &super::SessionDocumentObservationV1,
    target: &PresentationTargetV1,
    expected_paint_order: u32,
    expected_kind: PresentationRecordKindV1,
) {
    let root = direct_root_for_target(observation, target);
    assert_eq!(root.paint_order(), expected_paint_order);
    assert_eq!(
        root.kind(),
        DocumentDirectRootKindV1::Presentation(expected_kind)
    );
}

fn assert_rejected_presentation_direct_root(
    observation: &super::SessionDocumentObservationV1,
    issue: &super::PresentationProjectionIssueV1,
    expected_paint_order: u32,
) {
    let root = direct_root_for_target(observation, issue.target());
    assert_eq!(root.paint_order(), expected_paint_order);
    assert_eq!(
        root.kind(),
        DocumentDirectRootKindV1::RejectedPresentation(issue.code())
    );
}

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
    let [entry] = stack.entries() else {
        panic!("expected one polyline");
    };
    let PresentationRootProjectionV1::Polyline { polyline } = entry.root() else {
        panic!("expected one polyline");
    };
    assert_durable_target(polyline.target(), PresentationRecordKindV1::Polyline);
    assert_presentation_direct_root(
        &observation,
        polyline.target(),
        0,
        PresentationRecordKindV1::Polyline,
    );
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
    let [entry] = stack.entries() else {
        panic!("expected one supported normal arrow");
    };
    let PresentationRootProjectionV1::Arrow { arrow } = entry.root() else {
        panic!("expected one supported normal arrow");
    };
    assert_durable_target(arrow.target(), PresentationRecordKindV1::Arrow);
    assert_presentation_direct_root(
        &observation,
        arrow.target(),
        0,
        PresentationRecordKindV1::Arrow,
    );
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
        .entries()
        .iter()
        .map(|entry| {
            let crate::PresentationRootProjectionV1::Arrow { arrow } = entry.root() else {
                panic!("expected terminal arrow root");
            };
            assert_durable_target(arrow.target(), PresentationRecordKindV1::Arrow);
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
    assert!(stack.entries().is_empty());
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
    for (issue, expected_paint_order) in stack.issues().iter().zip([0, 1]) {
        assert_rejected_presentation_direct_root(&observation, issue, expected_paint_order);
    }
}

#[test]
fn curved_equilibrium_arrow_has_a_direct_closed_three_point_policy() {
    let accepted = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"accepted\" type=\"curved-equilibrium\" width=\"2\" color=\"#123456\"><point x=\"0\" y=\"0\"/><point x=\"40\" y=\"20\"/><point x=\"80\" y=\"0\"/></arrow></cdml>",
    );
    let stack = accepted.projection().presentation_stack();
    let [entry] = stack.entries() else {
        panic!("expected the direct curved-equilibrium root");
    };
    let PresentationRootProjectionV1::Arrow { arrow } = entry.root() else {
        panic!("expected the direct curved-equilibrium root");
    };
    assert_durable_target(arrow.target(), PresentationRecordKindV1::Arrow);
    assert_presentation_direct_root(
        &accepted,
        arrow.target(),
        0,
        PresentationRecordKindV1::Arrow,
    );
    let crate::ArrowProjectionKindV1::CurvedEquilibrium = arrow.kind() else {
        panic!("expected named curved-equilibrium policy");
    };
    assert_eq!(arrow.source_path().points().len(), 3);
    assert!(stack.issues().is_empty());

    let rejected = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"equilibrium2\" type=\"equilibrium2\"><point x=\"0\" y=\"0\"/><point x=\"40\" y=\"20\"/><point x=\"80\" y=\"0\"/></arrow><arrow id=\"facts\" type=\"curved-equilibrium\" spline=\"no\" start=\"yes\" end=\"yes\" shape=\"(8,10,3)\" properties=\"x\" association=\"reaction\" factory=\"generic\"><point x=\"0\" y=\"0\"/><point x=\"40\" y=\"20\"/><point x=\"80\" y=\"0\"/></arrow></cdml>",
    );
    let rejected_stack = rejected.projection().presentation_stack();
    assert!(rejected_stack.entries().is_empty());
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
    for (issue, expected_paint_order) in rejected_stack.issues().iter().zip([0, 1]) {
        assert_rejected_presentation_direct_root(&rejected, issue, expected_paint_order);
    }

    let rejected_geometry = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"reverse\" type=\"curved-equilibrium\"><point x=\"0\" y=\"0\"/><point x=\"-10\" y=\"5\"/><point x=\"80\" y=\"0\"/></arrow><arrow id=\"cusp\" type=\"curved-equilibrium\"><point x=\"0\" y=\"0\"/><point x=\"0\" y=\"0\"/><point x=\"80\" y=\"0\"/></arrow></cdml>",
    );
    let rejected_geometry_stack = rejected_geometry.projection().presentation_stack();
    assert!(rejected_geometry_stack.entries().is_empty());
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
    for (issue, expected_paint_order) in rejected_geometry_stack.issues().iter().zip([0, 1]) {
        assert_rejected_presentation_direct_root(&rejected_geometry, issue, expected_paint_order);
    }
}

#[test]
fn plus_projects_anchor_and_resolved_appearance_without_font_layout() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><standard font_size=\"12\" line_color=\"#123456\"/><plus id=\"p\" font_size=\"18\" color=\"#AbC\" background-color=\"#fedcba\"><point x=\"1cm\" y=\"2\"/></plus></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    let [entry] = stack.entries() else {
        panic!("expected one plus source projection");
    };
    let PresentationRootProjectionV1::Plus { plus } = entry.root() else {
        panic!("expected one plus source projection");
    };
    assert_durable_target(plus.target(), PresentationRecordKindV1::Plus);
    assert_presentation_direct_root(
        &observation,
        plus.target(),
        1,
        PresentationRecordKindV1::Plus,
    );
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
    let [entry] = stack.entries() else {
        panic!("expected one Text source projection");
    };
    let PresentationRootProjectionV1::Text { text } = entry.root() else {
        panic!("expected one Text source projection");
    };
    assert_durable_target(text.target(), PresentationRecordKindV1::Text);
    assert_presentation_direct_root(
        &observation,
        text.target(),
        1,
        PresentationRecordKindV1::Text,
    );
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
        assert!(stack.entries().is_empty());
        let [issue] = stack.issues() else {
            panic!("expected one Text content issue");
        };
        assert_eq!(
            issue.code(),
            PresentationProjectionIssueCodeV1::InvalidTextContent
        );
        assert_durable_target(issue.target(), PresentationRecordKindV1::Text);
        assert_rejected_presentation_direct_root(&observation, issue, 0);
    }
}

#[test]
fn vector_shapes_preserve_kind_geometry_appearance_and_root_order() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><standard line_color=\"#123456\" line_width=\"3\" area_color=\"#abc\"/><rect id=\"r\" x1=\"10\" y1=\"8\" x2=\"2\" y2=\"4\" area_color=\"none\"/><square id=\"s\" x1=\"1\" y1=\"2\" x2=\"5\" y2=\"6\" area_color=\"blue\"/><oval id=\"o\" x1=\"0\" y1=\"0\" x2=\"7\" y2=\"3\" area_color=\"#010203\"/><circle id=\"c\" x1=\"0\" y1=\"0\" x2=\"4\" y2=\"6\" area_color=\"\"/><polygon id=\"p\" line_color=\"#fedcba\" width=\"2\"><point x=\"0\" y=\"0\"/><point x=\"5\" y=\"1\"/><point x=\"2\" y=\"7\"/></polygon></cdml>",
    );
    let entries = observation.projection().presentation_stack().entries();
    let rectangle_entry = entries
        .iter()
        .find(|entry| entry.root().target().record_kind() == PresentationRecordKindV1::Rectangle)
        .expect("expected rectangle root");
    let square_entry = entries
        .iter()
        .find(|entry| entry.root().target().record_kind() == PresentationRecordKindV1::Square)
        .expect("expected square root");
    let oval_entry = entries
        .iter()
        .find(|entry| entry.root().target().record_kind() == PresentationRecordKindV1::Oval)
        .expect("expected oval root");
    let circle_entry = entries
        .iter()
        .find(|entry| entry.root().target().record_kind() == PresentationRecordKindV1::Circle)
        .expect("expected circle root");
    let polygon_entry = entries
        .iter()
        .find(|entry| entry.root().target().record_kind() == PresentationRecordKindV1::Polygon)
        .expect("expected polygon root");
    let PresentationRootProjectionV1::Rectangle { shape: rectangle } = rectangle_entry.root()
    else {
        panic!("expected rectangle root");
    };
    let PresentationRootProjectionV1::Square { shape: square } = square_entry.root() else {
        panic!("expected square root");
    };
    let PresentationRootProjectionV1::Oval { shape: oval } = oval_entry.root() else {
        panic!("expected oval root");
    };
    let PresentationRootProjectionV1::Circle { shape: circle } = circle_entry.root() else {
        panic!("expected circle root");
    };
    let PresentationRootProjectionV1::Polygon { polygon } = polygon_entry.root() else {
        panic!("expected polygon root");
    };
    assert_durable_target(rectangle.target(), PresentationRecordKindV1::Rectangle);
    assert_durable_target(square.target(), PresentationRecordKindV1::Square);
    assert_durable_target(oval.target(), PresentationRecordKindV1::Oval);
    assert_durable_target(circle.target(), PresentationRecordKindV1::Circle);
    assert_durable_target(polygon.target(), PresentationRecordKindV1::Polygon);
    assert_presentation_direct_root(
        &observation,
        rectangle.target(),
        1,
        PresentationRecordKindV1::Rectangle,
    );
    assert_presentation_direct_root(
        &observation,
        square.target(),
        2,
        PresentationRecordKindV1::Square,
    );
    assert_presentation_direct_root(
        &observation,
        oval.target(),
        3,
        PresentationRecordKindV1::Oval,
    );
    assert_presentation_direct_root(
        &observation,
        circle.target(),
        4,
        PresentationRecordKindV1::Circle,
    );
    assert_presentation_direct_root(
        &observation,
        polygon.target(),
        5,
        PresentationRecordKindV1::Polygon,
    );
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
    assert_eq!(
        (
            square.bounds().left(),
            square.bounds().top(),
            square.bounds().right(),
            square.bounds().bottom(),
        ),
        (1.0, 2.0, 5.0, 6.0)
    );
    assert_eq!(oval.fill().color().unwrap().as_str(), "#010203");
    assert_eq!(
        (
            oval.bounds().left(),
            oval.bounds().top(),
            oval.bounds().right(),
            oval.bounds().bottom(),
        ),
        (0.0, 0.0, 7.0, 3.0)
    );
    assert!(circle.fill().color().is_none());
    assert_eq!(
        (
            circle.bounds().left(),
            circle.bounds().top(),
            circle.bounds().right(),
            circle.bounds().bottom(),
        ),
        (0.0, 0.0, 4.0, 6.0)
    );
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
    assert_durable_target(issues[0].target(), PresentationRecordKindV1::Square);
}

#[test]
fn zero_extent_box_shapes_are_targeted_projection_issues_without_roots() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><rect id=\"rect-width\" x1=\"1\" y1=\"0\" x2=\"1\" y2=\"2\"/><square id=\"square-height\" x1=\"0\" y1=\"2\" x2=\"2\" y2=\"2\"/><oval id=\"oval-width\" x1=\"3\" y1=\"0\" x2=\"3\" y2=\"2\"/><circle id=\"circle-height\" x1=\"0\" y1=\"4\" x2=\"2\" y2=\"4\"/></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    assert!(stack.entries().is_empty());
    assert_eq!(
        stack
            .issues()
            .iter()
            .map(|issue| (issue.target().record_kind(), issue.code()))
            .collect::<Vec<_>>(),
        vec![
            (
                PresentationRecordKindV1::Rectangle,
                PresentationProjectionIssueCodeV1::InvalidShapeGeometry,
            ),
            (
                PresentationRecordKindV1::Square,
                PresentationProjectionIssueCodeV1::InvalidShapeGeometry,
            ),
            (
                PresentationRecordKindV1::Oval,
                PresentationProjectionIssueCodeV1::InvalidShapeGeometry,
            ),
            (
                PresentationRecordKindV1::Circle,
                PresentationProjectionIssueCodeV1::InvalidShapeGeometry,
            ),
        ]
    );
    for (issue, expected_paint_order) in stack.issues().iter().zip([0, 1, 2, 3]) {
        assert_rejected_presentation_direct_root(&observation, issue, expected_paint_order);
    }
}

#[test]
fn root_interleave_and_standard_precedence_remain_explicit() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><polyline id=\"first\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><standard line_color=\"#123456\" line_width=\"3\"/><polyline id=\"last\" line_color=\"#abcdef\" width=\"4\"><point x=\"2\" y=\"2\"/><point x=\"3\" y=\"3\"/></polyline></cdml>",
    );
    let entries = observation.projection().presentation_stack().entries();
    let [first_entry, last_entry] = entries else {
        panic!("expected two polylines");
    };
    let PresentationRootProjectionV1::Polyline { polyline: first } = first_entry.root() else {
        panic!("expected first polyline");
    };
    let PresentationRootProjectionV1::Polyline { polyline: last } = last_entry.root() else {
        panic!("expected last polyline");
    };
    assert_durable_target(first.target(), PresentationRecordKindV1::Polyline);
    assert_durable_target(last.target(), PresentationRecordKindV1::Polyline);
    assert_presentation_direct_root(
        &observation,
        first.target(),
        0,
        PresentationRecordKindV1::Polyline,
    );
    assert_presentation_direct_root(
        &observation,
        last.target(),
        3,
        PresentationRecordKindV1::Polyline,
    );
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
    let observation = session.observe(0).unwrap();
    let projection = observation.projection().presentation_stack();
    let [entry] = projection.entries() else {
        panic!("expected one polyline root");
    };
    let PresentationRootProjectionV1::Polyline { polyline } = entry.root() else {
        panic!("expected one polyline root");
    };
    assert_durable_target(polyline.target(), PresentationRecordKindV1::Polyline);
    assert_presentation_direct_root(
        &observation,
        polyline.target(),
        0,
        PresentationRecordKindV1::Polyline,
    );
}

#[test]
fn invalid_geometry_is_a_targeted_display_only_issue_without_fallback() {
    let observation = observed(
        "<cdml xmlns=\"urn:ferrum:cdml\"><polyline id=\"bad\"><point x=\"NaN\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline><polyline id=\"short\"><point x=\"0\" y=\"0\"/></polyline><polyline id=\"wave\" style=\"wavy\" spline=\"yes\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline><rect id=\"partial\" x1=\"0\" y1=\"0\" x2=\"1\"/><polygon id=\"triangle\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polygon><arrow id=\"retro\" type=\"retro\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"0\"/></arrow><arrow id=\"curve\" spline=\"yes\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"0\"/></arrow><arrow id=\"shape\" shape=\"(10,8,3)\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"0\"/></arrow><arrow id=\"zero\"><point x=\"0\" y=\"0\"/><point x=\"0\" y=\"0\"/></arrow></cdml>",
    );
    let stack = observation.projection().presentation_stack();
    assert!(stack.entries().is_empty());
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
    for issue in stack.issues() {
        assert!(
            issue
                .target()
                .document_object_id()
                .as_str()
                .starts_with("ferrum-document-object-v1/")
        );
    }
    for (issue, expected_paint_order) in stack.issues().iter().zip(0..9) {
        assert_rejected_presentation_direct_root(&observation, issue, expected_paint_order);
    }
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

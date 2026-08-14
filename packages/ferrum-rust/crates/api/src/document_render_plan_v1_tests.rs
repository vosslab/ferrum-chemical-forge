use ferrum_document::DocumentSession;
use ferrum_render::{
    DocumentRenderContentV1, DocumentRenderOutcomeV1, render_document_plan_to_svg_v1,
};
use xot::{Node, Xot};

use crate::{
    DepictionSuppressionV1, DocumentRenderPlanCompositionError, compose_document_render_plan_v1,
    observe_render_v1,
};

#[test]
fn composer_merges_molecule_plus_and_text_in_direct_root_order_with_one_page_provenance() {
    let session = DocumentSession::load(
        "<cdml><paper type=\"A4\"/><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><plus id=\"p\"><point x=\"30\" y=\"10\"/></plus><text id=\"t\"><point x=\"50\" y=\"20\"/><ftext>H&lt;sub&gt;2&lt;/sub&gt;O</ftext></text></cdml>",
    )
    .expect("session");
    let observation = observe_render_v1(&session, 0).expect("observation");
    let plan = compose_document_render_plan_v1(&observation).expect("document plan");

    assert_eq!(plan.provenance().revision().get(), 0);
    assert_eq!(
        plan.provenance().digest(),
        *observation.document().snapshot().digest()
    );
    assert_eq!((plan.page().x(), plan.page().y()), (0.0, 0.0));
    assert!(plan.page().width() > 0.0 && plan.page().height() > 0.0);
    assert_eq!(
        plan.outcomes()
            .iter()
            .map(DocumentRenderOutcomeV1::source_order)
            .collect::<Vec<_>>(),
        vec![1, 2, 3],
    );
    assert!(matches!(
        plan.outcomes()[0],
        DocumentRenderOutcomeV1::Root(ref root)
            if matches!(root.content(), DocumentRenderContentV1::Molecule(_))
    ));
    assert!(matches!(
        plan.outcomes()[1],
        DocumentRenderOutcomeV1::Root(ref root)
            if matches!(root.content(), DocumentRenderContentV1::Text(_))
    ));
    assert!(matches!(
        plan.outcomes()[2],
        DocumentRenderOutcomeV1::Root(ref root)
            if matches!(root.content(), DocumentRenderContentV1::Text(_))
    ));

    // This checks the actual document structure and paint-root order, not SVG bytes.
    let svg = render_document_plan_to_svg_v1(&plan).expect("structurally valid SVG");
    let mut tree = Xot::new();
    let document = tree.parse(svg.artifact().as_str()).expect("SVG parses");
    let root = tree.document_element(document).expect("SVG root");
    let painted = element_children(&tree, root);
    assert_eq!(painted.len(), 3);
    assert_eq!(
        attribute(
            &tree,
            element_children(&tree, painted[0])[0],
            "data-ferrum-space"
        ),
        Some("atom-local")
    );
    let text_roots = painted[1..]
        .iter()
        .map(|painted_root| element_children(&tree, *painted_root)[0])
        .collect::<Vec<_>>();
    assert!(text_roots.iter().all(|text_root| {
        attribute(&tree, *text_root, "data-ferrum-document-operation") == Some("text")
    }));
    assert!(painted.iter().all(|painted_root| {
        attribute(&tree, *painted_root, "data-ferrum-document-source-order").is_none()
            && attribute(&tree, *painted_root, "data-ferrum-document-identity").is_none()
    }));
}

fn element_children(tree: &Xot, node: Node) -> Vec<Node> {
    tree.children(node)
        .filter(|child| tree.element(*child).is_some())
        .collect()
}

fn attribute<'tree>(tree: &'tree Xot, node: Node, attribute: &str) -> Option<&'tree str> {
    tree.name(attribute)
        .and_then(|name| tree.get_attribute(node, name))
}

#[test]
fn composer_lowers_vector_roots_and_keeps_profile_excluded_text_without_synthetic_content() {
    let session = DocumentSession::load(
        "<cdml><polyline id=\"line\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"0\"/></polyline><text id=\"family\"><point x=\"0\" y=\"20\"/><font family=\"Arial\"/><ftext>label</ftext></text></cdml>",
    )
    .expect("session");
    let observation = observe_render_v1(&session, 0).expect("observation");
    let plan = compose_document_render_plan_v1(&observation).expect("document plan");

    assert_eq!(plan.outcomes().len(), 2);
    assert!(matches!(
        &plan.outcomes()[0],
        DocumentRenderOutcomeV1::Root(root)
            if matches!(root.content(), DocumentRenderContentV1::Vector(_))
    ));
    assert!(matches!(
        &plan.outcomes()[1],
        DocumentRenderOutcomeV1::Exclusion(exclusion)
            if exclusion.feature().starts_with("profile_excluded:")
    ));
}

#[test]
fn composer_preserves_direct_root_vector_geometry_and_paint_order() {
    let session = DocumentSession::load(concat!(
        "<cdml><arrow id=\"arrow\" type=\"normal\" start=\"no\" end=\"yes\" ",
        "shape=\"(8,10,3)\" color=\"#123456\"><point x=\"0\" y=\"0\"/>",
        "<point x=\"40\" y=\"0\"/></arrow><polyline id=\"line\"><point x=\"1\" y=\"2\"/>",
        "<point x=\"1\" y=\"2\"/><point x=\"3\" y=\"4\"/></polyline>",
        "<polyline id=\"wave\" style=\"wavy\"><point x=\"5\" y=\"6\"/>",
        "<point x=\"7\" y=\"8\"/><point x=\"9\" y=\"6\"/></polyline>",
        "<polyline id=\"bracket\" bracket_pair=\"bracket\" bracket_side=\"left\" spline=\"yes\">",
        "<point x=\"1\" y=\"0\"/><point x=\"0\" y=\"0\"/><point x=\"0\" y=\"10\"/>",
        "<point x=\"1\" y=\"10\"/></polyline><polyline id=\"bracket-right\" bracket_pair=\"bracket\" ",
        "bracket_side=\"right\" spline=\"yes\"><point x=\"9\" y=\"0\"/><point x=\"10\" y=\"0\"/>",
        "<point x=\"10\" y=\"10\"/><point x=\"9\" y=\"10\"/></polyline>",
        "<rect id=\"rect\" x1=\"1\" y1=\"2\" x2=\"5\" y2=\"6\" area_color=\"none\"/>",
        "<square id=\"square\" x1=\"7\" y1=\"8\" x2=\"11\" y2=\"12\" area_color=\"#abcdef\"/>",
        "<oval id=\"oval\" x1=\"13\" y1=\"14\" x2=\"19\" y2=\"18\" area_color=\"none\"/>",
        "<circle id=\"circle\" x1=\"20\" y1=\"21\" x2=\"26\" y2=\"29\" area_color=\"#010203\"/>",
        "<polygon id=\"polygon\" area_color=\"#fedcba\"><point x=\"0\" y=\"0\"/>",
        "<point x=\"5\" y=\"5\"/><point x=\"0\" y=\"5\"/><point x=\"5\" y=\"0\"/></polygon></cdml>"
    ))
    .expect("session");
    let plan =
        compose_document_render_plan_v1(&observe_render_v1(&session, 0).expect("observation"))
            .expect("document plan");
    let vectors = plan
        .outcomes()
        .iter()
        .filter_map(|outcome| match outcome {
            DocumentRenderOutcomeV1::Root(root) => match root.content() {
                DocumentRenderContentV1::Vector(vector) => Some(vector),
                _ => None,
            },
            DocumentRenderOutcomeV1::Exclusion(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(vectors.len(), 10);
    assert_eq!(vectors[0].operations().len(), 2);
    assert!(matches!(
        vectors[0].operations()[0].commands(),
        Some([
            ferrum_render::PathCommandV1::MoveTo(_),
            ferrum_render::PathCommandV1::LineTo(_)
        ])
    ));
    assert!(matches!(
        vectors[0].operations()[1].commands(),
        Some([
            ferrum_render::PathCommandV1::MoveTo(_),
            ferrum_render::PathCommandV1::LineTo(_),
            ferrum_render::PathCommandV1::LineTo(_),
            ferrum_render::PathCommandV1::LineTo(_),
            ferrum_render::PathCommandV1::Close
        ])
    ));
    assert!(matches!(
        vectors[2].operations()[0].commands(),
        Some([
            ferrum_render::PathCommandV1::MoveTo(_),
            ferrum_render::PathCommandV1::LineTo(_),
            ferrum_render::PathCommandV1::LineTo(_)
        ])
    ));
    assert!(matches!(
        vectors[3].operations()[0].commands(),
        Some([
            ferrum_render::PathCommandV1::MoveTo(_),
            ferrum_render::PathCommandV1::CubicTo { .. }
        ])
    ));
    assert!(vectors[5].operations()[0].fill().is_none());
    assert!(vectors[6].operations()[0].fill().is_some());
    assert!(vectors[7].operations()[0].ellipse_geometry().is_some());
    assert!(vectors[8].operations()[0].ellipse_geometry().is_some());
    assert!(matches!(
        vectors[9].operations()[0].commands(),
        Some([
            ferrum_render::PathCommandV1::MoveTo(_),
            ferrum_render::PathCommandV1::LineTo(_),
            ferrum_render::PathCommandV1::LineTo(_),
            ferrum_render::PathCommandV1::LineTo(_),
            ferrum_render::PathCommandV1::Close
        ])
    ));
}

#[test]
fn composer_records_rejected_projection_target_without_a_retained_root() {
    let session = DocumentSession::load(
        "<cdml><arrow id=\"bad\" type=\"normal\"><point x=\"0\" y=\"0\"/></arrow></cdml>",
    )
    .expect("session");
    let observation = observe_render_v1(&session, 0).expect("observation");
    let plan = compose_document_render_plan_v1(&observation).expect("document plan");

    let [DocumentRenderOutcomeV1::Exclusion(exclusion)] = plan.outcomes() else {
        panic!("one rejected projection exclusion");
    };
    assert!(exclusion.feature().starts_with("rejected_projection:"));
}

#[test]
fn composer_returns_typed_suppression_instead_of_an_empty_page_plan() {
    let session = DocumentSession::load(
        "<cdml><standard line_color=\"blue\"/><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect("session");
    let observation = observe_render_v1(&session, 0).expect("observation");

    assert!(matches!(
        compose_document_render_plan_v1(&observation),
        Err(DocumentRenderPlanCompositionError::Suppressed {
            suppression: DepictionSuppressionV1::InvalidPresentationFacts
        })
    ));
}

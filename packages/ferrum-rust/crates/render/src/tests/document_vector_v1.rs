use xot::{Node, Xot};

use crate::*;

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("test point")
}

fn paint(value: &str) -> Paint {
    Paint::rgb24(Rgb24::new(value).expect("test RGB"))
}

fn width(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("test positive width")
}

fn stroke(value: &str) -> StrokeV1 {
    StrokeV1::new(paint(value), width(1.5))
}

fn page() -> RenderViewportV1 {
    RenderViewportV1::new(0.0, 0.0, 100.0, 80.0).expect("test page")
}

fn plan(vector: DocumentVectorRootV1) -> DocumentRenderPlanV1 {
    DocumentRenderPlanV1::new(
        RenderProvenance::new(RenderRevision::new(1).expect("test revision"), [9; 32]),
        page(),
        vec![DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
            7,
            DocumentRenderIdentityV1::projection_local("vector-root").expect("test identity"),
            DocumentRenderContentV1::Vector(vector),
        ))],
    )
    .expect("test plan")
}

fn element_children(tree: &Xot, node: Node) -> Vec<Node> {
    tree.children(node)
        .filter(|child| tree.element(*child).is_some())
        .collect()
}

fn attribute<'tree>(tree: &'tree Xot, node: Node, name: &str) -> Option<&'tree str> {
    tree.name(name)
        .and_then(|name_id| tree.get_attribute(node, name_id))
}

#[test]
fn vector_open_stroke_emits_the_fixed_v1_profile() {
    let open = DocumentVectorOpV1::path(
        vec![
            PathCommandV1::MoveTo(point(1.0, 2.0)),
            PathCommandV1::LineTo(point(1.0, 2.0)),
            PathCommandV1::LineTo(point(4.0, 5.0)),
        ],
        Some(stroke("112233")),
        None,
    )
    .expect("open repeated-vertex path remains valid");
    let cubic = DocumentVectorOpV1::path(
        vec![
            PathCommandV1::MoveTo(point(0.0, 0.0)),
            PathCommandV1::CubicTo {
                control_1: point(1.0, 4.0),
                control_2: point(5.0, 4.0),
                end: point(6.0, 0.0),
            },
        ],
        Some(stroke("445566")),
        None,
    )
    .expect("four-point cubic path remains valid");
    let root = DocumentVectorRootV1::new(vec![open, cubic]).expect("vector root");
    let svg = render_document_plan_to_svg_v1(&plan(root)).expect("SVG lowering");
    let mut tree = Xot::new();
    let parsed = tree.parse(svg.artifact().as_str()).expect("SVG parses");
    let document = tree.document_element(parsed).expect("SVG root");
    let vector_root = element_children(&tree, document)[0];
    let paths = element_children(&tree, vector_root);
    assert_eq!(paths.len(), 2);
    assert_eq!(attribute(&tree, paths[0], "fill"), Some("none"));
    assert_eq!(attribute(&tree, paths[0], "stroke"), Some("#112233"));
    assert_eq!(attribute(&tree, paths[0], "stroke-width"), Some("1.5"));
    assert_eq!(attribute(&tree, paths[0], "stroke-linejoin"), Some("miter"));
    assert_eq!(attribute(&tree, paths[0], "stroke-linecap"), Some("butt"));
    assert_eq!(attribute(&tree, paths[0], "stroke-miterlimit"), Some("4"));
}

#[test]
fn vector_filled_self_intersection_uses_explicit_odd_even_and_closed_topology() {
    let polygon = DocumentVectorOpV1::path(
        vec![
            PathCommandV1::MoveTo(point(0.0, 0.0)),
            PathCommandV1::LineTo(point(8.0, 8.0)),
            PathCommandV1::LineTo(point(0.0, 8.0)),
            PathCommandV1::LineTo(point(8.0, 0.0)),
            PathCommandV1::Close,
        ],
        Some(stroke("102030")),
        Some(paint("aabbcc")),
    )
    .expect("self-intersecting closed polygon remains valid");
    let svg = render_document_plan_to_svg_v1(&plan(
        DocumentVectorRootV1::new(vec![polygon]).expect("vector root"),
    ))
    .expect("SVG lowering");
    let mut tree = Xot::new();
    let parsed = tree.parse(svg.artifact().as_str()).expect("SVG parses");
    let document = tree.document_element(parsed).expect("SVG root");
    let path = element_children(&tree, element_children(&tree, document)[0])[0];
    assert_eq!(attribute(&tree, path, "fill"), Some("#aabbcc"));
    assert_eq!(attribute(&tree, path, "fill-rule"), Some("evenodd"));
    assert_eq!(attribute(&tree, path, "stroke-linecap"), Some("butt"));
    assert_eq!(attribute(&tree, path, "stroke-linejoin"), Some("miter"));
    assert_eq!(attribute(&tree, path, "stroke-miterlimit"), Some("4"));
}

#[test]
fn vector_filled_path_keeps_two_closed_subpaths_in_one_ordered_paint() {
    let heads = DocumentVectorOpV1::path(
        vec![
            PathCommandV1::MoveTo(point(0.0, 0.0)),
            PathCommandV1::LineTo(point(3.0, 1.0)),
            PathCommandV1::LineTo(point(0.0, 2.0)),
            PathCommandV1::Close,
            PathCommandV1::MoveTo(point(6.0, 0.0)),
            PathCommandV1::LineTo(point(3.0, 1.0)),
            PathCommandV1::LineTo(point(6.0, 2.0)),
            PathCommandV1::Close,
        ],
        None,
        Some(paint("102030")),
    )
    .expect("two closed filled subpaths remain valid");
    let svg = render_document_plan_to_svg_v1(&plan(
        DocumentVectorRootV1::new(vec![heads]).expect("vector root"),
    ))
    .expect("SVG lowering");
    let mut tree = Xot::new();
    let parsed = tree.parse(svg.artifact().as_str()).expect("SVG parses");
    let document = tree.document_element(parsed).expect("SVG root");
    let path = element_children(&tree, element_children(&tree, document)[0])[0];
    assert_eq!(attribute(&tree, path, "fill-rule"), Some("evenodd"));
}

#[test]
fn curved_equilibrium_lowering_keeps_both_issued_axes_as_cubics() {
    let session = ferrum_document::DocumentSession::load(
        "<cdml xmlns=\"urn:ferrum:cdml\"><arrow id=\"equilibrium\" type=\"curved-equilibrium\" width=\"1\" color=\"#000000\"><point x=\"0\" y=\"0\"/><point x=\"40\" y=\"20\"/><point x=\"80\" y=\"0\"/></arrow></cdml>",
    )
    .expect("closed curved equilibrium source");
    let observation = session.observe(0).expect("initial observation");
    let [root] = observation.projection().presentation_stack().roots() else {
        panic!("one curved equilibrium root");
    };
    let lowered = lower_presentation_vector_v1(root).expect("curved equilibrium lowers");
    let [lower_axis, upper_axis, heads] = lowered.operations() else {
        panic!("two cubic lanes followed by both heads");
    };
    for axis in [lower_axis, upper_axis] {
        assert!(matches!(
            axis.commands(),
            Some([PathCommandV1::MoveTo(_), PathCommandV1::CubicTo { .. }])
        ));
    }
    assert!(matches!(
        heads.commands(),
        Some([
            PathCommandV1::MoveTo(_),
            PathCommandV1::LineTo(_),
            PathCommandV1::LineTo(_),
            PathCommandV1::LineTo(_),
            PathCommandV1::Close,
            PathCommandV1::MoveTo(_),
            PathCommandV1::LineTo(_),
            PathCommandV1::LineTo(_),
            PathCommandV1::LineTo(_),
            PathCommandV1::Close,
        ])
    ));
}

#[test]
fn vector_ellipse_maps_explicit_geometry_and_paint() {
    let ellipse = DocumentVectorOpV1::ellipse(
        point(12.0, 15.0),
        width(4.0),
        width(2.0),
        Some(stroke("334455")),
        None,
    )
    .expect("ellipse");
    let svg = render_document_plan_to_svg_v1(&plan(
        DocumentVectorRootV1::new(vec![ellipse]).expect("vector root"),
    ))
    .expect("SVG lowering");
    let mut tree = Xot::new();
    let parsed = tree.parse(svg.artifact().as_str()).expect("SVG parses");
    let document = tree.document_element(parsed).expect("SVG root");
    let ellipse = element_children(&tree, element_children(&tree, document)[0])[0];
    assert_eq!(
        attribute(&tree, ellipse, "data-ferrum-document-operation"),
        Some("ellipse")
    );
    assert_eq!(attribute(&tree, ellipse, "cx"), Some("12"));
    assert_eq!(attribute(&tree, ellipse, "cy"), Some("15"));
    assert_eq!(attribute(&tree, ellipse, "rx"), Some("4"));
    assert_eq!(attribute(&tree, ellipse, "ry"), Some("2"));
    assert_eq!(attribute(&tree, ellipse, "fill"), Some("none"));
    assert_eq!(attribute(&tree, ellipse, "stroke"), Some("#334455"));
    assert_eq!(attribute(&tree, ellipse, "stroke-linecap"), Some("butt"));
    assert_eq!(attribute(&tree, ellipse, "stroke-linejoin"), Some("miter"));
    assert_eq!(attribute(&tree, ellipse, "stroke-miterlimit"), Some("4"));
}

#[test]
fn vector_model_rejects_invalid_path_paint_and_radius_contracts() {
    let no_move = DocumentVectorOpV1::path(
        vec![PathCommandV1::LineTo(point(1.0, 1.0))],
        Some(stroke("000000")),
        None,
    );
    assert!(matches!(no_move, Err(RenderError::InvalidRequest(_))));

    let unclosed_fill = DocumentVectorOpV1::path(
        vec![
            PathCommandV1::MoveTo(point(0.0, 0.0)),
            PathCommandV1::LineTo(point(1.0, 1.0)),
        ],
        None,
        Some(paint("000000")),
    );
    assert!(matches!(unclosed_fill, Err(RenderError::InvalidRequest(_))));

    let no_paint = DocumentVectorOpV1::path(
        vec![
            PathCommandV1::MoveTo(point(0.0, 0.0)),
            PathCommandV1::LineTo(point(1.0, 1.0)),
        ],
        None,
        None,
    );
    assert!(matches!(no_paint, Err(RenderError::InvalidRequest(_))));

    assert!(matches!(
        PositiveFinite::new(0.0),
        Err(RenderError::InvalidRequest(_))
    ));
    assert!(matches!(
        DocumentVectorRootV1::new(vec![]),
        Err(RenderError::InvalidRequest(_))
    ));
}

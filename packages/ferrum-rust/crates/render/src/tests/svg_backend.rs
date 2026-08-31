use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_document_projection::DocumentObjectIdV1;
use xot::{Node, Xot};

use crate::atom_bond::build_atom_bond_plan;
use crate::render_target::RenderPlanEntryContextV1;
use crate::*;

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("test point is finite")
}

fn size(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("test extent is positive")
}

fn paint(value: &str) -> RenderPaintV3 {
    RenderPaintV3::authored_rgb24(Rgb24::new(value).expect("test paint is valid"))
}

fn target(id: u8) -> RenderTarget {
    RenderTarget::document_object(DocumentObjectIdV1::from_entropy_bytes([id; 16]))
}

fn context(
    kind: RecordKind,
    source: &str,
    target_id: u8,
    paint_order: u32,
) -> RenderPlanEntryContextV1 {
    RenderPlanEntryContextV1::new(
        target(target_id),
        RecordId::new(
            kind,
            Identifier::new(source).expect("test identifier is valid"),
        )
        .expect("test record ID"),
        paint_order,
        None,
    )
}

fn metrics() -> VerifiedMoleculeLabelGlyphMetrics {
    let environment =
        FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
    VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("verified Atkinson Hyperlegible Next opens")
}

fn mixed_plan() -> MoleculeRenderPlanV4 {
    let atom = AtomRenderTarget::new(
        context(RecordKind::Atom, "svg-atom", 0x11, 1),
        point(10.0, 20.0),
        AtomLabelFacts::new("N", None, 1, 2).expect("label facts"),
        TargetVisibility::Visible,
    )
    .expect("atom target")
    .with_marks(vec![
        AtomMarkRenderFacts::new(
            AtomMarkRenderKind::Plus,
            point(8.0, -12.0),
            45.0,
            size(10.0),
            true,
            size(1.0),
            paint("112233"),
        )
        .expect("plus mark"),
    ]);
    let bond = BondRenderTarget::new(
        context(RecordKind::Bond, "svg-bond", 0x12, 2),
        RecordId::new(
            RecordKind::Atom,
            Identifier::new("svg-atom").expect("test identifier"),
        )
        .expect("test record ID"),
        RecordId::new(
            RecordKind::Atom,
            Identifier::new("svg-neighbor").expect("test identifier"),
        )
        .expect("test record ID"),
        BondStyle::NormalSingle,
        TargetVisibility::Visible,
    )
    .expect("bond target");
    let neighbor = AtomRenderTarget::new(
        context(RecordKind::Atom, "svg-neighbor", 0x13, 3),
        point(50.0, 20.0),
        AtomLabelFacts::new("C", None, 0, 0).expect("label facts"),
        TargetVisibility::Visible,
    )
    .expect("neighbor target");
    let font = AtomLabelFontProfile::new(FontFace::molecule_label(), size(10.0), paint("000000"))
        .with_label_mask(paint("ffffff"));
    let request = AtomBondRenderRequest::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [13; 32]),
        vec![atom, neighbor],
        vec![bond],
        font,
        size(1.0),
        size(6.0),
        BondInkClearance::new(size(1.25)),
        paint("112233"),
    )
    .expect("render request");
    let plan = build_atom_bond_plan(&request, &metrics()).expect("atom and bond plan");
    let extra_batch = RenderBatchV4::test_compact_group_target(
        target(0x14),
        4,
        CompactGroupRenderBatchV1::new(
            point(-3.0, 4.0),
            vec![
                CompactGroupRenderOpV1::Ellipse(
                    EllipseOp::new(
                        point(8.0, -12.0),
                        size(5.0),
                        size(3.0),
                        45.0,
                        Some(size(1.0)),
                        Some(paint("112233")),
                        Some(paint("aabbcc")),
                        10,
                    )
                    .expect("ellipse"),
                ),
                CompactGroupRenderOpV1::Line(
                    LineOp::new(
                        point(0.0, 0.0),
                        point(1.0, 1.0),
                        size(1.0),
                        paint("112233"),
                        20,
                    )
                    .expect("line"),
                ),
            ],
        )
        .expect("compact-group content"),
    );
    MoleculeRenderPlanV4::new(
        plan.provenance(),
        [plan.batches(), &[extra_batch]].concat(),
        vec![],
    )
    .expect("mixed plan")
}

fn styled_single_bond_plan(style: BondStyle) -> MoleculeRenderPlanV4 {
    let first = AtomRenderTarget::new(
        context(RecordKind::Atom, "styled-a", 0x31, 1),
        point(10.0, 20.0),
        AtomLabelFacts::new("N", None, 0, 0).expect("label"),
        TargetVisibility::Visible,
    )
    .expect("first atom");
    let second = AtomRenderTarget::new(
        context(RecordKind::Atom, "styled-b", 0x33, 3),
        point(50.0, 20.0),
        AtomLabelFacts::new("C", None, 0, 0).expect("label"),
        TargetVisibility::Visible,
    )
    .expect("second atom");
    let bond = BondRenderTarget::new(
        context(RecordKind::Bond, "styled-bond", 0x32, 2),
        RecordId::new(
            RecordKind::Atom,
            Identifier::new("styled-a").expect("identifier"),
        )
        .expect("record ID"),
        RecordId::new(
            RecordKind::Atom,
            Identifier::new("styled-b").expect("identifier"),
        )
        .expect("record ID"),
        style,
        TargetVisibility::Visible,
    )
    .expect("bond");
    build_atom_bond_plan(
        &AtomBondRenderRequest::new(
            RenderProvenance::new(RenderRevision::new(1).expect("revision"), [0x31; 32]),
            vec![first, second],
            vec![bond],
            AtomLabelFontProfile::new(FontFace::molecule_label(), size(10.0), paint("000000")),
            size(1.0),
            size(6.0),
            BondInkClearance::new(size(1.25)),
            paint("112233"),
        )
        .expect("request"),
        &metrics(),
    )
    .expect("styled plan")
}

fn element_children(tree: &Xot, node: Node) -> Vec<Node> {
    tree.children(node)
        .filter(|child| tree.element(*child).is_some())
        .collect()
}

fn element_name(tree: &Xot, node: Node) -> (&str, &str) {
    let name = tree.element(node).expect("known element").name();
    tree.name_ns_str(name)
}

fn attribute<'tree>(tree: &'tree Xot, node: Node, name: &str) -> Option<&'tree str> {
    tree.name(name)
        .and_then(|name_id| tree.get_attribute(node, name_id))
}

fn descendant_elements(tree: &Xot, node: Node) -> Vec<Node> {
    let mut elements = vec![node];
    for child in element_children(tree, node) {
        elements.extend(descendant_elements(tree, child));
    }
    elements
}

#[test]
fn svg_backend_maps_the_closed_v1_plan_in_source_and_paint_order() {
    let plan = mixed_plan();
    let document = render_plan_to_svg_v1(
        &plan,
        SvgViewportV1::new(-5.0, -10.0, 100.0, 80.0).expect("viewport"),
    )
    .expect("SVG lowering succeeds");
    let source = document.as_str();

    let mut tree = Xot::new();
    let parsed = tree.parse(source).expect("generated SVG parses with xot");
    let root = tree.document_element(parsed).expect("SVG root exists");
    let root_name = tree.element(root).expect("root is an element").name();
    assert_eq!(
        tree.name_ns_str(root_name),
        ("svg", "http://www.w3.org/2000/svg")
    );
    let view_box = tree.name("viewBox").expect("viewBox name is known");
    assert_eq!(tree.get_attribute(root, view_box), Some("-5 -10 100 80"));
    let batches = element_children(&tree, root);
    assert_eq!(batches.len(), 4);
    assert_eq!(
        batches
            .iter()
            .map(|batch| attribute(&tree, *batch, "data-ferrum-source-order"))
            .collect::<Vec<_>>(),
        vec![Some("1"), Some("2"), Some("3"), Some("4")]
    );
    assert_eq!(
        attribute(&tree, batches[0], "data-ferrum-space"),
        Some("atom-local")
    );
    assert_eq!(
        attribute(&tree, batches[0], "transform"),
        Some("translate(10 20)")
    );
    assert_eq!(
        attribute(&tree, batches[1], "data-ferrum-space"),
        Some("scene")
    );
    assert_eq!(attribute(&tree, batches[1], "transform"), None);

    let bond_line = element_children(&tree, batches[1]);
    assert_eq!(bond_line.len(), 1);
    assert_eq!(element_name(&tree, bond_line[0]).0, "line");
    assert_eq!(attribute(&tree, bond_line[0], "stroke"), Some("#112233"));
    assert_eq!(
        attribute(&tree, bond_line[0], "stroke-linecap"),
        Some("butt")
    );
    assert_eq!(
        attribute(&tree, bond_line[0], "stroke-linejoin"),
        Some("miter")
    );
    assert_eq!(
        attribute(&tree, bond_line[0], "stroke-miterlimit"),
        Some("4")
    );
    assert_eq!(attribute(&tree, bond_line[0], "fill"), Some("none"));

    let shapes = element_children(&tree, batches[3]);
    assert_eq!(
        shapes
            .iter()
            .map(|shape| element_name(&tree, *shape).0)
            .collect::<Vec<_>>(),
        vec!["ellipse", "line"]
    );
    assert_eq!(attribute(&tree, shapes[0], "fill"), Some("#aabbcc"));
    assert_eq!(attribute(&tree, shapes[1], "fill"), Some("none"));
    assert_eq!(attribute(&tree, shapes[1], "stroke"), Some("#112233"));
    assert_eq!(attribute(&tree, shapes[1], "stroke-width"), Some("1"));
    assert_eq!(
        attribute(&tree, shapes[0], "transform"),
        Some("rotate(45 8 -12)")
    );

    let all_elements = descendant_elements(&tree, root);
    let text = all_elements
        .iter()
        .copied()
        .find(|element| attribute(&tree, *element, "data-ferrum-operation") == Some("text"))
        .expect("text group");
    assert_eq!(attribute(&tree, text, "fill"), Some("#000000"));
    assert!(
        element_children(&tree, text)
            .iter()
            .all(|path| element_name(&tree, *path).0 == "path"
                && attribute(&tree, *path, "d").is_some_and(|data| !data.is_empty()))
    );
    assert!(
        all_elements
            .iter()
            .all(|element| attribute(&tree, *element, "font-family").is_none())
    );
}

#[test]
fn svg_backend_rejects_a_nonfinite_or_nonpositive_viewport_before_emitting() {
    assert!(matches!(
        SvgViewportV1::new(f64::NAN, 0.0, 10.0, 10.0),
        Err(SvgRenderError::InvalidViewport)
    ));
    assert!(matches!(
        SvgViewportV1::new(0.0, 0.0, 0.0, 10.0),
        Err(SvgRenderError::InvalidViewport)
    ));
}

#[test]
fn svg_backend_emits_explicit_styled_bond_geometry_without_dash_semantics() {
    let viewport = SvgViewportV1::new(0.0, 0.0, 80.0, 40.0).expect("viewport");
    let bold = render_plan_to_svg_v1(&styled_single_bond_plan(BondStyle::Bold), viewport)
        .expect("bold SVG")
        .into_string();
    assert!(bold.contains("<line data-ferrum-z=\"10\""));
    assert!(bold.contains("stroke-width=\"2\""));

    let dashed = render_plan_to_svg_v1(&styled_single_bond_plan(BondStyle::Dashed), viewport)
        .expect("dashed SVG")
        .into_string();
    assert!(dashed.matches("<line data-ferrum-z=").count() > 1);
    assert!(!dashed.contains("stroke-dasharray"));

    let wavy = render_plan_to_svg_v1(&styled_single_bond_plan(BondStyle::Wavy), viewport)
        .expect("wavy SVG")
        .into_string();
    assert!(wavy.contains("<path data-ferrum-z=\"10\""));
    assert!(wavy.contains("stroke-linecap=\"round\""));
    assert!(wavy.contains(" C"));
}

#[test]
fn svg_backend_lowers_a_filled_stroked_v2_scene_path() {
    let path = PathOpV3::new(
        vec![
            ScenePathCommandV3::MoveTo(point(2.0, 2.0)),
            ScenePathCommandV3::LineTo(point(12.0, 2.0)),
            ScenePathCommandV3::LineTo(point(7.0, 10.0)),
            ScenePathCommandV3::Close,
        ],
        Some(ScenePathStrokeV3::new(paint("112233"), size(1.0))),
        Some(paint("aabbcc")),
        0,
    )
    .expect("scene path");
    let plan = MoleculeRenderPlanV4::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [1; 32]),
        vec![RenderBatchV4::bond_target(
            target(0x15),
            1,
            BondRenderBatchV1::new(
                BondAttachmentAxisV1::new(point(2.0, 2.0), point(12.0, 2.0))
                    .expect("attachment axis"),
                vec![BondRenderOpV1::Path(path)],
            )
            .expect("path content"),
        )],
        vec![],
    )
    .expect("path plan");
    let source = render_plan_to_svg_v1(
        &plan,
        SvgViewportV1::new(0.0, 0.0, 20.0, 20.0).expect("viewport"),
    )
    .expect("SVG lowering");
    let mut tree = Xot::new();
    let parsed = tree.parse(source.as_str()).expect("SVG parses");
    let root = tree.document_element(parsed).expect("SVG root");
    let path = descendant_elements(&tree, root)
        .into_iter()
        .find(|element| {
            element_name(&tree, *element).0 == "path"
                && attribute(&tree, *element, "fill") == Some("#aabbcc")
        })
        .expect("source path");

    assert_eq!(attribute(&tree, path, "fill"), Some("#aabbcc"));
    assert_eq!(attribute(&tree, path, "stroke"), Some("#112233"));
}

#[test]
fn svg_backend_accepts_finite_extreme_text_geometry() {
    let atom = AtomRenderTarget::new(
        context(RecordKind::Atom, "svg-extreme", 0x16, 1),
        point(0.0, 0.0),
        AtomLabelFacts::new("N", None, 0, 0).expect("label facts"),
        TargetVisibility::Visible,
    )
    .expect("atom target");
    let font =
        AtomLabelFontProfile::new(FontFace::molecule_label(), size(1.0e307), paint("000000"));
    let request = AtomBondRenderRequest::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [14; 32]),
        vec![atom],
        vec![],
        font,
        size(1.0),
        size(6.0),
        BondInkClearance::new(size(1.25)),
        paint("112233"),
    )
    .expect("extreme render request");
    let plan = build_atom_bond_plan(&request, &metrics()).expect("finite extreme plan");

    let result = std::panic::catch_unwind(|| {
        render_plan_to_svg_v1(
            &plan,
            SvgViewportV1::new(0.0, 0.0, 10.0, 10.0).expect("viewport"),
        )
    });
    assert!(matches!(result, Ok(Ok(_))));
}

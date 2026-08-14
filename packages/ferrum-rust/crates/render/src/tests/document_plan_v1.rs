use xot::{Node, Xot};

use crate::*;

fn provenance(value: u8) -> RenderProvenance {
    RenderProvenance::new(RenderRevision::new(1).expect("test revision"), [value; 32])
}

fn identity(value: &str) -> DocumentRenderIdentityV1 {
    DocumentRenderIdentityV1::projection_local(value).expect("test projection key")
}

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("test point")
}

fn plus_text() -> DocumentTextOpV1 {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment).expect("verified Telex opens");
    let paint = Paint::rgb24(Rgb24::new("000000").expect("test color"));
    let layout = metrics
        .layout_centered_plus(PositiveFinite::new(12.0).expect("test size"), paint.clone())
        .expect("plus layout");
    DocumentTextOpV1::fixed(
        point(20.0, 30.0),
        layout.operation().clone(),
        layout.bounds(),
        Some(Paint::rgb24(Rgb24::new("ffffff").expect("test background"))),
    )
    .expect("document text")
}

fn presentation_text() -> DocumentTextOpV1 {
    let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
    let metrics = VerifiedTelexGlyphMetrics::new(&environment).expect("verified Telex opens");
    let paint = Paint::rgb24(Rgb24::new("112233").expect("test color"));
    let source_runs = vec![
        PresentationTextSourceRun::new("SO", TextScript::Baseline).expect("baseline run"),
        PresentationTextSourceRun::new("4", TextScript::Subscript).expect("subscript run"),
    ];
    let layout = metrics
        .layout_presentation_text(
            &source_runs,
            PositiveFinite::new(10.0).expect("test size"),
            paint,
        )
        .expect("presentation layout");
    DocumentTextOpV1::presentation(
        point(45.0, 30.0),
        layout.operation().clone(),
        layout.bounds(),
        Some(Paint::rgb24(
            Rgb24::new("ddeeff").expect("test presentation background"),
        )),
    )
    .expect("document presentation text")
}

fn page() -> RenderViewportV1 {
    RenderViewportV1::new(0.0, 0.0, 200.0, 100.0).expect("test page")
}

fn empty_molecule(provenance: RenderProvenance) -> MoleculeRenderPlan {
    MoleculeRenderPlan::new(provenance, vec![], vec![]).expect("empty molecule plan")
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

fn finite_attribute(tree: &Xot, node: Node, name: &str) -> f64 {
    attribute(tree, node, name)
        .expect("known SVG geometry attribute")
        .parse::<f64>()
        .expect("finite SVG geometry attribute")
}

#[test]
fn document_plan_preserves_mixed_root_order_page_and_named_exclusion() {
    let source = provenance(4);
    let plan = DocumentRenderPlanV1::new(
        source,
        page(),
        vec![
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                2,
                identity("ferrum-projection-local-v1/2"),
                DocumentRenderContentV1::Molecule(empty_molecule(source)),
            )),
            DocumentRenderOutcomeV1::Exclusion(
                DocumentRenderExclusionV1::new(
                    5,
                    identity("ferrum-projection-local-v1/5"),
                    "not_yet_lowered:arrow",
                )
                .expect("named exclusion"),
            ),
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                8,
                identity("ferrum-projection-local-v1/8"),
                DocumentRenderContentV1::Text(plus_text()),
            )),
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                12,
                identity("ferrum-projection-local-v1/12"),
                DocumentRenderContentV1::Text(presentation_text()),
            )),
        ],
    )
    .expect("mixed document plan");

    assert_eq!(plan.provenance(), source);
    assert_eq!(plan.page(), page());
    assert_eq!(
        plan.outcomes()
            .iter()
            .map(DocumentRenderOutcomeV1::source_order)
            .collect::<Vec<_>>(),
        vec![2, 5, 8, 12]
    );
    assert!(matches!(
        &plan.outcomes()[1],
        DocumentRenderOutcomeV1::Exclusion(exclusion)
            if exclusion.feature() == "not_yet_lowered:arrow"
    ));

    let svg = render_document_plan_to_svg_v1(&plan).expect("SVG lowering");
    let mut tree = Xot::new();
    let parsed = tree.parse(svg.artifact().as_str()).expect("SVG parses");
    let root = tree.document_element(parsed).expect("SVG root");
    let painted = element_children(&tree, root);
    assert_eq!(painted.len(), 3);
    assert!(element_children(&tree, painted[0]).is_empty());
    assert!(painted.iter().all(|node| {
        attribute(&tree, *node, "data-ferrum-document-source-order").is_none()
            && attribute(&tree, *node, "data-ferrum-document-identity").is_none()
    }));
    let text_roots = painted
        .iter()
        .copied()
        .filter_map(|painted_root| element_children(&tree, painted_root).into_iter().next())
        .filter(|text_root| {
            attribute(&tree, *text_root, "data-ferrum-document-operation") == Some("text")
        })
        .collect::<Vec<_>>();
    assert_eq!(text_roots.len(), 2);
    assert_eq!(
        text_roots
            .iter()
            .map(|node| attribute(&tree, *node, "transform"))
            .collect::<Vec<_>>(),
        vec![Some("translate(20 30)"), Some("translate(45 30)")]
    );
    let fixed_children = element_children(&tree, text_roots[0]);
    let presentation_children = element_children(&tree, text_roots[1]);
    let text_bounds = plan
        .outcomes()
        .iter()
        .filter_map(|outcome| match outcome {
            DocumentRenderOutcomeV1::Root(root) => match root.content() {
                DocumentRenderContentV1::Text(text) => Some(text.bounds()),
                DocumentRenderContentV1::Molecule(_) | DocumentRenderContentV1::Vector(_) => None,
            },
            DocumentRenderOutcomeV1::Exclusion(_) => None,
        })
        .collect::<Vec<_>>();
    for ((children, expected_background), bounds) in [
        (&fixed_children, "#ffffff"),
        (&presentation_children, "#ddeeff"),
    ]
    .into_iter()
    .zip(text_bounds)
    {
        assert_eq!(
            attribute(&tree, children[0], "data-ferrum-document-text-background"),
            Some("true")
        );
        assert_eq!(
            attribute(&tree, children[0], "fill"),
            Some(expected_background)
        );
        assert_eq!(finite_attribute(&tree, children[0], "x"), bounds.min_x());
        assert_eq!(finite_attribute(&tree, children[0], "y"), bounds.min_y());
        assert_eq!(
            finite_attribute(&tree, children[0], "width"),
            bounds.max_x() - bounds.min_x()
        );
        assert_eq!(
            finite_attribute(&tree, children[0], "height"),
            bounds.max_y() - bounds.min_y()
        );
    }
    for glyph_group in [fixed_children[1], presentation_children[1]] {
        assert_eq!(
            attribute(&tree, glyph_group, "data-ferrum-operation"),
            Some("text")
        );
        assert!(
            !element_children(&tree, glyph_group).is_empty(),
            "both preserved text variants emit verified glyph paths"
        );
    }
}

#[test]
fn document_plan_rejects_duplicate_order_identity_and_mixed_provenance() {
    let source = provenance(5);
    let text = DocumentRenderContentV1::Text(plus_text());
    let duplicate_order = DocumentRenderPlanV1::new(
        source,
        page(),
        vec![
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                2,
                identity("root-a"),
                text.clone(),
            )),
            DocumentRenderOutcomeV1::Exclusion(
                DocumentRenderExclusionV1::new(2, identity("root-b"), "arrow").expect("exclusion"),
            ),
        ],
    );
    assert!(matches!(
        duplicate_order,
        Err(RenderError::InvalidRequest(_))
    ));

    let duplicate_identity = DocumentRenderPlanV1::new(
        source,
        page(),
        vec![
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                2,
                identity("same"),
                text.clone(),
            )),
            DocumentRenderOutcomeV1::Exclusion(
                DocumentRenderExclusionV1::new(3, identity("same"), "arrow").expect("exclusion"),
            ),
        ],
    );
    assert!(matches!(
        duplicate_identity,
        Err(RenderError::InvalidRequest(_))
    ));

    let unordered = DocumentRenderPlanV1::new(
        source,
        page(),
        vec![
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                4,
                identity("root-four"),
                text.clone(),
            )),
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                3,
                identity("root-three"),
                text,
            )),
        ],
    );
    assert!(matches!(unordered, Err(RenderError::InvalidRequest(_))));

    let mixed_provenance = DocumentRenderPlanV1::new(
        source,
        page(),
        vec![DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
            2,
            identity("molecule"),
            DocumentRenderContentV1::Molecule(empty_molecule(provenance(6))),
        ))],
    );
    assert!(matches!(
        mixed_provenance,
        Err(RenderError::InvalidRequest(_))
    ));
}

#[test]
fn document_plan_rejects_invalid_page_before_svg_emission() {
    assert!(matches!(
        RenderViewportV1::new(0.0, 0.0, f64::INFINITY, 10.0),
        Err(RenderError::InvalidRequest(_))
    ));
}

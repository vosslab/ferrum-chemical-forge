use ferrum_document_projection::{
    ArrowHeadShapeV1, ArrowProjectionV1, DocumentObjectIdV1, PlusProjectionV1, Point3V1,
    PositiveFiniteV1, PresentationFactProvenanceV1, PresentationFillV1, PresentationFontFaceV1,
    PresentationFontV1, PresentationRecordKindV1, PresentationRootProjectionV1,
    PresentationStackProjectionV1, PresentationStrokeV1, PresentationTargetV1, Rgb24V1,
};

use crate::{
    PRESENTATION_PREVIEW_RENDER_PLAN_SCHEMA_V1, PRESENTATION_RENDER_PLAN_SCHEMA_V1, PathCommandV1,
    PresentationPreviewRenderRootV1, PresentationRenderRootV1, RenderPoint,
    lower_standard_plus_preview_v1, render_presentation_stack_v1,
};

#[test]
fn renderer_plan_preserves_targets_and_finite_arrow_bounds_without_paint_order() {
    let stack = stack(vec![arrow(2, false, true), arrow(7, false, false)]);

    let plan = render_presentation_stack_v1(&stack).expect("typed stack renders");

    assert_eq!(plan.schema(), PRESENTATION_RENDER_PLAN_SCHEMA_V1);
    assert_eq!(plan.revision(), 19);
    assert_eq!(plan.digest(), &[9; 32]);
    assert_eq!(
        plan.roots()
            .iter()
            .map(|root| root.target().document_object_id().clone())
            .collect::<Vec<_>>(),
        vec![
            DocumentObjectIdV1::from_entropy_bytes([2; 16]),
            DocumentObjectIdV1::from_entropy_bytes([7; 16]),
        ]
    );
    for root in plan.roots() {
        let bounds = root.bounds();
        assert!(
            [bounds.left(), bounds.top(), bounds.right(), bounds.bottom()]
                .into_iter()
                .all(f64::is_finite)
        );
    }
}

#[test]
fn normal_head_policy_changes_issued_arrow_operations() {
    let with_head = render_presentation_stack_v1(&stack(vec![arrow(2, false, true)]))
        .expect("headed arrow renders");
    let without_head = render_presentation_stack_v1(&stack(vec![arrow(2, false, false)]))
        .expect("headless arrow renders");

    let has_filled_head = |root: &PresentationRenderRootV1| {
        root.vector()
            .expect("arrow is vector-backed")
            .operations()
            .iter()
            .any(|operation| operation.fill().is_some())
    };
    assert!(has_filled_head(&with_head.roots()[0]));
    assert!(!has_filled_head(&without_head.roots()[0]));
}

#[test]
fn short_normal_arrow_retains_a_positive_shaft_and_proportional_heads() {
    let authored_shape = ArrowHeadShapeV1::new(8.0, 10.0, 3.0).expect("fixed head shape");
    let root = PresentationRootProjectionV1::Arrow {
        arrow: ArrowProjectionV1::normal(
            target(2, PresentationRecordKindV1::Arrow),
            vec![point(0.0, 0.0), point(1.0, 1.0)],
            authored_shape,
            true,
            true,
            stroke(),
        )
        .expect("noncollapsed short arrow is valid semantic input"),
    };

    let plan = render_presentation_stack_v1(&stack(vec![root]))
        .expect("short normal arrow receives renderer geometry");
    let operations = plan.roots()[0]
        .vector()
        .expect("arrow is vector-backed")
        .operations();
    let (axis_start, axis_end) = open_segment(
        operations[0]
            .commands()
            .expect("normal arrow axis is a path"),
    );
    assert!(distance(axis_start, axis_end) > 0.0);

    let head_commands = operations[1]
        .commands()
        .expect("normal arrow heads are a path");
    for commands in [&head_commands[..5], &head_commands[5..]] {
        let (tip, base_a, axis, base_b) = closed_head(commands);
        let base_midpoint = RenderPoint::new(
            (base_a.x() + base_b.x()) / 2.0,
            (base_a.y() + base_b.y()) / 2.0,
        )
        .expect("head midpoint is finite");
        let total_length = distance(tip, base_midpoint);
        let inset = distance(tip, axis);
        let half_width = distance(base_a, base_b) / 2.0;
        assert_ratio(
            inset / total_length,
            authored_shape.line_inset() / authored_shape.total_length(),
        );
        assert_ratio(
            half_width / total_length,
            authored_shape.half_width() / authored_shape.total_length(),
        );
    }
}

#[test]
fn standard_plus_preview_uses_the_committed_builtin_plus_rendering_without_identity() {
    let anchor = RenderPoint::new(18.0, 24.0).expect("finite preview anchor");
    let preview = lower_standard_plus_preview_v1(anchor).expect("standard Plus preview");
    let committed =
        render_presentation_stack_v1(&stack(vec![PresentationRootProjectionV1::Plus {
            plus: standard_plus(anchor),
        }]))
        .expect("standard Plus root renders");

    assert_eq!(preview.schema(), PRESENTATION_PREVIEW_RENDER_PLAN_SCHEMA_V1);
    let PresentationPreviewRenderRootV1::Plus {
        anchor: preview_anchor,
        operation,
        bounds,
        background,
    } = &preview.roots()[0]
    else {
        panic!("standard Plus preview must issue a preview-only Plus root");
    };
    let PresentationRenderRootV1::Plus {
        render,
        bounds: committed_bounds,
        ..
    } = &committed.roots()[0]
    else {
        panic!("committed Plus must issue a persistent Plus root");
    };
    assert_eq!(*preview_anchor, anchor);
    assert_eq!(operation, render.operation());
    assert_eq!(*bounds, *committed_bounds);
    assert_eq!(background, &None);
}

fn stack(roots: Vec<PresentationRootProjectionV1>) -> PresentationStackProjectionV1 {
    PresentationStackProjectionV1::new(19, [9; 32], roots, Vec::new(), Vec::new())
        .expect("root set has no bracket-pair contract")
}

fn arrow(source_order: u32, start_head: bool, end_head: bool) -> PresentationRootProjectionV1 {
    PresentationRootProjectionV1::Arrow {
        arrow: ArrowProjectionV1::normal(
            target(source_order, PresentationRecordKindV1::Arrow),
            vec![
                point(0.0, source_order.into()),
                point(40.0, source_order.into()),
            ],
            ArrowHeadShapeV1::new(8.0, 10.0, 3.0).expect("fixed head shape"),
            start_head,
            end_head,
            stroke(),
        )
        .expect("finite nondegenerate arrow"),
    }
}

fn target(source_order: u32, record_kind: PresentationRecordKindV1) -> PresentationTargetV1 {
    PresentationTargetV1::new(
        DocumentObjectIdV1::from_entropy_bytes([source_order as u8; 16]),
        record_kind,
    )
}

fn standard_plus(anchor: RenderPoint) -> PlusProjectionV1 {
    let target = PresentationTargetV1::new(
        DocumentObjectIdV1::from_entropy_bytes([0; 16]),
        PresentationRecordKindV1::Plus,
    );
    let font = PresentationFontV1::try_new(
        PresentationFontFaceV1::TelexRegularV1,
        PresentationFactProvenanceV1::Builtin,
        PositiveFiniteV1::new(14.0).expect("built-in Plus font size"),
        PresentationFactProvenanceV1::Builtin,
        Rgb24V1::new("#000000").expect("built-in Plus colour"),
        PresentationFactProvenanceV1::Builtin,
    )
    .expect("built-in Plus font");
    let background = PresentationFillV1::try_new(None, PresentationFactProvenanceV1::Builtin)
        .expect("built-in Plus background");
    PlusProjectionV1::try_new(
        target,
        Point3V1::new(anchor.x(), anchor.y(), 0.0).expect("finite preview anchor"),
        font,
        background,
    )
    .expect("standard Plus projection")
}

fn stroke() -> PresentationStrokeV1 {
    PresentationStrokeV1::new(
        Rgb24V1::new("#000000".to_owned()).expect("RGB"),
        PresentationFactProvenanceV1::Builtin,
        PositiveFiniteV1::new(1.0).expect("positive width"),
        PresentationFactProvenanceV1::Builtin,
    )
    .expect("built-in stroke")
}

fn point(x: f64, y: f64) -> Point3V1 {
    Point3V1::new(x, y, 0.0).expect("finite point")
}

fn open_segment(commands: &[PathCommandV1]) -> (RenderPoint, RenderPoint) {
    let [PathCommandV1::MoveTo(start), PathCommandV1::LineTo(end)] = commands else {
        panic!("normal arrow axis must contain one line segment");
    };
    (*start, *end)
}

fn closed_head(commands: &[PathCommandV1]) -> (RenderPoint, RenderPoint, RenderPoint, RenderPoint) {
    let [
        PathCommandV1::MoveTo(tip),
        PathCommandV1::LineTo(base_a),
        PathCommandV1::LineTo(axis),
        PathCommandV1::LineTo(base_b),
        PathCommandV1::Close,
    ] = commands
    else {
        panic!("normal arrow head must be one closed quadrilateral");
    };
    (*tip, *base_a, *axis, *base_b)
}

fn distance(first: RenderPoint, second: RenderPoint) -> f64 {
    (first.x() - second.x()).hypot(first.y() - second.y())
}

fn assert_ratio(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1.0e-12,
        "{actual} != {expected}"
    );
}

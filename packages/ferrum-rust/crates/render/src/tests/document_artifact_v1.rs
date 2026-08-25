use std::num::NonZeroU32;

use ferrum_document_projection::DocumentObjectIdV1;

use crate::*;

fn target(value: u8) -> RenderTarget {
    RenderTarget::document_object(DocumentObjectIdV1::from_entropy_bytes([value; 16]))
}

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("finite test point")
}

fn paint(value: &str) -> Paint {
    Paint::rgb24(Rgb24::new(value).expect("valid test color"))
}

fn width(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("positive test width")
}

fn presentation_text() -> DocumentTextOpV1 {
    let metrics =
        VerifiedTelexGlyphMetrics::new(&FerrumFontEnvironmentV1::load().expect("verified Telex"))
            .expect("Telex metrics");
    let source_runs = vec![
        PresentationTextSourceRun::new("Line one\nH", TextScript::Baseline)
            .expect("baseline source"),
        PresentationTextSourceRun::new("2", TextScript::Subscript).expect("subscript source"),
        PresentationTextSourceRun::new("O", TextScript::Baseline).expect("baseline source"),
    ];
    let layout = metrics
        .layout_presentation_text(&source_runs, width(18.0), paint("123456"))
        .expect("presentation layout");
    let first_line = &layout.operation().runs()[0];
    assert!(
        first_line.glyphs()[5].origin().x() > first_line.glyphs()[4].origin().x(),
        "the visible glyph after the space retains its supplied origin"
    );
    DocumentTextOpV1::presentation(
        point(10.0, 20.0),
        layout.operation().clone(),
        layout.bounds(),
        None,
    )
    .expect("presentation text")
}

fn plan() -> DocumentRenderPlanV1 {
    let vector = DocumentVectorRootV1::new(vec![
        DocumentVectorOpV1::path(
            vec![
                PathCommandV1::MoveTo(point(1.0, 2.0)),
                PathCommandV1::LineTo(point(12.0, 2.0)),
            ],
            Some(StrokeV1::new(paint("112233"), width(1.0))),
            None,
        )
        .expect("vector path"),
    ])
    .expect("vector root");
    DocumentRenderPlanV1::new(
        RenderProvenance::new(RenderRevision::new(3).expect("revision"), [41; 32]),
        RenderViewportV1::new(-5.0, 7.0, 40.0, 30.0).expect("page"),
        vec![
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                target(2),
                2,
                DocumentRenderContentV1::Vector(vector),
            )),
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                target(3),
                3,
                DocumentRenderContentV1::Text(presentation_text()),
            )),
            DocumentRenderOutcomeV1::Exclusion(
                DocumentRenderExclusionV1::new(target(9), 9, "profile_excluded:unsupported-root")
                    .expect("exclusion"),
            ),
        ],
    )
    .expect("document plan")
}

#[test]
fn whole_document_sinks_issue_the_same_plan_coverage_receipt() {
    let plan = plan();
    let svg = render_document_plan_to_svg_v1(&plan).expect("SVG lowering");
    let png = render_document_plan_to_png_v1(
        &plan,
        PngRenderRequestV1 {
            pixels: PngPixelSizeV1::new(
                NonZeroU32::new(40).expect("width"),
                NonZeroU32::new(30).expect("height"),
            ),
            background: PngBackgroundV1::Transparent,
            budget: PngOutputBudgetV1 {
                max_raw_rgba_bytes: 4_800,
                max_encoded_bytes: 16_384,
            },
        },
    )
    .expect("PNG lowering");
    let pdf = render_document_plan_to_pdf_v1(
        &plan,
        PdfRenderRequestV1 {
            output: PdfOutputBudgetV1::new(16_384).expect("budget"),
            complexity: PdfPlanComplexityBudgetV1 {
                max_plan_items: 100,
                max_draw_path_commands: 1_000,
                max_exclusion_report_bytes: 1_000,
            },
        },
    )
    .expect("PDF lowering");

    assert_eq!(svg.report(), png.report());
    assert_eq!(png.report(), pdf.report());
    assert_eq!(svg.report().provenance(), plan.provenance());
    assert_eq!(svg.report().page(), plan.page());
    assert_eq!(svg.report().exclusions().len(), 1);
    assert_eq!(svg.report().exclusions()[0].paint_order(), 9);
    assert_eq!(svg.report().exclusions()[0].target(), &target(9));
    assert_eq!(
        svg.report().exclusions()[0].feature(),
        "profile_excluded:unsupported-root"
    );
}

#[test]
fn svg_completed_artifact_budget_accepts_the_result_or_withholds_it() {
    let plan = plan();
    let completed = render_document_plan_to_svg_v1(&plan).expect("SVG lowering");
    let exact_length = completed.artifact().as_str().len();

    let admitted = render_document_plan_to_svg_with_budget_v1(
        &plan,
        SvgOutputBudgetV1::new(exact_length).expect("exact output budget"),
    )
    .expect("exact completed length must be admitted");
    assert_eq!(admitted.report(), completed.report());

    let error = render_document_plan_to_svg_with_budget_v1(
        &plan,
        SvgOutputBudgetV1::new(exact_length - 1).expect("smaller output budget"),
    )
    .expect_err("oversized completed SVG must be withheld");
    assert!(matches!(
        error,
        SvgRenderError::OutputBudgetExceeded {
            limit,
            attempted
        } if limit + 1 == attempted
    ));
}

#[test]
fn svg_completed_artifact_budget_must_be_nonzero() {
    assert!(matches!(
        SvgOutputBudgetV1::new(0),
        Err(SvgRenderError::InvalidOutputBudget)
    ));
}

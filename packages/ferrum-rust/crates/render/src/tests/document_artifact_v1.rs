use std::num::NonZeroU32;

use crate::*;

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("finite test point")
}

fn paint(value: &str) -> Paint {
    Paint::rgb24(Rgb24::new(value).expect("valid test color"))
}

fn width(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("positive test width")
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
                2,
                DocumentRenderIdentityV1::projection_local("painted").expect("identity"),
                DocumentRenderContentV1::Vector(vector),
            )),
            DocumentRenderOutcomeV1::Exclusion(
                DocumentRenderExclusionV1::new(
                    9,
                    DocumentRenderIdentityV1::durable("excluded").expect("identity"),
                    "profile_excluded:unsupported-root",
                )
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
    assert_eq!(svg.report().exclusions()[0].source_order(), 9);
    assert_eq!(svg.report().exclusions()[0].identity().as_str(), "excluded");
    assert_eq!(
        svg.report().exclusions()[0].feature(),
        "profile_excluded:unsupported-root"
    );
}

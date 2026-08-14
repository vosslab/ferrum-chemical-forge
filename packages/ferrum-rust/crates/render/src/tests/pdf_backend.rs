use crate::*;
use ferrum_core::{Identifier, RecordId, RecordKind};

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("test point")
}

fn paint(value: &str) -> Paint {
    Paint::rgb24(Rgb24::new(value).expect("test RGB"))
}

fn width(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("test positive width")
}

fn provenance(value: u8) -> RenderProvenance {
    RenderProvenance::new(RenderRevision::new(8).expect("test revision"), [value; 32])
}

fn document_plan(page: RenderViewportV1) -> DocumentRenderPlanV1 {
    let root = DocumentVectorRootV1::new(vec![
        DocumentVectorOpV1::path(
            vec![
                PathCommandV1::MoveTo(point(10.0, 20.0)),
                PathCommandV1::LineTo(point(30.0, 40.0)),
                PathCommandV1::LineTo(point(10.0, 40.0)),
                PathCommandV1::Close,
            ],
            Some(StrokeV1::new(paint("112233"), width(1.5))),
            Some(paint("aabbcc")),
        )
        .expect("filled path"),
        DocumentVectorOpV1::ellipse(
            point(50.0, 30.0),
            width(6.0),
            width(4.0),
            Some(StrokeV1::new(paint("334455"), width(2.0))),
            None,
        )
        .expect("ellipse"),
    ])
    .expect("vector root");
    DocumentRenderPlanV1::new(
        provenance(7),
        page,
        vec![
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                2,
                DocumentRenderIdentityV1::projection_local("pdf-root").expect("identity"),
                DocumentRenderContentV1::Vector(root),
            )),
            DocumentRenderOutcomeV1::Exclusion(
                DocumentRenderExclusionV1::new(
                    9,
                    DocumentRenderIdentityV1::durable("excluded-root").expect("identity"),
                    "not_yet_lowered:arrow",
                )
                .expect("exclusion"),
            ),
        ],
    )
    .expect("document plan")
}

fn ample_request() -> PdfRenderRequestV1 {
    PdfRenderRequestV1 {
        output: PdfOutputBudgetV1::new(1_000_000).expect("nonzero test budget"),
        complexity: PdfPlanComplexityBudgetV1 {
            max_plan_items: 1_000,
            max_draw_path_commands: 100_000,
            max_exclusion_report_bytes: 100_000,
        },
    }
}

#[test]
fn pdf_backend_lowers_telex_quadratics_and_rotated_molecule_ellipses_as_cubics() {
    let source = provenance(8);
    let target = RenderTarget::new(
        RecordId::from_source(
            RecordKind::Atom,
            &Identifier::new("pdf-ellipse").expect("test identifier"),
        ),
        1,
    );
    let molecule = MoleculeRenderPlan::new(
        source,
        vec![
            RenderBatch::new(
                target,
                BatchSpace::AtomLocal {
                    anchor: point(0.0, 0.0),
                },
                vec![RenderOp::Ellipse(
                    EllipseOp::new(
                        point(50.0, 30.0),
                        width(6.0),
                        width(4.0),
                        30.0,
                        Some(width(1.0)),
                        Some(paint("112233")),
                        Some(paint("aabbcc")),
                        1,
                    )
                    .expect("rotated ellipse"),
                )],
            )
            .expect("ellipse batch"),
        ],
        vec![],
    )
    .expect("molecule plan");
    let metrics =
        VerifiedTelexGlyphMetrics::new(&FerrumFontEnvironmentV1::load().expect("verified Telex"))
            .expect("Telex metrics");
    let text_layout = metrics
        .layout_presentation_text(
            &[PresentationTextSourceRun::new("O", TextScript::Baseline).expect("Telex source")],
            width(12.0),
            paint("000000"),
        )
        .expect("Telex layout");
    let text = DocumentTextOpV1::presentation(
        point(20.0, 15.0),
        text_layout.operation().clone(),
        text_layout.bounds(),
        None,
    )
    .expect("document text");
    let plan = DocumentRenderPlanV1::new(
        source,
        RenderViewportV1::new(0.0, 0.0, 120.0, 80.0).expect("page"),
        vec![
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                1,
                DocumentRenderIdentityV1::projection_local("pdf-molecule").expect("identity"),
                DocumentRenderContentV1::Molecule(molecule),
            )),
            DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
                2,
                DocumentRenderIdentityV1::projection_local("pdf-telex").expect("identity"),
                DocumentRenderContentV1::Text(text),
            )),
        ],
    )
    .expect("document plan");

    let source = String::from_utf8_lossy(
        render_document_plan_to_pdf_v1(&plan, ample_request())
            .expect("PDF lowering")
            .artifact()
            .as_bytes(),
    )
    .into_owned();
    assert!(
        source.matches(" c\n").count() > 4,
        "the rotated ellipse contributes four cubics and Telex adds an outline cubic"
    );
    assert!(!source.contains(" Tf"));
    assert!(!source.contains(" Tj"));
}

#[test]
fn pdf_backend_emits_one_vector_page_and_exact_reported_coverage() {
    let page = RenderViewportV1::new(-5.0, 20.0, 120.0, 80.0).expect("test page");
    let plan = document_plan(page);
    let result = render_document_plan_to_pdf_v1(&plan, ample_request()).expect("PDF lowering");
    let bytes = result.artifact().as_bytes();
    let source = String::from_utf8_lossy(bytes);

    assert!(bytes.starts_with(b"%PDF-"));
    assert!(source.contains("/Type /Catalog"));
    assert!(source.contains("/Type /Pages"));
    assert!(source.contains("/Type /Page"));
    assert!(source.contains("/MediaBox [0 0 120 80]"));
    assert!(source.contains("1 0 0 -1 5 100 cm"));
    assert!(source.contains("0 J"));
    assert!(source.contains("0 j"));
    assert!(source.contains("4 M"));
    assert!(source.contains("B*"));
    assert!(source.contains("S"));
    assert!(source.contains(" m"));
    assert!(source.contains(" c"));
    assert!(source.contains("xref"));
    assert!(source.contains("trailer"));
    assert!(!source.contains(" Tf"));
    assert!(!source.contains(" Tj"));

    assert_eq!(result.report().provenance(), plan.provenance());
    assert_eq!(result.report().page(), page);
    assert_eq!(result.report().exclusions().len(), 1);
    assert_eq!(result.report().exclusions()[0].source_order(), 9);
    assert_eq!(
        result.report().exclusions()[0].identity().as_str(),
        "excluded-root"
    );
    assert_eq!(
        result.report().exclusions()[0].feature(),
        "not_yet_lowered:arrow"
    );
}

#[test]
fn pdf_backend_refuses_over_cap_output_without_a_document() {
    let plan = document_plan(RenderViewportV1::new(0.0, 0.0, 120.0, 80.0).expect("page"));
    let successful = render_document_plan_to_pdf_v1(&plan, ample_request()).expect("known PDF");
    let limit = successful.artifact().as_bytes().len() - 1;
    let request = PdfRenderRequestV1 {
        output: PdfOutputBudgetV1::new(limit).expect("nonzero smaller cap"),
        ..ample_request()
    };

    assert!(matches!(
        render_document_plan_to_pdf_v1(&plan, request),
        Err(PdfRenderError::OutputBudgetExceeded { limit: actual_limit, attempted })
            if actual_limit == limit && attempted > limit
    ));
}

#[test]
fn pdf_backend_rejects_f64_geometry_that_cannot_be_written_as_f32() {
    let plan =
        document_plan(RenderViewportV1::new(f64::MAX, 0.0, 120.0, 80.0).expect("finite page"));

    assert!(matches!(
        render_document_plan_to_pdf_v1(&plan, ample_request()),
        Err(PdfRenderError::NonFiniteGeometry)
    ));
}

#[test]
fn pdf_backend_rejects_nonzero_f64_geometry_that_underflows_f32() {
    let plan = document_plan(
        RenderViewportV1::new(f64::from_bits(1), 0.0, 120.0, 80.0).expect("finite page"),
    );

    assert!(matches!(
        render_document_plan_to_pdf_v1(&plan, ample_request()),
        Err(PdfRenderError::NonFiniteGeometry)
    ));
}

#[test]
fn pdf_backend_requires_an_explicit_nonzero_completed_artifact_cap() {
    assert!(matches!(
        PdfOutputBudgetV1::new(0),
        Err(PdfRenderError::InvalidOutputBudget)
    ));
}

#[test]
fn pdf_backend_rejects_a_vector_path_above_its_explicit_command_limit() {
    let plan = document_plan(RenderViewportV1::new(0.0, 0.0, 120.0, 80.0).expect("page"));
    let request = PdfRenderRequestV1 {
        complexity: PdfPlanComplexityBudgetV1 {
            max_draw_path_commands: 2,
            ..ample_request().complexity
        },
        ..ample_request()
    };

    assert!(matches!(
        render_document_plan_to_pdf_v1(&plan, request),
        Err(PdfRenderError::ComplexityLimitExceeded {
            resource: PdfComplexityResourceV1::DrawPathCommands,
            limit: 2,
            observed: 4,
        })
    ));
}

#[test]
fn pdf_backend_rejects_telex_outline_above_zero_command_limit() {
    let metrics =
        VerifiedTelexGlyphMetrics::new(&FerrumFontEnvironmentV1::load().expect("verified Telex"))
            .expect("Telex metrics");
    let layout = metrics
        .layout_presentation_text(
            &[PresentationTextSourceRun::new("O", TextScript::Baseline).expect("Telex source")],
            width(12.0),
            paint("000000"),
        )
        .expect("Telex layout");
    let text = DocumentTextOpV1::presentation(
        point(0.0, 0.0),
        layout.operation().clone(),
        layout.bounds(),
        None,
    )
    .expect("document text");
    let plan = DocumentRenderPlanV1::new(
        provenance(5),
        RenderViewportV1::new(0.0, 0.0, 40.0, 30.0).expect("page"),
        vec![DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
            1,
            DocumentRenderIdentityV1::projection_local("telex").expect("identity"),
            DocumentRenderContentV1::Text(text),
        ))],
    )
    .expect("document plan");
    let request = PdfRenderRequestV1 {
        complexity: PdfPlanComplexityBudgetV1 {
            max_draw_path_commands: 0,
            ..ample_request().complexity
        },
        ..ample_request()
    };

    assert!(matches!(
        render_document_plan_to_pdf_v1(&plan, request),
        Err(PdfRenderError::ComplexityLimitExceeded {
            resource: PdfComplexityResourceV1::DrawPathCommands,
            limit: 0,
            observed: 1,
        })
    ));
}

#[test]
fn pdf_backend_rejects_exclusion_report_data_above_its_explicit_byte_limit() {
    let plan = DocumentRenderPlanV1::new(
        provenance(6),
        RenderViewportV1::new(0.0, 0.0, 40.0, 30.0).expect("page"),
        vec![DocumentRenderOutcomeV1::Exclusion(
            DocumentRenderExclusionV1::new(
                1,
                DocumentRenderIdentityV1::durable("identity").expect("identity"),
                "not_yet_lowered",
            )
            .expect("exclusion"),
        )],
    )
    .expect("document plan");
    let request = PdfRenderRequestV1 {
        complexity: PdfPlanComplexityBudgetV1 {
            max_exclusion_report_bytes: 7,
            ..ample_request().complexity
        },
        ..ample_request()
    };

    assert!(matches!(
        render_document_plan_to_pdf_v1(&plan, request),
        Err(PdfRenderError::ComplexityLimitExceeded {
            resource: PdfComplexityResourceV1::ExclusionReportBytes,
            limit: 7,
            observed: 8,
        })
    ));
}

#[test]
fn pdf_backend_accepts_a_mixed_plan_with_caller_selected_complexity_limits() {
    let request = ample_request();
    let result = render_document_plan_to_pdf_v1(
        &document_plan(RenderViewportV1::new(0.0, 0.0, 120.0, 80.0).expect("page")),
        request,
    )
    .expect("PDF lowering");
    let observation = result.artifact().complexity();

    assert!(observation.plan_items() <= request.complexity.max_plan_items);
    assert!(observation.draw_path_commands() <= request.complexity.max_draw_path_commands);
    assert!(observation.exclusion_report_bytes() <= request.complexity.max_exclusion_report_bytes);
}

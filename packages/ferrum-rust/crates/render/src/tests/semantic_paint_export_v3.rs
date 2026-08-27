use std::num::NonZeroU32;

use crate::*;
use ferrum_document_projection::DocumentObjectIdV1;

fn point(x: f64, y: f64) -> RenderPoint {
    RenderPoint::new(x, y).expect("test point is finite")
}

fn size(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("test size is positive")
}

fn target() -> RenderTarget {
    RenderTarget::document_object(DocumentObjectIdV1::from_entropy_bytes([0x63; 16]))
}

fn plan(paint: RenderPaintV3) -> DocumentRenderPlanV1 {
    let ellipse =
        DocumentVectorOpV1::ellipse(point(5.0, 5.0), size(4.0), size(4.0), None, Some(paint))
            .expect("test ellipse");
    DocumentRenderPlanV1::new(
        RenderProvenance::new(RenderRevision::new(1).expect("revision"), [0x63; 32]),
        RenderViewportV1::new(0.0, 0.0, 10.0, 10.0).expect("page"),
        vec![DocumentRenderOutcomeV1::Root(DocumentRenderRootV1::new(
            target(),
            1,
            DocumentRenderContentV1::Vector(
                DocumentVectorRootV1::new(vec![ellipse]).expect("vector root"),
            ),
        ))],
    )
    .expect("document plan")
}

fn png_request() -> PngRenderRequestV1 {
    PngRenderRequestV1 {
        pixels: PngPixelSizeV1::new(
            NonZeroU32::new(10).expect("nonzero width"),
            NonZeroU32::new(10).expect("nonzero height"),
        ),
        background: PngBackgroundV1::Transparent,
        budget: PngOutputBudgetV1 {
            max_raw_rgba_bytes: 400,
            max_encoded_bytes: 4096,
        },
    }
}

fn pdf_request() -> PdfRenderRequestV1 {
    PdfRenderRequestV1 {
        output: PdfOutputBudgetV1::new(4096).expect("output budget"),
        complexity: PdfPlanComplexityBudgetV1 {
            max_plan_items: 32,
            max_draw_path_commands: 128,
            max_exclusion_report_bytes: 128,
        },
    }
}

#[test]
fn semantic_and_authored_paints_reach_svg_pdf_and_png_through_export_palette() {
    for (paint, expected_rgb, expected_png, expected_pdf) in [
        (
            RenderPaintV3::document_foreground(),
            "000000",
            [0, 0, 0],
            "0 0 0 rg",
        ),
        (
            RenderPaintV3::atom_number(),
            "0000c8",
            [0, 0, 200],
            "0 0 0.784",
        ),
        (
            RenderPaintV3::authored_rgb24(Rgb24::new("123456").expect("authored RGB")),
            "123456",
            [18, 52, 86],
            "0.070",
        ),
    ] {
        let plan = plan(paint);
        let svg = render_document_plan_to_svg_v1(&plan).expect("SVG render");
        assert!(svg.artifact().as_str().contains(expected_rgb));

        let pdf = render_document_plan_to_pdf_v1(&plan, pdf_request()).expect("PDF render");
        let pdf = String::from_utf8_lossy(pdf.artifact().as_bytes());
        assert!(pdf.contains(expected_pdf));

        let png = render_document_plan_to_png_v1(&plan, png_request()).expect("PNG render");
        let decoder = png::Decoder::new(std::io::Cursor::new(png.artifact().as_bytes()));
        let mut reader = decoder.read_info().expect("PNG info");
        let mut bytes = vec![0; reader.output_buffer_size().expect("PNG buffer size")];
        let info = reader.next_frame(&mut bytes).expect("PNG frame");
        let pixel = &bytes[((info.width * 5 + 5) * 4) as usize..][..4];
        assert_eq!(&pixel[..3], &expected_png);
        assert_eq!(pixel[3], 255);
    }
}

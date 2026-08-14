//! In-memory SVG serialization for the private V1 draw stream.

use std::fmt::Write;

use thiserror::Error;
use xot::Xot;

use crate::draw_stream_v1::{
    DrawEllipseV1, DrawMetadataV1, DrawPathCommandV1, DrawPathV1, DrawRectV1, DrawSinkV1,
    DrawStreamErrorV1, DrawStyleV1, lower_direct_glycosidic_haworth_plan_to_sink_v1,
    lower_document_plan_to_sink_v1, lower_molecule_plan_to_sink_v1,
};
use crate::{
    BatchSpace, DirectGlycosidicHaworthRenderPlanV1, DocumentRenderArtifactV1,
    DocumentRenderPlanV1, MoleculeRenderPlan, Paint, RenderPoint, RenderViewportV1,
};

const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";

/// A caller-owned finite scene viewport for an SVG V1 document.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SvgViewportV1 {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl SvgViewportV1 {
    /// Construct an explicit finite, positive SVG viewport.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Result<Self, SvgRenderError> {
        if ![x, y, width, height].into_iter().all(f64::is_finite) || width <= 0.0 || height <= 0.0 {
            return Err(SvgRenderError::InvalidViewport);
        }
        Ok(Self {
            x: canonical_zero(x),
            y: canonical_zero(y),
            width: canonical_zero(width),
            height: canonical_zero(height),
        })
    }
    #[must_use]
    pub const fn x(self) -> f64 {
        self.x
    }
    #[must_use]
    pub const fn y(self) -> f64 {
        self.y
    }
    #[must_use]
    pub const fn width(self) -> f64 {
        self.width
    }
    #[must_use]
    pub const fn height(self) -> f64 {
        self.height
    }
}

/// An owned, structurally validated SVG V1 document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SvgDocumentV1(String);

impl SvgDocumentV1 {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

/// A failure while lowering a validated plan to an owned SVG document.
#[derive(Debug, Error)]
pub enum SvgRenderError {
    #[error("SVG viewport must use finite coordinates and positive extents")]
    InvalidViewport,
    #[error("SVG geometry could not be represented as finite numeric text")]
    NonFiniteGeometry,
    #[error("could not parse verified Telex outline face: {0}")]
    Font(String),
    #[error("required Telex glyph {glyph_index} has no usable outline")]
    MissingGlyphOutline { glyph_index: u32 },
    #[error("generated SVG did not parse structurally: {0}")]
    Xml(#[from] xot::ParseError),
    #[error("could not allocate SVG output")]
    ResourceExhausted,
}

/// Lower one validated molecule plan into an in-memory SVG document.
pub fn render_plan_to_svg_v1(
    plan: &MoleculeRenderPlan,
    viewport: SvgViewportV1,
) -> Result<SvgDocumentV1, SvgRenderError> {
    let page = RenderViewportV1::new(
        viewport.x(),
        viewport.y(),
        viewport.width(),
        viewport.height(),
    )
    .map_err(|_| SvgRenderError::InvalidViewport)?;
    let mut sink = SvgSinkV1::default();
    lower_molecule_plan_to_sink_v1(plan, page, &mut sink).map_err(map_stream_error)?;
    sink.into_document()
}

/// Lower one direct Haworth private-profile plan into a structurally validated SVG.
pub fn render_direct_glycosidic_haworth_to_svg_v1(
    plan: &DirectGlycosidicHaworthRenderPlanV1,
    viewport: SvgViewportV1,
) -> Result<SvgDocumentV1, SvgRenderError> {
    let page = RenderViewportV1::new(
        viewport.x(),
        viewport.y(),
        viewport.width(),
        viewport.height(),
    )
    .map_err(|_| SvgRenderError::InvalidViewport)?;
    let mut sink = SvgSinkV1::default();
    lower_direct_glycosidic_haworth_plan_to_sink_v1(plan, page, &mut sink)
        .map_err(map_stream_error)?;
    sink.into_document()
}

/// Lower one validated, page-scoped document plan into an in-memory SVG document.
pub fn render_document_plan_to_svg_v1(
    plan: &DocumentRenderPlanV1,
) -> Result<DocumentRenderArtifactV1<SvgDocumentV1>, SvgRenderError> {
    let mut sink = SvgSinkV1::default();
    lower_document_plan_to_sink_v1(plan, &mut sink).map_err(map_stream_error)?;
    let document = sink.into_document()?;
    Ok(DocumentRenderArtifactV1::from_plan(document, plan))
}

fn map_stream_error(error: DrawStreamErrorV1<SvgSinkError>) -> SvgRenderError {
    match error {
        DrawStreamErrorV1::ResourceExhausted => SvgRenderError::ResourceExhausted,
        DrawStreamErrorV1::NonFiniteGeometry => SvgRenderError::NonFiniteGeometry,
        DrawStreamErrorV1::Font(message) => SvgRenderError::Font(message),
        DrawStreamErrorV1::MissingGlyphOutline { glyph_index } => {
            SvgRenderError::MissingGlyphOutline { glyph_index }
        }
        DrawStreamErrorV1::InvalidComposite => SvgRenderError::ResourceExhausted,
        DrawStreamErrorV1::Sink(SvgSinkError::ResourceExhausted) => {
            SvgRenderError::ResourceExhausted
        }
    }
}

#[derive(Default)]
struct SvgSinkV1 {
    output: String,
    current_molecule_batch: bool,
    document_text_open: bool,
}

#[derive(Debug)]
enum SvgSinkError {
    ResourceExhausted,
}

impl SvgSinkV1 {
    fn reserve(&mut self, additional: usize) -> Result<(), SvgSinkError> {
        self.output
            .try_reserve(additional)
            .map_err(|_| SvgSinkError::ResourceExhausted)
    }

    fn into_document(mut self) -> Result<SvgDocumentV1, SvgRenderError> {
        self.reserve(6)
            .map_err(|_| SvgRenderError::ResourceExhausted)?;
        self.output.push_str("</svg>");
        let mut tree = Xot::new();
        tree.parse(&self.output)?;
        Ok(SvgDocumentV1(self.output))
    }
}

impl DrawSinkV1 for SvgSinkV1 {
    type Error = SvgSinkError;

    fn begin_page(&mut self, page: RenderViewportV1) -> Result<(), Self::Error> {
        self.reserve(96)?;
        self.output.push_str("<svg xmlns=\"");
        self.output.push_str(SVG_NAMESPACE);
        self.output.push_str("\" viewBox=\"");
        for value in [page.x(), page.y(), page.width(), page.height()] {
            write_number(&mut self.output, value).map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push(' ');
        }
        self.output.pop();
        self.output.push_str("\">");
        Ok(())
    }

    fn begin_root(&mut self, _: u32, _: &str) -> Result<(), Self::Error> {
        self.reserve(3)?;
        self.output.push_str("<g>");
        Ok(())
    }
    fn end_root(&mut self) -> Result<(), Self::Error> {
        self.output.push_str("</g>");
        Ok(())
    }

    fn begin_molecule_batch(
        &mut self,
        source_order: u32,
        space: BatchSpace,
    ) -> Result<(), Self::Error> {
        self.reserve(96)?;
        self.output.push_str("<g data-ferrum-source-order=\"");
        write!(self.output, "{source_order}").expect("writing String cannot fail");
        self.output.push_str("\" data-ferrum-space=\"");
        match space {
            BatchSpace::AtomLocal { anchor } => {
                self.output.push_str("atom-local\" transform=\"translate(");
                write_point_pair(&mut self.output, anchor)
                    .map_err(|_| SvgSinkError::ResourceExhausted)?;
                self.output.push_str(")\">");
            }
            BatchSpace::Scene => self.output.push_str("scene\">"),
        }
        self.current_molecule_batch = true;
        Ok(())
    }
    fn end_molecule_batch(&mut self) -> Result<(), Self::Error> {
        self.output.push_str("</g>");
        self.current_molecule_batch = false;
        Ok(())
    }

    fn begin_document_text(&mut self) -> Result<(), Self::Error> {
        self.document_text_open = true;
        Ok(())
    }
    fn end_document_text(&mut self) -> Result<(), Self::Error> {
        self.output.push_str("</g>");
        self.document_text_open = false;
        Ok(())
    }
    fn begin_text_operation(&mut self, z: i32, paint: &Paint) -> Result<(), Self::Error> {
        self.reserve(80)?;
        self.output.push_str("<g data-ferrum-z=\"");
        write!(self.output, "{z}").expect("writing String cannot fail");
        self.output
            .push_str("\" data-ferrum-operation=\"text\" fill=\"#");
        self.output.push_str(paint.color().as_str());
        self.output.push_str("\">");
        Ok(())
    }
    fn end_text_operation(&mut self) -> Result<(), Self::Error> {
        self.output.push_str("</g>");
        Ok(())
    }
    fn save(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn concat_translate(&mut self, anchor: RenderPoint) -> Result<(), Self::Error> {
        if self.document_text_open && !self.current_molecule_batch {
            self.reserve(80)?;
            self.output
                .push_str("<g data-ferrum-document-operation=\"text\" transform=\"translate(");
            write_point_pair(&mut self.output, anchor)
                .map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push_str(")\">");
        }
        Ok(())
    }
    fn restore(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn fill_rect(
        &mut self,
        rect: DrawRectV1,
        paint: &Paint,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        self.reserve(192)?;
        self.output.push_str("<rect");
        match metadata {
            DrawMetadataV1::MoleculeMask { z } => {
                self.output.push_str(" data-ferrum-z=\"");
                write!(self.output, "{z}").expect("writing String cannot fail");
                self.output.push('"');
            }
            DrawMetadataV1::DocumentTextBackground => self
                .output
                .push_str(" data-ferrum-document-text-background=\"true\""),
            _ => {}
        }
        self.output.push_str(" x=\"");
        write_number(&mut self.output, rect.origin.x())
            .map_err(|_| SvgSinkError::ResourceExhausted)?;
        self.output.push_str("\" y=\"");
        write_number(&mut self.output, rect.origin.y())
            .map_err(|_| SvgSinkError::ResourceExhausted)?;
        self.output.push_str("\" width=\"");
        write_number(&mut self.output, rect.width.get())
            .map_err(|_| SvgSinkError::ResourceExhausted)?;
        self.output.push_str("\" height=\"");
        write_number(&mut self.output, rect.height.get())
            .map_err(|_| SvgSinkError::ResourceExhausted)?;
        self.output.push_str("\" fill=\"#");
        self.output.push_str(paint.color().as_str());
        self.output.push_str("\" stroke=\"none\"/>");
        Ok(())
    }

    fn draw_path(
        &mut self,
        path: &DrawPathV1,
        style: DrawStyleV1<'_>,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        if let (
            DrawMetadataV1::MoleculeLine { z },
            [
                DrawPathCommandV1::MoveTo(start),
                DrawPathCommandV1::LineTo(end),
            ],
            Some(stroke),
        ) = (metadata, path.commands.as_slice(), style.stroke)
        {
            self.reserve(192)?;
            self.output.push_str("<line data-ferrum-z=\"");
            write!(self.output, "{z}").expect("writing String cannot fail");
            self.output.push_str("\" x1=\"");
            write_number(&mut self.output, start.x())
                .map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push_str("\" y1=\"");
            write_number(&mut self.output, start.y())
                .map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push_str("\" x2=\"");
            write_number(&mut self.output, end.x()).map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push_str("\" y2=\"");
            write_number(&mut self.output, end.y()).map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push_str("\" stroke=\"#");
            self.output.push_str(stroke.paint.color().as_str());
            self.output.push_str("\" stroke-width=\"");
            write_number(&mut self.output, stroke.width.get())
                .map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push_str("\" stroke-linecap=\"");
            self.output.push_str(stroke.line_cap.svg_keyword());
            self.output.push_str("\" stroke-linejoin=\"");
            self.output.push_str(stroke.line_join.svg_keyword());
            self.output.push_str("\" stroke-miterlimit=\"");
            write_number(&mut self.output, stroke.miter_limit)
                .map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push_str("\" fill=\"none\"/>");
            return Ok(());
        }
        self.reserve(path.commands.len().saturating_mul(64).saturating_add(192))?;
        self.output.push_str("<path");
        match metadata {
            DrawMetadataV1::MoleculeLine { z } => {
                self.output.push_str(" data-ferrum-z=\"");
                write!(self.output, "{z}").expect("writing String cannot fail");
                self.output.push('"');
            }
            DrawMetadataV1::DocumentVectorPath => self
                .output
                .push_str(" data-ferrum-document-operation=\"path\""),
            DrawMetadataV1::DirectGlycosidicOrdinary => self
                .output
                .push_str(" data-ferrum-direct-glycosidic=\"ordinary\""),
            DrawMetadataV1::DirectGlycosidicQ1 => self
                .output
                .push_str(" data-ferrum-direct-glycosidic=\"q1\""),
            DrawMetadataV1::DirectGlycosidicW1 => self
                .output
                .push_str(" data-ferrum-direct-glycosidic=\"w1\""),
            _ => {}
        }
        self.output.push_str(" d=\"");
        write_path(&mut self.output, path).map_err(|_| SvgSinkError::ResourceExhausted)?;
        self.output.push_str("\" fill=\"");
        write_fill(&mut self.output, style.fill);
        self.output.push('"');
        if let Some(stroke) = style.stroke {
            self.output.push_str(" stroke=\"#");
            self.output.push_str(stroke.paint.color().as_str());
            self.output.push_str("\" stroke-width=\"");
            write_number(&mut self.output, stroke.width.get())
                .map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push_str("\" stroke-linecap=\"");
            self.output.push_str(stroke.line_cap.svg_keyword());
            self.output.push_str("\" stroke-linejoin=\"");
            self.output.push_str(stroke.line_join.svg_keyword());
            self.output.push_str("\" stroke-miterlimit=\"");
            write_number(&mut self.output, stroke.miter_limit)
                .map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push('"');
        } else {
            self.output.push_str(" stroke=\"none\"");
        }
        if let Some(fill_rule) = style.fill_rule {
            self.output.push_str(" fill-rule=\"");
            self.output.push_str(fill_rule.svg_keyword());
            self.output.push('"');
        }
        self.output.push_str("/>");
        Ok(())
    }

    fn draw_ellipse(
        &mut self,
        ellipse: DrawEllipseV1,
        style: DrawStyleV1<'_>,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        self.reserve(256)?;
        self.output.push_str("<ellipse");
        match metadata {
            DrawMetadataV1::MoleculeEllipse { z } => {
                self.output.push_str(" data-ferrum-z=\"");
                write!(self.output, "{z}").expect("writing String cannot fail");
                self.output.push('"');
            }
            DrawMetadataV1::DocumentVectorEllipse => self
                .output
                .push_str(" data-ferrum-document-operation=\"ellipse\""),
            _ => {}
        }
        self.output.push_str(" cx=\"");
        write_number(&mut self.output, ellipse.center.x())
            .map_err(|_| SvgSinkError::ResourceExhausted)?;
        self.output.push_str("\" cy=\"");
        write_number(&mut self.output, ellipse.center.y())
            .map_err(|_| SvgSinkError::ResourceExhausted)?;
        self.output.push_str("\" rx=\"");
        write_number(&mut self.output, ellipse.radius_x.get())
            .map_err(|_| SvgSinkError::ResourceExhausted)?;
        self.output.push_str("\" ry=\"");
        write_number(&mut self.output, ellipse.radius_y.get())
            .map_err(|_| SvgSinkError::ResourceExhausted)?;
        if ellipse.rotation_degrees != 0.0 {
            self.output.push_str("\" transform=\"rotate(");
            write_number(&mut self.output, ellipse.rotation_degrees)
                .map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push(' ');
            write_point_pair(&mut self.output, ellipse.center)
                .map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push(')');
        }
        self.output.push_str("\" fill=\"");
        write_fill(&mut self.output, style.fill);
        self.output.push('"');
        if let Some(stroke) = style.stroke {
            self.output.push_str(" stroke=\"#");
            self.output.push_str(stroke.paint.color().as_str());
            self.output.push_str("\" stroke-width=\"");
            write_number(&mut self.output, stroke.width.get())
                .map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push_str("\" stroke-linecap=\"");
            self.output.push_str(stroke.line_cap.svg_keyword());
            self.output.push_str("\" stroke-linejoin=\"");
            self.output.push_str(stroke.line_join.svg_keyword());
            self.output.push_str("\" stroke-miterlimit=\"");
            write_number(&mut self.output, stroke.miter_limit)
                .map_err(|_| SvgSinkError::ResourceExhausted)?;
            self.output.push('"');
        } else {
            self.output.push_str(" stroke=\"none\"");
        }
        self.output.push_str("/>");
        Ok(())
    }
    fn finish_page(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn write_path(output: &mut String, path: &DrawPathV1) -> Result<(), SvgRenderError> {
    for (index, command) in path.commands.iter().enumerate() {
        if index != 0 {
            output.push(' ');
        }
        match command {
            DrawPathCommandV1::MoveTo(point) => {
                output.push('M');
                write_point_pair(output, *point)?;
            }
            DrawPathCommandV1::LineTo(point) => {
                output.push('L');
                write_point_pair(output, *point)?;
            }
            DrawPathCommandV1::QuadraticTo { control, end } => {
                output.push('Q');
                write_point_pair(output, *control)?;
                output.push(' ');
                write_point_pair(output, *end)?;
            }
            DrawPathCommandV1::CubicTo {
                control_1,
                control_2,
                end,
            } => {
                output.push('C');
                write_point_pair(output, *control_1)?;
                output.push(' ');
                write_point_pair(output, *control_2)?;
                output.push(' ');
                write_point_pair(output, *end)?;
            }
            DrawPathCommandV1::Close => output.push('Z'),
        }
    }
    Ok(())
}

fn write_fill(output: &mut String, fill: Option<&Paint>) {
    match fill {
        Some(paint) => {
            output.push('#');
            output.push_str(paint.color().as_str());
        }
        None => output.push_str("none"),
    }
}
fn write_point_pair(output: &mut String, point: RenderPoint) -> Result<(), SvgRenderError> {
    write_number(output, point.x())?;
    output.push(' ');
    write_number(output, point.y())
}
fn write_number(output: &mut String, value: f64) -> Result<(), SvgRenderError> {
    if !value.is_finite() {
        return Err(SvgRenderError::NonFiniteGeometry);
    }
    write!(output, "{}", canonical_zero(value)).expect("writing String cannot fail");
    Ok(())
}
const fn canonical_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

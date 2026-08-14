//! In-memory, one-page vector PDF serialization for the private V1 draw stream.
//!
//! PDF V1 applies one page-level scene-down to PDF-up transform. Telex is lowered
//! through the verified outline stream, so this backend never emits PDF text or
//! font resources. `PdfOutputBudgetV1` is an accepted completed-artifact cap:
//! `pdf-writer` owns intermediate `Vec` allocations, and this module refuses to
//! return the completed bytes when their final length exceeds the caller's limit.

use pdf_writer::types::{LineCapStyle, LineJoinStyle};
use pdf_writer::{Content, Pdf, Rect, Ref};
use thiserror::Error;
use ttf_parser::{Face, GlyphId, OutlineBuilder};

use crate::draw_stream_v1::{
    DrawEllipseV1, DrawPathCommandV1, DrawPathV1, DrawRectV1, DrawSinkV1, DrawStreamErrorV1,
    DrawStyleV1, lower_document_plan_to_sink_v1,
};
use crate::verified_telex_glyph_metrics::is_verified_outlineless_whitespace_glyph;
use crate::{
    DocumentRenderArtifactV1, DocumentRenderContentV1, DocumentRenderOutcomeV1,
    DocumentRenderPlanV1, DocumentRenderReportV1, DocumentTextLayoutV1, DocumentVectorOpV1,
    FerrumFontEnvironmentV1, FerrumFontId, Paint, RenderOp, RenderPoint, RenderViewportV1,
};

const CATALOG_REFERENCE: i32 = 1;
const PAGES_REFERENCE: i32 = 2;
const PAGE_REFERENCE: i32 = 3;
const CONTENTS_REFERENCE: i32 = 4;
const PDF_STREAM_HARD_LIMIT: usize = i32::MAX as usize;
const CUBIC_CIRCLE_FACTOR: f64 = 0.552_284_749_830_793_6;

/// Caller-owned cap for the completed PDF artifact returned by this backend.
///
/// This does not promise to cap `pdf-writer`'s intermediate allocations. The
/// returned document is withheld unless its completed byte length is within this
/// explicit limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PdfOutputBudgetV1 {
    max_completed_bytes: usize,
}

impl PdfOutputBudgetV1 {
    /// Construct a nonzero completed-artifact cap.
    pub fn new(max_completed_bytes: usize) -> Result<Self, PdfRenderError> {
        if max_completed_bytes == 0 {
            return Err(PdfRenderError::InvalidOutputBudget);
        }
        Ok(Self {
            max_completed_bytes,
        })
    }

    /// Return the caller's completed-artifact cap.
    #[must_use]
    pub const fn max_completed_bytes(self) -> usize {
        self.max_completed_bytes
    }
}

/// Caller-selected structural admission limits for one PDF export.
///
/// Each field is an explicit policy selected by the export owner. Zero permits
/// only a plan whose corresponding measured work is zero; this module provides
/// no implicit renderer maximum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PdfPlanComplexityBudgetV1 {
    /// Maximum counted direct outcomes, batches, render operations, text runs,
    /// glyph placements, and document-vector operations.
    pub max_plan_items: usize,
    /// Maximum PDF path commands after line, ellipse, and Telex-outline lowering.
    pub max_draw_path_commands: usize,
    /// Maximum UTF-8 bytes cloned into the completed exclusion report.
    pub max_exclusion_report_bytes: usize,
}

/// Complete caller-owned policy required to request one PDF export.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PdfRenderRequestV1 {
    /// Publication cap applied after `pdf-writer` completes the artifact.
    pub output: PdfOutputBudgetV1,
    /// Structural preflight applied before report, sink, or writer allocation.
    pub complexity: PdfPlanComplexityBudgetV1,
}

/// A structural resource measured at the PDF export admission boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PdfComplexityResourceV1 {
    /// Direct outcomes and emitted-root traversal items.
    PlanItems,
    /// PDF path commands after backend-specific lowering.
    DrawPathCommands,
    /// UTF-8 bytes retained by named exclusion report entries.
    ExclusionReportBytes,
}

/// Exact structural work admitted for one successfully completed PDF export.
///
/// The values are observations of the accepted plan, not renderer-selected
/// thresholds or claims about allocations that occurred before export entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PdfRenderComplexityObservationV1 {
    plan_items: usize,
    draw_path_commands: usize,
    exclusion_report_bytes: usize,
}

impl PdfRenderComplexityObservationV1 {
    /// Return the counted direct traversal items.
    #[must_use]
    pub const fn plan_items(self) -> usize {
        self.plan_items
    }

    /// Return the counted PDF path commands after lowering.
    #[must_use]
    pub const fn draw_path_commands(self) -> usize {
        self.draw_path_commands
    }

    /// Return the UTF-8 bytes retained for named exclusion report entries.
    #[must_use]
    pub const fn exclusion_report_bytes(self) -> usize {
        self.exclusion_report_bytes
    }
}

/// Immutable owned bytes for one successfully lowered PDF document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PdfDocumentV1 {
    bytes: Vec<u8>,
    complexity: PdfRenderComplexityObservationV1,
}

impl PdfDocumentV1 {
    /// Borrow the complete PDF bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Consume the document into its complete PDF bytes.
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Return the admitted structural work for this completed export.
    #[must_use]
    pub const fn complexity(&self) -> PdfRenderComplexityObservationV1 {
        self.complexity
    }
}

/// A typed failure before the backend publishes a complete PDF artifact.
#[derive(Debug, Error)]
pub enum PdfRenderError {
    #[error("could not allocate private PDF draw geometry")]
    ResourceExhausted,
    #[error("PDF output budget must be nonzero")]
    InvalidOutputBudget,
    #[error("PDF {resource:?} complexity is {observed}, above the caller limit of {limit}")]
    ComplexityLimitExceeded {
        /// The resource whose caller-selected limit was exceeded.
        resource: PdfComplexityResourceV1,
        /// The caller-selected maximum for this resource.
        limit: usize,
        /// Exact measured work at the point of rejection.
        observed: usize,
    },
    #[error("PDF {resource:?} complexity counter overflowed")]
    ComplexityCountOverflow {
        /// The structural counter that could not be represented as `usize`.
        resource: PdfComplexityResourceV1,
    },
    #[error("PDF geometry is not finite or cannot be represented as finite f32")]
    NonFiniteGeometry,
    #[error("could not parse verified Telex outline face: {0}")]
    Font(String),
    #[error("required Telex glyph {glyph_index} has no usable outline")]
    MissingGlyphOutline { glyph_index: u32 },
    #[error("a PDF path command required a current point")]
    InvalidPath,
    #[error("the PDF content stream exceeds the format hard limit")]
    ContentStreamTooLarge,
    #[error(
        "generated PDF exceeds the completed-artifact cap of {limit} bytes ({attempted} bytes)"
    )]
    OutputBudgetExceeded { limit: usize, attempted: usize },
}

/// Lower one validated whole-page plan into an in-memory, outline-only PDF.
pub fn render_document_plan_to_pdf_v1(
    plan: &DocumentRenderPlanV1,
    request: PdfRenderRequestV1,
) -> Result<DocumentRenderArtifactV1<PdfDocumentV1>, PdfRenderError> {
    let complexity = measure_document_plan_complexity_v1(plan, request.complexity)?;
    let report = DocumentRenderReportV1::from_plan(plan);
    let page = plan.page();
    let mut sink = PdfSinkV1::new(page)?;
    lower_document_plan_to_sink_v1(plan, &mut sink).map_err(map_stream_error)?;
    let content = sink.finish()?;
    if content.len() > PDF_STREAM_HARD_LIMIT {
        return Err(PdfRenderError::ContentStreamTooLarge);
    }

    let catalog = checked_reference(CATALOG_REFERENCE)?;
    let pages = checked_reference(PAGES_REFERENCE)?;
    let page_reference = checked_reference(PAGE_REFERENCE)?;
    let contents = checked_reference(CONTENTS_REFERENCE)?;
    let estimated_capacity = content.len().saturating_add(512);
    let mut pdf = Pdf::with_capacity(estimated_capacity);
    pdf.catalog(catalog).pages(pages);
    pdf.pages(pages).kids([page_reference]).count(1);
    {
        let mut page_writer = pdf.page(page_reference);
        page_writer
            .parent(pages)
            .media_box(Rect::new(
                0.0,
                0.0,
                finite_f32(page.width())?,
                finite_f32(page.height())?,
            ))
            .contents(contents);
        page_writer.resources();
    }
    pdf.stream(contents, &content);
    let bytes = pdf.finish();
    if bytes.len() > request.output.max_completed_bytes() {
        return Err(PdfRenderError::OutputBudgetExceeded {
            limit: request.output.max_completed_bytes(),
            attempted: bytes.len(),
        });
    }

    Ok(DocumentRenderArtifactV1::new(
        PdfDocumentV1 { bytes, complexity },
        report,
    ))
}

/// Measure all PDF-side structural work without retaining a draw stream or report.
///
/// This direct borrowed traversal deliberately precedes report cloning, sink
/// construction, and `pdf-writer` allocation. It bounds only work after the
/// caller has already constructed a validated render plan.
fn measure_document_plan_complexity_v1(
    plan: &DocumentRenderPlanV1,
    budget: PdfPlanComplexityBudgetV1,
) -> Result<PdfRenderComplexityObservationV1, PdfRenderError> {
    let environment =
        FerrumFontEnvironmentV1::load().map_err(|error| PdfRenderError::Font(error.to_string()))?;
    let descriptor = environment.descriptor(FerrumFontId::TelexRegular);
    let face = Face::parse(descriptor.data(), 0)
        .map_err(|error| PdfRenderError::Font(error.to_string()))?;
    let mut counter = PdfComplexityCounterV1::new(budget);

    for outcome in plan.outcomes() {
        counter.add(PdfComplexityResourceV1::PlanItems, 1)?;
        match outcome {
            DocumentRenderOutcomeV1::Exclusion(exclusion) => {
                counter.add(
                    PdfComplexityResourceV1::ExclusionReportBytes,
                    exclusion.identity().as_str().len(),
                )?;
                counter.add(
                    PdfComplexityResourceV1::ExclusionReportBytes,
                    exclusion.feature().len(),
                )?;
            }
            DocumentRenderOutcomeV1::Root(root) => match root.content() {
                DocumentRenderContentV1::Molecule(molecule) => {
                    for batch in molecule.batches() {
                        counter.add(PdfComplexityResourceV1::PlanItems, 1)?;
                        for operation in batch.operations() {
                            counter.add(PdfComplexityResourceV1::PlanItems, 1)?;
                            measure_molecule_operation(operation, &face, &mut counter)?;
                        }
                    }
                }
                DocumentRenderContentV1::Text(text) => {
                    measure_document_text(text.operation(), &face, &mut counter)?;
                }
                DocumentRenderContentV1::Vector(vector) => {
                    for operation in vector.operations() {
                        counter.add(PdfComplexityResourceV1::PlanItems, 1)?;
                        match operation {
                            DocumentVectorOpV1::Path { commands, .. } => counter
                                .add(PdfComplexityResourceV1::DrawPathCommands, commands.len())?,
                            DocumentVectorOpV1::Ellipse { .. } => {
                                counter.add(PdfComplexityResourceV1::DrawPathCommands, 6)?
                            }
                        }
                    }
                }
            },
        }
    }
    Ok(counter.observation())
}

fn measure_molecule_operation(
    operation: &RenderOp,
    face: &Face<'_>,
    counter: &mut PdfComplexityCounterV1,
) -> Result<(), PdfRenderError> {
    match operation {
        RenderOp::Line(_) => counter.add(PdfComplexityResourceV1::DrawPathCommands, 2),
        RenderOp::Mask(_) => Ok(()),
        RenderOp::Ellipse(_) => counter.add(PdfComplexityResourceV1::DrawPathCommands, 6),
        RenderOp::Text(text) => measure_text_runs(text.runs(), face, counter),
    }
}

fn measure_document_text(
    text: &DocumentTextLayoutV1,
    face: &Face<'_>,
    counter: &mut PdfComplexityCounterV1,
) -> Result<(), PdfRenderError> {
    match text {
        DocumentTextLayoutV1::Fixed(text) => measure_text_runs(text.runs(), face, counter),
        DocumentTextLayoutV1::Presentation(text) => {
            measure_presentation_text_runs(text.runs(), face, counter)
        }
    }
}

trait MeasuredTextRunV1 {
    fn glyphs(&self) -> &[crate::GlyphPlacement];
}

impl MeasuredTextRunV1 for crate::TextRun {
    fn glyphs(&self) -> &[crate::GlyphPlacement] {
        self.glyphs()
    }
}

fn measure_text_runs<R: MeasuredTextRunV1>(
    runs: &[R],
    face: &Face<'_>,
    counter: &mut PdfComplexityCounterV1,
) -> Result<(), PdfRenderError> {
    for run in runs {
        counter.add(PdfComplexityResourceV1::PlanItems, 1)?;
        for glyph in run.glyphs() {
            counter.add(PdfComplexityResourceV1::PlanItems, 1)?;
            let glyph_id = u16::try_from(glyph.glyph_index()).map_err(|_| {
                PdfRenderError::MissingGlyphOutline {
                    glyph_index: glyph.glyph_index(),
                }
            })?;
            let mut builder = CountingOutlineBuilderV1 {
                counter,
                segments: false,
                error: None,
            };
            let outlined = face.outline_glyph(GlyphId(glyph_id), &mut builder);
            if let Some(error) = builder.error {
                return Err(error);
            }
            if outlined.is_none() || !builder.segments {
                return Err(PdfRenderError::MissingGlyphOutline {
                    glyph_index: glyph.glyph_index(),
                });
            }
        }
    }
    Ok(())
}

fn measure_presentation_text_runs(
    runs: &[crate::PresentationGlyphRun],
    face: &Face<'_>,
    counter: &mut PdfComplexityCounterV1,
) -> Result<(), PdfRenderError> {
    for run in runs {
        counter.add(PdfComplexityResourceV1::PlanItems, 1)?;
        for (scalar, glyph) in run.text().chars().zip(run.glyphs()) {
            counter.add(PdfComplexityResourceV1::PlanItems, 1)?;
            let glyph_id = u16::try_from(glyph.glyph_index()).map_err(|_| {
                PdfRenderError::MissingGlyphOutline {
                    glyph_index: glyph.glyph_index(),
                }
            })?;
            let mut builder = CountingOutlineBuilderV1 {
                counter,
                segments: false,
                error: None,
            };
            let outlined = face.outline_glyph(GlyphId(glyph_id), &mut builder);
            if let Some(error) = builder.error {
                return Err(error);
            }
            if outlined.is_none() || !builder.segments {
                if is_verified_outlineless_whitespace_glyph(face, scalar, glyph.glyph_index()) {
                    continue;
                }
                return Err(PdfRenderError::MissingGlyphOutline {
                    glyph_index: glyph.glyph_index(),
                });
            }
        }
    }
    Ok(())
}

struct PdfComplexityCounterV1 {
    budget: PdfPlanComplexityBudgetV1,
    plan_items: usize,
    draw_path_commands: usize,
    exclusion_report_bytes: usize,
}

impl PdfComplexityCounterV1 {
    const fn new(budget: PdfPlanComplexityBudgetV1) -> Self {
        Self {
            budget,
            plan_items: 0,
            draw_path_commands: 0,
            exclusion_report_bytes: 0,
        }
    }

    fn add(
        &mut self,
        resource: PdfComplexityResourceV1,
        amount: usize,
    ) -> Result<(), PdfRenderError> {
        let (observed, limit) = match resource {
            PdfComplexityResourceV1::PlanItems => {
                (&mut self.plan_items, self.budget.max_plan_items)
            }
            PdfComplexityResourceV1::DrawPathCommands => (
                &mut self.draw_path_commands,
                self.budget.max_draw_path_commands,
            ),
            PdfComplexityResourceV1::ExclusionReportBytes => (
                &mut self.exclusion_report_bytes,
                self.budget.max_exclusion_report_bytes,
            ),
        };
        *observed = observed
            .checked_add(amount)
            .ok_or(PdfRenderError::ComplexityCountOverflow { resource })?;
        if *observed > limit {
            return Err(PdfRenderError::ComplexityLimitExceeded {
                resource,
                limit,
                observed: *observed,
            });
        }
        Ok(())
    }

    const fn observation(&self) -> PdfRenderComplexityObservationV1 {
        PdfRenderComplexityObservationV1 {
            plan_items: self.plan_items,
            draw_path_commands: self.draw_path_commands,
            exclusion_report_bytes: self.exclusion_report_bytes,
        }
    }
}

struct CountingOutlineBuilderV1<'a> {
    counter: &'a mut PdfComplexityCounterV1,
    segments: bool,
    error: Option<PdfRenderError>,
}

impl CountingOutlineBuilderV1<'_> {
    fn command(&mut self, segment: bool) {
        if self.error.is_none() {
            if let Err(error) = self
                .counter
                .add(PdfComplexityResourceV1::DrawPathCommands, 1)
            {
                self.error = Some(error);
            } else if segment {
                self.segments = true;
            }
        }
    }
}

impl OutlineBuilder for CountingOutlineBuilderV1<'_> {
    fn move_to(&mut self, _: f32, _: f32) {
        self.command(false);
    }
    fn line_to(&mut self, _: f32, _: f32) {
        self.command(true);
    }
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
        self.command(true);
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        self.command(true);
    }
    fn close(&mut self) {
        self.command(false);
    }
}

fn checked_reference(value: i32) -> Result<Ref, PdfRenderError> {
    if value <= 0 {
        return Err(PdfRenderError::ContentStreamTooLarge);
    }
    Ok(Ref::new(value))
}

fn map_stream_error(error: DrawStreamErrorV1<PdfSinkError>) -> PdfRenderError {
    match error {
        DrawStreamErrorV1::ResourceExhausted => PdfRenderError::ResourceExhausted,
        DrawStreamErrorV1::NonFiniteGeometry => PdfRenderError::NonFiniteGeometry,
        DrawStreamErrorV1::Font(message) => PdfRenderError::Font(message),
        DrawStreamErrorV1::MissingGlyphOutline { glyph_index } => {
            PdfRenderError::MissingGlyphOutline { glyph_index }
        }
        DrawStreamErrorV1::InvalidComposite => PdfRenderError::ResourceExhausted,
        DrawStreamErrorV1::Sink(PdfSinkError::NonFiniteGeometry) => {
            PdfRenderError::NonFiniteGeometry
        }
        DrawStreamErrorV1::Sink(PdfSinkError::InvalidPath) => PdfRenderError::InvalidPath,
    }
}

#[derive(Debug)]
enum PdfSinkError {
    NonFiniteGeometry,
    InvalidPath,
}

struct PdfSinkV1 {
    content: Content,
    page_open: bool,
}

impl PdfSinkV1 {
    fn new(page: RenderViewportV1) -> Result<Self, PdfRenderError> {
        let transform_y =
            checked_add(page.y(), page.height()).map_err(|_| PdfRenderError::NonFiniteGeometry)?;
        let mut content = Content::new();
        content.save_state().transform([
            1.0,
            0.0,
            0.0,
            -1.0,
            finite_f32(-page.x())?,
            finite_f32(transform_y)?,
        ]);
        Ok(Self {
            content,
            page_open: true,
        })
    }

    fn finish(mut self) -> Result<Vec<u8>, PdfRenderError> {
        if self.page_open {
            self.content.restore_state();
            self.page_open = false;
        }
        Ok(self.content.finish().into_vec())
    }

    fn set_fill(&mut self, paint: &Paint) -> Result<(), PdfSinkError> {
        let [red, green, blue] = color_components(paint)?;
        self.content.set_fill_rgb(red, green, blue);
        Ok(())
    }

    fn set_stroke(
        &mut self,
        stroke: crate::draw_stream_v1::DrawStrokeV1<'_>,
    ) -> Result<(), PdfSinkError> {
        let [red, green, blue] = color_components(stroke.paint)?;
        self.content
            .set_stroke_rgb(red, green, blue)
            .set_line_width(
                finite_f32(stroke.width.get()).map_err(|_| PdfSinkError::NonFiniteGeometry)?,
            )
            .set_line_cap(match stroke.line_cap {
                crate::draw_stream_v1::DrawLineCapV1::Butt => LineCapStyle::ButtCap,
                crate::draw_stream_v1::DrawLineCapV1::Round => LineCapStyle::RoundCap,
            })
            .set_line_join(match stroke.line_join {
                crate::VectorStrokeLineJoinV1::Miter => LineJoinStyle::MiterJoin,
            })
            .set_miter_limit(
                finite_f32(stroke.miter_limit).map_err(|_| PdfSinkError::NonFiniteGeometry)?,
            );
        Ok(())
    }

    fn emit_path(&mut self, path: &DrawPathV1) -> Result<(), PdfSinkError> {
        let mut current = None;
        for command in &path.commands {
            match *command {
                DrawPathCommandV1::MoveTo(point) => {
                    self.content.move_to(point_x(point)?, point_y(point)?);
                    current = Some(point);
                }
                DrawPathCommandV1::LineTo(point) => {
                    self.content.line_to(point_x(point)?, point_y(point)?);
                    current = Some(point);
                }
                DrawPathCommandV1::QuadraticTo { control, end } => {
                    let start = current.ok_or(PdfSinkError::InvalidPath)?;
                    let control_1 = quadratic_control_1(start, control)?;
                    let control_2 = quadratic_control_2(end, control)?;
                    self.content.cubic_to(
                        point_x(control_1)?,
                        point_y(control_1)?,
                        point_x(control_2)?,
                        point_y(control_2)?,
                        point_x(end)?,
                        point_y(end)?,
                    );
                    current = Some(end);
                }
                DrawPathCommandV1::CubicTo {
                    control_1,
                    control_2,
                    end,
                } => {
                    self.content.cubic_to(
                        point_x(control_1)?,
                        point_y(control_1)?,
                        point_x(control_2)?,
                        point_y(control_2)?,
                        point_x(end)?,
                        point_y(end)?,
                    );
                    current = Some(end);
                }
                DrawPathCommandV1::Close => {
                    self.content.close_path();
                }
            }
        }
        Ok(())
    }

    fn paint_style(&mut self, style: DrawStyleV1<'_>) -> Result<(), PdfSinkError> {
        if let Some(fill) = style.fill {
            self.set_fill(fill)?;
        }
        if let Some(stroke) = style.stroke {
            self.set_stroke(stroke)?;
        }
        match (style.fill, style.stroke) {
            (Some(_), Some(_)) => self.content.fill_even_odd_and_stroke(),
            (Some(_), None) => self.content.fill_even_odd(),
            (None, Some(_)) => self.content.stroke(),
            (None, None) => return Err(PdfSinkError::InvalidPath),
        };
        Ok(())
    }

    fn emit_ellipse(&mut self, ellipse: DrawEllipseV1) -> Result<(), PdfSinkError> {
        let angle = checked_product(ellipse.rotation_degrees, std::f64::consts::PI / 180.0)?;
        let cosine = angle.cos();
        let sine = angle.sin();
        if !cosine.is_finite() || !sine.is_finite() {
            return Err(PdfSinkError::NonFiniteGeometry);
        }
        let rx = ellipse.radius_x.get();
        let ry = ellipse.radius_y.get();
        let kx = checked_product(rx, CUBIC_CIRCLE_FACTOR)?;
        let ky = checked_product(ry, CUBIC_CIRCLE_FACTOR)?;
        let points = [
            ellipse_point(ellipse.center, rx, 0.0, cosine, sine)?,
            ellipse_point(ellipse.center, rx, ky, cosine, sine)?,
            ellipse_point(ellipse.center, kx, ry, cosine, sine)?,
            ellipse_point(ellipse.center, 0.0, ry, cosine, sine)?,
            ellipse_point(ellipse.center, -kx, ry, cosine, sine)?,
            ellipse_point(ellipse.center, -rx, ky, cosine, sine)?,
            ellipse_point(ellipse.center, -rx, 0.0, cosine, sine)?,
            ellipse_point(ellipse.center, -rx, -ky, cosine, sine)?,
            ellipse_point(ellipse.center, -kx, -ry, cosine, sine)?,
            ellipse_point(ellipse.center, 0.0, -ry, cosine, sine)?,
            ellipse_point(ellipse.center, kx, -ry, cosine, sine)?,
            ellipse_point(ellipse.center, rx, -ky, cosine, sine)?,
            ellipse_point(ellipse.center, rx, 0.0, cosine, sine)?,
        ];
        self.content
            .move_to(point_x(points[0])?, point_y(points[0])?);
        for [control_1, control_2, end] in [
            [points[1], points[2], points[3]],
            [points[4], points[5], points[6]],
            [points[7], points[8], points[9]],
            [points[10], points[11], points[12]],
        ] {
            self.content.cubic_to(
                point_x(control_1)?,
                point_y(control_1)?,
                point_x(control_2)?,
                point_y(control_2)?,
                point_x(end)?,
                point_y(end)?,
            );
        }
        self.content.close_path();
        Ok(())
    }
}

impl DrawSinkV1 for PdfSinkV1 {
    type Error = PdfSinkError;

    fn begin_page(&mut self, _page: RenderViewportV1) -> Result<(), Self::Error> {
        Ok(())
    }

    fn begin_root(&mut self, _source_order: u32, _identity: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_root(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn begin_molecule_batch(
        &mut self,
        _source_order: u32,
        _space: crate::BatchSpace,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_molecule_batch(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn begin_document_text(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_document_text(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn begin_text_operation(&mut self, _z: i32, _paint: &Paint) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_text_operation(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn save(&mut self) -> Result<(), Self::Error> {
        self.content.save_state();
        Ok(())
    }

    fn concat_translate(&mut self, anchor: RenderPoint) -> Result<(), Self::Error> {
        self.content
            .transform([1.0, 0.0, 0.0, 1.0, point_x(anchor)?, point_y(anchor)?]);
        Ok(())
    }

    fn restore(&mut self) -> Result<(), Self::Error> {
        self.content.restore_state();
        Ok(())
    }

    fn fill_rect(
        &mut self,
        rect: DrawRectV1,
        paint: &Paint,
        _metadata: crate::draw_stream_v1::DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        self.set_fill(paint)?;
        self.content.rect(
            point_x(rect.origin)?,
            point_y(rect.origin)?,
            finite_f32(rect.width.get()).map_err(|_| PdfSinkError::NonFiniteGeometry)?,
            finite_f32(rect.height.get()).map_err(|_| PdfSinkError::NonFiniteGeometry)?,
        );
        self.content.fill_even_odd();
        Ok(())
    }

    fn draw_path(
        &mut self,
        path: &DrawPathV1,
        style: DrawStyleV1<'_>,
        _metadata: crate::draw_stream_v1::DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        self.emit_path(path)?;
        self.paint_style(style)
    }

    fn draw_ellipse(
        &mut self,
        ellipse: DrawEllipseV1,
        style: DrawStyleV1<'_>,
        _metadata: crate::draw_stream_v1::DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        self.emit_ellipse(ellipse)?;
        self.paint_style(style)
    }

    fn finish_page(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn color_components(paint: &Paint) -> Result<[f32; 3], PdfSinkError> {
    let value = paint.color().as_str();
    let component = |range| {
        u8::from_str_radix(&value[range], 16)
            .map(|component| f32::from(component) / 255.0)
            .map_err(|_| PdfSinkError::NonFiniteGeometry)
    };
    Ok([component(0..2)?, component(2..4)?, component(4..6)?])
}

fn quadratic_control_1(
    start: RenderPoint,
    control: RenderPoint,
) -> Result<RenderPoint, PdfSinkError> {
    checked_point(
        checked_add(
            start.x(),
            checked_product(2.0 / 3.0, control.x() - start.x())?,
        )?,
        checked_add(
            start.y(),
            checked_product(2.0 / 3.0, control.y() - start.y())?,
        )?,
    )
}

fn quadratic_control_2(
    end: RenderPoint,
    control: RenderPoint,
) -> Result<RenderPoint, PdfSinkError> {
    checked_point(
        checked_add(end.x(), checked_product(2.0 / 3.0, control.x() - end.x())?)?,
        checked_add(end.y(), checked_product(2.0 / 3.0, control.y() - end.y())?)?,
    )
}

fn ellipse_point(
    center: RenderPoint,
    x: f64,
    y: f64,
    cosine: f64,
    sine: f64,
) -> Result<RenderPoint, PdfSinkError> {
    checked_point(
        checked_add(
            center.x(),
            checked_add(checked_product(x, cosine)?, -checked_product(y, sine)?)?,
        )?,
        checked_add(
            center.y(),
            checked_add(checked_product(x, sine)?, checked_product(y, cosine)?)?,
        )?,
    )
}

fn point_x(point: RenderPoint) -> Result<f32, PdfSinkError> {
    finite_f32(point.x()).map_err(|_| PdfSinkError::NonFiniteGeometry)
}

fn point_y(point: RenderPoint) -> Result<f32, PdfSinkError> {
    finite_f32(point.y()).map_err(|_| PdfSinkError::NonFiniteGeometry)
}

fn finite_f32(value: f64) -> Result<f32, PdfRenderError> {
    let converted = value as f32;
    if converted.is_finite() && (value == 0.0 || converted != 0.0) {
        Ok(converted)
    } else {
        Err(PdfRenderError::NonFiniteGeometry)
    }
}

fn checked_add(first: f64, second: f64) -> Result<f64, PdfSinkError> {
    let value = first + second;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(PdfSinkError::NonFiniteGeometry)
    }
}

fn checked_product(first: f64, second: f64) -> Result<f64, PdfSinkError> {
    let value = first * second;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(PdfSinkError::NonFiniteGeometry)
    }
}

fn checked_point(x: f64, y: f64) -> Result<RenderPoint, PdfSinkError> {
    RenderPoint::new(x, y).map_err(|_| PdfSinkError::NonFiniteGeometry)
}

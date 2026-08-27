//! Bounded, in-memory PNG lowering for the private V1 draw stream.
//!
//! The raw-RGBA budget is a pre-allocation admission limit for the dominant
//! raster buffer. It is deliberately not described as a whole-process memory
//! cap: the PNG encoder has row buffers and compression workspace of its own.

use std::cell::RefCell;
use std::io::{self, Write};
use std::num::NonZeroU32;
use std::rc::Rc;

use thiserror::Error;

use crate::draw_stream_v1::{
    DrawEllipseV1, DrawMetadataV1, DrawPathCommandV1, DrawPathV1, DrawRectV1, DrawSinkV1,
    DrawStreamErrorV1, DrawStyleV1, lower_document_plan_to_sink_v1,
};
use crate::{
    DocumentRenderArtifactV1, DocumentRenderPlanV1, RenderPaintV3, RenderPoint, RenderViewportV1,
    Rgb24,
};

/// Exact caller-selected dimensions for a PNG artifact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PngPixelSizeV1 {
    width: NonZeroU32,
    height: NonZeroU32,
}

impl PngPixelSizeV1 {
    /// Construct nonzero device dimensions without choosing a DPI or rounding rule.
    #[must_use]
    pub const fn new(width: NonZeroU32, height: NonZeroU32) -> Self {
        Self { width, height }
    }
    #[must_use]
    pub const fn width(self) -> NonZeroU32 {
        self.width
    }
    #[must_use]
    pub const fn height(self) -> NonZeroU32 {
        self.height
    }
}

/// Explicit canvas clear policy; the document page has no implicit background.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PngBackgroundV1 {
    Transparent,
    Opaque(Rgb24),
}

/// Caller-owned logical output limits. There is intentionally no default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PngOutputBudgetV1 {
    pub max_raw_rgba_bytes: usize,
    pub max_encoded_bytes: usize,
}

/// One self-contained PNG render request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PngRenderRequestV1 {
    pub pixels: PngPixelSizeV1,
    pub background: PngBackgroundV1,
    pub budget: PngOutputBudgetV1,
}

/// Completed PNG bytes, published only after the encoder has finished within its cap.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PngDocumentV1(Vec<u8>);

impl PngDocumentV1 {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

/// Typed failures before a PNG artifact can be published.
#[derive(Debug, Error)]
pub enum PngRenderError {
    #[error("PNG dimensions exceed tiny-skia's supported raster domain")]
    RasterDimensionsUnsupported,
    #[error("PNG RGBA byte count overflowed for {width} by {height} pixels")]
    RasterByteCountOverflow { width: u32, height: u32 },
    #[error("PNG raster requires {required} bytes but the caller allowed {limit}")]
    RasterAllocationLimit { required: usize, limit: usize },
    #[error("PNG raster allocation failed")]
    RasterAllocationFailed,
    #[error("PNG encoded artifact exceeded the caller limit of {limit} bytes")]
    EncodedOutputLimit { limit: usize },
    #[error("PNG encoder failed: {0}")]
    Encoder(String),
    #[error("render geometry cannot be represented as a finite f32")]
    NonFiniteGeometry,
    #[error("could not parse verified Telex outline face: {0}")]
    Font(String),
    #[error("required Telex glyph {glyph_index} has no usable outline")]
    MissingGlyphOutline { glyph_index: u32 },
}

/// Lower a complete validated page to caller-bounded owned PNG bytes.
pub fn render_document_plan_to_png_v1(
    plan: &DocumentRenderPlanV1,
    request: PngRenderRequestV1,
) -> Result<DocumentRenderArtifactV1<PngDocumentV1>, PngRenderError> {
    let width = request.pixels.width.get();
    let height = request.pixels.height.get();
    if width > i32::MAX as u32 / 4 || height > i32::MAX as u32 {
        return Err(PngRenderError::RasterDimensionsUnsupported);
    }
    let required = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(PngRenderError::RasterByteCountOverflow { width, height })?;
    let required = usize::try_from(required)
        .map_err(|_| PngRenderError::RasterByteCountOverflow { width, height })?;
    if required > request.budget.max_raw_rgba_bytes {
        return Err(PngRenderError::RasterAllocationLimit {
            required,
            limit: request.budget.max_raw_rgba_bytes,
        });
    }
    let mut sink = PngSinkV1::new(request, plan.page())?;
    lower_document_plan_to_sink_v1(plan, &mut sink).map_err(map_draw_error)?;
    let document = sink.encode()?;
    Ok(DocumentRenderArtifactV1::from_plan(document, plan))
}

fn map_draw_error(error: DrawStreamErrorV1<PngRenderError>) -> PngRenderError {
    match error {
        DrawStreamErrorV1::ResourceExhausted => PngRenderError::RasterAllocationFailed,
        DrawStreamErrorV1::NonFiniteGeometry => PngRenderError::NonFiniteGeometry,
        DrawStreamErrorV1::Font(error) => PngRenderError::Font(error),
        DrawStreamErrorV1::MissingGlyphOutline { glyph_index } => {
            PngRenderError::MissingGlyphOutline { glyph_index }
        }
        DrawStreamErrorV1::InvalidComposite => PngRenderError::RasterAllocationFailed,
        DrawStreamErrorV1::Sink(error) => error,
    }
}

struct PngSinkV1 {
    pixmap: tiny_skia::Pixmap,
    page_transform: tiny_skia::Transform,
    translations: Vec<(f64, f64)>,
    current_translation: (f64, f64),
    encoded_limit: usize,
}

impl PngSinkV1 {
    fn new(request: PngRenderRequestV1, page: RenderViewportV1) -> Result<Self, PngRenderError> {
        let width = request.pixels.width.get();
        let height = request.pixels.height.get();
        let sx = f64::from(width) / page.width();
        let sy = f64::from(height) / page.height();
        let scale = sx.min(sy);
        let tx = (f64::from(width) - page.width() * scale) / 2.0 - page.x() * scale;
        let ty = (f64::from(height) - page.height() * scale) / 2.0 - page.y() * scale;
        let transform = finite_transform(scale, 0.0, 0.0, scale, tx, ty)?;
        let mut pixmap =
            tiny_skia::Pixmap::new(width, height).ok_or(PngRenderError::RasterAllocationFailed)?;
        if let PngBackgroundV1::Opaque(color) = request.background {
            pixmap.fill(to_color(&color)?);
        }
        Ok(Self {
            pixmap,
            page_transform: transform,
            translations: Vec::new(),
            current_translation: (0.0, 0.0),
            encoded_limit: request.budget.max_encoded_bytes,
        })
    }

    fn transform(&self) -> Result<tiny_skia::Transform, PngRenderError> {
        let (x, y) = self.current_translation;
        let translation = finite_transform(1.0, 0.0, 0.0, 1.0, x, y)?;
        Ok(self.page_transform.pre_concat(translation))
    }

    fn path(&self, path: &DrawPathV1) -> Result<tiny_skia::Path, PngRenderError> {
        let mut builder = tiny_skia::PathBuilder::new();
        for command in &path.commands {
            match *command {
                DrawPathCommandV1::MoveTo(point) => {
                    builder.move_to(f32_value(point.x())?, f32_value(point.y())?)
                }
                DrawPathCommandV1::LineTo(point) => {
                    builder.line_to(f32_value(point.x())?, f32_value(point.y())?)
                }
                DrawPathCommandV1::QuadraticTo { control, end } => builder.quad_to(
                    f32_value(control.x())?,
                    f32_value(control.y())?,
                    f32_value(end.x())?,
                    f32_value(end.y())?,
                ),
                DrawPathCommandV1::CubicTo {
                    control_1,
                    control_2,
                    end,
                } => builder.cubic_to(
                    f32_value(control_1.x())?,
                    f32_value(control_1.y())?,
                    f32_value(control_2.x())?,
                    f32_value(control_2.y())?,
                    f32_value(end.x())?,
                    f32_value(end.y())?,
                ),
                DrawPathCommandV1::Close => builder.close(),
            }
        }
        builder.finish().ok_or(PngRenderError::NonFiniteGeometry)
    }

    fn draw_style(
        &mut self,
        path: &tiny_skia::Path,
        style: DrawStyleV1<'_>,
    ) -> Result<(), PngRenderError> {
        self.draw_style_with_transform(path, style, self.transform()?)
    }

    fn draw_style_with_transform(
        &mut self,
        path: &tiny_skia::Path,
        style: DrawStyleV1<'_>,
        transform: tiny_skia::Transform,
    ) -> Result<(), PngRenderError> {
        if let Some(fill) = style.fill {
            self.pixmap.fill_path(
                path,
                &paint(fill)?,
                tiny_skia::FillRule::EvenOdd,
                transform,
                None,
            );
        }
        if let Some(stroke) = style.stroke {
            let raster_stroke = tiny_skia::Stroke {
                width: f32_value(stroke.width.get())?,
                miter_limit: f32_value(stroke.miter_limit)?,
                line_cap: match stroke.line_cap {
                    crate::draw_stream_v1::DrawLineCapV1::Butt => tiny_skia::LineCap::Butt,
                    crate::draw_stream_v1::DrawLineCapV1::Round => tiny_skia::LineCap::Round,
                },
                line_join: tiny_skia::LineJoin::Miter,
                dash: None,
            };
            self.pixmap
                .stroke_path(path, &paint(stroke.paint)?, &raster_stroke, transform, None);
        }
        Ok(())
    }

    fn encode(self) -> Result<PngDocumentV1, PngRenderError> {
        let width = self.pixmap.width();
        let output = BoundedOutput::new(self.encoded_limit);
        let encode_result = (|| -> Result<(), png::EncodingError> {
            let mut encoder = png::Encoder::new(output.writer(), width, self.pixmap.height());
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header()?;
            {
                let mut stream = writer.stream_writer()?;
                let row_len = usize::try_from(width)
                    .unwrap_or(usize::MAX)
                    .saturating_mul(4);
                let mut row = vec![0; row_len];
                for pixels in self
                    .pixmap
                    .pixels()
                    .chunks_exact(usize::try_from(width).unwrap_or(0))
                {
                    for (target, pixel) in row.as_chunks_mut::<4>().0.iter_mut().zip(pixels) {
                        let color = pixel.demultiply();
                        target.copy_from_slice(&[
                            color.red(),
                            color.green(),
                            color.blue(),
                            color.alpha(),
                        ]);
                    }
                    stream.write_all(&row)?;
                }
                stream.finish()?;
            }
            writer.finish()
        })();
        if output.exceeded() {
            return Err(PngRenderError::EncodedOutputLimit {
                limit: self.encoded_limit,
            });
        }
        encode_result.map_err(|error| PngRenderError::Encoder(error.to_string()))?;
        Ok(PngDocumentV1(output.into_bytes()))
    }
}

impl DrawSinkV1 for PngSinkV1 {
    type Error = PngRenderError;
    fn begin_page(&mut self, _: RenderViewportV1) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_root(
        &mut self,
        _: u32,
        _: &ferrum_document_projection::DocumentObjectIdV1,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_root(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_molecule_batch(&mut self, _: u32, _: crate::BatchSpace) -> Result<(), Self::Error> {
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
    fn begin_text_operation(&mut self, _: i32, _: &RenderPaintV3) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_text_operation(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn save(&mut self) -> Result<(), Self::Error> {
        self.translations.push(self.current_translation);
        Ok(())
    }
    fn concat_translate(&mut self, anchor: RenderPoint) -> Result<(), Self::Error> {
        self.current_translation.0 += anchor.x();
        self.current_translation.1 += anchor.y();
        if !self.current_translation.0.is_finite() || !self.current_translation.1.is_finite() {
            return Err(PngRenderError::NonFiniteGeometry);
        }
        Ok(())
    }
    fn restore(&mut self) -> Result<(), Self::Error> {
        self.current_translation = self
            .translations
            .pop()
            .ok_or(PngRenderError::NonFiniteGeometry)?;
        Ok(())
    }
    fn fill_rect(
        &mut self,
        rect: DrawRectV1,
        fill: &RenderPaintV3,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        let rect = tiny_skia::Rect::from_xywh(
            f32_value(rect.origin.x())?,
            f32_value(rect.origin.y())?,
            f32_value(rect.width.get())?,
            f32_value(rect.height.get())?,
        )
        .ok_or(PngRenderError::NonFiniteGeometry)?;
        let transform = self.transform()?;
        self.pixmap.fill_rect(rect, &paint(fill)?, transform, None);
        Ok(())
    }
    fn draw_path(
        &mut self,
        path: &DrawPathV1,
        style: DrawStyleV1<'_>,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        self.draw_style(&self.path(path)?, style)
    }
    fn draw_ellipse(
        &mut self,
        ellipse: DrawEllipseV1,
        style: DrawStyleV1<'_>,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        let mut builder = tiny_skia::PathBuilder::new();
        let x = ellipse.center.x() - ellipse.radius_x.get();
        let y = ellipse.center.y() - ellipse.radius_y.get();
        let rect = tiny_skia::Rect::from_xywh(
            f32_value(x)?,
            f32_value(y)?,
            f32_value(ellipse.radius_x.get() * 2.0)?,
            f32_value(ellipse.radius_y.get() * 2.0)?,
        )
        .ok_or(PngRenderError::NonFiniteGeometry)?;
        builder.push_oval(rect);
        let path = builder.finish().ok_or(PngRenderError::NonFiniteGeometry)?;
        let rotation = tiny_skia::Transform::from_rotate_at(
            f32_value(ellipse.rotation_degrees)?,
            f32_value(ellipse.center.x())?,
            f32_value(ellipse.center.y())?,
        );
        self.draw_style_with_transform(&path, style, self.transform()?.pre_concat(rotation))
    }
    fn finish_page(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn f32_value(value: f64) -> Result<f32, PngRenderError> {
    let converted = value as f32;
    if converted.is_finite() && (value == 0.0 || converted != 0.0) {
        Ok(converted)
    } else {
        Err(PngRenderError::NonFiniteGeometry)
    }
}
fn finite_transform(
    sx: f64,
    kx: f64,
    ky: f64,
    sy: f64,
    tx: f64,
    ty: f64,
) -> Result<tiny_skia::Transform, PngRenderError> {
    Ok(tiny_skia::Transform::from_row(
        f32_value(sx)?,
        f32_value(ky)?,
        f32_value(kx)?,
        f32_value(sy)?,
        f32_value(tx)?,
        f32_value(ty)?,
    ))
}
fn to_color(color: &Rgb24) -> Result<tiny_skia::Color, PngRenderError> {
    let value = color.as_str();
    let part = |range| {
        u8::from_str_radix(&value[range], 16).map_err(|_| PngRenderError::NonFiniteGeometry)
    };
    Ok(tiny_skia::Color::from_rgba8(
        part(0..2)?,
        part(2..4)?,
        part(4..6)?,
        255,
    ))
}
fn paint(paint: &RenderPaintV3) -> Result<tiny_skia::Paint<'static>, PngRenderError> {
    Ok(tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor(to_color(&paint.export_rgb())?),
        blend_mode: tiny_skia::BlendMode::SourceOver,
        anti_alias: true,
        colorspace: tiny_skia::ColorSpace::Linear,
        force_hq_pipeline: false,
    })
}

#[derive(Debug)]
struct OutputLimitExceeded;
impl std::fmt::Display for OutputLimitExceeded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PNG output limit exceeded")
    }
}
impl std::error::Error for OutputLimitExceeded {}
struct BoundedState {
    bytes: Vec<u8>,
    remaining: usize,
    exceeded: bool,
}
struct BoundedOutput(Rc<RefCell<BoundedState>>);
impl BoundedOutput {
    fn new(limit: usize) -> Self {
        Self(Rc::new(RefCell::new(BoundedState {
            bytes: Vec::new(),
            remaining: limit,
            exceeded: false,
        })))
    }
    fn writer(&self) -> BoundedWriter {
        BoundedWriter(Rc::clone(&self.0))
    }
    fn exceeded(&self) -> bool {
        self.0.borrow().exceeded
    }
    fn into_bytes(self) -> Vec<u8> {
        match Rc::try_unwrap(self.0) {
            Ok(state) => state.into_inner().bytes,
            Err(_) => Vec::new(),
        }
    }
}
struct BoundedWriter(Rc<RefCell<BoundedState>>);
impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut state = self.0.borrow_mut();
        if bytes.len() > state.remaining {
            state.exceeded = true;
            return Err(io::Error::other(OutputLimitExceeded));
        }
        state.bytes.extend_from_slice(bytes);
        state.remaining -= bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: u8) -> crate::RenderTarget {
        crate::RenderTarget::document_object(
            ferrum_document_projection::DocumentObjectIdV1::from_entropy_bytes([id; 16]),
        )
    }

    fn empty_plan() -> DocumentRenderPlanV1 {
        DocumentRenderPlanV1::new(
            crate::RenderProvenance::new(crate::RenderRevision::new(1).expect("revision"), [0; 32]),
            RenderViewportV1::new(0.0, 0.0, 10.0, 10.0).expect("page"),
            vec![],
        )
        .expect("empty plan")
    }

    fn painted_plan() -> DocumentRenderPlanV1 {
        let paint = RenderPaintV3::authored_rgb24(Rgb24::new("112233").expect("paint"));
        let ellipse = crate::DocumentVectorOpV1::ellipse(
            RenderPoint::new(5.0, 5.0).expect("center"),
            crate::PositiveFinite::new(2.0).expect("radius"),
            crate::PositiveFinite::new(1.0).expect("radius"),
            None,
            Some(paint),
        )
        .expect("ellipse");
        DocumentRenderPlanV1::new(
            crate::RenderProvenance::new(crate::RenderRevision::new(1).expect("revision"), [1; 32]),
            RenderViewportV1::new(0.0, 0.0, 10.0, 10.0).expect("page"),
            vec![crate::DocumentRenderOutcomeV1::Root(
                crate::DocumentRenderRootV1::new(
                    target(0x01),
                    1,
                    crate::DocumentRenderContentV1::Vector(
                        crate::DocumentVectorRootV1::new(vec![ellipse]).expect("root"),
                    ),
                ),
            )],
        )
        .expect("painted plan")
    }

    fn offset_painted_plan() -> DocumentRenderPlanV1 {
        let ellipse = crate::DocumentVectorOpV1::ellipse(
            RenderPoint::new(1.0, 1.0).expect("center"),
            crate::PositiveFinite::new(0.5).expect("radius"),
            crate::PositiveFinite::new(0.5).expect("radius"),
            None,
            Some(RenderPaintV3::authored_rgb24(
                Rgb24::new("112233").expect("paint"),
            )),
        )
        .expect("ellipse");
        DocumentRenderPlanV1::new(
            crate::RenderProvenance::new(crate::RenderRevision::new(1).expect("revision"), [2; 32]),
            RenderViewportV1::new(0.0, 0.0, 10.0, 20.0).expect("page"),
            vec![crate::DocumentRenderOutcomeV1::Root(
                crate::DocumentRenderRootV1::new(
                    target(0x02),
                    1,
                    crate::DocumentRenderContentV1::Vector(
                        crate::DocumentVectorRootV1::new(vec![ellipse]).expect("root"),
                    ),
                ),
            )],
        )
        .expect("offset painted plan")
    }

    fn request(
        width: u32,
        height: u32,
        background: PngBackgroundV1,
        raw_limit: usize,
        encoded_limit: usize,
    ) -> PngRenderRequestV1 {
        PngRenderRequestV1 {
            pixels: PngPixelSizeV1::new(
                NonZeroU32::new(width).expect("nonzero width"),
                NonZeroU32::new(height).expect("nonzero height"),
            ),
            background,
            budget: PngOutputBudgetV1 {
                max_raw_rgba_bytes: raw_limit,
                max_encoded_bytes: encoded_limit,
            },
        }
    }

    #[test]
    fn bounded_writer_refuses_whole_overflow_write() {
        let output = BoundedOutput::new(2);
        let mut writer = output.writer();
        assert_eq!(writer.write(b"ok").unwrap(), 2);
        assert!(writer.write(b"!").is_err());
        let state = output.0.borrow();
        assert_eq!(state.bytes, b"ok");
        assert_eq!(state.remaining, 0);
    }

    #[test]
    fn raw_cap_is_checked_before_raster_allocation() {
        let plan = empty_plan();
        let exact = render_document_plan_to_png_v1(
            &plan,
            request(2, 3, PngBackgroundV1::Transparent, 24, 4096),
        );
        assert!(exact.is_ok());
        let under = render_document_plan_to_png_v1(
            &plan,
            request(2, 3, PngBackgroundV1::Transparent, 23, 4096),
        );
        assert!(matches!(
            under,
            Err(PngRenderError::RasterAllocationLimit {
                required: 24,
                limit: 23
            })
        ));
    }

    #[test]
    fn encoded_limit_publishes_no_partial_document() {
        let result = render_document_plan_to_png_v1(
            &painted_plan(),
            request(2, 3, PngBackgroundV1::Transparent, 24, 0),
        );
        assert!(matches!(
            result,
            Err(PngRenderError::EncodedOutputLimit { limit: 0 })
        ));
    }

    #[test]
    fn transparent_and_opaque_pngs_decode_to_requested_dimensions() {
        for (background, expected_corner) in [
            (PngBackgroundV1::Transparent, [0, 0, 0, 0]),
            (
                PngBackgroundV1::Opaque(Rgb24::new("aabbcc").expect("color")),
                [170, 187, 204, 255],
            ),
        ] {
            let document =
                render_document_plan_to_png_v1(&empty_plan(), request(3, 2, background, 24, 4096))
                    .expect("PNG renders");
            let mut reader =
                png::Decoder::new(std::io::Cursor::new(document.artifact().as_bytes()))
                    .read_info()
                    .expect("PNG parses");
            assert_eq!((reader.info().width, reader.info().height), (3, 2));
            let mut pixels = vec![0; reader.output_buffer_size().expect("bounded test frame")];
            reader.next_frame(&mut pixels).expect("PNG frame decodes");
            assert_eq!(&pixels[..4], expected_corner);
        }
    }

    #[test]
    fn nonempty_plan_reaches_the_png_draw_sink() {
        let document = render_document_plan_to_png_v1(
            &painted_plan(),
            request(10, 10, PngBackgroundV1::Transparent, 400, 4096),
        )
        .expect("vector root lowers");
        let mut reader = png::Decoder::new(std::io::Cursor::new(document.artifact().as_bytes()))
            .read_info()
            .expect("PNG parses");
        let mut pixels = vec![0; reader.output_buffer_size().expect("bounded test frame")];
        reader.next_frame(&mut pixels).expect("PNG frame decodes");
        let center = (5 * 10 + 5) * 4;
        assert!(
            pixels[center + 3] > 0,
            "filled vector root reaches the raster"
        );
    }

    #[test]
    fn contained_page_transform_centers_a_nonmatching_aspect_page_without_stretching() {
        let document = render_document_plan_to_png_v1(
            &offset_painted_plan(),
            request(20, 20, PngBackgroundV1::Transparent, 1600, 4096),
        )
        .expect("vector root lowers");
        let mut reader = png::Decoder::new(std::io::Cursor::new(document.artifact().as_bytes()))
            .read_info()
            .expect("PNG parses");
        let mut pixels = vec![0; reader.output_buffer_size().expect("bounded test frame")];
        reader.next_frame(&mut pixels).expect("PNG frame decodes");
        let transformed_center = (20 + 6) * 4;
        let untransformed_center = (20 + 1) * 4;
        assert!(
            pixels[transformed_center + 3] > 0,
            "page is centered in pillarbox space"
        );
        assert_eq!(
            pixels[untransformed_center + 3],
            0,
            "page is not stretched to the canvas"
        );
    }

    #[test]
    fn mismatched_aspect_ratio_preserves_requested_canvas_dimensions() {
        let document = render_document_plan_to_png_v1(
            &empty_plan(),
            request(5, 2, PngBackgroundV1::Transparent, 40, 4096),
        )
        .expect("PNG renders");
        let reader = png::Decoder::new(std::io::Cursor::new(document.artifact().as_bytes()))
            .read_info()
            .expect("PNG parses");
        assert_eq!((reader.info().width, reader.info().height), (5, 2));
    }

    #[test]
    fn f64_geometry_outside_the_f32_domain_is_rejected() {
        assert!(matches!(
            f32_value(f64::MAX),
            Err(PngRenderError::NonFiniteGeometry)
        ));
    }

    #[test]
    fn nonzero_f64_geometry_that_underflows_f32_is_rejected() {
        assert!(matches!(
            f32_value(f64::from_bits(1)),
            Err(PngRenderError::NonFiniteGeometry)
        ));
    }
}

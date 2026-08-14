//! Conservative content fitting for an authenticated document render plan.

use thiserror::Error;

use crate::draw_stream_v1::{
    DrawEllipseV1, DrawMetadataV1, DrawPathCommandV1, DrawPathV1, DrawRectV1, DrawSinkV1,
    DrawStreamErrorV1, DrawStyleV1, lower_document_plan_to_sink_v1,
};
use crate::{BatchSpace, DocumentRenderPlanV1, RenderError, RenderPoint, RenderViewportV1};

/// Fit a document plan to the conservative painted bounds of its retained roots.
///
/// The pass consumes the same fully lowered paths, glyph outlines, masks, and
/// ellipses as every artifact backend. Curve control points and stroke profiles
/// form a conservative enclosure, so a fitted viewport may contain breathing
/// room but cannot clip verified content merely to make an exact-looking crop.
pub fn fit_document_render_plan_to_content_v1(
    plan: &DocumentRenderPlanV1,
) -> Result<DocumentRenderPlanV1, DocumentContentBoundsErrorV1> {
    let mut sink = ContentBoundsSinkV1::default();
    lower_document_plan_to_sink_v1(plan, &mut sink).map_err(map_draw_error)?;
    let bounds = sink
        .bounds
        .ok_or(DocumentContentBoundsErrorV1::EmptyContent)?;
    let page = RenderViewportV1::new(
        bounds.min_x,
        bounds.min_y,
        bounds.max_x - bounds.min_x,
        bounds.max_y - bounds.min_y,
    )
    .map_err(DocumentContentBoundsErrorV1::Plan)?;
    DocumentRenderPlanV1::new(plan.provenance(), page, plan.outcomes().to_vec())
        .map_err(DocumentContentBoundsErrorV1::Plan)
}

/// Failure while measuring or rebuilding one content-fitted document plan.
#[derive(Debug, Error)]
pub enum DocumentContentBoundsErrorV1 {
    /// The selected plan had no painted geometry.
    #[error("document render selection contains no paintable content")]
    EmptyContent,
    /// Private lowering could not reserve or represent its geometry.
    #[error("document content measurement could not reserve finite geometry")]
    Measurement,
    /// The verified Telex face could not be loaded or parsed.
    #[error("document content measurement could not load the verified font: {0}")]
    Font(String),
    /// A required verified glyph had no usable outline.
    #[error("document content measurement found no outline for glyph {glyph_index}")]
    MissingGlyphOutline { glyph_index: u32 },
    /// The fitted plan could not satisfy the checked render model.
    #[error(transparent)]
    Plan(#[from] RenderError),
}

fn map_draw_error(
    error: DrawStreamErrorV1<ContentBoundsSinkError>,
) -> DocumentContentBoundsErrorV1 {
    match error {
        DrawStreamErrorV1::ResourceExhausted
        | DrawStreamErrorV1::NonFiniteGeometry
        | DrawStreamErrorV1::InvalidComposite => DocumentContentBoundsErrorV1::Measurement,
        DrawStreamErrorV1::Font(message) => DocumentContentBoundsErrorV1::Font(message),
        DrawStreamErrorV1::MissingGlyphOutline { glyph_index } => {
            DocumentContentBoundsErrorV1::MissingGlyphOutline { glyph_index }
        }
        DrawStreamErrorV1::Sink(
            ContentBoundsSinkError::ResourceExhausted | ContentBoundsSinkError::NonFiniteGeometry,
        ) => DocumentContentBoundsErrorV1::Measurement,
    }
}

#[derive(Clone, Copy, Debug)]
struct ContentBoundsV1 {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl ContentBoundsV1 {
    fn point(point: RenderPoint) -> Self {
        Self {
            min_x: point.x(),
            min_y: point.y(),
            max_x: point.x(),
            max_y: point.y(),
        }
    }

    fn include_point(&mut self, point: RenderPoint) {
        self.min_x = self.min_x.min(point.x());
        self.min_y = self.min_y.min(point.y());
        self.max_x = self.max_x.max(point.x());
        self.max_y = self.max_y.max(point.y());
    }

    fn include_bounds(&mut self, other: Self) {
        self.min_x = self.min_x.min(other.min_x);
        self.min_y = self.min_y.min(other.min_y);
        self.max_x = self.max_x.max(other.max_x);
        self.max_y = self.max_y.max(other.max_y);
    }

    fn expanded(mut self, amount: f64) -> Self {
        self.min_x -= amount;
        self.min_y -= amount;
        self.max_x += amount;
        self.max_y += amount;
        self
    }
}

#[derive(Default)]
struct ContentBoundsSinkV1 {
    bounds: Option<ContentBoundsV1>,
    translation: Option<RenderPoint>,
    saved_translations: Vec<Option<RenderPoint>>,
}

impl ContentBoundsSinkV1 {
    fn translated(&self, point: RenderPoint) -> Result<RenderPoint, ContentBoundsSinkError> {
        let Some(translation) = self.translation else {
            return Ok(point);
        };
        RenderPoint::new(point.x() + translation.x(), point.y() + translation.y())
            .map_err(|_| ContentBoundsSinkError::NonFiniteGeometry)
    }

    fn include(&mut self, bounds: ContentBoundsV1) {
        if let Some(existing) = &mut self.bounds {
            existing.include_bounds(bounds);
        } else {
            self.bounds = Some(bounds);
        }
    }

    fn path_bounds(
        &self,
        path: &DrawPathV1,
    ) -> Result<Option<ContentBoundsV1>, ContentBoundsSinkError> {
        let mut bounds: Option<ContentBoundsV1> = None;
        let mut has_segment = false;
        for command in &path.commands {
            let mut include = |point: RenderPoint| -> Result<(), ContentBoundsSinkError> {
                let point = self.translated(point)?;
                if let Some(current) = &mut bounds {
                    current.include_point(point);
                } else {
                    bounds = Some(ContentBoundsV1::point(point));
                }
                Ok(())
            };
            match command {
                DrawPathCommandV1::MoveTo(point) => include(*point)?,
                DrawPathCommandV1::LineTo(point) => {
                    has_segment = true;
                    include(*point)?;
                }
                DrawPathCommandV1::QuadraticTo { control, end } => {
                    has_segment = true;
                    include(*control)?;
                    include(*end)?;
                }
                DrawPathCommandV1::CubicTo {
                    control_1,
                    control_2,
                    end,
                } => {
                    has_segment = true;
                    include(*control_1)?;
                    include(*control_2)?;
                    include(*end)?;
                }
                DrawPathCommandV1::Close => has_segment = true,
            }
        }
        Ok(has_segment.then_some(bounds).flatten())
    }
}

#[derive(Clone, Copy, Debug)]
enum ContentBoundsSinkError {
    ResourceExhausted,
    NonFiniteGeometry,
}

impl DrawSinkV1 for ContentBoundsSinkV1 {
    type Error = ContentBoundsSinkError;

    fn begin_page(&mut self, _: RenderViewportV1) -> Result<(), Self::Error> {
        Ok(())
    }

    fn begin_root(&mut self, _: u32, _: &str) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_root(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn begin_molecule_batch(&mut self, _: u32, _: BatchSpace) -> Result<(), Self::Error> {
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

    fn begin_text_operation(&mut self, _: i32, _: &crate::Paint) -> Result<(), Self::Error> {
        Ok(())
    }

    fn end_text_operation(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn save(&mut self) -> Result<(), Self::Error> {
        self.saved_translations
            .try_reserve(1)
            .map_err(|_| ContentBoundsSinkError::ResourceExhausted)?;
        self.saved_translations.push(self.translation);
        Ok(())
    }

    fn concat_translate(&mut self, anchor: RenderPoint) -> Result<(), Self::Error> {
        self.translation = Some(match self.translation {
            Some(current) => RenderPoint::new(current.x() + anchor.x(), current.y() + anchor.y())
                .map_err(|_| ContentBoundsSinkError::NonFiniteGeometry)?,
            None => anchor,
        });
        Ok(())
    }

    fn restore(&mut self) -> Result<(), Self::Error> {
        self.translation = self.saved_translations.pop().unwrap_or(None);
        Ok(())
    }

    fn fill_rect(
        &mut self,
        rect: DrawRectV1,
        _: &crate::Paint,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        let origin = self.translated(rect.origin)?;
        let max_x = origin.x() + rect.width.get();
        let max_y = origin.y() + rect.height.get();
        if !max_x.is_finite() || !max_y.is_finite() {
            return Err(ContentBoundsSinkError::NonFiniteGeometry);
        }
        self.include(ContentBoundsV1 {
            min_x: origin.x(),
            min_y: origin.y(),
            max_x,
            max_y,
        });
        Ok(())
    }

    fn draw_path(
        &mut self,
        path: &DrawPathV1,
        style: DrawStyleV1<'_>,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        if style.fill.is_none() && style.stroke.is_none() {
            return Ok(());
        }
        let Some(mut bounds) = self.path_bounds(path)? else {
            return Ok(());
        };
        if let Some(stroke) = style.stroke {
            bounds = bounds.expanded(stroke.width.get() * stroke.miter_limit.max(1.0) / 2.0);
        }
        if ![bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y]
            .into_iter()
            .all(f64::is_finite)
        {
            return Err(ContentBoundsSinkError::NonFiniteGeometry);
        }
        self.include(bounds);
        Ok(())
    }

    fn draw_ellipse(
        &mut self,
        ellipse: DrawEllipseV1,
        style: DrawStyleV1<'_>,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        if style.fill.is_none() && style.stroke.is_none() {
            return Ok(());
        }
        let center = self.translated(ellipse.center)?;
        let radians = ellipse.rotation_degrees.to_radians();
        let cos = radians.cos();
        let sin = radians.sin();
        let radius_x = ellipse.radius_x.get();
        let radius_y = ellipse.radius_y.get();
        let extent_x = ((radius_x * cos).powi(2) + (radius_y * sin).powi(2)).sqrt();
        let extent_y = ((radius_x * sin).powi(2) + (radius_y * cos).powi(2)).sqrt();
        let stroke = style
            .stroke
            .map(|value| value.width.get() / 2.0)
            .unwrap_or(0.0);
        let bounds = ContentBoundsV1 {
            min_x: center.x() - extent_x - stroke,
            min_y: center.y() - extent_y - stroke,
            max_x: center.x() + extent_x + stroke,
            max_y: center.y() + extent_y + stroke,
        };
        if ![bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y]
            .into_iter()
            .all(f64::is_finite)
        {
            return Err(ContentBoundsSinkError::NonFiniteGeometry);
        }
        self.include(bounds);
        Ok(())
    }

    fn finish_page(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

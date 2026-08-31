//! Private directional support derived from exact verified-Atkinson Hyperlegible Next outlines.

use ferrum_geometry::Vector2;
use ttf_parser::{Face, GlyphId, OutlineBuilder};

use crate::{GlyphBounds, GlyphPlacement, PositiveFinite, RenderError, RenderPoint, TextRun};

/// Exact directional support for one already-positioned visible glyph run.
///
/// The support is the farthest outline point along a requested unit direction.
/// It preserves curved-outline extrema rather than substituting a rectangular
/// glyph-bounds corner or a raster-derived approximation.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GlyphOutlineSupport {
    segments: Vec<OutlineSegment>,
}

impl GlyphOutlineSupport {
    /// Return the farthest outline projection along one finite direction.
    pub(crate) fn directional_extent(&self, direction: Vector2) -> f64 {
        self.segments
            .iter()
            .map(|segment| segment.directional_extent(direction))
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Return the tight axis-aligned envelope of the actual outline curves.
    pub(crate) fn axis_aligned_bounds(&self) -> Result<GlyphBounds, RenderError> {
        let positive_x = Vector2::new(1.0, 0.0).map_err(|_| {
            RenderError::InvalidRequest("molecule-label outline x direction is invalid".to_owned())
        })?;
        let negative_x = Vector2::new(-1.0, 0.0).map_err(|_| {
            RenderError::InvalidRequest(
                "molecule-label outline negative x direction is invalid".to_owned(),
            )
        })?;
        let positive_y = Vector2::new(0.0, 1.0).map_err(|_| {
            RenderError::InvalidRequest("molecule-label outline y direction is invalid".to_owned())
        })?;
        let negative_y = Vector2::new(0.0, -1.0).map_err(|_| {
            RenderError::InvalidRequest(
                "molecule-label outline negative y direction is invalid".to_owned(),
            )
        })?;
        GlyphBounds::new(
            -self.directional_extent(negative_x),
            -self.directional_extent(negative_y),
            self.directional_extent(positive_x),
            self.directional_extent(positive_y),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum OutlineSegment {
    Line {
        start: RenderPoint,
        end: RenderPoint,
    },
    Quadratic {
        start: RenderPoint,
        control: RenderPoint,
        end: RenderPoint,
    },
    Cubic {
        start: RenderPoint,
        control_1: RenderPoint,
        control_2: RenderPoint,
        end: RenderPoint,
    },
}

impl OutlineSegment {
    fn directional_extent(self, direction: Vector2) -> f64 {
        match self {
            Self::Line { start, end } => {
                projection(start, direction).max(projection(end, direction))
            }
            Self::Quadratic {
                start,
                control,
                end,
            } => quadratic_extent(start, control, end, direction),
            Self::Cubic {
                start,
                control_1,
                control_2,
                end,
            } => cubic_extent(start, control_1, control_2, end, direction),
        }
    }
}

/// Extract one run through the same face, glyph IDs, origins, and y-axis
/// conversion used by renderer draw-stream lowering.
pub(crate) fn molecule_label_run_outline_support(
    face: &Face<'_>,
    run: &TextRun,
    size: PositiveFinite,
) -> Result<GlyphOutlineSupport, RenderError> {
    molecule_label_glyph_outline_support(face, run.origin(), run.glyphs(), size, run.scale())
}

/// Extract exact outline segments for an already-positioned glyph sequence.
pub(crate) fn molecule_label_glyph_outline_support(
    face: &Face<'_>,
    run_origin: RenderPoint,
    glyphs: &[GlyphPlacement],
    size: PositiveFinite,
    scale: PositiveFinite,
) -> Result<GlyphOutlineSupport, RenderError> {
    let units_per_em = f64::from(face.units_per_em());
    let multiplier = size.get() * scale.get() / units_per_em;
    if !multiplier.is_finite() || multiplier <= 0.0 {
        return Err(RenderError::InvalidRequest(
            "Atkinson Hyperlegible Next outline support scale must be finite and positive"
                .to_owned(),
        ));
    }
    let mut segments = Vec::new();
    for glyph in glyphs {
        let glyph_index = u16::try_from(glyph.glyph_index()).map_err(|_| {
            RenderError::InvalidRequest(
                "Atkinson Hyperlegible Next outline support glyph identifier is not addressable"
                    .to_owned(),
            )
        })?;
        let origin = RenderPoint::new(
            run_origin.x() + glyph.origin().x(),
            run_origin.y() + glyph.origin().y(),
        )?;
        let mut builder = SupportOutlineBuilder::new(origin, multiplier);
        let outlined = face.outline_glyph(GlyphId(glyph_index), &mut builder);
        if let Some(error) = builder.error {
            return Err(error);
        }
        if outlined.is_none() || builder.segments.is_empty() {
            return Err(RenderError::InvalidRequest(format!(
                "Atkinson Hyperlegible Next glyph {} has no usable outline support",
                glyph.glyph_index()
            )));
        }
        segments.extend(builder.segments);
    }
    if segments.is_empty() {
        return Err(RenderError::InvalidRequest(
            "Atkinson Hyperlegible Next outline support requires visible segments".to_owned(),
        ));
    }
    Ok(GlyphOutlineSupport { segments })
}

fn projection(point: RenderPoint, direction: Vector2) -> f64 {
    point.x() * direction.x() + point.y() * direction.y()
}

fn quadratic_extent(
    start: RenderPoint,
    control: RenderPoint,
    end: RenderPoint,
    direction: Vector2,
) -> f64 {
    let first = projection(start, direction);
    let middle = projection(control, direction);
    let final_value = projection(end, direction);
    let denominator = first - 2.0 * middle + final_value;
    let mut maximum = first.max(final_value);
    if denominator != 0.0 {
        let parameter = (first - middle) / denominator;
        if (0.0..1.0).contains(&parameter) {
            maximum = maximum.max(quadratic_value(first, middle, final_value, parameter));
        }
    }
    maximum
}

fn quadratic_value(first: f64, middle: f64, final_value: f64, parameter: f64) -> f64 {
    let inverse = 1.0 - parameter;
    inverse * inverse * first
        + 2.0 * inverse * parameter * middle
        + parameter * parameter * final_value
}

fn cubic_extent(
    start: RenderPoint,
    control_1: RenderPoint,
    control_2: RenderPoint,
    end: RenderPoint,
    direction: Vector2,
) -> f64 {
    let first = projection(start, direction);
    let second = projection(control_1, direction);
    let third = projection(control_2, direction);
    let final_value = projection(end, direction);
    let derivative_quadratic = 3.0 * (-first + 3.0 * second - 3.0 * third + final_value);
    let derivative_linear = 2.0 * (3.0 * first - 6.0 * second + 3.0 * third);
    let derivative_constant = -3.0 * first + 3.0 * second;
    let mut maximum = first.max(final_value);
    for parameter in quadratic_roots(derivative_quadratic, derivative_linear, derivative_constant)
        .into_iter()
        .flatten()
    {
        if (0.0..1.0).contains(&parameter) {
            maximum = maximum.max(cubic_value(first, second, third, final_value, parameter));
        }
    }
    maximum
}

fn cubic_value(first: f64, second: f64, third: f64, final_value: f64, parameter: f64) -> f64 {
    let inverse = 1.0 - parameter;
    inverse.powi(3) * first
        + 3.0 * inverse.powi(2) * parameter * second
        + 3.0 * inverse * parameter.powi(2) * third
        + parameter.powi(3) * final_value
}

fn quadratic_roots(quadratic: f64, linear: f64, constant: f64) -> [Option<f64>; 2] {
    let coefficient_scale = quadratic.abs().max(linear.abs()).max(constant.abs());
    let near_zero = f64::EPSILON * coefficient_scale.max(1.0) * 16.0;
    if quadratic.abs() <= near_zero {
        if linear.abs() <= near_zero {
            return [None, None];
        }
        return [Some(-constant / linear), None];
    }
    let discriminant = linear * linear - 4.0 * quadratic * constant;
    if discriminant < 0.0 {
        return [None, None];
    }
    let root = discriminant.sqrt();
    [
        Some((-linear - root) / (2.0 * quadratic)),
        Some((-linear + root) / (2.0 * quadratic)),
    ]
}

struct SupportOutlineBuilder {
    origin: RenderPoint,
    multiplier: f64,
    contour_start: Option<RenderPoint>,
    current: Option<RenderPoint>,
    segments: Vec<OutlineSegment>,
    error: Option<RenderError>,
}

impl SupportOutlineBuilder {
    fn new(origin: RenderPoint, multiplier: f64) -> Self {
        Self {
            origin,
            multiplier,
            contour_start: None,
            current: None,
            segments: Vec::new(),
            error: None,
        }
    }

    fn point(&mut self, x: f32, y: f32) -> Option<RenderPoint> {
        if self.error.is_some() {
            return None;
        }
        match RenderPoint::new(
            self.origin.x() + f64::from(x) * self.multiplier,
            self.origin.y() - f64::from(y) * self.multiplier,
        ) {
            Ok(point) => Some(point),
            Err(error) => {
                self.error = Some(error);
                None
            }
        }
    }

    fn require_current(&mut self) -> Option<RenderPoint> {
        if let Some(current) = self.current {
            Some(current)
        } else {
            self.error = Some(RenderError::InvalidRequest(
                "Atkinson Hyperlegible Next outline segment has no current contour point"
                    .to_owned(),
            ));
            None
        }
    }
}

impl OutlineBuilder for SupportOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        if let Some(point) = self.point(x, y) {
            self.contour_start = Some(point);
            self.current = Some(point);
        }
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let start = self.require_current();
        let end = self.point(x, y);
        if let (Some(start), Some(end)) = (start, end) {
            self.segments.push(OutlineSegment::Line { start, end });
            self.current = Some(end);
        }
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let start = self.require_current();
        let control = self.point(x1, y1);
        let end = self.point(x, y);
        if let (Some(start), Some(control), Some(end)) = (start, control, end) {
            self.segments.push(OutlineSegment::Quadratic {
                start,
                control,
                end,
            });
            self.current = Some(end);
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let start = self.require_current();
        let control_1 = self.point(x1, y1);
        let control_2 = self.point(x2, y2);
        let end = self.point(x, y);
        if let (Some(start), Some(control_1), Some(control_2), Some(end)) =
            (start, control_1, control_2, end)
        {
            self.segments.push(OutlineSegment::Cubic {
                start,
                control_1,
                control_2,
                end,
            });
            self.current = Some(end);
        }
    }

    fn close(&mut self) {
        if let (Some(start), Some(end)) = (self.current, self.contour_start)
            && start != end
        {
            self.segments.push(OutlineSegment::Line { start, end });
        }
        self.current = self.contour_start;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> RenderPoint {
        RenderPoint::new(x, y).expect("test point")
    }

    fn direction(x: f64, y: f64) -> Vector2 {
        Vector2::new(x, y).expect("test direction")
    }

    #[test]
    fn curved_support_includes_interior_projection_extrema() {
        let quadratic = OutlineSegment::Quadratic {
            start: point(0.0, 0.0),
            control: point(2.0, 4.0),
            end: point(4.0, 0.0),
        };
        let cubic = OutlineSegment::Cubic {
            start: point(0.0, 0.0),
            control_1: point(1.0, 4.0),
            control_2: point(3.0, 4.0),
            end: point(4.0, 0.0),
        };
        assert_eq!(quadratic.directional_extent(direction(0.0, 1.0)), 2.0);
        assert_eq!(cubic.directional_extent(direction(0.0, 1.0)), 3.0);
    }
}

//! Directional wedge-bond geometry independent of a renderer.

use crate::{GeometryError, Point2, Vector2};

/// The four corners and scalar facts of a tapered wedge bond.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WedgeGeometry {
    /// Narrow-end center.
    pub tip: Point2,
    /// Wide-end center.
    pub base: Point2,
    /// Left corner at the narrow end while travelling from tip to base.
    pub narrow_left: Point2,
    /// Right corner at the narrow end while travelling from tip to base.
    pub narrow_right: Point2,
    /// Left corner at the wide end while travelling from tip to base.
    pub wide_left: Point2,
    /// Right corner at the wide end while travelling from tip to base.
    pub wide_right: Point2,
    /// Centerline length.
    pub length: f64,
    /// Centerline angle in radians.
    pub angle: f64,
    /// Trapezoid area before any renderer-specific rounding.
    pub area: f64,
}

impl WedgeGeometry {
    /// Computes a directional tapered wedge from a narrow tip to a wide base.
    pub fn new(
        tip: Point2,
        base: Point2,
        wide_width: f64,
        narrow_width: f64,
    ) -> Result<Self, GeometryError> {
        if !wide_width.is_finite() || !narrow_width.is_finite() {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        if wide_width <= 0.0 || narrow_width < 0.0 {
            return Err(GeometryError::NonPositiveExtent);
        }
        let centerline = base - tip;
        let length = centerline.length();
        let unit = centerline.normalized()?;
        let normal: Vector2 = unit.perpendicular_left();
        let narrow_half = narrow_width / 2.0;
        let wide_half = wide_width / 2.0;
        Ok(Self {
            tip,
            base,
            narrow_left: tip.offset(normal, narrow_half)?,
            narrow_right: tip.offset(normal, -narrow_half)?,
            wide_left: base.offset(normal, wide_half)?,
            wide_right: base.offset(normal, -wide_half)?,
            length,
            angle: centerline.y().atan2(centerline.x()),
            area: length * (narrow_width + wide_width) / 2.0,
        })
    }
}

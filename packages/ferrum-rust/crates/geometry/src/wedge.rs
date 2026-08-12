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
        let centerline = finite_vector(base.x() - tip.x(), base.y() - tip.y())?;
        let length = finite_scalar(centerline.length())?;
        let unit = centerline.normalized()?;
        let normal: Vector2 = unit.perpendicular_left();
        let narrow_half = finite_scalar(narrow_width / 2.0)?;
        let wide_half = finite_scalar(wide_width / 2.0)?;
        let narrow_offset = finite_vector(normal.x() * narrow_half, normal.y() * narrow_half)?;
        let wide_offset = finite_vector(normal.x() * wide_half, normal.y() * wide_half)?;
        let narrow_left = translated_point(tip, narrow_offset)?;
        let narrow_right = translated_point(tip, negate(narrow_offset)?)?;
        let wide_left = translated_point(base, wide_offset)?;
        let wide_right = translated_point(base, negate(wide_offset)?)?;
        let angle = finite_scalar(centerline.y().atan2(centerline.x()))?;
        // Halving each width before summing retains valid large finite trapezoids
        // that would overflow in `(narrow_width + wide_width) / 2.0`.
        let half_width_sum = finite_scalar(narrow_half + wide_half)?;
        let area = finite_scalar(length * half_width_sum)?;
        Ok(Self {
            tip,
            base,
            narrow_left,
            narrow_right,
            wide_left,
            wide_right,
            length,
            angle,
            area,
        })
    }
}

fn finite_scalar(value: f64) -> Result<f64, GeometryError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(GeometryError::UnrepresentableGeometry)
    }
}

fn finite_vector(x: f64, y: f64) -> Result<Vector2, GeometryError> {
    if x.is_finite() && y.is_finite() {
        Vector2::new(x, y)
    } else {
        Err(GeometryError::UnrepresentableGeometry)
    }
}

fn negate(vector: Vector2) -> Result<Vector2, GeometryError> {
    finite_vector(-vector.x(), -vector.y())
}

fn translated_point(point: Point2, offset: Vector2) -> Result<Point2, GeometryError> {
    Point2::new(point.x() + offset.x(), point.y() + offset.y())
        .map_err(|_| GeometryError::UnrepresentableGeometry)
}

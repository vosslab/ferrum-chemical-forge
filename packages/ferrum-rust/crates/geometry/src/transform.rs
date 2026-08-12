//! Explicit finite affine transforms for Ferrum coordinates.

use crate::{GeometryError, Point2};

/// A finite affine transform represented as a 2-by-3 matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2 {
    xx: f64,
    xy: f64,
    yx: f64,
    yy: f64,
    tx: f64,
    ty: f64,
}

impl Transform2 {
    /// The identity transform.
    pub const fn identity() -> Self {
        Self {
            xx: 1.0,
            xy: 0.0,
            yx: 0.0,
            yy: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }

    /// Creates a rotation about the coordinate origin.
    pub fn rotation(radians: f64) -> Result<Self, GeometryError> {
        if !radians.is_finite() {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        let (sin, cos) = radians.sin_cos();
        Ok(Self {
            xx: cos,
            xy: -sin,
            yx: sin,
            yy: cos,
            tx: 0.0,
            ty: 0.0,
        })
    }

    /// Creates a translation.
    pub fn translation(dx: f64, dy: f64) -> Result<Self, GeometryError> {
        if !dx.is_finite() || !dy.is_finite() {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        Ok(Self {
            tx: dx,
            ty: dy,
            ..Self::identity()
        })
    }

    /// Applies the transform to a finite point.
    pub fn apply(self, point: Point2) -> Result<Point2, GeometryError> {
        Point2::new(
            self.xx.mul_add(point.x(), self.xy * point.y()) + self.tx,
            self.yx.mul_add(point.x(), self.yy * point.y()) + self.ty,
        )
    }

    /// Applies this transform after `before`.
    pub fn after(self, before: Self) -> Result<Self, GeometryError> {
        Self::from_parts(
            self.xx.mul_add(before.xx, self.xy * before.yx),
            self.xx.mul_add(before.xy, self.xy * before.yy),
            self.yx.mul_add(before.xx, self.yy * before.yx),
            self.yx.mul_add(before.xy, self.yy * before.yy),
            self.xx.mul_add(before.tx, self.xy * before.ty) + self.tx,
            self.yx.mul_add(before.tx, self.yy * before.ty) + self.ty,
        )
    }

    fn from_parts(
        xx: f64,
        xy: f64,
        yx: f64,
        yy: f64,
        tx: f64,
        ty: f64,
    ) -> Result<Self, GeometryError> {
        if [xx, xy, yx, yy, tx, ty]
            .iter()
            .all(|value| value.is_finite())
        {
            Ok(Self {
                xx,
                xy,
                yx,
                yy,
                tx,
                ty,
            })
        } else {
            Err(GeometryError::NonFiniteCoordinate)
        }
    }
}

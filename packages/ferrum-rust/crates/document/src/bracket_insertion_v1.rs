//! Closed finite geometry for one Rust-owned bracket pair.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::Point3V1;

/// Persistent bracket geometry family.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BracketStyleV1 {
    /// Four connected straight segments on each side.
    Rectangular,
    /// Four control points on each spline side.
    Round,
}

/// Complete derived point paths for one new bracket pair.
#[derive(Clone, Debug, PartialEq)]
pub struct BracketInsertionV1 {
    style: BracketStyleV1,
    left: [Point3V1; 4],
    right: [Point3V1; 4],
}

impl BracketInsertionV1 {
    /// Derive classic proportional bracket paths from finite normalized bounds.
    pub fn new(
        style: BracketStyleV1,
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
    ) -> Result<Self, BracketInsertionV1Error> {
        if ![left, top, right, bottom].into_iter().all(f64::is_finite) {
            return Err(BracketInsertionV1Error::BoundsNotFinite);
        }
        if left >= right || top >= bottom {
            return Err(BracketInsertionV1Error::BoundsNotNormalized);
        }
        let width = right - left;
        let height = bottom - top;
        let inset = 0.05 * width.hypot(height);
        let shoulder = 0.05 * height;
        if !width.is_finite() || !height.is_finite() || !inset.is_finite() || !shoulder.is_finite()
        {
            return Err(BracketInsertionV1Error::DerivedGeometryNotFinite);
        }
        let (left_points, right_points) = match style {
            BracketStyleV1::Rectangular => (
                [
                    (left + inset, top),
                    (left, top),
                    (left, bottom),
                    (left + inset, bottom),
                ],
                [
                    (right - inset, top),
                    (right, top),
                    (right, bottom),
                    (right - inset, bottom),
                ],
            ),
            BracketStyleV1::Round => (
                [
                    (left + inset, top),
                    (left, top + shoulder),
                    (left, bottom - shoulder),
                    (left + inset, bottom),
                ],
                [
                    (right - inset, top),
                    (right, top + shoulder),
                    (right, bottom - shoulder),
                    (right - inset, bottom),
                ],
            ),
        };
        Ok(Self {
            style,
            left: points(left_points)?,
            right: points(right_points)?,
        })
    }

    /// Return the persistent bracket family.
    #[must_use]
    pub fn style(&self) -> BracketStyleV1 {
        self.style
    }

    /// Return the left side's four points in authored path order.
    #[must_use]
    pub fn left(&self) -> &[Point3V1; 4] {
        &self.left
    }

    /// Return the right side's four points in authored path order.
    #[must_use]
    pub fn right(&self) -> &[Point3V1; 4] {
        &self.right
    }
}

fn points(values: [(f64, f64); 4]) -> Result<[Point3V1; 4], BracketInsertionV1Error> {
    let [first, second, third, fourth] = values;
    let point = |(x, y)| {
        Point3V1::new(x, y, 0.0).map_err(|_| BracketInsertionV1Error::DerivedGeometryNotFinite)
    };
    Ok([point(first)?, point(second)?, point(third)?, point(fourth)?])
}

/// A bracket request that cannot produce trustworthy persistent paths.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum BracketInsertionV1Error {
    /// One source bound is NaN or infinite.
    #[error("bracket bounds must be finite")]
    BoundsNotFinite,
    /// The left/right or top/bottom extent is empty or reversed.
    #[error("bracket bounds must have strict left-right and top-bottom order")]
    BoundsNotNormalized,
    /// Finite extremes overflowed while deriving proportional points.
    #[error("bracket creation produced non-finite derived geometry")]
    DerivedGeometryNotFinite,
}

//! Closed geometric primitives used by atom-local annotations.

use serde::{Deserialize, Serialize};

use crate::{PositiveFinite, RenderError, RenderPaintV3, RenderPoint};

/// One explicit finite ellipse with optional outline and fill paint.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EllipseOp {
    center: RenderPoint,
    radius_x: PositiveFinite,
    radius_y: PositiveFinite,
    rotation_degrees: f64,
    stroke_width: Option<PositiveFinite>,
    stroke_paint: Option<RenderPaintV3>,
    fill_paint: Option<RenderPaintV3>,
    z: i32,
}

impl EllipseOp {
    /// Construct one fully specified ellipse without toolkit defaults.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        center: RenderPoint,
        radius_x: PositiveFinite,
        radius_y: PositiveFinite,
        rotation_degrees: f64,
        stroke_width: Option<PositiveFinite>,
        stroke_paint: Option<RenderPaintV3>,
        fill_paint: Option<RenderPaintV3>,
        z: i32,
    ) -> Result<Self, RenderError> {
        if !rotation_degrees.is_finite() {
            return Err(RenderError::InvalidRequest(
                "ellipse rotation must be finite".to_owned(),
            ));
        }
        if stroke_width.is_some() != stroke_paint.is_some() {
            return Err(RenderError::InvalidRequest(
                "ellipse outline requires both width and paint".to_owned(),
            ));
        }
        if stroke_paint.is_none() && fill_paint.is_none() {
            return Err(RenderError::InvalidRequest(
                "ellipse requires an explicit outline or fill".to_owned(),
            ));
        }
        Ok(Self {
            center,
            radius_x,
            radius_y,
            rotation_degrees: if rotation_degrees == 0.0 {
                0.0
            } else {
                rotation_degrees
            },
            stroke_width,
            stroke_paint,
            fill_paint,
            z,
        })
    }

    /// Return the atom-local ellipse center.
    #[must_use]
    pub fn center(&self) -> RenderPoint {
        self.center
    }
    /// Return the horizontal radius before rotation.
    #[must_use]
    pub fn radius_x(&self) -> PositiveFinite {
        self.radius_x
    }
    /// Return the vertical radius before rotation.
    #[must_use]
    pub fn radius_y(&self) -> PositiveFinite {
        self.radius_y
    }
    /// Return clockwise scene rotation in degrees.
    #[must_use]
    pub fn rotation_degrees(&self) -> f64 {
        self.rotation_degrees
    }
    /// Return explicit outline width when an outline exists.
    #[must_use]
    pub fn stroke_width(&self) -> Option<PositiveFinite> {
        self.stroke_width
    }
    /// Return explicit outline paint when an outline exists.
    #[must_use]
    pub fn stroke_paint(&self) -> Option<&RenderPaintV3> {
        self.stroke_paint.as_ref()
    }
    /// Return explicit fill paint when a fill exists.
    #[must_use]
    pub fn fill_paint(&self) -> Option<&RenderPaintV3> {
        self.fill_paint.as_ref()
    }
    /// Return deterministic z-order within the batch.
    #[must_use]
    pub fn z(&self) -> i32 {
        self.z
    }
}

impl<'de> Deserialize<'de> for EllipseOp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireEllipseOp {
            center: RenderPoint,
            radius_x: PositiveFinite,
            radius_y: PositiveFinite,
            rotation_degrees: f64,
            stroke_width: Option<PositiveFinite>,
            stroke_paint: Option<RenderPaintV3>,
            fill_paint: Option<RenderPaintV3>,
            z: i32,
        }
        let wire = WireEllipseOp::deserialize(deserializer)?;
        Self::new(
            wire.center,
            wire.radius_x,
            wire.radius_y,
            wire.rotation_degrees,
            wire.stroke_width,
            wire.stroke_paint,
            wire.fill_paint,
            wire.z,
        )
        .map_err(serde::de::Error::custom)
    }
}

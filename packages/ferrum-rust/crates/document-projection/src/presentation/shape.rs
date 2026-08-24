//! Immutable vector-shape projection values and validated wire conversion.

use serde::{Deserialize, Deserializer, Serialize};

use super::{
    PresentationFactProvenanceV1, PresentationStackProjectionV1Error, PresentationStrokeV1,
    PresentationTargetV1,
};
use crate::{Point3V1, Rgb24V1};

/// Normalized finite scene bounds for a direct-root box shape.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PresentationBoundsV1 {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl<'de> Deserialize<'de> for PresentationBoundsV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PresentationBoundsWireV1::deserialize(deserializer)?;
        Self::try_new(wire.left, wire.top, wire.right, wire.bottom)
            .map_err(serde::de::Error::custom)
    }
}

impl PresentationBoundsV1 {
    pub fn try_new(
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        ([left, top, right, bottom]
            .iter()
            .all(|value| value.is_finite())
            && left < right
            && top < bottom)
            .then_some(Self {
                left,
                top,
                right,
                bottom,
            })
            .ok_or(PresentationStackProjectionV1Error::InvalidBounds)
    }

    pub fn from_corners(
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        Self::try_new(x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2))
    }

    #[must_use]
    pub fn left(self) -> f64 {
        self.left
    }

    #[must_use]
    pub fn top(self) -> f64 {
        self.top
    }

    #[must_use]
    pub fn right(self) -> f64 {
        self.right
    }

    #[must_use]
    pub fn bottom(self) -> f64 {
        self.bottom
    }
}

/// Resolved optional fill color and its explicit precedence source.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PresentationFillV1 {
    color: Option<Rgb24V1>,
    color_provenance: PresentationFactProvenanceV1,
}

impl<'de> Deserialize<'de> for PresentationFillV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PresentationFillWireV1::deserialize(deserializer)?;
        let color = match wire.color {
            Some(value) => Some(
                Rgb24V1::new(value)
                    .ok_or_else(|| serde::de::Error::custom("invalid presentation fill colour"))?,
            ),
            None => None,
        };
        Self::try_new(color, wire.color_provenance).map_err(serde::de::Error::custom)
    }
}

impl PresentationFillV1 {
    /// Construct resolved fill facts with matching provenance.
    pub fn try_new(
        color: Option<Rgb24V1>,
        color_provenance: PresentationFactProvenanceV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        (color_provenance != PresentationFactProvenanceV1::Builtin || color.is_none())
            .then_some(Self {
                color,
                color_provenance,
            })
            .ok_or(PresentationStackProjectionV1Error::InvalidFill)
    }

    #[must_use]
    pub fn color(&self) -> Option<&Rgb24V1> {
        self.color.as_ref()
    }

    #[must_use]
    pub fn color_provenance(&self) -> PresentationFactProvenanceV1 {
        self.color_provenance
    }
}

/// One rectangle, square, oval, or circle projection.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BoxShapeProjectionV1 {
    target: PresentationTargetV1,
    bounds: PresentationBoundsV1,
    stroke: PresentationStrokeV1,
    fill: PresentationFillV1,
}

impl<'de> Deserialize<'de> for BoxShapeProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = BoxShapeWireV1::deserialize(deserializer)?;
        Self::try_new(wire.target, wire.bounds, wire.stroke, wire.fill)
            .map_err(serde::de::Error::custom)
    }
}

impl BoxShapeProjectionV1 {
    /// Construct one box-family payload for a box-family target.
    pub fn try_new(
        target: PresentationTargetV1,
        bounds: PresentationBoundsV1,
        stroke: PresentationStrokeV1,
        fill: PresentationFillV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        matches!(
            target.record_kind(),
            super::PresentationRecordKindV1::Rectangle
                | super::PresentationRecordKindV1::Square
                | super::PresentationRecordKindV1::Oval
                | super::PresentationRecordKindV1::Circle
        )
        .then_some(Self {
            target,
            bounds,
            stroke,
            fill,
        })
        .ok_or(PresentationStackProjectionV1Error::RootKindMismatch)
    }
    #[must_use]
    pub fn target(&self) -> &PresentationTargetV1 {
        &self.target
    }

    #[must_use]
    pub fn bounds(&self) -> PresentationBoundsV1 {
        self.bounds
    }

    #[must_use]
    pub fn stroke(&self) -> &PresentationStrokeV1 {
        &self.stroke
    }

    #[must_use]
    pub fn fill(&self) -> &PresentationFillV1 {
        &self.fill
    }
}

/// Three or more ordered finite points of a direct-root polygon.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PolygonPathV1 {
    points: Vec<Point3V1>,
}

impl<'de> Deserialize<'de> for PolygonPathV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PolygonPathWireV1::deserialize(deserializer)?;
        let points = wire
            .points
            .into_iter()
            .map(PointWireV1::into_point)
            .collect::<Result<_, _>>()
            .map_err(serde::de::Error::custom)?;
        Self::try_new(points).map_err(serde::de::Error::custom)
    }
}

impl PolygonPathV1 {
    /// Construct three or more ordered finite polygon points.
    pub fn try_new(points: Vec<Point3V1>) -> Result<Self, PresentationStackProjectionV1Error> {
        (points.len() >= 3)
            .then_some(Self { points })
            .ok_or(PresentationStackProjectionV1Error::InvalidPolygonPath)
    }
    #[must_use]
    pub fn points(&self) -> &[Point3V1] {
        &self.points
    }
}

/// One direct-root polygon projection.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PolygonProjectionV1 {
    target: PresentationTargetV1,
    path: PolygonPathV1,
    stroke: PresentationStrokeV1,
    fill: PresentationFillV1,
}

impl<'de> Deserialize<'de> for PolygonProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PolygonWireV1::deserialize(deserializer)?;
        Self::try_new(wire.target, wire.path, wire.stroke, wire.fill)
            .map_err(serde::de::Error::custom)
    }
}

impl PolygonProjectionV1 {
    /// Construct a polygon payload only for a Polygon target.
    pub fn try_new(
        target: PresentationTargetV1,
        path: PolygonPathV1,
        stroke: PresentationStrokeV1,
        fill: PresentationFillV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        (target.record_kind() == super::PresentationRecordKindV1::Polygon)
            .then_some(Self {
                target,
                path,
                stroke,
                fill,
            })
            .ok_or(PresentationStackProjectionV1Error::RootKindMismatch)
    }
    #[must_use]
    pub fn target(&self) -> &PresentationTargetV1 {
        &self.target
    }

    #[must_use]
    pub fn path(&self) -> &PolygonPathV1 {
        &self.path
    }

    #[must_use]
    pub fn stroke(&self) -> &PresentationStrokeV1 {
        &self.stroke
    }

    #[must_use]
    pub fn fill(&self) -> &PresentationFillV1 {
        &self.fill
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentationBoundsWireV1 {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentationFillWireV1 {
    pub color: Option<String>,
    pub color_provenance: PresentationFactProvenanceV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BoxShapeWireV1 {
    pub target: PresentationTargetV1,
    pub bounds: PresentationBoundsV1,
    pub stroke: PresentationStrokeV1,
    pub fill: PresentationFillV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolygonPathWireV1 {
    pub points: Vec<PointWireV1>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointWireV1 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PointWireV1 {
    fn into_point(self) -> Result<Point3V1, serde::de::value::Error> {
        Point3V1::new(self.x, self.y, self.z)
            .map_err(|error| serde::de::Error::custom(error.to_string()))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolygonWireV1 {
    pub target: PresentationTargetV1,
    pub path: PolygonPathV1,
    pub stroke: PresentationStrokeV1,
    pub fill: PresentationFillV1,
}

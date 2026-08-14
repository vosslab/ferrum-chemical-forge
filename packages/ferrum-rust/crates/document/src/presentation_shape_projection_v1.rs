//! Closed vector-shape facts for direct-root presentation projection.

use serde::{Deserialize, Deserializer, Serialize};

use super::presentation_polyline_projection_v1::{
    RootStrokeDefaultsV1, coordinate, points, stroke,
};
use super::presentation_stack_projection_v1::{
    PresentationFactProvenanceV1, PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1,
    PresentationStrokeV1, PresentationTargetV1,
};
use super::{Point3V1, Rgb24V1, TransparentOrRgb24V1, TypedChild, TypedRecord};

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
        Self::new(wire.left, wire.top, wire.right, wire.bottom)
            .filter(|bounds| bounds.left < bounds.right && bounds.top < bounds.bottom)
            .ok_or_else(|| serde::de::Error::custom("invalid normalized presentation bounds"))
    }
}

impl PresentationBoundsV1 {
    fn new(left: f64, top: f64, right: f64, bottom: f64) -> Option<Self> {
        [left, top, right, bottom]
            .iter()
            .all(|value| value.is_finite())
            .then_some(Self {
                left,
                top,
                right,
                bottom,
            })
    }

    fn from_corners(x1: f64, y1: f64, x2: f64, y2: f64) -> Option<Self> {
        Self::new(x1.min(x2), y1.min(y2), x1.max(x2), y1.max(y2))
            .filter(|bounds| bounds.left < bounds.right && bounds.top < bounds.bottom)
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
        if wire.color_provenance == PresentationFactProvenanceV1::Builtin && color.is_some() {
            return Err(serde::de::Error::custom(
                "built-in presentation fill must use the transparent V1 value",
            ));
        }
        Ok(Self {
            color,
            color_provenance: wire.color_provenance,
        })
    }
}

impl PresentationFillV1 {
    pub(crate) fn resolved(
        color: Option<Rgb24V1>,
        color_provenance: PresentationFactProvenanceV1,
    ) -> Self {
        Self {
            color,
            color_provenance,
        }
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
        Ok(Self {
            target: wire.target,
            bounds: wire.bounds,
            stroke: wire.stroke,
            fill: wire.fill,
        })
    }
}

impl BoxShapeProjectionV1 {
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
        if wire.points.len() < 3 {
            return Err(serde::de::Error::custom(
                "polygon path requires at least three finite points",
            ));
        }
        let points = wire
            .points
            .into_iter()
            .map(PointWireV1::into_point)
            .collect::<Result<_, _>>()
            .map_err(serde::de::Error::custom)?;
        Ok(Self { points })
    }
}

impl PolygonPathV1 {
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
        Ok(Self {
            target: wire.target,
            path: wire.path,
            stroke: wire.stroke,
            fill: wire.fill,
        })
    }
}

impl PolygonProjectionV1 {
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
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentationFillWireV1 {
    color: Option<String>,
    color_provenance: PresentationFactProvenanceV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BoxShapeWireV1 {
    target: PresentationTargetV1,
    bounds: PresentationBoundsV1,
    stroke: PresentationStrokeV1,
    fill: PresentationFillV1,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolygonPathWireV1 {
    points: Vec<PointWireV1>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointWireV1 {
    x: f64,
    y: f64,
    z: f64,
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
    target: PresentationTargetV1,
    path: PolygonPathV1,
    stroke: PresentationStrokeV1,
    fill: PresentationFillV1,
}

pub(crate) fn box_shape(
    child: &TypedChild,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<BoxShapeProjectionV1> {
    let target = PresentationTargetV1::from_child(child);
    let record = child.record();
    let bounds = match bounds(record) {
        Ok(bounds) => bounds,
        Err(detail) => {
            issues.push(PresentationProjectionIssueV1::new(
                target,
                PresentationProjectionIssueCodeV1::InvalidShapeGeometry,
                detail,
            ));
            return None;
        }
    };
    Some(BoxShapeProjectionV1 {
        stroke: stroke(record, defaults, &target, issues),
        fill: fill(record, defaults.standard, &target, issues),
        target,
        bounds,
    })
}

pub(crate) fn polygon(
    child: &TypedChild,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<PolygonProjectionV1> {
    let target = PresentationTargetV1::from_child(child);
    let record = child.record();
    let path = match points(record, 3, "polygon") {
        Ok(points) => PolygonPathV1 { points },
        Err(detail) => {
            issues.push(PresentationProjectionIssueV1::new(
                target,
                PresentationProjectionIssueCodeV1::InvalidPolygonGeometry,
                detail,
            ));
            return None;
        }
    };
    Some(PolygonProjectionV1 {
        stroke: stroke(record, defaults, &target, issues),
        fill: fill(record, defaults.standard, &target, issues),
        target,
        path,
    })
}

fn bounds(record: &TypedRecord) -> Result<PresentationBoundsV1, String> {
    let x1 = coordinate(record, "x1")?;
    let y1 = coordinate(record, "y1")?;
    let x2 = coordinate(record, "x2")?;
    let y2 = coordinate(record, "y2")?;
    PresentationBoundsV1::from_corners(x1, y1, x2, y2)
        .ok_or_else(|| "shape bounds must be finite with positive width and height".to_owned())
}

fn fill(
    root: &TypedRecord,
    standard: Option<&TypedRecord>,
    target: &PresentationTargetV1,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> PresentationFillV1 {
    for (record, field, provenance) in [
        (Some(root), "area_color", PresentationFactProvenanceV1::Root),
        (
            root.attribute("area_color").is_none().then_some(root),
            "background-color",
            PresentationFactProvenanceV1::Root,
        ),
        (
            standard,
            "area_color",
            PresentationFactProvenanceV1::Standard,
        ),
    ] {
        let Some(record) = record else {
            continue;
        };
        let Some(value) = record.attribute(field) else {
            continue;
        };
        match TransparentOrRgb24V1::new(value) {
            Some(TransparentOrRgb24V1::Transparent) => {
                return PresentationFillV1 {
                    color: None,
                    color_provenance: provenance,
                };
            }
            Some(TransparentOrRgb24V1::Rgb24(color)) => {
                return PresentationFillV1 {
                    color: Some(color),
                    color_provenance: provenance,
                };
            }
            None => issues.push(PresentationProjectionIssueV1::new(
                target.clone(),
                PresentationProjectionIssueCodeV1::InvalidFillFact,
                format!("{field} must be empty, none, #rgb, or #rrggbb"),
            )),
        }
    }
    PresentationFillV1 {
        color: None,
        color_provenance: PresentationFactProvenanceV1::Builtin,
    }
}

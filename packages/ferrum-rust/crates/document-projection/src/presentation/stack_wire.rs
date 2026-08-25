//! Private serde wire format for presentation stack projections.

use serde::Deserialize;

use super::super::{
    ArrowProjectionV1, BoxShapeProjectionV1, BracketPairProjectionV1, PlusProjectionV1,
    PolygonProjectionV1, TextProjectionV1,
};
use super::stack_model::{
    BUILTIN_LINE_COLOR, BUILTIN_LINE_WIDTH, PolylinePathV1, PolylineProjectionV1,
    PresentationFactProvenanceV1, PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1,
    PresentationRecordKindV1, PresentationRootProjectionV1, PresentationStrokeV1,
    PresentationTargetV1,
};
use crate::{DocumentObjectIdV1, Point3V1, PositiveFiniteV1, Rgb24V1};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PresentationStackWireV1 {
    pub schema: String,
    pub revision: u64,
    pub digest: [u8; 32],
    pub entries: Vec<PresentationRootWireV1>,
    pub bracket_pairs: Vec<BracketPairProjectionV1>,
    pub issues: Vec<PresentationProjectionIssueV1>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum PresentationRootWireV1 {
    Arrow { arrow: ArrowProjectionV1 },
    Plus { plus: PlusProjectionV1 },
    Text { text: TextProjectionV1 },
    Polyline { polyline: PolylineProjectionV1 },
    Wavy { polyline: PolylineProjectionV1 },
    RoundBracket { polyline: PolylineProjectionV1 },
    Rectangle { shape: BoxShapeProjectionV1 },
    Square { shape: BoxShapeProjectionV1 },
    Oval { shape: BoxShapeProjectionV1 },
    Circle { shape: BoxShapeProjectionV1 },
    Polygon { polygon: PolygonProjectionV1 },
}

impl TryFrom<PresentationRootWireV1> for PresentationRootProjectionV1 {
    type Error = serde::de::value::Error;

    fn try_from(value: PresentationRootWireV1) -> Result<Self, Self::Error> {
        let root = match value {
            PresentationRootWireV1::Arrow { arrow } => Self::arrow(arrow),
            PresentationRootWireV1::Plus { plus } => Self::plus(plus),
            PresentationRootWireV1::Text { text } => Self::text(text),
            PresentationRootWireV1::Polyline { polyline } => Self::polyline(polyline),
            PresentationRootWireV1::Wavy { polyline } => Self::wavy(polyline),
            PresentationRootWireV1::RoundBracket { polyline } => Self::round_bracket(polyline),
            PresentationRootWireV1::Rectangle { shape } => Self::rectangle(shape),
            PresentationRootWireV1::Square { shape } => Self::square(shape),
            PresentationRootWireV1::Oval { shape } => Self::oval(shape),
            PresentationRootWireV1::Circle { shape } => Self::circle(shape),
            PresentationRootWireV1::Polygon { polygon } => Self::polygon(polygon),
        };
        root.map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PolylineWireV1 {
    pub target: PresentationTargetV1,
    pub path: PolylinePathV1,
    pub stroke: PresentationStrokeV1,
}

impl TryFrom<PolylineWireV1> for PolylineProjectionV1 {
    type Error = serde::de::value::Error;

    fn try_from(value: PolylineWireV1) -> Result<Self, Self::Error> {
        Self::new(value.target, value.path, value.stroke).map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PresentationTargetWireV1 {
    pub document_object_id: String,
    pub record_kind: PresentationRecordKindV1,
}

impl TryFrom<PresentationTargetWireV1> for PresentationTargetV1 {
    type Error = serde::de::value::Error;

    fn try_from(value: PresentationTargetWireV1) -> Result<Self, Self::Error> {
        let document_object_id = DocumentObjectIdV1::parse(value.document_object_id)
            .map_err(|error| serde::de::Error::custom(error.to_string()))?;
        Ok(Self::new(document_object_id, value.record_kind))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PresentationIssueWireV1 {
    pub target: PresentationTargetV1,
    pub code: PresentationProjectionIssueCodeV1,
    pub detail: String,
}

impl From<PresentationIssueWireV1> for PresentationProjectionIssueV1 {
    fn from(value: PresentationIssueWireV1) -> Self {
        Self::new(value.target, value.code, value.detail)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PolylinePathWireV1 {
    pub points: Vec<PointWireV1>,
}

impl TryFrom<PolylinePathWireV1> for PolylinePathV1 {
    type Error = serde::de::value::Error;

    fn try_from(value: PolylinePathWireV1) -> Result<Self, Self::Error> {
        if value.points.len() < 2 {
            return Err(serde::de::Error::custom(
                "polyline path requires at least two finite points",
            ));
        }
        let points = value
            .points
            .iter()
            .map(PointWireV1::to_point)
            .collect::<Result<_, _>>()?;
        Self::try_new(points).map_err(serde::de::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PointWireV1 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PointWireV1 {
    fn to_point(&self) -> Result<Point3V1, serde::de::value::Error> {
        Point3V1::new(self.x, self.y, self.z)
            .map_err(|error| serde::de::Error::custom(error.to_string()))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PresentationStrokeWireV1 {
    pub color: String,
    pub color_provenance: PresentationFactProvenanceV1,
    pub width: f64,
    pub width_provenance: PresentationFactProvenanceV1,
}

impl TryFrom<PresentationStrokeWireV1> for PresentationStrokeV1 {
    type Error = serde::de::value::Error;

    fn try_from(value: PresentationStrokeWireV1) -> Result<Self, Self::Error> {
        let color = Rgb24V1::new(value.color)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation stroke colour"))?;
        let width = PositiveFiniteV1::new(value.width)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation stroke width"))?;
        if value.color_provenance == PresentationFactProvenanceV1::Builtin
            && color.as_str() != BUILTIN_LINE_COLOR
        {
            return Err(serde::de::Error::custom(
                "built-in presentation stroke colour must use the closed V1 value",
            ));
        }
        if value.width_provenance == PresentationFactProvenanceV1::Builtin
            && width.value() != BUILTIN_LINE_WIDTH
        {
            return Err(serde::de::Error::custom(
                "built-in presentation stroke width must use the closed V1 value",
            ));
        }
        Self::new(color, value.color_provenance, width, value.width_provenance)
            .map_err(serde::de::Error::custom)
    }
}

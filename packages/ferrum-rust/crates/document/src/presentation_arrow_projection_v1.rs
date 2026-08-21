//! Closed normal-arrow projection with backend-derived head geometry.

use serde::{Deserialize, Deserializer, Serialize};

use super::presentation_polyline_projection_v1::{
    RootStrokeDefaultsV1, points, stroke_with_color_field,
};
use super::presentation_stack_projection_v1::{
    PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1, PresentationStrokeV1,
    PresentationTargetV1,
};
use super::{Point3V1, TypedChild, TypedRecord};

const DEFAULT_HEAD_LINE_INSET: f64 = 8.0;
const DEFAULT_HEAD_TOTAL_LENGTH: f64 = 10.0;
const DEFAULT_HEAD_HALF_WIDTH: f64 = 3.0;

/// The ordered finite source path retained by a supported normal arrow.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ArrowPathV1 {
    points: Vec<Point3V1>,
}

impl<'de> Deserialize<'de> for ArrowPathV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ArrowPathWireV1::deserialize(deserializer)?;
        Self::from_wire(wire).map_err(serde::de::Error::custom)
    }
}

impl ArrowPathV1 {
    fn from_wire(wire: ArrowPathWireV1) -> Result<Self, serde::de::value::Error> {
        if wire.points.len() < 2 {
            return Err(serde::de::Error::custom(
                "arrow path requires at least two finite points",
            ));
        }
        Ok(Self {
            points: wire
                .points
                .into_iter()
                .map(PointWireV1::into_point)
                .collect::<Result<_, _>>()?,
        })
    }

    #[must_use]
    pub fn points(&self) -> &[Point3V1] {
        &self.points
    }
}

/// Validated normal-arrow head dimensions in scene points.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ArrowHeadShapeV1 {
    line_inset: f64,
    total_length: f64,
    half_width: f64,
}

impl<'de> Deserialize<'de> for ArrowHeadShapeV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ArrowHeadShapeWireV1::deserialize(deserializer)?;
        Self::new(wire.line_inset, wire.total_length, wire.half_width)
            .ok_or_else(|| serde::de::Error::custom("invalid normal-arrow head dimensions"))
    }
}

impl ArrowHeadShapeV1 {
    fn new(line_inset: f64, total_length: f64, half_width: f64) -> Option<Self> {
        (line_inset.is_finite()
            && total_length.is_finite()
            && half_width.is_finite()
            && line_inset > 0.0
            && total_length >= line_inset
            && half_width > 0.0)
            .then_some(Self {
                line_inset,
                total_length,
                half_width,
            })
    }

    #[must_use]
    pub fn line_inset(self) -> f64 {
        self.line_inset
    }

    #[must_use]
    pub fn total_length(self) -> f64 {
        self.total_length
    }

    #[must_use]
    pub fn half_width(self) -> f64 {
        self.half_width
    }
}

/// The endpoint at which one normal-arrow head polygon is anchored.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArrowHeadPositionV1 {
    Start,
    End,
}

/// One filled four-point normal-arrow head polygon.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ArrowHeadV1 {
    position: ArrowHeadPositionV1,
    points: [Point3V1; 4],
}

impl<'de> Deserialize<'de> for ArrowHeadV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ArrowHeadWireV1::deserialize(deserializer)?;
        Ok(Self {
            position: wire.position,
            points: wire
                .points
                .map(PointWireV1::into_point)
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .map_err(serde::de::Error::custom)?
                .try_into()
                .expect("four wire points remain four validated points"),
        })
    }
}

impl ArrowHeadV1 {
    #[must_use]
    pub fn position(&self) -> ArrowHeadPositionV1 {
        self.position
    }

    #[must_use]
    pub fn points(&self) -> &[Point3V1; 4] {
        &self.points
    }
}

/// Closed backend-issued display geometry for one semantic Arrow root.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArrowDisplayGeometryV1 {
    Normal {
        axis_path: ArrowPathV1,
        head_shape: ArrowHeadShapeV1,
        start_head: bool,
        end_head: bool,
        heads: Vec<ArrowHeadV1>,
    },
    Equilibrium {
        axes: [ArrowPathV1; 2],
        heads: [ArrowHeadV1; 2],
    },
}

/// One supported non-spline direct-root Arrow with kind-owned display geometry.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ArrowProjectionV1 {
    target: PresentationTargetV1,
    source_path: ArrowPathV1,
    geometry: ArrowDisplayGeometryV1,
    stroke: PresentationStrokeV1,
}

impl<'de> Deserialize<'de> for ArrowProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ArrowProjectionWireV1::deserialize(deserializer)?;
        let (axis_path, heads) = arrow_geometry(
            &wire.source_path.points,
            wire.head_shape,
            wire.start_head,
            wire.end_head,
        )
        .map_err(serde::de::Error::custom)?;
        if wire.axis_path != axis_path || wire.heads != heads {
            return Err(serde::de::Error::custom(
                "normal-arrow display geometry does not match its source facts",
            ));
        }
        Ok(Self {
            target: wire.target,
            source_path: wire.source_path,
            geometry: ArrowDisplayGeometryV1::Normal {
                axis_path,
                head_shape: wire.head_shape,
                start_head: wire.start_head,
                end_head: wire.end_head,
                heads,
            },
            stroke: wire.stroke,
        })
    }
}

impl ArrowProjectionV1 {
    #[must_use]
    pub fn target(&self) -> &PresentationTargetV1 {
        &self.target
    }

    #[must_use]
    pub fn source_path(&self) -> &ArrowPathV1 {
        &self.source_path
    }

    #[must_use]
    pub fn geometry(&self) -> &ArrowDisplayGeometryV1 {
        &self.geometry
    }

    #[must_use]
    pub fn stroke(&self) -> &PresentationStrokeV1 {
        &self.stroke
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArrowPathWireV1 {
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
struct ArrowHeadShapeWireV1 {
    line_inset: f64,
    total_length: f64,
    half_width: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArrowHeadWireV1 {
    position: ArrowHeadPositionV1,
    points: [PointWireV1; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArrowProjectionWireV1 {
    target: PresentationTargetV1,
    source_path: ArrowPathV1,
    axis_path: ArrowPathV1,
    head_shape: ArrowHeadShapeV1,
    start_head: bool,
    end_head: bool,
    heads: Vec<ArrowHeadV1>,
    stroke: PresentationStrokeV1,
}

pub(crate) fn arrow(
    child: &TypedChild,
    defaults: RootStrokeDefaultsV1<'_>,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<ArrowProjectionV1> {
    let target = PresentationTargetV1::from_child(child);
    let record = child.record();
    let arrow_type = record.attribute("type").unwrap_or("normal");
    if !matches!(arrow_type, "normal" | "equilibrium") {
        issues.push(PresentationProjectionIssueV1::new(
            target,
            PresentationProjectionIssueCodeV1::UnsupportedArrowType,
            "this arrow family has no closed V1 display geometry",
        ));
        return None;
    }
    let spline = match boolean(record, "spline", false) {
        Ok(value) => value,
        Err(detail) => {
            issues.push(PresentationProjectionIssueV1::new(
                target,
                PresentationProjectionIssueCodeV1::InvalidArrowFact,
                detail,
            ));
            return None;
        }
    };
    if spline {
        issues.push(PresentationProjectionIssueV1::new(
            target,
            PresentationProjectionIssueCodeV1::UnsupportedArrowSpline,
            "normal-arrow spline interpolation is preserved but not rendered by V1",
        ));
        return None;
    }
    let source_points = match points(record, 2, "arrow") {
        Ok(value) if value.len() == 2 => value,
        Ok(_) => {
            return invalid_geometry(
                target,
                "straight arrows require exactly two points".to_owned(),
                issues,
            );
        }
        Err(detail) => return invalid_geometry(target, detail, issues),
    };
    if arrow_type == "equilibrium" {
        if ["start", "end", "shape"]
            .into_iter()
            .any(|field| record.attribute(field).is_some())
        {
            return invalid_fact(
                target,
                "equilibrium arrows cannot carry normal-arrow head facts".to_owned(),
                issues,
            );
        }
        let start = source_points[0];
        let end = source_points[1];
        let issued = match crate::equilibrium_arrow_geometry_v1::geometry(start, end) {
            Ok(value) => value,
            Err(detail) => return invalid_geometry(target, detail, issues),
        };
        let axes = issued.axes.map(|points| ArrowPathV1 {
            points: points.to_vec(),
        });
        let heads = [
            ArrowHeadV1 {
                position: ArrowHeadPositionV1::Start,
                points: issued.heads[0],
            },
            ArrowHeadV1 {
                position: ArrowHeadPositionV1::End,
                points: issued.heads[1],
            },
        ];
        return Some(ArrowProjectionV1 {
            stroke: stroke_with_color_field(record, defaults, &target, issues, "color"),
            target,
            source_path: ArrowPathV1 {
                points: source_points,
            },
            geometry: ArrowDisplayGeometryV1::Equilibrium { axes, heads },
        });
    }
    let start_head = match boolean(record, "start", false) {
        Ok(value) => value,
        Err(detail) => return invalid_fact(target, detail, issues),
    };
    let end_head = match boolean(record, "end", true) {
        Ok(value) => value,
        Err(detail) => return invalid_fact(target, detail, issues),
    };
    let head_shape = match head_shape(record) {
        Ok(value) => value,
        Err(detail) => return invalid_fact(target, detail, issues),
    };
    let (axis_path, heads) = match arrow_geometry(&source_points, head_shape, start_head, end_head)
    {
        Ok(value) => value,
        Err(detail) => {
            issues.push(PresentationProjectionIssueV1::new(
                target,
                PresentationProjectionIssueCodeV1::InvalidArrowGeometry,
                detail,
            ));
            return None;
        }
    };
    Some(ArrowProjectionV1 {
        stroke: stroke_with_color_field(record, defaults, &target, issues, "color"),
        target,
        source_path: ArrowPathV1 {
            points: source_points,
        },
        geometry: ArrowDisplayGeometryV1::Normal {
            axis_path,
            head_shape,
            start_head,
            end_head,
            heads,
        },
    })
}

fn invalid_geometry(
    target: PresentationTargetV1,
    detail: String,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<ArrowProjectionV1> {
    issues.push(PresentationProjectionIssueV1::new(
        target,
        PresentationProjectionIssueCodeV1::InvalidArrowGeometry,
        detail,
    ));
    None
}

fn invalid_fact(
    target: PresentationTargetV1,
    detail: String,
    issues: &mut Vec<PresentationProjectionIssueV1>,
) -> Option<ArrowProjectionV1> {
    issues.push(PresentationProjectionIssueV1::new(
        target,
        PresentationProjectionIssueCodeV1::InvalidArrowFact,
        detail,
    ));
    None
}

fn boolean(record: &TypedRecord, field: &'static str, default: bool) -> Result<bool, String> {
    let Some(value) = record.attribute(field) else {
        return Ok(default);
    };
    match value.to_ascii_lowercase().as_str() {
        "1" | "both" | "true" | "yes" => Ok(true),
        "0" | "false" | "no" => Ok(false),
        _ => Err(format!("arrow {field} must be a supported yes/no value")),
    }
}

fn head_shape(record: &TypedRecord) -> Result<ArrowHeadShapeV1, String> {
    let Some(value) = record.attribute("shape") else {
        return Ok(ArrowHeadShapeV1 {
            line_inset: DEFAULT_HEAD_LINE_INSET,
            total_length: DEFAULT_HEAD_TOTAL_LENGTH,
            half_width: DEFAULT_HEAD_HALF_WIDTH,
        });
    };
    let inner = value
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| "arrow shape must be a three-number parenthesized tuple".to_owned())?;
    let values = inner
        .split(',')
        .map(str::trim)
        .map(str::parse::<f64>)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "arrow shape contains an invalid number".to_owned())?;
    let [line_inset, total_length, half_width] = values.as_slice() else {
        return Err("arrow shape requires exactly three numbers".to_owned());
    };
    ArrowHeadShapeV1::new(*line_inset, *total_length, *half_width).ok_or_else(|| {
        "arrow shape requires positive finite width and total length at least its line inset"
            .to_owned()
    })
}

fn arrow_geometry(
    source: &[Point3V1],
    shape: ArrowHeadShapeV1,
    start_head: bool,
    end_head: bool,
) -> Result<(ArrowPathV1, Vec<ArrowHeadV1>), String> {
    if source.len() < 2 {
        return Err("arrow geometry requires at least two points".to_owned());
    }
    let mut axis = source.to_vec();
    let mut heads = Vec::new();
    if start_head {
        let (axis_point, points) = head_geometry(source[1], source[0], shape)?;
        axis[0] = axis_point;
        heads.push(ArrowHeadV1 {
            position: ArrowHeadPositionV1::Start,
            points,
        });
    }
    if end_head {
        let last = source.len() - 1;
        let (axis_point, points) = head_geometry(source[last - 1], source[last], shape)?;
        axis[last] = axis_point;
        heads.push(ArrowHeadV1 {
            position: ArrowHeadPositionV1::End,
            points,
        });
    }
    Ok((ArrowPathV1 { points: axis }, heads))
}

fn head_geometry(
    before: Point3V1,
    tip: Point3V1,
    shape: ArrowHeadShapeV1,
) -> Result<(Point3V1, [Point3V1; 4]), String> {
    let dx = tip.x() - before.x();
    let dy = tip.y() - before.y();
    let length = dx.hypot(dy);
    if !length.is_finite() || length == 0.0 {
        return Err("an active arrow head requires a nonzero finite endpoint segment".to_owned());
    }
    let ux = dx / length;
    let uy = dy / length;
    let point = |distance: f64, offset: f64| {
        Point3V1::new(
            tip.x() - (distance * ux) - (offset * uy),
            tip.y() - (distance * uy) + (offset * ux),
            tip.z(),
        )
        .map_err(|error| error.to_string())
    };
    let inner = point(shape.line_inset, 0.0)?;
    let left = point(shape.total_length, shape.half_width)?;
    let right = point(shape.total_length, -shape.half_width)?;
    Ok((inner, [tip, left, inner, right]))
}

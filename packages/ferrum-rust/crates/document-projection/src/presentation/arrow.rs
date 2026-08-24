//! Immutable semantic arrow projection values.
//!
//! This module retains authored arrow intent only. The renderer owns every
//! display path, head polygon, lane offset, curve control, and painted bound.

use serde::{Deserialize, Deserializer, Serialize};

use super::{PresentationRecordKindV1, PresentationStrokeV1, PresentationTargetV1};
use crate::{Point3V1, ProjectionLocalObjectKeyV1};

/// Ordered finite authored points for one supported arrow.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ArrowPathV1 {
    points: Vec<Point3V1>,
}

impl ArrowPathV1 {
    /// Construct an ordered finite source path with the minimum arrow cardinality.
    pub fn try_new(points: Vec<Point3V1>) -> Result<Self, ArrowProjectionV1Error> {
        if points.len() < 2 {
            return Err(ArrowProjectionV1Error::TooFewPoints);
        }
        Ok(Self { points })
    }

    #[must_use]
    pub fn points(&self) -> &[Point3V1] {
        &self.points
    }
}

impl<'de> Deserialize<'de> for ArrowPathV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct PointWire {
            x: f64,
            y: f64,
            z: f64,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            points: Vec<PointWire>,
        }
        let points = Wire::deserialize(deserializer)?
            .points
            .into_iter()
            .map(|point| Point3V1::new(point.x, point.y, point.z))
            .collect::<Result<Vec<_>, _>>()
            .map_err(serde::de::Error::custom)?;
        Self::try_new(points).map_err(serde::de::Error::custom)
    }
}

/// Persisted authored normal-arrow style from direct-root CDML `shape`.
///
/// The renderer owns its visible realization, including short-arrow clipping
/// and scaling; these values are not issued display geometry.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ArrowHeadShapeV1 {
    line_inset: f64,
    total_length: f64,
    half_width: f64,
}

impl ArrowHeadShapeV1 {
    /// Return the closed authored normal-arrow default used when CDML omits `shape`.
    #[must_use]
    pub fn default_authored() -> Self {
        Self::new(8.0, 10.0, 3.0).expect("closed default normal-arrow head is valid")
    }

    #[must_use]
    pub fn new(line_inset: f64, total_length: f64, half_width: f64) -> Option<Self> {
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
    pub const fn line_inset(self) -> f64 {
        self.line_inset
    }
    #[must_use]
    pub const fn total_length(self) -> f64 {
        self.total_length
    }
    #[must_use]
    pub const fn half_width(self) -> f64 {
        self.half_width
    }
}

impl<'de> Deserialize<'de> for ArrowHeadShapeV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            line_inset: f64,
            total_length: f64,
            half_width: f64,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.line_inset, wire.total_length, wire.half_width)
            .ok_or_else(|| serde::de::Error::custom("invalid normal-arrow head dimensions"))
    }
}

/// Closed authored family for curved terminal arrows.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CurvedTerminalArrowKindV1 {
    Electron,
    Retro,
    Normal,
}

impl CurvedTerminalArrowKindV1 {
    #[must_use]
    pub const fn cdml_type(self) -> &'static str {
        match self {
            Self::Electron => "electron",
            Self::Retro => "retro",
            Self::Normal => "curved-normal",
        }
    }
}

/// Closed semantic intent for one authored arrow root.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ArrowProjectionKindV1 {
    Normal {
        head_shape: ArrowHeadShapeV1,
        start_head: bool,
        end_head: bool,
    },
    Equilibrium,
    CurvedEquilibrium,
    CurvedTerminal {
        terminal_kind: CurvedTerminalArrowKindV1,
    },
}

/// Closed refusal for immutable semantic arrow construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ArrowProjectionV1Error {
    #[error("arrow source path requires at least two finite points")]
    TooFewPoints,
    #[error("normal and equilibrium arrows require exactly two source points")]
    StraightCardinality,
    #[error("curved arrows require exactly three source points")]
    CurvedCardinality,
    #[error("arrow source start and end points must not collapse")]
    CollapsedSpan,
}

/// Immutable semantic arrow input for the renderer.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ArrowProjectionV1 {
    target: PresentationTargetV1,
    source_path: ArrowPathV1,
    kind: ArrowProjectionKindV1,
    stroke: PresentationStrokeV1,
}

impl ArrowProjectionV1 {
    /// Construct one valid semantic arrow. Renderer-derived geometry is absent.
    pub fn try_new(
        target: PresentationTargetV1,
        source_path: ArrowPathV1,
        kind: ArrowProjectionKindV1,
        stroke: PresentationStrokeV1,
    ) -> Result<Self, ArrowProjectionV1Error> {
        let points = source_path.points().len();
        match kind {
            ArrowProjectionKindV1::Normal { .. } | ArrowProjectionKindV1::Equilibrium
                if points != 2 =>
            {
                return Err(ArrowProjectionV1Error::StraightCardinality);
            }
            ArrowProjectionKindV1::CurvedEquilibrium
            | ArrowProjectionKindV1::CurvedTerminal { .. }
                if points != 3 =>
            {
                return Err(ArrowProjectionV1Error::CurvedCardinality);
            }
            _ => {}
        }
        let start = source_path
            .points()
            .first()
            .expect("validated source path is nonempty");
        let end = source_path
            .points()
            .last()
            .expect("validated source path is nonempty");
        if start.x() == end.x() && start.y() == end.y() {
            return Err(ArrowProjectionV1Error::CollapsedSpan);
        }
        Ok(Self {
            target,
            source_path,
            kind,
            stroke,
        })
    }

    pub fn normal(
        target: PresentationTargetV1,
        source_points: Vec<Point3V1>,
        head_shape: ArrowHeadShapeV1,
        start_head: bool,
        end_head: bool,
        stroke: PresentationStrokeV1,
    ) -> Result<Self, ArrowProjectionV1Error> {
        Self::try_new(
            target,
            ArrowPathV1::try_new(source_points)?,
            ArrowProjectionKindV1::Normal {
                head_shape,
                start_head,
                end_head,
            },
            stroke,
        )
    }

    #[must_use]
    pub const fn target(&self) -> &PresentationTargetV1 {
        &self.target
    }
    #[must_use]
    pub const fn source_path(&self) -> &ArrowPathV1 {
        &self.source_path
    }
    #[must_use]
    pub const fn kind(&self) -> &ArrowProjectionKindV1 {
        &self.kind
    }
    #[must_use]
    pub const fn stroke(&self) -> &PresentationStrokeV1 {
        &self.stroke
    }
}

impl<'de> Deserialize<'de> for ArrowProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            target: PresentationTargetV1,
            source_path: ArrowPathV1,
            kind: ArrowProjectionKindV1,
            stroke: PresentationStrokeV1,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::try_new(wire.target, wire.source_path, wire.kind, wire.stroke)
            .map_err(serde::de::Error::custom)
    }
}

/// Immutable lower request for one disposable renderer-owned arrow preview.
///
/// It carries only semantic arrow facts and a synthetic target required by the
/// shared presentation-plan grammar. Session fencing and mutation capability
/// remain owned by the calling document workflow.
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationArrowPreviewRequestV1 {
    arrow: ArrowProjectionV1,
}

impl PresentationArrowPreviewRequestV1 {
    /// Create one synthetic-root request from semantic arrow facts.
    pub fn new(
        source_points: Vec<Point3V1>,
        kind: ArrowProjectionKindV1,
        stroke: PresentationStrokeV1,
    ) -> Result<Self, ArrowProjectionV1Error> {
        let target = PresentationTargetV1::try_new(
            None,
            ProjectionLocalObjectKeyV1::from_path_components(&[0])
                .expect("preview target has a nonempty local path"),
            None,
            0,
            PresentationRecordKindV1::Arrow,
        )
        .expect("synthetic preview target has coherent local identity");
        Ok(Self {
            arrow: ArrowProjectionV1::try_new(
                target,
                ArrowPathV1::try_new(source_points)?,
                kind,
                stroke,
            )?,
        })
    }

    /// Return the semantic arrow lowered through the ordinary renderer path.
    #[must_use]
    pub const fn arrow(&self) -> &ArrowProjectionV1 {
        &self.arrow
    }
}

//! Closed geometry for quadratic, two-lane curved equilibrium arrows.

use crate::{
    Point3V1,
    equilibrium_arrow_geometry_v1::{
        EQUILIBRIUM_HALF_SPACING_PT_V1, EQUILIBRIUM_HEAD_HALF_WIDTH_PT_V1,
        EQUILIBRIUM_HEAD_LINE_INSET_PT_V1, EQUILIBRIUM_HEAD_TOTAL_LENGTH_PT_V1,
    },
};

/// Closed-admission refusal for a quadratic curved equilibrium arrow.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurvedEquilibriumArrowGeometryErrorV1 {
    InvalidPoint,
    CollapsedSpan,
    ControlTooNearChord,
}

impl std::fmt::Display for CurvedEquilibriumArrowGeometryErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidPoint => "curved equilibrium arrow points must be finite",
            Self::CollapsedSpan => {
                "curved equilibrium arrow source span is below its fixed geometry minimum"
            }
            Self::ControlTooNearChord => {
                "curved equilibrium arrow endpoint tangents must stay forward and within 45 degrees of the chord"
            }
        })
    }
}

const MINIMUM_SPAN_PT_V1: f64 = 20.0;
const MINIMUM_FORWARD_TANGENT_PT_V1: f64 = 10.0;
const MAXIMUM_TANGENT_DEVIATION_COSINE_V1: f64 = std::f64::consts::FRAC_1_SQRT_2;

/// Backend-issued cubic lanes and opposing terminal heads.
#[derive(Clone, Debug, PartialEq)]
pub struct CurvedEquilibriumArrowGeometryV1 {
    lower: CurvedEquilibriumArrowStartHeadLaneV1,
    upper: CurvedEquilibriumArrowEndHeadLaneV1,
}

impl CurvedEquilibriumArrowGeometryV1 {
    #[must_use]
    pub const fn lower(&self) -> &CurvedEquilibriumArrowStartHeadLaneV1 {
        &self.lower
    }

    #[must_use]
    pub const fn upper(&self) -> &CurvedEquilibriumArrowEndHeadLaneV1 {
        &self.upper
    }
}

/// The lower equilibrium lane is terminated by an arrow head at its start.
#[derive(Clone, Debug, PartialEq)]
pub struct CurvedEquilibriumArrowStartHeadLaneV1 {
    axis: [Point3V1; 4],
    head: [Point3V1; 4],
}

impl CurvedEquilibriumArrowStartHeadLaneV1 {
    #[must_use]
    pub const fn axis(&self) -> &[Point3V1; 4] {
        &self.axis
    }

    #[must_use]
    pub const fn head(&self) -> &[Point3V1; 4] {
        &self.head
    }
}

/// The upper equilibrium lane is terminated by an arrow head at its end.
#[derive(Clone, Debug, PartialEq)]
pub struct CurvedEquilibriumArrowEndHeadLaneV1 {
    axis: [Point3V1; 4],
    head: [Point3V1; 4],
}

impl CurvedEquilibriumArrowEndHeadLaneV1 {
    #[must_use]
    pub const fn axis(&self) -> &[Point3V1; 4] {
        &self.axis
    }

    #[must_use]
    pub const fn head(&self) -> &[Point3V1; 4] {
        &self.head
    }
}

/// Derive two translated quadratic lanes. The lower lane points toward its start;
/// the upper lane points toward its end. Neither cubic endpoint is trimmed.
pub fn curved_equilibrium_arrow_geometry_v1(
    start: Point3V1,
    control: Point3V1,
    end: Point3V1,
) -> Result<CurvedEquilibriumArrowGeometryV1, CurvedEquilibriumArrowGeometryErrorV1> {
    if [start, control, end]
        .into_iter()
        .any(|point| !point.x().is_finite() || !point.y().is_finite() || !point.z().is_finite())
    {
        return Err(CurvedEquilibriumArrowGeometryErrorV1::InvalidPoint);
    }
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let length = dx.hypot(dy);
    if !length.is_finite() || length < MINIMUM_SPAN_PT_V1 {
        return Err(CurvedEquilibriumArrowGeometryErrorV1::CollapsedSpan);
    }
    let chord_x = dx / length;
    let chord_y = dy / length;
    for (tangent_x, tangent_y) in [
        (control.x() - start.x(), control.y() - start.y()),
        (end.x() - control.x(), end.y() - control.y()),
    ] {
        let tangent = tangent_x.hypot(tangent_y);
        let forward = tangent_x * chord_x + tangent_y * chord_y;
        // The forward and 45-degree bounds reject endpoint reversals and cusps,
        // keeping the opposing equilibrium heads legible for every CDML caller.
        if !tangent.is_finite()
            || forward < MINIMUM_FORWARD_TANGENT_PT_V1
            || forward / tangent < MAXIMUM_TANGENT_DEVIATION_COSINE_V1
        {
            return Err(CurvedEquilibriumArrowGeometryErrorV1::ControlTooNearChord);
        }
    }
    let nx = -dy / length;
    let ny = dx / length;
    let translate = |point: Point3V1, sign: f64| {
        Point3V1::new(
            point.x() + nx * EQUILIBRIUM_HALF_SPACING_PT_V1 * sign,
            point.y() + ny * EQUILIBRIUM_HALF_SPACING_PT_V1 * sign,
            point.z(),
        )
        .map_err(|_| CurvedEquilibriumArrowGeometryErrorV1::InvalidPoint)
    };
    let lower = [
        translate(start, -1.0)?,
        translate(control, -1.0)?,
        translate(end, -1.0)?,
    ];
    let upper = [
        translate(start, 1.0)?,
        translate(control, 1.0)?,
        translate(end, 1.0)?,
    ];
    let cubic =
        |points: [Point3V1; 3]| -> Result<[Point3V1; 4], CurvedEquilibriumArrowGeometryErrorV1> {
            let first = Point3V1::new(
                points[0].x() + (2.0 / 3.0) * (points[1].x() - points[0].x()),
                points[0].y() + (2.0 / 3.0) * (points[1].y() - points[0].y()),
                points[0].z() + (2.0 / 3.0) * (points[1].z() - points[0].z()),
            )
            .map_err(|_| CurvedEquilibriumArrowGeometryErrorV1::InvalidPoint)?;
            let second = Point3V1::new(
                points[2].x() + (2.0 / 3.0) * (points[1].x() - points[2].x()),
                points[2].y() + (2.0 / 3.0) * (points[1].y() - points[2].y()),
                points[2].z() + (2.0 / 3.0) * (points[1].z() - points[2].z()),
            )
            .map_err(|_| CurvedEquilibriumArrowGeometryErrorV1::InvalidPoint)?;
            Ok([points[0], first, second, points[2]])
        };
    let head = |tip: Point3V1,
                tangent_x: f64,
                tangent_y: f64|
     -> Result<[Point3V1; 4], CurvedEquilibriumArrowGeometryErrorV1> {
        let magnitude = tangent_x.hypot(tangent_y);
        if !magnitude.is_finite() || magnitude < 10.0 {
            return Err(CurvedEquilibriumArrowGeometryErrorV1::ControlTooNearChord);
        }
        let ux = tangent_x / magnitude;
        let uy = tangent_y / magnitude;
        let px = -uy;
        let py = ux;
        let point = |along: f64, perpendicular: f64| {
            Point3V1::new(
                tip.x() + ux * along + px * perpendicular,
                tip.y() + uy * along + py * perpendicular,
                tip.z(),
            )
            .map_err(|_| CurvedEquilibriumArrowGeometryErrorV1::InvalidPoint)
        };
        Ok([
            tip,
            point(
                EQUILIBRIUM_HEAD_TOTAL_LENGTH_PT_V1,
                EQUILIBRIUM_HEAD_HALF_WIDTH_PT_V1,
            )?,
            point(EQUILIBRIUM_HEAD_LINE_INSET_PT_V1, 0.0)?,
            point(
                EQUILIBRIUM_HEAD_TOTAL_LENGTH_PT_V1,
                -EQUILIBRIUM_HEAD_HALF_WIDTH_PT_V1,
            )?,
        ])
    };
    Ok(CurvedEquilibriumArrowGeometryV1 {
        lower: CurvedEquilibriumArrowStartHeadLaneV1 {
            axis: cubic(lower)?,
            head: head(
                lower[0],
                lower[1].x() - lower[0].x(),
                lower[1].y() - lower[0].y(),
            )?,
        },
        upper: CurvedEquilibriumArrowEndHeadLaneV1 {
            axis: cubic(upper)?,
            head: head(
                upper[2],
                upper[1].x() - upper[2].x(),
                upper[1].y() - upper[2].y(),
            )?,
        },
    })
}

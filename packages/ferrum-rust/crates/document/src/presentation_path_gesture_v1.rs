//! Closed validation for authored multi-point presentation paths.

use crate::PresentationGesturePoint2V1;
use thiserror::Error;

pub const PRESENTATION_PATH_MAXIMUM_POINTS_V1: usize = 256;
pub const PRESENTATION_PATH_MAXIMUM_EXTENT_PT_V1: f64 = 20_000.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationPathKindV1 {
    Polyline,
    Polygon,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PresentationPathGestureV1 {
    kind: PresentationPathKindV1,
    points: Vec<PresentationGesturePoint2V1>,
}

impl PresentationPathGestureV1 {
    pub fn new(
        kind: PresentationPathKindV1,
        points: Vec<PresentationGesturePoint2V1>,
    ) -> Result<Self, PresentationPathGestureErrorV1> {
        let minimum = match kind {
            PresentationPathKindV1::Polyline => 2,
            PresentationPathKindV1::Polygon => 3,
        };
        if points.len() < minimum {
            return Err(PresentationPathGestureErrorV1::InsufficientPoints);
        }
        if points.len() > PRESENTATION_PATH_MAXIMUM_POINTS_V1 {
            return Err(PresentationPathGestureErrorV1::ResourceExhausted);
        }
        let (mut left, mut top, mut right, mut bottom) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for (index, point) in points.iter().enumerate() {
            if points[..index].iter().any(|prior| prior == point) {
                return Err(PresentationPathGestureErrorV1::DegenerateGeometry);
            }
        }
        for point in &points {
            left = left.min(point.x());
            top = top.min(point.y());
            right = right.max(point.x());
            bottom = bottom.max(point.y());
        }
        if right - left > PRESENTATION_PATH_MAXIMUM_EXTENT_PT_V1
            || bottom - top > PRESENTATION_PATH_MAXIMUM_EXTENT_PT_V1
        {
            return Err(PresentationPathGestureErrorV1::ExceedsGeometryLimit);
        }
        if kind == PresentationPathKindV1::Polygon && signed_double_area(&points) == 0.0 {
            return Err(PresentationPathGestureErrorV1::DegenerateGeometry);
        }
        Ok(Self { kind, points })
    }

    #[must_use]
    pub const fn kind(&self) -> PresentationPathKindV1 {
        self.kind
    }

    #[must_use]
    pub fn points(&self) -> &[PresentationGesturePoint2V1] {
        &self.points
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationPathGestureCategoryV1 {
    InsufficientPoints,
    DegenerateGeometry,
    ExceedsGeometryLimit,
    ResourceExhausted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationPathGestureRecoveryV1 {
    AddPoints,
    ChangeGeometry,
    ReduceRequest,
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum PresentationPathGestureErrorV1 {
    #[error("presentation path needs more ordered points")]
    InsufficientPoints,
    #[error("presentation path geometry is degenerate")]
    DegenerateGeometry,
    #[error("presentation path exceeds the geometry limit")]
    ExceedsGeometryLimit,
    #[error("presentation path exceeds the point limit")]
    ResourceExhausted,
}

impl PresentationPathGestureErrorV1 {
    #[must_use]
    pub const fn category(&self) -> PresentationPathGestureCategoryV1 {
        match self {
            Self::InsufficientPoints => PresentationPathGestureCategoryV1::InsufficientPoints,
            Self::DegenerateGeometry => PresentationPathGestureCategoryV1::DegenerateGeometry,
            Self::ExceedsGeometryLimit => PresentationPathGestureCategoryV1::ExceedsGeometryLimit,
            Self::ResourceExhausted => PresentationPathGestureCategoryV1::ResourceExhausted,
        }
    }

    #[must_use]
    pub const fn recovery(&self) -> PresentationPathGestureRecoveryV1 {
        match self {
            Self::InsufficientPoints => PresentationPathGestureRecoveryV1::AddPoints,
            Self::DegenerateGeometry | Self::ExceedsGeometryLimit => {
                PresentationPathGestureRecoveryV1::ChangeGeometry
            }
            Self::ResourceExhausted => PresentationPathGestureRecoveryV1::ReduceRequest,
        }
    }
}

fn signed_double_area(points: &[PresentationGesturePoint2V1]) -> f64 {
    points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .take(points.len())
        .map(|(left, right)| left.x() * right.y() - right.x() * left.y())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> PresentationGesturePoint2V1 {
        PresentationGesturePoint2V1::new(x, y).expect("finite point")
    }

    #[test]
    fn validates_cardinality_degeneracy_and_ordered_polygon_area() {
        assert!(matches!(
            PresentationPathGestureV1::new(PresentationPathKindV1::Polyline, vec![point(0.0, 0.0)]),
            Err(PresentationPathGestureErrorV1::InsufficientPoints)
        ));
        assert!(matches!(
            PresentationPathGestureV1::new(
                PresentationPathKindV1::Polygon,
                vec![point(0.0, 0.0), point(1.0, 0.0), point(2.0, 0.0)]
            ),
            Err(PresentationPathGestureErrorV1::DegenerateGeometry)
        ));
        assert!(matches!(
            PresentationPathGestureV1::new(
                PresentationPathKindV1::Polyline,
                vec![point(0.0, 0.0), point(4.0, 0.0), point(0.0, 0.0)]
            ),
            Err(PresentationPathGestureErrorV1::DegenerateGeometry)
        ));
        assert!(matches!(
            PresentationPathGestureV1::new(
                PresentationPathKindV1::Polygon,
                vec![
                    point(0.0, 0.0),
                    point(4.0, 0.0),
                    point(0.0, 3.0),
                    point(0.0, 0.0),
                ]
            ),
            Err(PresentationPathGestureErrorV1::DegenerateGeometry)
        ));
        let path = PresentationPathGestureV1::new(
            PresentationPathKindV1::Polygon,
            vec![point(0.0, 0.0), point(4.0, 0.0), point(0.0, 3.0)],
        )
        .expect("triangle");
        assert_eq!(path.points().len(), 3);
    }
}

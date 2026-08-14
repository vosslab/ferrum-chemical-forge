//! Bounded Rust-owned geometry for one persistent Wavy line.

use thiserror::Error;

use super::Point3V1;

/// Established scene-space segment length for Wavy creation.
pub const WAVY_SEGMENT_LENGTH_V1: f64 = 12.0;
/// Established maximum scene-space amplitude for Wavy creation.
pub const WAVY_MAX_AMPLITUDE_V1: f64 = 4.0;
/// Established work bound for one Wavy gesture.
///
/// At the V1 segment length this spans about 49,152 scene points, roughly 17 m
/// at 72 points/inch: beyond an ordinary page while bounding one operation.
pub const WAVY_MAX_SEGMENTS_V1: usize = 4096;

/// Complete finite point path for one new Wavy line.
#[derive(Clone, Debug, PartialEq)]
pub struct WavyInsertionV1 {
    points: Vec<Point3V1>,
}

impl WavyInsertionV1 {
    /// Construct one explicit zigzag from two finite scene-space endpoints.
    pub fn new(start: Point3V1, end: Point3V1) -> Result<Self, WavyInsertionV1Error> {
        let dx = end.x() - start.x();
        let dy = end.y() - start.y();
        if !dx.is_finite() || !dy.is_finite() {
            return Err(WavyInsertionV1Error::DerivedGeometryNotFinite);
        }
        let length = dx.hypot(dy);
        if !length.is_finite() {
            return Err(WavyInsertionV1Error::DerivedGeometryNotFinite);
        }
        if length == 0.0 {
            return Err(WavyInsertionV1Error::ZeroLength);
        }
        let segment_estimate = length / WAVY_SEGMENT_LENGTH_V1;
        if segment_estimate > WAVY_MAX_SEGMENTS_V1 as f64 + 0.5 {
            return Err(WavyInsertionV1Error::TooManySegments);
        }
        let segments = (segment_estimate.round_ties_even() as usize).max(2);
        let amplitude = WAVY_MAX_AMPLITUDE_V1.min(length / 6.0);
        let normal_x = -dy / length;
        let normal_y = dx / length;
        if !amplitude.is_finite() || !normal_x.is_finite() || !normal_y.is_finite() {
            return Err(WavyInsertionV1Error::DerivedGeometryNotFinite);
        }
        let mut points = Vec::with_capacity(segments + 1);
        points.push(start);
        for index in 1..segments {
            let fraction = index as f64 / segments as f64;
            let offset = if index % 2 == 0 {
                -amplitude
            } else {
                amplitude
            };
            let x = start.x() + dx * fraction + normal_x * offset;
            let y = start.y() + dy * fraction + normal_y * offset;
            let z = start.z() + (end.z() - start.z()) * fraction;
            points.push(
                Point3V1::new(x, y, z)
                    .map_err(|_| WavyInsertionV1Error::DerivedGeometryNotFinite)?,
            );
        }
        points.push(end);
        Ok(Self { points })
    }

    /// Return every explicit persistent point in path order.
    #[must_use]
    pub fn points(&self) -> &[Point3V1] {
        &self.points
    }
}

/// A Wavy gesture that cannot produce bounded finite persistent geometry.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum WavyInsertionV1Error {
    /// Both endpoints occupy the same scene position.
    #[error("Wavy creation requires distinct start and end points")]
    ZeroLength,
    /// The gesture exceeds the established bounded operation size.
    #[error("Wavy creation exceeds the 4096-segment safety bound")]
    TooManySegments,
    /// Finite endpoints overflowed during derived geometry construction.
    #[error("Wavy creation produced non-finite derived geometry")]
    DerivedGeometryNotFinite,
}

//! BSD-3-derived arithmetic port of RDKit's `straightenDepiction` algorithm.

use std::collections::BTreeMap;

use crate::{GeometryError, Point2, Transform2};

const ALMOST_ZERO: f64 = 1.0e-5;
const INCREMENT_DEGREES: f64 = 30.0;
const HALF_INCREMENT_DEGREES: f64 = INCREMENT_DEGREES / 2.0;
const QUARTER_INCREMENT_DEGREES: f64 = INCREMENT_DEGREES / 4.0;

/// A rotated depiction plus the angle that was applied to its coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct StraightenedDepiction {
    /// Coordinates after applying the selected rotation around the origin.
    pub coordinates: Vec<Point2>,
    /// The applied rotation in radians (positive is counter-clockwise).
    pub rotation_radians: f64,
}

#[derive(Clone, Debug, Default)]
struct ThetaBin {
    average_delta: f64,
    theta_values: Vec<f64>,
}

/// Aligns bond angles to the chemical 30-degree grid using the RDKit policy.
///
/// This is an arithmetic-only Rust port of RDKit's BSD-3 `straightenDepiction`.
/// It intentionally does not call or link RDKit. `minimize_rotation = false`
/// selects the traditional depiction orientation; `true` preserves an already
/// near-grid orientation where the RDKit branch says to do so. Coordinates are
/// unitless drawing coordinates in the crate's y-up Cartesian frame and rotate about
/// the origin. A zero-length bond is deliberately RDKit-compatible: its clamped
/// horizontal direction contributes a zero-angle bin rather than returning
/// [`GeometryError::ZeroLengthVector`]. Callers that reject degenerate bonds must do
/// so before calling this normalization routine.
pub fn straighten_depiction(
    coordinates: &[Point2],
    bonds: &[(usize, usize)],
    minimize_rotation: bool,
) -> Result<StraightenedDepiction, GeometryError> {
    if bonds.is_empty() {
        return Ok(StraightenedDepiction {
            coordinates: coordinates.to_vec(),
            rotation_radians: 0.0,
        });
    }
    let mut bins = BTreeMap::<i32, ThetaBin>::new();
    for &(begin, end) in bonds {
        let begin_point = *coordinates
            .get(begin)
            .ok_or(GeometryError::BondIndexOutOfBounds {
                index: begin,
                len: coordinates.len(),
            })?;
        let end_point = *coordinates
            .get(end)
            .ok_or(GeometryError::BondIndexOutOfBounds {
                index: end,
                len: coordinates.len(),
            })?;
        let mut dx = begin_point.x() - end_point.x();
        let dy = begin_point.y() - end_point.y();
        dx = if dx < 0.0 {
            dx.min(-ALMOST_ZERO)
        } else {
            dx.max(ALMOST_ZERO)
        };
        // This intentionally uses atan(y / x), not atan2. The source clamps x
        // away from zero first, then treats 180-degree-opposite bond directions
        // as the same depicted line.
        let theta = (dy / dx).atan().to_degrees();
        let mut delta = (-theta) % INCREMENT_DEGREES;
        if delta.abs() > HALF_INCREMENT_DEGREES {
            delta -= copy_sign(INCREMENT_DEGREES, delta);
        }
        let key = (delta + copy_sign(0.5, delta)) as i32;
        let bin = bins.entry(key).or_default();
        bin.average_delta += delta;
        bin.theta_values.push(theta);
    }

    let mut smallest_delta = f64::MAX;
    for bin in bins.values_mut() {
        bin.average_delta /= bin.theta_values.len() as f64;
        if bin.average_delta.abs() < smallest_delta.abs() {
            smallest_delta = bin.average_delta;
        }
    }
    let selected = bins
        .values()
        .max_by(|left, right| {
            left.theta_values
                .len()
                .cmp(&right.theta_values.len())
                .then_with(|| {
                    right
                        .average_delta
                        .abs()
                        .total_cmp(&left.average_delta.abs())
                })
        })
        .expect("at least one bond creates a theta bin");
    let mut rotation_degrees = selected.average_delta;
    if !minimize_rotation {
        let mut counts = [0_u32, 0_u32];
        for theta in &selected.theta_values {
            let absolute_theta = (theta + rotation_degrees).abs();
            if absolute_theta < ALMOST_ZERO {
                continue;
            }
            let index = ((absolute_theta + 0.5) / INCREMENT_DEGREES) as usize % 2;
            counts[index] += 1;
        }
        if counts[0] > counts[1] {
            rotation_degrees -= copy_sign(INCREMENT_DEGREES, rotation_degrees);
        }
    } else if smallest_delta.abs() < ALMOST_ZERO
        || (smallest_delta.abs() < rotation_degrees.abs()
            && rotation_degrees.abs() > QUARTER_INCREMENT_DEGREES)
    {
        rotation_degrees = smallest_delta;
    }
    let rotation_radians = rotation_degrees.to_radians();
    if rotation_degrees.abs() <= ALMOST_ZERO {
        return Ok(StraightenedDepiction {
            coordinates: coordinates.to_vec(),
            rotation_radians: 0.0,
        });
    }
    let transform = Transform2::rotation(rotation_radians)?;
    let coordinates = coordinates
        .iter()
        .copied()
        .map(|point| transform.apply(point))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(StraightenedDepiction {
        coordinates,
        rotation_radians,
    })
}

fn copy_sign(magnitude: f64, sign: f64) -> f64 {
    magnitude.copysign(if sign.abs() < ALMOST_ZERO { 1.0 } else { sign })
}

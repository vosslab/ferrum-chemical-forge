//! Deterministic placement of chemistry-engine molecule depictions.

use crate::{GeometryError, Point2};

/// Explicit scale and anchor for one molecule insertion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoleculePlacementV1 {
    bond_length: f64,
    anchor: Point2,
}

impl MoleculePlacementV1 {
    /// Construct a placement with a finite positive target bond length.
    pub fn new(bond_length: f64, anchor: Point2) -> Result<Self, GeometryError> {
        if !bond_length.is_finite() || bond_length <= 0.0 {
            return Err(GeometryError::NonPositiveExtent);
        }
        Ok(Self {
            bond_length,
            anchor,
        })
    }

    /// Return the requested mean bond length in the destination coordinate space.
    #[must_use]
    pub const fn bond_length(self) -> f64 {
        self.bond_length
    }

    /// Return the destination centroid anchor.
    #[must_use]
    pub const fn anchor(self) -> Point2 {
        self.anchor
    }
}

/// Scale, center, and y-flip one atom-order chemistry depiction.
///
/// Chemistry engines use a conventional y-up Cartesian frame, while CDML and Qt
/// scene coordinates are y-down. Every bonded pair contributes equally to the
/// source mean bond length. A one-atom or otherwise bondless graph is translated
/// without an invented scale.
pub fn place_molecule_depiction_v1(
    points: &[Point2],
    bonds: &[(usize, usize)],
    placement: MoleculePlacementV1,
) -> Result<Vec<Point2>, GeometryError> {
    if points.is_empty() {
        return Err(GeometryError::EmptyCoordinateSet);
    }
    let scale = depiction_scale(points, bonds, placement.bond_length)?;
    let count = points.len() as f64;
    let centroid_x = points.iter().map(|point| point.x()).sum::<f64>() / count;
    let centroid_y = points.iter().map(|point| point.y()).sum::<f64>() / count;
    if !centroid_x.is_finite() || !centroid_y.is_finite() {
        return Err(GeometryError::UnrepresentableGeometry);
    }
    points
        .iter()
        .map(|point| {
            Point2::new(
                placement.anchor.x() + (point.x() - centroid_x) * scale,
                placement.anchor.y() - (point.y() - centroid_y) * scale,
            )
        })
        .collect()
}

fn depiction_scale(
    points: &[Point2],
    bonds: &[(usize, usize)],
    target_length: f64,
) -> Result<f64, GeometryError> {
    if bonds.is_empty() {
        return Ok(1.0);
    }
    let mut total = 0.0;
    for &(start, end) in bonds {
        let first = points
            .get(start)
            .ok_or(GeometryError::BondIndexOutOfBounds {
                index: start,
                len: points.len(),
            })?;
        let second = points.get(end).ok_or(GeometryError::BondIndexOutOfBounds {
            index: end,
            len: points.len(),
        })?;
        let length = first.distance_to(*second);
        if !length.is_finite() {
            return Err(GeometryError::UnrepresentableGeometry);
        }
        if length == 0.0 {
            return Err(GeometryError::ZeroLengthBond);
        }
        total += length;
    }
    let mean = total / bonds.len() as f64;
    let scale = target_length / mean;
    if !scale.is_finite() || scale <= 0.0 {
        return Err(GeometryError::UnrepresentableGeometry);
    }
    Ok(scale)
}

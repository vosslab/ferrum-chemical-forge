//! Pointy-top chemical hex-grid arithmetic.

use crate::{GeometryError, Point2};

const HALF_SQRT_3: f64 = 0.866_025_403_784_438_6;
const MAX_GRID_POINTS: usize = 5_000;
const MAX_GRID_EDGES: usize = 8_000;

/// An integer coordinate in the skew pointy-top hex-grid basis.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HexIndex {
    /// Coordinate along the 30-degree basis vector.
    pub n: i64,
    /// Coordinate along the vertical basis vector.
    pub m: i64,
}

/// A finite honeycomb segment, emitted once in deterministic row-major order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HexEdge {
    /// First endpoint.
    pub start: Point2,
    /// Second endpoint.
    pub end: Point2,
}

/// A pointy-top hex grid whose neighbor distance is `spacing`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HexGrid {
    spacing: f64,
    origin: Point2,
}

impl HexGrid {
    /// Creates a finite grid with positive neighbor spacing.
    pub fn new(spacing: f64, origin: Point2) -> Result<Self, GeometryError> {
        if !spacing.is_finite() {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        if spacing <= 0.0 {
            return Err(GeometryError::NonPositiveExtent);
        }
        Ok(Self { spacing, origin })
    }

    /// Returns the two grid basis vectors as points relative to the origin.
    pub fn basis_vectors(self) -> (Point2, Point2) {
        // Construction uses finite spacing, and this simple multiplication remains
        // finite for valid practical canvas spacing.
        (
            Point2::new(self.spacing * HALF_SQRT_3, self.spacing / 2.0).expect("finite grid basis"),
            Point2::new(0.0, self.spacing).expect("finite grid basis"),
        )
    }

    /// Converts a grid index into Cartesian coordinates.
    pub fn point(self, index: HexIndex) -> Result<Point2, GeometryError> {
        Point2::new(
            self.origin.x() + (index.n as f64) * self.spacing * HALF_SQRT_3,
            self.origin.y()
                + (index.n as f64) * self.spacing / 2.0
                + (index.m as f64) * self.spacing,
        )
    }

    /// Returns the deterministic Euclidean-nearest lattice index.
    pub fn nearest_index(self, point: Point2) -> Result<HexIndex, GeometryError> {
        let n_fraction = (point.x() - self.origin.x()) / (self.spacing * HALF_SQRT_3);
        let m_fraction =
            (point.y() - self.origin.y() - n_fraction * self.spacing / 2.0) / self.spacing;
        if !n_fraction.is_finite() || !m_fraction.is_finite() {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        let n_candidates = nearest_axis_candidates(n_fraction)?;
        let m_candidates = nearest_axis_candidates(m_fraction)?;
        let mut best: Option<(f64, HexIndex)> = None;
        for n in n_candidates {
            for m in m_candidates {
                let dn = n as f64 - n_fraction;
                let dm = m as f64 - m_fraction;
                let distance_squared = dn.mul_add(dn, dm.mul_add(dm, dn * dm));
                let candidate = HexIndex { n, m };
                if best.is_none_or(|(distance, index)| {
                    distance_squared < distance
                        || (distance_squared == distance && candidate < index)
                }) {
                    best = Some((distance_squared, candidate));
                }
            }
        }
        Ok(best.expect("four lattice candidates are always examined").1)
    }

    /// Snaps a point to its deterministic Euclidean-nearest lattice vertex.
    pub fn snap(self, point: Point2) -> Result<Point2, GeometryError> {
        self.point(self.nearest_index(point)?)
    }

    /// Returns the point-to-nearest-vertex Euclidean distance.
    pub fn distance_to_grid(self, point: Point2) -> Result<f64, GeometryError> {
        Ok(point.distance_to(self.snap(point)?))
    }

    /// Generates lattice vertices inside an inclusive Cartesian rectangle.
    ///
    /// `None` means the requested display would exceed the fixed 5,000-point UI
    /// budget. It is not an error and lets a renderer omit an impractically dense
    /// dot overlay without silently changing the underlying grid.
    pub fn points_in_rect(
        self,
        minimum: Point2,
        maximum: Point2,
    ) -> Result<Option<Vec<Point2>>, GeometryError> {
        if minimum.x() > maximum.x() || minimum.y() > maximum.y() {
            return Err(GeometryError::InvalidBounds);
        }
        let horizontal_step = self.spacing * HALF_SQRT_3;
        let n_minimum = lattice_index((minimum.x() - self.origin.x()) / horizontal_step, -1)?;
        let n_maximum = lattice_index((maximum.x() - self.origin.x()) / horizontal_step, 1)?;
        let columns = inclusive_count(n_minimum, n_maximum)?;
        let rows = display_count((maximum.y() - minimum.y()) / self.spacing, 2)?;
        if exceeds_budget(columns, rows, MAX_GRID_POINTS) {
            return Ok(None);
        }
        let mut points = Vec::new();
        for n in n_minimum..=n_maximum {
            let y_offset = self.origin.y() + (n as f64) * self.spacing / 2.0;
            let m_minimum = lattice_index((minimum.y() - y_offset) / self.spacing, -1)?;
            let m_maximum = lattice_index((maximum.y() - y_offset) / self.spacing, 1)?;
            for m in m_minimum..=m_maximum {
                let point = self.point(HexIndex { n, m })?;
                if point.x() >= minimum.x()
                    && point.x() <= maximum.x()
                    && point.y() >= minimum.y()
                    && point.y() <= maximum.y()
                {
                    points.push(point);
                }
            }
        }
        Ok(Some(points))
    }

    /// Generates the pointy-top honeycomb segments inside an inclusive rectangle.
    ///
    /// `None` carries the same bounded-display meaning as [`Self::points_in_rect`].
    pub fn honeycomb_edges_in_rect(
        self,
        minimum: Point2,
        maximum: Point2,
    ) -> Result<Option<Vec<HexEdge>>, GeometryError> {
        if minimum.x() > maximum.x() || minimum.y() > maximum.y() {
            return Err(GeometryError::InvalidBounds);
        }
        let center_step_x = self.spacing * 2.0 * HALF_SQRT_3;
        let center_step_y = self.spacing * 1.5;
        let margin = self.spacing * 2.0;
        let x_minimum = minimum.x() - margin;
        let x_maximum = maximum.x() + margin;
        let y_minimum = minimum.y() - margin;
        let y_maximum = maximum.y() + margin;
        let rows = display_count((y_maximum - y_minimum) / center_step_y, 4)?;
        let columns = display_count((x_maximum - x_minimum) / center_step_x, 4)?;
        if exceeds_budget(rows, columns, MAX_GRID_EDGES / 3) {
            return Ok(None);
        }
        let row_minimum = lattice_index((y_minimum - self.origin.y()) / center_step_y, -1)?;
        let row_maximum = lattice_index((y_maximum - self.origin.y()) / center_step_y, 1)?;
        let angles = [
            std::f64::consts::FRAC_PI_6,
            std::f64::consts::FRAC_PI_2,
            5.0 * std::f64::consts::FRAC_PI_6,
            7.0 * std::f64::consts::FRAC_PI_6,
        ];
        let mut edges = Vec::new();
        for row in row_minimum..=row_maximum {
            let center_y = self.origin.y() + (row as f64) * center_step_y;
            let offset = if row.rem_euclid(2) == 0 {
                0.0
            } else {
                center_step_x / 2.0
            };
            let column_minimum =
                lattice_index((x_minimum - self.origin.x() - offset) / center_step_x, -1)?;
            let column_maximum =
                lattice_index((x_maximum - self.origin.x() - offset) / center_step_x, 1)?;
            for column in column_minimum..=column_maximum {
                let center_x = self.origin.x() + offset + (column as f64) * center_step_x;
                for index in 0..3 {
                    let start = Point2::new(
                        center_x + self.spacing * angles[index].cos(),
                        center_y + self.spacing * angles[index].sin(),
                    )?;
                    let end = Point2::new(
                        center_x + self.spacing * angles[index + 1].cos(),
                        center_y + self.spacing * angles[index + 1].sin(),
                    )?;
                    if inside(start, minimum, maximum) && inside(end, minimum, maximum) {
                        edges.push(HexEdge { start, end });
                    }
                }
            }
        }
        Ok(Some(edges))
    }
}

/// Returns the floor and following index needed for a nearest-grid comparison.
///
/// `i64::MAX as f64` is exactly `2^63`, so it is outside the range of values
/// that can be converted to `i64` without saturation. The lower endpoint,
/// `i64::MIN as f64`, is representable and can participate because its following
/// index is created with checked arithmetic.
fn nearest_axis_candidates(ratio: f64) -> Result<[i64; 2], GeometryError> {
    let floor = ratio.floor();
    if floor < i64::MIN as f64 || floor >= i64::MAX as f64 {
        return Err(GeometryError::GridIndexUnrepresentable);
    }
    let lower = floor as i64;
    let upper = lower
        .checked_add(1)
        .ok_or(GeometryError::GridIndexUnrepresentable)?;
    Ok([lower, upper])
}

/// Converts a finite lattice ratio to an index with a checked inclusive margin.
///
/// Floating-point values near `i64` limits cannot preserve every individual integer.
/// Refusing that ambiguous fringe is preferable to a saturating cast followed by an
/// overflowing range expression in a public renderer helper.
fn lattice_index(ratio: f64, margin: i64) -> Result<i64, GeometryError> {
    let rounded = if margin < 0 {
        ratio.floor()
    } else {
        ratio.ceil()
    };
    let limit = i64::MAX as f64;
    if !rounded.is_finite() || rounded <= -limit || rounded >= limit {
        return Err(GeometryError::GridIndexUnrepresentable);
    }
    (rounded as i64)
        .checked_add(margin)
        .ok_or(GeometryError::GridIndexUnrepresentable)
}

/// Returns an inclusive integer range length without signed subtraction overflow.
fn inclusive_count(minimum: i64, maximum: i64) -> Result<usize, GeometryError> {
    maximum
        .checked_sub(minimum)
        .and_then(|difference| difference.checked_add(1))
        .and_then(|count| usize::try_from(count).ok())
        .ok_or(GeometryError::GridIndexUnrepresentable)
}

/// Converts a non-negative finite display span into a checked count plus padding.
fn display_count(span: f64, padding: usize) -> Result<usize, GeometryError> {
    let rounded = span.ceil();
    if !rounded.is_finite() || rounded < 0.0 || rounded > usize::MAX as f64 {
        return Err(GeometryError::GridIndexUnrepresentable);
    }
    (rounded as usize)
        .checked_add(padding)
        .ok_or(GeometryError::GridIndexUnrepresentable)
}

/// Tests a product against a budget without constructing an overflow-prone product.
fn exceeds_budget(first: usize, second: usize, budget: usize) -> bool {
    first > budget || second > budget || first > budget / second.max(1)
}

fn inside(point: Point2, minimum: Point2, maximum: Point2) -> bool {
    point.x() >= minimum.x()
        && point.x() <= maximum.x()
        && point.y() >= minimum.y()
        && point.y() <= maximum.y()
}

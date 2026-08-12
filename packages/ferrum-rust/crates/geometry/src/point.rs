//! Ferrum's finite two-dimensional coordinate representation.

use std::fmt;

/// A finite Cartesian point in Ferrum's document and depiction coordinate space.
///
/// Coordinates use the caller's drawing units; Ferrum does not attach a physical
/// unit or DPI conversion to them. Geometry arithmetic uses a conventional y-up
/// Cartesian frame, with positive angles counter-clockwise. A y-down renderer must
/// convert at its own boundary before or after using this API.
///
/// This is the primary representation used by this crate. Conversion traits at
/// the boundary allow renderers to use `kurbo` and numerical routines to use
/// `nalgebra` without making either dependency part of Ferrum's public model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    x: f64,
    y: f64,
}

impl Point2 {
    /// Creates a finite point.
    pub fn new(x: f64, y: f64) -> Result<Self, GeometryError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        Ok(Self { x, y })
    }

    /// Returns the horizontal coordinate.
    pub const fn x(self) -> f64 {
        self.x
    }

    /// Returns the vertical coordinate.
    pub const fn y(self) -> f64 {
        self.y
    }

    /// Returns this point as the maintained renderer path type.
    pub const fn to_kurbo(self) -> kurbo::Point {
        kurbo::Point::new(self.x, self.y)
    }

    /// Returns this point as the maintained linear-algebra point type.
    pub const fn to_nalgebra(self) -> nalgebra::Point2<f64> {
        nalgebra::Point2::new(self.x, self.y)
    }

    /// Computes Euclidean distance.
    pub fn distance_to(self, other: Self) -> f64 {
        (self - other).length()
    }

    /// Returns this point translated by a scaled finite vector.
    pub fn offset(self, vector: Vector2, scale: f64) -> Result<Self, GeometryError> {
        if !scale.is_finite() {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        Self::new(self.x + vector.x * scale, self.y + vector.y * scale)
    }
}

impl TryFrom<kurbo::Point> for Point2 {
    type Error = GeometryError;

    fn try_from(value: kurbo::Point) -> Result<Self, Self::Error> {
        Self::new(value.x, value.y)
    }
}

impl TryFrom<nalgebra::Point2<f64>> for Point2 {
    type Error = GeometryError;

    fn try_from(value: nalgebra::Point2<f64>) -> Result<Self, Self::Error> {
        Self::new(value.x, value.y)
    }
}

impl fmt::Display for Point2 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "({}, {})", self.x, self.y)
    }
}

/// A finite two-dimensional displacement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector2 {
    x: f64,
    y: f64,
}

impl Vector2 {
    /// Creates a finite displacement.
    pub fn new(x: f64, y: f64) -> Result<Self, GeometryError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        Ok(Self { x, y })
    }

    /// Returns the horizontal component.
    pub const fn x(self) -> f64 {
        self.x
    }

    /// Returns the vertical component.
    pub const fn y(self) -> f64 {
        self.y
    }

    /// Returns the vector magnitude.
    pub fn length(self) -> f64 {
        self.x.hypot(self.y)
    }

    /// Returns the dot product.
    pub const fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Returns the left-handed perpendicular vector.
    pub const fn perpendicular_left(self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }

    /// Returns a unit vector, or a descriptive error for a zero vector.
    pub fn normalized(self) -> Result<Self, GeometryError> {
        let length = self.length();
        if length == 0.0 {
            return Err(GeometryError::ZeroLengthVector);
        }
        Self::new(self.x / length, self.y / length)
    }
}

impl std::ops::Sub<Point2> for Point2 {
    type Output = Vector2;

    fn sub(self, other: Point2) -> Self::Output {
        Vector2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

/// Failures that can occur when constructing or applying finite geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum GeometryError {
    /// A public coordinate, extent, or transform parameter was not finite.
    #[error("geometry values must be finite")]
    NonFiniteCoordinate,
    /// An operation requiring a direction received coincident points.
    #[error("geometry requires a non-zero-length vector")]
    ZeroLengthVector,
    /// A positive extent such as a grid spacing or wedge width was invalid.
    #[error("geometry extent must be positive")]
    NonPositiveExtent,
    /// A display rectangle has its minimum beyond its maximum.
    #[error("geometry rectangle minimum must not exceed its maximum")]
    InvalidBounds,
    /// A finite canvas coordinate cannot be represented in the integer grid basis.
    #[error("hex grid coordinate-to-spacing ratio is not representable")]
    GridIndexUnrepresentable,
    /// A bond refers to an atom position that was not supplied.
    #[error("bond endpoint {index} is outside the coordinate array of length {len}")]
    BondIndexOutOfBounds { index: usize, len: usize },
}

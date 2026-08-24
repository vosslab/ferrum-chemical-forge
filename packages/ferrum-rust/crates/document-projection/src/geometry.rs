//! Intrinsically validated geometry carried by immutable projections.

use thiserror::Error;

use crate::molecule::CompactGroupV1Error;

/// Reusable finite Cartesian coordinates carried by document-geometry projections.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct Point3V1 {
    x: f64,
    y: f64,
    z: f64,
}

impl Point3V1 {
    /// Construct a finite coordinate.
    pub fn new(x: f64, y: f64, z: f64) -> Result<Self, ProjectionError> {
        for (axis, value) in [("x", x), ("y", y), ("z", z)] {
            if !value.is_finite() {
                return Err(ProjectionError::NonFiniteCoordinate { axis });
            }
        }
        Ok(Self { x, y, z })
    }

    /// Return the x coordinate.
    #[must_use]
    pub fn x(self) -> f64 {
        self.x
    }

    /// Return the y coordinate.
    #[must_use]
    pub fn y(self) -> f64 {
        self.y
    }

    /// Return the z coordinate.
    #[must_use]
    pub fn z(self) -> f64 {
        self.z
    }
}

/// A finite scalar whose sign is meaningful and whose magnitude is nonzero.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct NonZeroFiniteV1(f64);

impl NonZeroFiniteV1 {
    /// Construct a finite nonzero scalar without discarding its sign.
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        (value.is_finite() && value != 0.0).then_some(Self(value))
    }

    /// Return the carried signed scalar.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

/// Projection construction rejected a required or invalid typed fact.
#[derive(Debug, Error)]
pub enum ProjectionError {
    /// A renderable atom lacked its required point child.
    #[error("{context}: required point is absent")]
    MissingPoint { context: String },
    /// A source scalar could not be parsed for the named projection fact.
    #[error("{context}: {field} value {value:?} is invalid")]
    InvalidValue {
        context: String,
        field: &'static str,
        value: String,
    },
    /// A coordinate was not finite after CDML unit conversion.
    #[error("coordinate {axis} is not finite")]
    NonFiniteCoordinate { axis: &'static str },
    /// A typed compact-group record has an invalid or unsupported V1 fact.
    #[error("compact-group at {path} is invalid: {source}")]
    CompactGroup {
        path: String,
        #[source]
        source: CompactGroupV1Error,
    },
    /// A required compact-group typed field is absent or malformed.
    #[error("compact-group at {path} has invalid {field}: {value:?}")]
    InvalidCompactGroupField {
        path: String,
        field: &'static str,
        value: String,
    },
}

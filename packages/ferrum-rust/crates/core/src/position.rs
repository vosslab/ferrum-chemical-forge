use serde::{Deserialize, Serialize};

use crate::ModelError;

/// A finite 3D coordinate retained without assigning chemistry meaning.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Position {
    x: f64,
    y: f64,
    z: f64,
}

impl Position {
    /// Construct a finite position.
    pub fn new(x: f64, y: f64, z: f64) -> Result<Self, ModelError> {
        let value = Self { x, y, z };
        value.validate()?;
        Ok(value)
    }
    /// Return x.
    #[must_use]
    pub fn x(self) -> f64 {
        self.x
    }
    /// Return y.
    #[must_use]
    pub fn y(self) -> f64 {
        self.y
    }
    /// Return z.
    #[must_use]
    pub fn z(self) -> f64 {
        self.z
    }
    pub(crate) fn validate(self) -> Result<(), ModelError> {
        for (axis, value) in [("x", self.x), ("y", self.y), ("z", self.z)] {
            if !value.is_finite() {
                return Err(ModelError::NonFiniteCoordinate { axis });
            }
        }
        Ok(())
    }
    pub(crate) fn canonical(self) -> String {
        format!(
            "{:016x}{:016x}{:016x}",
            self.x.to_bits(),
            self.y.to_bits(),
            self.z.to_bits()
        )
    }
}

#[derive(Deserialize)]
struct WirePosition {
    x: f64,
    y: f64,
    z: f64,
}
impl<'de> Deserialize<'de> for Position {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WirePosition::deserialize(deserializer)?;
        Self::new(wire.x, wire.y, wire.z).map_err(serde::de::Error::custom)
    }
}

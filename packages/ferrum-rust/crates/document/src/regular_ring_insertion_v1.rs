//! Private detached saturated regular-ring authoring facts.
//!
//! The operation persists only ordinary carbon atoms and `n1` bonds.  Its
//! request and preview vertices are runtime authoring facts, not CDML metadata.

use thiserror::Error;

use crate::{
    DocumentBondOrderV1, MoleculeInsertionAtomV1, MoleculeInsertionBondV1, MoleculeInsertionV1,
    Point3V1,
};

/// Closed cardinality range for the detached regular-ring family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegularRingSizeV1(u8);

impl RegularRingSizeV1 {
    /// Admit the ordinary three through eight member regular-ring family.
    pub fn new(value: u8) -> Result<Self, RegularRingInsertionErrorV1> {
        if !(3..=8).contains(&value) {
            return Err(RegularRingInsertionErrorV1::Size(value));
        }
        Ok(Self(value))
    }
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// The only documented V1 orientation in y-down document coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegularRingOrientationV1 {
    FlatTop,
}

/// Private immutable intent for one detached saturated carbon regular ring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DetachedRegularRingInsertionV1 {
    size: RegularRingSizeV1,
    center: Point3V1,
    side_length: f64,
    orientation: RegularRingOrientationV1,
}

impl DetachedRegularRingInsertionV1 {
    pub fn new(
        size: RegularRingSizeV1,
        center: Point3V1,
        side_length: f64,
        orientation: RegularRingOrientationV1,
    ) -> Result<Self, RegularRingInsertionErrorV1> {
        if !side_length.is_finite() || side_length <= 0.0 {
            return Err(RegularRingInsertionErrorV1::SideLength);
        }
        Ok(Self {
            size,
            center,
            side_length,
            orientation,
        })
    }
    #[must_use]
    pub const fn size(self) -> RegularRingSizeV1 {
        self.size
    }
    #[must_use]
    pub const fn center(self) -> Point3V1 {
        self.center
    }
    #[must_use]
    pub const fn side_length(self) -> f64 {
        self.side_length
    }
    #[must_use]
    pub const fn orientation(self) -> RegularRingOrientationV1 {
        self.orientation
    }

    /// Produce the exact clockwise, flat-top, y-down polygon used by preview and commit.
    pub fn vertices(self) -> Result<Vec<Point3V1>, RegularRingInsertionErrorV1> {
        let count = f64::from(self.size.get());
        let radius = self.side_length / (2.0 * (std::f64::consts::PI / count).sin());
        if !radius.is_finite() {
            return Err(RegularRingInsertionErrorV1::Geometry);
        }
        (0..self.size.get())
            .map(|index| {
                // Start at the upper-right corner of a flat-top polygon; increasing angles
                // are clockwise because document y increases downward.
                let angle = -std::f64::consts::FRAC_PI_2
                    + std::f64::consts::PI / count
                    + std::f64::consts::TAU * f64::from(index) / count;
                Point3V1::new(
                    self.center.x() + radius * angle.cos(),
                    self.center.y() + radius * angle.sin(),
                    self.center.z(),
                )
                .map_err(|_| RegularRingInsertionErrorV1::Geometry)
            })
            .collect()
    }

    /// Build the complete ordinary CDML molecule before any session identity allocation.
    pub fn molecule(self) -> Result<MoleculeInsertionV1, RegularRingInsertionErrorV1> {
        let atoms = self
            .vertices()?
            .into_iter()
            .map(|point| {
                MoleculeInsertionAtomV1::new("C", point, None, None, None)
                    .map_err(|_| RegularRingInsertionErrorV1::Geometry)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let count = atoms.len();
        let bonds = (0..count)
            .map(|index| {
                MoleculeInsertionBondV1::new(
                    index,
                    (index + 1) % count,
                    DocumentBondOrderV1::Single,
                )
            })
            .collect();
        MoleculeInsertionV1::new(atoms, bonds).map_err(|_| RegularRingInsertionErrorV1::Geometry)
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum RegularRingInsertionErrorV1 {
    #[error("regular ring size must be from 3 through 8, got {0}")]
    Size(u8),
    #[error("regular ring side length must be positive and finite")]
    SideLength,
    #[error("regular ring geometry must remain finite")]
    Geometry,
}

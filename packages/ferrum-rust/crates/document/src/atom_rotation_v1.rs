//! Closed immutable intent for rotating durable direct-core atoms.

use std::collections::HashSet;

use thiserror::Error;

use super::PersistentId;

/// One direct-root molecule and direct-core atom pair.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AtomRotationTargetV1 {
    molecule_id: PersistentId,
    atom_id: PersistentId,
}

impl AtomRotationTargetV1 {
    pub fn new(
        molecule_id: impl Into<String>,
        atom_id: impl Into<String>,
    ) -> Result<Self, AtomRotationV1Error> {
        let molecule_id = PersistentId::new(molecule_id.into())
            .map_err(|_| AtomRotationV1Error::InvalidMoleculeId)?;
        let atom_id =
            PersistentId::new(atom_id.into()).map_err(|_| AtomRotationV1Error::InvalidAtomId)?;
        Ok(Self {
            molecule_id,
            atom_id,
        })
    }

    #[must_use]
    pub fn molecule_id(&self) -> &PersistentId {
        &self.molecule_id
    }

    #[must_use]
    pub fn atom_id(&self) -> &PersistentId {
        &self.atom_id
    }
}

/// One fully validated scene-space atom rotation.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomRotationV1 {
    targets: Vec<AtomRotationTargetV1>,
    center_x: f64,
    center_y: f64,
    angle_radians: f64,
}

impl AtomRotationV1 {
    pub fn new(
        targets: Vec<AtomRotationTargetV1>,
        center_x: f64,
        center_y: f64,
        angle_radians: f64,
    ) -> Result<Self, AtomRotationV1Error> {
        if targets.is_empty() {
            return Err(AtomRotationV1Error::EmptyTargets);
        }
        let mut unique = HashSet::with_capacity(targets.len());
        if targets.iter().any(|target| !unique.insert(target.clone())) {
            return Err(AtomRotationV1Error::DuplicateTarget);
        }
        if !center_x.is_finite() || !center_y.is_finite() {
            return Err(AtomRotationV1Error::NonFiniteCenter);
        }
        if !angle_radians.is_finite() {
            return Err(AtomRotationV1Error::NonFiniteAngle);
        }
        Ok(Self {
            targets,
            center_x,
            center_y,
            angle_radians,
        })
    }

    #[must_use]
    pub fn targets(&self) -> &[AtomRotationTargetV1] {
        &self.targets
    }

    #[must_use]
    pub const fn center(&self) -> (f64, f64) {
        (self.center_x, self.center_y)
    }

    #[must_use]
    pub const fn angle_radians(&self) -> f64 {
        self.angle_radians
    }
}

/// Invalid atom-rotation intent rejected before document resolution.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AtomRotationV1Error {
    #[error("atom rotation requires a valid persistent molecule ID")]
    InvalidMoleculeId,
    #[error("atom rotation requires a valid persistent atom ID")]
    InvalidAtomId,
    #[error("atom rotation requires at least one atom")]
    EmptyTargets,
    #[error("atom rotation targets must be unique")]
    DuplicateTarget,
    #[error("atom rotation center must contain finite scene-point values")]
    NonFiniteCenter,
    #[error("atom rotation angle must be finite radians")]
    NonFiniteAngle,
}

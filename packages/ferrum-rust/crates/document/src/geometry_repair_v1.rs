//! Closed document-owned requests for supported geometry repair.

use std::collections::HashSet;

use thiserror::Error;

use super::PersistentId;

/// Geometry repair kinds with an implemented lossless document adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeometryRepairKindV1 {
    SnapToHexGrid,
    StraightenBonds,
    NormalizeBondLengths,
    NormalizeBondAngles,
    NormalizeRings,
}

/// One validated multi-molecule geometry-repair request.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryRepairV1 {
    molecule_ids: Vec<PersistentId>,
    kind: GeometryRepairKindV1,
    target_spacing_points: f64,
}

impl GeometryRepairV1 {
    pub fn new(
        molecule_ids: Vec<String>,
        kind: GeometryRepairKindV1,
        target_spacing_points: f64,
    ) -> Result<Self, GeometryRepairV1Error> {
        if molecule_ids.is_empty() {
            return Err(GeometryRepairV1Error::EmptyMolecules);
        }
        let molecule_ids = molecule_ids
            .into_iter()
            .map(|id| PersistentId::new(id).map_err(|_| GeometryRepairV1Error::InvalidMoleculeId))
            .collect::<Result<Vec<_>, _>>()?;
        let mut unique = HashSet::with_capacity(molecule_ids.len());
        if molecule_ids.iter().any(|id| !unique.insert(id.clone())) {
            return Err(GeometryRepairV1Error::DuplicateMolecule);
        }
        if !target_spacing_points.is_finite() || target_spacing_points <= 0.0 {
            return Err(GeometryRepairV1Error::InvalidTargetSpacing);
        }
        Ok(Self {
            molecule_ids,
            kind,
            target_spacing_points,
        })
    }

    #[must_use]
    pub fn molecule_ids(&self) -> &[PersistentId] {
        &self.molecule_ids
    }

    #[must_use]
    pub const fn kind(&self) -> GeometryRepairKindV1 {
        self.kind
    }

    #[must_use]
    pub const fn target_spacing_points(&self) -> f64 {
        self.target_spacing_points
    }
}

/// Invalid geometry-repair intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum GeometryRepairV1Error {
    #[error("geometry repair requires at least one durable molecule")]
    EmptyMolecules,
    #[error("geometry repair requires valid persistent molecule IDs")]
    InvalidMoleculeId,
    #[error("geometry repair molecule IDs must be unique")]
    DuplicateMolecule,
    #[error("geometry repair target spacing must be finite and greater than zero")]
    InvalidTargetSpacing,
}

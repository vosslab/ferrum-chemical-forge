//! Immutable, provenance-bound results for native clean-geometry preparation.

use std::collections::HashSet;

use ferrum_geometry::Point2;
use thiserror::Error;

use super::DocumentObjectIdV1;

/// One source-order-aligned molecule layout prepared outside the document session.
#[derive(Clone, Debug, PartialEq)]
pub struct CleanGeometryMoleculeV1 {
    molecule_id: DocumentObjectIdV1,
    positions: Vec<Point2>,
}

impl CleanGeometryMoleculeV1 {
    /// Construct one nonempty, finite coordinate result.
    pub fn new(
        molecule_id: DocumentObjectIdV1,
        positions: Vec<Point2>,
    ) -> Result<Self, CleanGeometryUpdateV1Error> {
        if positions.is_empty() {
            return Err(CleanGeometryUpdateV1Error::EmptyPositions);
        }
        Ok(Self {
            molecule_id,
            positions,
        })
    }

    /// Return the durable molecule selector used for preparation.
    #[must_use]
    pub fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return direct-atom-source-order replacement positions.
    #[must_use]
    pub fn positions(&self) -> &[Point2] {
        &self.positions
    }
}

/// One complete clean-geometry result prepared from an exact document observation.
#[derive(Clone, Debug, PartialEq)]
pub struct CleanGeometryUpdateV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    molecules: Vec<CleanGeometryMoleculeV1>,
}

impl CleanGeometryUpdateV1 {
    /// Construct a nonempty update with unique durable molecule selectors.
    pub fn new(
        source_revision: u64,
        source_digest: [u8; 32],
        molecules: Vec<CleanGeometryMoleculeV1>,
    ) -> Result<Self, CleanGeometryUpdateV1Error> {
        if molecules.is_empty() {
            return Err(CleanGeometryUpdateV1Error::EmptyMolecules);
        }
        let mut unique = HashSet::with_capacity(molecules.len());
        if molecules
            .iter()
            .any(|molecule| !unique.insert(molecule.molecule_id.clone()))
        {
            return Err(CleanGeometryUpdateV1Error::DuplicateMolecule);
        }
        Ok(Self {
            source_revision,
            source_digest,
            molecules,
        })
    }

    /// Return the document revision used for native preparation.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the exact document digest used for native preparation.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return every prepared target in requested order.
    #[must_use]
    pub fn molecules(&self) -> &[CleanGeometryMoleculeV1] {
        &self.molecules
    }
}

/// Invalid prepared clean-geometry result.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CleanGeometryUpdateV1Error {
    /// At least one durable molecule is required.
    #[error("clean geometry requires at least one molecule")]
    EmptyMolecules,
    /// Every target must carry at least one direct atom position.
    #[error("clean geometry requires at least one atom position per molecule")]
    EmptyPositions,
    /// A durable molecule may appear only once.
    #[error("clean geometry molecule targets must be unique")]
    DuplicateMolecule,
}

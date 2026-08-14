//! Revision-bound complete coordinate replacement for one durable molecule.

use thiserror::Error;

use super::{DocumentObjectIdV1, Point3V1};

/// Immutable atom-order-aligned coordinates prepared from one document revision.
///
/// The source provenance is part of the value rather than a frontend convention.
/// A session accepts this update only while both its monotonic revision and content
/// digest still match, then validates the target molecule and exact atom count.
#[derive(Clone, Debug, PartialEq)]
pub struct MoleculeCoordinateUpdateV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    positions: Vec<Point3V1>,
}

impl MoleculeCoordinateUpdateV1 {
    /// Construct one complete nonempty coordinate update.
    pub fn new(
        source_revision: u64,
        source_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
        positions: Vec<Point3V1>,
    ) -> Result<Self, MoleculeCoordinateUpdateV1Error> {
        if positions.is_empty() {
            return Err(MoleculeCoordinateUpdateV1Error::Empty);
        }
        Ok(Self {
            source_revision,
            source_digest,
            molecule_id,
            positions,
        })
    }

    /// Return the session revision whose chemistry facts were prepared.
    #[must_use]
    pub fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the exact content digest whose chemistry facts were prepared.
    #[must_use]
    pub fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return the durable molecule selector used for preparation.
    #[must_use]
    pub fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return replacement points in direct typed-atom source order.
    #[must_use]
    pub fn positions(&self) -> &[Point3V1] {
        &self.positions
    }
}

/// Construction failure for a complete molecule coordinate update.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum MoleculeCoordinateUpdateV1Error {
    /// Coordinate generation cannot operate on an atomless molecule.
    #[error("molecule coordinate update requires at least one atom position")]
    Empty,
}

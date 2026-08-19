//! Atomic, revision-bound Point3 coordinate replacement for several molecules.

use std::collections::HashSet;

use thiserror::Error;

use super::MoleculeCoordinateUpdateV1;

/// Complete Point3 coordinate replacements prepared from one source snapshot.
///
/// The common source fence prevents a caller from accidentally treating a
/// sequence of one-molecule mutations as one operation. Each entry also keeps
/// its own provenance so an entry cannot silently be reused from another
/// observation.
#[derive(Clone, Debug, PartialEq)]
pub struct MoleculeCoordinateBatchUpdateV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    updates: Vec<MoleculeCoordinateUpdateV1>,
}

impl MoleculeCoordinateBatchUpdateV1 {
    /// Construct a nonempty batch whose unique targets all share one snapshot.
    pub fn new(
        source_revision: u64,
        source_digest: [u8; 32],
        updates: Vec<MoleculeCoordinateUpdateV1>,
    ) -> Result<Self, MoleculeCoordinateBatchUpdateV1Error> {
        if updates.is_empty() {
            return Err(MoleculeCoordinateBatchUpdateV1Error::Empty);
        }

        let mut targets = HashSet::with_capacity(updates.len());
        for update in &updates {
            if update.source_revision() != source_revision
                || update.source_digest() != &source_digest
            {
                return Err(MoleculeCoordinateBatchUpdateV1Error::SourceMismatch);
            }
            if !targets.insert(update.molecule_id().clone()) {
                return Err(MoleculeCoordinateBatchUpdateV1Error::DuplicateMolecule);
            }
        }

        Ok(Self {
            source_revision,
            source_digest,
            updates,
        })
    }

    /// Return the session revision shared by every prepared target.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the exact content digest shared by every prepared target.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return every complete target replacement in requested order.
    #[must_use]
    pub fn updates(&self) -> &[MoleculeCoordinateUpdateV1] {
        &self.updates
    }
}

/// Construction failure for a Point3 coordinate replacement batch.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum MoleculeCoordinateBatchUpdateV1Error {
    /// A batch with no targets cannot represent an atomic coordinate operation.
    #[error("molecule coordinate batch requires at least one molecule")]
    Empty,
    /// A durable molecule may appear only once in one batch.
    #[error("molecule coordinate batch molecule targets must be unique")]
    DuplicateMolecule,
    /// All entries must have been prepared from the batch's exact observation.
    #[error("molecule coordinate batch entries must share the batch source snapshot")]
    SourceMismatch,
}

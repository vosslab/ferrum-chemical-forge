//! Immutable, revision-bound document observations for frontend projections.

use super::{DocumentProjectionV1, DocumentSnapshot, ProjectionError, TypedDocument};

/// The complete Rust-owned document observation available before render-plan
/// resolution is part of the document dependency graph.
///
/// This value is intentionally named `SessionDocumentObservationV1`, rather
/// than `SessionObservationV1`: the latter is reserved for the later closed
/// snapshot/projection/render-plan composition. Both members are constructed
/// from one accepted retained state; callers cannot stitch a snapshot from one
/// revision to a projection from another. The API layer supplies its explicit
/// verified depiction metrics and composes that final frontend observation;
/// `ferrum-document` neither chooses metrics nor provides a render fallback.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionDocumentObservationV1 {
    snapshot: DocumentSnapshot,
    projection: DocumentProjectionV1,
}

impl SessionDocumentObservationV1 {
    /// Construct an observation from one retained document state and snapshot.
    ///
    /// This is crate-private so foreign clients cannot forge matching-looking
    /// revision or digest provenance.
    pub(crate) fn from_state(
        document: &TypedDocument,
        snapshot: DocumentSnapshot,
    ) -> Result<Self, ProjectionError> {
        let projection = DocumentProjectionV1::from_snapshot(document, &snapshot)?;
        debug_assert_eq!(snapshot.revision(), projection.revision());
        debug_assert_eq!(snapshot.digest(), projection.digest());
        debug_assert_eq!(snapshot.is_dirty(), projection.is_dirty());
        Ok(Self {
            snapshot,
            projection,
        })
    }

    /// Return the immutable authoritative snapshot.
    #[must_use]
    pub fn snapshot(&self) -> &DocumentSnapshot {
        &self.snapshot
    }

    /// Return the immutable presentation projection from the same state.
    #[must_use]
    pub fn projection(&self) -> &DocumentProjectionV1 {
        &self.projection
    }
}

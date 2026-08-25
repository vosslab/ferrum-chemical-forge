//! Immutable, revision-bound document observations for frontend projections.

use super::{
    DocumentObjectIdV1, DocumentProjectionV1, DocumentSnapshot, DocumentStereoDepictionReportV1,
    ProjectionError, TypedDocument, TypedDocumentError,
};

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
    /// Construct an observation from one authoritative retained snapshot.
    ///
    /// This is crate-private so foreign clients cannot forge matching-looking
    /// revision or digest provenance.
    pub(crate) fn from_snapshot(snapshot: DocumentSnapshot) -> Result<Self, ProjectionError> {
        let projection =
            crate::projection_adapter::document_projection_from_snapshot_v1(&snapshot)?;
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

    /// Return durable drawing facts through the typed document boundary.
    ///
    /// Projection and rendering callers receive the report owned by this exact
    /// snapshot rather than inspecting CDML or deriving directional marks.
    pub fn molecule_stereo_depictions_v1(
        &self,
        molecule_id: &DocumentObjectIdV1,
    ) -> Result<Option<DocumentStereoDepictionReportV1>, TypedDocumentError> {
        TypedDocument::parse(self.snapshot.cdml())?.molecule_stereo_depictions_v1(molecule_id)
    }
}

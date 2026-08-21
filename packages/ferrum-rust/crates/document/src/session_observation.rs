//! Immutable, revision-bound document observations for frontend projections.

use ferrum_chemistry::MolGraph;
use ferrum_core::RecordId;

use super::{
    DocumentProjectionV1, DocumentSnapshot, ProjectionError, TypedDocument,
    document_molecule_graph_v1,
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
    direct_molecule_graphs: Vec<ObservedDirectMoleculeGraphV1>,
}

/// Direct-molecule chemistry facts made while this observation was admitted.
/// They are graph-position aligned and contain no renderer association.
#[derive(Clone, Debug, PartialEq)]
pub struct ObservedDirectMoleculeGraphV1 {
    // This is the authored identity domain used by renderer direct roots. It
    // must never be replaced with the durable document object identity used
    // only to lower this observation.
    source_id: Option<String>,
    source_order: u32,
    graph: Option<MolGraph>,
    graph_position_to_record_id: Vec<RecordId>,
}

impl ObservedDirectMoleculeGraphV1 {
    #[must_use]
    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    #[must_use]
    pub fn graph(&self) -> Option<&MolGraph> {
        self.graph.as_ref()
    }
    #[must_use]
    pub fn graph_position_to_record_id(&self) -> &[RecordId] {
        &self.graph_position_to_record_id
    }
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
        let mut direct_molecule_graphs = Vec::new();
        for root in projection.molecules() {
            let Some(id) = root.id() else { continue };
            let source_id = root
                .source_id()
                .filter(|source_id| !source_id.is_empty())
                .map(ToOwned::to_owned);
            let lowered = document
                .core_molecule(id)
                .ok()
                .flatten()
                .and_then(|molecule| document_molecule_graph_v1(&molecule).ok())
                .map(|value| value.into_parts_with_atom_records());
            let (graph, graph_position_to_record_id) = match lowered {
                Some((graph, _edges, records)) if records.len() == graph.atoms().len() => {
                    (Some(graph), records)
                }
                _ => (None, Vec::new()),
            };
            direct_molecule_graphs.push(ObservedDirectMoleculeGraphV1 {
                source_id,
                source_order: root.source_order(),
                graph,
                graph_position_to_record_id,
            });
        }
        Ok(Self {
            snapshot,
            projection,
            direct_molecule_graphs,
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

    /// Exact direct-molecule graphs derived from this accepted retained state.
    #[must_use]
    pub fn direct_molecule_graphs_v1(&self) -> &[ObservedDirectMoleculeGraphV1] {
        &self.direct_molecule_graphs
    }
}

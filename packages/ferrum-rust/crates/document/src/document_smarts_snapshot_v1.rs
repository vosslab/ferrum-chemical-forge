//! SMARTS-specific graph preparation for one accepted session revision.

use ferrum_chemistry::MolGraph;
use thiserror::Error;

use crate::document_direct_root_index_v1::document_direct_root_paint_orders_v1;
use crate::{
    DocumentObjectIdV1, DocumentSnapshot, PersistentId, TypedDocument, document_molecule_graph_v1,
};

const MAX_SMARTS_TARGETS_V1: usize = 256;

/// A graph-ready direct root retained only for the SMARTS operation boundary.
pub struct DocumentSmartsTargetV1 {
    durable_selector: DocumentObjectIdV1,
    document_paint_order: u32,
    graph: MolGraph,
    graph_position_to_document_object_ids: Vec<DocumentObjectIdV1>,
}

impl DocumentSmartsTargetV1 {
    #[must_use]
    pub const fn durable_selector(&self) -> &DocumentObjectIdV1 {
        &self.durable_selector
    }
    #[must_use]
    pub const fn document_paint_order(&self) -> u32 {
        self.document_paint_order
    }
    #[must_use]
    pub const fn graph(&self) -> &MolGraph {
        &self.graph
    }
    #[must_use]
    pub fn graph_position_to_document_object_ids(&self) -> &[DocumentObjectIdV1] {
        &self.graph_position_to_document_object_ids
    }
}

/// One opaque, revision-bound SMARTS graph preparation.
pub struct PreparedDocumentSmartsSnapshotV1 {
    revision: u64,
    digest: [u8; 32],
    targets: Vec<DocumentSmartsTargetV1>,
}

impl PreparedDocumentSmartsSnapshotV1 {
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    #[must_use]
    pub fn targets(&self) -> &[DocumentSmartsTargetV1] {
        &self.targets
    }
}

/// Typed SMARTS preparation refusal without a generic chemistry cache fallback.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DocumentSmartsSnapshotErrorV1 {
    #[error("SMARTS snapshot revision is stale: expected {expected}, current {actual}")]
    StaleRevision { expected: u64, actual: u64 },
    #[error("SMARTS target limit exceeded")]
    TargetLimitExceeded,
    #[error("a direct root cannot be prepared for SMARTS")]
    UnsupportedDocument,
}

pub(crate) fn prepare_smarts_snapshot_v1(
    document: &TypedDocument,
    snapshot: &DocumentSnapshot,
) -> Result<PreparedDocumentSmartsSnapshotV1, DocumentSmartsSnapshotErrorV1> {
    let projection = crate::projection_adapter::document_projection_from_snapshot_v1(snapshot)
        .map_err(|_| DocumentSmartsSnapshotErrorV1::UnsupportedDocument)?;
    let document_paint_orders = document_direct_root_paint_orders_v1(&projection)
        .map_err(|_| DocumentSmartsSnapshotErrorV1::UnsupportedDocument)?;
    let mut targets = Vec::new();
    for root in projection.molecules() {
        if targets.len() == MAX_SMARTS_TARGETS_V1 {
            return Err(DocumentSmartsSnapshotErrorV1::TargetLimitExceeded);
        }
        let selector = root
            .id()
            .cloned()
            .ok_or(DocumentSmartsSnapshotErrorV1::UnsupportedDocument)?;
        let molecule = document
            .core_molecule(&selector)
            .map_err(|_| DocumentSmartsSnapshotErrorV1::UnsupportedDocument)?
            .ok_or(DocumentSmartsSnapshotErrorV1::UnsupportedDocument)?;
        let (graph, _, graph_position_to_record_id) = document_molecule_graph_v1(&molecule)
            .map_err(|_| DocumentSmartsSnapshotErrorV1::UnsupportedDocument)?
            .into_parts_with_atom_records();
        if graph_position_to_record_id.len() != graph.atoms().len() {
            return Err(DocumentSmartsSnapshotErrorV1::UnsupportedDocument);
        }
        // Chemistry constructs graph positions from source-local atom records.
        // Resolve those positions immediately through the document identity index
        // so every caller beyond this boundary owns durable document identities.
        let graph_position_to_document_object_ids = graph_position_to_record_id
            .iter()
            .map(|record_id| {
                let source_id = PersistentId::new(record_id.source_id().as_str().to_owned())
                    .map_err(|_| DocumentSmartsSnapshotErrorV1::UnsupportedDocument)?;
                document
                    .document_object_id_for_source_id_v1(&source_id)
                    .ok_or(DocumentSmartsSnapshotErrorV1::UnsupportedDocument)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let document_paint_order = *document_paint_orders
            .get(&selector)
            .ok_or(DocumentSmartsSnapshotErrorV1::UnsupportedDocument)?;
        targets.push(DocumentSmartsTargetV1 {
            durable_selector: selector,
            document_paint_order,
            graph,
            graph_position_to_document_object_ids,
        });
    }
    Ok(PreparedDocumentSmartsSnapshotV1 {
        revision: snapshot.revision(),
        digest: *snapshot.digest(),
        targets,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DocumentSession;

    #[test]
    fn snapshot_maps_graph_positions_to_durable_atom_ids() {
        let session = DocumentSession::load(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
        )
        .expect("document loads");
        let snapshot = session
            .prepare_smarts_snapshot_v1(0)
            .expect("SMARTS snapshot prepares");
        let document = session.current_document_v1();
        let expected = ["a"].map(|source_id| {
            document
                .document_object_id_for_source_id_v1(
                    &PersistentId::new(source_id).expect("source ID"),
                )
                .expect("durable atom identity")
        });

        assert_eq!(
            snapshot.targets()[0].graph_position_to_document_object_ids(),
            expected
        );
    }
}

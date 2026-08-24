//! SMARTS-specific graph preparation for one accepted session revision.

use ferrum_chemistry::MolGraph;
use ferrum_core::RecordId;
use thiserror::Error;

use crate::{DocumentObjectIdV1, DocumentSnapshot, TypedDocument, document_molecule_graph_v1};

const MAX_SMARTS_TARGETS_V1: usize = 256;

/// A graph-ready direct root retained only for the SMARTS operation boundary.
pub struct DocumentSmartsTargetV1 {
    durable_selector: DocumentObjectIdV1,
    renderer_source_id: Option<String>,
    source_order: u32,
    graph: MolGraph,
    graph_position_to_record_id: Vec<RecordId>,
}

impl DocumentSmartsTargetV1 {
    #[must_use]
    pub const fn durable_selector(&self) -> &DocumentObjectIdV1 {
        &self.durable_selector
    }
    #[must_use]
    pub fn renderer_source_id(&self) -> Option<&str> {
        self.renderer_source_id.as_deref()
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    #[must_use]
    pub const fn graph(&self) -> &MolGraph {
        &self.graph
    }
    #[must_use]
    pub fn graph_position_to_record_id(&self) -> &[RecordId] {
        &self.graph_position_to_record_id
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
    let mut targets = Vec::new();
    for root in crate::projection_adapter::document_projection_from_snapshot_v1(snapshot)
        .map_err(|_| DocumentSmartsSnapshotErrorV1::UnsupportedDocument)?
        .molecules()
    {
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
        targets.push(DocumentSmartsTargetV1 {
            durable_selector: selector,
            renderer_source_id: root.source_id().map(str::to_owned),
            source_order: root.source_order(),
            graph,
            graph_position_to_record_id,
        });
    }
    Ok(PreparedDocumentSmartsSnapshotV1 {
        revision: snapshot.revision(),
        digest: *snapshot.digest(),
        targets,
    })
}

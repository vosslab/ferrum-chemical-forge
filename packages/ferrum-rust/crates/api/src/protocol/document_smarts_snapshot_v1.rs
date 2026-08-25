//! Private API-owned SMARTS target storage for one document preparation.
//!
//! This keeps one graph-position-to-durable-object mapping while the API owns
//! the query. Neither this snapshot nor its targets are serializable, cloneable,
//! debuggable, or re-exported.

use ferrum_chemistry::MolGraph;
use ferrum_document::PreparedDocumentSmartsSnapshotV1;
use ferrum_render::RenderTarget;

pub(crate) struct OwnedDocumentSmartsSnapshotV1 {
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    revision: u64,
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    digest: [u8; 32],
    targets: Vec<OwnedSmartsTargetV1>,
}

pub(crate) struct OwnedSmartsTargetV1 {
    target: RenderTarget,
    document_paint_order: u32,
    graph: MolGraph,
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    graph_position_to_document_object_ids: Vec<ferrum_document::DocumentObjectIdV1>,
}

impl OwnedDocumentSmartsSnapshotV1 {
    pub(crate) fn from_prepared_snapshot_v1(prepared: PreparedDocumentSmartsSnapshotV1) -> Self {
        let revision = prepared.revision();
        let digest = *prepared.digest();
        let targets = prepared
            .targets()
            .iter()
            .map(|target| OwnedSmartsTargetV1 {
                target: RenderTarget::document_object(target.durable_selector().clone()),
                document_paint_order: target.document_paint_order(),
                graph: target.graph().clone(),
                graph_position_to_document_object_ids: target
                    .graph_position_to_document_object_ids()
                    .to_vec(),
            })
            .collect();
        Self {
            revision,
            digest,
            targets,
        }
    }
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    pub(crate) const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    pub(crate) fn targets(&self) -> &[OwnedSmartsTargetV1] {
        &self.targets
    }
    pub(crate) fn selected_target_by_render_target(
        &self,
        render_target: &RenderTarget,
    ) -> Option<&OwnedSmartsTargetV1> {
        let mut matches = self
            .targets
            .iter()
            .filter(|target| target.target == *render_target);
        let target = matches.next()?;
        matches.next().is_none().then_some(target)
    }
}

impl OwnedSmartsTargetV1 {
    pub(crate) const fn document_paint_order(&self) -> u32 {
        self.document_paint_order
    }
    pub(crate) fn graph(&self) -> &MolGraph {
        &self.graph
    }
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    pub(crate) fn graph_position_to_document_object_ids(
        &self,
    ) -> &[ferrum_document::DocumentObjectIdV1] {
        &self.graph_position_to_document_object_ids
    }
}

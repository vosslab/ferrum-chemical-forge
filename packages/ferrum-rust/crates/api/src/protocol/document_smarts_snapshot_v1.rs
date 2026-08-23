//! Private API-owned SMARTS target storage for one document preparation.
//!
//! This retains the separate durable-selector and renderer-source identity domains
//! only while the API owns the query. Neither this snapshot nor its targets are
//! serializable, cloneable, debuggable, or re-exported.

use ferrum_chemistry::MolGraph;
use ferrum_core::RecordId;
use ferrum_document::PreparedDocumentSmartsSnapshotV1;

pub(crate) struct OwnedDocumentSmartsSnapshotV1 {
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    revision: u64,
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    digest: [u8; 32],
    targets: Vec<OwnedSmartsTargetV1>,
}

pub(crate) struct OwnedSmartsTargetV1 {
    // The stateless protocol accepts only the projection's durable direct-root
    // selector. The live renderer capability joins only its authored source
    // identity. Both remain private and deliberately have separate lookup APIs.
    durable_selector: String,
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    renderer_source_id: Option<String>,
    source_order: u32,
    graph: MolGraph,
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    graph_position_to_record_id: Vec<RecordId>,
}

impl OwnedDocumentSmartsSnapshotV1 {
    pub(crate) fn from_prepared_snapshot_v1(prepared: PreparedDocumentSmartsSnapshotV1) -> Self {
        let revision = prepared.revision();
        let digest = *prepared.digest();
        let targets = prepared
            .targets()
            .iter()
            .map(|target| OwnedSmartsTargetV1 {
                durable_selector: target.durable_selector().as_str().to_owned(),
                renderer_source_id: target.renderer_source_id().map(str::to_owned),
                source_order: target.source_order(),
                graph: target.graph().clone(),
                graph_position_to_record_id: target.graph_position_to_record_id().to_vec(),
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
    pub(crate) fn selected_target_by_durable_selector(
        &self,
        selector: &str,
    ) -> Option<&OwnedSmartsTargetV1> {
        let mut matches = self
            .targets
            .iter()
            .filter(|target| target.durable_selector == selector);
        let target = matches.next()?;
        matches.next().is_none().then_some(target)
    }
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    pub(crate) fn selected_target_by_renderer_source_id(
        &self,
        source_id: &str,
    ) -> Option<&OwnedSmartsTargetV1> {
        let mut matches = self
            .targets
            .iter()
            .filter(|target| target.renderer_source_id.as_deref() == Some(source_id));
        let target = matches.next()?;
        matches.next().is_none().then_some(target)
    }
}

impl OwnedSmartsTargetV1 {
    pub(crate) const fn source_order(&self) -> u32 {
        self.source_order
    }
    pub(crate) fn graph(&self) -> &MolGraph {
        &self.graph
    }
    #[cfg_attr(not(feature = "python-binding"), allow(dead_code))]
    pub(crate) fn graph_position_to_record_id(&self) -> &[RecordId] {
        &self.graph_position_to_record_id
    }
}

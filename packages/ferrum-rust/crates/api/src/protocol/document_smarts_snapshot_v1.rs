//! Private, one-pass SMARTS target lowering for an accepted document observation.
//!
//! This retains the separate durable-selector and renderer-source identity domains
//! only while the API owns the query. Neither this snapshot nor its targets are
//! serializable, cloneable, debuggable, or re-exported.

use ferrum_chemistry::MolGraph;
use ferrum_core::RecordId;
use ferrum_document::SessionDocumentObservationV1;

use super::execution::ExecutionFailureV1;

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
    pub(crate) fn from_accepted_observation_v1(
        observation: &SessionDocumentObservationV1,
    ) -> Result<Self, ExecutionFailureV1> {
        let mut targets = Vec::new();
        for root in observation.direct_molecule_graphs_v1() {
            if targets.len() == 256 {
                return Err(ExecutionFailureV1::document_invalid(
                    "target_limit_exceeded".to_owned(),
                ));
            }
            let graph = root
                .graph()
                .ok_or_else(|| {
                    ExecutionFailureV1::document_invalid("target_not_matchable".to_owned())
                })?
                .clone();
            let positions = root.graph_position_to_record_id().to_vec();
            if positions.len() != graph.atoms().len() {
                return Err(unsupported_document());
            }
            let durable_selector = durable_direct_root_id(observation, root)?;
            let renderer_source_id = root
                .source_id()
                .filter(|source_id| !source_id.is_empty())
                .map(str::to_owned);
            targets.push(OwnedSmartsTargetV1 {
                durable_selector,
                renderer_source_id,
                source_order: root.source_order(),
                graph,
                graph_position_to_record_id: positions,
            });
        }
        let document = observation.snapshot();
        Ok(Self {
            revision: document.revision(),
            digest: *document.digest(),
            targets,
        })
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

fn durable_direct_root_id(
    observation: &SessionDocumentObservationV1,
    target: &ferrum_document::ObservedDirectMoleculeGraphV1,
) -> Result<String, ExecutionFailureV1> {
    let mut roots = observation.projection().molecules().iter().filter(|root| {
        root.source_order() == target.source_order() && root.source_id() == target.source_id()
    });
    let root = roots.next().ok_or_else(unsupported_document)?;
    if roots.next().is_some() {
        return Err(unsupported_document());
    }
    root.id()
        .map(|id| id.as_str().to_owned())
        .ok_or_else(unsupported_document)
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

fn unsupported_document() -> ExecutionFailureV1 {
    ExecutionFailureV1::document_invalid("unsupported_document".to_owned())
}

#[cfg(test)]
mod tests {
    use ferrum_document::DocumentSession;

    use super::OwnedDocumentSmartsSnapshotV1;

    #[test]
    fn selected_target_lookup_keeps_durable_and_renderer_identity_domains_separate() {
        let observation = DocumentSession::load(concat!(
            "<cdml><molecule id=\"authored-root\"><atom id=\"a\" name=\"C\">",
            "<point x=\"0\" y=\"0\"/></atom></molecule></cdml>"
        ))
        .expect("document")
        .observe(0)
        .expect("observation");
        let snapshot = OwnedDocumentSmartsSnapshotV1::from_accepted_observation_v1(&observation)
            .expect("snapshot");
        let target = snapshot.targets().first().expect("one target");

        assert_ne!(target.durable_selector, "authored-root");
        assert_eq!(target.renderer_source_id.as_deref(), Some("authored-root"));
        assert!(
            snapshot
                .selected_target_by_durable_selector(&target.durable_selector)
                .is_some()
        );
        assert!(
            snapshot
                .selected_target_by_renderer_source_id("authored-root")
                .is_some()
        );
        assert!(
            snapshot
                .selected_target_by_durable_selector("authored-root")
                .is_none()
        );
        assert!(
            snapshot
                .selected_target_by_renderer_source_id(&target.durable_selector)
                .is_none()
        );
    }
}

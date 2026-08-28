//! Private API-owned graph facts from one accepted document observation.
//!
//! The observation is the sole source for both stateless protocol execution
//! and the optional live binding. This module deliberately owns no receipt,
//! renderer plan, query text, or reveal capability.

use std::collections::HashSet;

use ferrum_chemistry::MolGraph;
use ferrum_document::{
    DocumentObjectIdV1, SessionDocumentObservationV1, document_direct_root_paint_orders_v1,
};
use ferrum_graph_lowering::lower_direct_molecule_graph;
use ferrum_render::RenderTarget;

const MAX_DIRECT_MOLECULE_TARGETS: usize = 256;

/// One API-private refusal while lowering an accepted immutable observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SnapshotConstructionError {
    TargetLimitExceeded,
    UnsupportedDocument,
}

/// The only owned SMARTS target storage.
///
/// This type intentionally has no public, clone, debug, serialization, or
/// dereference surface. It is constructed only from one accepted observation.
pub(crate) struct OwnedDocumentSmartsSnapshot {
    #[cfg(any(feature = "python-binding", test))]
    revision: u64,
    digest: [u8; 32],
    targets: Vec<OwnedSmartsTarget>,
}

pub(crate) struct OwnedSmartsTarget {
    target: RenderTarget,
    document_paint_order: u32,
    graph: MolGraph,
    #[cfg(any(feature = "python-binding", test))]
    graph_position_to_document_object_ids: Vec<DocumentObjectIdV1>,
}

impl OwnedDocumentSmartsSnapshot {
    /// Consume one session-authenticated observation without reloading CDML or
    /// consulting a second document session.
    pub(crate) fn from_accepted_observation(
        observation: SessionDocumentObservationV1,
    ) -> Result<Self, SnapshotConstructionError> {
        let snapshot = observation.snapshot();
        let paint_orders = document_direct_root_paint_orders_v1(observation.projection())
            .map_err(|_| SnapshotConstructionError::UnsupportedDocument)?;
        let molecules = observation.projection().molecules();
        if molecules.len() > MAX_DIRECT_MOLECULE_TARGETS {
            return Err(SnapshotConstructionError::TargetLimitExceeded);
        }
        let mut targets = Vec::new();
        targets
            .try_reserve_exact(molecules.len())
            .map_err(|_| SnapshotConstructionError::UnsupportedDocument)?;
        for molecule in molecules {
            let document_paint_order = *paint_orders
                .get(molecule.document_object_id())
                .ok_or(SnapshotConstructionError::UnsupportedDocument)?;
            let (graph, _graph_position_to_document_object_ids) =
                lower_graph_with_input_ordered_document_ids(molecule)?;
            targets.push(OwnedSmartsTarget {
                target: RenderTarget::document_object(molecule.document_object_id().clone()),
                document_paint_order,
                graph,
                #[cfg(any(feature = "python-binding", test))]
                graph_position_to_document_object_ids: _graph_position_to_document_object_ids,
            });
        }
        Ok(Self {
            #[cfg(any(feature = "python-binding", test))]
            revision: snapshot.revision(),
            digest: *snapshot.digest(),
            targets,
        })
    }

    #[cfg(any(feature = "python-binding", test))]
    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub(crate) fn targets(&self) -> &[OwnedSmartsTarget] {
        &self.targets
    }

    pub(crate) fn selected_target_by_render_target(
        &self,
        render_target: &RenderTarget,
    ) -> Option<&OwnedSmartsTarget> {
        let mut matches = self
            .targets
            .iter()
            .filter(|target| target.target == *render_target);
        let target = matches.next()?;
        matches.next().is_none().then_some(target)
    }
}

impl OwnedSmartsTarget {
    pub(crate) const fn document_paint_order(&self) -> u32 {
        self.document_paint_order
    }

    pub(crate) fn graph(&self) -> &MolGraph {
        &self.graph
    }

    #[cfg(any(feature = "python-binding", test))]
    pub(crate) fn graph_position_to_document_object_ids(&self) -> &[DocumentObjectIdV1] {
        &self.graph_position_to_document_object_ids
    }
}

/// Lower one projection and retain durable atom IDs in the exact atom-input order.
///
/// Graph positions are defined by `graph_input_atoms`; the durable vector is
/// emitted from that same named sequence, never by associating two produced
/// collections or zipping independently obtained projections.
fn lower_graph_with_input_ordered_document_ids(
    molecule: &ferrum_document::MoleculeProjectionV1,
) -> Result<(MolGraph, Vec<DocumentObjectIdV1>), SnapshotConstructionError> {
    if molecule.atoms().is_empty() {
        return Err(SnapshotConstructionError::UnsupportedDocument);
    }
    let (facts, graph_position_to_document_object_ids) = molecule
        .direct_molecule_graph_facts_with_atom_metadata(false, |atom| {
            atom.document_object_id().clone()
        })
        .map_err(|_| SnapshotConstructionError::UnsupportedDocument)?;
    validate_document_atom_correspondence(
        &facts,
        &graph_position_to_document_object_ids,
        &graph_position_to_document_object_ids,
    )?;
    let (graph, _) = lower_direct_molecule_graph(&facts)
        .map_err(|_| SnapshotConstructionError::UnsupportedDocument)?
        .into_parts();
    if graph_position_to_document_object_ids.len() != graph.atoms().len()
        || graph_position_to_document_object_ids.len()
            != unique_document_atom_count(&graph_position_to_document_object_ids)?
    {
        return Err(SnapshotConstructionError::UnsupportedDocument);
    }
    Ok((graph, graph_position_to_document_object_ids))
}

/// Refuse any graph-position identity correspondence that cannot be total and unique.
///
/// This boundary runs before native SMARTS execution. Production IDs are emitted
/// by the projection's atom-fact loop; tests can exercise malformed mappings
/// here without granting a public graph/identity carrier.
fn validate_document_atom_correspondence(
    facts: &ferrum_document_projection::DirectMoleculeGraphFacts,
    expected_document_atom_ids: &[DocumentObjectIdV1],
    document_atom_ids: &[DocumentObjectIdV1],
) -> Result<(), SnapshotConstructionError> {
    if facts.atoms().len() != expected_document_atom_ids.len()
        || facts.atoms().len() != document_atom_ids.len()
    {
        return Err(SnapshotConstructionError::UnsupportedDocument);
    }
    for (position, object_id) in document_atom_ids.iter().enumerate() {
        if expected_document_atom_ids.get(position) != Some(object_id) {
            return Err(SnapshotConstructionError::UnsupportedDocument);
        }
    }
    unique_document_atom_count(document_atom_ids).map(|_| ())
}

fn unique_document_atom_count(
    document_atom_ids: &[DocumentObjectIdV1],
) -> Result<usize, SnapshotConstructionError> {
    let mut ids = HashSet::new();
    ids.try_reserve(document_atom_ids.len())
        .map_err(|_| SnapshotConstructionError::UnsupportedDocument)?;
    for object_id in document_atom_ids {
        if !ids.insert(object_id) {
            return Err(SnapshotConstructionError::UnsupportedDocument);
        }
    }
    Ok(ids.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::DocumentSession;
    use ferrum_document_projection::{
        DirectMoleculeGraphAtomFact, DirectMoleculeGraphAtomInput, DirectMoleculeGraphFacts,
        Point3V1,
    };

    const SOURCE: &str = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\">",
        "<molecule id=\"first\">",
        "<atom id=\"first-carbon\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"first-oxygen\" name=\"O\"><point x=\"20\" y=\"0\"/></atom>",
        "<bond id=\"first-bond\" start=\"first-carbon\" end=\"first-oxygen\" type=\"n1\"/>",
        "</molecule><plus id=\"plus\"><point x=\"30\" y=\"0\"/></plus>",
        "<molecule id=\"second\">",
        "<atom id=\"second-nitrogen\" name=\"N\"><point x=\"40\" y=\"0\"/></atom>",
        "</molecule></cdml>"
    );

    #[test]
    fn accepted_observations_lower_identical_source_ordered_graph_identity_facts() {
        let session = DocumentSession::load(SOURCE).expect("document loads");
        let first = OwnedDocumentSmartsSnapshot::from_accepted_observation(
            session.observe(0).expect("first observation"),
        )
        .expect("first snapshot lowers");
        let second = OwnedDocumentSmartsSnapshot::from_accepted_observation(
            session.observe(0).expect("second observation"),
        )
        .expect("second snapshot lowers");

        assert_eq!(first.revision(), second.revision());
        assert_eq!(first.digest(), second.digest());
        assert_eq!(first.targets().len(), 2);
        for (left, right) in first.targets().iter().zip(second.targets()) {
            assert_eq!(left.document_paint_order(), right.document_paint_order());
            assert_eq!(left.graph(), right.graph());
            assert_eq!(
                left.graph_position_to_document_object_ids(),
                right.graph_position_to_document_object_ids()
            );
        }
        assert_eq!(first.targets()[0].graph().atoms().len(), 2);
        assert_eq!(first.targets()[1].graph().atoms().len(), 1);
        assert_eq!(
            first.targets()[0].graph_position_to_document_object_ids(),
            &[
                session
                    .observe(0)
                    .expect("identity observation")
                    .projection()
                    .molecules()[0]
                    .atoms()[0]
                    .document_object_id()
                    .clone(),
                session
                    .observe(0)
                    .expect("identity observation")
                    .projection()
                    .molecules()[0]
                    .atoms()[1]
                    .document_object_id()
                    .clone(),
            ]
        );
    }

    fn atom_facts(atom_count: usize) -> DirectMoleculeGraphFacts {
        DirectMoleculeGraphFacts::new(
            (0..atom_count)
                .map(|_| {
                    DirectMoleculeGraphAtomFact::new(DirectMoleculeGraphAtomInput {
                        element: Some("C".to_owned()),
                        position: Point3V1::new(0.0, 0.0, 0.0).expect("finite point"),
                        formal_charge: None,
                        isotope: None,
                        explicit_hydrogens: None,
                        valence: None,
                        multiplicity: None,
                        free_sites: None,
                    })
                })
                .collect(),
            Vec::new(),
            Vec::new(),
            false,
        )
    }

    fn id(byte: u8) -> DocumentObjectIdV1 {
        DocumentObjectIdV1::from_entropy_bytes([byte; 16])
    }

    #[test]
    fn correspondence_refuses_missing_duplicate_foreign_reordered_and_surplus_ids_before_lowering()
    {
        let facts = atom_facts(2);
        let expected = vec![id(1), id(2)];
        for supplied in [
            vec![id(1)],
            vec![id(1), id(1)],
            vec![id(1), id(3)],
            vec![id(2), id(1)],
            vec![id(1), id(2), id(3)],
        ] {
            assert_eq!(
                validate_document_atom_correspondence(&facts, &expected, &supplied),
                Err(SnapshotConstructionError::UnsupportedDocument),
            );
        }
        assert_eq!(
            validate_document_atom_correspondence(&atom_facts(1), &expected, &expected),
            Err(SnapshotConstructionError::UnsupportedDocument),
        );
    }
}

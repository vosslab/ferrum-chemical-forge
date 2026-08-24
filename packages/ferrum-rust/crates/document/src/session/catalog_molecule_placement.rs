//! Renderer-admitted molecule placement transactions for closed catalog recipes.

use thiserror::Error;

use super::{
    DocumentFenceV1, DocumentSession, MoleculeInsertionV1, PendingAdmittedMoleculeInsertionV1,
    PendingStandaloneHaworthV1, PersistentId, Point3V1, SessionOperationResultV1,
};
use crate::{AuthoringCapabilityAccessErrorV1, AuthoringCapabilityV1};
use ferrum_domain::haworth::StandaloneDGlucoseHaworthRecipeV1;

/// Session-issued capability for one closed renderer-admitted molecule placement.
#[derive(Clone, Debug)]
pub struct CatalogMoleculePlacementGestureV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
}

impl CatalogMoleculePlacementGestureV1 {
    #[must_use]
    pub fn same_gesture_v1(&self, other: &Self) -> bool {
        self.fence == other.fence && self.capability.same_capability(&other.capability)
    }
}

/// Closed molecule insertion alternatives accepted by the catalog placement transaction.
#[derive(Clone, Debug)]
pub enum CatalogMoleculePlacementRequestV1 {
    Molecule(MoleculeInsertionV1),
    StandaloneHaworth {
        recipe: StandaloneDGlucoseHaworthRecipeV1,
        anchor: Point3V1,
    },
}

/// Stable reasons a catalog molecule placement cannot be prepared or committed.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CatalogMoleculePlacementRefusalV1 {
    #[error("catalog placement source is stale")]
    StaleSnapshot,
    #[error("catalog placement handle belongs to another document session")]
    ForeignSession,
    #[error("catalog placement capability was already used")]
    ReplayedGesture,
    #[error("catalog placement candidate could not be rendered completely")]
    RendererAdmission,
    #[error("catalog placement commit was rejected by document session")]
    SessionConflict,
}

/// Opaque, one-use renderer-admitted catalog molecule placement.
#[derive(Debug)]
pub struct PendingCatalogMoleculePlacementV1 {
    capability: AuthoringCapabilityV1,
    fence: DocumentFenceV1,
    pending: CatalogMoleculePendingV1,
}

#[derive(Debug)]
enum CatalogMoleculePendingV1 {
    Molecule(PendingAdmittedMoleculeInsertionV1),
    StandaloneHaworth(PendingStandaloneHaworthV1),
}

impl CatalogMoleculePendingV1 {
    fn identifier(&self) -> &PersistentId {
        match self {
            Self::Molecule(pending) => pending.molecule_identifier(),
            Self::StandaloneHaworth(pending) => pending.molecule_identifier(),
        }
    }
}

impl PendingCatalogMoleculePlacementV1 {
    #[must_use]
    pub fn identifier(&self) -> &str {
        self.pending.identifier().as_str()
    }

    /// Return the immutable renderer-issued plan for the exact pending candidate.
    #[must_use]
    pub fn render_plan_v1(&self) -> &ferrum_render::DocumentRenderPlanV1 {
        match &self.pending {
            CatalogMoleculePendingV1::Molecule(pending) => pending.document_render_plan_v1(),
            CatalogMoleculePendingV1::StandaloneHaworth(pending) => pending.render_plan_v1(),
        }
    }
}

impl DocumentSession {
    /// Begin one catalog molecule placement at an exact document fence.
    pub fn begin_catalog_molecule_placement_v1(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<CatalogMoleculePlacementGestureV1, CatalogMoleculePlacementRefusalV1> {
        require_catalog_fence(self, fence)?;
        Ok(CatalogMoleculePlacementGestureV1 {
            capability: self.authoring_capability_issuer_v1().issue(),
            fence,
        })
    }

    /// Validate a catalog gesture before a non-mutating recipe preview.
    pub fn validate_catalog_molecule_placement_v1(
        &self,
        gesture: &CatalogMoleculePlacementGestureV1,
    ) -> Result<(), CatalogMoleculePlacementRefusalV1> {
        if !gesture
            .capability
            .belongs_to(&self.authoring_capability_issuer_v1())
        {
            return Err(CatalogMoleculePlacementRefusalV1::ForeignSession);
        }
        require_catalog_fence(self, gesture.fence)
    }

    /// Build and renderer-admit one exact catalog insertion without mutating history.
    pub fn prepare_catalog_molecule_placement_v1(
        &mut self,
        gesture: &CatalogMoleculePlacementGestureV1,
        request: CatalogMoleculePlacementRequestV1,
    ) -> Result<PendingCatalogMoleculePlacementV1, CatalogMoleculePlacementRefusalV1> {
        self.validate_catalog_molecule_placement_v1(gesture)?;
        let pending = match request {
            CatalogMoleculePlacementRequestV1::Molecule(molecule) => {
                CatalogMoleculePendingV1::Molecule(
                    self.prepare_admitted_molecule_insertion_v1(
                        gesture.fence.revision(),
                        &molecule,
                    )
                    .map_err(|_| CatalogMoleculePlacementRefusalV1::SessionConflict)?,
                )
            }
            CatalogMoleculePlacementRequestV1::StandaloneHaworth { recipe, anchor } => {
                CatalogMoleculePendingV1::StandaloneHaworth(
                    self.prepare_create_standalone_haworth_v1(
                        gesture.fence.revision(),
                        recipe,
                        anchor,
                    )
                    .map_err(|_| CatalogMoleculePlacementRefusalV1::SessionConflict)?,
                )
            }
        };
        Ok(PendingCatalogMoleculePlacementV1 {
            capability: gesture.capability.clone(),
            fence: gesture.fence,
            pending,
        })
    }

    /// Verify and atomically append one exact renderer-admitted catalog insertion.
    pub fn commit_catalog_molecule_placement_v1(
        &mut self,
        pending: &mut PendingCatalogMoleculePlacementV1,
    ) -> Result<SessionOperationResultV1, CatalogMoleculePlacementRefusalV1> {
        if !pending
            .capability
            .belongs_to(&self.authoring_capability_issuer_v1())
        {
            return Err(CatalogMoleculePlacementRefusalV1::ForeignSession);
        }
        let claim = pending
            .capability
            .claim_for_commit(&self.authoring_capability_issuer_v1())
            .map_err(|error| match error {
                AuthoringCapabilityAccessErrorV1::ForeignSession => {
                    CatalogMoleculePlacementRefusalV1::ForeignSession
                }
                AuthoringCapabilityAccessErrorV1::Replayed => {
                    CatalogMoleculePlacementRefusalV1::ReplayedGesture
                }
            })?;
        require_catalog_fence(self, pending.fence)?;
        let operation = match &mut pending.pending {
            CatalogMoleculePendingV1::Molecule(value) => self
                .commit_admitted_molecule_insertion_v1(pending.fence.revision(), value)
                .map_err(|_| CatalogMoleculePlacementRefusalV1::SessionConflict)?,
            CatalogMoleculePendingV1::StandaloneHaworth(value) => self
                .commit_create_standalone_haworth_v1(pending.fence.revision(), value)
                .map_err(|_| CatalogMoleculePlacementRefusalV1::SessionConflict)?,
        };
        claim.consume();
        Ok(operation)
    }
}

fn require_catalog_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), CatalogMoleculePlacementRefusalV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| CatalogMoleculePlacementRefusalV1::SessionConflict)?;
    if snapshot.revision() != fence.revision() || *snapshot.digest() != fence.digest() {
        return Err(CatalogMoleculePlacementRefusalV1::StaleSnapshot);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MoleculeInsertionAtomV1, MoleculeInsertionV1, PresentationGesturePoint2V1};

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    fn molecule_request() -> CatalogMoleculePlacementRequestV1 {
        let point = PresentationGesturePoint2V1::new(20.0, 10.0).expect("point");
        let atom = MoleculeInsertionAtomV1::new(
            "C",
            Point3V1::new(point.x(), point.y(), 0.0).expect("three-dimensional point"),
            None,
            None,
            None,
        )
        .expect("atom");
        let molecule = MoleculeInsertionV1::new(vec![atom], Vec::new()).expect("molecule");
        CatalogMoleculePlacementRequestV1::Molecule(molecule)
    }

    #[test]
    fn stale_catalog_pending_keeps_its_exact_refusal_and_leaves_history_unchanged() {
        let source = "<cdml xmlns=\"urn:ferrum:cdml\"/>";
        let mut session = DocumentSession::load(source).expect("session");
        let start = fence(&session);
        let gesture = session
            .begin_catalog_molecule_placement_v1(start)
            .expect("gesture");
        let mut pending = session
            .prepare_catalog_molecule_placement_v1(&gesture, molecule_request())
            .expect("renderer admission");
        let mut replacement = session
            .prepare_complete_cdml_mutation_v1(start, source)
            .expect("separate document transition prepares");
        session
            .commit_complete_cdml_mutation_v1(&mut replacement)
            .expect("separate document transition commits");
        assert!(matches!(
            session.commit_catalog_molecule_placement_v1(&mut pending),
            Err(CatalogMoleculePlacementRefusalV1::StaleSnapshot)
        ));
        assert!(matches!(
            session.commit_catalog_molecule_placement_v1(&mut pending),
            Err(CatalogMoleculePlacementRefusalV1::StaleSnapshot)
        ));
        assert_eq!(session.snapshot().expect("snapshot").revision(), 1);
    }
}

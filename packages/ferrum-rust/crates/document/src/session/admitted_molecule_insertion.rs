//! Renderer-admitted molecule and interchange transactions owned by a document session.

use crate::InterchangeRecordBatchInsertionV1;
use ferrum_render::{DocumentRenderContentV1, MoleculeRenderPlan};

use super::{
    DetachedRegularRingInsertionV1, DocumentSession, DocumentSessionError, MoleculeInsertionV1,
    PendingCreateInterchangeBatchV1, PendingCreateMolecule, PersistentId,
    RendererAdmittedPendingV1, SessionDocumentObservationV1, SessionOperationResultV1,
};

/// Opaque one-use renderer-admitted complete molecule insertion.
#[derive(Debug)]
pub struct PendingAdmittedMoleculeInsertionV1 {
    pending: PendingCreateMolecule,
    observation: SessionDocumentObservationV1,
    admission: RendererAdmittedPendingV1,
}

impl PendingAdmittedMoleculeInsertionV1 {
    /// Return the exact complete renderer plan admitted for this pending insertion.
    #[must_use]
    pub(crate) fn document_render_plan_v1(&self) -> &ferrum_render::DocumentRenderPlanV1 {
        self.admission.plan()
    }

    #[must_use]
    pub fn molecule_identifier(&self) -> &PersistentId {
        self.pending.molecule_identifier()
    }

    #[must_use]
    pub fn atom_identifiers(&self) -> &[PersistentId] {
        self.pending.atom_identifiers()
    }

    #[must_use]
    pub fn bond_identifiers(&self) -> &[PersistentId] {
        self.pending.bond_identifiers()
    }

    /// Return the exact renderer plan for this pending molecule root.
    #[must_use]
    pub fn molecule_render_plan_v1(&self) -> Option<&MoleculeRenderPlan> {
        // Complete-molecule insertion appends its direct root. The complete renderer
        // plan preserves document source order, so its final molecule root is exactly
        // this pending insertion rather than frontend-reconstructed geometry.
        self.admission
            .plan()
            .outcomes()
            .iter()
            .filter_map(|outcome| match outcome {
                ferrum_render::DocumentRenderOutcomeV1::Root(root) => match root.content() {
                    DocumentRenderContentV1::Molecule(plan) => Some(plan),
                    _ => None,
                },
                ferrum_render::DocumentRenderOutcomeV1::Exclusion(_) => None,
            })
            .last()
    }
}

/// Opaque one-use renderer-admitted complete interchange insertion.
pub struct PendingAdmittedInterchangeBatchV1 {
    pending: PendingCreateInterchangeBatchV1,
    molecule_identifiers: Vec<PersistentId>,
    atom_identifiers: Vec<Vec<PersistentId>>,
    bond_identifiers: Vec<Vec<PersistentId>>,
}

impl PendingAdmittedInterchangeBatchV1 {
    /// Return the exact candidate identity used in the import summary.
    #[must_use]
    pub fn candidate_revision_and_digest_v1(&self) -> Option<(u64, [u8; 32])> {
        self.pending.candidate_revision_and_digest_v1()
    }

    #[must_use]
    pub fn molecule_identifiers(&self) -> &[PersistentId] {
        &self.molecule_identifiers
    }

    #[must_use]
    pub fn atom_identifiers(&self) -> &[Vec<PersistentId>] {
        &self.atom_identifiers
    }

    #[must_use]
    pub fn bond_identifiers(&self) -> &[Vec<PersistentId>] {
        &self.bond_identifiers
    }
}

impl DocumentSession {
    /// Prepare and renderer-admit one complete worker-built molecule.
    pub fn prepare_admitted_molecule_insertion_v1(
        &mut self,
        expected_revision: u64,
        molecule: &MoleculeInsertionV1,
    ) -> Result<PendingAdmittedMoleculeInsertionV1, DocumentSessionError> {
        let pending = self.prepare_create_molecule_v1(expected_revision, molecule)?;
        Self::admit_molecule_pending_v1(self, pending)
    }

    /// Prepare and renderer-admit one detached regular-ring molecule.
    pub fn prepare_admitted_regular_ring_insertion_v1(
        &mut self,
        expected_revision: u64,
        request: DetachedRegularRingInsertionV1,
    ) -> Result<PendingAdmittedMoleculeInsertionV1, DocumentSessionError> {
        let pending = self.prepare_create_regular_ring_v1(expected_revision, request)?;
        Self::admit_molecule_pending_v1(self, pending)
    }

    /// Verify one renderer proof and atomically accept its exact molecule candidate.
    pub fn commit_admitted_molecule_insertion_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingAdmittedMoleculeInsertionV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        pending
            .admission
            .verify(&pending.observation)
            .map_err(|_| DocumentSessionError::RendererAdmission)?;
        self.commit_create_molecule(expected_revision, &mut pending.pending)
    }

    /// Prepare and renderer-admit one atomic worker-built interchange batch.
    pub fn prepare_admitted_interchange_records_v1(
        &mut self,
        expected_revision: u64,
        batch: &InterchangeRecordBatchInsertionV1,
    ) -> Result<PendingAdmittedInterchangeBatchV1, DocumentSessionError> {
        let pending = self.prepare_create_interchange_records_v1(expected_revision, batch)?;
        let molecule_identifiers = pending.molecule_identifiers();
        let atom_identifiers = pending.atom_identifiers();
        let bond_identifiers = pending.bond_identifiers();
        Ok(PendingAdmittedInterchangeBatchV1 {
            pending,
            molecule_identifiers,
            atom_identifiers,
            bond_identifiers,
        })
    }

    /// Verify one renderer proof and atomically accept its exact interchange batch.
    pub fn commit_admitted_interchange_records_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingAdmittedInterchangeBatchV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_create_interchange_records_v1(expected_revision, &mut pending.pending)
    }

    fn admit_molecule_pending_v1(
        &mut self,
        pending: PendingCreateMolecule,
    ) -> Result<PendingAdmittedMoleculeInsertionV1, DocumentSessionError> {
        let observation = pending
            .candidate_observation_v1()
            .ok_or(DocumentSessionError::PreparedOperationConsumed)?;
        let admission = RendererAdmittedPendingV1::admit(self, &observation)
            .map_err(|_| DocumentSessionError::RendererAdmission)?;
        Ok(PendingAdmittedMoleculeInsertionV1 {
            pending,
            observation,
            admission,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MoleculeInsertionAtomV1, Point3V1};

    fn molecule() -> MoleculeInsertionV1 {
        let position = Point3V1::new(12.0, -4.0, 0.0).expect("finite position");
        let atom =
            MoleculeInsertionAtomV1::new("C", position, None, None, None).expect("carbon atom");
        MoleculeInsertionV1::new(vec![atom], Vec::new()).expect("complete molecule")
    }

    #[test]
    fn admitted_molecule_uses_its_renderer_plan_and_commits_once() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let mut pending = session
            .prepare_admitted_molecule_insertion_v1(0, &molecule())
            .expect("renderer admission");
        assert!(pending.molecule_render_plan_v1().is_some());
        let result = session
            .commit_admitted_molecule_insertion_v1(0, &mut pending)
            .expect("accepted admission");
        assert_eq!(result.observation().snapshot().revision(), 1);
    }
}

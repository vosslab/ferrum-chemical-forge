//! Revision-bound atomic insertion of complete ordered SDF records.

use crate::{
    PersistentId, SdfRecordBatchInsertionV1, SessionDocumentObservationV1, SessionOperationError,
    SessionOperationResultV1,
};

use super::{DocumentSession, DocumentSessionError, ProvisionalToken, RevisionState};

/// A one-use, revision-bound prepared batch of complete SDF records.
pub struct PendingCreateSdfRecords {
    revision: u64,
    token: ProvisionalToken,
    molecule_identifiers: Vec<PersistentId>,
    atom_identifiers: Vec<Vec<PersistentId>>,
    bond_identifiers: Vec<Vec<PersistentId>>,
    candidate: Option<RevisionState>,
}

impl std::fmt::Debug for PendingCreateSdfRecords {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateSdfRecords")
            .field("revision", &self.revision)
            .field("molecule_count", &self.molecule_identifiers.len())
            .field(
                "atom_count",
                &self.atom_identifiers.iter().map(Vec::len).sum::<usize>(),
            )
            .field(
                "bond_count",
                &self.bond_identifiers.iter().map(Vec::len).sum::<usize>(),
            )
            .field("is_resolved", &self.candidate.is_none())
            .finish()
    }
}

impl PendingCreateSdfRecords {
    /// Return durable molecule IDs in exact SDF record order.
    #[must_use]
    pub fn molecule_identifiers(&self) -> &[PersistentId] {
        &self.molecule_identifiers
    }

    /// Return each record's durable atom IDs in source order.
    #[must_use]
    pub fn atom_identifiers(&self) -> &[Vec<PersistentId>] {
        &self.atom_identifiers
    }

    /// Return each record's durable bond IDs in source order.
    #[must_use]
    pub fn bond_identifiers(&self) -> &[Vec<PersistentId>] {
        &self.bond_identifiers
    }
}

impl DocumentSession {
    /// Prepare every source-ordered SDF record as one atomic history candidate.
    pub fn prepare_create_sdf_records_v1(
        &mut self,
        expected_revision: u64,
        batch: &SdfRecordBatchInsertionV1,
    ) -> Result<PendingCreateSdfRecords, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let current = self.history.current();
        let mut generated_ids = self.generated_ids;
        let mut candidate = None;
        let mut molecule_identifiers = Vec::with_capacity(batch.records().len());
        let mut atom_identifiers = Vec::with_capacity(batch.records().len());
        let mut bond_identifiers = Vec::with_capacity(batch.records().len());

        for record in batch.records() {
            let (identities, next_generated_ids) = generated_ids.reserve_molecule(
                current.document().indexed(),
                record.molecule().atoms().len(),
                record.molecule().bonds().len(),
            )?;
            let source = candidate.as_ref().unwrap_or_else(|| current.document());
            candidate = Some(
                source
                    .with_insert_sdf_record(
                        &identities.molecule,
                        &identities.atoms,
                        &identities.bonds,
                        record,
                    )
                    .map_err(SessionOperationError::Candidate)?,
            );
            generated_ids = next_generated_ids;
            molecule_identifiers.push(identities.molecule);
            atom_identifiers.push(identities.atoms);
            bond_identifiers.push(identities.bonds);
        }

        let revision = current
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = candidate.expect("validated SDF batches contain at least one record");
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let candidate_snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        SessionDocumentObservationV1::from_state(candidate.document(), candidate_snapshot)
            .map_err(DocumentSessionError::Projection)?;
        let token = self
            .history
            .current_mut()
            .document_mut()
            .try_issue_provisional_token()
            .map_err(SessionOperationError::Candidate)?;
        self.generated_ids = generated_ids;
        Ok(PendingCreateSdfRecords {
            revision: expected_revision,
            token,
            molecule_identifiers,
            atom_identifiers,
            bond_identifiers,
            candidate: Some(candidate),
        })
    }

    /// Accept one prepared batch of complete SDF records exactly once.
    pub fn commit_create_sdf_records(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateSdfRecords,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_prepared_candidate(
            expected_revision,
            pending.revision,
            &pending.token,
            &mut pending.candidate,
        )
    }
}

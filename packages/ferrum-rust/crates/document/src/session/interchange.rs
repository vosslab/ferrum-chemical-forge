//! Revision-bound atomic insertion of complete ordered interchange records.

use crate::{
    InterchangeRecordBatchInsertionV1, PersistentId, SessionDocumentObservationV1,
    SessionOperationError, SessionOperationResultV1,
};

use super::{DocumentSession, DocumentSessionError, GeneratedIdSequences, RevisionState, prepared};

/// A one-use, revision-bound prepared batch of complete interchange records.
pub struct PendingCreateInterchangeBatchV1 {
    revision: u64,
    session_origin: u64,
    tentative_generated_ids: GeneratedIdSequences,
    molecule_identifiers: Vec<PersistentId>,
    atom_identifiers: Vec<Vec<PersistentId>>,
    bond_identifiers: Vec<Vec<PersistentId>>,
    candidate: Option<RevisionState>,
}

impl std::fmt::Debug for PendingCreateInterchangeBatchV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateInterchangeBatchV1")
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

impl PendingCreateInterchangeBatchV1 {
    /// Return the candidate revision and digest before this batch can commit.
    #[must_use]
    pub fn candidate_revision_and_digest_v1(&self) -> Option<(u64, [u8; 32])> {
        self.candidate.as_ref().map(|candidate| {
            let snapshot = candidate.snapshot(true);
            (snapshot.revision(), *snapshot.digest())
        })
    }

    /// Return durable molecule IDs in exact source record order.
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
    /// Prepare every source-ordered interchange record as one atomic history candidate.
    pub fn prepare_create_interchange_records_v1(
        &mut self,
        expected_revision: u64,
        batch: &InterchangeRecordBatchInsertionV1,
    ) -> Result<PendingCreateInterchangeBatchV1, DocumentSessionError> {
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
                    .with_insert_interchange_record(
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
        let candidate =
            candidate.expect("validated interchange batches contain at least one record");
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let candidate_snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        SessionDocumentObservationV1::from_state(candidate.document(), candidate_snapshot)
            .map_err(DocumentSessionError::Projection)?;
        Ok(PendingCreateInterchangeBatchV1 {
            revision: expected_revision,
            session_origin: self.bridge_session_origin,
            tentative_generated_ids: generated_ids,
            molecule_identifiers,
            atom_identifiers,
            bond_identifiers,
            candidate: Some(candidate),
        })
    }

    /// Accept one prepared batch of complete interchange records exactly once.
    pub fn commit_create_interchange_records_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateInterchangeBatchV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if pending.candidate.is_none() {
            return Err(DocumentSessionError::PreparedOperationConsumed);
        }
        if pending.revision != expected_revision {
            return Err(DocumentSessionError::RevisionConflict {
                expected: pending.revision,
                actual: expected_revision,
            });
        }
        if pending.session_origin != self.bridge_session_origin {
            return Err(DocumentSessionError::PreparedOperationForeignSession);
        }
        self.history
            .try_reserve_append()
            .map_err(|_| SessionOperationError::HistoryResourceExhausted)?;
        let token = prepared::issue_prepared_token(self.history.current_mut().document_mut())?;
        self.history
            .current_mut()
            .document_mut()
            .consume_provisional_token(&token)
            .map_err(SessionOperationError::Candidate)?;
        let candidate = pending
            .candidate
            .take()
            .expect("the candidate presence check established this invariant");
        self.generated_ids = pending.tentative_generated_ids;
        self.history.append_reserved(candidate);
        self.operation_result()
    }
}

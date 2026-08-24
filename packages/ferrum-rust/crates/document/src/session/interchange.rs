//! Ordered interchange insertion over the generic admitted transition boundary.

use crate::{
    InterchangeRecordBatchInsertionV1, PersistentId, SessionOperationError,
    SessionOperationResultV1,
};

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentSession, DocumentSessionError,
    PreparedSessionTransitionV1, RevisionState,
};

/// A one-use prepared batch retaining only source-order durable identifiers.
pub(crate) struct PendingCreateInterchangeBatchV1 {
    revision: u64,
    tentative_generated_ids: Vec<(PersistentId, Vec<PersistentId>, Vec<PersistentId>)>,
    transition: PreparedSessionTransitionV1,
}

impl std::fmt::Debug for PendingCreateInterchangeBatchV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateInterchangeBatchV1")
            .field("revision", &self.revision)
            .field("molecule_count", &self.tentative_generated_ids.len())
            .field("is_resolved", &self.transition.is_consumed_v1())
            .finish()
    }
}

impl PendingCreateInterchangeBatchV1 {
    pub(super) fn candidate_observation_v1(&self) -> Option<super::SessionDocumentObservationV1> {
        self.transition
            .metadata_v1()
            .map(|metadata| metadata.observation().clone())
    }

    #[must_use]
    pub(crate) fn candidate_revision_and_digest_v1(&self) -> Option<(u64, [u8; 32])> {
        self.candidate_observation_v1().map(|observation| {
            let snapshot = observation.snapshot();
            (snapshot.revision(), *snapshot.digest())
        })
    }

    #[must_use]
    pub(crate) fn molecule_identifiers(&self) -> Vec<PersistentId> {
        self.tentative_generated_ids
            .iter()
            .map(|(molecule, _, _)| molecule.clone())
            .collect()
    }

    #[must_use]
    pub(crate) fn atom_identifiers(&self) -> Vec<Vec<PersistentId>> {
        self.tentative_generated_ids
            .iter()
            .map(|(_, atoms, _)| atoms.clone())
            .collect()
    }

    #[must_use]
    pub(crate) fn bond_identifiers(&self) -> Vec<Vec<PersistentId>> {
        self.tentative_generated_ids
            .iter()
            .map(|(_, _, bonds)| bonds.clone())
            .collect()
    }
}

impl DocumentSession {
    /// Prepare every source-ordered interchange record as one admitted candidate.
    pub(crate) fn prepare_create_interchange_records_v1(
        &mut self,
        expected_revision: u64,
        batch: &InterchangeRecordBatchInsertionV1,
    ) -> Result<PendingCreateInterchangeBatchV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let (identities, effects) =
            self.reserve_generated_ids_for_transition_v1(|mut ids, indexed| {
                let mut identities = Vec::with_capacity(batch.records().len());
                for record in batch.records() {
                    let (next, generated) = ids.reserve_molecule(
                        indexed,
                        record.molecule().atoms().len(),
                        record.molecule().bonds().len(),
                    )?;
                    identities.push((next.molecule, next.atoms, next.bonds));
                    ids = generated;
                }
                Ok((identities, ids))
            })?;
        let (source_revision, source_digest, revision) = {
            let current = self.current_state_v1();
            (
                current.revision(),
                *current.digest(),
                current.next_revision(),
            )
        };
        let revision = revision.ok_or(DocumentSessionError::RevisionExhausted)?;
        let mut candidate = None;
        for (record, (molecule, atoms, bonds)) in batch.records().iter().zip(&identities) {
            let source = candidate
                .as_ref()
                .unwrap_or_else(|| self.current_state_v1().document());
            candidate = Some(
                source
                    .with_insert_interchange_record(molecule, atoms, bonds, record)
                    .map_err(SessionOperationError::Candidate)?,
            );
        }
        let candidate =
            candidate.expect("validated interchange batches contain at least one record");
        let state = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let transition = self.prepare_changed_session_transition_v1(
            source_revision,
            source_digest,
            state,
            effects,
        )?;
        Ok(PendingCreateInterchangeBatchV1 {
            revision: expected_revision,
            tentative_generated_ids: identities,
            transition,
        })
    }

    /// Redeem one prepared batch exactly once through its admitted transition.
    pub(crate) fn commit_create_interchange_records_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateInterchangeBatchV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        if pending.revision != expected_revision {
            return Err(DocumentSessionError::RevisionConflict {
                expected: pending.revision,
                actual: expected_revision,
            });
        }
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(|refusal| map_transition_refusal(self, pending.revision, refusal))
    }
}

fn map_transition_refusal(
    session: &DocumentSession,
    expected_revision: u64,
    refusal: AdmittedSessionTransitionRefusalV1,
) -> DocumentSessionError {
    match refusal {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            DocumentSessionError::PreparedOperationForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Replayed
        | AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            DocumentSessionError::PreparedOperationConsumed
        }
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            DocumentSessionError::RevisionConflict {
                expected: expected_revision,
                actual: session.current_revision_v1(),
            }
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            DocumentSessionError::RendererAdmission
        }
        AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            SessionOperationError::HistoryResourceExhausted.into()
        }
    }
}

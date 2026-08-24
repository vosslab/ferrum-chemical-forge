//! Ordered interchange insertion over the generic admitted transition boundary.

use crate::{InterchangeRecordBatchInsertionV1, SessionOperationError};

use super::{DocumentSession, DocumentSessionError, PreparedSessionTransitionV1, RevisionState};

impl DocumentSession {
    /// Lower every source-ordered interchange record into one generic transition.
    pub(in crate::session) fn prepare_insert_interchange_record_batch_transition_v1(
        &mut self,
        expected_revision: u64,
        batch: &InterchangeRecordBatchInsertionV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
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
        self.prepare_changed_session_transition_with_interchange_batch_outcome_v1(
            source_revision,
            source_digest,
            state,
            effects,
            identities,
        )
    }
}

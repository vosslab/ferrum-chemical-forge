//! One atomic document transition for a nonempty batch of complete molecules.

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentSession, DocumentSessionError, MoleculeInsertionV1,
    PreparedSessionTransitionV1, RevisionState, SessionOperationError, SessionOperationResultV1,
};

/// Opaque prepared batch held by the document transaction owner.
pub(crate) struct PendingCreateMoleculeBatchV1 {
    transition: PreparedSessionTransitionV1,
}

impl std::fmt::Debug for PendingCreateMoleculeBatchV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateMoleculeBatchV1")
            .field("is_resolved", &self.transition.is_consumed_v1())
            .finish()
    }
}

impl PendingCreateMoleculeBatchV1 {
    /// Return pre-commit candidate facts for response admission only.
    #[must_use]
    pub(crate) fn candidate_revision_and_digest_v1(&self) -> Option<(u64, [u8; 32])> {
        self.transition.metadata_v1().map(|metadata| {
            let snapshot = metadata.observation().snapshot();
            (snapshot.revision(), *snapshot.digest())
        })
    }
}

impl DocumentSession {
    /// Prepare one nonempty molecule batch as exactly one candidate revision.
    pub(crate) fn prepare_create_molecule_batch_v1(
        &mut self,
        expected_revision: u64,
        molecules: &[MoleculeInsertionV1],
    ) -> Result<PendingCreateMoleculeBatchV1, DocumentSessionError> {
        if molecules.is_empty() {
            return Err(DocumentSessionError::EmptyMoleculeBatch);
        }
        self.require_current(expected_revision)?;
        let (mut generated_ids, effects) =
            self.reserve_generated_ids_for_transition_v1(|ids, _| Ok((ids, ids)))?;
        let current = self.current_state_v1();
        let mut candidate = None;
        for molecule in molecules {
            let (identities, next_generated_ids) = generated_ids.reserve_molecule(
                current.document().indexed(),
                molecule.atoms().len(),
                molecule.bonds().len(),
            )?;
            let source = candidate.as_ref().unwrap_or_else(|| current.document());
            candidate = Some(
                source
                    .with_insert_molecule(
                        &identities.molecule,
                        &identities.atoms,
                        &identities.bonds,
                        molecule,
                    )
                    .map_err(SessionOperationError::Candidate)
                    .map_err(DocumentSessionError::Operation)?,
            );
            generated_ids = next_generated_ids;
        }
        let revision = current
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(
            revision,
            candidate.expect("nonempty molecule batch produces a candidate"),
        )
        .map_err(DocumentSessionError::Load)?;
        let transition = self.prepare_changed_session_transition_v1(
            expected_revision,
            *current.digest(),
            candidate,
            effects.installing_generated_ids(generated_ids),
        )?;
        Ok(PendingCreateMoleculeBatchV1 { transition })
    }

    /// Commit one prepared molecule batch as exactly one document history transition.
    pub(crate) fn commit_create_molecule_batch_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateMoleculeBatchV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MoleculeInsertionAtomV1, Point3V1};

    fn molecule(element: &str, x: f64) -> MoleculeInsertionV1 {
        MoleculeInsertionV1::new(
            vec![
                MoleculeInsertionAtomV1::new(
                    element,
                    Point3V1::new(x, 0.0, 0.0).expect("test coordinate is finite"),
                    None,
                    None,
                    None,
                )
                .expect("test atom is valid"),
            ],
            Vec::new(),
        )
        .expect("test molecule is valid")
    }

    #[test]
    fn empty_batch_refuses_without_a_candidate() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let revision = session.snapshot().expect("snapshot").revision();

        assert!(matches!(
            session.prepare_create_molecule_batch_v1(revision, &[]),
            Err(DocumentSessionError::EmptyMoleculeBatch)
        ));
        assert_eq!(session.snapshot().expect("snapshot").revision(), revision);
    }

    #[test]
    fn batch_commits_once_and_cannot_be_replayed() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let revision = session.snapshot().expect("snapshot").revision();
        let molecules = [molecule("C", 0.0), molecule("O", 30.0)];
        let mut pending = session
            .prepare_create_molecule_batch_v1(revision, &molecules)
            .expect("prepared batch");

        assert_eq!(
            pending
                .candidate_revision_and_digest_v1()
                .expect("candidate facts")
                .0,
            revision + 1
        );
        session
            .commit_create_molecule_batch_v1(revision, &mut pending)
            .expect("one atomic commit");
        assert_eq!(
            session.snapshot().expect("snapshot").revision(),
            revision + 1
        );
        assert!(matches!(
            session.commit_create_molecule_batch_v1(revision + 1, &mut pending),
            Err(DocumentSessionError::PreparedOperationConsumed)
        ));
        assert_eq!(
            session.snapshot().expect("snapshot").revision(),
            revision + 1
        );
    }

    #[test]
    fn discarded_batch_does_not_advance_generated_id_sequences() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let revision = session.snapshot().expect("snapshot").revision();
        let pending = session
            .prepare_create_molecule_batch_v1(revision, &[molecule("C", 0.0)])
            .expect("prepared batch");
        drop(pending);

        let mut committed = session
            .prepare_create_molecule_batch_v1(revision, &[molecule("O", 30.0)])
            .expect("second prepared batch");
        session
            .commit_create_molecule_batch_v1(revision, &mut committed)
            .expect("one atomic commit");
        let cdml = session.snapshot().expect("snapshot").cdml().to_owned();
        assert!(cdml.contains("ferrum-molecule-v1-0"));
        assert!(cdml.contains("ferrum-atom-v1-0"));
        assert!(!cdml.contains("ferrum-molecule-v1-1"));
    }

    #[test]
    fn foreign_batch_commit_preserves_the_receiving_session() {
        let mut owner = DocumentSession::create_empty_document_v1().expect("empty document");
        let mut foreign = DocumentSession::create_empty_document_v1().expect("empty document");
        let revision = owner.snapshot().expect("snapshot").revision();
        let foreign_before = foreign.snapshot().expect("snapshot");
        let mut pending = owner
            .prepare_create_molecule_batch_v1(revision, &[molecule("C", 0.0), molecule("O", 30.0)])
            .expect("prepared batch");

        assert!(matches!(
            foreign.commit_create_molecule_batch_v1(revision, &mut pending),
            Err(DocumentSessionError::PreparedOperationForeignSession)
        ));
        assert_eq!(foreign.snapshot().expect("snapshot"), foreign_before);

        owner
            .commit_create_molecule_batch_v1(revision, &mut pending)
            .expect("foreign refusal leaves the owner batch commit-ready");
        let owner_snapshot = owner.snapshot().expect("owner snapshot");
        assert_eq!(owner_snapshot.revision(), revision + 1);
        let owner_cdml = owner_snapshot.cdml();
        assert!(owner_cdml.contains("ferrum-molecule-v1-0"));
        assert!(owner_cdml.contains("ferrum-molecule-v1-1"));

        let mut foreign_pending = foreign
            .prepare_create_molecule_batch_v1(revision, &[molecule("O", 30.0)])
            .expect("foreign batch");
        foreign
            .commit_create_molecule_batch_v1(revision, &mut foreign_pending)
            .expect("foreign batch commits normally");
        let cdml = foreign.snapshot().expect("snapshot").cdml().to_owned();
        assert!(cdml.contains("ferrum-molecule-v1-0"));
        assert!(!cdml.contains("ferrum-molecule-v1-1"));
    }
}

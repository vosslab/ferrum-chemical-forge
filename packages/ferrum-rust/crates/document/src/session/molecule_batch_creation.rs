//! One atomic document transition for a nonempty batch of complete molecules.

use super::{
    DocumentSession, DocumentSessionError, MoleculeInsertionV1, RevisionState,
    SessionDocumentObservationV1, SessionOperationError, SessionOperationResultV1,
};
use crate::{AuthoringCapabilityIssuerV1, generated_ids::GeneratedIdSequences};

/// Opaque prepared batch held by the document transaction owner.
pub struct PendingCreateMoleculeBatchV1 {
    session_issuer: AuthoringCapabilityIssuerV1,
    revision: u64,
    candidate: Option<RevisionState>,
    next_generated_ids: GeneratedIdSequences,
}

impl std::fmt::Debug for PendingCreateMoleculeBatchV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateMoleculeBatchV1")
            .field("revision", &self.revision)
            .field("is_resolved", &self.candidate.is_none())
            .finish()
    }
}

impl PendingCreateMoleculeBatchV1 {
    /// Return pre-commit candidate facts for response admission only.
    #[must_use]
    pub fn candidate_revision_and_digest_v1(&self) -> Option<(u64, [u8; 32])> {
        self.candidate.as_ref().map(|candidate| {
            let snapshot = candidate.snapshot(true);
            (snapshot.revision(), *snapshot.digest())
        })
    }
}

impl DocumentSession {
    /// Prepare one nonempty molecule batch as exactly one candidate revision.
    pub fn prepare_create_molecule_batch_v1(
        &mut self,
        expected_revision: u64,
        molecules: &[MoleculeInsertionV1],
    ) -> Result<PendingCreateMoleculeBatchV1, DocumentSessionError> {
        if molecules.is_empty() {
            return Err(DocumentSessionError::EmptyMoleculeBatch);
        }
        self.require_current(expected_revision)?;
        let current = self.history.current();
        let mut generated_ids = self.generated_ids;
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
        let candidate_snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        SessionDocumentObservationV1::from_state(candidate.document(), candidate_snapshot)
            .map_err(DocumentSessionError::Projection)?;
        Ok(PendingCreateMoleculeBatchV1 {
            session_issuer: self.authoring_capability_issuer.clone(),
            revision: expected_revision,
            candidate: Some(candidate),
            next_generated_ids: generated_ids,
        })
    }

    /// Commit one prepared molecule batch as exactly one document history transition.
    pub fn commit_create_molecule_batch_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateMoleculeBatchV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if pending.candidate.is_none() {
            return Err(DocumentSessionError::PreparedOperationConsumed);
        }
        if !pending
            .session_issuer
            .same_issuer(&self.authoring_capability_issuer)
        {
            return Err(DocumentSessionError::PreparedOperationForeignSession);
        }
        if pending.revision != expected_revision {
            return Err(DocumentSessionError::RevisionConflict {
                expected: pending.revision,
                actual: expected_revision,
            });
        }
        let token =
            super::prepared::issue_prepared_token(self.history.current_mut().document_mut())?;
        self.history
            .current()
            .document()
            .verify_provisional_token(&token)
            .map_err(super::prepared::map_prepared_token_error)?;
        self.history
            .current_mut()
            .document_mut()
            .consume_provisional_token(&token)
            .map_err(SessionOperationError::Candidate)?;
        self.generated_ids = pending.next_generated_ids;
        let state = pending
            .candidate
            .take()
            .expect("the candidate presence check established this invariant");
        self.history.append(state);
        self.operation_result()
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

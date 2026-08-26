//! Session-owned atomic commit path for explicit hydrogen materialization.

use crate::{
    DocumentAtomOxidationObservationRequestV1, DocumentAtomOxidationObservationV1,
    DocumentAtomOxidationResultV1, DocumentMoleculeHydrogenMaterializationRefusalV1,
    DocumentMoleculeHydrogenMaterializationRequestV1,
    DocumentMoleculeHydrogenMaterializationResultV1, PersistentId, SessionDocumentObservationV1,
    SessionOperationError, SessionOperationOutcomeV1, SessionOperationResultV1, TypedDocument,
    hydrogen_materialization_v1::plan_hydrogen_materialization_v1,
};

use super::{
    DocumentSession, DocumentSessionError, PreparedSessionTransitionV1, RevisionState,
    admitted_transition_v1::SessionOperationOutcomeStagingV1,
};

impl DocumentSession {
    /// Materialize one fenced durable molecule through the session-owned transition lifecycle.
    pub fn materialize_molecule_hydrogens_v1(
        &mut self,
        expected_revision: u64,
        request: DocumentMoleculeHydrogenMaterializationRequestV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        let mut transition =
            self.prepare_materialize_molecule_hydrogens_transition_v1(expected_revision, request)?;
        self.commit_session_operation_transition_v1(&mut transition)
            .map_err(|refusal| map_transition_refusal(self, expected_revision, refusal))
    }

    /// Prepare one generic explicit-hydrogen transition without mutating the session.
    ///
    /// This private lowering keeps planning, source fences, candidate validation,
    /// generated IDs, and deferred effects inside the document. The generic
    /// transition lifecycle exclusively owns public preparation and redemption.
    pub(in crate::session) fn prepare_materialize_molecule_hydrogens_transition_v1(
        &mut self,
        expected_revision: u64,
        request: DocumentMoleculeHydrogenMaterializationRequestV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let snapshot = self.snapshot()?;
        if request.expected_revision() != expected_revision {
            return Err(DocumentSessionError::RevisionConflict {
                expected: request.expected_revision(),
                actual: snapshot.revision(),
            });
        }
        let source = self.current_document_v1();
        let plan = plan_hydrogen_materialization_v1(source, &snapshot, &request)
            .map_err(SessionOperationError::from)?;
        if plan.is_already_materialized() {
            self.validate_materialized_candidate(source, &request)
                .map_err(SessionOperationError::from)?;
            return self.prepare_no_change_session_transition_with_outcome_v1(
                expected_revision,
                SessionOperationOutcomeV1::MoleculeHydrogensMaterializedV1(
                    DocumentMoleculeHydrogenMaterializationResultV1::new(
                        0,
                        false,
                        request.anchor_atom_id().clone(),
                    ),
                ),
            );
        }
        let added = plan.added_hydrogen_count();
        let temporary_atoms =
            temporary_ids(source, "atom", added).map_err(SessionOperationError::from)?;
        let temporary_bonds =
            temporary_ids(source, "bond", added).map_err(SessionOperationError::from)?;
        let temporary = source
            .with_materialized_hydrogens_v1(&plan, &temporary_atoms, &temporary_bonds)
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)
            .map_err(SessionOperationError::from)?;
        self.validate_materialized_candidate(&temporary, &request)
            .map_err(SessionOperationError::from)?;

        let ((atom_ids, bond_ids), effects) = self
            .reserve_generated_ids_for_transition_v1(|mut ids, indexed| {
                let mut atom_ids = Vec::with_capacity(added);
                let mut bond_ids = Vec::with_capacity(added);
                for _ in 0..added {
                    let (atom, after_atom) = ids.reserve_atom(indexed)?;
                    let (bond, after_bond) = after_atom.reserve_bond(indexed)?;
                    atom_ids.push(atom);
                    bond_ids.push(bond);
                    ids = after_bond;
                }
                Ok(((atom_ids, bond_ids), ids))
            })
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)
            .map_err(SessionOperationError::from)?;
        let candidate = source
            .with_materialized_hydrogens_v1(&plan, &atom_ids, &bond_ids)
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)
            .map_err(SessionOperationError::from)?;
        self.validate_materialized_candidate(&candidate, &request)
            .map_err(SessionOperationError::from)?;
        let revision = self
            .next_revision_v1()
            .ok_or(DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)
            .map_err(SessionOperationError::from)?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)
            .map_err(SessionOperationError::from)?;
        self.prepare_changed_session_transition_with_commit_v1(
            super::admitted_transition_v1::ChangedSessionTransitionRequestV1::new(
                snapshot.revision(),
                *snapshot.digest(),
                state,
                effects,
            ),
            super::admitted_transition_v1::ChangedSessionTransitionCommitRequestV1::new(
                super::admitted_transition_v1::ChangedTransitionCommitV1::Append,
                SessionOperationOutcomeStagingV1::MoleculeHydrogensMaterializedV1(
                    DocumentMoleculeHydrogenMaterializationResultV1::new(
                        added,
                        true,
                        request.anchor_atom_id().clone(),
                    ),
                ),
                None,
            ),
        )
    }

    fn validate_materialized_candidate(
        &self,
        candidate: &TypedDocument,
        request: &DocumentMoleculeHydrogenMaterializationRequestV1,
    ) -> Result<(), DocumentMoleculeHydrogenMaterializationRefusalV1> {
        let revision = self
            .next_revision_v1()
            .ok_or(DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        let state = RevisionState::from_document(
            revision,
            candidate.detached_candidate().map_err(|_| {
                DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument
            })?,
        )
        .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        let snapshot = state.snapshot(!self.saved_baseline.is_current(&state));
        self.validate_projection_safety(snapshot.clone())?;
        let oxidation_request = DocumentAtomOxidationObservationRequestV1::new(
            snapshot.revision(),
            *snapshot.digest(),
            request.molecule_id().clone(),
            request.anchor_atom_id().clone(),
        );
        match super::super::chemistry::observe_current_document_atom_oxidation_v1(
            state.document(),
            &snapshot,
            &oxidation_request,
        ) {
            Ok(DocumentAtomOxidationResultV1::Observation(
                DocumentAtomOxidationObservationV1::Accepted { .. },
            )) => Ok(()),
            Ok(DocumentAtomOxidationResultV1::ResourceLimit { .. }) => {
                Err(DocumentMoleculeHydrogenMaterializationRefusalV1::ResourceLimit)
            }
            _ => Err(DocumentMoleculeHydrogenMaterializationRefusalV1::OxidationPostcondition),
        }
    }

    /// Admit the strongest presentation safety guarantee available to the
    /// document crate without creating a reverse dependency on renderer metrics.
    fn validate_projection_safety(
        &self,
        snapshot: crate::DocumentSnapshot,
    ) -> Result<(), DocumentMoleculeHydrogenMaterializationRefusalV1> {
        SessionDocumentObservationV1::from_snapshot(snapshot)
            .map(|_| ())
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnrenderableCandidate)
    }
}

fn map_transition_refusal(
    session: &DocumentSession,
    expected_revision: u64,
    refusal: super::AdmittedSessionTransitionRefusalV1,
) -> DocumentSessionError {
    match refusal {
        super::AdmittedSessionTransitionRefusalV1::ForeignSession => {
            DocumentSessionError::PreparedOperationForeignSession
        }
        super::AdmittedSessionTransitionRefusalV1::Consumed
        | super::AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            DocumentSessionError::PreparedOperationConsumed
        }
        super::AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            DocumentSessionError::RevisionConflict {
                expected: expected_revision,
                actual: session.current_revision_v1(),
            }
        }
        super::AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            DocumentSessionError::RendererAdmission
        }
    }
}

fn temporary_ids(
    document: &TypedDocument,
    kind: &str,
    count: usize,
) -> Result<Vec<PersistentId>, DocumentMoleculeHydrogenMaterializationRefusalV1> {
    let mut result = Vec::with_capacity(count);
    let mut sequence = 0_u64;
    while result.len() < count {
        let candidate = PersistentId::new(format!(
            "ferrum-materialization-candidate-{kind}-{sequence}"
        ))
        .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        sequence = sequence
            .checked_add(1)
            .ok_or(DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        if document.indexed().resolve_id(&candidate).is_none() {
            result.push(candidate);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        DocumentAtomOxidationObservationRequestV1, DocumentAtomOxidationObservationV1,
        DocumentAtomOxidationResultV1, DocumentSession, MoleculeInsertionAtomV1,
        MoleculeInsertionV1, Point3V1, SessionOperation, SessionOperationOutcomeV1,
        SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
    };

    fn atom(element: &str, formal_charge: Option<i32>) -> MoleculeInsertionAtomV1 {
        MoleculeInsertionAtomV1::new(
            element,
            Point3V1::new(0.0, 0.0, 0.0).expect("finite coordinate"),
            formal_charge,
            None,
            None,
        )
        .expect("valid atom")
    }

    fn session_with_one_atom(element: &str, formal_charge: Option<i32>) -> DocumentSession {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let insertion = MoleculeInsertionV1::new(vec![atom(element, formal_charge)], Vec::new())
            .expect("valid complete molecule");
        let revision = session.snapshot().expect("initial snapshot").revision();
        let mut pending = session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                revision,
                SessionOperation::V1(SessionOperationV1::InsertMoleculeV1(insertion.into())),
                TransitionAuthorizationV1::None,
            ))
            .expect("molecule candidate");
        session
            .commit_session_operation_transition_v1(&mut pending)
            .expect("molecule commit");
        session
    }

    fn materialization_request(
        session: &DocumentSession,
    ) -> DocumentMoleculeHydrogenMaterializationRequestV1 {
        let revision = session.snapshot().expect("current snapshot").revision();
        let observation = session.observe(revision).expect("committed observation");
        let molecule = &observation.projection().molecules()[0];
        DocumentMoleculeHydrogenMaterializationRequestV1::new(
            observation.snapshot().revision(),
            *observation.snapshot().digest(),
            molecule.id().expect("durable molecule").clone(),
            molecule.atoms()[0].id().expect("durable atom").clone(),
        )
    }

    fn oxidation_request(session: &DocumentSession) -> DocumentAtomOxidationObservationRequestV1 {
        let revision = session.snapshot().expect("current snapshot").revision();
        let observation = session.observe(revision).expect("materialized observation");
        let molecule = &observation.projection().molecules()[0];
        DocumentAtomOxidationObservationRequestV1::new(
            observation.snapshot().revision(),
            *observation.snapshot().digest(),
            molecule.id().expect("durable molecule").clone(),
            molecule.atoms()[0].id().expect("durable atom").clone(),
        )
    }

    #[test]
    fn generic_materialization_changes_one_history_transition() {
        let mut session = session_with_one_atom("O", None);
        let before = session.snapshot().expect("before materialization");
        let request = materialization_request(&session);
        let mut prepared = session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                request.expected_revision(),
                SessionOperation::V1(SessionOperationV1::MaterializeMoleculeHydrogensV1(request)),
                TransitionAuthorizationV1::None,
            ))
            .expect("materialization prepares");
        let result = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("materialization commits");
        let SessionOperationOutcomeV1::MoleculeHydrogensMaterializedV1(outcome) = result.outcome()
        else {
            panic!("materialization produces its generic outcome");
        };
        assert!(outcome.changed());
        assert!(outcome.added_hydrogen_count() > 0);
        let after = session.snapshot().expect("after materialization");
        assert_ne!(after, before);
        assert_eq!(
            session.observe_atom_oxidation_v1(&oxidation_request(&session)),
            Ok(DocumentAtomOxidationResultV1::Observation(
                DocumentAtomOxidationObservationV1::Accepted {
                    oxidation_number: -2,
                }
            ))
        );

        let undone = session
            .undo(after.revision())
            .expect("materialization undoes");
        assert_eq!(undone.observation().snapshot().cdml(), before.cdml());
        let redone = session
            .redo(undone.observation().snapshot().revision())
            .expect("materialization redoes");
        assert_eq!(redone.observation().snapshot().cdml(), after.cdml());
    }

    #[test]
    fn already_materialized_root_returns_validated_no_op() {
        let mut session = session_with_one_atom("O", None);
        let request = materialization_request(&session);
        let mut prepared = session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                request.expected_revision(),
                SessionOperation::V1(SessionOperationV1::MaterializeMoleculeHydrogensV1(request)),
                TransitionAuthorizationV1::None,
            ))
            .expect("first materialization prepares");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("first materialization commits");
        let before = session.snapshot().expect("materialized snapshot");
        let request = materialization_request(&session);
        let mut prepared = session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                request.expected_revision(),
                SessionOperation::V1(SessionOperationV1::MaterializeMoleculeHydrogensV1(request)),
                TransitionAuthorizationV1::None,
            ))
            .expect("no-change materialization prepares");
        let result = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("no-change materialization commits");
        let SessionOperationOutcomeV1::MoleculeHydrogensMaterializedV1(outcome) = result.outcome()
        else {
            panic!("materialization produces its generic outcome");
        };

        assert!(!outcome.changed());
        assert_eq!(outcome.added_hydrogen_count(), 0);
        assert_eq!(session.snapshot().expect("no-op snapshot"), before);
    }

    #[test]
    fn nonneutral_root_refuses_without_mutating_session() {
        let mut session = session_with_one_atom("O", Some(1));
        let before = session.snapshot().expect("before refusal");
        let can_undo_before = session.can_undo();
        let can_redo_before = session.can_redo();
        let refusal = session.prepare_session_operation_transition_v1(
            SessionOperationTransitionRequestV1::new(
                materialization_request(&session).expected_revision(),
                SessionOperation::V1(SessionOperationV1::MaterializeMoleculeHydrogensV1(
                    materialization_request(&session),
                )),
                TransitionAuthorizationV1::None,
            ),
        );
        assert!(matches!(
            refusal,
            Err(crate::DocumentSessionError::Operation(
                SessionOperationError::HydrogenMaterialization(
                    DocumentMoleculeHydrogenMaterializationRefusalV1::NonzeroFormalCharge,
                ),
            ))
        ));
        assert_eq!(session.snapshot().expect("after refusal"), before);
        assert_eq!(session.can_undo(), can_undo_before);
        assert_eq!(session.can_redo(), can_redo_before);
    }

    #[test]
    fn generic_materialization_prepared_transition_is_one_use() {
        let mut session = session_with_one_atom("O", None);
        let request = materialization_request(&session);
        let mut prepared = session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                request.expected_revision(),
                SessionOperation::V1(SessionOperationV1::MaterializeMoleculeHydrogensV1(request)),
                TransitionAuthorizationV1::None,
            ))
            .expect("materialization prepares");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("first materialization commit");
        assert_eq!(
            session.commit_session_operation_transition_v1(&mut prepared),
            Err(crate::AdmittedSessionTransitionRefusalV1::Consumed)
        );
    }
}

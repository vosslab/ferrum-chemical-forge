//! Session-owned atomic commit path for explicit hydrogen materialization.

use crate::{
    DocumentAtomOxidationObservationRequestV1, DocumentAtomOxidationObservationV1,
    DocumentAtomOxidationResultV1, DocumentMoleculeHydrogenMaterializationRefusalV1,
    DocumentMoleculeHydrogenMaterializationRequestV1,
    DocumentMoleculeHydrogenMaterializationResultV1, PersistentId, SessionDocumentObservationV1,
    SessionOperationResultV1, TypedDocument,
    hydrogen_materialization_v1::plan_hydrogen_materialization_v1,
};

use super::{DocumentSession, PreparedSessionTransitionV1, RevisionState};

/// Opaque one-use, session-bound materialization candidate for renderer admission.
///
/// The receipt retains a core-owned transition and public materialization facts.
#[derive(Debug)]
pub struct PendingHydrogenMaterializationV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    transition: PreparedSessionTransitionV1,
    result: DocumentMoleculeHydrogenMaterializationResultV1,
}

impl PendingHydrogenMaterializationV1 {
    #[must_use]
    pub fn is_consumed_v1(&self) -> bool {
        self.transition.is_consumed_v1()
    }
}

impl DocumentSession {
    /// Build one bounded explicit-H candidate without changing the session.
    ///
    /// The renderer must admit the exact candidate observation before redeeming
    /// this opaque receipt. Preparation performs the document
    /// projection and oxidation checks, reserves history capacity, and retains
    /// tentative IDs without installing any of them.
    pub fn prepare_materialize_molecule_hydrogens_v1(
        &mut self,
        request: &DocumentMoleculeHydrogenMaterializationRequestV1,
    ) -> Result<PendingHydrogenMaterializationV1, DocumentMoleculeHydrogenMaterializationRefusalV1>
    {
        let snapshot = self
            .snapshot()
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        let source = self.current_document_v1();
        let plan = plan_hydrogen_materialization_v1(source, &snapshot, request)?;
        if plan.is_already_materialized() {
            self.validate_materialized_candidate(source, request)?;
            let transition = self
                .prepare_no_change_session_transition_v1(snapshot.revision())
                .map_err(|_| {
                    DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument
                })?;
            return Ok(PendingHydrogenMaterializationV1 {
                expected_revision: snapshot.revision(),
                expected_digest: *snapshot.digest(),
                transition,
                result: DocumentMoleculeHydrogenMaterializationResultV1::new(
                    0,
                    false,
                    request.anchor_atom_id().clone(),
                    snapshot.revision(),
                    *snapshot.digest(),
                ),
            });
        }
        let added = plan.added_hydrogen_count();
        let temporary_atoms = temporary_ids(source, "atom", added)?;
        let temporary_bonds = temporary_ids(source, "bond", added)?;
        let temporary = source
            .with_materialized_hydrogens_v1(&plan, &temporary_atoms, &temporary_bonds)
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        self.validate_materialized_candidate(&temporary, request)?;

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
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        let candidate = source
            .with_materialized_hydrogens_v1(&plan, &atom_ids, &bond_ids)
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        self.validate_materialized_candidate(&candidate, request)?;
        let revision = self
            .next_revision_v1()
            .ok_or(DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        let candidate_snapshot = state.snapshot(!self.saved_baseline.is_current(&state));
        let transition = self
            .prepare_changed_session_transition_v1(
                snapshot.revision(),
                *snapshot.digest(),
                state,
                effects,
            )
            .map_err(map_prepare_refusal)?;
        Ok(PendingHydrogenMaterializationV1 {
            expected_revision: snapshot.revision(),
            expected_digest: *snapshot.digest(),
            transition,
            result: DocumentMoleculeHydrogenMaterializationResultV1::new(
                added,
                true,
                request.anchor_atom_id().clone(),
                candidate_snapshot.revision(),
                *candidate_snapshot.digest(),
            ),
        })
    }

    /// Atomically install one renderer-admitted materialization candidate.
    ///
    /// This compatibility form returns only the materialization facts. Renderer
    /// bridges that install a live projection use
    /// [`Self::commit_materialize_molecule_hydrogens_with_operation_result_v1`]
    /// so their accepted mutation retains its authoritative post-commit receipt.
    pub fn commit_materialize_molecule_hydrogens_v1(
        &mut self,
        pending: &mut PendingHydrogenMaterializationV1,
    ) -> Result<
        DocumentMoleculeHydrogenMaterializationResultV1,
        DocumentMoleculeHydrogenMaterializationRefusalV1,
    > {
        self.commit_materialize_molecule_hydrogens_with_operation_result_v1(pending)
            .map(|(result, _)| result)
    }

    /// Atomically install one renderer-admitted materialization candidate and
    /// return its authoritative post-commit observation when it changed.
    ///
    /// The operation result is prepared from the same validated candidate state
    /// that this method installs. A validated no-op has no mutation receipt.
    pub fn commit_materialize_molecule_hydrogens_with_operation_result_v1(
        &mut self,
        pending: &mut PendingHydrogenMaterializationV1,
    ) -> Result<
        (
            DocumentMoleculeHydrogenMaterializationResultV1,
            Option<SessionOperationResultV1>,
        ),
        DocumentMoleculeHydrogenMaterializationRefusalV1,
    > {
        if pending.transition.is_consumed_v1() {
            return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::StaleObservation);
        }
        let snapshot = self
            .snapshot()
            .map_err(|_| DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument)?;
        if snapshot.revision() != pending.expected_revision {
            return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::StaleObservation);
        }
        if snapshot.digest() != &pending.expected_digest {
            return Err(DocumentMoleculeHydrogenMaterializationRefusalV1::DigestMismatch);
        }
        let operation = self
            .commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(map_commit_refusal)?;
        let changed = pending.result.changed();
        Ok((pending.result.clone(), changed.then_some(operation)))
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

fn map_prepare_refusal(
    error: super::DocumentSessionError,
) -> DocumentMoleculeHydrogenMaterializationRefusalV1 {
    match error {
        super::DocumentSessionError::RendererAdmission => {
            DocumentMoleculeHydrogenMaterializationRefusalV1::RendererAdmission
        }
        _ => DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument,
    }
}

fn map_commit_refusal(
    refusal: super::AdmittedSessionTransitionRefusalV1,
) -> DocumentMoleculeHydrogenMaterializationRefusalV1 {
    match refusal {
        super::AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            DocumentMoleculeHydrogenMaterializationRefusalV1::RendererAdmission
        }
        super::AdmittedSessionTransitionRefusalV1::StaleSnapshot
        | super::AdmittedSessionTransitionRefusalV1::ForeignSession
        | super::AdmittedSessionTransitionRefusalV1::Replayed
        | super::AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            DocumentMoleculeHydrogenMaterializationRefusalV1::StaleObservation
        }
        super::AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            DocumentMoleculeHydrogenMaterializationRefusalV1::UnsupportedDocument
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
        MoleculeInsertionBondV1, MoleculeInsertionV1, Point3V1,
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
            .prepare_create_molecule_v1(revision, &insertion)
            .expect("molecule candidate");
        session
            .commit_create_molecule(revision, &mut pending)
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

    fn commit_materialization(
        session: &mut DocumentSession,
    ) -> Result<
        DocumentMoleculeHydrogenMaterializationResultV1,
        DocumentMoleculeHydrogenMaterializationRefusalV1,
    > {
        let request = materialization_request(session);
        let mut pending = session.prepare_materialize_molecule_hydrogens_v1(&request)?;
        session.commit_materialize_molecule_hydrogens_v1(&mut pending)
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
    fn materialization_is_observable_and_undoable_as_one_transition() {
        let mut session = session_with_one_atom("O", None);
        let before = session.snapshot().expect("before materialization");
        let result = commit_materialization(&mut session).expect("oxygen materializes");
        assert!(result.changed());
        assert!(result.added_hydrogen_count() > 0);
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
        commit_materialization(&mut session).expect("oxygen materializes");
        let before = session.snapshot().expect("materialized snapshot");
        let result = commit_materialization(&mut session).expect("materialized root validates");

        assert!(!result.changed());
        assert_eq!(result.added_hydrogen_count(), 0);
        assert_eq!(session.snapshot().expect("no-op snapshot"), before);
    }

    #[test]
    fn nonneutral_root_refuses_without_mutating_session() {
        let mut session = session_with_one_atom("O", Some(1));
        let before = session.snapshot().expect("before refusal");
        let can_undo_before = session.can_undo();
        let can_redo_before = session.can_redo();
        assert_eq!(
            commit_materialization(&mut session),
            Err(DocumentMoleculeHydrogenMaterializationRefusalV1::NonzeroFormalCharge)
        );
        assert_eq!(session.snapshot().expect("after refusal"), before);
        assert_eq!(session.can_undo(), can_undo_before);
        assert_eq!(session.can_redo(), can_redo_before);
    }

    #[test]
    fn over_component_explicit_root_refuses_instead_of_reporting_no_op() {
        let mut atoms = Vec::new();
        let mut bonds = Vec::new();
        for component in 0..65 {
            let oxygen = atoms.len();
            let x = f64::from(component) * 80.0;
            atoms.push(
                MoleculeInsertionAtomV1::new(
                    "O",
                    Point3V1::new(x, 0.0, 0.0).expect("finite coordinate"),
                    Some(0),
                    None,
                    Some(0),
                )
                .expect("valid oxygen"),
            );
            for offset in [-1.0, 1.0] {
                let hydrogen = atoms.len();
                atoms.push(
                    MoleculeInsertionAtomV1::new(
                        "H",
                        Point3V1::new(x + offset * 24.0, 0.0, 0.0).expect("finite coordinate"),
                        Some(0),
                        None,
                        Some(0),
                    )
                    .expect("valid hydrogen"),
                );
                bonds.push(MoleculeInsertionBondV1::new(
                    oxygen,
                    hydrogen,
                    crate::DocumentBondOrderV1::Single,
                ));
            }
        }
        let insertion = MoleculeInsertionV1::new(atoms, bonds).expect("valid explicit root");
        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let revision = session.snapshot().expect("initial snapshot").revision();
        let mut pending = session
            .prepare_create_molecule_v1(revision, &insertion)
            .expect("explicit root candidate");
        session
            .commit_create_molecule(revision, &mut pending)
            .expect("explicit root commit");
        let before = session.snapshot().expect("before refusal");

        assert_eq!(
            commit_materialization(&mut session),
            Err(DocumentMoleculeHydrogenMaterializationRefusalV1::ResourceLimit)
        );
        assert_eq!(session.snapshot().expect("after refusal"), before);
    }
}

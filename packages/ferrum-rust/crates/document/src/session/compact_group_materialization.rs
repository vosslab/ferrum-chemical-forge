//! Session-owned transition path for typed compact-group materialization.

use crate::compact_group_materialization_v1::{
    CompactGroupMaterializationRefusalV1, TypedCompactGroupMaterializationRequestV1,
};
use crate::{
    DocumentCompactGroupMaterializationRefusalV1, DocumentCompactGroupMaterializationRequestV1,
    DocumentCompactGroupMaterializationResultV1,
    DocumentCompactGroupMaterializationTargetErrorV1, DocumentObjectIdV1, PersistentId,
    SessionOperationError, TypedClass, document_object_id_from_record_v1,
};

use super::{
    DocumentSession, DocumentSessionError, PreparedSessionTransitionV1, RevisionState,
    admitted_transition_v1::SessionOperationOutcomeStagingV1,
};

impl DocumentSession {
    /// Lower one current durable molecule/group pair for the shared compact transition.
    pub fn lower_compact_group_materialization_targets_v1(
        &self,
        molecule_object_id: &DocumentObjectIdV1,
        compact_group_object_id: &DocumentObjectIdV1,
    ) -> Result<
        (PersistentId, PersistentId),
        DocumentCompactGroupMaterializationTargetErrorV1,
    > {
        let document = self.current_document_v1();
        let molecule = document
            .resolve_document_object_id(molecule_object_id)
            .ok_or(DocumentCompactGroupMaterializationTargetErrorV1::UnknownMolecule)?;
        if molecule.class() != TypedClass::Molecule {
            return Err(DocumentCompactGroupMaterializationTargetErrorV1::InvalidMolecule);
        }
        let molecule_id = persistent_id(
            molecule,
            DocumentCompactGroupMaterializationTargetErrorV1::InvalidMolecule,
        )?;
        let compact_group = molecule
            .typed_children()
            .iter()
            .find(|child| {
                document_object_id_from_record_v1(child.record()).as_ref()
                    == Some(compact_group_object_id)
            })
            .ok_or(
                DocumentCompactGroupMaterializationTargetErrorV1::UnknownOrForeignCompactGroup,
            )?
            .record();
        if compact_group.class() != TypedClass::CompactGroup {
            return Err(DocumentCompactGroupMaterializationTargetErrorV1::InvalidCompactGroup);
        }
        let compact_group_id = persistent_id(
            compact_group,
            DocumentCompactGroupMaterializationTargetErrorV1::InvalidCompactGroup,
        )?;
        Ok((molecule_id, compact_group_id))
    }

    /// Prepare one replacement through the generic renderer-admitted lifecycle.
    pub(in crate::session) fn prepare_materialize_compact_group_transition_v1(
        &mut self,
        expected_revision: u64,
        request: DocumentCompactGroupMaterializationRequestV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let snapshot = self.snapshot()?;
        if request.expected_revision() != expected_revision {
            return Err(DocumentSessionError::RevisionConflict {
                expected: request.expected_revision(),
                actual: snapshot.revision(),
            });
        }
        if request.expected_digest() != snapshot.digest() {
            return Err(SessionOperationError::from(
                DocumentCompactGroupMaterializationRefusalV1::DigestMismatch,
            )
            .into());
        }

        let source = self.current_document_v1();
        let initial_request = compact_request(&request, Vec::new(), Vec::new());
        let initial_plan = source
            .prepare_compact_group_materialization_v1(initial_request)
            .map_err(map_compact_refusal)?;
        // Source-absent probe IDs size and validate only the detached candidate; they never
        // enter history or session allocation. Durable IDs are reserved and installed only after
        // validation through generic transition effects.
        let probe_candidate_atoms = probe_candidate_ids(source, "atom", initial_plan.atom_count())?;
        let probe_candidate_bonds = probe_candidate_ids(source, "bond", initial_plan.bond_count())?;
        let probe_candidate_plan = source
            .prepare_compact_group_materialization_v1(compact_request(
                &request,
                probe_candidate_atoms,
                probe_candidate_bonds,
            ))
            .map_err(map_compact_refusal)?;
        source
            .materialize_compact_group_v1(&probe_candidate_plan)
            .map_err(map_compact_refusal)?;

        let atom_count = initial_plan.atom_count();
        let bond_count = initial_plan.bond_count();
        let ((atom_ids, bond_ids), effects) = self
            .reserve_generated_ids_for_transition_v1(|mut ids, indexed| {
                let mut atom_ids = Vec::with_capacity(atom_count);
                let mut bond_ids = Vec::with_capacity(bond_count);
                for _ in 0..atom_count {
                    let (atom, after_atom) = ids.reserve_atom(indexed)?;
                    atom_ids.push(atom);
                    ids = after_atom;
                }
                for _ in 0..bond_count {
                    let (bond, after_bond) = ids.reserve_bond(indexed)?;
                    bond_ids.push(bond);
                    ids = after_bond;
                }
                Ok(((atom_ids, bond_ids), ids))
            })
            .map_err(|_| DocumentCompactGroupMaterializationRefusalV1::InvalidCandidate)
            .map_err(SessionOperationError::from)?;
        let plan = source
            .prepare_compact_group_materialization_v1(compact_request(&request, atom_ids, bond_ids))
            .map_err(map_compact_refusal)?;
        let candidate = source
            .materialize_compact_group_v1(&plan)
            .map_err(map_compact_refusal)?;
        let focus_source_id = candidate.attachment_focus().clone();
        let candidate = candidate.into_candidate();
        let molecule_id = candidate
            .root()
            .children_of(TypedClass::Molecule)
            .find(|molecule| molecule.attribute("id") == Some(request.molecule_id().as_str()))
            .and_then(document_object_id_from_record_v1)
            .ok_or(DocumentCompactGroupMaterializationRefusalV1::InvalidCandidate)
            .map_err(SessionOperationError::from)?;
        let focus_atom_id = candidate
            .root()
            .children_of(TypedClass::Molecule)
            .find(|molecule| molecule.attribute("id") == Some(request.molecule_id().as_str()))
            .and_then(|molecule| {
                molecule.typed_children().iter().find(|child| {
                    child.record().class() == TypedClass::Atom
                        && child.record().attribute("id") == Some(focus_source_id.as_str())
                })
            })
            .and_then(|atom| document_object_id_from_record_v1(atom.record()))
            .ok_or(DocumentCompactGroupMaterializationRefusalV1::InvalidCandidate)
            .map_err(SessionOperationError::from)?;
        let revision = self
            .next_revision_v1()
            .ok_or(DocumentCompactGroupMaterializationRefusalV1::InvalidCandidate)
            .map_err(SessionOperationError::from)?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(|_| DocumentCompactGroupMaterializationRefusalV1::InvalidCandidate)
            .map_err(SessionOperationError::from)?;
        self.prepare_changed_session_transition_with_commit_v1(
            snapshot.revision(),
            *snapshot.digest(),
            state,
            effects,
            super::admitted_transition_v1::ChangedTransitionCommitV1::Append,
            SessionOperationOutcomeStagingV1::CompactGroupMaterializedV1(
                DocumentCompactGroupMaterializationResultV1::new(
                    molecule_id,
                    request.compact_group_id().clone(),
                    focus_atom_id,
                ),
            ),
            None,
        )
    }
}

fn persistent_id(
    record: &crate::TypedRecord,
    error: DocumentCompactGroupMaterializationTargetErrorV1,
) -> Result<PersistentId, DocumentCompactGroupMaterializationTargetErrorV1> {
    let source_id = record.attribute("id").ok_or(error)?;
    PersistentId::new(source_id.to_owned()).map_err(|_| error)
}

fn compact_request(
    request: &DocumentCompactGroupMaterializationRequestV1,
    atom_ids: Vec<PersistentId>,
    bond_ids: Vec<PersistentId>,
) -> TypedCompactGroupMaterializationRequestV1 {
    TypedCompactGroupMaterializationRequestV1::new(
        request.molecule_id().clone(),
        request.compact_group_id().clone(),
        atom_ids,
        bond_ids,
    )
}

fn probe_candidate_ids(
    document: &crate::TypedDocument,
    kind: &str,
    count: usize,
) -> Result<Vec<PersistentId>, DocumentSessionError> {
    let mut result = Vec::with_capacity(count);
    let mut sequence = 0_u64;
    while result.len() < count {
        let candidate =
            PersistentId::new(format!("ferrum-compact-group-candidate-{kind}-{sequence}"))
                .map_err(|_| DocumentCompactGroupMaterializationRefusalV1::InvalidCandidate)
                .map_err(SessionOperationError::from)?;
        sequence = sequence
            .checked_add(1)
            .ok_or(DocumentCompactGroupMaterializationRefusalV1::InvalidCandidate)
            .map_err(SessionOperationError::from)?;
        if document.indexed().resolve_id(&candidate).is_none() {
            result.push(candidate);
        }
    }
    Ok(result)
}

fn map_compact_refusal(refusal: CompactGroupMaterializationRefusalV1) -> DocumentSessionError {
    let mapped = match refusal {
        CompactGroupMaterializationRefusalV1::InvalidTarget
        | CompactGroupMaterializationRefusalV1::StalePlan => {
            DocumentCompactGroupMaterializationRefusalV1::InvalidTarget
        }
        CompactGroupMaterializationRefusalV1::UnsupportedRecipe => {
            DocumentCompactGroupMaterializationRefusalV1::UnsupportedRecipe
        }
        CompactGroupMaterializationRefusalV1::InvalidTopology => {
            DocumentCompactGroupMaterializationRefusalV1::InvalidTopology
        }
        CompactGroupMaterializationRefusalV1::InvalidSuppliedIds
        | CompactGroupMaterializationRefusalV1::InvalidCandidate => {
            DocumentCompactGroupMaterializationRefusalV1::InvalidCandidate
        }
    };
    SessionOperationError::from(mapped).into()
}

#[cfg(test)]
mod tests {
    use crate::{
        DocumentCompactGroupMaterializationRequestV1, DocumentSession, PersistentId,
        SessionOperation, SessionOperationOutcomeV1, SessionOperationTransitionRequestV1,
        SessionOperationV1, TransitionAuthorizationV1,
    };

    fn id(value: &str) -> PersistentId {
        PersistentId::new(value.to_owned()).expect("test identifier")
    }

    fn session() -> DocumentSession {
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\"/></molecule></cdml>").expect("typed compact-group session")
    }

    fn request(session: &DocumentSession) -> DocumentCompactGroupMaterializationRequestV1 {
        let snapshot = session.snapshot().expect("current snapshot");
        DocumentCompactGroupMaterializationRequestV1::new(
            snapshot.revision(),
            *snapshot.digest(),
            id("m"),
            id("group"),
        )
    }

    #[test]
    fn generic_compact_group_materialization_preserves_exterior_bond_and_history() {
        let mut session = session();
        let before = session.snapshot().expect("compact source");
        let request = request(&session);
        let mut prepared = session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                request.expected_revision(),
                SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
                TransitionAuthorizationV1::None,
            ))
            .expect("materialization prepares");
        let result = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("materialization commits");
        let SessionOperationOutcomeV1::CompactGroupMaterializedV1(outcome) = result.outcome()
        else {
            panic!("compact group returns focused outcome");
        };
        let after = session.snapshot().expect("materialized snapshot");
        assert!(after.cdml().contains("id=\"outside\""));
        assert!(after.cdml().contains(outcome.focus_atom_id().as_str()));
        assert!(!after.cdml().contains("<compact-group"));
        assert_eq!(
            session.commit_session_operation_transition_v1(&mut prepared),
            Err(crate::AdmittedSessionTransitionRefusalV1::Replayed)
        );
        let undone = session
            .undo(after.revision())
            .expect("materialization undoes");
        assert!(
            undone
                .observation()
                .snapshot()
                .cdml()
                .contains("<compact-group")
        );
        let redone = session
            .redo(undone.observation().snapshot().revision())
            .expect("materialization redoes");
        assert!(
            !redone
                .observation()
                .snapshot()
                .cdml()
                .contains("<compact-group")
        );
        assert_ne!(before, after);
    }
}

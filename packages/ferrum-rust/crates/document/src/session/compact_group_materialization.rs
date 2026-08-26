//! Session-owned transition path for typed compact-group materialization.

use crate::compact_group_materialization_v1::{
    CompactGroupMaterializationRefusalV1, TypedCompactGroupMaterializationRequestV1,
};
use crate::{
    DocumentCompactGroupMaterializationRefusalV1, DocumentCompactGroupMaterializationRequestV1,
    DocumentCompactGroupMaterializationResultV1, DocumentCompactGroupMaterializationTargetErrorV1,
    DocumentObjectIdV1, PersistentId, SessionOperationError, TypedClass,
    document_object_id_from_record_v1,
};

use super::{
    DocumentSession, DocumentSessionError, PreparedSessionTransitionV1, RevisionState,
    admitted_transition_v1::SessionOperationOutcomeStagingV1,
};

impl DocumentSession {
    /// Privately lower one current durable molecule/group pair for detached mutation.
    fn lower_compact_group_materialization_targets_v1(
        &self,
        molecule_object_id: &DocumentObjectIdV1,
        compact_group_object_id: &DocumentObjectIdV1,
    ) -> Result<(PersistentId, PersistentId), DocumentCompactGroupMaterializationTargetErrorV1>
    {
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
            .ok_or(DocumentCompactGroupMaterializationTargetErrorV1::UnknownOrForeignCompactGroup)?
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

        let (molecule_source_id, compact_group_source_id) = self
            .lower_compact_group_materialization_targets_v1(
                request.molecule_id(),
                request.compact_group_id(),
            )
            .map_err(|_| DocumentCompactGroupMaterializationRefusalV1::InvalidTarget)
            .map_err(SessionOperationError::from)?;
        let source = self.current_document_v1();
        let initial_request = compact_request(
            &molecule_source_id,
            &compact_group_source_id,
            Vec::new(),
            Vec::new(),
        );
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
                &molecule_source_id,
                &compact_group_source_id,
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
            .prepare_compact_group_materialization_v1(compact_request(
                &molecule_source_id,
                &compact_group_source_id,
                atom_ids,
                bond_ids,
            ))
            .map_err(map_compact_refusal)?;
        let candidate = source
            .materialize_compact_group_v1(&plan)
            .map_err(map_compact_refusal)?;
        let focus_source_id = candidate.attachment_focus().clone();
        let candidate = candidate.into_candidate();
        let molecule_id = candidate
            .root()
            .children_of(TypedClass::Molecule)
            .find(|molecule| molecule.attribute("id") == Some(molecule_source_id.as_str()))
            .and_then(document_object_id_from_record_v1)
            .ok_or(DocumentCompactGroupMaterializationRefusalV1::InvalidCandidate)
            .map_err(SessionOperationError::from)?;
        let focus_atom_id = candidate
            .root()
            .children_of(TypedClass::Molecule)
            .find(|molecule| molecule.attribute("id") == Some(molecule_source_id.as_str()))
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
            super::admitted_transition_v1::ChangedSessionTransitionRequestV1::new(
                snapshot.revision(),
                *snapshot.digest(),
                state,
                effects,
            ),
            super::admitted_transition_v1::ChangedSessionTransitionCommitRequestV1::new(
                super::admitted_transition_v1::ChangedTransitionCommitV1::Append,
                SessionOperationOutcomeStagingV1::CompactGroupMaterializedV1(
                    DocumentCompactGroupMaterializationResultV1::new(
                        molecule_id,
                        request.compact_group_id().clone(),
                        focus_atom_id,
                    ),
                ),
                None,
            ),
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
    molecule_id: &PersistentId,
    compact_group_id: &PersistentId,
    atom_ids: Vec<PersistentId>,
    bond_ids: Vec<PersistentId>,
) -> TypedCompactGroupMaterializationRequestV1 {
    TypedCompactGroupMaterializationRequestV1::new(
        molecule_id.clone(),
        compact_group_id.clone(),
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
        DocumentCompactGroupMaterializationRequestV1,
        DocumentCompactGroupMaterializationTargetErrorV1, DocumentObjectIdV1, DocumentSession,
        PersistentId, SessionOperation, SessionOperationOutcomeV1,
        SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
    };

    fn object(session: &DocumentSession, source_id: &str) -> DocumentObjectIdV1 {
        session
            .current_document_v1()
            .document_object_id_for_source_id_v1(
                &PersistentId::new(source_id).expect("test source identifier"),
            )
            .expect("typed ingress persists the test record identity")
    }

    fn session() -> DocumentSession {
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\"/></molecule></cdml>").expect("typed compact-group session")
    }

    fn nitro_session() -> DocumentSession {
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"nitro\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\"/></molecule></cdml>").expect("typed nitro compact-group session")
    }

    fn ethyl_session() -> DocumentSession {
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"ethyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\"/></molecule></cdml>").expect("typed ethyl compact-group session")
    }

    fn two_molecule_session() -> DocumentSession {
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m-a\"><atom id=\"anchor-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group-a\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside-a\" start=\"anchor-a\" end=\"group-a\" type=\"n1\"/></molecule><molecule id=\"m-b\"><atom id=\"anchor-b\" name=\"C\"><point x=\"40\" y=\"0\"/></atom><compact-group id=\"group-b\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"60\" y=\"0\"/></compact-group><bond id=\"outside-b\" start=\"anchor-b\" end=\"group-b\" type=\"n1\"/></molecule></cdml>").expect("two typed compact-group molecules")
    }

    fn request(session: &DocumentSession) -> DocumentCompactGroupMaterializationRequestV1 {
        let snapshot = session.snapshot().expect("current snapshot");
        DocumentCompactGroupMaterializationRequestV1::new(
            snapshot.revision(),
            *snapshot.digest(),
            object(session, "m"),
            object(session, "group"),
        )
    }

    fn assert_ethyl_materialization(
        molecule: &crate::MoleculeProjectionV1,
        attachment_source_id: &str,
    ) {
        use ferrum_core::{BondOrder, BondStyle};

        let attachment = molecule
            .atoms()
            .iter()
            .find(|atom| atom.source_id() == Some(attachment_source_id))
            .expect("ethyl attachment carbon");
        let internal = molecule
            .bonds()
            .iter()
            .find(|bond| bond.start().source_id() == Some(attachment_source_id))
            .expect("ethyl internal bond");
        let terminal = molecule
            .atoms()
            .iter()
            .find(|atom| atom.source_id() == internal.end().source_id())
            .expect("ethyl terminal carbon");
        assert_eq!(
            (attachment.element(), attachment.formal_charge()),
            (Some("C"), None)
        );
        assert_eq!(
            (terminal.element(), terminal.formal_charge()),
            (Some("C"), None)
        );
        assert_eq!(
            (
                internal.order(),
                internal.style(),
                internal.start().source_id(),
            ),
            (
                Some(BondOrder::Single),
                Some(&BondStyle::Normal),
                Some(attachment_source_id),
            )
        );
    }

    #[test]
    fn durable_targets_require_one_current_compact_group_child_of_the_selected_molecule() {
        let session = two_molecule_session();
        let molecule = object(&session, "m-a");
        assert_eq!(
            session.lower_compact_group_materialization_targets_v1(
                &molecule,
                &object(&session, "group-a"),
            ),
            Ok((
                PersistentId::new("m-a").expect("molecule source ID"),
                PersistentId::new("group-a").expect("compact-group source ID"),
            ))
        );
        assert_eq!(
            session.lower_compact_group_materialization_targets_v1(
                &molecule,
                &object(&session, "anchor-a"),
            ),
            Err(DocumentCompactGroupMaterializationTargetErrorV1::InvalidCompactGroup)
        );
        assert_eq!(
            session.lower_compact_group_materialization_targets_v1(
                &molecule,
                &object(&session, "group-b"),
            ),
            Err(DocumentCompactGroupMaterializationTargetErrorV1::UnknownOrForeignCompactGroup)
        );
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
        let focused_molecule = result
            .observation()
            .projection()
            .molecules()
            .iter()
            .find(|molecule| molecule.id() == Some(outcome.molecule_id()))
            .expect("materialized molecule remains projected");
        assert!(
            focused_molecule
                .atoms()
                .iter()
                .any(|atom| { atom.id() == Some(outcome.focus_atom_id()) })
        );
        assert!(!after.cdml().contains("<compact-group"));
        assert_eq!(
            session.commit_session_operation_transition_v1(&mut prepared),
            Err(crate::AdmittedSessionTransitionRefusalV1::Consumed)
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

    #[test]
    fn attached_nitro_materialization_preserves_charge_topology_through_history_and_reopen() {
        use ferrum_core::BondOrder;

        let mut session = nitro_session();
        let request = request(&session);
        let mut prepared = session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                request.expected_revision(),
                SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
                TransitionAuthorizationV1::None,
            ))
            .expect("nitro materialization prepares");
        let result = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("nitro materialization commits");
        let after = result.observation().snapshot().clone();
        let molecule = &result.observation().projection().molecules()[0];
        assert_eq!(molecule.atoms().len(), 4);
        assert_eq!(
            molecule
                .atoms()
                .iter()
                .find(|atom| atom.element() == Some("N"))
                .expect("nitro nitrogen")
                .formal_charge(),
            Some(1)
        );
        let oxygen_charges = molecule
            .atoms()
            .iter()
            .filter(|atom| atom.element() == Some("O"))
            .map(|atom| atom.formal_charge())
            .collect::<Vec<_>>();
        assert_eq!(oxygen_charges, vec![None, Some(-1)]);
        assert_eq!(molecule.bonds().len(), 3);
        assert_eq!(
            molecule
                .bonds()
                .iter()
                .filter(|bond| bond.order() == Some(BondOrder::Double))
                .count(),
            1
        );
        assert_eq!(
            molecule
                .bonds()
                .iter()
                .filter(|bond| bond.order() == Some(BondOrder::Single))
                .count(),
            2
        );

        let undone = session
            .undo(after.revision())
            .expect("nitro materialization undoes");
        let redone = session
            .redo(undone.observation().snapshot().revision())
            .expect("nitro materialization redoes");
        let redone_molecule = &redone.observation().projection().molecules()[0];
        assert_eq!(
            redone_molecule
                .atoms()
                .iter()
                .find(|atom| atom.element() == Some("N"))
                .expect("redone nitro nitrogen")
                .formal_charge(),
            Some(1)
        );
        assert_eq!(
            redone_molecule
                .atoms()
                .iter()
                .filter(|atom| atom.element() == Some("O"))
                .map(|atom| atom.formal_charge())
                .collect::<Vec<_>>(),
            vec![None, Some(-1)]
        );

        let reopened = DocumentSession::load(after.cdml()).expect("nitro document reopens");
        let reopened_observation = reopened
            .document_observation()
            .expect("reopened nitro observation");
        let reopened_molecule = &reopened_observation.projection().molecules()[0];
        assert_eq!(reopened_molecule.bonds().len(), 3);
        assert_eq!(
            reopened_molecule
                .atoms()
                .iter()
                .find(|atom| atom.element() == Some("N"))
                .expect("reopened nitro nitrogen")
                .formal_charge(),
            Some(1)
        );
        assert_eq!(
            reopened_molecule
                .atoms()
                .iter()
                .filter(|atom| atom.element() == Some("O"))
                .map(|atom| atom.formal_charge())
                .collect::<Vec<_>>(),
            vec![None, Some(-1)]
        );
    }

    #[test]
    fn attached_ethyl_materialization_preserves_typed_topology_through_history_and_reopen() {
        let mut session = ethyl_session();
        let before = session.snapshot().expect("ethyl compact source");
        let request = request(&session);
        let mut prepared = session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                request.expected_revision(),
                SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
                TransitionAuthorizationV1::None,
            ))
            .expect("ethyl materialization prepares");
        let result = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("ethyl materialization commits");
        let SessionOperationOutcomeV1::CompactGroupMaterializedV1(outcome) = result.outcome()
        else {
            panic!("ethyl materialization returns a focused outcome");
        };
        let after = result.observation().snapshot().clone();
        let attachment_source_id = result.observation().projection().molecules()[0]
            .atoms()
            .iter()
            .find(|atom| atom.id() == Some(outcome.focus_atom_id()))
            .and_then(|atom| atom.source_id())
            .expect("accepted focus has a durable source ID")
            .to_owned();
        assert_eq!(after.revision(), before.revision() + 1);
        assert_ethyl_materialization(
            &result.observation().projection().molecules()[0],
            &attachment_source_id,
        );

        let undone = session
            .undo(after.revision())
            .expect("ethyl materialization undoes");
        assert_eq!(
            undone.observation().projection().molecules()[0].compact_groups()[0]
                .catalog_key()
                .as_str(),
            "ethyl"
        );

        let redone = session
            .redo(undone.observation().snapshot().revision())
            .expect("ethyl materialization redoes");
        assert_ethyl_materialization(
            &redone.observation().projection().molecules()[0],
            &attachment_source_id,
        );

        let reopened = DocumentSession::load(after.cdml()).expect("ethyl document reopens");
        let reopened_observation = reopened
            .document_observation()
            .expect("reopened ethyl observation");
        assert_ethyl_materialization(
            &reopened_observation.projection().molecules()[0],
            &attachment_source_id,
        );
    }

    #[test]
    fn free_methyl_materializes_through_one_history_transition_and_persists() {
        use crate::{CompactGroupCatalogKeyV1, DocumentFenceV1, PlaceFreeCompactGroupV1, Point3V1};

        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let fence =
            DocumentFenceV1::new(session.current_revision_v1(), session.current_digest_v1());
        let mut placement = session
            .prepare_place_free_compact_group_v1(
                fence,
                PlaceFreeCompactGroupV1::new(
                    CompactGroupCatalogKeyV1::Methyl,
                    Point3V1::new(12.0, -4.0, 0.0).expect("anchor"),
                ),
            )
            .expect("free methyl placement prepares");
        let placed = session
            .commit_place_free_compact_group_v1(&mut placement)
            .expect("free methyl placement commits");
        let placed_snapshot = placed.observation().snapshot().clone();
        let request = DocumentCompactGroupMaterializationRequestV1::new(
            placed_snapshot.revision(),
            *placed_snapshot.digest(),
            placed.molecule_object_id().clone(),
            placed.compact_group_object_id().clone(),
        );
        let mut transition = session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                request.expected_revision(),
                SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
                TransitionAuthorizationV1::None,
            ))
            .expect("direct-root materialization prepares");
        let result = session
            .commit_session_operation_transition_v1(&mut transition)
            .expect("direct-root materialization commits");
        let SessionOperationOutcomeV1::CompactGroupMaterializedV1(outcome) = result.outcome()
        else {
            panic!("materialization returns a focus outcome");
        };
        let after = result.observation().snapshot().clone();
        let molecule = result
            .observation()
            .projection()
            .molecules()
            .iter()
            .find(|molecule| molecule.id() == Some(outcome.molecule_id()))
            .expect("materialized molecule remains projected");
        assert_eq!(molecule.atoms().len(), 1);
        assert!(molecule.bonds().is_empty());
        assert!(molecule.compact_groups().is_empty());
        session
            .undo(after.revision())
            .expect("materialization undoes");
        let restored = session.document_observation().expect("undo observation");
        assert_eq!(
            restored.projection().molecules()[0].compact_groups().len(),
            1
        );
        let redone = session
            .redo(session.current_revision_v1())
            .expect("materialization redoes");
        assert_eq!(
            redone.observation().projection().molecules()[0]
                .atoms()
                .len(),
            1
        );
        let reopened = DocumentSession::load(after.cdml()).expect("materialized document reopens");
        let reopened_observation = reopened
            .document_observation()
            .expect("reopened observation");
        let reopened_molecule = &reopened_observation.projection().molecules()[0];
        assert_eq!(reopened_molecule.atoms().len(), 1);
        assert!(reopened_molecule.compact_groups().is_empty());
    }
}

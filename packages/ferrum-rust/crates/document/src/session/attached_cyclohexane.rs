//! One deferred-ID, shared-anchor cyclohexane document transition.

use thiserror::Error;

use super::{
    DocumentFenceV1, DocumentObjectIdV1, DocumentSession, GeneratedIdSequences, PersistentId,
    RevisionState, SessionDocumentObservationV1, SessionOperationResultV1,
};
use crate::{
    Point3V1,
    attached_cyclohexane_v1::{
        AttachedCyclohexaneAnchorV1, AttachedCyclohexaneErrorV1, AttachedCyclohexaneIncidentBondV1,
        AttachedCyclohexaneReleaseV1, attached_cyclohexane_candidate_v1,
    },
};

/// Opaque one-use prepared shared-anchor C6 transition.
pub struct PendingAttachedCyclohexaneV1 {
    session_origin: u64,
    fence: DocumentFenceV1,
    candidate: Option<RevisionState>,
    preview_vertices: [Point3V1; 6],
    next_generated_ids: GeneratedIdSequences,
}

impl std::fmt::Debug for PendingAttachedCyclohexaneV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingAttachedCyclohexaneV1")
            .field("revision", &self.fence.revision())
            .field("is_resolved", &self.candidate.is_none())
            .finish()
    }
}

impl PendingAttachedCyclohexaneV1 {
    /// Return copied C6 preview coordinates while this candidate remains live.
    #[must_use]
    pub fn preview_vertices(&self) -> Option<&[Point3V1; 6]> {
        self.candidate.as_ref().map(|_| &self.preview_vertices)
    }

    /// Return complete candidate facts for later response admission only.
    #[must_use]
    pub fn candidate_revision_and_digest_v1(&self) -> Option<(u64, [u8; 32])> {
        self.candidate.as_ref().map(|candidate| {
            let snapshot = candidate.snapshot(true);
            (snapshot.revision(), *snapshot.digest())
        })
    }
}

/// Closed refusal vocabulary for the one attached-C6 capability.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum AttachedCyclohexaneSessionErrorV1 {
    #[error("cyclohexane attachment revision is stale")]
    StaleRevision,
    #[error("cyclohexane attachment digest is stale")]
    StaleDigest,
    #[error("cyclohexane attachment belongs to another session")]
    ForeignSession,
    #[error("cyclohexane attachment is retired")]
    Retired,
    #[error("cyclohexane attachment anchor is unknown or not a direct atom")]
    UnknownAnchor,
    #[error("cyclohexane attachment anchor is ineligible")]
    IneligibleAnchor,
    #[error("cyclohexane attachment pose is invalid")]
    InvalidPose,
    #[error("cyclohexane attachment session conflict")]
    SessionConflict,
}

impl DocumentSession {
    /// Prepare exactly one shared-anchor C6 candidate without mutating this session.
    pub fn prepare_attach_cyclohexane_v1(
        &mut self,
        fence: DocumentFenceV1,
        anchor: DocumentObjectIdV1,
        release: AttachedCyclohexaneReleaseV1,
    ) -> Result<PendingAttachedCyclohexaneV1, AttachedCyclohexaneSessionErrorV1> {
        require_fence(self, fence)?;
        let observation = self
            .document_observation()
            .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        let resolved = resolve_anchor(&observation, &anchor)?;
        let candidate = attached_cyclohexane_candidate_v1(
            AttachedCyclohexaneAnchorV1 {
                position: resolved.position,
                element: &resolved.element,
                formal_charge: resolved.formal_charge,
                explicit_hydrogens: resolved.explicit_hydrogens,
                valence: resolved.valence,
                multiplicity: resolved.multiplicity,
                incident_bonds: &resolved.incident,
            },
            release,
        )
        .map_err(map_core_error)?;

        let indexed = self.history.current().document().indexed();
        let mut next_generated_ids = self.generated_ids;
        let mut atom_ids = Vec::with_capacity(5);
        let mut bond_ids = Vec::with_capacity(6);
        for _ in 0..5 {
            let (identifier, next) = next_generated_ids
                .reserve_atom(indexed)
                .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
            atom_ids.push(identifier);
            next_generated_ids = next;
        }
        for _ in 0..6 {
            let (identifier, next) = next_generated_ids
                .reserve_bond(indexed)
                .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
            bond_ids.push(identifier);
            next_generated_ids = next;
        }
        let atom_ids: [PersistentId; 5] = atom_ids.try_into().expect("fixed C6 atom reservation");
        let bond_ids: [PersistentId; 6] = bond_ids.try_into().expect("fixed C6 bond reservation");
        let document = self
            .history
            .current()
            .document()
            .with_attach_cyclohexane_v1(
                &resolved.molecule_id,
                &resolved.anchor_id,
                &atom_ids,
                &bond_ids,
                &candidate,
            )
            .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        let state = RevisionState::from_document(revision, document)
            .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        let snapshot = state.snapshot(!self.saved_baseline.is_current(&state));
        SessionDocumentObservationV1::from_state(state.document(), snapshot)
            .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        self.history
            .try_reserve_append()
            .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        Ok(PendingAttachedCyclohexaneV1 {
            session_origin: self.bridge_session_origin,
            fence,
            candidate: Some(state),
            preview_vertices: candidate.vertices().to_owned(),
            next_generated_ids,
        })
    }

    /// Commit one prepared C6 candidate as one history transition.
    pub fn commit_attach_cyclohexane_v1(
        &mut self,
        pending: &mut PendingAttachedCyclohexaneV1,
    ) -> Result<SessionOperationResultV1, AttachedCyclohexaneSessionErrorV1> {
        if pending.candidate.is_none() {
            return Err(AttachedCyclohexaneSessionErrorV1::Retired);
        }
        if pending.session_origin != self.bridge_session_origin {
            return Err(AttachedCyclohexaneSessionErrorV1::ForeignSession);
        }
        require_fence(self, pending.fence)?;
        let token =
            super::prepared::issue_prepared_token(self.history.current_mut().document_mut())
                .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        self.history
            .current()
            .document()
            .verify_provisional_token(&token)
            .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        self.history
            .current_mut()
            .document_mut()
            .consume_provisional_token(&token)
            .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        self.generated_ids = pending.next_generated_ids;
        let state = pending
            .candidate
            .take()
            .expect("live candidate checked above");
        self.history.append_reserved(state);
        self.operation_result()
            .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)
    }

    /// Retire one preview without requiring that its document is still current.
    pub fn retire_attach_cyclohexane_v1(
        &self,
        pending: &mut PendingAttachedCyclohexaneV1,
    ) -> Result<(), AttachedCyclohexaneSessionErrorV1> {
        if pending.session_origin != self.bridge_session_origin {
            return Err(AttachedCyclohexaneSessionErrorV1::ForeignSession);
        }
        if pending.candidate.take().is_none() {
            return Err(AttachedCyclohexaneSessionErrorV1::Retired);
        }
        Ok(())
    }
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), AttachedCyclohexaneSessionErrorV1> {
    let current = session.history.current();
    if current.revision() != fence.revision() {
        return Err(AttachedCyclohexaneSessionErrorV1::StaleRevision);
    }
    if *current.digest() != fence.digest() {
        return Err(AttachedCyclohexaneSessionErrorV1::StaleDigest);
    }
    Ok(())
}

struct ResolvedAnchorV1 {
    molecule_id: PersistentId,
    anchor_id: PersistentId,
    position: Point3V1,
    element: String,
    formal_charge: Option<i32>,
    explicit_hydrogens: Option<u16>,
    valence: Option<u16>,
    multiplicity: Option<u16>,
    incident: Vec<AttachedCyclohexaneIncidentBondV1>,
}

fn resolve_anchor(
    observation: &SessionDocumentObservationV1,
    anchor: &DocumentObjectIdV1,
) -> Result<ResolvedAnchorV1, AttachedCyclohexaneSessionErrorV1> {
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| {
            molecule
                .atoms()
                .iter()
                .any(|atom| atom.id() == Some(anchor))
        })
        .ok_or(AttachedCyclohexaneSessionErrorV1::UnknownAnchor)?;
    let atom = molecule
        .atoms()
        .iter()
        .find(|atom| atom.id() == Some(anchor))
        .ok_or(AttachedCyclohexaneSessionErrorV1::UnknownAnchor)?;
    let molecule_id = molecule
        .source_id()
        .and_then(|id| PersistentId::new(id.to_owned()).ok())
        .ok_or(AttachedCyclohexaneSessionErrorV1::UnknownAnchor)?;
    let atom_id = atom
        .source_id()
        .and_then(|id| PersistentId::new(id.to_owned()).ok())
        .ok_or(AttachedCyclohexaneSessionErrorV1::UnknownAnchor)?;
    let mut incident = Vec::new();
    for bond in molecule.bonds() {
        if bond.start().source_id() == Some(atom_id.as_str())
            || bond.end().source_id() == Some(atom_id.as_str())
        {
            incident.push(if bond.source_type() == Some("n1") {
                AttachedCyclohexaneIncidentBondV1::NormalSingle
            } else {
                AttachedCyclohexaneIncidentBondV1::Other
            });
        }
    }
    let element = atom
        .element()
        .ok_or(AttachedCyclohexaneSessionErrorV1::IneligibleAnchor)?;
    Ok(ResolvedAnchorV1 {
        molecule_id,
        anchor_id: atom_id,
        position: atom.position(),
        element: element.to_owned(),
        formal_charge: atom.formal_charge(),
        explicit_hydrogens: atom.explicit_hydrogens(),
        valence: atom.valence(),
        multiplicity: atom.multiplicity(),
        incident,
    })
}

fn map_core_error(error: AttachedCyclohexaneErrorV1) -> AttachedCyclohexaneSessionErrorV1 {
    match error {
        AttachedCyclohexaneErrorV1::IneligibleAnchor => {
            AttachedCyclohexaneSessionErrorV1::IneligibleAnchor
        }
        AttachedCyclohexaneErrorV1::InvalidPose => AttachedCyclohexaneSessionErrorV1::InvalidPose,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MoleculeInsertionAtomV1, MoleculeInsertionV1};

    const SOURCE: &str = "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }
    fn anchor(session: &DocumentSession) -> DocumentObjectIdV1 {
        session
            .document_observation()
            .expect("observation")
            .projection()
            .molecules()[0]
            .atoms()[0]
            .id()
            .expect("direct atom selector")
            .clone()
    }

    fn molecule() -> MoleculeInsertionV1 {
        MoleculeInsertionV1::new(
            vec![
                MoleculeInsertionAtomV1::new(
                    "O",
                    Point3V1::new(120.0, 0.0, 0.0).expect("finite point"),
                    None,
                    None,
                    None,
                )
                .expect("valid atom"),
            ],
            Vec::new(),
        )
        .expect("valid molecule")
    }

    #[test]
    fn attached_c6_prepares_without_mutation_and_commits_one_complete_transition() {
        let mut session = DocumentSession::load(SOURCE).expect("source loads");
        let before = session.snapshot().expect("before");
        let mut pending = session
            .prepare_attach_cyclohexane_v1(
                fence(&session),
                anchor(&session),
                AttachedCyclohexaneReleaseV1::new(40.0, 0.0).expect("pose"),
            )
            .expect("prepare");
        assert_eq!(session.snapshot().expect("prepare snapshot"), before);
        let preview = *pending.preview_vertices().expect("live preview");
        let result = session
            .commit_attach_cyclohexane_v1(&mut pending)
            .expect("commit");
        let after = result.observation().snapshot();
        assert_eq!(after.revision(), before.revision() + 1);
        assert_eq!(result.observation().projection().molecules().len(), 1);
        let molecule = &result.observation().projection().molecules()[0];
        assert_eq!(molecule.atoms().len(), 6);
        assert_eq!(molecule.bonds().len(), 6);
        assert_eq!(molecule.atoms()[1].position(), preview[1]);
        assert_eq!(
            molecule
                .bonds()
                .iter()
                .filter(|bond| bond.start().source_id() == Some("a")
                    || bond.end().source_id() == Some("a"))
                .count(),
            2
        );
        assert!(matches!(
            session.commit_attach_cyclohexane_v1(&mut pending),
            Err(AttachedCyclohexaneSessionErrorV1::Retired)
        ));
    }

    #[test]
    fn retired_foreign_stale_and_ineligible_capabilities_preserve_authoritative_state() {
        let mut owner = DocumentSession::load(SOURCE).expect("source loads");
        let mut foreign = DocumentSession::load(SOURCE).expect("source loads");
        let owner_before = owner.snapshot().expect("snapshot");
        let foreign_before = foreign.snapshot().expect("snapshot");
        let mut pending = owner
            .prepare_attach_cyclohexane_v1(
                fence(&owner),
                anchor(&owner),
                AttachedCyclohexaneReleaseV1::new(40.0, 0.0).expect("pose"),
            )
            .expect("prepare");
        assert!(matches!(
            foreign.commit_attach_cyclohexane_v1(&mut pending),
            Err(AttachedCyclohexaneSessionErrorV1::ForeignSession)
        ));
        assert_eq!(
            foreign.snapshot().expect("foreign unchanged"),
            foreign_before
        );
        owner
            .retire_attach_cyclohexane_v1(&mut pending)
            .expect("retire");
        assert_eq!(owner.snapshot().expect("owner unchanged"), owner_before);
        assert!(matches!(
            owner.commit_attach_cyclohexane_v1(&mut pending),
            Err(AttachedCyclohexaneSessionErrorV1::Retired)
        ));
        assert!(matches!(
            owner.prepare_attach_cyclohexane_v1(
                fence(&owner),
                anchor(&owner),
                AttachedCyclohexaneReleaseV1::new(0.0, 0.0).expect("finite pose")
            ),
            Err(AttachedCyclohexaneSessionErrorV1::InvalidPose)
        ));
        assert_eq!(owner.snapshot().expect("refusal unchanged"), owner_before);

        let mut fresh = owner
            .prepare_attach_cyclohexane_v1(
                fence(&owner),
                anchor(&owner),
                AttachedCyclohexaneReleaseV1::new(40.0, 0.0).expect("pose"),
            )
            .expect("fresh prepare retains deferred IDs");
        owner
            .commit_attach_cyclohexane_v1(&mut fresh)
            .expect("fresh commit");
        let committed = owner.snapshot().expect("committed snapshot");
        assert!(committed.cdml().contains("ferrum-atom-v1-0"));
        assert!(committed.cdml().contains("ferrum-bond-v1-0"));
    }

    #[test]
    fn allocation_failure_and_later_transition_leave_prepared_c6_nonmutating() {
        let mut session = DocumentSession::load(SOURCE).expect("source loads");
        let before = session.snapshot().expect("snapshot");
        session.set_next_generated_atom_sequence_for_test(None);
        assert!(matches!(
            session.prepare_attach_cyclohexane_v1(
                fence(&session),
                anchor(&session),
                AttachedCyclohexaneReleaseV1::new(40.0, 0.0).expect("pose"),
            ),
            Err(AttachedCyclohexaneSessionErrorV1::SessionConflict)
        ));
        assert_eq!(
            session.snapshot().expect("allocation refusal snapshot"),
            before
        );
        session.set_next_generated_atom_sequence_for_test(Some(0));
        let mut pending = session
            .prepare_attach_cyclohexane_v1(
                fence(&session),
                anchor(&session),
                AttachedCyclohexaneReleaseV1::new(40.0, 0.0).expect("pose"),
            )
            .expect("prepare before independent change");
        let revision = before.revision();
        let mut unrelated = session
            .prepare_create_molecule_batch_v1(revision, &[molecule()])
            .expect("independent candidate");
        session
            .commit_create_molecule_batch_v1(revision, &mut unrelated)
            .expect("independent transition");
        let after_transition = session.snapshot().expect("transition snapshot");
        assert!(matches!(
            session.commit_attach_cyclohexane_v1(&mut pending),
            Err(AttachedCyclohexaneSessionErrorV1::StaleRevision)
        ));
        assert_eq!(
            session.snapshot().expect("stale refusal snapshot"),
            after_transition
        );
    }
}

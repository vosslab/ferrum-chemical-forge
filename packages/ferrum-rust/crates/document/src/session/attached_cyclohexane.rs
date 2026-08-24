//! One deferred-ID, shared-anchor cyclohexane document transition.

use thiserror::Error;

use super::{
    AdmittedSessionTransitionRefusalV1, DocumentFenceV1, DocumentObjectIdV1, DocumentSession,
    PersistentId, PreparedSessionTransitionV1, RevisionState, SessionDocumentObservationV1,
    SessionOperationResultV1,
};
use crate::{
    AuthoringCapabilityIssuerV1, Point3V1,
    attached_cyclohexane_v1::{
        AttachedCyclohexaneAnchorV1, AttachedCyclohexaneErrorV1, AttachedCyclohexaneIncidentBondV1,
        AttachedCyclohexaneReleaseV1, attached_cyclohexane_candidate_v1,
    },
};
use ferrum_render::{DocumentRenderContentV1, DocumentRenderOutcomeV1, MoleculeRenderPlan};

/// Opaque one-use prepared shared-anchor C6 transition.
pub struct PendingAttachedCyclohexaneV1 {
    session_issuer: AuthoringCapabilityIssuerV1,
    fence: DocumentFenceV1,
    transition: PreparedSessionTransitionV1,
    molecule_source_order: u32,
    render_plan: ferrum_render::DocumentRenderPlanV1,
}

impl std::fmt::Debug for PendingAttachedCyclohexaneV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingAttachedCyclohexaneV1")
            .field("revision", &self.fence.revision())
            .field("is_resolved", &self.transition.is_consumed_v1())
            .finish()
    }
}

impl PendingAttachedCyclohexaneV1 {
    /// Return the renderer-issued molecule plan while this candidate remains live.
    #[must_use]
    pub fn render_plan_v1(&self) -> Option<&MoleculeRenderPlan> {
        if self.transition.is_consumed_v1() {
            return None;
        }
        self.render_plan.outcomes().iter().find_map(|outcome| {
            let DocumentRenderOutcomeV1::Root(root) = outcome else {
                return None;
            };
            if root.source_order() != self.molecule_source_order {
                return None;
            }
            let DocumentRenderContentV1::Molecule(plan) = root.content() else {
                return None;
            };
            Some(plan)
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
    #[error("cyclohexane attachment candidate could not be rendered completely")]
    RendererAdmission,
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

        let ((atom_ids, bond_ids), effects) = self
            .reserve_generated_ids_for_transition_v1(|mut sequences, indexed| {
                let mut atom_ids = Vec::with_capacity(5);
                let mut bond_ids = Vec::with_capacity(6);
                for _ in 0..5 {
                    let (identifier, next) = sequences.reserve_atom(indexed)?;
                    atom_ids.push(identifier);
                    sequences = next;
                }
                for _ in 0..6 {
                    let (identifier, next) = sequences.reserve_bond(indexed)?;
                    bond_ids.push(identifier);
                    sequences = next;
                }
                Ok(((atom_ids, bond_ids), sequences))
            })
            .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        let atom_ids: [PersistentId; 5] = atom_ids.try_into().expect("fixed C6 atom reservation");
        let bond_ids: [PersistentId; 6] = bond_ids.try_into().expect("fixed C6 bond reservation");
        let document = self
            .current_document_v1()
            .with_attach_cyclohexane_v1(
                &resolved.molecule_id,
                &resolved.anchor_id,
                &atom_ids,
                &bond_ids,
                &candidate,
            )
            .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        let revision = self
            .next_revision_v1()
            .ok_or(AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        let state = RevisionState::from_document(revision, document)
            .map_err(|_| AttachedCyclohexaneSessionErrorV1::SessionConflict)?;
        let transition = self
            .prepare_changed_session_transition_v1(fence.revision(), fence.digest(), state, effects)
            .map_err(map_prepare_error)?;
        let render_plan = transition
            .metadata_v1()
            .expect("live transition metadata")
            .renderer_plan()
            .expect("changed C6 transition has a renderer plan")
            .clone();
        Ok(PendingAttachedCyclohexaneV1 {
            session_issuer: self.authoring_capability_issuer.clone(),
            fence,
            transition,
            molecule_source_order: resolved.molecule_source_order,
            render_plan,
        })
    }

    /// Commit one prepared C6 candidate as one history transition.
    pub fn commit_attach_cyclohexane_v1(
        &mut self,
        pending: &mut PendingAttachedCyclohexaneV1,
    ) -> Result<SessionOperationResultV1, AttachedCyclohexaneSessionErrorV1> {
        if pending.transition.is_consumed_v1() {
            return Err(AttachedCyclohexaneSessionErrorV1::Retired);
        }
        if !pending
            .session_issuer
            .same_issuer(&self.authoring_capability_issuer)
        {
            return Err(AttachedCyclohexaneSessionErrorV1::ForeignSession);
        }
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(map_commit_error)
    }

    /// Retire one preview without requiring that its document is still current.
    pub fn retire_attach_cyclohexane_v1(
        &mut self,
        pending: &mut PendingAttachedCyclohexaneV1,
    ) -> Result<(), AttachedCyclohexaneSessionErrorV1> {
        if !pending
            .session_issuer
            .same_issuer(&self.authoring_capability_issuer)
        {
            return Err(AttachedCyclohexaneSessionErrorV1::ForeignSession);
        }
        if pending.transition.is_consumed_v1() {
            return Err(AttachedCyclohexaneSessionErrorV1::Retired);
        }
        self.retire_session_operation_transition_v1(&mut pending.transition)
            .map_err(map_commit_error)
    }
}

fn map_prepare_error(error: super::DocumentSessionError) -> AttachedCyclohexaneSessionErrorV1 {
    match error {
        super::DocumentSessionError::RendererAdmission => {
            AttachedCyclohexaneSessionErrorV1::RendererAdmission
        }
        _ => AttachedCyclohexaneSessionErrorV1::SessionConflict,
    }
}

fn map_commit_error(
    error: AdmittedSessionTransitionRefusalV1,
) -> AttachedCyclohexaneSessionErrorV1 {
    match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            AttachedCyclohexaneSessionErrorV1::ForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Replayed => AttachedCyclohexaneSessionErrorV1::Retired,
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            AttachedCyclohexaneSessionErrorV1::StaleRevision
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            AttachedCyclohexaneSessionErrorV1::RendererAdmission
        }
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability
        | AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            AttachedCyclohexaneSessionErrorV1::SessionConflict
        }
    }
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), AttachedCyclohexaneSessionErrorV1> {
    if session.current_revision_v1() != fence.revision() {
        return Err(AttachedCyclohexaneSessionErrorV1::StaleRevision);
    }
    if session.current_digest_v1() != fence.digest() {
        return Err(AttachedCyclohexaneSessionErrorV1::StaleDigest);
    }
    Ok(())
}

struct ResolvedAnchorV1 {
    molecule_id: PersistentId,
    molecule_source_order: u32,
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
        molecule_source_order: molecule.source_order(),
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

    const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";

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
        let preview = pending.render_plan_v1().expect("renderer-admitted preview");
        assert!(!preview.batches().is_empty());
        let result = session
            .commit_attach_cyclohexane_v1(&mut pending)
            .expect("commit");
        let after = result.observation().snapshot();
        assert_eq!(after.revision(), before.revision() + 1);
        assert_eq!(result.observation().projection().molecules().len(), 1);
        let molecule = &result.observation().projection().molecules()[0];
        assert_eq!(molecule.atoms().len(), 6);
        assert_eq!(molecule.bonds().len(), 6);
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

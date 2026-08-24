use super::*;
use crate::direct_haworth_insertion_v1::{DirectHaworthInsertionV1, validate_candidate};
use crate::{
    AuthoringCapabilityClaimV1, CreateHaworthMoleculeV1, SessionOperation,
    SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
};
use ferrum_domain::haworth::DirectGlycosidicHaworthAuthoringReceiptV1;

impl DocumentSession {
    /// Re-authenticate one exact durable direct-Haworth profile without mutation.
    pub fn observe_direct_glycosidic_haworth_v1(
        &self,
        expected_revision: u64,
        molecule: &DocumentObjectIdV1,
    ) -> Result<super::super::ReobservedDirectHaworthV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let extracted = super::super::direct_haworth_reobservation_v1::extract(
            self.current_document_v1(),
            molecule,
        )?;
        let observation = self.document_observation()?;
        super::super::direct_haworth_reobservation_v1::finish(extracted, observation)
            .map_err(DocumentSessionError::DirectHaworthReobservation)
    }

    /// Resolve parsed direct-Haworth source facts into generic transition authority.
    pub fn resolve_direct_haworth_transition_v1(
        &self,
        expected_revision: u64,
        receipt: DirectGlycosidicHaworthAuthoringReceiptV1,
        anchor: Point3V1,
    ) -> Result<SessionOperationTransitionRequestV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        Ok(SessionOperationTransitionRequestV1::new(
            expected_revision,
            SessionOperation::V1(SessionOperationV1::CreateHaworthMoleculeV1(
                CreateHaworthMoleculeV1::direct_glycosidic(receipt, anchor),
            )),
            TransitionAuthorizationV1::authoring_capability(self.issue_authoring_capability_v1()),
        ))
    }

    pub(in crate::session) fn prepare_create_haworth_molecule_transition_v1(
        &mut self,
        expected_revision: u64,
        request: CreateHaworthMoleculeV1,
        authorization_claim: AuthoringCapabilityClaimV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let (candidate, molecule, atoms, bonds, effects) = match request {
            CreateHaworthMoleculeV1::DirectGlycosidic { receipt, anchor } => {
                let insertion = DirectHaworthInsertionV1::from_receipt(&receipt, anchor)
                    .map_err(DocumentSessionError::Operation)?;
                let (identities, effects) =
                    self.reserve_generated_ids_for_transition_v1(|sequences, indexed| {
                        sequences.reserve_molecule(
                            indexed,
                            insertion.atom_count(),
                            insertion.bond_count(),
                        )
                    })?;
                let candidate = self
                    .current_document_v1()
                    .with_insert_direct_haworth(
                        &identities.molecule,
                        &identities.atoms,
                        &identities.bonds,
                        &insertion,
                    )
                    .map_err(SessionOperationError::Candidate)?;
                validate_candidate(
                    &candidate,
                    &identities.molecule,
                    &identities.atoms,
                    &identities.bonds,
                    &insertion,
                )
                .map_err(DocumentSessionError::Operation)?;
                (
                    candidate,
                    identities.molecule,
                    identities.atoms,
                    identities.bonds,
                    effects,
                )
            }
            CreateHaworthMoleculeV1::StandaloneDGlucose { recipe, anchor } => {
                let receipt =
                    ferrum_domain::haworth::standalone_d_glucose_haworth_recipe_v1(recipe)
                        .map_err(|error| {
                            DocumentSessionError::Operation(
                                SessionOperationError::InvalidStandaloneHaworthInsertion(
                                    error.to_string(),
                                ),
                            )
                        })?;
                let insertion = crate::standalone_haworth_insertion_v1::StandaloneHaworthInsertionV1::from_receipt(&receipt, anchor)
                    .map_err(DocumentSessionError::Operation)?;
                let (identities, effects) =
                    self.reserve_generated_ids_for_transition_v1(|sequences, indexed| {
                        sequences.reserve_molecule(
                            indexed,
                            insertion.atom_count(),
                            insertion.bond_count(),
                        )
                    })?;
                let candidate = self
                    .current_document_v1()
                    .with_insert_standalone_haworth(
                        &identities.molecule,
                        &identities.atoms,
                        &identities.bonds,
                        &insertion,
                    )
                    .map_err(SessionOperationError::Candidate)?;
                (
                    candidate,
                    identities.molecule,
                    identities.atoms,
                    identities.bonds,
                    effects,
                )
            }
        };
        let revision = self
            .next_revision_v1()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let mut transition = self
            .prepare_changed_session_transition_with_authorized_molecule_insertion_outcome_v1(
                expected_revision,
                self.current_digest_v1(),
                state,
                effects,
                molecule,
                atoms.clone(),
                bonds.clone(),
                authorization_claim,
            )?;
        install_haworth_overlay(&mut transition, &atoms, &bonds)?;
        Ok(transition)
    }
}

fn install_haworth_overlay(
    transition: &mut PreparedSessionTransitionV1,
    atoms: &[PersistentId],
    bonds: &[PersistentId],
) -> Result<(), DocumentSessionError> {
    let targets = atoms
        .iter()
        .map(|atom| ferrum_render::AcceptedRenderOverlayTargetV1::atom(atom.as_str()))
        .chain(
            bonds
                .iter()
                .map(|bond| ferrum_render::AcceptedRenderOverlayTargetV1::bond(bond.as_str())),
        )
        .collect::<Vec<_>>();
    let request = ferrum_render::AcceptedRenderOverlayRequestV1::new(targets)
        .map_err(|_| DocumentSessionError::RendererAdmission)?;
    let overlay = transition
        .renderer_precommit_overlay_v1(&request)
        .map_err(|_| DocumentSessionError::RendererAdmission)?;
    transition
        .install_precommit_overlay_v1(overlay)
        .map_err(|_| DocumentSessionError::RendererAdmission)
}

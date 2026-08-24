//! Revision-bound standalone D-glucose Haworth insertion.

use ferrum_domain::haworth::{
    StandaloneDGlucoseHaworthRecipeV1, standalone_d_glucose_haworth_recipe_v1,
};

use super::*;
use crate::standalone_haworth_insertion_v1::StandaloneHaworthInsertionV1;

/// One opaque, one-use native standalone Haworth candidate.
#[derive(Debug)]
pub struct PendingStandaloneHaworthV1 {
    molecule_identifier: PersistentId,
    atom_identifiers: Vec<PersistentId>,
    bond_identifiers: Vec<PersistentId>,
    transition: PreparedSessionTransitionV1,
    render_plan: ferrum_render::DocumentRenderPlanV1,
}
impl PendingStandaloneHaworthV1 {
    #[must_use]
    pub fn molecule_identifier(&self) -> &PersistentId {
        &self.molecule_identifier
    }
    #[must_use]
    pub fn atom_identifiers(&self) -> &[PersistentId] {
        &self.atom_identifiers
    }
    #[must_use]
    pub fn bond_identifiers(&self) -> &[PersistentId] {
        &self.bond_identifiers
    }
    /// Return the renderable candidate observation without exposing candidate XML.
    #[must_use]
    pub fn candidate_observation_v1(&self) -> Option<SessionDocumentObservationV1> {
        self.transition
            .metadata_v1()
            .map(|metadata| metadata.observation().clone())
    }
    /// Return the immutable renderer-issued plan for the exact pending candidate.
    #[must_use]
    pub fn render_plan_v1(&self) -> &ferrum_render::DocumentRenderPlanV1 {
        &self.render_plan
    }
}

impl DocumentSession {
    /// Prepare one closed detached D-glucose Haworth recipe at one resolved anchor.
    pub fn prepare_create_standalone_haworth_v1(
        &mut self,
        expected_revision: u64,
        recipe: StandaloneDGlucoseHaworthRecipeV1,
        anchor: Point3V1,
    ) -> Result<PendingStandaloneHaworthV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let receipt = standalone_d_glucose_haworth_recipe_v1(recipe).map_err(|error| {
            DocumentSessionError::Operation(
                SessionOperationError::InvalidStandaloneHaworthInsertion(error.to_string()),
            )
        })?;
        let insertion = StandaloneHaworthInsertionV1::from_receipt(&receipt, anchor)
            .map_err(DocumentSessionError::Operation)?;
        let (identities, effects) =
            self.reserve_generated_ids_for_transition_v1(|sequences, indexed| {
                sequences.reserve_molecule(indexed, insertion.atom_count(), insertion.bond_count())
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
        let revision = self
            .next_revision_v1()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let token_effect = self.issue_transition_provisional_token_effect_v1()?;
        let effects = Self::compose_transition_effects_v1(effects, token_effect).map_err(|_| {
            DocumentSessionError::Operation(
                SessionOperationError::InvalidStandaloneHaworthInsertion(
                    "conflicting deferred Haworth transition effects".to_owned(),
                ),
            )
        })?;
        let transition = self
            .prepare_changed_session_transition_v1(
                expected_revision,
                self.current_digest_v1(),
                candidate,
                effects,
            )
            .map_err(|error| {
                DocumentSessionError::Operation(
                    SessionOperationError::InvalidStandaloneHaworthInsertion(format!("{error:?}")),
                )
            })?;
        let render_plan = transition
            .metadata_v1()
            .expect("live transition metadata")
            .renderer_plan()
            .expect("changed Haworth transition has a renderer plan")
            .clone();
        Ok(PendingStandaloneHaworthV1 {
            molecule_identifier: identities.molecule,
            atom_identifiers: identities.atoms,
            bond_identifiers: identities.bonds,
            transition,
            render_plan,
        })
    }
    /// Commit a current standalone Haworth candidate exactly once.
    pub fn commit_create_standalone_haworth_v1(
        &mut self,
        _expected_revision: u64,
        pending: &mut PendingStandaloneHaworthV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(|error| {
                DocumentSessionError::Operation(
                    SessionOperationError::InvalidStandaloneHaworthInsertion(format!("{error:?}")),
                )
            })
    }
}

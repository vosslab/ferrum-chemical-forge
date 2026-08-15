//! Revision-bound standalone D-glucose Haworth insertion.

use ferrum_domain::haworth::{
    StandaloneDGlucoseHaworthRecipeV1, standalone_d_glucose_haworth_recipe_v1,
};

use super::*;
use crate::standalone_haworth_insertion_v1::StandaloneHaworthInsertionV1;

/// One opaque, one-use native standalone Haworth candidate.
pub struct PendingStandaloneHaworthV1 {
    pub(super) pending: PendingCreateMolecule,
    recipe: StandaloneDGlucoseHaworthRecipeV1,
    vertices: Vec<Point3V1>,
    edges: Vec<[usize; 2]>,
}
impl PendingStandaloneHaworthV1 {
    #[must_use]
    pub fn molecule_identifier(&self) -> &PersistentId {
        self.pending.molecule_identifier()
    }
    #[must_use]
    pub fn atom_identifiers(&self) -> &[PersistentId] {
        self.pending.atom_identifiers()
    }
    #[must_use]
    pub fn bond_identifiers(&self) -> &[PersistentId] {
        self.pending.bond_identifiers()
    }
    #[must_use]
    pub const fn recipe(&self) -> StandaloneDGlucoseHaworthRecipeV1 {
        self.recipe
    }
    #[must_use]
    pub fn vertices(&self) -> &[Point3V1] {
        &self.vertices
    }
    #[must_use]
    pub fn edges(&self) -> &[[usize; 2]] {
        &self.edges
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
        let vertices =
            insertion_atoms(&receipt, anchor).map_err(DocumentSessionError::Operation)?;
        let edges = receipt
            .bonds()
            .iter()
            .map(|bond| [bond.start(), bond.end()])
            .collect();
        let (identities, generated_ids) = self.generated_ids.reserve_molecule(
            self.history.current().document().indexed(),
            insertion.atom_count(),
            insertion.bond_count(),
        )?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_standalone_haworth(
                &identities.molecule,
                &identities.atoms,
                &identities.bonds,
                &insertion,
            )
            .map_err(SessionOperationError::Candidate)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        SessionDocumentObservationV1::from_state(
            candidate.document(),
            candidate.snapshot(!self.saved_baseline.is_current(&candidate)),
        )
        .map_err(DocumentSessionError::Projection)?;
        let token = prepared::issue_prepared_token(self.history.current_mut().document_mut())?;
        self.generated_ids = generated_ids;
        Ok(PendingStandaloneHaworthV1 {
            pending: PendingCreateMolecule {
                revision: expected_revision,
                token,
                molecule_identifier: identities.molecule,
                atom_identifiers: identities.atoms,
                bond_identifiers: identities.bonds,
                candidate: Some(candidate),
            },
            recipe,
            vertices,
            edges,
        })
    }
    /// Commit a current standalone Haworth candidate exactly once.
    pub fn commit_create_standalone_haworth_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingStandaloneHaworthV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_create_molecule(expected_revision, &mut pending.pending)
    }
}

fn insertion_atoms(
    receipt: &ferrum_domain::haworth::StandaloneDGlucoseHaworthReceiptV1,
    anchor: Point3V1,
) -> Result<Vec<Point3V1>, SessionOperationError> {
    receipt
        .atoms()
        .iter()
        .map(|fact| {
            let local = fact.local();
            Point3V1::new(local.x + anchor.x(), local.y + anchor.y(), anchor.z()).map_err(|_| {
                SessionOperationError::InvalidStandaloneHaworthInsertion(
                    "translated coordinate is not finite".to_owned(),
                )
            })
        })
        .collect()
}

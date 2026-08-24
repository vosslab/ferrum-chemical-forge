//! Document-owned lowering for closed catalog molecule placement.

use super::{
    DocumentSession, DocumentSessionError, PersistentId, Point3V1, RevisionState,
    SessionOperationError,
};
use crate::standalone_haworth_insertion_v1::StandaloneHaworthInsertionV1;
use crate::{CatalogMoleculePlacementContentV1, CatalogMoleculePlacementV1, CatalogPlacementKeyV1};
use ferrum_domain::haworth::standalone_d_glucose_haworth_recipe_v1;

/// Private facts staged with one generic catalog transition until it commits.
#[derive(Debug)]
pub(super) struct CatalogMoleculePlacementOutcomeStagingV1 {
    pub(super) catalog_key: CatalogPlacementKeyV1,
    pub(super) anchor: super::PresentationGesturePoint2V1,
    pub(super) root_identifier: PersistentId,
}

impl DocumentSession {
    /// Lower one closed catalog request into the sole generic prepared transition.
    pub(super) fn prepare_place_catalog_molecule_v1(
        &mut self,
        expected_revision: u64,
        request: CatalogMoleculePlacementV1,
    ) -> Result<super::PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let source_digest = self.current_digest_v1();
        let catalog_key = request.catalog_key().clone();
        let anchor = request.anchor();
        match request.content() {
            CatalogMoleculePlacementContentV1::Molecule(molecule) => {
                let (identities, effects) =
                    self.reserve_generated_ids_for_transition_v1(|ids, indexed| {
                        ids.reserve_molecule(
                            indexed,
                            molecule.atoms().len(),
                            molecule.bonds().len(),
                        )
                    })?;
                let candidate = self
                    .current_document_v1()
                    .with_insert_molecule(
                        &identities.molecule,
                        &identities.atoms,
                        &identities.bonds,
                        molecule,
                    )
                    .map_err(SessionOperationError::Candidate)?;
                let state = catalog_revision_state(self, candidate)?;
                self.prepare_changed_session_transition_with_catalog_outcome_v1(
                    expected_revision,
                    source_digest,
                    state,
                    effects,
                    CatalogMoleculePlacementOutcomeStagingV1 {
                        catalog_key,
                        anchor,
                        root_identifier: identities.molecule,
                    },
                )
            }
            CatalogMoleculePlacementContentV1::StandaloneHaworth(recipe) => {
                let receipt = standalone_d_glucose_haworth_recipe_v1(*recipe).map_err(|error| {
                    DocumentSessionError::Operation(SessionOperationError::InvalidCatalogPlacement(
                        error.to_string(),
                    ))
                })?;
                let insertion = StandaloneHaworthInsertionV1::from_receipt(
                    &receipt,
                    Point3V1::new(anchor.x(), anchor.y(), 0.0).map_err(|error| {
                        DocumentSessionError::Operation(
                            SessionOperationError::InvalidCatalogPlacement(error.to_string()),
                        )
                    })?,
                )
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
                let state = catalog_revision_state(self, candidate)?;
                let token_effect = self.issue_transition_provisional_token_effect_v1()?;
                let effects =
                    Self::compose_transition_effects_v1(effects, token_effect).map_err(|_| {
                        DocumentSessionError::Operation(
                            SessionOperationError::InvalidCatalogPlacement(
                                "conflicting deferred catalog transition effects".to_owned(),
                            ),
                        )
                    })?;
                self.prepare_changed_session_transition_with_catalog_outcome_v1(
                    expected_revision,
                    source_digest,
                    state,
                    effects,
                    CatalogMoleculePlacementOutcomeStagingV1 {
                        catalog_key,
                        anchor,
                        root_identifier: identities.molecule,
                    },
                )
            }
        }
    }
}

fn catalog_revision_state(
    session: &DocumentSession,
    candidate: crate::TypedDocument,
) -> Result<RevisionState, DocumentSessionError> {
    let revision = session
        .next_revision_v1()
        .ok_or(DocumentSessionError::RevisionExhausted)?;
    RevisionState::from_document(revision, candidate).map_err(DocumentSessionError::Load)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        MoleculeInsertionAtomV1, MoleculeInsertionV1, SessionOperation, SessionOperationOutcomeV1,
        SessionOperationV1, TransitionAuthorizationV1,
    };

    #[test]
    fn generic_catalog_transition_returns_committed_intent_and_root() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let anchor = super::super::PresentationGesturePoint2V1::new(12.0, -4.0).expect("anchor");
        let atom = MoleculeInsertionAtomV1::new(
            "C",
            Point3V1::new(anchor.x(), anchor.y(), 0.0).expect("point"),
            None,
            None,
            None,
        )
        .expect("atom");
        let request = CatalogMoleculePlacementV1::new(
            CatalogPlacementKeyV1::new("system/test/carbon".to_owned()).expect("key"),
            anchor,
            CatalogMoleculePlacementContentV1::Molecule(
                MoleculeInsertionV1::new(vec![atom], Vec::new()).expect("molecule"),
            ),
        );
        let mut prepared = session
            .prepare_session_operation_transition_v1(
                crate::SessionOperationTransitionRequestV1::new(
                    0,
                    SessionOperation::V1(SessionOperationV1::PlaceCatalogMoleculeV1(request)),
                    TransitionAuthorizationV1::None,
                ),
            )
            .expect("generic preparation");
        let result = session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("generic commit");
        let SessionOperationOutcomeV1::CatalogMoleculePlacementV1(outcome) = result.outcome()
        else {
            panic!("catalog outcome");
        };
        assert_eq!(outcome.catalog_key().as_str(), "system/test/carbon");
        assert_eq!(outcome.anchor(), anchor);
        assert_eq!(outcome.root_identifier().as_str(), "ferrum-molecule-v1-0");
    }
}

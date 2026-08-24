use super::*;
use crate::direct_haworth_insertion_v1::{
    CommittedDirectHaworthBondFactV1, DirectHaworthInsertionV1, PreparedDirectHaworthReceiptV1,
    validate_candidate,
};

/// Render-neutral immutable receipt from an accepted direct-Haworth commit.
#[derive(Clone, Debug, PartialEq)]
pub struct CommittedDirectHaworthV1 {
    revision: u64,
    digest: [u8; 32],
    molecule_identifier: PersistentId,
    atom_identifiers: Vec<PersistentId>,
    bond_identifiers: Vec<PersistentId>,
    bond_facts: Vec<CommittedDirectHaworthBondFactV1>,
    authored_depiction: ferrum_domain::haworth::AuthoredDirectGlycosidicHaworthDepictionV1,
}

impl CommittedDirectHaworthV1 {
    fn from_prepared(
        prepared: PreparedDirectHaworthReceiptV1,
        revision: u64,
        digest: [u8; 32],
    ) -> Self {
        let (
            molecule_identifier,
            atom_identifiers,
            bond_identifiers,
            bond_facts,
            authored_depiction,
        ) = prepared.into_parts();
        Self {
            revision,
            digest,
            molecule_identifier,
            atom_identifiers,
            bond_identifiers,
            bond_facts,
            authored_depiction,
        }
    }
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
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
    #[must_use]
    pub fn bond_facts(&self) -> &[CommittedDirectHaworthBondFactV1] {
        &self.bond_facts
    }
    #[must_use]
    pub fn authored_depiction(
        &self,
    ) -> &ferrum_domain::haworth::AuthoredDirectGlycosidicHaworthDepictionV1 {
        &self.authored_depiction
    }
}

/// Exact accepted operation result bound to the committed direct-Haworth receipt.
#[derive(Clone, Debug, PartialEq)]
pub struct CommittedDirectHaworthResultV1 {
    operation: SessionOperationResultV1,
    receipt: CommittedDirectHaworthV1,
}

impl CommittedDirectHaworthResultV1 {
    fn new(operation: SessionOperationResultV1, receipt: CommittedDirectHaworthV1) -> Self {
        Self { operation, receipt }
    }
    #[must_use]
    pub fn operation(&self) -> &SessionOperationResultV1 {
        &self.operation
    }
    #[must_use]
    pub fn receipt(&self) -> &CommittedDirectHaworthV1 {
        &self.receipt
    }
}

fn committed_result(
    operation: SessionOperationResultV1,
    prepared: PreparedDirectHaworthReceiptV1,
) -> CommittedDirectHaworthResultV1 {
    let revision = operation.observation().snapshot().revision();
    let digest = *operation.observation().snapshot().digest();
    CommittedDirectHaworthResultV1::new(
        operation,
        CommittedDirectHaworthV1::from_prepared(prepared, revision, digest),
    )
}

/// A one-use closed direct-Haworth candidate.
pub struct PendingDirectHaworthV1 {
    molecule_identifier: PersistentId,
    atom_identifiers: Vec<PersistentId>,
    bond_identifiers: Vec<PersistentId>,
    pub(super) prepared_receipt: Option<PreparedDirectHaworthReceiptV1>,
    transition: PreparedSessionTransitionV1,
    render_plan: ferrum_render::DocumentRenderPlanV1,
}

impl std::fmt::Debug for PendingDirectHaworthV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingDirectHaworthV1")
            .field("molecule_identifier", &self.molecule_identifier)
            .finish()
    }
}
impl PendingDirectHaworthV1 {
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
    /// Return the immutable renderer-issued plan for the exact pending candidate.
    #[must_use]
    pub fn render_plan_v1(&self) -> &ferrum_render::DocumentRenderPlanV1 {
        &self.render_plan
    }
}

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

    /// Prepare one closed direct-Haworth insertion without changing generic insertion APIs.
    pub fn prepare_create_direct_haworth_v1(
        &mut self,
        expected_revision: u64,
        receipt: &ferrum_domain::haworth::DirectGlycosidicHaworthAuthoringReceiptV1,
        anchor: Point3V1,
    ) -> Result<PendingDirectHaworthV1, DocumentSessionError> {
        let insertion = DirectHaworthInsertionV1::from_receipt(receipt, anchor)
            .map_err(DocumentSessionError::Operation)?;
        self.require_current(expected_revision)?;
        let (identities, effects) =
            self.reserve_generated_ids_for_transition_v1(|sequences, indexed| {
                sequences.reserve_molecule(indexed, insertion.atom_count(), insertion.bond_count())
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
        let revision = self
            .next_revision_v1()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let prepared_receipt = insertion
            .prepared_receipt(
                receipt,
                &identities.molecule,
                &identities.atoms,
                &identities.bonds,
                anchor,
            )
            .map_err(DocumentSessionError::Operation)?;
        let token_effect = self
            .issue_transition_provisional_token_effect_v1()
            .map_err(|error| {
                DocumentSessionError::Operation(
                    SessionOperationError::InvalidDirectHaworthInsertion(error.to_string()),
                )
            })?;
        let effects = Self::compose_transition_effects_v1(effects, token_effect).map_err(|_| {
            DocumentSessionError::Operation(SessionOperationError::InvalidDirectHaworthInsertion(
                "conflicting deferred Haworth transition effects".to_owned(),
            ))
        })?;
        let transition = self
            .prepare_changed_session_transition_v1(
                expected_revision,
                self.current_digest_v1(),
                candidate,
                effects,
            )
            .map_err(|error| match error {
                DocumentSessionError::RendererAdmission => DocumentSessionError::Operation(
                    SessionOperationError::InvalidDirectHaworthInsertion(
                        "candidate was refused by renderer admission".to_owned(),
                    ),
                ),
                other => DocumentSessionError::Operation(
                    SessionOperationError::InvalidDirectHaworthInsertion(other.to_string()),
                ),
            })?;
        let render_plan = transition
            .metadata_v1()
            .expect("live transition metadata")
            .renderer_plan()
            .expect("changed Haworth transition has a renderer plan")
            .clone();
        Ok(PendingDirectHaworthV1 {
            molecule_identifier: identities.molecule,
            atom_identifiers: identities.atoms,
            bond_identifiers: identities.bonds,
            prepared_receipt: Some(prepared_receipt),
            transition,
            render_plan,
        })
    }
    /// Commit one closed direct-Haworth candidate and return its exact accepted observation.
    pub fn commit_create_direct_haworth_v1(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingDirectHaworthV1,
    ) -> Result<CommittedDirectHaworthResultV1, DocumentSessionError> {
        let Some(prepared) = pending.prepared_receipt.take() else {
            return Err(DocumentSessionError::PreparedOperationConsumed);
        };
        if let Err(error) = self.require_current(expected_revision) {
            pending.prepared_receipt = Some(prepared);
            return Err(error);
        }
        match self.commit_session_operation_transition_v1(&mut pending.transition) {
            Ok(operation) => Ok(committed_result(operation, prepared)),
            Err(AdmittedSessionTransitionRefusalV1::StaleSnapshot) => {
                pending.prepared_receipt = Some(prepared);
                Err(DocumentSessionError::RevisionConflict {
                    expected: expected_revision,
                    actual: self.current_revision_v1(),
                })
            }
            Err(error) => {
                pending.prepared_receipt = Some(prepared);
                Err(DocumentSessionError::Operation(
                    SessionOperationError::InvalidDirectHaworthInsertion(format!("{error:?}")),
                ))
            }
        }
    }
}

use super::types::*;
use super::*;
use crate::session_operation::{
    AtomCreatedOutcomeV1, BondCreatedOutcomeV1, ReactionCreatedOutcomeV1,
    ReactionDefinitionDeletedOutcomeV1, ReactionMembershipReplacedOutcomeV1,
    ReactionOperationOutcomeStagingV1,
};
use crate::{InterchangeRecordBatchInsertedOutcomeV1, MoleculeInsertedOutcomeV1, PersistentId};

impl DocumentSession {
    /// Read the retained revision state without exposing timeline navigation.
    #[must_use]
    pub(crate) fn current_state_v1(&self) -> &RevisionState {
        self.admitted_history.current()
    }

    /// Read the current typed document without granting mutation authority.
    #[must_use]
    pub(crate) fn current_document_v1(&self) -> &super::super::TypedDocument {
        self.current_state_v1().document()
    }

    /// Read the current indexed document without exposing a mutable timeline.
    #[must_use]
    pub(crate) fn current_index_v1(&self) -> &IndexedDocument {
        self.current_document_v1().indexed()
    }

    /// Read the authoritative current revision.
    #[must_use]
    pub(crate) fn current_revision_v1(&self) -> u64 {
        self.current_state_v1().revision()
    }

    /// Read the authoritative digest for the current state.
    #[must_use]
    pub(crate) fn current_digest_v1(&self) -> [u8; 32] {
        *self.current_state_v1().digest()
    }

    /// Read the next monotonic revision if the sequence has capacity.
    #[must_use]
    pub(crate) fn next_revision_v1(&self) -> Option<u64> {
        self.current_state_v1().next_revision()
    }

    pub(in crate::session) fn has_undo_history_v1(&self) -> bool {
        self.admitted_history.undo_target().is_some()
    }

    pub(in crate::session) fn has_redo_history_v1(&self) -> bool {
        self.admitted_history.redo_target().is_some()
    }

    /// Issue one source-document provisional token as a deferred transition effect.
    ///
    /// The token remains document-owned and is consumed only when the matching
    /// renderer-admitted transition commits. Route modules cannot mutate the
    /// retained document to issue tokens directly.
    pub(in crate::session) fn issue_transition_provisional_token_effect_v1(
        &mut self,
    ) -> Result<SessionTransitionEffectsV1, DocumentSessionError> {
        let token = super::super::prepared::issue_prepared_token(
            self.admitted_history.current_mut().document_mut(),
        )?;
        Ok(SessionTransitionEffectsV1::none().consuming_provisional_token(token))
    }

    /// Combine complementary deferred effects for one admitted transition.
    ///
    /// The core owns effect-slot conflict detection so routes cannot silently
    /// overwrite token consumption or generated-ID installation behavior.
    pub(in crate::session) fn compose_transition_effects_v1(
        primary: SessionTransitionEffectsV1,
        extension: SessionTransitionEffectsV1,
    ) -> Result<SessionTransitionEffectsV1, SessionTransitionEffectCompositionRefusalV1> {
        primary.compose(extension)
    }

    #[cfg(test)]
    pub(in crate::session) fn set_current_revision_for_test_v1(&mut self, revision: u64) {
        self.admitted_history
            .set_current_revision_for_test(revision);
    }

    /// Retire one opaque transition without changing document state or effects.
    ///
    /// Retirement is issuer-bound and one-use. It invalidates the prospective
    /// state, renderer proof, and deferred effects without installing them.
    /// A foreign session cannot invalidate the owner's pending transition.
    pub fn retire_session_operation_transition_v1(
        &mut self,
        prepared: &mut PreparedSessionTransitionV1,
    ) -> Result<(), AdmittedSessionTransitionRefusalV1> {
        if !prepared
            .issuer
            .same_issuer(&self.authoring_capability_issuer_v1())
        {
            return Err(AdmittedSessionTransitionRefusalV1::ForeignSession);
        }
        if prepared.is_consumed_v1() {
            return Err(AdmittedSessionTransitionRefusalV1::Replayed);
        }
        prepared.consume_terminal_authorization_v1();
        match &mut prepared.kind {
            PreparedSessionTransitionKindV1::NoChange { result } => {
                debug_assert!(result.is_none());
            }
            PreparedSessionTransitionKindV1::Changed(changed) => {
                let _ = changed.state.take();
                let _ = changed.result.take();
            }
        }
        Ok(())
    }

    /// Prepare one no-change generic transition with committed operation facts.
    pub(in crate::session) fn prepare_no_change_session_transition_with_outcome_v1(
        &self,
        expected_revision: u64,
        outcome: SessionOperationOutcomeV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let current = self.admitted_history.current();
        Ok(PreparedSessionTransitionV1 {
            issuer: self.authoring_capability_issuer_v1(),
            source_revision: current.revision(),
            source_digest: *current.digest(),
            kind: PreparedSessionTransitionKindV1::NoChange {
                result: Some(self.operation_result()?.with_outcome(outcome)),
            },
        })
    }

    /// Prepare one renderer-admitted typed session operation without changing history.
    pub fn prepare_session_operation_transition_v1(
        &mut self,
        request: SessionOperationTransitionRequestV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.prepare_session_operation_transition_request_v1(request)
    }

    /// Consume one opaque generic request through the document-owned transition lifecycle.
    fn prepare_session_operation_transition_request_v1(
        &mut self,
        SessionOperationTransitionRequestV1 {
            expected_revision,
            operation,
            authorization,
        }: SessionOperationTransitionRequestV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        let authorization_claim = match (&operation, authorization) {
            (
                SessionOperation::V1(
                    SessionOperationV1::CreateDirectBondV1(_)
                    | SessionOperationV1::CreateAtomV1(_)
                    | SessionOperationV1::CreateBondV1(_)
                    | SessionOperationV1::CreateHaworthMoleculeV1(_)
                    | SessionOperationV1::CreateCurvedTerminalArrowV1(_)
                    | SessionOperationV1::CreateCurvedEquilibriumArrowV1(_)
                    | SessionOperationV1::CreatePresentationPathV1(_)
                    | SessionOperationV1::CreatePresentationVectorV1(_)
                    | SessionOperationV1::CreatePresentationRootV1(_)
                    | SessionOperationV1::CreateReactionV1(_)
                    | SessionOperationV1::ReplaceReactionMembersV1(_)
                    | SessionOperationV1::DeleteReactionV1(_)
                    | SessionOperationV1::TranslateTopLevelRootsV1(_),
                ),
                TransitionAuthorizationV1::AuthoringCapability(capability),
            ) => Some(
                capability
                    .claim_for_commit(&self.authoring_capability_issuer_v1())
                    .map_err(map_authorization_access_error_v1)?,
            ),
            (
                SessionOperation::V1(
                    SessionOperationV1::CreateDirectBondV1(_)
                    | SessionOperationV1::CreateAtomV1(_)
                    | SessionOperationV1::CreateBondV1(_)
                    | SessionOperationV1::CreateHaworthMoleculeV1(_)
                    | SessionOperationV1::CreateCurvedTerminalArrowV1(_)
                    | SessionOperationV1::CreateCurvedEquilibriumArrowV1(_)
                    | SessionOperationV1::CreatePresentationPathV1(_)
                    | SessionOperationV1::CreatePresentationVectorV1(_)
                    | SessionOperationV1::CreatePresentationRootV1(_)
                    | SessionOperationV1::CreateReactionV1(_)
                    | SessionOperationV1::ReplaceReactionMembersV1(_)
                    | SessionOperationV1::DeleteReactionV1(_)
                    | SessionOperationV1::TranslateTopLevelRootsV1(_),
                ),
                _,
            ) => {
                return Err(DocumentSessionError::TransitionAuthorization(
                    TransitionAuthorizationRefusalV1::AuthoringCapabilityRequired,
                ));
            }
            (_, TransitionAuthorizationV1::None) => None,
            (_, TransitionAuthorizationV1::AuthoringCapability(_)) => {
                return Err(DocumentSessionError::TransitionAuthorization(
                    TransitionAuthorizationRefusalV1::UnexpectedAuthoringCapability,
                ));
            }
        };
        if let SessionOperation::V1(SessionOperationV1::CreateDirectBondV1(request)) = operation {
            self.require_current(request.fence().revision())?;
            return self
                .prepare_create_direct_bond_v1(
                    request,
                    authorization_claim.expect("direct-bond authorization requirement was checked"),
                )
                .map_err(Into::into);
        }
        if let SessionOperation::V1(SessionOperationV1::CreateAtomV1(request)) = operation {
            return self.prepare_create_atom_transition_v1(
                expected_revision,
                request,
                authorization_claim.expect("atom authorization requirement was checked"),
            );
        }
        if let SessionOperation::V1(SessionOperationV1::CreateBondV1(request)) = operation {
            return self.prepare_create_bond_transition_v1(
                expected_revision,
                request,
                authorization_claim.expect("bond authorization requirement was checked"),
            );
        }
        if let SessionOperation::V1(SessionOperationV1::CreateHaworthMoleculeV1(request)) =
            operation
        {
            return self.prepare_create_haworth_molecule_transition_v1(
                expected_revision,
                request,
                authorization_claim.expect("Haworth authorization requirement was checked"),
            );
        }
        if let SessionOperation::V1(SessionOperationV1::InsertMoleculeV1(molecule)) = &operation {
            return self.prepare_insert_molecule_transition_v1(expected_revision, molecule);
        }
        if let SessionOperation::V1(SessionOperationV1::InsertInterchangeRecordBatchV1(batch)) =
            &operation
        {
            return self
                .prepare_insert_interchange_record_batch_transition_v1(expected_revision, batch);
        }
        if let SessionOperation::V1(SessionOperationV1::MaterializeMoleculeHydrogensV1(request)) =
            operation
        {
            return self
                .prepare_materialize_molecule_hydrogens_transition_v1(expected_revision, request);
        }
        let presentation = match operation {
            SessionOperation::V1(SessionOperationV1::CreateCurvedTerminalArrowV1(ref request)) => {
                Some((
                    crate::PresentationCreateRequestV1::CurvedTerminalArrow {
                        kind: request.kind(),
                        start: request.start(),
                        control: request.control(),
                        end: request.end(),
                    },
                    CreatedPresentationRootKindV1::CurvedTerminalArrow,
                ))
            }
            SessionOperation::V1(SessionOperationV1::CreateCurvedEquilibriumArrowV1(
                ref request,
            )) => Some((
                crate::PresentationCreateRequestV1::CurvedEquilibriumArrow {
                    start: request.start(),
                    control: request.control(),
                    end: request.end(),
                },
                CreatedPresentationRootKindV1::CurvedEquilibriumArrow,
            )),
            SessionOperation::V1(SessionOperationV1::CreatePresentationPathV1(ref request)) => {
                Some((
                    crate::PresentationCreateRequestV1::Path {
                        path: request.path().clone(),
                        appearance: request.appearance().clone(),
                    },
                    CreatedPresentationRootKindV1::Path,
                ))
            }
            SessionOperation::V1(SessionOperationV1::CreatePresentationVectorV1(ref request)) => {
                Some((
                    crate::PresentationCreateRequestV1::Vector {
                        kind: request.kind(),
                        start: request.start(),
                        end: request.end(),
                        appearance: request.appearance().clone(),
                    },
                    CreatedPresentationRootKindV1::Vector,
                ))
            }
            SessionOperation::V1(SessionOperationV1::CreatePresentationRootV1(ref request)) => {
                match request {
                    crate::CreatePresentationRootV1::StraightNormalArrow { start, end, style } => {
                        Some((
                            crate::PresentationCreateRequestV1::StraightNormalArrow {
                                start: *start,
                                end: *end,
                                style: *style,
                            },
                            CreatedPresentationRootKindV1::StraightNormalArrow,
                        ))
                    }
                    crate::CreatePresentationRootV1::StraightEquilibriumArrow { start, end } => {
                        Some((
                            crate::PresentationCreateRequestV1::StraightEquilibriumArrow {
                                start: *start,
                                end: *end,
                            },
                            CreatedPresentationRootKindV1::StraightEquilibriumArrow,
                        ))
                    }
                    crate::CreatePresentationRootV1::StandardPlus { anchor } => Some((
                        crate::PresentationCreateRequestV1::StandardPlus { anchor: *anchor },
                        CreatedPresentationRootKindV1::Plus,
                    )),
                }
            }
            _ => None,
        };
        if let Some((request, kind)) = presentation {
            return self.prepare_create_presentation_transition_v1(
                expected_revision,
                request,
                kind,
                authorization_claim.expect("presentation authorization requirement was checked"),
            );
        }
        if let SessionOperation::V1(SessionOperationV1::PlaceCatalogMoleculeV1(request)) = operation
        {
            return self.prepare_place_catalog_molecule_v1(expected_revision, request);
        }
        self.require_current(expected_revision)?;
        let current = self.admitted_history.current();
        let source_revision = current.revision();
        let source_digest = *current.digest();
        let issuer = self.authoring_capability_issuer_v1();
        let (candidate, reaction_outcome) = operation.prepare_with_outcome_v1(
            current.document(),
            source_revision,
            &source_digest,
        )?;
        let outcome = stage_reaction_operation_outcome(reaction_outcome);
        match candidate {
            Candidate::NoChange => Ok(PreparedSessionTransitionV1 {
                issuer,
                source_revision,
                source_digest,
                kind: PreparedSessionTransitionKindV1::NoChange {
                    result: Some(self.operation_result()?),
                },
            }),
            Candidate::Changed(document) => {
                let revision = current
                    .next_revision()
                    .ok_or(DocumentSessionError::RevisionExhausted)?;
                let state = RevisionState::from_document(revision, *document)
                    .map_err(DocumentSessionError::Load)?;
                self.prepare_changed_session_transition_with_commit_v1(
                    source_revision,
                    source_digest,
                    state,
                    SessionTransitionEffectsV1::none(),
                    ChangedTransitionCommitV1::Append,
                    outcome,
                    authorization_claim,
                )
            }
        }
    }

    /// Verify and atomically redeem one opaque renderer-admitted session transition.
    pub fn commit_session_operation_transition_v1(
        &mut self,
        prepared: &mut PreparedSessionTransitionV1,
    ) -> Result<SessionOperationResultV1, AdmittedSessionTransitionRefusalV1> {
        if !prepared
            .issuer
            .same_issuer(&self.authoring_capability_issuer_v1())
        {
            return Err(AdmittedSessionTransitionRefusalV1::ForeignSession);
        }
        if prepared.is_consumed_v1() {
            return Err(AdmittedSessionTransitionRefusalV1::Replayed);
        }
        if self.admitted_history.current().revision() != prepared.source_revision
            || *self.admitted_history.current().digest() != prepared.source_digest
        {
            return Err(AdmittedSessionTransitionRefusalV1::StaleSnapshot);
        }
        match &mut prepared.kind {
            PreparedSessionTransitionKindV1::NoChange { result } => Ok(result
                .take()
                .expect("the consumed check established the no-change result invariant")),
            PreparedSessionTransitionKindV1::Changed(changed) => {
                changed
                    .renderer_admission
                    .verify(&changed.observation)
                    .map_err(|_| AdmittedSessionTransitionRefusalV1::RendererAdmission)?;
                changed
                    .effects
                    .verify_provisional_token(self)
                    .map_err(|_| AdmittedSessionTransitionRefusalV1::ProvisionalCapability)?;
                // No fallible work follows authorization settlement. The
                // immutable transition was fully revalidated above, so the
                // remaining state moves are infallible finalization.
                changed.consume_authorization_claim_v1();
                match changed.commit {
                    ChangedTransitionCommitV1::Append => {
                        changed.effects.consume_provisional_token(self);
                        let state = changed
                            .state
                            .take()
                            .expect("the consumed check established the changed state invariant");
                        let result = changed
                            .result
                            .take()
                            .expect("the consumed check established the changed result invariant");
                        self.admitted_history.append(state);
                        changed.effects.install_generated_ids(self);
                        Ok(result.with_outcome(take_operation_outcome(changed)))
                    }
                    ChangedTransitionCommitV1::Navigate(direction) => {
                        changed.effects.consume_provisional_token(self);
                        let state = changed
                            .state
                            .take()
                            .expect("the consumed check established the changed state invariant");
                        let result = changed
                            .result
                            .take()
                            .expect("the consumed check established the changed result invariant");
                        match direction {
                            HistoryNavigationDirectionV1::Undo => self.admitted_history.move_undo(),
                            HistoryNavigationDirectionV1::Redo => self.admitted_history.move_redo(),
                        }
                        self.admitted_history.replace_current(state);
                        changed.effects.install_generated_ids(self);
                        Ok(result.with_outcome(take_operation_outcome(changed)))
                    }
                }
            }
        }
    }

    /// Execute one typed operation through the renderer-admitted transition boundary.
    pub fn apply_document_operation_v1(
        &mut self,
        expected_revision: u64,
        operation: SessionOperation,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        // Closed primitive operations are declarative and session-neutral.  The
        // session therefore issues their one-shot authority at this public
        // boundary, while gesture routes continue to supply their own receipt.
        let authorization = match &operation {
            SessionOperation::V1(
                SessionOperationV1::CreateAtomV1(_) | SessionOperationV1::CreateBondV1(_),
            ) => TransitionAuthorizationV1::authoring_capability(
                self.authoring_capability_issuer_v1().issue(),
            ),
            _ => TransitionAuthorizationV1::None,
        };
        let mut prepared = self.prepare_session_operation_transition_v1(
            SessionOperationTransitionRequestV1::new(
                expected_revision,
                operation,
                authorization,
            ),
        )?;
        self.commit_session_operation_transition_v1(&mut prepared)
            .map_err(|refusal| self.map_admitted_transition_refusal_v1(&prepared, refusal))
    }

    /// Build one changed renderer-admitted transition from a session-owned candidate.
    pub(in crate::session) fn prepare_changed_session_transition_v1(
        &mut self,
        source_revision: u64,
        source_digest: [u8; 32],
        state: RevisionState,
        effects: SessionTransitionEffectsV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.prepare_changed_session_transition_with_commit_v1(
            source_revision,
            source_digest,
            state,
            effects,
            ChangedTransitionCommitV1::Append,
            SessionOperationOutcomeStagingV1::Standard,
            None,
        )
    }

    pub(in crate::session) fn prepare_changed_session_transition_with_direct_bond_outcome_v1(
        &mut self,
        source_revision: u64,
        source_digest: [u8; 32],
        state: RevisionState,
        effects: SessionTransitionEffectsV1,
        outcome: super::super::direct_bond::DirectBondOutcomeStagingV1,
        authorization_claim: AuthoringCapabilityClaimV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.prepare_changed_session_transition_with_commit_v1(
            source_revision,
            source_digest,
            state,
            effects,
            ChangedTransitionCommitV1::Append,
            SessionOperationOutcomeStagingV1::DirectBondV1(outcome),
            Some(authorization_claim),
        )
    }

    pub(in crate::session) fn prepare_changed_session_transition_with_presentation_outcome_v1(
        &mut self,
        source_revision: u64,
        source_digest: [u8; 32],
        state: RevisionState,
        effects: SessionTransitionEffectsV1,
        root: crate::PresentationRootSelectorV1,
        kind: CreatedPresentationRootKindV1,
        authorization_claim: AuthoringCapabilityClaimV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.prepare_changed_session_transition_with_commit_v1(
            source_revision,
            source_digest,
            state,
            effects,
            ChangedTransitionCommitV1::Append,
            SessionOperationOutcomeStagingV1::CreatedPresentationRootV1(root, kind),
            Some(authorization_claim),
        )
    }

    pub(in crate::session) fn prepare_changed_session_transition_with_catalog_outcome_v1(
        &mut self,
        source_revision: u64,
        source_digest: [u8; 32],
        state: RevisionState,
        effects: SessionTransitionEffectsV1,
        outcome: super::super::catalog_molecule_placement::CatalogMoleculePlacementOutcomeStagingV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.prepare_changed_session_transition_with_commit_v1(
            source_revision,
            source_digest,
            state,
            effects,
            ChangedTransitionCommitV1::Append,
            SessionOperationOutcomeStagingV1::CatalogMoleculePlacementV1(outcome),
            None,
        )
    }

    pub(in crate::session) fn prepare_changed_session_transition_with_molecule_insertion_outcome_v1(
        &mut self,
        source_revision: u64,
        source_digest: [u8; 32],
        state: RevisionState,
        effects: SessionTransitionEffectsV1,
        molecule_identifier: PersistentId,
        atom_identifiers: Vec<PersistentId>,
        bond_identifiers: Vec<PersistentId>,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.prepare_changed_session_transition_with_commit_v1(
            source_revision,
            source_digest,
            state,
            effects,
            ChangedTransitionCommitV1::Append,
            SessionOperationOutcomeStagingV1::MoleculeInsertedV1 {
                molecule_identifier,
                atom_identifiers,
                bond_identifiers,
            },
            None,
        )
    }

    pub(in crate::session) fn prepare_changed_session_transition_with_authorized_molecule_insertion_outcome_v1(
        &mut self,
        source_revision: u64,
        source_digest: [u8; 32],
        state: RevisionState,
        effects: SessionTransitionEffectsV1,
        molecule_identifier: PersistentId,
        atom_identifiers: Vec<PersistentId>,
        bond_identifiers: Vec<PersistentId>,
        authorization_claim: AuthoringCapabilityClaimV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.prepare_changed_session_transition_with_commit_v1(
            source_revision,
            source_digest,
            state,
            effects,
            ChangedTransitionCommitV1::Append,
            SessionOperationOutcomeStagingV1::MoleculeInsertedV1 {
                molecule_identifier,
                atom_identifiers,
                bond_identifiers,
            },
            Some(authorization_claim),
        )
    }

    pub(in crate::session) fn prepare_changed_session_transition_with_interchange_batch_outcome_v1(
        &mut self,
        source_revision: u64,
        source_digest: [u8; 32],
        state: RevisionState,
        effects: SessionTransitionEffectsV1,
        records: Vec<(PersistentId, Vec<PersistentId>, Vec<PersistentId>)>,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.prepare_changed_session_transition_with_commit_v1(
            source_revision,
            source_digest,
            state,
            effects,
            ChangedTransitionCommitV1::Append,
            SessionOperationOutcomeStagingV1::InterchangeRecordBatchInsertedV1(records),
            None,
        )
    }

    /// Reserve generated IDs for a candidate without changing this live session.
    ///
    /// The returned effects install the resulting sequence only after the core
    /// commits the prepared transition. Routes must retain those effects rather
    /// than assigning `generated_ids` after an append.
    pub(in crate::session) fn reserve_generated_ids_for_transition_v1<T>(
        &self,
        reserve: impl FnOnce(
            super::super::GeneratedIdSequences,
            &IndexedDocument,
        )
            -> Result<(T, super::super::GeneratedIdSequences), SessionOperationError>,
    ) -> Result<(T, SessionTransitionEffectsV1), SessionOperationError> {
        let (value, next_generated_ids) = reserve(
            self.generated_ids,
            self.admitted_history.current().document().indexed(),
        )?;
        Ok((
            value,
            SessionTransitionEffectsV1::none().installing_generated_ids(next_generated_ids),
        ))
    }

    pub(in crate::session) fn prepare_history_navigation_transition_v1(
        &mut self,
        expected_revision: u64,
        direction: HistoryNavigationDirectionV1,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let source = match direction {
            HistoryNavigationDirectionV1::Undo => self.admitted_history.undo_target(),
            HistoryNavigationDirectionV1::Redo => self.admitted_history.redo_target(),
        }
        .ok_or(DocumentSessionError::HistoryUnavailable)?
        .canonical_cdml()
        .to_owned();
        let (source_revision, source_digest, revision) = {
            let current = self.admitted_history.current();
            (
                current.revision(),
                *current.digest(),
                current
                    .next_revision()
                    .ok_or(DocumentSessionError::RevisionExhausted)?,
            )
        };
        let document =
            super::super::TypedDocument::parse(&source).map_err(DocumentSessionError::Load)?;
        let state =
            RevisionState::from_document(revision, document).map_err(DocumentSessionError::Load)?;
        self.prepare_changed_session_transition_with_commit_v1(
            source_revision,
            source_digest,
            state,
            SessionTransitionEffectsV1::none(),
            ChangedTransitionCommitV1::Navigate(direction),
            SessionOperationOutcomeStagingV1::Standard,
            None,
        )
    }

    pub(in crate::session) fn execute_history_navigation_transition_v1(
        &mut self,
        expected_revision: u64,
        direction: HistoryNavigationDirectionV1,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        let mut prepared =
            self.prepare_history_navigation_transition_v1(expected_revision, direction)?;
        self.commit_session_operation_transition_v1(&mut prepared)
            .map_err(|refusal| self.map_admitted_transition_refusal_v1(&prepared, refusal))
    }

    pub(in crate::session) fn prepare_changed_session_transition_with_commit_v1(
        &mut self,
        source_revision: u64,
        source_digest: [u8; 32],
        state: RevisionState,
        effects: SessionTransitionEffectsV1,
        commit: ChangedTransitionCommitV1,
        outcome: SessionOperationOutcomeStagingV1,
        authorization_claim: Option<AuthoringCapabilityClaimV1>,
    ) -> Result<PreparedSessionTransitionV1, DocumentSessionError> {
        self.require_current(source_revision)?;
        if *self.admitted_history.current().digest() != source_digest {
            return Err(DocumentSessionError::RevisionConflict {
                expected: source_revision,
                actual: self.admitted_history.current().revision(),
            });
        }
        if state.revision()
            != self
                .admitted_history
                .current()
                .next_revision()
                .ok_or(DocumentSessionError::RevisionExhausted)?
        {
            return Err(DocumentSessionError::RevisionConflict {
                expected: source_revision,
                actual: self.admitted_history.current().revision(),
            });
        }
        let snapshot = state.snapshot(!self.saved_baseline.is_current(&state));
        let observation = SessionDocumentObservationV1::from_snapshot(snapshot)
            .map_err(DocumentSessionError::Projection)?;
        let renderer_admission = RendererAdmittedPendingV1::admit(self, &observation)
            .map_err(|_| DocumentSessionError::RendererAdmission)?;
        effects.verify_provisional_token(self)?;
        if commit == ChangedTransitionCommitV1::Append {
            self.admitted_history.ensure_append_slot().map_err(|_| {
                DocumentSessionError::Operation(SessionOperationError::HistoryResourceExhausted)
            })?;
        }
        Ok(PreparedSessionTransitionV1 {
            issuer: self.authoring_capability_issuer_v1(),
            source_revision,
            source_digest,
            kind: PreparedSessionTransitionKindV1::Changed(PreparedChangedSessionTransitionV1 {
                state: Some(state),
                result: Some(SessionOperationResultV1::new(observation.clone())),
                observation,
                renderer_admission,
                effects,
                commit,
                outcome: Some(outcome),
                precommit_overlay: None,
                authorization_claim,
            }),
        })
    }

    fn map_admitted_transition_refusal_v1(
        &self,
        prepared: &PreparedSessionTransitionV1,
        refusal: AdmittedSessionTransitionRefusalV1,
    ) -> DocumentSessionError {
        match refusal {
            AdmittedSessionTransitionRefusalV1::ForeignSession => {
                DocumentSessionError::PreparedOperationForeignSession
            }
            AdmittedSessionTransitionRefusalV1::Replayed => {
                DocumentSessionError::PreparedOperationConsumed
            }
            AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
                DocumentSessionError::RevisionConflict {
                    expected: prepared.source_revision,
                    actual: self.admitted_history.current().revision(),
                }
            }
            AdmittedSessionTransitionRefusalV1::RendererAdmission => {
                DocumentSessionError::RendererAdmission
            }
            AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
                DocumentSessionError::PreparedOperationConsumed
            }
            AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
                DocumentSessionError::Operation(SessionOperationError::HistoryResourceExhausted)
            }
        }
    }
}

fn map_authorization_access_error_v1(
    error: AuthoringCapabilityAccessErrorV1,
) -> DocumentSessionError {
    let refusal = match error {
        AuthoringCapabilityAccessErrorV1::ForeignSession => {
            TransitionAuthorizationRefusalV1::ForeignSession
        }
        AuthoringCapabilityAccessErrorV1::Replayed => TransitionAuthorizationRefusalV1::Replayed,
    };
    DocumentSessionError::TransitionAuthorization(refusal)
}

fn take_operation_outcome(
    changed: &mut PreparedChangedSessionTransitionV1,
) -> SessionOperationOutcomeV1 {
    match changed
        .outcome
        .take()
        .expect("the consumed check established the changed outcome invariant")
    {
        SessionOperationOutcomeStagingV1::Standard => SessionOperationOutcomeV1::Standard,
        SessionOperationOutcomeStagingV1::DirectBondV1(staging) => {
            SessionOperationOutcomeV1::DirectBondV1(DirectBondOperationOutcomeV1::new(
                staging.bond,
                staging.end_atom,
                staging.second_created_atom,
                staging.created_new_atom,
                staging.created_new_molecule,
            ))
        }
        SessionOperationOutcomeStagingV1::AtomCreatedV1(identifier) => {
            SessionOperationOutcomeV1::AtomCreatedV1(AtomCreatedOutcomeV1::new(identifier))
        }
        SessionOperationOutcomeStagingV1::BondCreatedV1(identifier) => {
            SessionOperationOutcomeV1::BondCreatedV1(BondCreatedOutcomeV1::new(identifier))
        }
        SessionOperationOutcomeStagingV1::MoleculeHydrogensMaterializedV1(result) => {
            SessionOperationOutcomeV1::MoleculeHydrogensMaterializedV1(result)
        }
        SessionOperationOutcomeStagingV1::MoleculeInsertedV1 {
            molecule_identifier,
            atom_identifiers,
            bond_identifiers,
        } => SessionOperationOutcomeV1::MoleculeInsertedV1(MoleculeInsertedOutcomeV1::new(
            molecule_identifier,
            atom_identifiers,
            bond_identifiers,
        )),
        SessionOperationOutcomeStagingV1::InterchangeRecordBatchInsertedV1(records) => {
            SessionOperationOutcomeV1::InterchangeRecordBatchInsertedV1(
                InterchangeRecordBatchInsertedOutcomeV1::new(
                    records
                        .into_iter()
                        .map(
                            |(molecule_identifier, atom_identifiers, bond_identifiers)| {
                                MoleculeInsertedOutcomeV1::new(
                                    molecule_identifier,
                                    atom_identifiers,
                                    bond_identifiers,
                                )
                            },
                        )
                        .collect(),
                ),
            )
        }
        SessionOperationOutcomeStagingV1::CatalogMoleculePlacementV1(staging) => {
            SessionOperationOutcomeV1::CatalogMoleculePlacementV1(
                CatalogMoleculePlacementOutcomeV1::new(
                    staging.catalog_key,
                    staging.anchor,
                    staging.root_identifier,
                ),
            )
        }
        SessionOperationOutcomeStagingV1::CreatedPresentationRootV1(root, kind) => {
            SessionOperationOutcomeV1::CreatedPresentationRootV1(
                CreatedPresentationRootOutcomeV1::new(root, kind),
            )
        }
        SessionOperationOutcomeStagingV1::ReactionCreatedV1(reaction_id) => {
            SessionOperationOutcomeV1::ReactionCreatedV1(ReactionCreatedOutcomeV1::new(reaction_id))
        }
        SessionOperationOutcomeStagingV1::ReactionMembershipReplacedV1(reaction_id) => {
            SessionOperationOutcomeV1::ReactionMembershipReplacedV1(
                ReactionMembershipReplacedOutcomeV1::new(reaction_id),
            )
        }
        SessionOperationOutcomeStagingV1::ReactionDefinitionDeletedV1(reaction_id) => {
            SessionOperationOutcomeV1::ReactionDefinitionDeletedV1(
                ReactionDefinitionDeletedOutcomeV1::new(reaction_id),
            )
        }
    }
}

fn stage_reaction_operation_outcome(
    outcome: Option<ReactionOperationOutcomeStagingV1>,
) -> SessionOperationOutcomeStagingV1 {
    match outcome {
        None => SessionOperationOutcomeStagingV1::Standard,
        Some(ReactionOperationOutcomeStagingV1::ReactionCreatedV1(reaction_id)) => {
            SessionOperationOutcomeStagingV1::ReactionCreatedV1(reaction_id)
        }
        Some(ReactionOperationOutcomeStagingV1::ReactionMembershipReplacedV1(reaction_id)) => {
            SessionOperationOutcomeStagingV1::ReactionMembershipReplacedV1(reaction_id)
        }
        Some(ReactionOperationOutcomeStagingV1::ReactionDefinitionDeletedV1(reaction_id)) => {
            SessionOperationOutcomeStagingV1::ReactionDefinitionDeletedV1(reaction_id)
        }
    }
}

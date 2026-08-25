use std::fmt;

use crate::PersistentId;

use super::*;

/// Closed refusal set for a document-owned admitted transition.
///
/// The pending value remains owned by its source session after every refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum AdmittedSessionTransitionRefusalV1 {
    /// The opaque transition was prepared by another live session.
    ForeignSession,
    /// The opaque transition was already redeemed or retired.
    Replayed,
    /// The source revision or digest no longer identifies the current state.
    StaleSnapshot,
    /// The renderer no longer accepts the exact prospective observation.
    RendererAdmission,
    /// A deferred session capability can no longer be redeemed.
    ProvisionalCapability,
    /// The session could not reserve bounded history storage for this transition.
    HistoryCapacity,
}

/// Closed authorization supplied beside one semantic session operation.
#[derive(Debug)]
pub enum TransitionAuthorizationV1 {
    /// Explicitly authorize an operation that requires no authoring receipt.
    None,
    /// Move one opaque authoring receipt into generic transition preparation.
    AuthoringCapability(AuthoringCapabilityV1),
}

impl TransitionAuthorizationV1 {
    #[must_use]
    pub const fn none() -> Self {
        Self::None
    }

    #[must_use]
    pub fn authoring_capability(capability: AuthoringCapabilityV1) -> Self {
        Self::AuthoringCapability(capability)
    }
}

/// Opaque input for one generic renderer-admitted session transition.
///
/// The request moves a semantic operation and its authorization together into
/// [`DocumentSession::prepare_session_operation_transition_v1`]. Its fields
/// remain private so only the generic transition lifecycle can inspect the
/// revision fence, operation facts, or capability-bearing authorization.
#[derive(Debug)]
pub struct SessionOperationTransitionRequestV1 {
    pub(super) expected_revision: u64,
    pub(super) operation: SessionOperation,
    pub(super) authorization: TransitionAuthorizationV1,
}

impl SessionOperationTransitionRequestV1 {
    /// Assemble the complete input consumed by generic transition preparation.
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        operation: SessionOperation,
        authorization: TransitionAuthorizationV1,
    ) -> Self {
        Self {
            expected_revision,
            operation,
            authorization,
        }
    }
}

/// Closed refusal set for semantic-operation authorization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionAuthorizationRefusalV1 {
    /// The semantic operation requires an authoring capability.
    AuthoringCapabilityRequired,
    /// The semantic operation accepts only explicit absent authorization.
    UnexpectedAuthoringCapability,
    /// The supplied capability belongs to another live document session.
    ForeignSession,
    /// The supplied capability is already claimed or terminally consumed.
    Replayed,
}

/// Opaque prepared session transition.
///
/// It deliberately exposes neither candidate CDML, mutable state, renderer proof,
/// pending identity, nor deferred session effects. Callers can only redeem it through
/// [`DocumentSession::commit_session_operation_transition_v1`] or retire it through
/// [`DocumentSession::retire_session_operation_transition_v1`].
pub struct PreparedSessionTransitionV1 {
    pub(super) issuer: AuthoringCapabilityIssuerV1,
    pub(super) source_revision: u64,
    pub(super) source_digest: [u8; 32],
    pub(super) kind: PreparedSessionTransitionKindV1,
}

impl fmt::Debug for PreparedSessionTransitionV1 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lifecycle = if self.is_consumed_v1() {
            "consumed"
        } else {
            "pending"
        };
        formatter
            .debug_struct("PreparedSessionTransitionV1")
            .field("lifecycle", &lifecycle)
            .finish()
    }
}

/// Immutable display facts copied from one live prepared session transition.
///
/// This value is deliberately detached from the transition that produced it.
/// It carries only inert precommit paint content. It cannot redeem, retire,
/// expose prospective document identifiers, or otherwise affect the source
/// transition.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedSessionTransitionPresentationV1 {
    pub(super) precommit_overlay: Option<ferrum_render::DocumentPrecommitOverlayV1>,
}

impl PreparedSessionTransitionPresentationV1 {
    /// Return the copied inert precommit paint subset when the transition has one.
    #[must_use]
    pub const fn precommit_overlay(&self) -> Option<&ferrum_render::DocumentPrecommitOverlayV1> {
        self.precommit_overlay.as_ref()
    }
}

/// Refusal from extracting a display value from a prepared transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparedSessionTransitionPresentationRefusalV1 {
    /// The source transition was already redeemed or explicitly retired.
    Retired,
}

#[derive(Debug)]
pub(super) enum PreparedSessionTransitionKindV1 {
    NoChange {
        result: Option<SessionOperationResultV1>,
    },
    Changed(PreparedChangedSessionTransitionV1),
}

#[derive(Debug)]
pub(super) struct PreparedChangedSessionTransitionV1 {
    pub(super) state: Option<RevisionState>,
    pub(super) observation: SessionDocumentObservationV1,
    pub(super) renderer_admission: RendererAdmittedPendingV1,
    pub(super) effects: SessionTransitionEffectsV1,
    pub(super) commit: ChangedTransitionCommitV1,
    pub(super) result: Option<SessionOperationResultV1>,
    pub(super) outcome: Option<SessionOperationOutcomeStagingV1>,
    pub(super) precommit_overlay: Option<ferrum_render::DocumentPrecommitOverlayV1>,
    pub(super) authorization_claim: Option<AuthoringCapabilityClaimV1>,
}

impl PreparedChangedSessionTransitionV1 {
    pub(super) fn consume_authorization_claim_v1(&mut self) {
        if let Some(claim) = self.authorization_claim.take() {
            claim.consume();
        }
    }
}

#[derive(Debug)]
pub(in crate::session) enum SessionOperationOutcomeStagingV1 {
    Standard,
    DirectBondV1(super::super::direct_bond::DirectBondOutcomeStagingV1),
    AtomCreatedV1(PersistentId),
    BondCreatedV1(PersistentId),
    MoleculeHydrogensMaterializedV1(crate::DocumentMoleculeHydrogenMaterializationResultV1),
    CompactGroupMaterializedV1(crate::DocumentCompactGroupMaterializationResultV1),
    MoleculeInsertedV1 {
        molecule_identifier: PersistentId,
        atom_identifiers: Vec<PersistentId>,
        bond_identifiers: Vec<PersistentId>,
    },
    InterchangeRecordBatchInsertedV1(Vec<(PersistentId, Vec<PersistentId>, Vec<PersistentId>)>),
    CatalogMoleculePlacementV1(
        super::super::catalog_molecule_placement::CatalogMoleculePlacementOutcomeStagingV1,
    ),
    CreatedPresentationRootV1(
        crate::PresentationRootSelectorV1,
        CreatedPresentationRootKindV1,
    ),
    ReactionCreatedV1(String),
    ReactionMembershipReplacedV1(String),
    ReactionDefinitionDeletedV1(String),
}

impl PreparedSessionTransitionV1 {
    /// Consume any retained authorization and discard the transition exactly once.
    ///
    /// The caller reaches this only after ownership and all fallible commit
    /// preflight have succeeded, or while terminally retiring the transition.
    pub(super) fn consume_terminal_authorization_v1(&mut self) {
        match &mut self.kind {
            PreparedSessionTransitionKindV1::NoChange { result } => {
                let _ = result.take();
            }
            PreparedSessionTransitionKindV1::Changed(changed) => {
                changed.consume_authorization_claim_v1();
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::session) enum ChangedTransitionCommitV1 {
    Append,
    Navigate(HistoryNavigationDirectionV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::session) enum HistoryNavigationDirectionV1 {
    Undo,
    Redo,
}

/// Deferred document-owned effects redeemed with one changed transition.
///
/// Routes add effects here instead of mutating the live document during preparation.
#[derive(Debug, Default)]
pub(in crate::session) struct SessionTransitionEffectsV1 {
    provisional_token: Option<ProvisionalToken>,
    next_generated_ids: Option<super::super::GeneratedIdSequences>,
}

/// Refusal from combining two document-owned deferred effect sets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::session) enum SessionTransitionEffectCompositionRefusalV1 {
    /// Both effect sets attempted to consume a provisional token.
    DuplicateProvisionalToken,
    /// Both effect sets attempted to install generated-ID sequences.
    DuplicateGeneratedIds,
}

impl SessionTransitionEffectsV1 {
    pub(in crate::session) fn none() -> Self {
        Self::default()
    }

    /// Add a capability consumption to this deferred transition.
    #[must_use]
    pub(in crate::session) fn consuming_provisional_token(
        mut self,
        token: ProvisionalToken,
    ) -> Self {
        self.provisional_token = Some(token);
        self
    }

    /// Add installation of one tentative generated-ID sequence set.
    #[must_use]
    pub(in crate::session) fn installing_generated_ids(
        mut self,
        next_generated_ids: super::super::GeneratedIdSequences,
    ) -> Self {
        self.next_generated_ids = Some(next_generated_ids);
        self
    }

    pub(super) fn compose(
        self,
        extension: Self,
    ) -> Result<Self, SessionTransitionEffectCompositionRefusalV1> {
        let provisional_token = match (self.provisional_token, extension.provisional_token) {
            (Some(_), Some(_)) => {
                return Err(SessionTransitionEffectCompositionRefusalV1::DuplicateProvisionalToken);
            }
            (primary, extension) => primary.or(extension),
        };
        let next_generated_ids = match (self.next_generated_ids, extension.next_generated_ids) {
            (Some(_), Some(_)) => {
                return Err(SessionTransitionEffectCompositionRefusalV1::DuplicateGeneratedIds);
            }
            (primary, extension) => primary.or(extension),
        };
        Ok(Self {
            provisional_token,
            next_generated_ids,
        })
    }

    pub(super) fn verify_provisional_token(
        &self,
        session: &DocumentSession,
    ) -> Result<(), DocumentSessionError> {
        if let Some(token) = &self.provisional_token {
            session
                .admitted_history
                .current()
                .document()
                .verify_provisional_token(token)
                .map_err(super::super::prepared::map_prepared_token_error)?;
        }
        Ok(())
    }

    pub(super) fn consume_provisional_token(&self, session: &mut DocumentSession) {
        if let Some(token) = &self.provisional_token {
            session
                .admitted_history
                .current_mut()
                .document_mut()
                .consume_provisional_token(token)
                .expect(
                    "the immediately preceding capability verification established this invariant",
                );
        }
    }

    pub(super) fn install_generated_ids(&self, session: &mut DocumentSession) {
        if let Some(next_generated_ids) = self.next_generated_ids {
            session.generated_ids = next_generated_ids;
        }
    }
}

impl PreparedSessionTransitionV1 {
    /// Return whether this prepared transition was already redeemed or retired.
    #[must_use]
    pub fn is_consumed_v1(&self) -> bool {
        match &self.kind {
            PreparedSessionTransitionKindV1::NoChange { result } => result.is_none(),
            PreparedSessionTransitionKindV1::Changed(changed) => changed.state.is_none(),
        }
    }

    /// Copy the immutable display facts from one live prepared transition.
    ///
    /// This read-only extraction leaves the transition redeemable by its owner.
    pub fn presentation_v1(
        &self,
    ) -> Result<
        PreparedSessionTransitionPresentationV1,
        PreparedSessionTransitionPresentationRefusalV1,
    > {
        let precommit_overlay = match &self.kind {
            PreparedSessionTransitionKindV1::NoChange { result: Some(_) } => None,
            PreparedSessionTransitionKindV1::Changed(PreparedChangedSessionTransitionV1 {
                state: Some(_),
                precommit_overlay,
                ..
            }) => precommit_overlay.as_ref(),
            _ => return Err(PreparedSessionTransitionPresentationRefusalV1::Retired),
        };
        Ok(PreparedSessionTransitionPresentationV1 {
            precommit_overlay: precommit_overlay.cloned(),
        })
    }

    pub(crate) fn install_precommit_overlay_v1(
        &mut self,
        overlay: ferrum_render::DocumentPrecommitOverlayV1,
    ) -> Result<(), PreparedSessionTransitionPresentationRefusalV1> {
        let PreparedSessionTransitionKindV1::Changed(changed) = &mut self.kind else {
            return Err(PreparedSessionTransitionPresentationRefusalV1::Retired);
        };
        if changed.state.is_none() || changed.result.is_none() {
            return Err(PreparedSessionTransitionPresentationRefusalV1::Retired);
        }
        changed.precommit_overlay = Some(overlay);
        Ok(())
    }

    pub(crate) fn renderer_precommit_overlay_v1(
        &self,
        request: &ferrum_render::AcceptedRenderOverlayRequestV1,
    ) -> Result<
        ferrum_render::DocumentPrecommitOverlayV1,
        PreparedSessionTransitionPresentationRefusalV1,
    > {
        let PreparedSessionTransitionKindV1::Changed(changed) = &self.kind else {
            return Err(PreparedSessionTransitionPresentationRefusalV1::Retired);
        };
        if changed.state.is_none() || changed.result.is_none() {
            return Err(PreparedSessionTransitionPresentationRefusalV1::Retired);
        }
        changed
            .renderer_admission
            .precommit_overlay_v1(request)
            .map_err(|_| PreparedSessionTransitionPresentationRefusalV1::Retired)
    }
}

impl Drop for PreparedSessionTransitionV1 {
    fn drop(&mut self) {
        self.consume_terminal_authorization_v1();
    }
}

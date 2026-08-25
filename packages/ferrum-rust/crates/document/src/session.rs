//! Immutable CDML snapshots and safe same-directory publication.
//!
//! The session owns the mutable retained CDML tree, optimistic revisions, bounded
//! history, save baseline, and revision-bound prepared operations. Clients receive
//! immutable snapshots and operation envelopes only; no frontend may invent or
//! retain a competing document state.

use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use renderer_admitted_pending_v1::RendererAdmittedPendingV1;

use super::identity_index::ProvisionalToken;
use super::{
    AuthoringCapabilityIssuerV1, BracketInsertionV1, BracketStyleV1, DocumentBondCapacityOutcomeV1,
    DocumentBondPresentationV1, DocumentObjectIdV1, MoleculeInsertionAtomV1,
    MoleculeInsertionBondV1, MoleculeInsertionV1, PersistentId, Point3V1,
    PreparedStraightenDepictionsV1, ProjectionError, SessionDocumentObservationV1, TypedClass,
    TypedDocument, TypedDocumentError, WavyInsertionV1, XmlSerializationError,
    direct_bond_primitives_v1::{
        DirectBondAdmissionRefusalV1, DirectBondGestureErrorV1, DirectBondPoint2V1,
        DirectBondSnapPolicyV1, DocumentFenceV1,
    },
    generated_ids::GeneratedIdSequences,
    presentation_creation_gesture_v1::{
        PresentationCreationGestureV1, PresentationCreationPreviewV1, PresentationGestureErrorV1,
        PresentationGestureKindV1, PresentationGesturePoint2V1, PresentationGestureSnapPolicyV1,
        PresentationGestureStyleV1,
    },
    publication::{PublicationDurability, publish_snapshot},
    session_operation::{SessionOperation, SessionOperationError, SessionOperationResultV1},
    session_state::{RevisionState, SavedBaseline},
    typed_bond_insertion::BondedAtomInsertion,
};

mod admitted_transition_v1;
#[doc(hidden)]
pub mod attached_compact_group;
#[doc(hidden)]
pub mod attached_cyclohexane;
mod bracket;
mod catalog_molecule_placement;
mod clipboard;
mod clipboard_cut;
mod compact_group_materialization;
mod construction;
mod direct_bond;
mod direct_haworth;
mod explicit_fragment;
mod free_compact_group;
mod gestures;
mod hydrogen_materialization;
mod interchange;
mod linear_form;
mod live_chemical_targets;
mod live_presentation_targets;
mod live_structural_targets;
pub use live_chemical_targets::LiveChemicalPresentationTargetV1;
mod molecule_creation;
mod prepared;
mod presentation_creation;
mod presentation_gesture;
mod primitive_bond;
mod reaction_authoring_command_v1;
mod reaction_member_selection_v1;
mod renderer_admitted_pending_v1;
mod renderer_translation_snap_v1;
mod standalone_haworth;
mod straighten;
mod structural_deletion;
mod text_placement;
mod user_template;
mod wavy;
use admitted_transition_v1::SessionTransitionEffectsV1;
pub use admitted_transition_v1::{
    AdmittedSessionTransitionRefusalV1, PreparedSessionTransitionPresentationRefusalV1,
    PreparedSessionTransitionPresentationV1, PreparedSessionTransitionV1,
    SessionOperationTransitionRequestV1, TransitionAuthorizationRefusalV1,
    TransitionAuthorizationV1,
};
pub use attached_compact_group::{
    AttachedCompactGroupAvailabilityCategoryV1, AttachedCompactGroupAvailabilityV1,
    AttachedCompactGroupCommitResultV1, AttachedCompactGroupSessionErrorV1,
    PendingAttachedCompactGroupV1,
};
/// Concrete internal Rust transaction seam for the API-owned attached-C6 bridge.
///
/// This remains public because `ferrum-api` must retain and redeem the opaque prepared
/// transaction under the current crate dependency direction. It is intentionally not a
/// general attachment API: the document session retains admission, fencing, deferred IDs,
/// and atomic commit authority.
pub use attached_cyclohexane::{AttachedCyclohexaneSessionErrorV1, PendingAttachedCyclohexaneV1};
pub use bracket::PendingCreateBracket;
pub use clipboard::DocumentClipboardPasteResultV1;
#[allow(unused_imports)]
pub use explicit_fragment::PendingCreateExplicitFragmentV1;
pub use free_compact_group::{
    FreeCompactGroupPlacementCommitResultV1, FreeCompactGroupPlacementSessionErrorV1,
    PendingPlaceFreeCompactGroupV1,
};
pub use linear_form::{PendingLinearFormConvertV1, PreparedLinearFormConvertResultV1};
pub use presentation_creation::{
    PresentationAppearanceV1, PresentationCreateRequestV1, PresentationVectorCreateKindV1,
};
pub use reaction_authoring_command_v1::{
    DocumentCreateReactionCommandV1, DocumentDeleteReactionCommandV1,
    DocumentReactionAuthoringCommandKindV1, DocumentReactionMemberTargetsV1,
    DocumentReplaceReactionMembersCommandV1, ReactionAuthoringCommandRefusalV1,
};
#[allow(unused_imports)]
pub use reaction_member_selection_v1::{
    DocumentReactionListDispositionV1, DocumentReactionListObservationV1,
    DocumentReactionListReactionV1, DocumentReactionMemberObservationV1,
    DocumentReactionMemberSelectionV1, DocumentReactionSelectionObservationV1,
    ReactionMemberSelectionRefusalV1,
};
pub use structural_deletion::{PendingDeleteCompactGroupV1, PendingDeleteStructureV1};
pub use text_placement::PendingTextPlacementV1;
pub use user_template::DocumentUserTemplateResultV1;
pub use wavy::PendingCreateWavy;

/// An owned structural serialization of the authoritative CDML tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSnapshot {
    revision: u64,
    cdml: String,
    digest: [u8; 32],
    is_dirty: bool,
}

impl DocumentSnapshot {
    pub(super) fn new(revision: u64, cdml: String, digest: [u8; 32], is_dirty: bool) -> Self {
        Self {
            revision,
            cdml,
            digest,
            is_dirty,
        }
    }

    /// Return the monotonic revision that produced this snapshot.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }
    /// Return the structural serialization of the authoritative CDML tree.
    #[must_use]
    pub fn cdml(&self) -> &str {
        &self.cdml
    }

    /// Return the SHA-256 digest of the structural CDML serialization.
    #[must_use]
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// Return whether this content differs from the saved baseline.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}

/// The authoritative outcome of one ordinary save attempt.
///
/// `Confirmed` means the session advanced its saved baseline. An unconfirmed
/// directory-entry replacement leaves the session dirty, so the caller can verify
/// the destination or make a recovery export without losing the unsaved indication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SaveOutcome {
    /// The replacement and its directory entry received supported confirmation.
    Confirmed,
    /// Replacement completed, but the platform cannot confirm the directory entry.
    DirectoryEntryUnconfirmed,
}

/// Result of publishing an immutable snapshot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Publication {
    published_snapshot: DocumentSnapshot,
    snapshot: DocumentSnapshot,
    outcome: SaveOutcome,
}

impl Publication {
    /// Return the current session snapshot after the publication attempt.
    ///
    /// A confirmed ordinary save returns a clean snapshot. A recovery export and
    /// an unconfirmed replacement return the still-dirty current snapshot.
    #[must_use]
    pub fn snapshot(&self) -> &DocumentSnapshot {
        &self.snapshot
    }

    /// Return the exact snapshot handed to the replacement operation.
    #[must_use]
    pub fn published_snapshot(&self) -> &DocumentSnapshot {
        &self.published_snapshot
    }

    /// Return the typed ordinary-save outcome.
    #[must_use]
    pub fn outcome(&self) -> SaveOutcome {
        self.outcome
    }
}

/// Failures while loading, serializing, or publishing a CDML snapshot.
#[derive(Debug, Error)]
pub enum DocumentSessionError {
    /// An atomic molecule batch must contain at least one validated molecule.
    #[error("molecule batch must contain at least one molecule")]
    EmptyMoleculeBatch,
    /// The supplied text did not produce a valid retained CDML document.
    #[error("cannot load CDML document: {0}")]
    Load(#[source] TypedDocumentError),
    /// A bounded native clipboard fragment was not insertion-valid for this session.
    #[error(transparent)]
    ClipboardPaste(#[from] super::DocumentClipboardPasteErrorV1),
    /// A prepared native Cut was invalid for this exact session state.
    #[error(transparent)]
    ClipboardCut(#[from] super::DocumentClipboardCutErrorV1),
    /// A bounded native user template was not valid for this insertion.
    #[error(transparent)]
    UserTemplate(#[from] super::DocumentUserTemplateErrorV1),
    /// The retained tree could not be structurally serialized.
    #[error("cannot serialize CDML document: {0}")]
    Serialize(#[source] XmlSerializationError),
    /// The caller did not name the current authoritative revision.
    #[error("document revision conflict: expected {expected}, current revision is {actual}")]
    RevisionConflict { expected: u64, actual: u64 },
    /// A revision-changing transition cannot advance beyond `u64::MAX`.
    #[error("document revision space is exhausted")]
    RevisionExhausted,
    /// A prepared insertion was already resolved by its owning session.
    #[error("prepared document insertion was already accepted")]
    PreparedOperationConsumed,
    /// A prepared insertion belongs to another session and remains retryable there.
    #[error("prepared document insertion belongs to a different document session")]
    PreparedOperationForeignSession,
    /// A typed operation was rejected before a state transition.
    #[error(transparent)]
    Operation(#[from] SessionOperationError),
    /// The supplied authorization does not match the semantic operation.
    #[error("session transition authorization refused: {0:?}")]
    TransitionAuthorization(TransitionAuthorizationRefusalV1),
    /// A direct-bond semantic request was rejected before a state transition.
    #[error(transparent)]
    DirectBondAdmission(#[from] DirectBondAdmissionRefusalV1),
    /// The renderer rejected the exact prospective document state before it could
    /// become a visible prepared operation.
    #[error("renderer rejected the prospective document state")]
    RendererAdmission,
    /// Projection extraction rejected a required retained document fact.
    ///
    /// The enclosed [`ProjectionError`] remains the error source, preserving its
    /// stable category and source context for foreign-language error mapping.
    #[error(transparent)]
    Projection(#[from] ProjectionError),
    /// A selected durable direct-Haworth profile could not be re-observed.
    #[error(transparent)]
    DirectHaworthReobservation(#[from] super::DirectHaworthReobservationErrorV1),
    /// No adjacent retained logical history entry exists.
    #[error("document history navigation is unavailable")]
    HistoryUnavailable,
    /// The requested destination is not a regular file path suitable for replacement.
    #[error("cannot atomically publish to {path}: {reason}")]
    InvalidDestination {
        /// The rejected destination.
        path: PathBuf,
        /// Stable explanation for the rejection.
        reason: &'static str,
    },
    /// Publication failed before replacement, and temporary cleanup succeeded.
    #[error("could not publish CDML to {path} before replacement: {source}")]
    PublishNotStarted {
        /// Intended destination.
        path: PathBuf,
        /// I/O failure before replacement.
        #[source]
        source: io::Error,
    },
    /// Publication failed before replacement and removing its temporary file failed.
    #[error(
        "could not publish CDML to {path} before replacement: {source}; temporary cleanup failed: {cleanup}"
    )]
    PublishNotStartedWithCleanup {
        /// Intended destination.
        path: PathBuf,
        /// I/O failure before replacement.
        source: io::Error,
        /// Failure while removing the temporary artifact.
        cleanup: io::Error,
    },
    /// The destination changed to an invalid entry before replacement, and cleanup failed.
    #[error(
        "destination {path} became invalid before replacement: {reason}; temporary cleanup failed: {cleanup}"
    )]
    ReplacementRejectedWithCleanup {
        /// Intended destination.
        path: PathBuf,
        /// The validation failure observed immediately before replacement.
        reason: String,
        /// Failure while removing the temporary artifact.
        cleanup: io::Error,
    },
    /// Replacement completed, but supported directory confirmation failed.
    #[error("CDML was published to {path}, but directory durability confirmation failed: {source}")]
    PublishPossiblyCompleted {
        /// Intended destination.
        path: PathBuf,
        /// I/O failure after replacement.
        #[source]
        source: io::Error,
    },
    /// Random temporary-name generation failed.
    #[error("could not create a unique temporary name for {path}: {detail}")]
    TemporaryName {
        /// Intended destination.
        path: PathBuf,
        /// Random-source failure.
        detail: String,
    },
    /// All bounded attempts at a unique same-directory temporary name collided.
    #[error("could not reserve a unique temporary file beside {path}")]
    TemporaryNameExhausted {
        /// Intended destination.
        path: PathBuf,
    },
}

/// One authoritative retained CDML tree and its revision-bound transaction state.
#[derive(Debug)]
pub struct DocumentSession {
    authoring_capability_issuer: AuthoringCapabilityIssuerV1,
    renderer_admission_issuer: u64,
    next_renderer_admission_sequence: u64,
    admitted_history: admitted_transition_v1::AdmittedHistoryV1,
    saved_baseline: SavedBaseline,
    generated_ids: GeneratedIdSequences,
}

impl DocumentSession {
    #[must_use]
    pub(crate) fn authoring_capability_issuer_v1(&self) -> AuthoringCapabilityIssuerV1 {
        self.authoring_capability_issuer.clone()
    }

    /// Issue one opaque authoring receipt for this live document session.
    ///
    /// Receipt ownership, validation, claiming, and terminal disposition stay
    /// document-private; interaction routes receive only this opaque input.
    #[must_use]
    pub fn issue_authoring_capability_v1(&self) -> crate::AuthoringCapabilityV1 {
        self.authoring_capability_issuer.issue()
    }

    /// Return whether the current authoritative CDML index owns this durable ID.
    ///
    /// This generic index query is provided for bridge-side candidate allocation;
    /// it does not expose XML records or grant a mutation capability.
    #[doc(hidden)]
    #[must_use]
    pub fn contains_durable_id_v1(&self, identifier: &str) -> bool {
        PersistentId::new(identifier.to_owned())
            .ok()
            .is_some_and(|id| self.current_index_v1().resolve_id(&id).is_some())
    }
    /// Produce an owned structural serialization of the retained tree.
    pub fn snapshot(&self) -> Result<DocumentSnapshot, DocumentSessionError> {
        let current = self.current_state_v1();
        Ok(current.snapshot(!self.saved_baseline.is_current(current)))
    }

    /// Observe the current state through one revision-bound immutable envelope.
    pub fn observe(
        &self,
        expected_revision: u64,
    ) -> Result<SessionDocumentObservationV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        self.document_observation()
    }

    /// Observe one selected direct-root atom with the oxidation V1 convention.
    ///
    /// The request is authenticated against this session's current revision and
    /// digest. Root counts are admitted before a chemistry graph is materialized.
    pub fn observe_atom_oxidation_v1(
        &self,
        request: &super::DocumentAtomOxidationObservationRequestV1,
    ) -> Result<super::DocumentAtomOxidationResultV1, super::DocumentAtomOxidationRefusalV1> {
        let current = self.current_state_v1();
        let snapshot = current.snapshot(!self.saved_baseline.is_current(current));
        super::chemistry::observe_current_document_atom_oxidation_v1(
            current.document(),
            &snapshot,
            request,
        )
    }

    /// Prepare graph inputs for the SMARTS query operation at one exact revision.
    pub fn prepare_smarts_snapshot_v1(
        &self,
        expected_revision: u64,
    ) -> Result<super::PreparedDocumentSmartsSnapshotV1, super::DocumentSmartsSnapshotErrorV1> {
        let current = self.current_state_v1();
        if current.revision() != expected_revision {
            return Err(super::DocumentSmartsSnapshotErrorV1::StaleRevision {
                expected: expected_revision,
                actual: current.revision(),
            });
        }
        let snapshot = current.snapshot(!self.saved_baseline.is_current(current));
        super::document_smarts_snapshot_v1::prepare_smarts_snapshot_v1(
            current.document(),
            &snapshot,
        )
    }

    /// Return whether the retained session history has an earlier state.
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.has_undo_history_v1()
    }

    /// Return whether the retained session history has a later state.
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.has_redo_history_v1()
    }

    /// Observe complete-document literal-ID ambiguity facts at one exact revision.
    ///
    /// The facts include opaque and unsupported retained XML content, without
    /// exposing that XML to callers.
    pub fn observe_complete_document_identity_facts_v1(
        &self,
        expected_revision: u64,
    ) -> Result<super::CompleteDocumentIdentityFactsV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        Ok(self
            .current_index_v1()
            .complete_document_identity_facts_v1())
    }

    /// Apply one narrow typed operation through renderer admission.
    ///
    /// This compatibility spelling delegates to the opaque admitted-transition
    /// boundary and never appends a raw operation candidate directly.
    /// Navigate to the preceding retained logical state as a new monotonic revision.
    pub fn undo(
        &mut self,
        expected_revision: u64,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.execute_history_navigation_transition_v1(
            expected_revision,
            admitted_transition_v1::HistoryNavigationDirectionV1::Undo,
        )
    }

    /// Navigate to the succeeding retained logical state as a new monotonic revision.
    pub fn redo(
        &mut self,
        expected_revision: u64,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.execute_history_navigation_transition_v1(
            expected_revision,
            admitted_transition_v1::HistoryNavigationDirectionV1::Redo,
        )
    }

    /// Prepare one V1 free-standing atom insertion at the current revision.
    ///
    /// The durable molecule selector is resolved against this exact document. The
    /// session allocates the atom identity and validates the complete detached
    /// candidate before issuing its document-local token. Rejected requests consume
    /// neither a token nor a generated identity.
    pub fn recovery_export(
        &self,
        path: &Path,
        expected_revision: u64,
    ) -> Result<Publication, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let snapshot = self.snapshot()?;
        let durability = publish_snapshot(path, snapshot.cdml())?;
        Ok(Publication::from_durability(
            snapshot.clone(),
            snapshot,
            durability,
        ))
    }

    fn require_fence(&self, fence: DocumentFenceV1) -> Result<(), DirectBondGestureErrorV1> {
        if self.current_revision_v1() != fence.revision() {
            return Err(DirectBondGestureErrorV1::StaleRevision);
        }
        if self.current_digest_v1() != fence.digest() {
            return Err(DirectBondGestureErrorV1::StaleDigest);
        }
        Ok(())
    }

    fn direct_atom_point(
        &self,
        object_id: &DocumentObjectIdV1,
    ) -> Result<Option<DirectBondPoint2V1>, crate::ProjectionError> {
        let Some(target) = self
            .current_document_v1()
            .resolve_document_object_id(object_id)
        else {
            return Ok(None);
        };
        if target.class() != TypedClass::Atom {
            return Ok(None);
        }
        let point_record = target
            .children_of(TypedClass::Point)
            .next()
            .ok_or_else(|| crate::ProjectionError::MissingPoint {
                context: format!("direct-bond endpoint {}", object_id.as_str()),
            })?;
        let point = super::projection_v1::point(point_record)?;
        DirectBondPoint2V1::new(point.x(), point.y())
            .map(Some)
            .map_err(|error| crate::ProjectionError::InvalidValue {
                context: format!("direct-bond endpoint {}", object_id.as_str()),
                field: "point",
                value: error.to_string(),
            })
    }

    fn reject_existing_bond_for_object_ids(
        &self,
        start: &DocumentObjectIdV1,
        end: &DocumentObjectIdV1,
    ) -> Result<(), SessionOperationError> {
        let (molecule, start_atom) = self.resolve_bond_atom(start)?;
        let (_, end_atom) = self.resolve_bond_atom(end)?;
        self.reject_existing_bond(&molecule, &start_atom, &end_atom)
    }

    fn require_current(&self, expected_revision: u64) -> Result<(), DocumentSessionError> {
        let actual = self.current_revision_v1();
        if actual == expected_revision {
            Ok(())
        } else {
            Err(DocumentSessionError::RevisionConflict {
                expected: expected_revision,
                actual,
            })
        }
    }

    fn operation_result(&self) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.document_observation()
            .map(SessionOperationResultV1::new)
    }

    fn document_observation(&self) -> Result<SessionDocumentObservationV1, DocumentSessionError> {
        let current = self.current_state_v1();
        let snapshot = current.snapshot(!self.saved_baseline.is_current(current));
        SessionDocumentObservationV1::from_snapshot(snapshot)
            .map_err(DocumentSessionError::Projection)
    }

    pub fn save_atomic(
        &mut self,
        path: &Path,
        expected_revision: u64,
    ) -> Result<Publication, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let snapshot = self.snapshot()?;
        let durability = publish_snapshot(path, snapshot.cdml())?;
        if durability == PublicationDurability::Confirmed {
            self.saved_baseline = SavedBaseline::from_state(self.current_state_v1());
        }
        let current = self.snapshot()?;
        Ok(Publication::from_durability(snapshot, current, durability))
    }

    #[cfg(test)]
    pub(super) fn set_revision_for_test(&mut self, revision: u64) {
        self.set_current_revision_for_test_v1(revision);
    }

    #[cfg(test)]
    pub(super) fn record_save_outcome_for_test(
        &mut self,
        durability: PublicationDurability,
    ) -> Result<Publication, DocumentSessionError> {
        let published = self.snapshot()?;
        if durability == PublicationDurability::Confirmed {
            self.saved_baseline = SavedBaseline::from_state(self.current_state_v1());
        }
        let current = self.snapshot()?;
        Ok(Publication::from_durability(published, current, durability))
    }
}

impl Publication {
    fn from_durability(
        published_snapshot: DocumentSnapshot,
        snapshot: DocumentSnapshot,
        durability: PublicationDurability,
    ) -> Self {
        let outcome = match durability {
            PublicationDurability::Confirmed => SaveOutcome::Confirmed,
            PublicationDurability::DirectoryEntryUnconfirmed => {
                SaveOutcome::DirectoryEntryUnconfirmed
            }
        };
        Self {
            published_snapshot,
            snapshot,
            outcome,
        }
    }
}
/// Renderer-only values surfaced by the crate's explicit admission namespace.
pub(crate) mod renderer_admission {
    pub use super::renderer_translation_snap_v1::{
        RendererTranslationSnapDeltaV1, RendererTranslationSnapRefusalV1,
    };
}

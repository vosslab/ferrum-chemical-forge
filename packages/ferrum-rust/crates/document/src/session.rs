//! Immutable CDML snapshots and safe same-directory publication.
//!
//! The session owns the mutable retained CDML tree, optimistic revisions, bounded
//! history, save baseline, and revision-bound prepared operations. Clients receive
//! immutable snapshots and operation envelopes only; no frontend may invent or
//! retain a competing document state.

use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use super::identity_index::ProvisionalToken;
use super::{
    BracketInsertionV1, BracketStyleV1, DetachedRegularRingInsertionV1, DocumentBondPresentationV1,
    DocumentObjectIdV1, MoleculeInsertionV1, PersistentId, Point3V1,
    PreparedStraightenDepictionsV1, ProjectionError, SessionDocumentObservationV1, TypedClass,
    TypedDocument, TypedDocumentError, WavyInsertionV1, XmlSerializationError,
    direct_bond_gesture_v1::{
        self, CommittedDirectBondGestureV1, DirectBondEndIntentV1, DirectBondEndpointV1,
        DirectBondGestureErrorV1, DirectBondGestureV1, DirectBondPoint2V1, DirectBondPreviewV1,
        DirectBondSessionOriginV1, DirectBondSnapPolicyV1, DocumentFenceV1,
    },
    generated_ids::GeneratedIdSequences,
    presentation_creation_gesture_v1::{
        self, ArrowGestureStyleV1, CommittedPresentationGestureV1, PresentationCreationGestureV1,
        PresentationCreationPreviewV1, PresentationGestureErrorV1, PresentationGestureKindV1,
        PresentationGesturePoint2V1, PresentationGestureSessionOriginV1,
        PresentationGestureSnapPolicyV1,
    },
    publication::{PublicationDurability, publish_snapshot},
    session_history::SessionHistory,
    session_operation::{
        Candidate, SessionOperation, SessionOperationError, SessionOperationResultV1,
        SessionOperationV1,
    },
    session_state::{RevisionState, SavedBaseline},
    typed_bond_insertion::BondedAtomInsertion,
};

mod bracket;
mod attached_cyclohexane;
mod clipboard;
mod clipboard_cut;
mod construction;
mod direct_haworth;
mod explicit_fragment;
mod linear_form;
mod molecule_batch_creation;
mod molecule_creation;
mod prepared;
mod sdf;
mod standalone_haworth;
mod straighten;
mod structural_deletion;
mod user_template;
mod wavy;
pub use bracket::PendingCreateBracket;
pub use attached_cyclohexane::{AttachedCyclohexaneSessionErrorV1, PendingAttachedCyclohexaneV1};
pub use clipboard::DocumentClipboardPasteResultV1;
pub use direct_haworth::{
    CommittedDirectHaworthResultV1, CommittedDirectHaworthV1, PendingDirectHaworthV1,
};
#[allow(unused_imports)]
pub use explicit_fragment::PendingCreateExplicitFragmentV1;
pub use linear_form::{PendingLinearFormConvertV1, PreparedLinearFormConvertResultV1};
pub use molecule_batch_creation::PendingCreateMoleculeBatchV1;
pub use sdf::PendingCreateSdfRecords;
pub use standalone_haworth::PendingStandaloneHaworthV1;
pub use structural_deletion::PendingDeleteStructureV1;
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

fn same_point(first: DirectBondPoint2V1, second: DirectBondPoint2V1) -> bool {
    first.x() == second.x() && first.y() == second.y()
}

fn snap_point(
    start: DirectBondPoint2V1,
    raw: DirectBondPoint2V1,
    policy: DirectBondSnapPolicyV1,
) -> Result<DirectBondPoint2V1, DirectBondGestureErrorV1> {
    let mut dx = raw.x() - start.x();
    let mut dy = raw.y() - start.y();
    let mut length = dx.hypot(dy);
    if let Some(increment) = policy.angle_increment_degrees()
        && length > 0.0
    {
        let step = f64::from(increment).to_radians();
        let angle = dy.atan2(dx);
        let snapped = (angle / step).round() * step;
        dx = length * snapped.cos();
        dy = length * snapped.sin();
    }
    if let Some(fixed) = policy.fixed_length_pt() {
        if length == 0.0 {
            return Err(DirectBondGestureErrorV1::CollapsedEndpoint);
        }
        length = fixed;
        let scale = length / dx.hypot(dy);
        dx *= scale;
        dy *= scale;
    }
    if policy.hex_grid() {
        const GRID: f64 = 10.0;
        dx = (dx / GRID).round() * GRID;
        dy = (dy / GRID).round() * GRID;
    }
    DirectBondPoint2V1::new(start.x() + dx, start.y() + dy)
}

fn map_direct_bond_commit_error(_: DocumentSessionError) -> DirectBondGestureErrorV1 {
    DirectBondGestureErrorV1::SessionConflict
}

/// A one-use, revision-bound prepared atom insertion.
///
/// The token is intentionally opaque. It originates from the exact current
/// document, can be committed only at its prepared revision, and is consumed only
/// after the fully validated candidate is accepted.
pub struct PendingCreateAtom {
    revision: u64,
    token: ProvisionalToken,
    identifier: PersistentId,
    candidate: Option<RevisionState>,
}

/// A one-use, revision-bound prepared bond insertion.
pub struct PendingCreateBond {
    revision: u64,
    token: ProvisionalToken,
    identifier: PersistentId,
    candidate: Option<RevisionState>,
}

/// A one-use, revision-bound prepared atom-plus-bond insertion.
pub struct PendingCreateBondedAtom {
    revision: u64,
    token: ProvisionalToken,
    atom_identifier: PersistentId,
    bond_identifier: PersistentId,
    candidate: Option<RevisionState>,
}

/// A one-use, revision-bound prepared complete molecule insertion.
pub struct PendingCreateMolecule {
    revision: u64,
    token: ProvisionalToken,
    molecule_identifier: PersistentId,
    atom_identifiers: Vec<PersistentId>,
    bond_identifiers: Vec<PersistentId>,
    candidate: Option<RevisionState>,
}

impl std::fmt::Debug for PendingCreateMolecule {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateMolecule")
            .field("revision", &self.revision)
            .field("molecule_identifier", &self.molecule_identifier)
            .field("atom_count", &self.atom_identifiers.len())
            .field("bond_count", &self.bond_identifiers.len())
            .field("is_resolved", &self.candidate.is_none())
            .finish()
    }
}

impl PendingCreateMolecule {
    /// Return the durable molecule ID created when this candidate is committed.
    #[must_use]
    pub fn molecule_identifier(&self) -> &PersistentId {
        &self.molecule_identifier
    }

    /// Return durable atom IDs in inserted source order.
    #[must_use]
    pub fn atom_identifiers(&self) -> &[PersistentId] {
        &self.atom_identifiers
    }

    /// Return durable bond IDs in inserted source order.
    #[must_use]
    pub fn bond_identifiers(&self) -> &[PersistentId] {
        &self.bond_identifiers
    }
}

impl std::fmt::Debug for PendingCreateAtom {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateAtom")
            .field("revision", &self.revision)
            .field("identifier", &self.identifier)
            .field("is_resolved", &self.candidate.is_none())
            .finish()
    }
}

impl PendingCreateAtom {
    /// Return the durable ID that will be created if this candidate is committed.
    #[must_use]
    pub fn identifier(&self) -> &PersistentId {
        &self.identifier
    }
}

impl std::fmt::Debug for PendingCreateBond {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateBond")
            .field("revision", &self.revision)
            .field("identifier", &self.identifier)
            .field("is_resolved", &self.candidate.is_none())
            .finish()
    }
}

impl PendingCreateBond {
    /// Return the durable ID that will be created if this candidate is committed.
    #[must_use]
    pub fn identifier(&self) -> &PersistentId {
        &self.identifier
    }
}

impl std::fmt::Debug for PendingCreateBondedAtom {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PendingCreateBondedAtom")
            .field("revision", &self.revision)
            .field("atom_identifier", &self.atom_identifier)
            .field("bond_identifier", &self.bond_identifier)
            .field("is_resolved", &self.candidate.is_none())
            .finish()
    }
}

impl PendingCreateBondedAtom {
    /// Return the durable atom ID created if this candidate is committed.
    #[must_use]
    pub fn atom_identifier(&self) -> &PersistentId {
        &self.atom_identifier
    }

    /// Return the durable bond ID created if this candidate is committed.
    #[must_use]
    pub fn bond_identifier(&self) -> &PersistentId {
        &self.bond_identifier
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
    bridge_session_origin: u64,
    history: SessionHistory,
    saved_baseline: SavedBaseline,
    generated_ids: GeneratedIdSequences,
    direct_bond_origin: DirectBondSessionOriginV1,
    presentation_gesture_origin: PresentationGestureSessionOriginV1,
    text_placement_origin: crate::text_placement_gesture_v1::TextPlacementSessionOriginV1,
    text_placement_consumed: std::collections::HashSet<u64>,
}

impl DocumentSession {
    /// Stable process-local identity for a bridge-owned opaque capability.
    ///
    /// This is not a document operation or durable document fact. It exists so
    /// renderer-owning transaction bridges can survive ordinary Rust moves of a
    /// session value without using a memory address as authority.
    #[doc(hidden)]
    #[must_use]
    pub const fn bridge_session_origin_v1(&self) -> u64 {
        self.bridge_session_origin
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
            .is_some_and(|id| {
                self.history
                    .current()
                    .document()
                    .indexed()
                    .resolve_id(&id)
                    .is_some()
            })
    }
    /// Begin one opaque direct-root Text placement.  The returned token has no
    /// XML or mutable document state and is valid only for this exact snapshot.
    pub fn begin_text_placement_gesture_v1(
        &self,
        fence: DocumentFenceV1,
        anchor: PresentationGesturePoint2V1,
    ) -> Result<crate::TextPlacementGestureV1, crate::TextPlacementErrorV1> {
        crate::text_placement_gesture_v1::begin(self.text_placement_origin, fence, anchor)
    }

    pub fn preview_text_placement_gesture_v1(
        &self,
        gesture: &crate::TextPlacementGestureV1,
        content: crate::TextPlacementContentV1,
    ) -> Result<crate::TextPlacementPreviewV1, crate::TextPlacementErrorV1> {
        crate::text_placement_gesture_v1::preview(
            self.text_placement_origin,
            self.history.current().revision(),
            *self.history.current().digest(),
            gesture,
            content,
        )
    }

    pub fn commit_text_placement_gesture_v1(
        &mut self,
        gesture: &crate::TextPlacementGestureV1,
        preview: &crate::TextPlacementPreviewV1,
    ) -> Result<crate::CommittedTextPlacementV1, crate::TextPlacementErrorV1> {
        use crate::text_placement_gesture_v1::{
            CommittedTextPlacementV1, TextPlacementErrorV1, belongs_to,
        };
        if !belongs_to(self.text_placement_origin, gesture)
            || !belongs_to(self.text_placement_origin, &preview.gesture)
        {
            return Err(TextPlacementErrorV1::ForeignSession);
        }
        if gesture.capability != preview.gesture.capability {
            return Err(TextPlacementErrorV1::MismatchedPreview);
        }
        if self
            .text_placement_consumed
            .contains(&gesture.capability.nonce)
        {
            return Err(TextPlacementErrorV1::ReplayedGesture);
        }
        if gesture.fence.revision() != self.history.current().revision()
            || gesture.fence.digest() != *self.history.current().digest()
        {
            return Err(TextPlacementErrorV1::StaleSnapshot);
        }
        let (identifier, next_ids) = self
            .generated_ids
            .reserve_presentation(self.history.current().document().indexed())
            .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_authored_text_v1(
                &identifier,
                gesture.anchor,
                preview.content.runs(),
                preview.content.font_size(),
                preview.content.color(),
            )
            .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(TextPlacementErrorV1::SessionConflict)?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
        self.generated_ids = next_ids;
        self.text_placement_consumed
            .insert(gesture.capability.nonce);
        self.history.append(state);
        let result = self
            .operation_result()
            .map_err(|_| TextPlacementErrorV1::SessionConflict)?;
        Ok(CommittedTextPlacementV1::new(identifier, result))
    }

    pub fn begin_presentation_creation_gesture_v1(
        &self,
        fence: DocumentFenceV1,
        kind: PresentationGestureKindV1,
        start: PresentationGesturePoint2V1,
        style: ArrowGestureStyleV1,
        snap: PresentationGestureSnapPolicyV1,
    ) -> Result<PresentationCreationGestureV1, PresentationGestureErrorV1> {
        self.require_presentation_fence(fence)?;
        Ok(PresentationCreationGestureV1 {
            capability: self.presentation_gesture_origin.issue_gesture(),
            fence,
            kind,
            start,
            style,
            snap,
        })
    }
    pub fn preview_presentation_creation_gesture_v1(
        &self,
        gesture: &PresentationCreationGestureV1,
        end: PresentationGesturePoint2V1,
    ) -> Result<PresentationCreationPreviewV1, PresentationGestureErrorV1> {
        self.require_presentation_origin(gesture)?;
        self.require_presentation_fence(gesture.fence)?;
        presentation_creation_gesture_v1::preview(gesture.clone(), end)
    }
    pub fn commit_presentation_creation_gesture_v1(
        &mut self,
        gesture: &PresentationCreationGestureV1,
        preview: &PresentationCreationPreviewV1,
    ) -> Result<CommittedPresentationGestureV1, PresentationGestureErrorV1> {
        self.require_presentation_origin(gesture)?;
        self.require_presentation_origin(&preview.gesture)?;
        self.require_presentation_fence(gesture.fence)?;
        if gesture.capability != preview.gesture.capability {
            return Err(PresentationGestureErrorV1::PreviewMismatch);
        }
        let (identifier, next_ids) = self
            .generated_ids
            .reserve_presentation(self.history.current().document().indexed())
            .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
        let candidate = match gesture.kind {
            PresentationGestureKindV1::StraightNormalArrow => self
                .history
                .current()
                .document()
                .with_insert_straight_normal_arrow(
                    &identifier,
                    gesture.start,
                    preview.end,
                    gesture.style.start_head(),
                    gesture.style.end_head(),
                ),
            PresentationGestureKindV1::Plus => self
                .history
                .current()
                .document()
                .with_insert_standard_plus(&identifier, gesture.start),
        }
        .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(PresentationGestureErrorV1::SessionConflict)?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
        self.generated_ids = next_ids;
        self.history.append(state);
        let result = self
            .operation_result()
            .map_err(|_| PresentationGestureErrorV1::SessionConflict)?;
        Ok(CommittedPresentationGestureV1::new(
            gesture.kind,
            identifier,
            result,
        ))
    }
    fn require_presentation_origin(
        &self,
        gesture: &PresentationCreationGestureV1,
    ) -> Result<(), PresentationGestureErrorV1> {
        if gesture
            .capability
            .belongs_to(self.presentation_gesture_origin)
        {
            Ok(())
        } else {
            Err(PresentationGestureErrorV1::ForeignSession)
        }
    }
    fn require_presentation_fence(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<(), PresentationGestureErrorV1> {
        if self.history.current().revision() != fence.revision() {
            return Err(PresentationGestureErrorV1::StaleRevision);
        }
        if *self.history.current().digest() != fence.digest() {
            return Err(PresentationGestureErrorV1::StaleDigest);
        }
        Ok(())
    }
    /// Begin a pure direct normal-bond gesture from one existing direct atom.
    pub fn begin_direct_bond_gesture_v1(
        &self,
        fence: DocumentFenceV1,
        start_atom: DocumentObjectIdV1,
        presentation: DocumentBondPresentationV1,
        new_atom_element: String,
        snap: DirectBondSnapPolicyV1,
    ) -> Result<DirectBondGestureV1, DirectBondGestureErrorV1> {
        self.require_fence(fence)?;
        if !matches!(presentation, DocumentBondPresentationV1::Normal(_)) {
            return Err(DirectBondGestureErrorV1::UnsupportedPresentation);
        }
        let (start_molecule, _) = self
            .resolve_bond_atom(&start_atom)
            .map_err(|_| DirectBondGestureErrorV1::UnknownStartAtom)?;
        let start_point = self
            .direct_atom_point(&start_atom)
            .ok_or(DirectBondGestureErrorV1::UnknownStartAtom)?;
        Ok(DirectBondGestureV1 {
            capability: self.direct_bond_origin.issue_gesture(),
            fence,
            start_atom,
            start_molecule,
            presentation,
            new_atom_element,
            snap,
            start_point,
        })
    }

    /// Compute one disposable direct-bond preview without changing the document.
    pub fn preview_direct_bond_gesture_v1(
        &self,
        gesture: &DirectBondGestureV1,
        end: DirectBondEndIntentV1,
    ) -> Result<DirectBondPreviewV1, DirectBondGestureErrorV1> {
        self.require_direct_bond_origin(gesture)?;
        self.require_fence(gesture.fence)?;
        let endpoint = match end {
            DirectBondEndIntentV1::ExistingAtom { atom } => {
                if atom == gesture.start_atom {
                    return Err(DirectBondGestureErrorV1::SelfLoop);
                }
                let (molecule, _) = self
                    .resolve_bond_atom(&atom)
                    .map_err(|_| DirectBondGestureErrorV1::UnknownEndAtom)?;
                if molecule != gesture.start_molecule {
                    return Err(DirectBondGestureErrorV1::CrossMolecule);
                }
                self.reject_existing_bond_for_object_ids(&gesture.start_atom, &atom)
                    .map_err(|_| DirectBondGestureErrorV1::DuplicateBond)?;
                DirectBondEndpointV1::ExistingAtom {
                    point: self
                        .direct_atom_point(&atom)
                        .ok_or(DirectBondGestureErrorV1::UnknownEndAtom)?,
                    atom,
                }
            }
            DirectBondEndIntentV1::NewAtomAt { raw_point } => {
                let point = snap_point(gesture.start_point, raw_point, gesture.snap)?;
                if same_point(gesture.start_point, point) {
                    return Err(DirectBondGestureErrorV1::CollapsedEndpoint);
                }
                DirectBondEndpointV1::NewAtom {
                    point,
                    element: gesture.new_atom_element.clone(),
                }
            }
        };
        Ok(direct_bond_gesture_v1::overlay(gesture.clone(), endpoint))
    }

    /// Commit one checked preview through the existing prepared insertion seam.
    pub fn commit_direct_bond_gesture_v1(
        &mut self,
        gesture: &DirectBondGestureV1,
        preview: &DirectBondPreviewV1,
    ) -> Result<CommittedDirectBondGestureV1, DirectBondGestureErrorV1> {
        self.require_direct_bond_origin(gesture)?;
        self.require_direct_bond_origin(&preview.gesture)?;
        self.require_fence(gesture.fence)?;
        if preview.gesture.capability != gesture.capability {
            return Err(DirectBondGestureErrorV1::PreviewMismatch);
        }
        match &preview.endpoint {
            DirectBondEndpointV1::ExistingAtom { atom, .. } => {
                let (_, end_atom) = self
                    .resolve_bond_atom(atom)
                    .map_err(|_| DirectBondGestureErrorV1::UnknownEndAtom)?;
                let mut pending = self
                    .prepare_create_bond_v2(
                        gesture.fence.revision(),
                        &gesture.start_atom,
                        atom,
                        gesture.presentation,
                    )
                    .map_err(map_direct_bond_commit_error)?;
                let bond = pending.identifier().clone();
                let result = self
                    .commit_create_bond(gesture.fence.revision(), &mut pending)
                    .map_err(map_direct_bond_commit_error)?;
                Ok(CommittedDirectBondGestureV1::ExistingEndpoint {
                    bond,
                    end_atom,
                    result,
                })
            }
            DirectBondEndpointV1::NewAtom { point, element } => {
                let position = Point3V1::new(point.x(), point.y(), 0.0)
                    .map_err(|_| DirectBondGestureErrorV1::NonFinitePoint)?;
                let mut pending = self
                    .prepare_create_bonded_atom_v2(
                        gesture.fence.revision(),
                        &gesture.start_atom,
                        element,
                        position,
                        gesture.presentation,
                    )
                    .map_err(map_direct_bond_commit_error)?;
                let atom = pending.atom_identifier().clone();
                let bond = pending.bond_identifier().clone();
                let result = self
                    .commit_create_bonded_atom(gesture.fence.revision(), &mut pending)
                    .map_err(map_direct_bond_commit_error)?;
                Ok(CommittedDirectBondGestureV1::NewEndpoint { bond, atom, result })
            }
        }
    }

    fn require_direct_bond_origin(
        &self,
        gesture: &DirectBondGestureV1,
    ) -> Result<(), DirectBondGestureErrorV1> {
        if gesture.capability.belongs_to(self.direct_bond_origin) {
            Ok(())
        } else {
            Err(DirectBondGestureErrorV1::ForeignSession)
        }
    }
    /// Observe one complete-root translation anchor at an exact retained revision.
    pub fn observe_top_level_translation_anchor_v1(
        &self,
        expected_revision: u64,
        targets: Vec<super::TopLevelRootSelectorV1>,
    ) -> Result<super::TopLevelTranslationAnchorV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let current = self.history.current();
        current
            .document()
            .top_level_translation_anchor_v1(current.revision(), *current.digest(), targets)
            .map_err(|error| {
                DocumentSessionError::Operation(super::SessionOperationError::Candidate(error))
            })
    }

    /// Produce an owned structural serialization of the retained tree.
    pub fn snapshot(&self) -> Result<DocumentSnapshot, DocumentSessionError> {
        let current = self.history.current();
        Ok(current.snapshot(!self.saved_baseline.is_current(current)))
    }

    /// Commit one complete, already-admitted CDML replacement as the next revision.
    ///
    /// This is the generic compatibility transaction for complete CDML clients.
    /// It deliberately knows nothing about tools, gestures, render preflight, or
    /// durable-ID allocation. Tool-specific owners must validate their exact
    /// candidate before calling this method.
    pub fn commit_complete_cdml_transaction_v1(
        &mut self,
        fence: DocumentFenceV1,
        candidate_cdml: &str,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(fence.revision())?;
        if *self.history.current().digest() != fence.digest() {
            return Err(DocumentSessionError::RevisionConflict {
                expected: fence.revision(),
                actual: self.history.current().revision(),
            });
        }
        let document = TypedDocument::parse(candidate_cdml).map_err(DocumentSessionError::Load)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let state =
            RevisionState::from_document(revision, document).map_err(DocumentSessionError::Load)?;
        let snapshot = state.snapshot(!self.saved_baseline.is_current(&state));
        SessionDocumentObservationV1::from_state(state.document(), snapshot)
            .map_err(DocumentSessionError::Projection)?;
        self.history
            .try_reserve_append()
            .map_err(|_| SessionOperationError::HistoryResourceExhausted)?;
        self.history.append_reserved(state);
        self.operation_result()
    }

    /// Observe the current state through one revision-bound immutable envelope.
    pub fn observe(
        &self,
        expected_revision: u64,
    ) -> Result<SessionDocumentObservationV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        self.document_observation()
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
            .history
            .current()
            .document()
            .indexed()
            .complete_document_identity_facts_v1())
    }

    /// Apply one narrow typed operation with optimistic revision control.
    pub fn submit(
        &mut self,
        expected_revision: u64,
        operation: SessionOperation,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if let SessionOperation::V1(SessionOperationV1::DeleteStructure {
            molecule_id,
            atom_ids,
            bond_ids,
        }) = &operation
        {
            let mut pending = self.prepare_delete_structure_v1(
                expected_revision,
                molecule_id.clone(),
                atom_ids.clone(),
                bond_ids.clone(),
            )?;
            return self.commit_delete_structure_v1(expected_revision, &mut pending);
        }
        let current = self.history.current();
        match operation.prepare(current.document(), current.revision(), current.digest())? {
            Candidate::NoChange => self.operation_result(),
            Candidate::Changed(document) => {
                let revision = current
                    .next_revision()
                    .ok_or(DocumentSessionError::RevisionExhausted)?;
                let state = RevisionState::from_document(revision, *document)
                    .map_err(DocumentSessionError::Load)?;
                self.history.append(state);
                self.operation_result()
            }
        }
    }

    /// Navigate to the preceding retained logical state as a new monotonic revision.
    pub fn undo(
        &mut self,
        expected_revision: u64,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let source = self
            .history
            .undo_target()
            .ok_or(DocumentSessionError::HistoryUnavailable)?
            .canonical_cdml()
            .to_owned();
        let next_revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let document = TypedDocument::parse(&source).map_err(DocumentSessionError::Load)?;
        self.history.move_undo();
        let state = RevisionState::from_document(next_revision, document)
            .map_err(DocumentSessionError::Load)?;
        self.history.replace_current(state);
        self.operation_result()
    }

    /// Navigate to the succeeding retained logical state as a new monotonic revision.
    pub fn redo(
        &mut self,
        expected_revision: u64,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let source = self
            .history
            .redo_target()
            .ok_or(DocumentSessionError::HistoryUnavailable)?
            .canonical_cdml()
            .to_owned();
        let next_revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let document = TypedDocument::parse(&source).map_err(DocumentSessionError::Load)?;
        self.history.move_redo();
        let state = RevisionState::from_document(next_revision, document)
            .map_err(DocumentSessionError::Load)?;
        self.history.replace_current(state);
        self.operation_result()
    }

    /// Prepare one V1 free-standing atom insertion at the current revision.
    ///
    /// The durable molecule selector is resolved against this exact document. The
    /// session allocates the atom identity and validates the complete detached
    /// candidate before issuing its document-local token. Rejected requests consume
    /// neither a token nor a generated identity.
    pub fn prepare_create_atom_v1(
        &mut self,
        expected_revision: u64,
        molecule_object_id: &DocumentObjectIdV1,
        element: &str,
        position: Point3V1,
    ) -> Result<PendingCreateAtom, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let molecule_id = self.resolve_molecule_id(molecule_object_id)?;
        let (atom_id, generated_ids) = self
            .generated_ids
            .reserve_atom(self.history.current().document().indexed())?;
        let pending = self.prepare_create_atom_candidate(
            expected_revision,
            &molecule_id,
            atom_id,
            element,
            position,
        )?;
        self.generated_ids = generated_ids;
        Ok(pending)
    }

    fn prepare_create_atom_candidate(
        &mut self,
        expected_revision: u64,
        molecule_id: &PersistentId,
        atom_id: PersistentId,
        element: &str,
        position: Point3V1,
    ) -> Result<PendingCreateAtom, DocumentSessionError> {
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_atom(molecule_id, &atom_id, element, position)
            .map_err(SessionOperationError::Candidate)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let candidate_snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        SessionDocumentObservationV1::from_state(candidate.document(), candidate_snapshot)
            .map_err(DocumentSessionError::Projection)?;
        let token = prepared::issue_prepared_token(self.history.current_mut().document_mut())?;
        Ok(PendingCreateAtom {
            revision: expected_revision,
            token,
            identifier: atom_id,
            candidate: Some(candidate),
        })
    }

    fn resolve_molecule_id(
        &self,
        molecule_object_id: &DocumentObjectIdV1,
    ) -> Result<PersistentId, SessionOperationError> {
        let object_id = molecule_object_id.as_str().to_owned();
        let record = self
            .history
            .current()
            .document()
            .resolve_document_object_id(molecule_object_id)
            .ok_or_else(|| SessionOperationError::UnknownDocumentObject(object_id.clone()))?;
        if record.class() != TypedClass::Molecule {
            return Err(SessionOperationError::InvalidCreateAtomTarget(object_id));
        }
        let source_id = record
            .attribute("id")
            .ok_or_else(|| SessionOperationError::InvalidCreateAtomTarget(object_id.clone()))?;
        PersistentId::new(source_id.to_owned())
            .map_err(|_| SessionOperationError::InvalidCreateAtomTarget(object_id))
    }

    #[cfg(test)]
    pub(super) fn set_next_generated_molecule_sequence_for_test(&mut self, sequence: Option<u64>) {
        self.generated_ids = self.generated_ids.with_molecule_sequence(sequence);
    }

    #[cfg(test)]
    pub(super) fn set_next_generated_atom_sequence_for_test(&mut self, sequence: Option<u64>) {
        self.generated_ids = self.generated_ids.with_atom_sequence(sequence);
    }

    /// Accept one prepared atom insertion exactly once.
    pub fn commit_create_atom(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateAtom,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_prepared_candidate(
            expected_revision,
            pending.revision,
            &pending.token,
            &mut pending.candidate,
        )
    }

    /// Prepare one molecule-local bond insertion at the current revision.
    ///
    /// Endpoint selectors must name two distinct durable atoms under the same
    /// durable molecule. The session allocates the bond identity and validates the
    /// complete detached candidate before issuing its document-local token.
    pub fn prepare_create_bond_v2(
        &mut self,
        expected_revision: u64,
        start_atom_object_id: &DocumentObjectIdV1,
        end_atom_object_id: &DocumentObjectIdV1,
        presentation: DocumentBondPresentationV1,
    ) -> Result<PendingCreateBond, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if start_atom_object_id == end_atom_object_id {
            return Err(SessionOperationError::CreateBondSelfLoop(
                start_atom_object_id.as_str().to_owned(),
            )
            .into());
        }
        let (start_molecule, start_atom) = self.resolve_bond_atom(start_atom_object_id)?;
        let (end_molecule, end_atom) = self.resolve_bond_atom(end_atom_object_id)?;
        if start_molecule != end_molecule {
            return Err(SessionOperationError::CreateBondAcrossMolecules.into());
        }
        self.reject_existing_bond(&start_molecule, &start_atom, &end_atom)?;
        let (bond_id, generated_ids) = self
            .generated_ids
            .reserve_bond(self.history.current().document().indexed())?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_bond(
                &start_molecule,
                &bond_id,
                &start_atom,
                &end_atom,
                presentation,
            )
            .map_err(SessionOperationError::Candidate)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let candidate_snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        SessionDocumentObservationV1::from_state(candidate.document(), candidate_snapshot)
            .map_err(DocumentSessionError::Projection)?;
        let token = prepared::issue_prepared_token(self.history.current_mut().document_mut())?;
        self.generated_ids = generated_ids;
        Ok(PendingCreateBond {
            revision: expected_revision,
            token,
            identifier: bond_id,
            candidate: Some(candidate),
        })
    }

    /// Accept one prepared bond insertion exactly once.
    pub fn commit_create_bond(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateBond,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_prepared_candidate(
            expected_revision,
            pending.revision,
            &pending.token,
            &mut pending.candidate,
        )
    }

    /// Prepare one atom and its bond to an existing durable atom as one edit.
    ///
    /// Rust resolves the start atom and its containing molecule, allocates both
    /// durable identities, and validates the complete projected candidate before
    /// issuing a one-use token. No intermediate free-standing atom can become
    /// visible or enter history.
    pub fn prepare_create_bonded_atom_v2(
        &mut self,
        expected_revision: u64,
        start_atom_object_id: &DocumentObjectIdV1,
        element: &str,
        position: Point3V1,
        presentation: DocumentBondPresentationV1,
    ) -> Result<PendingCreateBondedAtom, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let (molecule_id, start_atom_id) = self.resolve_bond_atom(start_atom_object_id)?;
        let (identities, generated_ids) = self
            .generated_ids
            .reserve_bonded_atom(self.history.current().document().indexed())?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_bonded_atom(
                &molecule_id,
                &start_atom_id,
                BondedAtomInsertion::new(
                    &identities.atom,
                    &identities.bond,
                    element,
                    position,
                    presentation,
                ),
            )
            .map_err(SessionOperationError::Candidate)?;
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = RevisionState::from_document(revision, candidate)
            .map_err(DocumentSessionError::Load)?;
        let candidate_snapshot = candidate.snapshot(!self.saved_baseline.is_current(&candidate));
        SessionDocumentObservationV1::from_state(candidate.document(), candidate_snapshot)
            .map_err(DocumentSessionError::Projection)?;
        let token = prepared::issue_prepared_token(self.history.current_mut().document_mut())?;
        self.generated_ids = generated_ids;
        Ok(PendingCreateBondedAtom {
            revision: expected_revision,
            token,
            atom_identifier: identities.atom,
            bond_identifier: identities.bond,
            candidate: Some(candidate),
        })
    }

    /// Accept one prepared atom-plus-bond insertion exactly once.
    pub fn commit_create_bonded_atom(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateBondedAtom,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_prepared_candidate(
            expected_revision,
            pending.revision,
            &pending.token,
            &mut pending.candidate,
        )
    }

    fn resolve_bond_atom(
        &self,
        object_id: &DocumentObjectIdV1,
    ) -> Result<(PersistentId, PersistentId), SessionOperationError> {
        let object_key = object_id.as_str().to_owned();
        let document = self.history.current().document();
        let target = document
            .resolve_document_object_id(object_id)
            .ok_or_else(|| SessionOperationError::UnknownDocumentObject(object_key.clone()))?;
        if target.class() != TypedClass::Atom {
            return Err(SessionOperationError::InvalidCreateBondTarget(object_key));
        }
        let atom_id = target
            .attribute("id")
            .and_then(|value| PersistentId::new(value.to_owned()).ok())
            .ok_or_else(|| SessionOperationError::InvalidCreateBondTarget(object_key.clone()))?;
        for molecule_child in document.root().typed_children() {
            let molecule = molecule_child.record();
            if molecule.class() != TypedClass::Molecule {
                continue;
            }
            let contains_target = molecule.typed_children().iter().any(|child| {
                child.record().path() == target.path() && child.record().class() == TypedClass::Atom
            });
            if !contains_target {
                continue;
            }
            let molecule_id = molecule
                .attribute("id")
                .and_then(|value| PersistentId::new(value.to_owned()).ok())
                .ok_or(SessionOperationError::InvalidCreateBondTarget(object_key))?;
            return Ok((molecule_id, atom_id));
        }
        Err(SessionOperationError::InvalidCreateBondTarget(object_key))
    }

    fn reject_existing_bond(
        &self,
        molecule_id: &PersistentId,
        start_atom_id: &PersistentId,
        end_atom_id: &PersistentId,
    ) -> Result<(), SessionOperationError> {
        let document = self.history.current().document();
        let molecule = document
            .root()
            .children_of(TypedClass::Molecule)
            .find(|record| record.attribute("id") == Some(molecule_id.as_str()))
            .ok_or_else(|| {
                SessionOperationError::InvalidCreateBondTarget(molecule_id.to_string())
            })?;
        let duplicate = molecule.children_of(TypedClass::Bond).any(|bond| {
            let start = bond.attribute("start");
            let end = bond.attribute("end");
            (start == Some(start_atom_id.as_str()) && end == Some(end_atom_id.as_str()))
                || (start == Some(end_atom_id.as_str()) && end == Some(start_atom_id.as_str()))
        });
        if duplicate {
            return Err(SessionOperationError::CreateBondDuplicate {
                start: start_atom_id.as_str().to_owned(),
                end: end_atom_id.as_str().to_owned(),
            });
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn set_next_generated_bond_sequence_for_test(&mut self, sequence: Option<u64>) {
        self.generated_ids = self.generated_ids.with_bond_sequence(sequence);
    }

    fn commit_prepared_candidate(
        &mut self,
        expected_revision: u64,
        prepared_revision: u64,
        token: &ProvisionalToken,
        candidate: &mut Option<RevisionState>,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if candidate.is_none() {
            return Err(DocumentSessionError::PreparedOperationConsumed);
        }
        if prepared_revision != expected_revision {
            return Err(DocumentSessionError::RevisionConflict {
                expected: prepared_revision,
                actual: expected_revision,
            });
        }
        self.history
            .current()
            .document()
            .verify_provisional_token(token)
            .map_err(prepared::map_prepared_token_error)?;
        self.history
            .current_mut()
            .document_mut()
            .consume_provisional_token(token)
            .map_err(SessionOperationError::Candidate)?;
        let state = candidate
            .take()
            .expect("the candidate presence check established this invariant");
        self.history.append(state);
        self.operation_result()
    }

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
        if self.history.current().revision() != fence.revision() {
            return Err(DirectBondGestureErrorV1::StaleRevision);
        }
        if *self.history.current().digest() != fence.digest() {
            return Err(DirectBondGestureErrorV1::StaleDigest);
        }
        Ok(())
    }

    fn direct_atom_point(&self, object_id: &DocumentObjectIdV1) -> Option<DirectBondPoint2V1> {
        let target = self
            .history
            .current()
            .document()
            .resolve_document_object_id(object_id)?;
        if target.class() != TypedClass::Atom {
            return None;
        }
        let point_record = target.children_of(TypedClass::Point).next()?;
        let point = super::projection_v1::point(point_record).ok()?;
        DirectBondPoint2V1::new(point.x(), point.y()).ok()
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
        let actual = self.history.current().revision();
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
        let current = self.history.current();
        let snapshot = current.snapshot(!self.saved_baseline.is_current(current));
        SessionDocumentObservationV1::from_state(current.document(), snapshot)
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
            self.saved_baseline = SavedBaseline::from_state(self.history.current());
        }
        let current = self.snapshot()?;
        Ok(Publication::from_durability(snapshot, current, durability))
    }

    #[cfg(test)]
    pub(super) fn set_revision_for_test(&mut self, revision: u64) {
        self.history.current_mut().set_revision_for_test(revision);
    }

    #[cfg(test)]
    pub(super) fn record_save_outcome_for_test(
        &mut self,
        durability: PublicationDurability,
    ) -> Result<Publication, DocumentSessionError> {
        let published = self.snapshot()?;
        if durability == PublicationDurability::Confirmed {
            self.saved_baseline = SavedBaseline::from_state(self.history.current());
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

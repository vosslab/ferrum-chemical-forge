//! Immutable CDML snapshots and safe same-directory publication.
//!
//! This is the narrow M8a foundation: it loads one retained [`TypedDocument`],
//! creates owned structural snapshots, and publishes a snapshot.  Editing state,
//! revision numbers, saved baselines, and conflict detection deliberately belong to
//! a later transaction milestone.

use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

use super::{
    PersistentId, ProvisionalToken, TypedDocument, TypedDocumentError, XmlSerializationError,
    publication::{PublicationDurability, publish_snapshot},
    session_history::SessionHistory,
    session_operation::{Candidate, SessionObservationV1, SessionOperation, SessionOperationError},
    session_state::{RevisionState, SavedBaseline},
};

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

/// A one-use, revision-bound prepared atom insertion.
///
/// The token is intentionally opaque. It originates from the exact current
/// document, can be committed only at its prepared revision, and is consumed only
/// after the fully validated candidate is accepted.
#[derive(Debug)]
pub struct PendingCreateAtom {
    revision: u64,
    token: ProvisionalToken,
    identifier: PersistentId,
    candidate: Option<Box<TypedDocument>>,
}

impl PendingCreateAtom {
    /// Return the durable ID that will be created if this candidate is committed.
    #[must_use]
    pub fn identifier(&self) -> &PersistentId {
        &self.identifier
    }
}

/// Failures while loading, serializing, or publishing a CDML snapshot.
#[derive(Debug, Error)]
pub enum DocumentSessionError {
    /// The supplied text did not produce a valid retained CDML document.
    #[error("cannot load CDML document: {0}")]
    Load(#[source] TypedDocumentError),
    /// The retained tree could not be structurally serialized.
    #[error("cannot serialize CDML document: {0}")]
    Serialize(#[source] XmlSerializationError),
    /// The caller did not name the current authoritative revision.
    #[error("document revision conflict: expected {expected}, current revision is {actual}")]
    RevisionConflict { expected: u64, actual: u64 },
    /// A revision-changing transition cannot advance beyond `u64::MAX`.
    #[error("document revision space is exhausted")]
    RevisionExhausted,
    /// A prepared insertion was already accepted.
    #[error("prepared atom insertion was already accepted")]
    PreparedOperationConsumed,
    /// A typed operation was rejected before a state transition.
    #[error(transparent)]
    Operation(#[from] SessionOperationError),
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
    history: SessionHistory,
    saved_baseline: SavedBaseline,
}

impl DocumentSession {
    /// Parse CDML into the sole authoritative retained tree.
    pub fn load(source: &str) -> Result<Self, DocumentSessionError> {
        let document = TypedDocument::parse(source).map_err(DocumentSessionError::Load)?;
        let initial =
            RevisionState::from_document(0, document).map_err(DocumentSessionError::Load)?;
        let saved_baseline = SavedBaseline::from_state(&initial);
        Ok(Self {
            history: SessionHistory::new(initial, 20),
            saved_baseline,
        })
    }

    /// Produce an owned structural serialization of the retained tree.
    pub fn snapshot(&self) -> Result<DocumentSnapshot, DocumentSessionError> {
        let current = self.history.current();
        Ok(current.snapshot(!self.saved_baseline.is_current(current)))
    }

    /// Observe the current state through one revision-bound immutable envelope.
    pub fn observe(
        &self,
        expected_revision: u64,
    ) -> Result<SessionObservationV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        self.snapshot().map(SessionObservationV1::new)
    }

    /// Apply one narrow typed operation with optimistic revision control.
    pub fn submit(
        &mut self,
        expected_revision: u64,
        operation: SessionOperation,
    ) -> Result<DocumentSnapshot, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let current = self.history.current();
        match operation.prepare(current.document())? {
            Candidate::NoChange => self.snapshot(),
            Candidate::Changed(document) => {
                let revision = current
                    .next_revision()
                    .ok_or(DocumentSessionError::RevisionExhausted)?;
                let state = RevisionState::from_document(revision, *document)
                    .map_err(DocumentSessionError::Load)?;
                self.history.append(state);
                self.snapshot()
            }
        }
    }

    /// Navigate to the preceding retained logical state as a new monotonic revision.
    pub fn undo(
        &mut self,
        expected_revision: u64,
    ) -> Result<DocumentSnapshot, DocumentSessionError> {
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
        self.snapshot()
    }

    /// Navigate to the succeeding retained logical state as a new monotonic revision.
    pub fn redo(
        &mut self,
        expected_revision: u64,
    ) -> Result<DocumentSnapshot, DocumentSessionError> {
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
        self.snapshot()
    }

    /// Prepare a typed atom insertion at the current revision.
    ///
    /// Preparation validates the complete detached candidate before issuing the
    /// document-local token, so a rejected request cannot consume a token.
    pub fn prepare_create_atom(
        &mut self,
        expected_revision: u64,
        molecule_id: &PersistentId,
        atom_id: PersistentId,
        element: &str,
    ) -> Result<PendingCreateAtom, DocumentSessionError> {
        self.require_current(expected_revision)?;
        let candidate = self
            .history
            .current()
            .document()
            .with_insert_atom(molecule_id, &atom_id, element)
            .map_err(SessionOperationError::Candidate)?;
        let token = self
            .history
            .current_mut()
            .document_mut()
            .issue_provisional_token();
        Ok(PendingCreateAtom {
            revision: expected_revision,
            token,
            identifier: atom_id,
            candidate: Some(Box::new(candidate)),
        })
    }

    /// Accept one prepared atom insertion exactly once.
    pub fn commit_create_atom(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateAtom,
    ) -> Result<DocumentSnapshot, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if pending.candidate.is_none() {
            return Err(DocumentSessionError::PreparedOperationConsumed);
        }
        if pending.revision != expected_revision {
            return Err(DocumentSessionError::RevisionConflict {
                expected: pending.revision,
                actual: expected_revision,
            });
        }
        let revision = self
            .history
            .current()
            .next_revision()
            .ok_or(DocumentSessionError::RevisionExhausted)?;
        let candidate = pending
            .candidate
            .take()
            .expect("the prepared-candidate presence check established this invariant");
        self.history
            .current_mut()
            .document_mut()
            .consume_provisional_token(pending.token.clone())
            .map_err(SessionOperationError::Candidate)?;
        let state = RevisionState::from_document(revision, *candidate)
            .map_err(DocumentSessionError::Load)?;
        self.history.append(state);
        self.snapshot()
    }

    /// Export one exact snapshot without changing baseline, history, or revision.
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

    /// Atomically publish the current snapshot to an explicit regular-file path.
    ///
    /// On Unix, every parent component is opened without following links. The final
    /// entry is then inspected and replaced relative to that retained directory
    /// descriptor. Renaming the visible parent path after it has been opened cannot
    /// redirect publication. Existing-target permissions are intentionally not copied.
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

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
    BracketInsertionV1, BracketStyleV1, DocumentBondOrderV1, DocumentObjectIdV1,
    MoleculeInsertionV1, PersistentId, Point3V1, PreparedStraightenDepictionsV1, ProjectionError,
    SessionDocumentObservationV1, TypedClass, TypedDocument, TypedDocumentError, WavyInsertionV1,
    XmlSerializationError,
    generated_ids::GeneratedIdSequences,
    publication::{PublicationDurability, publish_snapshot},
    session_history::SessionHistory,
    session_operation::{
        Candidate, SessionOperation, SessionOperationError, SessionOperationResultV1,
    },
    session_state::{RevisionState, SavedBaseline},
    typed_bond_insertion::BondedAtomInsertion,
};

mod bracket;
mod construction;
mod direct_haworth;
mod linear_form;
mod prepared;
mod sdf;
mod straighten;
mod wavy;
pub use bracket::PendingCreateBracket;
pub use direct_haworth::{
    CommittedDirectHaworthResultV1, CommittedDirectHaworthV1, PendingDirectHaworthV1,
};
pub use linear_form::{PendingLinearFormConvertV1, PreparedLinearFormConvertResultV1};
pub use sdf::PendingCreateSdfRecords;
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
    history: SessionHistory,
    saved_baseline: SavedBaseline,
    generated_ids: GeneratedIdSequences,
}

impl DocumentSession {
    /// Produce an owned structural serialization of the retained tree.
    pub fn snapshot(&self) -> Result<DocumentSnapshot, DocumentSessionError> {
        let current = self.history.current();
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

    /// Apply one narrow typed operation with optimistic revision control.
    pub fn submit(
        &mut self,
        expected_revision: u64,
        operation: SessionOperation,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
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
    pub fn prepare_create_bond_v1(
        &mut self,
        expected_revision: u64,
        start_atom_object_id: &DocumentObjectIdV1,
        end_atom_object_id: &DocumentObjectIdV1,
        order: DocumentBondOrderV1,
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
            .with_insert_bond(&start_molecule, &bond_id, &start_atom, &end_atom, order)
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
    pub fn prepare_create_bonded_atom_v1(
        &mut self,
        expected_revision: u64,
        start_atom_object_id: &DocumentObjectIdV1,
        element: &str,
        position: Point3V1,
        order: DocumentBondOrderV1,
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
                    order,
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

    pub fn prepare_create_molecule_v1(
        &mut self,
        expected_revision: u64,
        molecule: &MoleculeInsertionV1,
    ) -> Result<PendingCreateMolecule, DocumentSessionError> {
        self.prepare_complete_molecule_candidate(
            expected_revision,
            molecule.atoms().len(),
            molecule.bonds().len(),
            |document, molecule_id, atom_ids, bond_ids| {
                document
                    .with_insert_molecule(molecule_id, atom_ids, bond_ids, molecule)
                    .map_err(SessionOperationError::Candidate)
            },
        )
    }

    fn prepare_complete_molecule_candidate<F>(
        &mut self,
        expected_revision: u64,
        atom_count: usize,
        bond_count: usize,
        writer: F,
    ) -> Result<PendingCreateMolecule, DocumentSessionError>
    where
        F: FnOnce(
            &TypedDocument,
            &PersistentId,
            &[PersistentId],
            &[PersistentId],
        ) -> Result<TypedDocument, SessionOperationError>,
    {
        self.require_current(expected_revision)?;
        let (identities, generated_ids) = self.generated_ids.reserve_molecule(
            self.history.current().document().indexed(),
            atom_count,
            bond_count,
        )?;
        let candidate = writer(
            self.history.current().document(),
            &identities.molecule,
            &identities.atoms,
            &identities.bonds,
        )
        .map_err(DocumentSessionError::Operation)?;
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
        Ok(PendingCreateMolecule {
            revision: expected_revision,
            token,
            molecule_identifier: identities.molecule,
            atom_identifiers: identities.atoms,
            bond_identifiers: identities.bonds,
            candidate: Some(candidate),
        })
    }

    pub fn commit_create_molecule(
        &mut self,
        expected_revision: u64,
        pending: &mut PendingCreateMolecule,
    ) -> Result<SessionOperationResultV1, DocumentSessionError> {
        self.commit_prepared_candidate(
            expected_revision,
            pending.revision,
            &pending.token,
            &mut pending.candidate,
        )
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

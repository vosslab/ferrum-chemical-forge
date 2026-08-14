//! Immutable state values owned by the document transaction session.

use sha2::{Digest, Sha256};

use super::{DocumentSnapshot, TypedDocument, TypedDocumentError};

/// Canonical retained content and its stable digest for one session revision.
#[derive(Debug)]
pub(super) struct RevisionState {
    revision: u64,
    document: TypedDocument,
    canonical_cdml: String,
    digest: [u8; 32],
}

impl RevisionState {
    pub(super) fn from_document(
        revision: u64,
        document: TypedDocument,
    ) -> Result<Self, TypedDocumentError> {
        let canonical_cdml = document.to_xml()?;
        let digest = digest(&canonical_cdml);
        Ok(Self {
            revision,
            document,
            canonical_cdml,
            digest,
        })
    }

    pub(super) fn revision(&self) -> u64 {
        self.revision
    }

    pub(super) fn next_revision(&self) -> Option<u64> {
        self.revision.checked_add(1)
    }

    pub(super) fn document_mut(&mut self) -> &mut TypedDocument {
        &mut self.document
    }

    #[cfg(test)]
    pub(super) fn set_revision_for_test(&mut self, revision: u64) {
        self.revision = revision;
    }
    pub(super) fn document(&self) -> &TypedDocument {
        &self.document
    }
    pub(super) fn canonical_cdml(&self) -> &str {
        &self.canonical_cdml
    }

    pub(super) fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    pub(super) fn snapshot(&self, is_dirty: bool) -> DocumentSnapshot {
        DocumentSnapshot::new(
            self.revision,
            self.canonical_cdml.clone(),
            self.digest,
            is_dirty,
        )
    }
}

/// Saved content is separate from evictable editing history.
#[derive(Clone, Debug)]
pub(super) struct SavedBaseline {
    canonical_cdml: String,
    digest: [u8; 32],
}

impl SavedBaseline {
    pub(super) fn from_state(state: &RevisionState) -> Self {
        Self {
            canonical_cdml: state.canonical_cdml.clone(),
            digest: state.digest,
        }
    }

    pub(super) fn is_current(&self, state: &RevisionState) -> bool {
        self.digest == state.digest && self.canonical_cdml == state.canonical_cdml
    }
}

pub(super) fn digest(content: &str) -> [u8; 32] {
    Sha256::digest(content.as_bytes()).into()
}

//! Closed durable selector for one supported direct-root presentation.

use std::collections::HashSet;

use thiserror::Error;

use super::{DocumentObjectIdV1, PresentationRecordKindV1};

/// One validated durable direct-root presentation selector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationRootSelectorV1 {
    document_object_id: DocumentObjectIdV1,
    kind: PresentationRecordKindV1,
}

impl PresentationRootSelectorV1 {
    /// Select one direct-root presentation record by its document-owned ID.
    #[must_use]
    pub const fn new(
        document_object_id: DocumentObjectIdV1,
        kind: PresentationRecordKindV1,
    ) -> Self {
        Self {
            document_object_id,
            kind,
        }
    }

    /// Return the opaque durable document-object selector.
    #[must_use]
    pub const fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.document_object_id
    }

    /// Return the exact direct-root record kind the caller selected.
    #[must_use]
    pub const fn kind(&self) -> PresentationRecordKindV1 {
        self.kind
    }
}

/// One validated deletion request for a direct-root presentation record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationRootDeletionV1 {
    document_object_id: DocumentObjectIdV1,
    kind: PresentationRecordKindV1,
}

impl PresentationRootDeletionV1 {
    /// Delete one direct-root presentation record by its document-owned ID.
    #[must_use]
    pub const fn new(
        document_object_id: DocumentObjectIdV1,
        kind: PresentationRecordKindV1,
    ) -> Self {
        Self {
            document_object_id,
            kind,
        }
    }

    /// Return the opaque durable document-object selector.
    #[must_use]
    pub const fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.document_object_id
    }

    /// Return the exact direct-root record kind the caller selected.
    #[must_use]
    pub const fn kind(&self) -> PresentationRecordKindV1 {
        self.kind
    }
}

/// Complete validated selector set for one atomic presentation deletion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationRootDeletionSetV1 {
    targets: Vec<PresentationRootDeletionV1>,
}

impl PresentationRootDeletionSetV1 {
    /// Validate a nonempty unique durable target set before document lookup.
    pub fn new(
        targets: Vec<PresentationRootDeletionV1>,
    ) -> Result<Self, PresentationRootDeletionSetV1Error> {
        if targets.is_empty() {
            return Err(PresentationRootDeletionSetV1Error::EmptyTargets);
        }
        let mut identifiers = HashSet::with_capacity(targets.len());
        if targets
            .iter()
            .any(|target| !identifiers.insert(target.document_object_id().clone()))
        {
            return Err(PresentationRootDeletionSetV1Error::DuplicateTarget);
        }
        Ok(Self { targets })
    }

    /// Return exact-kind durable direct-root selectors.
    #[must_use]
    pub fn targets(&self) -> &[PresentationRootDeletionV1] {
        &self.targets
    }
}

/// Invalid multi-root deletion intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PresentationRootDeletionSetV1Error {
    /// No persistent target was supplied.
    #[error("presentation deletion requires at least one target")]
    EmptyTargets,
    /// A durable source ID occurred more than once.
    #[error("presentation deletion targets must be unique")]
    DuplicateTarget,
}

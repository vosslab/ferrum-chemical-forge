//! Closed durable selector for one supported direct-root presentation.

use std::collections::HashSet;

use thiserror::Error;

use super::{PersistentId, PresentationRecordKindV1};

/// One validated durable direct-root presentation selector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationRootSelectorV1 {
    presentation_id: PersistentId,
    kind: PresentationRecordKindV1,
}

impl PresentationRootSelectorV1 {
    /// Validate one durable selector before document lookup.
    pub fn new(
        presentation_id: impl Into<String>,
        kind: PresentationRecordKindV1,
    ) -> Result<Self, PresentationRootSelectorV1Error> {
        let presentation_id = PersistentId::new(presentation_id.into())
            .map_err(|_| PresentationRootSelectorV1Error::InvalidPresentationId)?;
        Ok(Self {
            presentation_id,
            kind,
        })
    }

    /// Return the durable authored direct-root identifier.
    #[must_use]
    pub fn presentation_id(&self) -> &PersistentId {
        &self.presentation_id
    }

    /// Return the exact direct-root record kind the caller selected.
    #[must_use]
    pub const fn kind(&self) -> PresentationRecordKindV1 {
        self.kind
    }
}

/// Invalid presentation-root selector rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PresentationRootSelectorV1Error {
    /// The durable direct-root presentation identifier is invalid.
    #[error("presentation selection requires a valid persistent presentation ID")]
    InvalidPresentationId,
}

/// Compatibility name for the single-root deletion operation's selector.
pub type PresentationRootDeletionV1 = PresentationRootSelectorV1;

/// Compatibility name for invalid single-root deletion selectors.
pub type PresentationRootDeletionV1Error = PresentationRootSelectorV1Error;

/// Complete validated selector set for one atomic presentation deletion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationRootDeletionSetV1 {
    targets: Vec<PresentationRootSelectorV1>,
}

impl PresentationRootDeletionSetV1 {
    /// Validate a nonempty unique durable target set before document lookup.
    pub fn new(
        targets: Vec<PresentationRootSelectorV1>,
    ) -> Result<Self, PresentationRootDeletionSetV1Error> {
        if targets.is_empty() {
            return Err(PresentationRootDeletionSetV1Error::EmptyTargets);
        }
        let mut identifiers = HashSet::with_capacity(targets.len());
        if targets
            .iter()
            .any(|target| !identifiers.insert(target.presentation_id().clone()))
        {
            return Err(PresentationRootDeletionSetV1Error::DuplicateTarget);
        }
        Ok(Self { targets })
    }

    /// Return exact-kind durable direct-root selectors.
    #[must_use]
    pub fn targets(&self) -> &[PresentationRootSelectorV1] {
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

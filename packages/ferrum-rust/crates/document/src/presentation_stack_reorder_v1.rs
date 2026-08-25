//! Closed ordering intent for durable direct-root presentation records.

use std::collections::HashSet;

use thiserror::Error;

use super::PresentationRootSelectorV1;

/// Supported direct-root ordering transformations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationStackOrderV1 {
    /// Retain selected source order after every unselected root element.
    BringToFront,
    /// Retain selected source order before every unselected root element.
    SendToBack,
    /// Reverse selected roots only within their existing element slots.
    ReverseSelectedSlots,
}

/// Complete validated intent for one presentation-stack reorder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationStackReorderV1 {
    order: PresentationStackOrderV1,
    targets: Vec<PresentationRootSelectorV1>,
}

impl PresentationStackReorderV1 {
    /// Validate a nonempty, unique target set before document lookup.
    pub fn new(
        order: PresentationStackOrderV1,
        targets: Vec<PresentationRootSelectorV1>,
    ) -> Result<Self, PresentationStackReorderV1Error> {
        if targets.is_empty() {
            return Err(PresentationStackReorderV1Error::EmptyTargets);
        }
        let mut identifiers = HashSet::with_capacity(targets.len());
        if targets
            .iter()
            .any(|target| !identifiers.insert(target.document_object_id().clone()))
        {
            return Err(PresentationStackReorderV1Error::DuplicateTarget);
        }
        if order == PresentationStackOrderV1::ReverseSelectedSlots && targets.len() < 2 {
            return Err(PresentationStackReorderV1Error::ReverseRequiresTwoTargets);
        }
        Ok(Self { order, targets })
    }

    /// Return the closed ordering transformation.
    #[must_use]
    pub const fn order(&self) -> PresentationStackOrderV1 {
        self.order
    }

    /// Return durable exact-kind target selectors.
    #[must_use]
    pub fn targets(&self) -> &[PresentationRootSelectorV1] {
        &self.targets
    }
}

/// Invalid presentation-stack intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PresentationStackReorderV1Error {
    /// No persistent target was supplied.
    #[error("presentation stack reorder requires at least one target")]
    EmptyTargets,
    /// A durable source ID occurred more than once.
    #[error("presentation stack reorder targets must be unique")]
    DuplicateTarget,
    /// Slot reversal cannot change one target.
    #[error("presentation stack slot reversal requires at least two targets")]
    ReverseRequiresTwoTargets,
}

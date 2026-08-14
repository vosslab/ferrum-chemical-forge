//! Closed common appearance intent for one durable bracket pair.

use std::collections::HashSet;

use thiserror::Error;

use super::{GeometricLineWidthV1, PersistentId, Rgb24V1};

/// One supported common bracket-pair appearance change.
#[derive(Clone, Debug, PartialEq)]
pub enum BracketPropertyChangeV1 {
    /// Replace both sides' visible line width.
    LineWidth(GeometricLineWidthV1),
    /// Replace both sides' visible line colour.
    LineColor(Rgb24V1),
}

impl BracketPropertyChangeV1 {
    fn kind(&self) -> BracketPropertyKindV1 {
        match self {
            Self::LineWidth(_) => BracketPropertyKindV1::LineWidth,
            Self::LineColor(_) => BracketPropertyKindV1::LineColor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum BracketPropertyKindV1 {
    LineWidth,
    LineColor,
}

impl BracketPropertyKindV1 {
    fn name(self) -> &'static str {
        match self {
            Self::LineWidth => "line width",
            Self::LineColor => "line color",
        }
    }
}

/// One validated pair-ID-targeted bracket appearance patch.
#[derive(Clone, Debug, PartialEq)]
pub struct BracketPropertiesPatchV1 {
    pair_id: PersistentId,
    changes: Vec<BracketPropertyChangeV1>,
}

impl BracketPropertiesPatchV1 {
    /// Validate one complete two-field edit intent without reading a document.
    pub fn new(
        pair_id: impl Into<String>,
        changes: Vec<BracketPropertyChangeV1>,
    ) -> Result<Self, BracketPropertiesPatchV1Error> {
        let pair_id = PersistentId::new(pair_id.into())
            .map_err(|_| BracketPropertiesPatchV1Error::InvalidPairId)?;
        if changes.len() > 2 {
            return Err(BracketPropertiesPatchV1Error::TooManyChanges);
        }
        let mut kinds = HashSet::with_capacity(changes.len());
        for change in &changes {
            let kind = change.kind();
            if !kinds.insert(kind) {
                return Err(BracketPropertiesPatchV1Error::DuplicateChange {
                    property: kind.name(),
                });
            }
        }
        Ok(Self { pair_id, changes })
    }

    /// Return the durable pair identifier, which is the left member ID.
    #[must_use]
    pub fn pair_id(&self) -> &PersistentId {
        &self.pair_id
    }

    /// Return unique common appearance changes in caller order.
    #[must_use]
    pub fn changes(&self) -> &[BracketPropertyChangeV1] {
        &self.changes
    }
}

/// Invalid bracket-pair appearance intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum BracketPropertiesPatchV1Error {
    /// The durable pair identifier is invalid.
    #[error("bracket properties require a valid persistent pair ID")]
    InvalidPairId,
    /// A request exceeded the two-field closed grammar.
    #[error("bracket properties accept at most two changes")]
    TooManyChanges,
    /// One closed property appeared more than once in one patch.
    #[error("bracket property change is duplicated: {property}")]
    DuplicateChange { property: &'static str },
}

//! Closed appearance intent for one durable Wavy presentation root.

use std::collections::HashSet;

use thiserror::Error;

use super::{DocumentObjectIdV1, GeometricLineWidthV1, Rgb24V1};

/// One supported Wavy root appearance change.
#[derive(Clone, Debug, PartialEq)]
pub enum WavyPropertyChangeV1 {
    /// Replace the visible line width.
    LineWidth(GeometricLineWidthV1),
    /// Replace the visible line colour.
    LineColor(Rgb24V1),
}

impl WavyPropertyChangeV1 {
    fn kind(&self) -> WavyPropertyKindV1 {
        match self {
            Self::LineWidth(_) => WavyPropertyKindV1::LineWidth,
            Self::LineColor(_) => WavyPropertyKindV1::LineColor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum WavyPropertyKindV1 {
    LineWidth,
    LineColor,
}

impl WavyPropertyKindV1 {
    fn name(self) -> &'static str {
        match self {
            Self::LineWidth => "line width",
            Self::LineColor => "line color",
        }
    }
}

/// One validated durable-ID-targeted Wavy appearance patch.
#[derive(Clone, Debug, PartialEq)]
pub struct WavyPropertiesPatchV1 {
    wavy_id: DocumentObjectIdV1,
    changes: Vec<WavyPropertyChangeV1>,
}

impl WavyPropertiesPatchV1 {
    /// Validate one complete two-field edit intent without reading a document.
    pub fn new(
        wavy_id: DocumentObjectIdV1,
        changes: Vec<WavyPropertyChangeV1>,
    ) -> Result<Self, WavyPropertiesPatchV1Error> {
        if changes.len() > 2 {
            return Err(WavyPropertiesPatchV1Error::TooManyChanges);
        }
        let mut kinds = HashSet::with_capacity(changes.len());
        for change in &changes {
            let kind = change.kind();
            if !kinds.insert(kind) {
                return Err(WavyPropertiesPatchV1Error::DuplicateChange {
                    property: kind.name(),
                });
            }
        }
        Ok(Self { wavy_id, changes })
    }

    /// Return the durable Wavy selector.
    #[must_use]
    pub fn wavy_id(&self) -> &DocumentObjectIdV1 {
        &self.wavy_id
    }

    /// Return unique property changes in caller order.
    #[must_use]
    pub fn changes(&self) -> &[WavyPropertyChangeV1] {
        &self.changes
    }
}

/// Invalid Wavy appearance intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum WavyPropertiesPatchV1Error {
    /// A request exceeded the two-field closed grammar.
    #[error("Wavy properties accept at most two changes")]
    TooManyChanges,
    /// One closed property appeared more than once in one patch.
    #[error("Wavy property change is duplicated: {property}")]
    DuplicateChange { property: &'static str },
}

//! Closed, validated Arrow-properties intent for one atomic session operation.

use std::collections::HashSet;

use thiserror::Error;

use super::{PersistentId, Rgb24V1};

/// A finite Arrow line width in the documented editable range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArrowLineWidthV1(f64);

impl ArrowLineWidthV1 {
    /// Construct a width from 0.1 through 20 scene points.
    pub fn new(value: f64) -> Option<Self> {
        (value.is_finite() && (0.1..=20.0).contains(&value)).then_some(Self(value))
    }

    /// Return the validated scalar.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

/// One supported durable direct-root Arrow property change.
#[derive(Clone, Debug, PartialEq)]
pub enum ArrowPropertyChangeV1 {
    /// Replace start-head visibility.
    StartHead(bool),
    /// Replace end-head visibility.
    EndHead(bool),
    /// Replace spline interpolation intent.
    Spline(bool),
    /// Replace the finite root line width.
    LineWidth(ArrowLineWidthV1),
    /// Replace the root line colour.
    Color(Rgb24V1),
}

impl ArrowPropertyChangeV1 {
    fn kind(&self) -> ArrowPropertyKindV1 {
        match self {
            Self::StartHead(_) => ArrowPropertyKindV1::StartHead,
            Self::EndHead(_) => ArrowPropertyKindV1::EndHead,
            Self::Spline(_) => ArrowPropertyKindV1::Spline,
            Self::LineWidth(_) => ArrowPropertyKindV1::LineWidth,
            Self::Color(_) => ArrowPropertyKindV1::Color,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ArrowPropertyKindV1 {
    StartHead,
    EndHead,
    Spline,
    LineWidth,
    Color,
}

impl ArrowPropertyKindV1 {
    fn name(self) -> &'static str {
        match self {
            Self::StartHead => "start head",
            Self::EndHead => "end head",
            Self::Spline => "spline",
            Self::LineWidth => "line width",
            Self::Color => "color",
        }
    }
}

/// One validated, source-ID-targeted direct-root Arrow properties patch.
#[derive(Clone, Debug, PartialEq)]
pub struct ArrowPropertiesPatchV1 {
    arrow_id: PersistentId,
    changes: Vec<ArrowPropertyChangeV1>,
}

impl ArrowPropertiesPatchV1 {
    /// Validate one complete edit intent without reading a document.
    pub fn new(
        arrow_id: impl Into<String>,
        changes: Vec<ArrowPropertyChangeV1>,
    ) -> Result<Self, ArrowPropertiesPatchV1Error> {
        let arrow_id = PersistentId::new(arrow_id.into())
            .map_err(|_| ArrowPropertiesPatchV1Error::InvalidArrowId)?;
        let mut kinds = HashSet::with_capacity(changes.len());
        for change in &changes {
            let kind = change.kind();
            if !kinds.insert(kind) {
                return Err(ArrowPropertiesPatchV1Error::DuplicateChange {
                    property: kind.name(),
                });
            }
        }
        Ok(Self { arrow_id, changes })
    }

    /// Return the durable authored direct-root Arrow identifier.
    #[must_use]
    pub fn arrow_id(&self) -> &PersistentId {
        &self.arrow_id
    }

    /// Return unique property changes in caller order.
    #[must_use]
    pub fn changes(&self) -> &[ArrowPropertyChangeV1] {
        &self.changes
    }
}

/// Invalid Arrow-properties intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ArrowPropertiesPatchV1Error {
    /// The durable direct-root Arrow identifier is invalid.
    #[error("Arrow properties require a valid persistent Arrow ID")]
    InvalidArrowId,
    /// One closed property appeared more than once in one patch.
    #[error("Arrow property change is duplicated: {property}")]
    DuplicateChange { property: &'static str },
}

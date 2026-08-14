//! Closed appearance intent for one durable geometric presentation root.

use std::collections::HashSet;

use thiserror::Error;

use super::{PersistentId, Rgb24V1};

/// A finite geometric line width in the documented editable range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeometricLineWidthV1(f64);

impl GeometricLineWidthV1 {
    /// Construct a line width from 0.1 through 20 scene points.
    pub fn new(value: f64) -> Option<Self> {
        (value.is_finite() && (0.1..=20.0).contains(&value)).then_some(Self(value))
    }

    /// Return the validated scalar.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

/// One supported geometric presentation appearance change.
#[derive(Clone, Debug, PartialEq)]
pub enum GeometricPropertyChangeV1 {
    /// Replace the root line width.
    LineWidth(GeometricLineWidthV1),
    /// Replace the root stroke colour.
    StrokeColor(Rgb24V1),
    /// Replace the root fill colour, or author explicit no-fill.
    FillColor(Option<Rgb24V1>),
}

impl GeometricPropertyChangeV1 {
    fn kind(&self) -> GeometricPropertyKindV1 {
        match self {
            Self::LineWidth(_) => GeometricPropertyKindV1::LineWidth,
            Self::StrokeColor(_) => GeometricPropertyKindV1::StrokeColor,
            Self::FillColor(_) => GeometricPropertyKindV1::FillColor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum GeometricPropertyKindV1 {
    LineWidth,
    StrokeColor,
    FillColor,
}

impl GeometricPropertyKindV1 {
    fn name(self) -> &'static str {
        match self {
            Self::LineWidth => "line width",
            Self::StrokeColor => "stroke color",
            Self::FillColor => "fill color",
        }
    }
}

/// One validated source-ID-targeted geometric appearance patch.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometricPropertiesPatchV1 {
    presentation_id: PersistentId,
    changes: Vec<GeometricPropertyChangeV1>,
}

impl GeometricPropertiesPatchV1 {
    /// Validate one complete edit intent without reading a document.
    pub fn new(
        presentation_id: impl Into<String>,
        changes: Vec<GeometricPropertyChangeV1>,
    ) -> Result<Self, GeometricPropertiesPatchV1Error> {
        let presentation_id = PersistentId::new(presentation_id.into())
            .map_err(|_| GeometricPropertiesPatchV1Error::InvalidPresentationId)?;
        if changes.len() > 3 {
            return Err(GeometricPropertiesPatchV1Error::TooManyChanges);
        }
        let mut kinds = HashSet::with_capacity(changes.len());
        for change in &changes {
            let kind = change.kind();
            if !kinds.insert(kind) {
                return Err(GeometricPropertiesPatchV1Error::DuplicateChange {
                    property: kind.name(),
                });
            }
        }
        Ok(Self {
            presentation_id,
            changes,
        })
    }

    /// Return the durable authored direct-root presentation identifier.
    #[must_use]
    pub fn presentation_id(&self) -> &PersistentId {
        &self.presentation_id
    }

    /// Return unique property changes in caller order.
    #[must_use]
    pub fn changes(&self) -> &[GeometricPropertyChangeV1] {
        &self.changes
    }
}

/// Invalid geometric appearance intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum GeometricPropertiesPatchV1Error {
    /// The durable direct-root presentation identifier is invalid.
    #[error("geometric properties require a valid persistent presentation ID")]
    InvalidPresentationId,
    /// A request exceeded the three-field closed grammar.
    #[error("geometric properties accept at most three changes")]
    TooManyChanges,
    /// One closed property appeared more than once in one patch.
    #[error("geometric property change is duplicated: {property}")]
    DuplicateChange { property: &'static str },
}

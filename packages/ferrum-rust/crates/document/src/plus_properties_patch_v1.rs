//! Closed, validated plus-properties intent for one atomic session operation.

use std::collections::HashSet;

use thiserror::Error;

use super::{DocumentObjectIdV1, PresentationFontFaceV1, Rgb24V1};

/// Minimum editable Plus font size documented by the CDML V1 contract.
pub const MIN_PLUS_FONT_SIZE_V1: u16 = 4;
/// Maximum editable Plus font size documented by the CDML V1 contract.
pub const MAX_PLUS_FONT_SIZE_V1: u16 = 144;

/// One supported durable direct-root Plus property change.
#[derive(Clone, Debug, PartialEq)]
pub enum PlusPropertyChangeV1 {
    /// Select the closed semantic face emitted with its canonical CDML spelling.
    FontFace(PresentationFontFaceV1),
    /// Replace the root-authoritative integer font size.
    FontSize(u16),
    /// Replace the root-authoritative foreground colour.
    Color(Rgb24V1),
    /// Replace the root background, or author explicit transparency.
    BackgroundColor(Option<Rgb24V1>),
}

impl PlusPropertyChangeV1 {
    fn kind(&self) -> PlusPropertyKindV1 {
        match self {
            Self::FontFace(_) => PlusPropertyKindV1::FontFace,
            Self::FontSize(_) => PlusPropertyKindV1::FontSize,
            Self::Color(_) => PlusPropertyKindV1::Color,
            Self::BackgroundColor(_) => PlusPropertyKindV1::BackgroundColor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum PlusPropertyKindV1 {
    FontFace,
    FontSize,
    Color,
    BackgroundColor,
}

impl PlusPropertyKindV1 {
    fn name(self) -> &'static str {
        match self {
            Self::FontFace => "font face",
            Self::FontSize => "font size",
            Self::Color => "color",
            Self::BackgroundColor => "background color",
        }
    }
}

/// One validated, durable-ID-targeted direct-root Plus properties patch.
#[derive(Clone, Debug, PartialEq)]
pub struct PlusPropertiesPatchV1 {
    plus_object_id: DocumentObjectIdV1,
    changes: Vec<PlusPropertyChangeV1>,
}

impl PlusPropertiesPatchV1 {
    /// Validate and normalize one complete edit intent without reading a document.
    pub fn new(
        plus_object_id: DocumentObjectIdV1,
        mut changes: Vec<PlusPropertyChangeV1>,
    ) -> Result<Self, PlusPropertiesPatchV1Error> {
        let mut kinds = HashSet::with_capacity(changes.len());
        for change in &mut changes {
            let kind = change.kind();
            if !kinds.insert(kind) {
                return Err(PlusPropertiesPatchV1Error::DuplicateChange {
                    property: kind.name(),
                });
            }
            match change {
                PlusPropertyChangeV1::FontSize(value)
                    if !(MIN_PLUS_FONT_SIZE_V1..=MAX_PLUS_FONT_SIZE_V1).contains(value) =>
                {
                    return Err(PlusPropertiesPatchV1Error::FontSizeOutOfRange);
                }
                _ => {}
            }
        }
        Ok(Self {
            plus_object_id,
            changes,
        })
    }

    /// Return the durable authored direct-root Plus object identifier.
    #[must_use]
    pub fn plus_object_id(&self) -> &DocumentObjectIdV1 {
        &self.plus_object_id
    }

    /// Return unique normalized property changes in caller order.
    #[must_use]
    pub fn changes(&self) -> &[PlusPropertyChangeV1] {
        &self.changes
    }
}

/// Invalid Plus-properties intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PlusPropertiesPatchV1Error {
    /// Font size falls outside the documented editable CDML range.
    #[error("Plus font size must be from 4 through 144")]
    FontSizeOutOfRange,
    /// One closed property appeared more than once in one patch.
    #[error("Plus property change is duplicated: {property}")]
    DuplicateChange { property: &'static str },
}

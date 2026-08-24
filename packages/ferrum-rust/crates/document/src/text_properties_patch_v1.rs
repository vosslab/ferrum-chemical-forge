//! Closed, validated direct-root Text editing intent.

use std::collections::HashSet;

use thiserror::Error;

use super::{
    AuthoredTextRunV1, AuthoredTextStyleV1, PersistentId, PresentationFontFaceV1, Rgb24V1,
    normalize_authored_text_runs_v1,
};

/// Minimum Text font size represented by the established CDML editor control.
pub const MIN_TEXT_FONT_SIZE_V1: u16 = 4;
/// Maximum Text font size represented by the established CDML editor control.
pub const MAX_TEXT_FONT_SIZE_V1: u16 = 144;

/// Compatibility aliases for the pre-existing Text-properties Python surface.
/// New document contracts use the source-neutral AuthoredText names directly.
pub type TextEditStyleV1 = AuthoredTextStyleV1;
pub type TextEditRunV1 = AuthoredTextRunV1;

/// One supported durable direct-root Text property change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextPropertyChangeV1 {
    /// Replace the complete formatted character run sequence.
    Runs(Vec<TextEditRunV1>),
    /// Select the closed semantic face emitted with its canonical CDML spelling.
    FontFace(PresentationFontFaceV1),
    /// Replace the direct font's integer size.
    FontSize(u16),
    /// Replace the direct font's foreground colour.
    Color(Rgb24V1),
    /// Replace the root background, or author explicit transparency.
    BackgroundColor(Option<Rgb24V1>),
}

impl TextPropertyChangeV1 {
    fn kind(&self) -> TextPropertyKindV1 {
        match self {
            Self::Runs(_) => TextPropertyKindV1::Runs,
            Self::FontFace(_) => TextPropertyKindV1::FontFace,
            Self::FontSize(_) => TextPropertyKindV1::FontSize,
            Self::Color(_) => TextPropertyKindV1::Color,
            Self::BackgroundColor(_) => TextPropertyKindV1::BackgroundColor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TextPropertyKindV1 {
    Runs,
    FontFace,
    FontSize,
    Color,
    BackgroundColor,
}

impl TextPropertyKindV1 {
    const fn name(self) -> &'static str {
        match self {
            Self::Runs => "runs",
            Self::FontFace => "font face",
            Self::FontSize => "font size",
            Self::Color => "color",
            Self::BackgroundColor => "background color",
        }
    }
}

/// One validated source-ID-targeted direct-root Text properties patch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextPropertiesPatchV1 {
    text_id: PersistentId,
    changes: Vec<TextPropertyChangeV1>,
}

impl TextPropertiesPatchV1 {
    /// Validate and normalize one complete edit intent without reading a document.
    pub fn new(
        text_id: impl Into<String>,
        mut changes: Vec<TextPropertyChangeV1>,
    ) -> Result<Self, TextPropertiesPatchV1Error> {
        let text_id = PersistentId::new(text_id.into())
            .map_err(|_| TextPropertiesPatchV1Error::InvalidTextId)?;
        let mut kinds = HashSet::with_capacity(changes.len());
        for change in &mut changes {
            let kind = change.kind();
            if !kinds.insert(kind) {
                return Err(TextPropertiesPatchV1Error::DuplicateChange {
                    property: kind.name(),
                });
            }
            match change {
                TextPropertyChangeV1::Runs(runs) => normalize_runs(runs)?,
                TextPropertyChangeV1::FontSize(value)
                    if !(MIN_TEXT_FONT_SIZE_V1..=MAX_TEXT_FONT_SIZE_V1).contains(value) =>
                {
                    return Err(TextPropertiesPatchV1Error::FontSizeOutOfRange);
                }
                _ => {}
            }
        }
        Ok(Self { text_id, changes })
    }

    /// Return the durable authored direct-root Text identifier.
    #[must_use]
    pub fn text_id(&self) -> &PersistentId {
        &self.text_id
    }

    /// Return unique normalized changes in caller order.
    #[must_use]
    pub fn changes(&self) -> &[TextPropertyChangeV1] {
        &self.changes
    }
}

fn normalize_runs(runs: &mut Vec<TextEditRunV1>) -> Result<(), TextPropertiesPatchV1Error> {
    normalize_authored_text_runs_v1(runs)
}

/// Invalid Text-properties intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TextPropertiesPatchV1Error {
    /// The durable direct-root Text identifier is invalid.
    #[error("Text properties require a valid persistent Text ID")]
    InvalidTextId,
    /// One formatted run was empty.
    #[error("Text formatted runs must not be empty")]
    EmptyRun,
    /// A run contained a control other than the supported newline.
    #[error("Text formatted runs do not support this control character")]
    UnsupportedControlCharacter,
    /// One style occurred more than once on a run.
    #[error("Text formatted run styles must be unique")]
    DuplicateRunStyle,
    /// A run combined both vertical script directions.
    #[error("Text formatted runs cannot combine subscript and superscript")]
    ConflictingScriptStyles,
    /// The complete replacement has no visible character.
    #[error("Text content must contain a visible character")]
    BlankText,
    /// Font size falls outside the established editable CDML range.
    #[error("Text font size must be from 4 through 144")]
    FontSizeOutOfRange,
    /// One closed property appeared more than once in one patch.
    #[error("Text property change is duplicated: {property}")]
    DuplicateChange { property: &'static str },
}

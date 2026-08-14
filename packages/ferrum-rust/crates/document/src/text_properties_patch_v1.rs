//! Closed, validated direct-root Text editing intent.

use std::collections::HashSet;

use thiserror::Error;

use super::{PersistentId, Rgb24V1};

/// Minimum Text font size represented by the established CDML editor control.
pub const MIN_TEXT_FONT_SIZE_V1: u16 = 4;
/// Maximum Text font size represented by the established CDML editor control.
pub const MAX_TEXT_FONT_SIZE_V1: u16 = 144;

/// One closed formatted-text style accepted by an edit run.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TextEditStyleV1 {
    /// Bold source intent.
    Bold,
    /// Italic source intent.
    Italic,
    /// Lowered script.
    Subscript,
    /// Raised script.
    Superscript,
}

/// One nonempty character-data run and its canonical unique styles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextEditRunV1 {
    text: String,
    styles: Vec<TextEditStyleV1>,
}

impl TextEditRunV1 {
    /// Validate one run before it can enter a document operation.
    pub fn new(
        text: impl Into<String>,
        mut styles: Vec<TextEditStyleV1>,
    ) -> Result<Self, TextPropertiesPatchV1Error> {
        let text = text.into();
        if text.is_empty() {
            return Err(TextPropertiesPatchV1Error::EmptyRun);
        }
        if text
            .chars()
            .any(|character| character.is_control() && character != '\n')
        {
            return Err(TextPropertiesPatchV1Error::UnsupportedControlCharacter);
        }
        styles.sort_unstable();
        if styles.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(TextPropertiesPatchV1Error::DuplicateRunStyle);
        }
        if styles.contains(&TextEditStyleV1::Subscript)
            && styles.contains(&TextEditStyleV1::Superscript)
        {
            return Err(TextPropertiesPatchV1Error::ConflictingScriptStyles);
        }
        Ok(Self { text, styles })
    }

    /// Return literal rendered character data.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return styles in canonical bold, italic, subscript, superscript order.
    #[must_use]
    pub fn styles(&self) -> &[TextEditStyleV1] {
        &self.styles
    }
}

/// One supported durable direct-root Text property change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TextPropertyChangeV1 {
    /// Replace the complete formatted character run sequence.
    Runs(Vec<TextEditRunV1>),
    /// Replace or clear the optional direct font family.
    FontFamily(Option<String>),
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
            Self::FontFamily(_) => TextPropertyKindV1::FontFamily,
            Self::FontSize(_) => TextPropertyKindV1::FontSize,
            Self::Color(_) => TextPropertyKindV1::Color,
            Self::BackgroundColor(_) => TextPropertyKindV1::BackgroundColor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum TextPropertyKindV1 {
    Runs,
    FontFamily,
    FontSize,
    Color,
    BackgroundColor,
}

impl TextPropertyKindV1 {
    const fn name(self) -> &'static str {
        match self {
            Self::Runs => "runs",
            Self::FontFamily => "font family",
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
                TextPropertyChangeV1::FontFamily(Some(value)) => {
                    let trimmed = value.trim();
                    if trimmed.is_empty() {
                        return Err(TextPropertiesPatchV1Error::BlankFontFamily);
                    }
                    *value = trimmed.to_owned();
                }
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
    if runs.is_empty()
        || !runs
            .iter()
            .flat_map(|run| run.text.chars())
            .any(|character| !character.is_whitespace())
    {
        return Err(TextPropertiesPatchV1Error::BlankText);
    }
    let mut normalized: Vec<TextEditRunV1> = Vec::with_capacity(runs.len());
    for run in runs.drain(..) {
        if let Some(previous) = normalized
            .last_mut()
            .filter(|previous| previous.styles == run.styles)
        {
            previous.text.push_str(&run.text);
        } else {
            normalized.push(run);
        }
    }
    *runs = normalized;
    Ok(())
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
    /// Font family cannot be blank after trimming.
    #[error("Text font family must not be blank")]
    BlankFontFamily,
    /// Font size falls outside the established editable CDML range.
    #[error("Text font size must be from 4 through 144")]
    FontSizeOutOfRange,
    /// One closed property appeared more than once in one patch.
    #[error("Text property change is duplicated: {property}")]
    DuplicateChange { property: &'static str },
}

//! Closed document drawing-standard mutation values.

use thiserror::Error;

use super::Rgb24V1;

/// Maximum accepted scene-point width for a document drawing standard.
pub const MAX_DRAWING_STANDARD_WIDTH_V1: f64 = 1000.0;

/// Minimum accepted integer font size for a document drawing standard.
pub const MIN_DRAWING_STANDARD_FONT_SIZE_V1: u16 = 4;

/// Maximum accepted integer font size for a document drawing standard.
pub const MAX_DRAWING_STANDARD_FONT_SIZE_V1: u16 = 144;

/// Maximum UTF-8 bytes retained in a document drawing-standard font family.
pub const MAX_DRAWING_STANDARD_FONT_FAMILY_BYTES_V1: usize = 128;

/// One explicit document drawing-standard field replacement.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawingStandardPropertyChangeV1 {
    /// Replace the default line width in scene points.
    LineWidth(f64),
    /// Replace the default atom-label font size.
    FontSize(u16),
    /// Replace the default atom-label font family.
    FontFamily(String),
    /// Replace the default line and text colour.
    LineColor(Rgb24V1),
    /// Replace the default label mask; `None` means transparent.
    AreaColor(Option<Rgb24V1>),
    /// Replace the default spacing between parallel bond lanes.
    BondWidth(f64),
    /// Replace the default wedge width.
    WedgeWidth(f64),
    /// Replace the shorter double-bond line ratio.
    DoubleRatio(f64),
    /// Replace the heteroatom hydrogen-display default.
    ShowHydrogens(bool),
}

impl DrawingStandardPropertyChangeV1 {
    const fn field_bit(&self) -> u16 {
        match self {
            Self::LineWidth(_) => 1 << 0,
            Self::FontSize(_) => 1 << 1,
            Self::FontFamily(_) => 1 << 2,
            Self::LineColor(_) => 1 << 3,
            Self::AreaColor(_) => 1 << 4,
            Self::BondWidth(_) => 1 << 5,
            Self::WedgeWidth(_) => 1 << 6,
            Self::DoubleRatio(_) => 1 << 7,
            Self::ShowHydrogens(_) => 1 << 8,
        }
    }
}

/// Complete unique-field document drawing-standard patch.
#[derive(Clone, Debug, PartialEq)]
pub struct DrawingStandardPatchV1 {
    changes: Vec<DrawingStandardPropertyChangeV1>,
}

impl DrawingStandardPatchV1 {
    /// Validate and normalize one ordered explicit-field patch.
    pub fn new(
        mut changes: Vec<DrawingStandardPropertyChangeV1>,
    ) -> Result<Self, DrawingStandardPatchV1Error> {
        let mut fields = 0_u16;
        for change in &mut changes {
            let bit = change.field_bit();
            if fields & bit != 0 {
                return Err(DrawingStandardPatchV1Error::DuplicateChange);
            }
            fields |= bit;
            match change {
                DrawingStandardPropertyChangeV1::LineWidth(value)
                | DrawingStandardPropertyChangeV1::BondWidth(value)
                | DrawingStandardPropertyChangeV1::WedgeWidth(value) => {
                    if !value.is_finite() || *value <= 0.0 || *value > MAX_DRAWING_STANDARD_WIDTH_V1
                    {
                        return Err(DrawingStandardPatchV1Error::WidthOutOfRange);
                    }
                }
                DrawingStandardPropertyChangeV1::FontSize(value) => {
                    if !(MIN_DRAWING_STANDARD_FONT_SIZE_V1..=MAX_DRAWING_STANDARD_FONT_SIZE_V1)
                        .contains(value)
                    {
                        return Err(DrawingStandardPatchV1Error::FontSizeOutOfRange);
                    }
                }
                DrawingStandardPropertyChangeV1::FontFamily(value) => {
                    trim_in_place(value);
                    if value.is_empty() || value.len() > MAX_DRAWING_STANDARD_FONT_FAMILY_BYTES_V1 {
                        return Err(DrawingStandardPatchV1Error::InvalidFontFamily);
                    }
                }
                DrawingStandardPropertyChangeV1::DoubleRatio(value) => {
                    if !value.is_finite() || *value <= 0.0 || *value > 1.0 {
                        return Err(DrawingStandardPatchV1Error::DoubleRatioOutOfRange);
                    }
                }
                DrawingStandardPropertyChangeV1::LineColor(_)
                | DrawingStandardPropertyChangeV1::AreaColor(_)
                | DrawingStandardPropertyChangeV1::ShowHydrogens(_) => {}
            }
        }
        Ok(Self { changes })
    }

    /// Return the validated changes in caller order.
    #[must_use]
    pub fn changes(&self) -> &[DrawingStandardPropertyChangeV1] {
        &self.changes
    }
}

/// Validation failures for a document drawing-standard patch.
#[derive(Clone, Debug, Error, PartialEq)]
pub enum DrawingStandardPatchV1Error {
    /// One field occurs more than once.
    #[error("drawing-standard property is repeated")]
    DuplicateChange,
    /// A width is nonfinite, nonpositive, or above the closed V1 range.
    #[error("drawing-standard width must be above 0 and at most 1000")]
    WidthOutOfRange,
    /// Font size is outside the closed form range.
    #[error("drawing-standard font size must be from 4 to 144")]
    FontSizeOutOfRange,
    /// Font family is blank or exceeds the closed UTF-8 byte budget.
    #[error("drawing-standard font family must be nonblank and at most 128 UTF-8 bytes")]
    InvalidFontFamily,
    /// Double-line ratio is nonfinite or outside `(0, 1]`.
    #[error("drawing-standard double ratio must be above 0 and at most 1")]
    DoubleRatioOutOfRange,
}

fn trim_in_place(value: &mut String) {
    let trimmed = value.trim();
    let start = trimmed.as_ptr() as usize - value.as_ptr() as usize;
    let end = start + trimmed.len();
    value.truncate(end);
    if start != 0 {
        value.drain(..start);
    }
}

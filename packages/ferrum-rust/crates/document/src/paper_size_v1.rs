//! Closed physical dimensions for CDML's recognized paper-size names.

use thiserror::Error;

/// Positive physical paper dimensions in millimetres, before orientation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaperDimensionsMmV1 {
    width: f64,
    height: f64,
}

impl PaperDimensionsMmV1 {
    const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Validate one caller-authored custom paper size in millimetres.
    pub fn try_new(width: f64, height: f64) -> Result<Self, PaperDimensionsMmV1Error> {
        if !width.is_finite() || !height.is_finite() {
            return Err(PaperDimensionsMmV1Error::NonFinite);
        }
        if width <= 0.0 || height <= 0.0 {
            return Err(PaperDimensionsMmV1Error::NonPositive);
        }
        Ok(Self { width, height })
    }

    /// Return the portrait width in millimetres.
    #[must_use]
    pub const fn width(self) -> f64 {
        self.width
    }

    /// Return the portrait height in millimetres.
    #[must_use]
    pub const fn height(self) -> f64 {
        self.height
    }
}

/// Invalid caller-authored custom paper dimensions.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PaperDimensionsMmV1Error {
    /// At least one dimension is NaN or infinite.
    #[error("custom paper dimensions must be finite")]
    NonFinite,
    /// At least one dimension is zero or negative.
    #[error("custom paper dimensions must be positive")]
    NonPositive,
}

/// One exact CDML paper-size name and its optional fixed dimensions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaperSizeV1 {
    name: &'static str,
    dimensions: Option<PaperDimensionsMmV1>,
}

impl PaperSizeV1 {
    const fn fixed(name: &'static str, width: f64, height: f64) -> Self {
        Self {
            name,
            dimensions: Some(PaperDimensionsMmV1::new(width, height)),
        }
    }

    const fn custom() -> Self {
        Self {
            name: "custom",
            dimensions: None,
        }
    }

    /// Return the exact case-sensitive CDML paper-size name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// Return fixed portrait dimensions, or `None` for caller-supplied custom size.
    #[must_use]
    pub const fn dimensions(self) -> Option<PaperDimensionsMmV1> {
        self.dimensions
    }
}

const PAPER_SIZE_CATALOG_V1: [PaperSizeV1; 38] = [
    PaperSizeV1::fixed("A0", 841.0, 1189.0),
    PaperSizeV1::fixed("A1", 594.0, 841.0),
    PaperSizeV1::fixed("A2", 420.0, 594.0),
    PaperSizeV1::fixed("A3", 297.0, 420.0),
    PaperSizeV1::fixed("A4", 210.0, 297.0),
    PaperSizeV1::fixed("A5", 148.0, 210.0),
    PaperSizeV1::fixed("A6", 105.0, 148.0),
    PaperSizeV1::fixed("A7", 74.0, 105.0),
    PaperSizeV1::fixed("A8", 52.0, 74.0),
    PaperSizeV1::fixed("A9", 37.0, 52.0),
    PaperSizeV1::fixed("A10", 26.0, 37.0),
    PaperSizeV1::fixed("B0", 1000.0, 1414.0),
    PaperSizeV1::fixed("B1", 707.0, 1000.0),
    PaperSizeV1::fixed("B2", 500.0, 707.0),
    PaperSizeV1::fixed("B3", 353.0, 500.0),
    PaperSizeV1::fixed("B4", 250.0, 353.0),
    PaperSizeV1::fixed("B5", 176.0, 250.0),
    PaperSizeV1::fixed("B6", 125.0, 176.0),
    PaperSizeV1::fixed("B7", 88.0, 125.0),
    PaperSizeV1::fixed("B8", 62.0, 88.0),
    PaperSizeV1::fixed("B9", 44.0, 62.0),
    PaperSizeV1::fixed("B10", 31.0, 44.0),
    PaperSizeV1::fixed("C0", 917.0, 1297.0),
    PaperSizeV1::fixed("C1", 648.0, 917.0),
    PaperSizeV1::fixed("C2", 458.0, 648.0),
    PaperSizeV1::fixed("C3", 324.0, 458.0),
    PaperSizeV1::fixed("C4", 229.0, 324.0),
    PaperSizeV1::fixed("C5", 162.0, 229.0),
    PaperSizeV1::fixed("C6", 114.0, 162.0),
    PaperSizeV1::fixed("C7", 81.0, 114.0),
    PaperSizeV1::fixed("C8", 57.0, 81.0),
    PaperSizeV1::fixed("C9", 40.0, 57.0),
    PaperSizeV1::fixed("C10", 28.0, 40.0),
    PaperSizeV1::fixed("Ledger", 279.4, 431.8),
    PaperSizeV1::fixed("Legal", 215.9, 355.6),
    PaperSizeV1::fixed("Letter", 215.9, 279.4),
    PaperSizeV1::fixed("Tabloid", 279.4, 431.8),
    PaperSizeV1::custom(),
];

/// Return every exact recognized CDML paper-size entry in established order.
#[must_use]
pub fn paper_size_catalog_v1() -> &'static [PaperSizeV1] {
    &PAPER_SIZE_CATALOG_V1
}

/// Resolve one exact case-sensitive CDML paper-size name.
#[must_use]
pub fn paper_size_v1(name: &str) -> Option<PaperSizeV1> {
    PAPER_SIZE_CATALOG_V1
        .iter()
        .copied()
        .find(|entry| entry.name == name)
}

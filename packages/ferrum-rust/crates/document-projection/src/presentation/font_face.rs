//! Closed semantic font-face identities for presentation content.

use serde::{Deserialize, Serialize};

/// The bundled face identity supported by the V1 presentation contract.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationFontFaceV1 {
    /// The verified bundled Telex Regular resource.
    TelexRegularV1,
}

impl PresentationFontFaceV1 {
    /// Stable public identifier for API and UI transport.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::TelexRegularV1 => "telex_regular_v1",
        }
    }

    /// Canonical CDML family spelling when a family attribute is emitted.
    #[must_use]
    pub const fn cdml_family(self) -> &'static str {
        match self {
            Self::TelexRegularV1 => "Telex",
        }
    }

    /// Parse the stable public identity without accepting system-family names.
    #[must_use]
    pub fn from_id(value: &str) -> Option<Self> {
        (value == Self::TelexRegularV1.id()).then_some(Self::TelexRegularV1)
    }

    /// Normalize the finite approved CDML aliases.
    #[must_use]
    pub fn from_cdml_family(value: &str) -> Option<Self> {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "telex" | "telex regular" | "ferrum-telex-regular-v1"
        )
        .then_some(Self::TelexRegularV1)
    }
}

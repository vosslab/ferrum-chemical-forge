//! Closed semantic font-face identities for presentation content.

use serde::{Deserialize, Serialize};

/// The bundled face identity supported by the V1 presentation contract.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationFontFaceV1 {
    /// The verified bundled Atkinson Hyperlegible Next Regular resource.
    MoleculeLabel,
}

impl PresentationFontFaceV1 {
    /// Stable public identifier for API and UI transport.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::MoleculeLabel => "molecule_label",
        }
    }

    /// Canonical CDML family spelling when a family attribute is emitted.
    #[must_use]
    pub const fn cdml_family(self) -> &'static str {
        match self {
            Self::MoleculeLabel => "Atkinson Hyperlegible Next",
        }
    }

    /// Parse the stable public identity without accepting system-family names.
    #[must_use]
    pub fn from_id(value: &str) -> Option<Self> {
        (value == Self::MoleculeLabel.id()).then_some(Self::MoleculeLabel)
    }

    /// Parse the sole canonical CDML family spelling.
    #[must_use]
    pub fn from_cdml_family(value: &str) -> Option<Self> {
        (value == Self::MoleculeLabel.cdml_family()).then_some(Self::MoleculeLabel)
    }
}

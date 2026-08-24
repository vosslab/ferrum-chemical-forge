//! Closed compact-group catalog facts shared by admission and rendering.
//!
//! A compact group enters a complete-render profile by its catalog identity,
//! never by an independently interpreted display label. The same closed key
//! determines its label and valid exterior attachment sites everywhere.

use serde::Serialize;

/// One exact compact-group definition supported by Ferrum V1.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompactGroupCatalogKeyV1 {
    Methyl,
    Ethyl,
    Phenyl,
    Methoxy,
    Nitro,
    Cyano,
    Carboxyl,
    AcylChloride,
    Hydroxymethyl,
}

impl CompactGroupCatalogKeyV1 {
    /// Decode one exact persisted catalog key.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "methyl" => Some(Self::Methyl),
            "ethyl" => Some(Self::Ethyl),
            "phenyl" => Some(Self::Phenyl),
            "methoxy" => Some(Self::Methoxy),
            "nitro" => Some(Self::Nitro),
            "cyano" => Some(Self::Cyano),
            "carboxyl" => Some(Self::Carboxyl),
            "acyl_chloride" => Some(Self::AcylChloride),
            "hydroxymethyl" => Some(Self::Hydroxymethyl),
            _ => None,
        }
    }

    /// Resolve one exact catalog label back to its canonical definition.
    #[must_use]
    pub fn from_label(value: &str) -> Option<Self> {
        match value {
            "Me" => Some(Self::Methyl),
            "Et" => Some(Self::Ethyl),
            "Ph" => Some(Self::Phenyl),
            "OMe" => Some(Self::Methoxy),
            "NO2" => Some(Self::Nitro),
            "CN" => Some(Self::Cyano),
            "COOH" => Some(Self::Carboxyl),
            "COCl" => Some(Self::AcylChloride),
            "CH2OH" => Some(Self::Hydroxymethyl),
            _ => None,
        }
    }

    /// Return the exact persisted catalog key.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Methyl => "methyl",
            Self::Ethyl => "ethyl",
            Self::Phenyl => "phenyl",
            Self::Methoxy => "methoxy",
            Self::Nitro => "nitro",
            Self::Cyano => "cyano",
            Self::Carboxyl => "carboxyl",
            Self::AcylChloride => "acyl_chloride",
            Self::Hydroxymethyl => "hydroxymethyl",
        }
    }

    /// Return the canonical user-visible compact label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Methyl => "Me",
            Self::Ethyl => "Et",
            Self::Phenyl => "Ph",
            Self::Methoxy => "OMe",
            Self::Nitro => "NO2",
            Self::Cyano => "CN",
            Self::Carboxyl => "COOH",
            Self::AcylChloride => "COCl",
            Self::Hydroxymethyl => "CH2OH",
        }
    }

    /// Return whether one V1 exterior attachment-site index exists.
    #[must_use]
    pub const fn supports_attachment_index(self, index: u8) -> bool {
        let _ = self;
        index == 0
    }
}

/// Return whether an element uses the exact V1 atom-symbol grammar.
#[must_use]
pub fn is_admitted_atom_symbol_v1(value: &str) -> bool {
    let mut scalars = value.chars();
    let Some(first) = scalars.next() else {
        return false;
    };
    first.is_ascii_uppercase()
        && scalars.clone().count() <= 2
        && scalars.all(|scalar| scalar.is_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::{CompactGroupCatalogKeyV1, is_admitted_atom_symbol_v1};

    #[test]
    fn catalog_identity_binds_label_and_attachment_sites() {
        let methyl = CompactGroupCatalogKeyV1::parse("methyl").expect("known key");
        assert_eq!(methyl.label(), "Me");
        assert_eq!(CompactGroupCatalogKeyV1::from_label("Me"), Some(methyl));
        assert!(methyl.supports_attachment_index(0));
        assert!(!methyl.supports_attachment_index(1));
        assert_eq!(CompactGroupCatalogKeyV1::from_label("C"), None);
    }

    #[test]
    fn atom_symbol_grammar_has_exact_ascii_boundaries() {
        for symbol in ["C", "Cl", "Uuo"] {
            assert!(is_admitted_atom_symbol_v1(symbol), "{symbol}");
        }
        for symbol in ["", "c", "ABC", "abc", "Clll", "C1"] {
            assert!(!is_admitted_atom_symbol_v1(symbol), "{symbol}");
        }
    }
}

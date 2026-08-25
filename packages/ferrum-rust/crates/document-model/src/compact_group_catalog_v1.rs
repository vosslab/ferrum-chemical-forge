//! Closed compact-group catalog facts shared by admission and rendering.
//!
//! A compact group enters a complete-render profile by its catalog identity,
//! never by an independently interpreted display label. The same closed key
//! determines its label and valid exterior attachment sites everywhere.

use serde::Serialize;

/// One immutable local atom fact used only by closed compact materialization.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompactGroupRecipeAtomV1 {
    /// Stable recipe-local role, never a persisted document identity.
    pub role: &'static str,
    /// Canonical element symbol for the materialized atom.
    pub element: &'static str,
    /// Position in the recipe's attachment-relative local coordinate frame.
    pub x: f64,
    /// Position in the recipe's attachment-relative local coordinate frame.
    pub y: f64,
}

/// One immutable internal bond fact used only by closed compact materialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompactGroupRecipeBondV1 {
    /// Recipe-local role at the bond start.
    pub start_role: &'static str,
    /// Recipe-local role at the bond end.
    pub end_role: &'static str,
    /// Closed bond order that the typed-CDML writer must preserve exactly.
    pub order: CompactGroupRecipeBondOrderV1,
    /// Closed bond presentation that the typed-CDML writer must preserve exactly.
    pub presentation: CompactGroupRecipeBondPresentationV1,
}

/// Bond orders admitted in immutable compact-materialization recipes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactGroupRecipeBondOrderV1 {
    Single,
    Double,
}

/// Bond presentations admitted in immutable compact-materialization recipes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactGroupRecipeBondPresentationV1 {
    Normal,
}

/// Closed immutable topology and local geometry for one materializable group.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CompactGroupMaterializationRecipeV1 {
    /// Recipe atoms in deterministic source insertion order.
    pub atoms: &'static [CompactGroupRecipeAtomV1],
    /// Recipe-local internal bonds in deterministic source insertion order.
    pub bonds: &'static [CompactGroupRecipeBondV1],
    /// Role rewired to the pre-existing exterior bond.
    pub attachment_atom_role: &'static str,
}

const METHYL_ATOMS: [CompactGroupRecipeAtomV1; 1] = [CompactGroupRecipeAtomV1 {
    role: "attachment_carbon",
    element: "C",
    x: 0.0,
    y: 0.0,
}];

const NITRO_ATOMS: [CompactGroupRecipeAtomV1; 3] = [
    CompactGroupRecipeAtomV1 {
        role: "attachment_nitrogen",
        element: "N",
        x: 0.0,
        y: 0.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "double_oxygen",
        element: "O",
        x: 24.0,
        y: 18.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "single_oxygen",
        element: "O",
        x: 24.0,
        y: -18.0,
    },
];

const NITRO_BONDS: [CompactGroupRecipeBondV1; 2] = [
    CompactGroupRecipeBondV1 {
        start_role: "attachment_nitrogen",
        end_role: "double_oxygen",
        order: CompactGroupRecipeBondOrderV1::Double,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
    CompactGroupRecipeBondV1 {
        start_role: "attachment_nitrogen",
        end_role: "single_oxygen",
        order: CompactGroupRecipeBondOrderV1::Single,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
];

const METHYL_RECIPE: CompactGroupMaterializationRecipeV1 = CompactGroupMaterializationRecipeV1 {
    atoms: &METHYL_ATOMS,
    bonds: &[],
    attachment_atom_role: "attachment_carbon",
};

const NITRO_RECIPE: CompactGroupMaterializationRecipeV1 = CompactGroupMaterializationRecipeV1 {
    atoms: &NITRO_ATOMS,
    bonds: &NITRO_BONDS,
    attachment_atom_role: "attachment_nitrogen",
};

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

/// Return the closed materialization recipe for one catalog key.
///
/// Keys without a recipe remain valid persisted compact groups but cannot be
/// expanded until their immutable topology is explicitly added here.
#[must_use]
pub const fn materialization_recipe_v1(
    key: CompactGroupCatalogKeyV1,
) -> Option<CompactGroupMaterializationRecipeV1> {
    match key {
        CompactGroupCatalogKeyV1::Methyl => Some(METHYL_RECIPE),
        CompactGroupCatalogKeyV1::Nitro => Some(NITRO_RECIPE),
        CompactGroupCatalogKeyV1::Ethyl
        | CompactGroupCatalogKeyV1::Phenyl
        | CompactGroupCatalogKeyV1::Methoxy
        | CompactGroupCatalogKeyV1::Cyano
        | CompactGroupCatalogKeyV1::Carboxyl
        | CompactGroupCatalogKeyV1::AcylChloride
        | CompactGroupCatalogKeyV1::Hydroxymethyl => None,
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
    use super::{CompactGroupCatalogKeyV1, is_admitted_atom_symbol_v1, materialization_recipe_v1};

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

    #[test]
    fn closed_materialization_recipes_do_not_interpret_labels() {
        let methyl =
            materialization_recipe_v1(CompactGroupCatalogKeyV1::Methyl).expect("methyl recipe");
        assert_eq!(methyl.attachment_atom_role, "attachment_carbon");
        let nitro =
            materialization_recipe_v1(CompactGroupCatalogKeyV1::Nitro).expect("nitro recipe");
        assert_eq!(nitro.attachment_atom_role, "attachment_nitrogen");
        assert!(materialization_recipe_v1(CompactGroupCatalogKeyV1::Ethyl).is_none());
    }
}

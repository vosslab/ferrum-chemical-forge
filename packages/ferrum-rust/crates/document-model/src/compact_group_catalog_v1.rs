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
    /// Authored formal charge when the closed topology requires one.
    ///
    /// `None` preserves the ordinary uncharged atom representation rather
    /// than manufacturing a zero-valued source attribute.
    pub formal_charge: Option<i32>,
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
    Triple,
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
    formal_charge: None,
    x: 0.0,
    y: 0.0,
}];

const ETHYL_ATOMS: [CompactGroupRecipeAtomV1; 2] = [
    CompactGroupRecipeAtomV1 {
        role: "attachment_carbon",
        element: "C",
        formal_charge: None,
        x: 0.0,
        y: 0.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "terminal_carbon",
        element: "C",
        formal_charge: None,
        x: 24.0,
        y: 0.0,
    },
];

const ETHYL_BONDS: [CompactGroupRecipeBondV1; 1] = [CompactGroupRecipeBondV1 {
    start_role: "attachment_carbon",
    end_role: "terminal_carbon",
    order: CompactGroupRecipeBondOrderV1::Single,
    presentation: CompactGroupRecipeBondPresentationV1::Normal,
}];

const METHOXY_ATOMS: [CompactGroupRecipeAtomV1; 2] = [
    CompactGroupRecipeAtomV1 {
        role: "attachment_oxygen",
        element: "O",
        formal_charge: None,
        x: 0.0,
        y: 0.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "methyl_carbon",
        element: "C",
        formal_charge: None,
        x: 24.0,
        y: 0.0,
    },
];

const METHOXY_BONDS: [CompactGroupRecipeBondV1; 1] = [CompactGroupRecipeBondV1 {
    start_role: "attachment_oxygen",
    end_role: "methyl_carbon",
    order: CompactGroupRecipeBondOrderV1::Single,
    presentation: CompactGroupRecipeBondPresentationV1::Normal,
}];

const HYDROXYMETHYL_ATOMS: [CompactGroupRecipeAtomV1; 2] = [
    CompactGroupRecipeAtomV1 {
        role: "attachment_carbon",
        element: "C",
        formal_charge: None,
        x: 0.0,
        y: 0.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "hydroxyl_oxygen",
        element: "O",
        formal_charge: None,
        x: 24.0,
        y: 0.0,
    },
];

const HYDROXYMETHYL_BONDS: [CompactGroupRecipeBondV1; 1] = [CompactGroupRecipeBondV1 {
    start_role: "attachment_carbon",
    end_role: "hydroxyl_oxygen",
    order: CompactGroupRecipeBondOrderV1::Single,
    presentation: CompactGroupRecipeBondPresentationV1::Normal,
}];

const CARBOXYL_ATOMS: [CompactGroupRecipeAtomV1; 3] = [
    CompactGroupRecipeAtomV1 {
        role: "attachment_carbon",
        element: "C",
        formal_charge: None,
        x: 0.0,
        y: 0.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "carbonyl_oxygen",
        element: "O",
        formal_charge: None,
        x: 24.0,
        y: 18.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "hydroxyl_oxygen",
        element: "O",
        formal_charge: None,
        x: 24.0,
        y: -18.0,
    },
];

const CARBOXYL_BONDS: [CompactGroupRecipeBondV1; 2] = [
    CompactGroupRecipeBondV1 {
        start_role: "attachment_carbon",
        end_role: "carbonyl_oxygen",
        order: CompactGroupRecipeBondOrderV1::Double,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
    CompactGroupRecipeBondV1 {
        start_role: "attachment_carbon",
        end_role: "hydroxyl_oxygen",
        order: CompactGroupRecipeBondOrderV1::Single,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
];

const ACYL_CHLORIDE_ATOMS: [CompactGroupRecipeAtomV1; 3] = [
    CompactGroupRecipeAtomV1 {
        role: "attachment_carbon",
        element: "C",
        formal_charge: None,
        x: 0.0,
        y: 0.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "carbonyl_oxygen",
        element: "O",
        formal_charge: None,
        x: 24.0,
        y: 18.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "chlorine",
        element: "Cl",
        formal_charge: None,
        x: 24.0,
        y: -18.0,
    },
];

const ACYL_CHLORIDE_BONDS: [CompactGroupRecipeBondV1; 2] = [
    CompactGroupRecipeBondV1 {
        start_role: "attachment_carbon",
        end_role: "carbonyl_oxygen",
        order: CompactGroupRecipeBondOrderV1::Double,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
    CompactGroupRecipeBondV1 {
        start_role: "attachment_carbon",
        end_role: "chlorine",
        order: CompactGroupRecipeBondOrderV1::Single,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
];

const CYANO_ATOMS: [CompactGroupRecipeAtomV1; 2] = [
    CompactGroupRecipeAtomV1 {
        role: "attachment_carbon",
        element: "C",
        formal_charge: None,
        x: 0.0,
        y: 0.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "terminal_nitrogen",
        element: "N",
        formal_charge: None,
        x: 24.0,
        y: 0.0,
    },
];

const CYANO_BONDS: [CompactGroupRecipeBondV1; 1] = [CompactGroupRecipeBondV1 {
    start_role: "attachment_carbon",
    end_role: "terminal_nitrogen",
    order: CompactGroupRecipeBondOrderV1::Triple,
    presentation: CompactGroupRecipeBondPresentationV1::Normal,
}];

const PHENYL_ATOMS: [CompactGroupRecipeAtomV1; 6] = [
    CompactGroupRecipeAtomV1 {
        role: "attachment_carbon",
        element: "C",
        formal_charge: None,
        x: 0.0,
        y: 0.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "ortho_upper_carbon",
        element: "C",
        formal_charge: None,
        x: 12.0,
        y: -20.784609690826528,
    },
    CompactGroupRecipeAtomV1 {
        role: "meta_upper_carbon",
        element: "C",
        formal_charge: None,
        x: 36.0,
        y: -20.784609690826528,
    },
    CompactGroupRecipeAtomV1 {
        role: "para_carbon",
        element: "C",
        formal_charge: None,
        x: 48.0,
        y: 0.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "meta_lower_carbon",
        element: "C",
        formal_charge: None,
        x: 36.0,
        y: 20.784609690826528,
    },
    CompactGroupRecipeAtomV1 {
        role: "ortho_lower_carbon",
        element: "C",
        formal_charge: None,
        x: 12.0,
        y: 20.784609690826528,
    },
];

const PHENYL_BONDS: [CompactGroupRecipeBondV1; 6] = [
    CompactGroupRecipeBondV1 {
        start_role: "attachment_carbon",
        end_role: "ortho_upper_carbon",
        order: CompactGroupRecipeBondOrderV1::Double,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
    CompactGroupRecipeBondV1 {
        start_role: "ortho_upper_carbon",
        end_role: "meta_upper_carbon",
        order: CompactGroupRecipeBondOrderV1::Single,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
    CompactGroupRecipeBondV1 {
        start_role: "meta_upper_carbon",
        end_role: "para_carbon",
        order: CompactGroupRecipeBondOrderV1::Double,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
    CompactGroupRecipeBondV1 {
        start_role: "para_carbon",
        end_role: "meta_lower_carbon",
        order: CompactGroupRecipeBondOrderV1::Single,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
    CompactGroupRecipeBondV1 {
        start_role: "meta_lower_carbon",
        end_role: "ortho_lower_carbon",
        order: CompactGroupRecipeBondOrderV1::Double,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
    CompactGroupRecipeBondV1 {
        start_role: "ortho_lower_carbon",
        end_role: "attachment_carbon",
        order: CompactGroupRecipeBondOrderV1::Single,
        presentation: CompactGroupRecipeBondPresentationV1::Normal,
    },
];

const NITRO_ATOMS: [CompactGroupRecipeAtomV1; 3] = [
    CompactGroupRecipeAtomV1 {
        role: "attachment_nitrogen",
        element: "N",
        formal_charge: Some(1),
        x: 0.0,
        y: 0.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "double_oxygen",
        element: "O",
        formal_charge: None,
        x: 24.0,
        y: 18.0,
    },
    CompactGroupRecipeAtomV1 {
        role: "single_oxygen",
        element: "O",
        formal_charge: Some(-1),
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

const ETHYL_RECIPE: CompactGroupMaterializationRecipeV1 = CompactGroupMaterializationRecipeV1 {
    atoms: &ETHYL_ATOMS,
    bonds: &ETHYL_BONDS,
    attachment_atom_role: "attachment_carbon",
};

const METHOXY_RECIPE: CompactGroupMaterializationRecipeV1 = CompactGroupMaterializationRecipeV1 {
    atoms: &METHOXY_ATOMS,
    bonds: &METHOXY_BONDS,
    attachment_atom_role: "attachment_oxygen",
};

const HYDROXYMETHYL_RECIPE: CompactGroupMaterializationRecipeV1 =
    CompactGroupMaterializationRecipeV1 {
        atoms: &HYDROXYMETHYL_ATOMS,
        bonds: &HYDROXYMETHYL_BONDS,
        attachment_atom_role: "attachment_carbon",
    };

const CARBOXYL_RECIPE: CompactGroupMaterializationRecipeV1 = CompactGroupMaterializationRecipeV1 {
    atoms: &CARBOXYL_ATOMS,
    bonds: &CARBOXYL_BONDS,
    attachment_atom_role: "attachment_carbon",
};

const ACYL_CHLORIDE_RECIPE: CompactGroupMaterializationRecipeV1 =
    CompactGroupMaterializationRecipeV1 {
        atoms: &ACYL_CHLORIDE_ATOMS,
        bonds: &ACYL_CHLORIDE_BONDS,
        attachment_atom_role: "attachment_carbon",
    };

const CYANO_RECIPE: CompactGroupMaterializationRecipeV1 = CompactGroupMaterializationRecipeV1 {
    atoms: &CYANO_ATOMS,
    bonds: &CYANO_BONDS,
    attachment_atom_role: "attachment_carbon",
};

const PHENYL_RECIPE: CompactGroupMaterializationRecipeV1 = CompactGroupMaterializationRecipeV1 {
    atoms: &PHENYL_ATOMS,
    bonds: &PHENYL_BONDS,
    attachment_atom_role: "attachment_carbon",
};

const NITRO_RECIPE: CompactGroupMaterializationRecipeV1 = CompactGroupMaterializationRecipeV1 {
    // Nitro materializes canonically as R-[N+](=O)[O-]. This closed recipe
    // owns one resonance form so materialization remains deterministic.
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

const ATTACHED_COMPACT_GROUP_AUTHORING_KEYS_V1: &[CompactGroupCatalogKeyV1] = &[
    CompactGroupCatalogKeyV1::Methyl,
    CompactGroupCatalogKeyV1::Nitro,
    CompactGroupCatalogKeyV1::Ethyl,
    CompactGroupCatalogKeyV1::Methoxy,
    CompactGroupCatalogKeyV1::Hydroxymethyl,
    CompactGroupCatalogKeyV1::Carboxyl,
    CompactGroupCatalogKeyV1::Cyano,
    CompactGroupCatalogKeyV1::AcylChloride,
    CompactGroupCatalogKeyV1::Phenyl,
];

/// Return the closed catalog keys supported by attached compact-group authoring.
///
/// Persisted catalog support and materialization support are intentionally broader
/// and separate from this authoring capability.
#[must_use]
pub const fn attached_compact_group_authoring_keys_v1() -> &'static [CompactGroupCatalogKeyV1] {
    ATTACHED_COMPACT_GROUP_AUTHORING_KEYS_V1
}

/// Return whether attached compact-group authoring supports one persisted key.
#[must_use]
pub fn supports_attached_compact_group_authoring_v1(key: CompactGroupCatalogKeyV1) -> bool {
    attached_compact_group_authoring_keys_v1().contains(&key)
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
        CompactGroupCatalogKeyV1::Ethyl => Some(ETHYL_RECIPE),
        CompactGroupCatalogKeyV1::Methoxy => Some(METHOXY_RECIPE),
        CompactGroupCatalogKeyV1::Nitro => Some(NITRO_RECIPE),
        CompactGroupCatalogKeyV1::Hydroxymethyl => Some(HYDROXYMETHYL_RECIPE),
        CompactGroupCatalogKeyV1::Carboxyl => Some(CARBOXYL_RECIPE),
        CompactGroupCatalogKeyV1::Cyano => Some(CYANO_RECIPE),
        CompactGroupCatalogKeyV1::AcylChloride => Some(ACYL_CHLORIDE_RECIPE),
        CompactGroupCatalogKeyV1::Phenyl => Some(PHENYL_RECIPE),
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
#[path = "compact_group_catalog_v1_tests.rs"]
mod tests;

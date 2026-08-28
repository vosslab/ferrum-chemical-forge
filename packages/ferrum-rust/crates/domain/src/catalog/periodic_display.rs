//! Closed display metadata for the element-picker symbols Ferrum currently exposes.
//!
//! This is intentionally a display catalog, not a periodic-table chemistry
//! model. It owns only the symbols rendered by Ferrum's element picker,
//! their closed category vocabulary, and the V1 palette. Query pseudo-elements
//! are absent because the picker does not offer them.

use std::fmt;

/// Stable provenance for the authored element-picker display contract.
pub const PERIODIC_DISPLAY_CATALOG_PROVENANCE_V1: PeriodicDisplayCatalogProvenanceV1 =
    PeriodicDisplayCatalogProvenanceV1 {
        catalog_id: "ferrum-periodic-display-v1",
        revision: "2026-08-12",
        source: "Ferrum-authored element-picker display catalog",
        scope: "Ferrum periodic-table popup symbols only; no query pseudo-elements",
    };

/// Immutable provenance for the bounded display catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PeriodicDisplayCatalogProvenanceV1 {
    catalog_id: &'static str,
    revision: &'static str,
    source: &'static str,
    scope: &'static str,
}

impl PeriodicDisplayCatalogProvenanceV1 {
    pub const fn catalog_id(&self) -> &'static str {
        self.catalog_id
    }

    pub const fn revision(&self) -> &'static str {
        self.revision
    }

    pub const fn source(&self) -> &'static str {
        self.source
    }

    pub const fn scope(&self) -> &'static str {
        self.scope
    }
}

/// Closed V1 palette categories for Ferrum's periodic-table popup.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ElementDisplayCategoryV1 {
    Nonmetal,
    Halogen,
    NobleGas,
    Metalloid,
    Metal,
    TransitionMetal,
    Lanthanide,
    Actinide,
}

impl ElementDisplayCategoryV1 {
    /// Return the exact lowercase CSS color assigned by the V1 palette.
    pub const fn color(self) -> &'static str {
        match self {
            Self::Nonmetal => "#a0ffa0",
            Self::Halogen => "#ffff80",
            Self::NobleGas => "#a0e0ff",
            Self::Metalloid => "#ffffa0",
            Self::Metal => "#ffa0a0",
            Self::TransitionMetal => "#ffc0c0",
            Self::Lanthanide => "#ffbfff",
            Self::Actinide => "#ff99cc",
        }
    }
}

/// One immutable element-picker display entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElementDisplayFactsV1 {
    symbol: &'static str,
    display_name: &'static str,
    grid_row: u8,
    grid_column: u8,
    category: ElementDisplayCategoryV1,
}

impl ElementDisplayFactsV1 {
    pub const fn symbol(&self) -> &'static str {
        self.symbol
    }

    /// Return the authored accessible name shown for this picker symbol.
    pub const fn display_name(&self) -> &'static str {
        self.display_name
    }

    /// Return this entry's zero-based row in the bounded picker grid.
    pub const fn grid_row(&self) -> u8 {
        self.grid_row
    }

    /// Return this entry's zero-based column in the bounded picker grid.
    pub const fn grid_column(&self) -> u8 {
        self.grid_column
    }

    pub const fn category(&self) -> ElementDisplayCategoryV1 {
        self.category
    }

    pub const fn color(&self) -> &'static str {
        self.category.color()
    }
}

/// A typed rejection for a symbol outside the explicit V1 picker contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownElementSymbolError {
    symbol: String,
}

impl UnknownElementSymbolError {
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

impl fmt::Display for UnknownElementSymbolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "unsupported periodic display symbol: {:?}",
            self.symbol
        )
    }
}

impl std::error::Error for UnknownElementSymbolError {}

const ELEMENTS: &[ElementDisplayFactsV1] = &[
    facts("H", "Hydrogen", 0, 0, ElementDisplayCategoryV1::Nonmetal),
    facts("He", "Helium", 0, 17, ElementDisplayCategoryV1::NobleGas),
    facts("Li", "Lithium", 1, 0, ElementDisplayCategoryV1::Metal),
    facts("Be", "Beryllium", 1, 1, ElementDisplayCategoryV1::Metal),
    facts("B", "Boron", 1, 12, ElementDisplayCategoryV1::Metalloid),
    facts("C", "Carbon", 1, 13, ElementDisplayCategoryV1::Nonmetal),
    facts("N", "Nitrogen", 1, 14, ElementDisplayCategoryV1::Nonmetal),
    facts("O", "Oxygen", 1, 15, ElementDisplayCategoryV1::Nonmetal),
    facts("F", "Fluorine", 1, 16, ElementDisplayCategoryV1::Halogen),
    facts("Ne", "Neon", 1, 17, ElementDisplayCategoryV1::NobleGas),
    facts("Na", "Sodium", 2, 0, ElementDisplayCategoryV1::Metal),
    facts("Mg", "Magnesium", 2, 1, ElementDisplayCategoryV1::Metal),
    facts("Al", "Aluminium", 2, 12, ElementDisplayCategoryV1::Metal),
    facts("Si", "Silicon", 2, 13, ElementDisplayCategoryV1::Metalloid),
    facts("P", "Phosphorus", 2, 14, ElementDisplayCategoryV1::Nonmetal),
    facts("S", "Sulfur", 2, 15, ElementDisplayCategoryV1::Nonmetal),
    facts("Cl", "Chlorine", 2, 16, ElementDisplayCategoryV1::Halogen),
    facts("Ar", "Argon", 2, 17, ElementDisplayCategoryV1::NobleGas),
    facts("K", "Potassium", 3, 0, ElementDisplayCategoryV1::Metal),
    facts("Ca", "Calcium", 3, 1, ElementDisplayCategoryV1::Metal),
    facts(
        "Ti",
        "Titanium",
        3,
        3,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts(
        "Cr",
        "Chromium",
        3,
        5,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts(
        "Mn",
        "Manganese",
        3,
        6,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts(
        "Fe",
        "Iron",
        3,
        7,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts(
        "Co",
        "Cobalt",
        3,
        8,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts(
        "Ni",
        "Nickel",
        3,
        9,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts(
        "Cu",
        "Copper",
        3,
        10,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts(
        "Zn",
        "Zinc",
        3,
        11,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts("Ga", "Gallium", 3, 12, ElementDisplayCategoryV1::Metal),
    facts(
        "Ge",
        "Germanium",
        3,
        13,
        ElementDisplayCategoryV1::Metalloid,
    ),
    facts("As", "Arsenic", 3, 14, ElementDisplayCategoryV1::Metalloid),
    facts("Se", "Selenium", 3, 15, ElementDisplayCategoryV1::Nonmetal),
    facts("Br", "Bromine", 3, 16, ElementDisplayCategoryV1::Halogen),
    facts("Kr", "Krypton", 3, 17, ElementDisplayCategoryV1::NobleGas),
    facts(
        "Ag",
        "Silver",
        4,
        10,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts("Sn", "Tin", 4, 13, ElementDisplayCategoryV1::Metal),
    facts("I", "Iodine", 4, 16, ElementDisplayCategoryV1::Halogen),
    facts("Xe", "Xenon", 4, 17, ElementDisplayCategoryV1::NobleGas),
    facts(
        "Pt",
        "Platinum",
        5,
        9,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts(
        "Au",
        "Gold",
        5,
        10,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts(
        "Hg",
        "Mercury",
        5,
        11,
        ElementDisplayCategoryV1::TransitionMetal,
    ),
    facts("Pb", "Lead", 5, 13, ElementDisplayCategoryV1::Metal),
];

const fn facts(
    symbol: &'static str,
    display_name: &'static str,
    grid_row: u8,
    grid_column: u8,
    category: ElementDisplayCategoryV1,
) -> ElementDisplayFactsV1 {
    ElementDisplayFactsV1 {
        symbol,
        display_name,
        grid_row,
        grid_column,
        category,
    }
}

/// Return every supported picker entry in its user-visible order.
pub const fn periodic_display_elements_v1() -> &'static [ElementDisplayFactsV1] {
    ELEMENTS
}

/// Return immutable display facts for one exact supported element symbol.
pub fn periodic_display_facts_v1(
    symbol: &str,
) -> Result<&'static ElementDisplayFactsV1, UnknownElementSymbolError> {
    ELEMENTS
        .iter()
        .find(|entry| entry.symbol == symbol)
        .ok_or_else(|| UnknownElementSymbolError {
            symbol: symbol.to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use super::{
        ElementDisplayCategoryV1, PERIODIC_DISPLAY_CATALOG_PROVENANCE_V1,
        periodic_display_elements_v1, periodic_display_facts_v1,
    };

    #[test]
    fn picker_catalog_has_one_explicit_entry_for_every_supported_symbol() {
        let symbols: Vec<_> = periodic_display_elements_v1()
            .iter()
            .map(|entry| entry.symbol())
            .collect();
        assert_eq!(symbols.len(), 42);
        assert_eq!(symbols.first(), Some(&"H"));
        assert_eq!(symbols.last(), Some(&"Pb"));
        assert_eq!(symbols.iter().filter(|symbol| **symbol == "C").count(), 1);
    }

    #[test]
    fn representative_picker_entries_expose_names_grid_palette_and_category() {
        let iron = periodic_display_facts_v1("Fe").expect("supported picker symbol");
        assert_eq!(iron.display_name(), "Iron");
        assert_eq!((iron.grid_row(), iron.grid_column()), (3, 7));
        assert_eq!(iron.category(), ElementDisplayCategoryV1::TransitionMetal);
        assert_eq!(iron.color(), "#ffc0c0");
        let helium = periodic_display_facts_v1("He").expect("supported picker symbol");
        assert_eq!(helium.display_name(), "Helium");
        assert_eq!((helium.grid_row(), helium.grid_column()), (0, 17));
        assert_eq!(ElementDisplayCategoryV1::NobleGas.color(), "#a0e0ff");
    }

    #[test]
    fn unknown_symbols_are_rejected_without_alias_or_fallback() {
        let error = periodic_display_facts_v1("fe").expect_err("case aliases are unsupported");
        assert_eq!(error.symbol(), "fe");
        assert!(periodic_display_facts_v1("X").is_err());
    }

    #[test]
    fn provenance_scopes_the_catalog_to_the_picker() {
        assert_eq!(
            PERIODIC_DISPLAY_CATALOG_PROVENANCE_V1.catalog_id(),
            "ferrum-periodic-display-v1"
        );
        assert!(
            PERIODIC_DISPLAY_CATALOG_PROVENANCE_V1
                .scope()
                .contains("popup")
        );
    }
}

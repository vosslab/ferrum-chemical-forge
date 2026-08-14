//! Independently authored reference catalogs for Ferrum domain workflows.
//!
//! Catalogs are deliberately distinct: a functional-group lookup is not a
//! biomolecule template, and a sugar name is not a structure assertion.  Each
//! dataset carries enough provenance to make its scope and integrity explicit.

mod biomolecules;
mod functional_groups;
mod periodic_display;
mod provenance;
mod sugar_names;

pub use biomolecules::{BiomoleculeKind, BiomoleculeTemplate, BiomoleculeTemplateCatalog};
pub use functional_groups::{FunctionalGroup, FunctionalGroupCatalog};
pub use periodic_display::{
    ElementDisplayCategoryV1, ElementDisplayFactsV1, PERIODIC_DISPLAY_CATALOG_PROVENANCE_V1,
    PeriodicDisplayCatalogProvenanceV1, UnknownElementSymbolError, periodic_display_elements_v1,
    periodic_display_facts_v1,
};
pub use provenance::{CatalogError, CatalogProvenance, VerifiedCatalog};
pub use sugar_names::{SugarName, SugarNameCatalog};

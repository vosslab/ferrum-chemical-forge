//! Independently authored reference catalogs for Ferrum domain workflows.
//!
//! Catalogs are deliberately distinct: a functional-group lookup is not a
//! biomolecule template, and a sugar name is not a structure assertion.  Each
//! dataset carries enough provenance to make its scope and integrity explicit.

mod periodic_display;

pub use periodic_display::{
    ElementDisplayCategoryV1, ElementDisplayFactsV1, PERIODIC_DISPLAY_CATALOG_PROVENANCE_V1,
    PeriodicDisplayCatalogProvenanceV1, UnknownElementSymbolError, periodic_display_elements_v1,
    periodic_display_facts_v1,
};

//! Independently authored reference catalogs for Ferrum domain workflows.
//!
//! Catalogs are deliberately distinct: a functional-group lookup is not a
//! biomolecule template, and a sugar name is not a structure assertion.  Each
//! dataset carries enough provenance to make its scope and integrity explicit.

mod biomolecules;
mod functional_groups;
mod provenance;
mod sugar_names;

pub use biomolecules::{BiomoleculeKind, BiomoleculeTemplate, BiomoleculeTemplateCatalog};
pub use functional_groups::{FunctionalGroup, FunctionalGroupCatalog};
pub use provenance::{CatalogError, CatalogProvenance, VerifiedCatalog};
pub use sugar_names::{SugarName, SugarNameCatalog};

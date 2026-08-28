//! Immutable Rust-owned shipped and local CDML template catalog (ASVS 1.5, 2.1-2.3, 5.1-5.3).
mod error;
mod snapshot;
mod types;
mod user_directory;

#[cfg(test)]
mod tests;

pub use error::{
    TemplateCatalogErrorV1, TemplateCatalogRecoveryV1, TemplateCatalogRefusalCategoryV1,
    TemplateCatalogRefusalV1,
};
pub use snapshot::{
    TemplateCatalogApplyErrorV1, TemplateCatalogPlacementResultV1, TemplateCatalogSnapshotV1,
    apply_template_catalog_entry_v1, snapshot_template_catalog_v1,
};
pub use types::{
    TemplateCatalogEntryV1, TemplateCatalogKeyV1, TemplateCatalogLimitsV1,
    TemplateCatalogProvenanceV1, TemplateCatalogSourceV1, TemplateCompatibilityV1,
    TemplateContentIdentityV1, TemplateFormatV1,
};

pub const TEMPLATE_CATALOG_SCHEMA_V1: &str = "ferrum-template-catalog-v1";

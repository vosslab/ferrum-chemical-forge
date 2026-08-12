use std::collections::BTreeMap;

use serde::Serialize;

pub(crate) const INSPECTION_SCHEMA: &str = "ferrum-cdml-inspection-v1";
pub(crate) const VALIDATION_SCHEMA: &str = "ferrum-cdml-validation-v1";
pub(crate) const REWRITE_CHECK_SCHEMA: &str = "ferrum-cdml-rewrite-check-v1";

/// Stable machine-readable summary emitted by `ferrum cdml inspect`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CdmlInspection {
    /// Versioned JSON schema identifier.
    pub schema: &'static str,
    /// CDML version declared by the core projection, when present.
    pub document_version: Option<String>,
    /// Count of persistent identities, including retained opaque XML content.
    pub persistent_id_count: usize,
    /// Count of direct children of the CDML root.
    pub top_level_record_count: usize,
    /// Counts keyed by Ferrum's stable typed record class names.
    pub typed_record_counts: BTreeMap<&'static str, usize>,
    /// Non-demoting typed-record diagnostics.
    pub diagnostic_count: usize,
    /// Core-projected molecule observations in source order.
    pub molecules: Vec<MoleculeInspection>,
}

/// One molecule's source-order summary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MoleculeInspection {
    /// Persistent source identifier, if the molecule declared one.
    pub source_id: Option<String>,
    /// Optional molecule name.
    pub name: Option<String>,
    /// Atom count.
    pub atom_count: usize,
    /// Non-atom vertex count.
    pub group_count: usize,
    /// Text vertex count.
    pub text_count: usize,
    /// Query vertex count.
    pub query_count: usize,
    /// Bond count.
    pub bond_count: usize,
}

/// Successful CDML validation at an explicitly selected Ferrum level.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CdmlValidation {
    /// Versioned JSON schema identifier.
    pub schema: &'static str,
    /// Always true for a successful report; invalid inputs use stderr and exit 1.
    pub valid: bool,
    /// `structural` retains and indexes CDML; `core` also creates molecule facts.
    pub level: &'static str,
    /// CDML `version` attribute, when present.
    pub document_version: Option<String>,
    /// Count of persistent identities, including retained opaque XML content.
    pub persistent_id_count: usize,
    /// Count of direct children of the CDML root.
    pub top_level_record_count: usize,
    /// Non-demoting typed-record diagnostics.
    pub diagnostic_count: usize,
}

/// A successful structural preservation check for `cdml rewrite --check`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RewriteCheck {
    /// Versioned JSON schema identifier.
    pub schema: &'static str,
    /// True only when the Ferrum-owned structural observations survive reparse.
    pub valid: bool,
    /// Count of persistent identities verified before and after serialization.
    pub persistent_id_count: usize,
    /// Count of direct-root records verified before and after serialization.
    pub top_level_record_count: usize,
    /// Typed record counts verified before and after serialization.
    pub typed_record_counts: BTreeMap<&'static str, usize>,
    /// Retained opaque-child count verified before and after serialization.
    pub opaque_child_count: usize,
}

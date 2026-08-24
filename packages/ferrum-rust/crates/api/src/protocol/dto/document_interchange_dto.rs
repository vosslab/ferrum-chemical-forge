//! DTOs for fixed-target molecular interchange import.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Fixed new-document interchange import. No snapshot, placement, or append mode exists.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeInterchangeImportRequestV1 {
    /// Exact lower-case registry alias. No suffix sniffing or fallback occurs.
    pub format_alias: String,
    /// Request-owned UTF-8 source; no path, handle, or source identity crosses this boundary.
    pub source_utf8: String,
}

/// Provenance-safe origin class for one interchange source.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentInterchangeSourceKindV1 {
    RequestText,
    RegularFile,
    StandardInput,
}

/// Provenance facts that cannot disclose a path, source identifier, title, property, or bytes.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentInterchangeProvenanceV1 {
    pub format_id: String,
    pub profile_id: String,
    pub source_kind: DocumentInterchangeSourceKindV1,
}

/// A semantic category intentionally not retained by an import profile.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentInterchangeLossCategoryV1 {
    LexicalSyntax,
}

/// Bounded protocol-owned facts for one fixed-target interchange import.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentInterchangeImportSummaryV1 {
    pub format_id: String,
    pub profile_id: String,
    pub imported_record_count: u32,
    pub atom_count: u32,
    pub bond_count: u32,
    pub document_revision: u64,
    pub document_digest_hex: String,
    pub provenance: DocumentInterchangeProvenanceV1,
    pub loss_report: DocumentInterchangeImportLossReportV1,
}

/// Closed loss facts for the public interchange-import summary.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentInterchangeImportLossReportV1 {
    pub source_identifiers_reallocated: bool,
    pub dropped_categories: Vec<DocumentInterchangeLossCategoryV1>,
}

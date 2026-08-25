//! DTOs for catalog placement and durable reaction operations.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Optional filters over immutable shipped template catalog summary facts.
///
/// Family and category use exact closed identities. Query trims ASCII whitespace,
/// compares ASCII case-insensitively, and matches only emitted summary names and
/// identifiers; catalog source, recipe, and document payloads are never searched.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogListRequestV1 {
    pub family: Option<ProtocolCatalogFamilyV1>,
    pub category: Option<String>,
    pub query: Option<String>,
}

/// Closed, stateless catalog insertion request. The caller names a catalog ID,
/// never a recipe, CDML fragment, path, or rendering payload.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogInsertRequestV1 {
    pub document: String,
    pub expected_revision: u64,
    pub expected_digest_hex: String,
    pub catalog_id: String,
    pub anchor_x: f64,
    pub anchor_y: f64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionCreateRequestV1 {
    pub document: String,
    pub expected_revision: u64,
    pub expected_digest_hex: String,
    pub reactant_document_object_ids: Vec<String>,
    pub product_document_object_ids: Vec<String>,
    pub arrow_document_object_id: String,
    pub reagent_document_object_ids: Vec<String>,
    pub plus_document_object_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionObservationRequestV1 {
    pub document: String,
    pub expected_revision: u64,
    pub expected_digest_hex: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionObserveRequestV1 {
    pub document: String,
    pub expected_revision: u64,
    pub expected_digest_hex: String,
    pub reaction_document_object_id: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionPatchMembershipRequestV1 {
    pub document: String,
    pub expected_revision: u64,
    pub expected_digest_hex: String,
    pub reaction_document_object_id: String,
    pub reactant_document_object_ids: Vec<String>,
    pub product_document_object_ids: Vec<String>,
    pub arrow_document_object_id: String,
    pub reagent_document_object_ids: Vec<String>,
    pub plus_document_object_ids: Vec<String>,
}

#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionObservationSummaryV1 {
    pub reaction_document_object_id: String,
    pub disposition: ProtocolReactionDefinitionDispositionV1,
    pub diagnostics: Vec<String>,
    pub members: Vec<ReactionMemberSummaryV1>,
}

#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionMemberSummaryV1 {
    pub document_object_id: String,
    pub role: String,
    pub role_ordinal: u32,
    /// Global order is available only for admitted renderer-backed members.
    pub document_paint_order: u32,
}

#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolReactionDefinitionDispositionV1 {
    Strict,
    DisplayOnly,
}

/// Immutable, provenance-safe catalog entry facts. This DTO deliberately has
/// no template source, CDML, filesystem location, or presentation asset.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogEntrySummaryV1 {
    pub id: String,
    pub family: ProtocolCatalogFamilyV1,
    pub category: CatalogCategorySummaryV1,
    pub name: String,
    pub provenance: CatalogProvenanceSummaryV1,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolCatalogFamilyV1 {
    System,
    Biomolecule,
}

#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogCategorySummaryV1 {
    pub id: String,
    pub name: String,
    pub order: u16,
}

#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogProvenanceSummaryV1 {
    pub source_kind: String,
    pub source_id: String,
    pub license_spdx: String,
}

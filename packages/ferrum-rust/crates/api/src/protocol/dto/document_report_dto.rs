//! DTOs for read-only molecular composition reports.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Closed read-only molecule-report request. The runtime and report graph are
/// intentionally absent from this transport contract.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportRequestV1 {
    pub document: String,
    pub expected_revision: u64,
    pub expected_digest_hex: String,
    pub molecule_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportSummaryV1 {
    pub schema: String,
    pub source_revision: u64,
    pub source_digest_hex: String,
    pub records: Vec<DocumentMoleculeReportRecordSummaryV1>,
    /// The one complete aggregate composition or its closed omission reason.
    /// Ferrum never emits a subset aggregate.
    pub aggregate: DocumentMoleculeReportAggregateOutcomeSummaryV1,
}

/// The all-or-none aggregate result for one molecule report.
///
/// This tagged DTO makes a complete composition and an omission reason mutually
/// exclusive at both the Rust and JSON protocol boundaries.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DocumentMoleculeReportAggregateOutcomeSummaryV1 {
    Complete {
        composition: DocumentMoleculeReportCompositionSummaryV1,
    },
    Omitted {
        reason: DocumentMoleculeReportAggregateOmissionReasonSummaryV1,
    },
}

/// Closed reasons that a molecule-report aggregate is unavailable.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentMoleculeReportAggregateOmissionReasonSummaryV1 {
    FewerThanTwoSelected,
    IncompleteRecordComposition,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportRecordSummaryV1 {
    pub molecule_id: String,
    pub source_id: String,
    pub document_root_order: u32,
    pub authored_name: Option<String>,
    pub atom_count: usize,
    pub bond_count: usize,
    pub authored_charge: Option<i64>,
    pub authored_elements: Vec<DocumentMoleculeReportElementCountSummaryV1>,
    /// Complete engine-derived facts, or `None` when this root cannot produce a
    /// supported composition. `finding_codes` explains that absence.
    pub composition: Option<DocumentMoleculeReportCompositionSummaryV1>,
    pub neutral_bond_capacity: String,
    pub finding_codes: Vec<String>,
}

/// One all-or-none, finite composition receipt mapped from an authenticated
/// chemistry result. No graph, CDML, toolkit, or runtime capability is exposed.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportCompositionSummaryV1 {
    pub formula: String,
    pub net_formal_charge: i64,
    pub average_molecular_weight_da: f64,
    pub monoisotopic_mass_da: f64,
    /// Isotope-aware counts and average-mass percentages in canonical formula order.
    pub elements: Vec<DocumentMoleculeReportCompositionElementSummaryV1>,
}

/// One isotope-aware elemental contribution to a complete composition receipt.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportCompositionElementSummaryV1 {
    pub symbol: String,
    pub isotope: Option<u16>,
    pub atom_count: u64,
    pub average_mass_contribution_da: f64,
    pub mass_percentage: f64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportElementCountSummaryV1 {
    pub symbol: String,
    pub atom_count: usize,
}

//! DTOs for bounded, read-only direct-root structure diagnostics.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::document_report_dto::DocumentMoleculeReportFindingSummaryV1;

/// One request-owned source snapshot for structure diagnostics.
///
/// `revision` is preserved as the initiating document's source fence. The
/// detached executor verifies `digest_hex` against `cdml` before it evaluates
/// any selected root.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeDiagnosticsSnapshotV1 {
    pub cdml: String,
    pub revision: u64,
    pub digest_hex: String,
}

/// Closed read-only request for deterministic direct-root diagnostics.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeDiagnosticsRequestV1 {
    pub snapshot: DocumentMoleculeDiagnosticsSnapshotV1,
    pub molecule_ids: Vec<String>,
}

/// Bounded diagnostic receipt for one immutable source fence.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeDiagnosticsSummaryV1 {
    pub schema: String,
    pub source_revision: u64,
    pub source_digest_hex: String,
    /// Records are ordered by durable document-root order, never caller order.
    pub records: Vec<DocumentMoleculeDiagnosticRecordSummaryV1>,
}

/// One authenticated selected direct root and its bounded findings.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeDiagnosticRecordSummaryV1 {
    pub molecule_id: String,
    pub document_paint_order: u32,
    /// Shared closed diagnostic vocabulary with the molecule report route.
    pub findings: Vec<DocumentMoleculeReportFindingSummaryV1>,
}

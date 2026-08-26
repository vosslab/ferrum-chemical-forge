//! DTOs for fenced document observations and explicit-hydrogen materialization.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::DocumentRequestFenceV1;

/// Fenced request-owned document supplied to stateless document observations.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSnapshotRequestV1 {
    pub cdml: String,
    pub expected_revision: u64,
    pub expected_digest_hex: String,
}

/// One exact query representation admitted by the SMARTS query operation.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DocumentSmartsQueryInputV1 {
    Smarts { value: String },
    SelectedMolecule { molecule_id: String },
}

/// Optional bounded enumeration limits. Omitted fields use the V1 defaults.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSmartsQueryLimitsV1 {
    pub max_matches_per_molecule: Option<u32>,
    pub max_total_matches: Option<u32>,
}

/// Stateless V1 SMARTS query request. It has no renderer, receipt, or runtime value.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSmartsQueryRequestV1 {
    pub document: DocumentSnapshotRequestV1,
    pub query: DocumentSmartsQueryInputV1,
    #[serde(default)]
    pub limits: DocumentSmartsQueryLimitsV1,
}

/// Stateless selected-atom oxidation observation over one fenced document snapshot.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentAtomOxidationObserveRequestV1 {
    pub document: DocumentSnapshotRequestV1,
    pub molecule_id: String,
    pub atom_id: String,
}

/// Stateless renderer-safe hydrogen materialization request over one fenced root.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeHydrogenMaterializationRequestV1 {
    pub document: DocumentSnapshotRequestV1,
    pub molecule_id: String,
    pub anchor_atom_id: String,
}

/// Stateless request to materialize one attached compact group from a fenced snapshot.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentCompactGroupMaterializationRequestV1 {
    pub document: DocumentSnapshotRequestV1,
    pub molecule_id: String,
    pub compact_group_id: String,
}

/// Committed compact-group replacement facts. Preparation state remains session-private.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentCompactGroupMaterializationResultV1 {
    pub schema: String,
    pub source_revision: u64,
    pub source_digest_hex: String,
    pub molecule_id: String,
    pub compact_group_id: String,
    pub replacement_focus_atom_id: String,
    pub document: String,
    pub document_fence: DocumentRequestFenceV1,
}

/// Closed public recovery facts for compact-group materialization refusals.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompactGroupMaterializationRefusalV1 {
    pub category: ProtocolCompactGroupMaterializationCategoryV1,
    pub recovery: ProtocolCompactGroupMaterializationRecoveryV1,
}

/// Stable compact-group refusal categories. No candidate or source CDML is exposed.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolCompactGroupMaterializationCategoryV1 {
    StaleDocumentFence,
    UnknownOrForeignTarget,
    IneligibleTarget,
    RendererPreparationRefusal,
    SessionConflictOrConsumedPreparation,
}

/// Stable client recovery instructions for compact-group materialization.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolCompactGroupMaterializationRecoveryV1 {
    RefreshAndRetry,
    CorrectTarget,
    ChooseEligibleTarget,
    DocumentUnchanged,
}

/// Public SMARTS query facts. Match membership remains private to the live bridge.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSmartsQuerySummaryV1 {
    pub schema: String,
    pub traversal: DocumentSmartsQueryTraversalSummaryV1,
    pub molecules: Vec<DocumentSmartsQueryMoleculeSummaryV1>,
}

/// Completed V1 oxidation observation for one fenced durable atom selection.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentAtomOxidationObservationV1 {
    pub schema: String,
    pub source_revision: u64,
    pub source_digest_hex: String,
    pub molecule_id: String,
    pub atom_id: String,
    pub document_paint_order: u32,
    pub convention: String,
    #[serde(flatten)]
    pub outcome: DocumentAtomOxidationObservationOutcomeV1,
}

/// Mutually exclusive completed oxidation outcomes.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DocumentAtomOxidationObservationOutcomeV1 {
    Accepted {
        oxidation_number: i16,
    },
    Unavailable {
        unavailable_reason: DocumentAtomOxidationUnavailableReasonV1,
    },
}

/// Closed root-profile reasons for a completed unavailable observation.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentAtomOxidationUnavailableReasonV1 {
    ElementOutsideProfile,
    FormalChargeUnavailable,
    HydrogenTopologyUnsupported,
    AromaticityUnsupported,
    RadicalUnsupported,
    BondOrderUnavailable,
    BondOrderUnsupported,
    NonAtomVertexUnsupported,
    CoordinationOrDelocalizationUnsupported,
    ComponentInvariantFailed,
    ArithmeticOverflow,
}

/// Completed V1 materialization for one fenced durable root and anchor.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeHydrogenMaterializationResultV1 {
    pub schema: String,
    pub source_revision: u64,
    pub source_digest_hex: String,
    pub molecule_id: String,
    pub anchor_atom_id: String,
    #[serde(flatten)]
    pub outcome: DocumentMoleculeHydrogenMaterializationOutcomeV1,
}

/// Closed successful and unavailable V1 materialization outcomes.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DocumentMoleculeHydrogenMaterializationOutcomeV1 {
    Applied {
        added_hydrogen_count: u32,
        document: String,
        document_fence: DocumentRequestFenceV1,
    },
    NoOp {
        added_hydrogen_count: u32,
        document: String,
        document_fence: DocumentRequestFenceV1,
    },
    Unavailable {
        unavailable_reason: DocumentMoleculeHydrogenMaterializationUnavailableReasonV1,
    },
}

/// Closed source-profile reasons for a completed unavailable materialization.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentMoleculeHydrogenMaterializationUnavailableReasonV1 {
    ElementOutsideProfile,
    NonzeroFormalCharge,
    NonzeroExplicitHydrogens,
    UnsupportedBondOrRadical,
    ExistingHydrogenTopology,
    ValenceExceeded,
    UnsupportedDocument,
    ResourceLimit,
    UnrenderableCandidate,
    OxidationPostcondition,
    RenderPreparation,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DocumentSmartsQueryTraversalSummaryV1 {
    Complete,
    Incomplete { reason: String },
}

/// One document-paint-ordered target with at least one retained match. No atom identity crosses JSON.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSmartsQueryMoleculeSummaryV1 {
    pub document_paint_order: u32,
    pub match_count: u32,
    pub completeness: String,
}

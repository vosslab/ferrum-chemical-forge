//! Frozen request, receipt, and refusal DTOs for attached compact groups.

use ferrum_document::CompactGroupCatalogKeyV1;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{DocumentRequestFenceV1, DocumentSnapshotRequestV1};

/// Stateless request to attach one closed catalog group to one fenced atom target.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentCompactGroupAttachmentRequestV1 {
    pub document: DocumentSnapshotRequestV1,
    pub molecule_id: String,
    pub anchor_atom_id: String,
    pub catalog_key: ProtocolCompactGroupCatalogKeyV1,
    pub release: DocumentCompactGroupAttachmentReleaseV1,
}

/// Finite pointer-release direction supplied to the document-owned attachment authority.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentCompactGroupAttachmentReleaseV1 {
    pub x: f64,
    pub y: f64,
}

/// Committed attachment facts. Candidate geometry and release intent remain private.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentCompactGroupAttachmentResultV1 {
    pub schema: String,
    pub source_revision: u64,
    pub source_digest_hex: String,
    pub molecule_id: String,
    pub anchor_atom_id: String,
    pub catalog_key: ProtocolCompactGroupCatalogKeyV1,
    pub compact_group_id: String,
    pub document: String,
    pub document_fence: DocumentRequestFenceV1,
}

/// Closed public recovery facts for compact-group attachment refusals.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompactGroupAttachmentRefusalV1 {
    pub category: ProtocolCompactGroupAttachmentCategoryV1,
    pub recovery: ProtocolCompactGroupAttachmentRecoveryV1,
}

/// Stable attachment refusal categories. Invalid JSON catalog keys fail decode instead.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolCompactGroupAttachmentCategoryV1 {
    StaleDocumentFence,
    UnknownTarget,
    ForeignTarget,
    InvalidRelease,
    CandidateAdmission,
    RendererAdmission,
    SessionConflict,
}

/// Stable recovery instructions for compact-group attachment refusals.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolCompactGroupAttachmentRecoveryV1 {
    RefreshAndRetry,
    CorrectTarget,
    ChangeRelease,
    ChooseAvailableTarget,
    DocumentUnchanged,
}

/// Closed catalog identity admitted by the attached compact-group protocol route.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolCompactGroupCatalogKeyV1 {
    Methyl,
    Ethyl,
    Phenyl,
    Methoxy,
    Nitro,
    Cyano,
    Carboxyl,
    AcylChloride,
    Hydroxymethyl,
}

impl ProtocolCompactGroupCatalogKeyV1 {
    /// Convert the wire's closed catalog identity into the document-owned key.
    #[must_use]
    pub const fn into_document(self) -> CompactGroupCatalogKeyV1 {
        match self {
            Self::Methyl => CompactGroupCatalogKeyV1::Methyl,
            Self::Ethyl => CompactGroupCatalogKeyV1::Ethyl,
            Self::Phenyl => CompactGroupCatalogKeyV1::Phenyl,
            Self::Methoxy => CompactGroupCatalogKeyV1::Methoxy,
            Self::Nitro => CompactGroupCatalogKeyV1::Nitro,
            Self::Cyano => CompactGroupCatalogKeyV1::Cyano,
            Self::Carboxyl => CompactGroupCatalogKeyV1::Carboxyl,
            Self::AcylChloride => CompactGroupCatalogKeyV1::AcylChloride,
            Self::Hydroxymethyl => CompactGroupCatalogKeyV1::Hydroxymethyl,
        }
    }
}

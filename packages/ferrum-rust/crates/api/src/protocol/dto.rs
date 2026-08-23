//! Closed, stateless JSON operation protocol V1.
//!
//! This module owns only the portable request/response data contract.

use ferrum_document::{CdmlInspection, CdmlValidation, InterchangeFormatV1, RewriteCheck};
use ferrum_render::{
    LOCAL_PDF_COMPLETED_BYTES_V1, LOCAL_PNG_ENCODED_BYTES_V1, LOCAL_SVG_COMPLETED_BYTES_V1,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod dto_errors;
mod presentation_author_dto;
pub use dto_errors::*;
pub use presentation_author_dto::*;

/// Exact schema identifier accepted for V1 requests.
pub const OPERATION_PROTOCOL_REQUEST_SCHEMA_V1: &str = "ferrum-operation-request-v1";
/// Exact schema identifier emitted for V1 successful responses.
pub const OPERATION_PROTOCOL_RESPONSE_SCHEMA_V1: &str = "ferrum-operation-response-v1";
/// Exact schema identifier emitted for V1 protocol error responses.
pub const OPERATION_PROTOCOL_ERROR_SCHEMA_V1: &str = "ferrum-operation-error-v1";

// V1 returns one base64 completion, so its ceiling is the tightest accepted
// complete-artifact limit across the three existing native profiles.
pub(super) const MAX_ARTIFACT_BYTES_V1: usize = smallest_artifact_limit(
    LOCAL_SVG_COMPLETED_BYTES_V1,
    LOCAL_PDF_COMPLETED_BYTES_V1,
    LOCAL_PNG_ENCODED_BYTES_V1,
);
pub(super) const MAX_ARTIFACT_BASE64_BYTES_V1: usize = base64_encoded_len(MAX_ARTIFACT_BYTES_V1)
    .expect("the accepted V1 artifact limit has a representable base64 expansion");

// A JSON string can encode every ASCII CDML byte as a six-byte `\\u00XX`
// escape.  The fixed allowance covers V1's closed framing and its opaque request
// ID; it is intentionally separate from the document-source policy.
const OPERATION_PROTOCOL_V1_FRAMING_AND_REQUEST_ID_UTF8_BYTES: usize = 4 * 1024;
/// Maximum UTF-8 length of the opaque request identifier admitted by V1.
///
/// This independent cap keeps every echoed success or admitted-error envelope
/// bounded even when the surrounding transport budget has spare capacity.
pub const MAX_REQUEST_ID_UTF8_BYTES_V1: usize = 2 * 1024;
/// Maximum serialized public JSON envelope for one SMARTS query response.
///
/// The executor measures the exact canonical response-envelope bytes before a
/// CLI or PyO3 transport can deliver them. The terminal newline written by the
/// CLI is not part of the JSON envelope.
pub const DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1: usize = 1024 * 1024;
/// Maximum UTF-8 request text accepted before any JSON allocation or parsing.
///
/// This is `LOCAL_CDML_SOURCE_UTF8_BYTES_V1 * 6` for worst-case valid JSON
/// escaping of an accepted CDML source, plus the named V1 framing/request-ID
/// allowance above. It is a transport allocation boundary, not a document or
/// corpus validity rule.
pub const OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1: usize = protocol_request_limit(
    ferrum_document::LOCAL_CDML_SOURCE_UTF8_BYTES_V1,
    OPERATION_PROTOCOL_V1_FRAMING_AND_REQUEST_ID_UTF8_BYTES,
)
.expect("the accepted local CDML limit has a representable V1 request expansion");

/// One closed V1 request after its exact schema identifier has been admitted.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationProtocolRequestV1 {
    /// Exact V1 request schema identifier.
    pub schema: ProtocolRequestSchemaV1,
    /// Caller-owned opaque text echoed unchanged after envelope admission.
    pub request_id: String,
    /// The one fully typed operation to execute.
    pub operation: OperationProtocolOperationV1,
}

/// Closed V1 request schema identifiers.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolRequestSchemaV1 {
    /// `ferrum-operation-request-v1`.
    #[serde(rename = "ferrum-operation-request-v1")]
    V1,
}

/// The closed protocol operations admitted by V1.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum OperationProtocolOperationV1 {
    /// Inspect one bounded CDML document.
    #[serde(rename = "document.inspect")]
    Inspect(DocumentInspectRequestV1),
    /// Validate one bounded CDML document at the selected semantic level.
    #[serde(rename = "document.validate")]
    Validate(DocumentValidateRequestV1),
    /// Structurally rewrite one bounded CDML document.
    #[serde(rename = "document.rewrite")]
    Rewrite(DocumentRewriteRequestV1),
    /// Render one complete bounded CDML document to an in-memory artifact.
    #[serde(rename = "document.render_artifact")]
    RenderArtifact(DocumentRenderArtifactRequestV1),
    /// Convert bounded molecular interchange through an injected chemistry runtime.
    #[serde(rename = "chemistry.convert")]
    ChemistryConvert(ChemistryConvertRequestV1),
    /// Regenerate all direct typed molecule coordinates as one document transition.
    #[serde(rename = "document.generate_coordinates")]
    GenerateCoordinates(DocumentGenerateCoordinatesRequestV1),
    /// Author one closed presentation family through a request-owned Rust session.
    #[serde(rename = "presentation.author.v1")]
    PresentationAuthor(PresentationAuthorRequestV1),
    /// List immutable Ferrum-authored template catalog summary facts.
    #[serde(rename = "catalog.list.v1")]
    CatalogList(CatalogListRequestV1),
    /// Insert one catalog-selected template through Ferrum's renderer-preflighted gesture.
    #[serde(rename = "catalog.insert.v1")]
    CatalogInsert(CatalogInsertRequestV1),
    /// Create one canonical durable reaction aggregate from direct-root IDs.
    #[serde(rename = "reaction.create.v1")]
    ReactionCreate(ReactionCreateRequestV1),
    #[serde(rename = "reaction.list.v1")]
    ReactionList(ReactionObservationRequestV1),
    #[serde(rename = "reaction.observe.v1")]
    ReactionObserve(ReactionObserveRequestV1),
    #[serde(rename = "reaction.select.v1")]
    ReactionSelect(ReactionObserveRequestV1),
    #[serde(rename = "reaction.patch-membership.v1")]
    ReactionPatchMembership(ReactionPatchMembershipRequestV1),
    #[serde(rename = "reaction.delete-definition.v1")]
    ReactionDeleteDefinition(ReactionObserveRequestV1),
    /// Translate every direct durable member of one strict reaction aggregate.
    #[serde(rename = "reaction.translate.v1")]
    ReactionTranslate(ReactionTranslateRequestV1),
    /// Read bounded composition and authored-fact reports for direct molecule roots.
    #[serde(rename = "document.molecule.report.v1")]
    DocumentMoleculeReport(DocumentMoleculeReportRequestV1),
    /// Search direct molecules from one bounded, request-owned document.
    #[serde(rename = "document.molecule.smarts.query.v1")]
    DocumentSmartsQuery(DocumentSmartsQueryRequestV1),
    /// Import one explicitly selected interchange format into a new request-owned document.
    #[serde(rename = "document.molecule.interchange.import.v1")]
    DocumentMoleculeInterchangeImport(DocumentMoleculeInterchangeImportRequestV1),
}

/// Fixed new-document interchange import. No snapshot, placement, or append mode exists.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeInterchangeImportRequestV1 {
    /// Exact lower-case registry alias. No suffix sniffing or fallback occurs.
    pub format_alias: String,
    /// Request-owned UTF-8 source; no path, handle, or source identity crosses this boundary.
    pub source_utf8: String,
}

/// Fenced request-owned document supplied to the SMARTS query operation.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSmartsQueryDocumentV1 {
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
    pub document: DocumentSmartsQueryDocumentV1,
    pub query: DocumentSmartsQueryInputV1,
    #[serde(default)]
    pub limits: DocumentSmartsQueryLimitsV1,
}

/// Closed read-only molecule-report request.  The runtime and report graph are
/// intentionally absent from this transport contract.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeReportRequestV1 {
    pub document: String,
    pub expected_revision: u64,
    pub expected_digest_hex: String,
    pub molecule_ids: Vec<String>,
}

/// Request payload for `chemistry.convert`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChemistryConvertRequestV1 {
    /// Closed source interchange syntax and bounded owned text.
    pub input: ChemistryConvertInputV1,
    /// Closed target interchange syntax.
    pub output_format: InterchangeFormatV1,
}

/// Owned chemistry conversion input.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChemistryConvertInputV1 {
    /// Closed source interchange syntax.
    pub format: InterchangeFormatV1,
    /// Bounded source text; never a filesystem path or native handle.
    pub text: String,
}

/// Request payload for `document.generate_coordinates`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentGenerateCoordinatesRequestV1 {
    /// Uncompressed CDML text admitted under Ferrum's existing V1 profile.
    pub document: String,
}

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
    pub reactants: Vec<String>,
    pub products: Vec<String>,
    pub arrow: String,
    pub conditions: Vec<String>,
    pub pluses: Vec<String>,
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
    pub reaction_id: String,
}
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionPatchMembershipRequestV1 {
    pub document: String,
    pub expected_revision: u64,
    pub expected_digest_hex: String,
    pub reaction_id: String,
    pub reactants: Vec<String>,
    pub products: Vec<String>,
    pub arrow: String,
    pub conditions: Vec<String>,
    pub pluses: Vec<String>,
}
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionTranslateRequestV1 {
    pub document: String,
    pub expected_revision: u64,
    pub expected_digest_hex: String,
    pub reaction_id: String,
    pub press_x: f64,
    pub press_y: f64,
    pub pointer_x: f64,
    pub pointer_y: f64,
    pub snap: ProtocolReactionTranslationSnapV1,
}
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolReactionTranslationSnapV1 {
    Free,
    ViewHexGrid,
}

/// Request payload for `document.inspect`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentInspectRequestV1 {
    /// Uncompressed CDML text admitted under Ferrum's existing V1 profile.
    pub document: String,
}

/// Immutable request fence derived from one admitted document snapshot.
///
/// The caller keeps the original request-owned document and may submit these
/// values unchanged to a subsequent document mutation.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentRequestFenceV1 {
    /// Revision of the admitted snapshot.
    pub expected_revision: u64,
    /// Lowercase hexadecimal SHA-256 digest of the admitted snapshot.
    pub expected_digest_hex: String,
}

/// Request payload for `document.validate`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentValidateRequestV1 {
    /// Uncompressed CDML text admitted under Ferrum's existing V1 profile.
    pub document: String,
    /// The requested structural or typed validation level.
    pub level: ProtocolValidationLevelV1,
}

/// Closed validation levels exposed by V1.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolValidationLevelV1 {
    /// Retain and structurally validate CDML.
    #[serde(rename = "structural")]
    Structural,
    /// Require the existing typed Ferrum core projection.
    #[serde(rename = "typed")]
    Typed,
}

/// Request payload for `document.rewrite`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentRewriteRequestV1 {
    /// Uncompressed CDML text admitted under Ferrum's existing V1 profile.
    pub document: String,
}

/// Request payload for `document.render_artifact`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentRenderArtifactRequestV1 {
    /// Uncompressed CDML text admitted under Ferrum's existing V1 profile.
    pub document: String,
    /// The complete artifact format to render.
    pub format: ProtocolArtifactFormatV1,
}

/// Closed complete-document artifact formats exposed by V1.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolArtifactFormatV1 {
    /// Complete SVG.
    #[serde(rename = "svg")]
    Svg,
    /// Complete vector PDF.
    #[serde(rename = "pdf")]
    Pdf,
    /// Transparent PNG at one pixel per Rust page point.
    #[serde(rename = "png_one_pixel_per_point_transparent")]
    PngOnePixelPerPointTransparent,
}

/// One successful V1 response.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationProtocolResponseV1 {
    /// Exact V1 success schema identifier.
    pub schema: ProtocolResponseSchemaV1,
    /// Unchanged caller-owned request identifier.
    pub request_id: String,
    /// The typed operation result.
    pub outcome: OperationProtocolOutcomeV1,
}

/// Closed V1 success schema identifiers.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolResponseSchemaV1 {
    /// `ferrum-operation-response-v1`.
    #[serde(rename = "ferrum-operation-response-v1")]
    V1,
}

/// Tagged successful results for the V1 operation set.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum OperationProtocolOutcomeV1 {
    /// Semantic CDML inspection facts.
    #[serde(rename = "document.inspect")]
    Inspect {
        report: CdmlInspection,
        document_fence: DocumentRequestFenceV1,
    },
    /// Validation facts plus the caller-selected protocol level.
    #[serde(rename = "document.validate")]
    Validate {
        /// The selected protocol level, retained even when lower report labels differ.
        level: ProtocolValidationLevelV1,
        /// Existing Ferrum validation report.
        report: CdmlValidation,
    },
    /// Rewritten CDML plus its structural preservation report.
    #[serde(rename = "document.rewrite")]
    Rewrite {
        /// Structurally re-emitted CDML, not a byte-preservation promise.
        document: String,
        /// Existing structural rewrite-check report.
        report: RewriteCheck,
    },
    /// One complete native artifact encoded as standard base64.
    #[serde(rename = "document.render_artifact")]
    RenderArtifact {
        /// The requested closed artifact format.
        format: ProtocolArtifactFormatV1,
        /// The media type associated with `format`.
        media_type: String,
        /// Complete artifact bytes encoded with RFC 4648 standard base64.
        artifact_base64: String,
    },
    /// Converted owned molecular interchange text.
    #[serde(rename = "chemistry.convert")]
    ChemistryConvert {
        /// Closed target syntax.
        format: InterchangeFormatV1,
        /// Complete converted text.
        text: String,
        /// Number of preserved molecular records.
        record_count: usize,
    },
    /// Structural CDML after atomic coordinate regeneration.
    #[serde(rename = "document.generate_coordinates")]
    GenerateCoordinates {
        /// Structural CDML snapshot after one shared coordinate batch commit.
        document: String,
        /// Number of direct typed molecular roots regenerated.
        regenerated_molecule_count: usize,
    },
    /// Accepted authoring mutation, durable root facts, and stateless continuation.
    #[serde(rename = "presentation.author.v1")]
    PresentationAuthor {
        /// Closed authoring family that produced the accepted mutation.
        authoring_kind: PresentationAuthoringKindV1,
        /// Accepted canonical CDML for the next stateless request.
        document: String,
        /// Durable identifier of the committed direct root.
        identifier: String,
        /// Durable kind of the committed direct root.
        root_kind: String,
        /// Revision of the local session commit represented by this result.
        committed_revision: u64,
        /// Portable request fence for submitting this exact accepted document.
        document_fence: DocumentRequestFenceV1,
        /// Direct-bond facts present only for the direct-bond authoring family.
        #[serde(skip_serializing_if = "Option::is_none")]
        direct_bond: Option<PresentationAuthorDirectBondOutcomeV1>,
    },
    /// Immutable shipped catalog facts, without recipe or CDML payloads.
    #[serde(rename = "catalog.list.v1")]
    CatalogList {
        catalog_schema: String,
        catalog_version: String,
        entries: Vec<CatalogEntrySummaryV1>,
    },
    /// Canonical document after one renderer-preflighted catalog insertion.
    #[serde(rename = "catalog.insert.v1")]
    CatalogInsert {
        document: String,
        identifier: String,
        committed_revision: u64,
        /// Portable request fence for submitting this exact changed document.
        document_fence: DocumentRequestFenceV1,
    },
    #[serde(rename = "reaction.create.v1")]
    ReactionCreate {
        document: String,
        reaction_id: String,
        input_revision: u64,
        committed_revision: u64,
        next_input_expected_revision: u64,
        digest_hex: String,
    },
    #[serde(rename = "reaction.list.v1")]
    ReactionList {
        input_revision: u64,
        next_input_expected_revision: u64,
        digest_hex: String,
        reactions: Vec<ReactionObservationSummaryV1>,
    },
    #[serde(rename = "reaction.observe.v1")]
    ReactionObserve {
        input_revision: u64,
        next_input_expected_revision: u64,
        digest_hex: String,
        reaction: ReactionObservationSummaryV1,
    },
    #[serde(rename = "reaction.select.v1")]
    ReactionSelect {
        input_revision: u64,
        next_input_expected_revision: u64,
        digest_hex: String,
        reaction_id: String,
        membership_digest: String,
    },
    #[serde(rename = "reaction.patch-membership.v1")]
    ReactionPatchMembership {
        document: String,
        reaction_id: String,
        input_revision: u64,
        committed_revision: u64,
        next_input_expected_revision: u64,
        digest_hex: String,
    },
    #[serde(rename = "reaction.delete-definition.v1")]
    ReactionDeleteDefinition {
        document: String,
        reaction_id: String,
        input_revision: u64,
        committed_revision: u64,
        next_input_expected_revision: u64,
        digest_hex: String,
    },
    #[serde(rename = "reaction.translate.v1")]
    ReactionTranslate {
        document: String,
        reaction_id: String,
        input_revision: u64,
        committed_revision: u64,
        next_input_expected_revision: u64,
        digest_hex: String,
    },
    /// Read-only molecule-report facts.  No CDML, graph, engine, or path crosses this boundary.
    #[serde(rename = "document.molecule.report.v1")]
    DocumentMoleculeReport {
        report: DocumentMoleculeReportSummaryV1,
    },
    /// Bounded non-redeemable SMARTS query facts.
    #[serde(rename = "document.molecule.smarts.query.v1")]
    DocumentSmartsQuery { query: DocumentSmartsQuerySummaryV1 },
    /// Bounded interchange import summary without a document artifact or identifiers.
    #[serde(rename = "document.molecule.interchange.import.v1")]
    DocumentMoleculeInterchangeImport {
        summary: DocumentInterchangeImportSummaryV1,
    },
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

/// Public SMARTS query facts. Match membership remains private to the live bridge.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSmartsQuerySummaryV1 {
    pub schema: String,
    pub traversal: DocumentSmartsQueryTraversalSummaryV1,
    pub molecules: Vec<DocumentSmartsQueryMoleculeSummaryV1>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DocumentSmartsQueryTraversalSummaryV1 {
    Complete,
    Incomplete { reason: String },
}

/// One source-ordered target with at least one retained match. No atom identity crosses JSON.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSmartsQueryMoleculeSummaryV1 {
    pub source_order: u32,
    pub match_count: u32,
    pub completeness: String,
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

#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionObservationSummaryV1 {
    pub reaction_id: String,
    pub source_order: u32,
    pub disposition: ProtocolReactionDefinitionDispositionV1,
    pub diagnostics: Vec<String>,
    pub membership_digest: String,
    pub members: Vec<ReactionMemberSummaryV1>,
    pub union_bounds: Option<ReactionBoundsSummaryV1>,
}
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionMemberSummaryV1 {
    pub identifier: String,
    pub role: String,
    pub role_ordinal: u32,
    pub source_order: u32,
    pub bounds: Option<ReactionBoundsSummaryV1>,
}
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionBoundsSummaryV1 {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
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

/// A completed operation response or typed protocol refusal.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum OperationProtocolEnvelopeV1 {
    /// Successful response.
    Success(OperationProtocolResponseV1),
    /// Typed refusal.
    Error(OperationProtocolErrorResponseV1),
}

/// Failure before a JSON response envelope can exist.
#[derive(Debug, Error)]
pub enum OperationProtocolInputErrorV1 {
    /// Request text exceeded the derived V1 ingress budget before JSON parsing.
    #[error(
        "operation request exceeds the {limit}-byte V1 ingress limit ({observed} bytes observed)"
    )]
    ResourceLimit {
        /// Derived V1 request transport budget.
        limit: usize,
        /// Exact UTF-8 request length observed before parsing.
        observed: usize,
    },
    /// Input was not valid JSON text.
    #[error("operation input is not valid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
}

pub(super) const fn base64_encoded_len(byte_len: usize) -> Option<usize> {
    match byte_len.checked_add(2) {
        Some(rounded) => match rounded.checked_div(3) {
            Some(groups) => groups.checked_mul(4),
            None => None,
        },
        None => None,
    }
}

const fn smallest_artifact_limit(first: usize, second: usize, third: usize) -> usize {
    let smaller = if first < second { first } else { second };
    if smaller < third { smaller } else { third }
}

const fn protocol_request_limit(
    document_source_bytes: usize,
    framing_and_request_id_bytes: usize,
) -> Option<usize> {
    match document_source_bytes.checked_mul(6) {
        Some(escaped_document_bytes) => {
            escaped_document_bytes.checked_add(framing_and_request_id_bytes)
        }
        None => None,
    }
}

impl OperationProtocolOperationV1 {
    pub(super) const fn kind(&self) -> ProtocolOperationKindV1 {
        match self {
            Self::Inspect(_) => ProtocolOperationKindV1::Inspect,
            Self::Validate(_) => ProtocolOperationKindV1::Validate,
            Self::Rewrite(_) => ProtocolOperationKindV1::Rewrite,
            Self::RenderArtifact(_) => ProtocolOperationKindV1::RenderArtifact,
            Self::ChemistryConvert(_) => ProtocolOperationKindV1::ChemistryConvert,
            Self::GenerateCoordinates(_) => ProtocolOperationKindV1::GenerateCoordinates,
            Self::PresentationAuthor(_) => ProtocolOperationKindV1::PresentationAuthor,
            Self::CatalogList(_) => ProtocolOperationKindV1::CatalogList,
            Self::CatalogInsert(_) => ProtocolOperationKindV1::CatalogInsert,
            Self::ReactionCreate(_) => ProtocolOperationKindV1::ReactionCreate,
            Self::ReactionList(_) => ProtocolOperationKindV1::ReactionList,
            Self::ReactionObserve(_) => ProtocolOperationKindV1::ReactionObserve,
            Self::ReactionSelect(_) => ProtocolOperationKindV1::ReactionSelect,
            Self::ReactionPatchMembership(_) => ProtocolOperationKindV1::ReactionPatchMembership,
            Self::ReactionDeleteDefinition(_) => ProtocolOperationKindV1::ReactionDeleteDefinition,
            Self::ReactionTranslate(_) => ProtocolOperationKindV1::ReactionTranslate,
            Self::DocumentMoleculeReport(_) => ProtocolOperationKindV1::DocumentMoleculeReport,
            Self::DocumentSmartsQuery(_) => ProtocolOperationKindV1::DocumentSmartsQuery,
            Self::DocumentMoleculeInterchangeImport(_) => {
                ProtocolOperationKindV1::DocumentMoleculeInterchangeImport
            }
        }
    }
}

//! DTOs for general bounded document transformations and rendering.

use ferrum_document::InterchangeFormatV1;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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

/// Request payload for decoded-semantic `interchange.inspect_graph`.
///
/// The request owns already-bounded source text and never carries a path,
/// stream, document, or runtime handle. Unsupported resolved formats return a
/// typed refusal rather than a partial graph summary.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InspectInterchangeGraphRequestV1 {
    /// Closed input format and owned bounded source text to inspect.
    pub input: InspectInterchangeGraphInputV1,
}

/// Owned source input for declared graph inspection profiles.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InspectInterchangeGraphInputV1 {
    /// Resolved closed interchange format; declared CML and native SDF profiles are admitted.
    pub format: InterchangeFormatV1,
    /// Bounded UTF-8 source text owned by this request, never a path or native handle.
    pub text: String,
}

/// Whether a source fact was supplied, omitted, or outside the inspection profile.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SourceFactV1<T> {
    /// The source explicitly supplied this fact.
    Known { value: T },
    /// The profile can represent this fact but the source omitted it.
    Unknown,
    /// The profile does not inspect or preserve this fact.
    Unsupported,
}

/// Declared inspection availability for one source fact category.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectGraphFactCoverageStatusV1 {
    /// The profile always reports this fact.
    Known,
    /// The profile reports this fact only when the source supplies it.
    UnknownWhenOmitted,
    /// The profile does not inspect this fact.
    Unsupported,
}

/// Per-category source-fact coverage declared by the selected inspection profile.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InspectGraphFactCoverageV1 {
    /// Source record ordering coverage.
    pub source_record_ordering: InspectGraphFactCoverageStatusV1,
    /// Per-record atom-count coverage.
    pub atom_count: InspectGraphFactCoverageStatusV1,
    /// Per-record bond-count coverage.
    pub bond_count: InspectGraphFactCoverageStatusV1,
    /// Atom source-ID coverage.
    pub atom_source_id: InspectGraphFactCoverageStatusV1,
    /// Element coverage.
    pub element: InspectGraphFactCoverageStatusV1,
    /// Coordinate coverage.
    pub coordinates: InspectGraphFactCoverageStatusV1,
    /// Bond-endpoint coverage.
    pub bond_endpoints: InspectGraphFactCoverageStatusV1,
    /// Bond-order coverage.
    pub bond_order: InspectGraphFactCoverageStatusV1,
    /// Molecule source-ID coverage.
    pub source_molecule_id: InspectGraphFactCoverageStatusV1,
    /// Formal-charge coverage.
    pub formal_charge: InspectGraphFactCoverageStatusV1,
    /// Isotope coverage.
    pub isotope: InspectGraphFactCoverageStatusV1,
    /// Bond source-ID coverage.
    pub bond_source_id: InspectGraphFactCoverageStatusV1,
    /// Bond-stereo-direction coverage.
    pub bond_stereo_direction: InspectGraphFactCoverageStatusV1,
    /// Radical coverage.
    pub radicals: InspectGraphFactCoverageStatusV1,
    /// Atom-label and property coverage.
    pub atom_labels_properties: InspectGraphFactCoverageStatusV1,
    /// Reaction atom-map coverage.
    pub reaction_atom_maps: InspectGraphFactCoverageStatusV1,
    /// Record source-ID coverage.
    pub record_source_id: InspectGraphFactCoverageStatusV1,
    /// Record display-title coverage.
    pub record_title: InspectGraphFactCoverageStatusV1,
    /// Ordered record-property coverage.
    pub property_count: InspectGraphFactCoverageStatusV1,
    /// Aromaticity coverage.
    pub aromaticity: InspectGraphFactCoverageStatusV1,
    /// Stereo coverage.
    pub stereo: InspectGraphFactCoverageStatusV1,
}

/// Bounded graph counts for one source record in source order.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InspectInterchangeGraphRecordSummaryV1 {
    /// Zero-based decoded record index, the only cross-format record identity.
    pub record_index: u32,
    /// Source record ID when the selected profile retains one.
    pub record_source_id: SourceFactV1<String>,
    /// Display title when the selected profile retains one; never an identity.
    pub record_title: SourceFactV1<String>,
    /// Ordered property count when the selected profile retains properties.
    pub property_count: SourceFactV1<u32>,
    /// Exact atom count for this source record.
    pub atom_count: u32,
    /// Exact bond count for this source record.
    pub bond_count: u32,
}

/// Bounded decoded-semantic graph summary returned by `interchange.inspect_graph`.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InspectInterchangeGraphSummaryV1 {
    /// Versioned summary schema identifier.
    pub schema: String,
    /// Resolver-owned public format identifier.
    pub format_id: String,
    /// Resolver-owned declared inspection profile identifier.
    pub profile_id: String,
    /// The semantic interpretation shared by all admitted inspection profiles.
    pub graph_meaning: String,
    /// Number of records represented in `records`.
    pub record_count: u32,
    /// Checked aggregate atom count across all records.
    pub atom_count: u32,
    /// Checked aggregate bond count across all records.
    pub bond_count: u32,
    /// Per-record summaries in retained zero-based source order.
    pub records: Vec<InspectInterchangeGraphRecordSummaryV1>,
    /// Profile-declared known, unknown-when-omitted, and unsupported fact categories.
    pub declared_fact_coverage: InspectGraphFactCoverageV1,
    /// Profile-declared normalization and source-fidelity boundary.
    pub normalization: InspectGraphNormalizationV1,
}

/// Profile-specific normalization disclosure for a decoded semantic graph.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InspectGraphNormalizationV1 {
    /// Coordinate-space meaning for the decoded graph.
    pub source_coordinate_space: String,
    /// Graph normalization policy.
    pub graph_normalization: String,
    /// Aromaticity interpretation.
    pub aromaticity: String,
    /// Stereo interpretation.
    pub stereo: String,
    /// Whether raw source representation fidelity is claimed.
    pub raw_source_fidelity: String,
}

/// Request payload for `document.generate_coordinates`.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentGenerateCoordinatesRequestV1 {
    /// Uncompressed CDML text admitted under Ferrum's existing V1 profile.
    pub document: String,
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

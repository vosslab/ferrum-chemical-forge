//! Closed Rust-owned interchange registry and transport foundations.
//!
//! This module intentionally has no XML parser, document transaction, CLI
//! command, PyO3 receipt, or Qt dependency.  It freezes the values those later
//! layers must join rather than allowing each adapter to maintain its own
//! format table.

use ferrum_chemistry::{
    CDXML_SIMPLE_MOLECULE_IMPORT_FORMAT_ID_V1 as CHEMISTRY_CDXML_FORMAT_ID_V1,
    CDXML_SIMPLE_MOLECULE_IMPORT_MAX_SOURCE_BYTES_V1,
    CDXML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1 as CHEMISTRY_CDXML_PROFILE_ID_V1,
    CML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1, SDF_MAX_INPUT_BYTES,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Exact CML profile selected by the static format descriptor.
pub const CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1: &str = CML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1;
/// Exact API format identifier selected by the static format descriptor.
pub const CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1: &str = "cml_simple_molecule_import_v1";
/// Exact API format identifier selected by the Rust-owned CDXML descriptor.
pub const CDXML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1: &str = CHEMISTRY_CDXML_FORMAT_ID_V1;
/// Exact CDXML profile selected by the static format descriptor.
pub const CDXML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1: &str = CHEMISTRY_CDXML_PROFILE_ID_V1;
/// Exact API format identifier selected by the SDF V1 descriptor.
pub const SDF_IMPORT_FORMAT_V1: &str = "sdf_v1";
/// Exact SDF profile selected by the static format descriptor.
pub const SDF_IMPORT_PROFILE_V1: &str = "sdf_v1";
/// API-owned source cap for the CML simple-molecule import profile.
const CML_IMPORT_MAX_SOURCE_BYTES_V1: usize = 1_048_576;
const INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1: usize = 1_048_576;

/// Product-facing disposition for one locally selected document source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalDocumentIngressDirectionV1 {
    ReplacePristineOrNewTab,
    NewDocumentOnly,
}

/// Decoder selected by the closed local-document ingress registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalDocumentIngressDecoderV1 {
    Cdml,
    DecodedCdsvg,
    CmlSimpleMolecule,
    CdxmlSimpleMolecule,
}

/// Stable route identity issued to desktop adapters for an accepted source.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalDocumentIngressRouteV1 {
    Cdml,
    DecodedCdsvg,
    CmlSimpleMolecule,
    CdxmlSimpleMolecule,
}

impl LocalDocumentIngressRouteV1 {
    #[must_use]
    pub const fn source_kind(self) -> &'static str {
        match self {
            Self::Cdml => "cdml",
            Self::DecodedCdsvg => "decoded_cdsvg",
            Self::CmlSimpleMolecule => "cml",
            Self::CdxmlSimpleMolecule => "cdxml",
        }
    }
}

/// Accepted local-document route facts owned by the API layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalDocumentIngressDescriptorV1 {
    display_name: &'static str,
    suffixes: &'static [&'static str],
    direction: LocalDocumentIngressDirectionV1,
    decoder: LocalDocumentIngressDecoderV1,
    route: LocalDocumentIngressRouteV1,
    profile_id: &'static str,
}

impl LocalDocumentIngressDescriptorV1 {
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        self.display_name
    }
    #[must_use]
    pub const fn suffixes(self) -> &'static [&'static str] {
        self.suffixes
    }
    #[must_use]
    pub const fn direction(self) -> LocalDocumentIngressDirectionV1 {
        self.direction
    }
    #[must_use]
    pub const fn decoder(self) -> LocalDocumentIngressDecoderV1 {
        self.decoder
    }
    #[must_use]
    pub const fn route(self) -> LocalDocumentIngressRouteV1 {
        self.route
    }
    #[must_use]
    pub const fn profile_id(self) -> &'static str {
        self.profile_id
    }
}

/// Stable typed refusal identity for a known-but-closed local source form.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalDocumentIngressRefusalV1 {
    category: &'static str,
    recovery: &'static str,
}

impl LocalDocumentIngressRefusalV1 {
    #[must_use]
    pub const fn category(self) -> &'static str {
        self.category
    }
    #[must_use]
    pub const fn recovery(self) -> &'static str {
        self.recovery
    }
}

const LOCAL_CDML_SUFFIXES_V1: [&str; 1] = [".cdml"];
const LOCAL_CDSVG_SUFFIXES_V1: [&str; 1] = [".svg"];
const LOCAL_CML_SUFFIXES_V1: [&str; 1] = [".cml"];
const LOCAL_CDXML_SUFFIXES_V1: [&str; 1] = [".cdxml"];
const LOCAL_REFUSAL_V1: LocalDocumentIngressRefusalV1 = LocalDocumentIngressRefusalV1 {
    category: "unsupported_local_document",
    recovery: "choose_supported_format",
};
const LOCAL_DOCUMENT_INGRESS_DESCRIPTORS_V1: [LocalDocumentIngressDescriptorV1; 4] = [
    LocalDocumentIngressDescriptorV1 {
        display_name: "Ferrum CDML",
        suffixes: &LOCAL_CDML_SUFFIXES_V1,
        direction: LocalDocumentIngressDirectionV1::ReplacePristineOrNewTab,
        decoder: LocalDocumentIngressDecoderV1::Cdml,
        route: LocalDocumentIngressRouteV1::Cdml,
        profile_id: "local_cdml_v1",
    },
    LocalDocumentIngressDescriptorV1 {
        display_name: "SVG with embedded CDML",
        suffixes: &LOCAL_CDSVG_SUFFIXES_V1,
        direction: LocalDocumentIngressDirectionV1::ReplacePristineOrNewTab,
        decoder: LocalDocumentIngressDecoderV1::DecodedCdsvg,
        route: LocalDocumentIngressRouteV1::DecodedCdsvg,
        profile_id: "local_decoded_cdsvg_v1",
    },
    LocalDocumentIngressDescriptorV1 {
        display_name: "Chemical Markup Language (CML/CML2)",
        suffixes: &LOCAL_CML_SUFFIXES_V1,
        direction: LocalDocumentIngressDirectionV1::NewDocumentOnly,
        decoder: LocalDocumentIngressDecoderV1::CmlSimpleMolecule,
        route: LocalDocumentIngressRouteV1::CmlSimpleMolecule,
        profile_id: CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1,
    },
    LocalDocumentIngressDescriptorV1 {
        display_name: "ChemDraw XML (CDXML)",
        suffixes: &LOCAL_CDXML_SUFFIXES_V1,
        direction: LocalDocumentIngressDirectionV1::NewDocumentOnly,
        decoder: LocalDocumentIngressDecoderV1::CdxmlSimpleMolecule,
        route: LocalDocumentIngressRouteV1::CdxmlSimpleMolecule,
        profile_id: CDXML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1,
    },
];

/// The sole API-owned product contract for local document ingress.
pub struct LocalDocumentIngressRegistryV1;

impl LocalDocumentIngressRegistryV1 {
    #[must_use]
    pub const fn descriptors() -> &'static [LocalDocumentIngressDescriptorV1] {
        &LOCAL_DOCUMENT_INGRESS_DESCRIPTORS_V1
    }

    #[must_use]
    pub fn lookup_suffix(suffix: &str) -> Option<&'static LocalDocumentIngressDescriptorV1> {
        Self::descriptors()
            .iter()
            .find(|descriptor| descriptor.suffixes().contains(&suffix))
    }

    #[must_use]
    pub fn refusal_for_suffix(suffix: &str) -> Option<LocalDocumentIngressRefusalV1> {
        match suffix {
            ".cdsvg" | ".svgz" | ".gz" | ".bz2" | ".xz" | ".zip" | ".zst" => Some(LOCAL_REFUSAL_V1),
            _ => None,
        }
    }

    pub fn validate_exact_join() -> Result<(), InterchangeImportRefusalV1> {
        let cml = Self::lookup_suffix(".cml");
        let cdxml = Self::lookup_suffix(".cdxml");
        if cml.is_some_and(|descriptor| {
            descriptor.decoder() == LocalDocumentIngressDecoderV1::CmlSimpleMolecule
                && descriptor.direction() == LocalDocumentIngressDirectionV1::NewDocumentOnly
                && descriptor.profile_id() == CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1
        }) && cdxml.is_some_and(|descriptor| {
            descriptor.decoder() == LocalDocumentIngressDecoderV1::CdxmlSimpleMolecule
                && descriptor.direction() == LocalDocumentIngressDirectionV1::NewDocumentOnly
                && descriptor.profile_id() == CDXML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1
        }) {
            Ok(())
        } else {
            Err(InterchangeImportRefusalV1::for_reason(
                InterchangeImportRefusalReasonV1::InternalFailure,
            ))
        }
    }
}

/// Closed operations advertised by a Ferrum interchange descriptor.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterchangeOperationV1 {
    ChemistryConvert,
    DocumentImportNew,
}

/// Typed refusal issued when a known input does not admit one requested operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterchangeOperationRefusalV1 {
    requested_operation: InterchangeOperationV1,
    supported_operations: &'static [InterchangeOperationV1],
}

impl InterchangeOperationRefusalV1 {
    #[must_use]
    pub const fn new(
        requested_operation: InterchangeOperationV1,
        supported_operations: &'static [InterchangeOperationV1],
    ) -> Self {
        Self {
            requested_operation,
            supported_operations,
        }
    }

    #[must_use]
    pub const fn requested_operation(self) -> InterchangeOperationV1 {
        self.requested_operation
    }

    #[must_use]
    pub const fn supported_operations(self) -> &'static [InterchangeOperationV1] {
        self.supported_operations
    }

    #[must_use]
    pub const fn recovery_message(self) -> &'static str {
        match self.requested_operation {
            InterchangeOperationV1::ChemistryConvert => {
                "this source creates a new document; use ferrum open"
            }
            InterchangeOperationV1::DocumentImportNew => {
                "this source is not available for opening a new document"
            }
        }
    }
}

/// Closed compression policy.  interchange import never implicitly decompresses input.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterchangeCompressionPolicyV1 {
    Forbidden,
}

/// Closed loss policy.  This profile refuses any fact it cannot represent.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterchangeSemanticLossPolicyV1 {
    RejectUnrepresentedSemantics,
}

/// Closed native decoder selected by one static interchange descriptor.
///
/// This is deliberately not a string or callback. The API exhaustively owns
/// every enabled decoder, so adapters cannot invent a format branch.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterchangeDecoderKeyV1 {
    CmlSimpleMolecule,
    CdxmlSimpleMolecule,
    Sdf,
}

/// Whether a closed interchange capability needs the native chemistry runtime.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterchangeRuntimeRequirementV1 {
    RuntimeFree,
    RuntimeRequired,
}

impl InterchangeRuntimeRequirementV1 {
    #[must_use]
    pub const fn requires_chemistry_runtime(self) -> bool {
        matches!(self, Self::RuntimeRequired)
    }
}

/// Conversion facts selected by one closed input capability.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct ConversionInputProfileV1 {
    max_source_bytes: usize,
    runtime_requirement: InterchangeRuntimeRequirementV1,
}

impl ConversionInputProfileV1 {
    #[must_use]
    pub const fn new(
        max_source_bytes: usize,
        runtime_requirement: InterchangeRuntimeRequirementV1,
    ) -> Self {
        Self {
            max_source_bytes,
            runtime_requirement,
        }
    }

    #[must_use]
    pub const fn max_source_bytes(self) -> usize {
        self.max_source_bytes
    }

    #[must_use]
    pub const fn runtime_requirement(self) -> InterchangeRuntimeRequirementV1 {
        self.runtime_requirement
    }
}

/// Resolved execution facts for one conversion request.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct ConversionExecutionProfileV1 {
    max_source_bytes: usize,
    runtime_requirement: InterchangeRuntimeRequirementV1,
}

impl ConversionExecutionProfileV1 {
    #[must_use]
    pub const fn join(
        input: ConversionInputProfileV1,
        output_requirement: InterchangeRuntimeRequirementV1,
    ) -> Self {
        let runtime_requirement = match (input.runtime_requirement(), output_requirement) {
            (
                InterchangeRuntimeRequirementV1::RuntimeFree,
                InterchangeRuntimeRequirementV1::RuntimeFree,
            ) => InterchangeRuntimeRequirementV1::RuntimeFree,
            _ => InterchangeRuntimeRequirementV1::RuntimeRequired,
        };
        Self {
            max_source_bytes: input.max_source_bytes(),
            runtime_requirement,
        }
    }

    #[must_use]
    pub const fn max_source_bytes(self) -> usize {
        self.max_source_bytes
    }

    #[must_use]
    pub const fn requires_chemistry_runtime(self) -> bool {
        self.runtime_requirement.requires_chemistry_runtime()
    }
}

/// Source and final-response bounds selected by an interchange descriptor.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct InterchangeImportLimitsV1 {
    max_source_bytes: usize,
    max_response_bytes: usize,
}

impl InterchangeImportLimitsV1 {
    #[must_use]
    pub const fn new(max_source_bytes: usize, max_response_bytes: usize) -> Self {
        Self {
            max_source_bytes,
            max_response_bytes,
        }
    }

    #[must_use]
    pub const fn max_source_bytes(self) -> usize {
        self.max_source_bytes
    }

    #[must_use]
    pub const fn max_response_bytes(self) -> usize {
        self.max_response_bytes
    }
}

/// One static descriptor consumed by CLI and Qt surfaces.
#[derive(Debug, JsonSchema, Serialize)]
pub struct InterchangeFormatDescriptorV1 {
    canonical_name: &'static str,
    display_name: &'static str,
    format_id: &'static str,
    profile_id: &'static str,
    input_aliases: &'static [&'static str],
    input_suffixes: &'static [&'static str],
    operations: &'static [InterchangeOperationV1],
    output_suffixes: &'static [&'static str],
    compression: InterchangeCompressionPolicyV1,
    semantic_loss_policy: InterchangeSemanticLossPolicyV1,
    decoder: InterchangeDecoderKeyV1,
    #[serde(skip)]
    #[schemars(skip)]
    graph_inspection_profile: Option<crate::InterchangeGraphInspectionProfileV1>,
    limits: InterchangeImportLimitsV1,
    conversion_profile: Option<ConversionInputProfileV1>,
}

impl InterchangeFormatDescriptorV1 {
    #[must_use]
    pub const fn canonical_name(&self) -> &'static str {
        self.canonical_name
    }

    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        self.display_name
    }
    #[must_use]
    pub const fn format_id(&self) -> &'static str {
        self.format_id
    }
    #[must_use]
    pub const fn profile_id(&self) -> &'static str {
        self.profile_id
    }
    #[must_use]
    pub const fn input_aliases(&self) -> &'static [&'static str] {
        self.input_aliases
    }
    #[must_use]
    pub const fn input_suffixes(&self) -> &'static [&'static str] {
        self.input_suffixes
    }
    #[must_use]
    pub const fn operations(&self) -> &'static [InterchangeOperationV1] {
        self.operations
    }
    #[must_use]
    pub fn supports_operation(&self, operation: InterchangeOperationV1) -> bool {
        self.operations.contains(&operation)
    }
    #[must_use]
    pub const fn output_suffixes(&self) -> &'static [&'static str] {
        self.output_suffixes
    }
    #[must_use]
    pub const fn compression(&self) -> InterchangeCompressionPolicyV1 {
        self.compression
    }
    #[must_use]
    pub const fn semantic_loss_policy(&self) -> InterchangeSemanticLossPolicyV1 {
        self.semantic_loss_policy
    }
    #[must_use]
    pub const fn decoder(&self) -> InterchangeDecoderKeyV1 {
        self.decoder
    }
    /// Return the explicitly admitted decoded-graph inspection profile.
    #[must_use]
    pub const fn graph_inspection_profile(
        &self,
    ) -> Option<crate::InterchangeGraphInspectionProfileV1> {
        self.graph_inspection_profile
    }
    #[must_use]
    pub const fn limits(&self) -> InterchangeImportLimitsV1 {
        self.limits
    }
    #[must_use]
    pub const fn conversion_profile(&self) -> Option<ConversionInputProfileV1> {
        self.conversion_profile
    }
}

const CML_EXPECTED_INPUT_ALIASES_V1: [&str; 3] = ["cml", "cml1", "cml2"];
const CML_EXPECTED_INPUT_SUFFIXES_V1: [&str; 1] = [".cml"];
const CDXML_EXPECTED_INPUT_ALIASES_V1: [&str; 1] = ["cdxml"];
const CDXML_EXPECTED_INPUT_SUFFIXES_V1: [&str; 1] = [".cdxml"];
const NO_OUTPUT_SUFFIXES_V1: [&str; 0] = [];
const DOCUMENT_IMPORT_NEW_OPERATIONS_V1: [InterchangeOperationV1; 1] =
    [InterchangeOperationV1::DocumentImportNew];
const DOCUMENT_IMPORT_AND_CHEMISTRY_CONVERT_OPERATIONS_V1: [InterchangeOperationV1; 2] = [
    InterchangeOperationV1::DocumentImportNew,
    InterchangeOperationV1::ChemistryConvert,
];
const CML_DESCRIPTOR_V1: InterchangeFormatDescriptorV1 = InterchangeFormatDescriptorV1 {
    canonical_name: "cml",
    display_name: "Chemical Markup Language (CML/CML2)",
    format_id: CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1,
    profile_id: CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1,
    input_aliases: &CML_EXPECTED_INPUT_ALIASES_V1,
    input_suffixes: &CML_EXPECTED_INPUT_SUFFIXES_V1,
    operations: &DOCUMENT_IMPORT_AND_CHEMISTRY_CONVERT_OPERATIONS_V1,
    output_suffixes: &NO_OUTPUT_SUFFIXES_V1,
    compression: InterchangeCompressionPolicyV1::Forbidden,
    semantic_loss_policy: InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics,
    decoder: InterchangeDecoderKeyV1::CmlSimpleMolecule,
    graph_inspection_profile: Some(crate::InterchangeGraphInspectionProfileV1::CmlSimpleMolecule),
    limits: InterchangeImportLimitsV1::new(
        CML_IMPORT_MAX_SOURCE_BYTES_V1,
        INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1,
    ),
    conversion_profile: Some(ConversionInputProfileV1::new(
        CML_IMPORT_MAX_SOURCE_BYTES_V1,
        InterchangeRuntimeRequirementV1::RuntimeFree,
    )),
};
const CDXML_DESCRIPTOR_V1: InterchangeFormatDescriptorV1 = InterchangeFormatDescriptorV1 {
    canonical_name: "cdxml",
    display_name: "ChemDraw XML (CDXML)",
    format_id: CDXML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1,
    profile_id: CDXML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1,
    input_aliases: &CDXML_EXPECTED_INPUT_ALIASES_V1,
    input_suffixes: &CDXML_EXPECTED_INPUT_SUFFIXES_V1,
    operations: &DOCUMENT_IMPORT_NEW_OPERATIONS_V1,
    output_suffixes: &NO_OUTPUT_SUFFIXES_V1,
    compression: InterchangeCompressionPolicyV1::Forbidden,
    semantic_loss_policy: InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics,
    decoder: InterchangeDecoderKeyV1::CdxmlSimpleMolecule,
    graph_inspection_profile: None,
    limits: InterchangeImportLimitsV1::new(
        CDXML_SIMPLE_MOLECULE_IMPORT_MAX_SOURCE_BYTES_V1,
        INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1,
    ),
    conversion_profile: None,
};
const SDF_EXPECTED_INPUT_ALIASES_V1: [&str; 2] = ["sdf", "sd"];
const SDF_EXPECTED_INPUT_SUFFIXES_V1: [&str; 2] = [".sdf", ".sd"];
const SDF_DESCRIPTOR_V1: InterchangeFormatDescriptorV1 = InterchangeFormatDescriptorV1 {
    canonical_name: "sdf",
    display_name: "Structure Data File (SDF)",
    format_id: SDF_IMPORT_FORMAT_V1,
    profile_id: SDF_IMPORT_PROFILE_V1,
    input_aliases: &SDF_EXPECTED_INPUT_ALIASES_V1,
    input_suffixes: &SDF_EXPECTED_INPUT_SUFFIXES_V1,
    operations: &DOCUMENT_IMPORT_AND_CHEMISTRY_CONVERT_OPERATIONS_V1,
    output_suffixes: &NO_OUTPUT_SUFFIXES_V1,
    compression: InterchangeCompressionPolicyV1::Forbidden,
    semantic_loss_policy: InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics,
    decoder: InterchangeDecoderKeyV1::Sdf,
    graph_inspection_profile: Some(crate::InterchangeGraphInspectionProfileV1::SdfNativeSemantic),
    limits: InterchangeImportLimitsV1::new(
        SDF_MAX_INPUT_BYTES,
        INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1,
    ),
    conversion_profile: Some(ConversionInputProfileV1::new(
        SDF_MAX_INPUT_BYTES,
        InterchangeRuntimeRequirementV1::RuntimeRequired,
    )),
};

/// The sole static API-owned interchange registry.
pub struct InterchangeFormatRegistryV1;

impl InterchangeFormatRegistryV1 {
    /// Return every enabled descriptor in deterministic API order.
    #[must_use]
    pub const fn descriptors() -> &'static [InterchangeFormatDescriptorV1] {
        &[CML_DESCRIPTOR_V1, CDXML_DESCRIPTOR_V1, SDF_DESCRIPTOR_V1]
    }

    /// Resolve one exact lower-case input alias without guessing or suffix fallback.
    pub fn lookup_input_alias(
        alias: &str,
    ) -> Result<&'static InterchangeFormatDescriptorV1, InterchangeImportRefusalV1> {
        Self::descriptors()
            .iter()
            .find(|descriptor| descriptor.input_aliases.contains(&alias))
            .ok_or_else(|| {
                InterchangeImportRefusalV1::for_reason(
                    InterchangeImportRefusalReasonV1::FormatAliasUnsupported,
                )
            })
    }

    /// Resolve one exact lower-case suffix without filename inference.
    pub fn lookup_input_suffix(
        suffix: &str,
    ) -> Result<&'static InterchangeFormatDescriptorV1, InterchangeImportRefusalV1> {
        Self::descriptors()
            .iter()
            .find(|descriptor| descriptor.input_suffixes.contains(&suffix))
            .ok_or_else(|| {
                InterchangeImportRefusalV1::for_reason(
                    InterchangeImportRefusalReasonV1::FormatAliasUnsupported,
                )
            })
    }

    /// Prove the API descriptor exactly joins the chemistry profile and document targets.
    pub fn validate_exact_join() -> Result<(), InterchangeImportRefusalV1> {
        let document_import_and_chemistry_convert = [
            InterchangeOperationV1::DocumentImportNew,
            InterchangeOperationV1::ChemistryConvert,
        ];
        let document_import_only = [InterchangeOperationV1::DocumentImportNew];
        if CML_DESCRIPTOR_V1.display_name == "Chemical Markup Language (CML/CML2)"
            && CML_DESCRIPTOR_V1.format_id == CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1
            && CML_DESCRIPTOR_V1.profile_id == CML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1
            && CML_DESCRIPTOR_V1.input_aliases == CML_EXPECTED_INPUT_ALIASES_V1
            && CML_DESCRIPTOR_V1.input_suffixes == CML_EXPECTED_INPUT_SUFFIXES_V1
            && CML_DESCRIPTOR_V1.operations == document_import_and_chemistry_convert
            && CML_DESCRIPTOR_V1.output_suffixes == NO_OUTPUT_SUFFIXES_V1
            && CDXML_DESCRIPTOR_V1.display_name == "ChemDraw XML (CDXML)"
            && CDXML_DESCRIPTOR_V1.format_id == CHEMISTRY_CDXML_FORMAT_ID_V1
            && CDXML_DESCRIPTOR_V1.profile_id == CHEMISTRY_CDXML_PROFILE_ID_V1
            && CDXML_DESCRIPTOR_V1.input_aliases == CDXML_EXPECTED_INPUT_ALIASES_V1
            && CDXML_DESCRIPTOR_V1.input_suffixes == CDXML_EXPECTED_INPUT_SUFFIXES_V1
            && CDXML_DESCRIPTOR_V1.operations == document_import_only
            && CDXML_DESCRIPTOR_V1.output_suffixes == NO_OUTPUT_SUFFIXES_V1
            && SDF_DESCRIPTOR_V1.format_id == SDF_IMPORT_FORMAT_V1
            && SDF_DESCRIPTOR_V1.profile_id == SDF_IMPORT_PROFILE_V1
            && SDF_DESCRIPTOR_V1.input_aliases == SDF_EXPECTED_INPUT_ALIASES_V1
            && SDF_DESCRIPTOR_V1.input_suffixes == SDF_EXPECTED_INPUT_SUFFIXES_V1
            && SDF_DESCRIPTOR_V1.operations == document_import_and_chemistry_convert
            && SDF_DESCRIPTOR_V1.output_suffixes == NO_OUTPUT_SUFFIXES_V1
            && CML_DESCRIPTOR_V1.compression == InterchangeCompressionPolicyV1::Forbidden
            && SDF_DESCRIPTOR_V1.compression == InterchangeCompressionPolicyV1::Forbidden
            && CDXML_DESCRIPTOR_V1.compression == InterchangeCompressionPolicyV1::Forbidden
            && CML_DESCRIPTOR_V1.semantic_loss_policy
                == InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics
            && SDF_DESCRIPTOR_V1.semantic_loss_policy
                == InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics
            && CDXML_DESCRIPTOR_V1.semantic_loss_policy
                == InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics
            && CML_DESCRIPTOR_V1.decoder == InterchangeDecoderKeyV1::CmlSimpleMolecule
            && CDXML_DESCRIPTOR_V1.decoder == InterchangeDecoderKeyV1::CdxmlSimpleMolecule
            && SDF_DESCRIPTOR_V1.decoder == InterchangeDecoderKeyV1::Sdf
            && CML_DESCRIPTOR_V1.graph_inspection_profile
                == Some(crate::InterchangeGraphInspectionProfileV1::CmlSimpleMolecule)
            && SDF_DESCRIPTOR_V1.graph_inspection_profile
                == Some(crate::InterchangeGraphInspectionProfileV1::SdfNativeSemantic)
            && CDXML_DESCRIPTOR_V1.graph_inspection_profile.is_none()
            && CML_DESCRIPTOR_V1.limits.max_source_bytes == CML_IMPORT_MAX_SOURCE_BYTES_V1
            && SDF_DESCRIPTOR_V1.limits.max_source_bytes == SDF_MAX_INPUT_BYTES
            && CDXML_DESCRIPTOR_V1.limits.max_source_bytes
                == CDXML_SIMPLE_MOLECULE_IMPORT_MAX_SOURCE_BYTES_V1
            && CML_DESCRIPTOR_V1.limits.max_response_bytes
                == INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1
            && SDF_DESCRIPTOR_V1.limits.max_response_bytes
                == INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1
            && CDXML_DESCRIPTOR_V1.limits.max_response_bytes
                == INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1
            && CML_DESCRIPTOR_V1.conversion_profile.is_some_and(|profile| {
                profile.max_source_bytes == CML_DESCRIPTOR_V1.limits.max_source_bytes
                    && profile.runtime_requirement == InterchangeRuntimeRequirementV1::RuntimeFree
            })
            && SDF_DESCRIPTOR_V1.conversion_profile.is_some_and(|profile| {
                profile.max_source_bytes == SDF_DESCRIPTOR_V1.limits.max_source_bytes
                    && profile.runtime_requirement
                        == InterchangeRuntimeRequirementV1::RuntimeRequired
            })
            && CDXML_DESCRIPTOR_V1.conversion_profile.is_none()
        {
            Ok(())
        } else {
            Err(InterchangeImportRefusalV1::for_reason(
                InterchangeImportRefusalReasonV1::InternalFailure,
            ))
        }
    }
}

/// Closed refusal categories for interchange import.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterchangeImportRefusalCategoryV1 {
    ConversionFailed,
    ConversionUnsupported,
    ResourceLimit,
    DocumentAdmissionFailed,
    StaleDocument,
    ChemistryUnavailable,
}

/// Closed recovery instruction paired exactly with one refusal category.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterchangeImportRecoveryV1 {
    ChooseSupportedFormat,
    RemoveUnsupportedFeatures,
    ReduceInput,
    RetryOrReportProblem,
    ReopenOrRetry,
    InstallChemistryRuntime,
}

/// Every interchange failure and resource reason, with no free-text variant.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[allow(clippy::enum_variant_names)]
#[serde(rename_all = "snake_case")]
pub enum InterchangeImportRefusalReasonV1 {
    InvalidUtf8,
    InvalidXml,
    InvalidXmlDeclaration,
    UnexpectedXmlText,
    UnexpectedXmlNode,
    InvalidScalar,
    InvalidCoordinate,
    CoordinateNotFinite,
    CoordinateOutOfRange,
    DuplicateSourceId,
    DuplicateAtomId,
    DanglingBond,
    SelfBond,
    DuplicateBond,
    InvalidGraph,
    EmptyDocument,
    NamespaceUnsupported,
    RootUnsupported,
    ProfileMismatch,
    AttributeUnsupported,
    ArrayAttributeUnsupported,
    UnrepresentedSemanticFact,
    DtdForbidden,
    EntityForbidden,
    ExternalResourceForbidden,
    XincludeForbidden,
    StylesheetForbidden,
    CompressionForbidden,
    FormatAliasUnsupported,
    DirectionUnsupported,
    InputBytesLimit,
    XmlTextBytesLimit,
    XmlDeclarationLimit,
    CommentBytesLimit,
    PiBytesLimit,
    XmlElementLimit,
    XmlDepthLimit,
    XmlAttributeLimit,
    AttributeValueLimit,
    RecordLimit,
    AtomsPerRecordLimit,
    AtomLimit,
    BondsPerRecordLimit,
    BondLimit,
    SourceIdMapLimit,
    IdentifierBytesLimit,
    CandidateBytesLimit,
    ResponseBytesLimit,
    CandidateValidationFailed,
    SerializationFailed,
    InternalFailure,
    RevisionMismatch,
    DigestMismatch,
    LiveReceiptStale,
    LiveReceiptUnavailable,
    ChemistryRuntimeUnavailable,
}

/// Redacted exact refusal triple. Fields are private so callers cannot forge invalid triples.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct InterchangeImportRefusalV1 {
    category: InterchangeImportRefusalCategoryV1,
    reason: InterchangeImportRefusalReasonV1,
    recovery: InterchangeImportRecoveryV1,
}

impl InterchangeImportRefusalV1 {
    #[must_use]
    pub const fn for_reason(reason: InterchangeImportRefusalReasonV1) -> Self {
        let (category, recovery) = match reason {
            InterchangeImportRefusalReasonV1::InvalidUtf8
            | InterchangeImportRefusalReasonV1::InvalidXml
            | InterchangeImportRefusalReasonV1::InvalidXmlDeclaration
            | InterchangeImportRefusalReasonV1::UnexpectedXmlText
            | InterchangeImportRefusalReasonV1::UnexpectedXmlNode
            | InterchangeImportRefusalReasonV1::InvalidScalar
            | InterchangeImportRefusalReasonV1::InvalidCoordinate
            | InterchangeImportRefusalReasonV1::CoordinateNotFinite
            | InterchangeImportRefusalReasonV1::CoordinateOutOfRange
            | InterchangeImportRefusalReasonV1::DuplicateSourceId
            | InterchangeImportRefusalReasonV1::DuplicateAtomId
            | InterchangeImportRefusalReasonV1::DanglingBond
            | InterchangeImportRefusalReasonV1::SelfBond
            | InterchangeImportRefusalReasonV1::DuplicateBond
            | InterchangeImportRefusalReasonV1::InvalidGraph
            | InterchangeImportRefusalReasonV1::EmptyDocument => (
                InterchangeImportRefusalCategoryV1::ConversionFailed,
                InterchangeImportRecoveryV1::ChooseSupportedFormat,
            ),
            InterchangeImportRefusalReasonV1::NamespaceUnsupported
            | InterchangeImportRefusalReasonV1::RootUnsupported
            | InterchangeImportRefusalReasonV1::ProfileMismatch
            | InterchangeImportRefusalReasonV1::AttributeUnsupported
            | InterchangeImportRefusalReasonV1::ArrayAttributeUnsupported
            | InterchangeImportRefusalReasonV1::UnrepresentedSemanticFact
            | InterchangeImportRefusalReasonV1::DtdForbidden
            | InterchangeImportRefusalReasonV1::EntityForbidden
            | InterchangeImportRefusalReasonV1::ExternalResourceForbidden
            | InterchangeImportRefusalReasonV1::XincludeForbidden
            | InterchangeImportRefusalReasonV1::StylesheetForbidden
            | InterchangeImportRefusalReasonV1::CompressionForbidden
            | InterchangeImportRefusalReasonV1::FormatAliasUnsupported
            | InterchangeImportRefusalReasonV1::DirectionUnsupported => (
                InterchangeImportRefusalCategoryV1::ConversionUnsupported,
                InterchangeImportRecoveryV1::RemoveUnsupportedFeatures,
            ),
            InterchangeImportRefusalReasonV1::InputBytesLimit
            | InterchangeImportRefusalReasonV1::XmlTextBytesLimit
            | InterchangeImportRefusalReasonV1::XmlDeclarationLimit
            | InterchangeImportRefusalReasonV1::CommentBytesLimit
            | InterchangeImportRefusalReasonV1::PiBytesLimit
            | InterchangeImportRefusalReasonV1::XmlElementLimit
            | InterchangeImportRefusalReasonV1::XmlDepthLimit
            | InterchangeImportRefusalReasonV1::XmlAttributeLimit
            | InterchangeImportRefusalReasonV1::AttributeValueLimit
            | InterchangeImportRefusalReasonV1::RecordLimit
            | InterchangeImportRefusalReasonV1::AtomsPerRecordLimit
            | InterchangeImportRefusalReasonV1::AtomLimit
            | InterchangeImportRefusalReasonV1::BondsPerRecordLimit
            | InterchangeImportRefusalReasonV1::BondLimit
            | InterchangeImportRefusalReasonV1::SourceIdMapLimit
            | InterchangeImportRefusalReasonV1::IdentifierBytesLimit
            | InterchangeImportRefusalReasonV1::CandidateBytesLimit
            | InterchangeImportRefusalReasonV1::ResponseBytesLimit => (
                InterchangeImportRefusalCategoryV1::ResourceLimit,
                InterchangeImportRecoveryV1::ReduceInput,
            ),
            InterchangeImportRefusalReasonV1::CandidateValidationFailed
            | InterchangeImportRefusalReasonV1::SerializationFailed
            | InterchangeImportRefusalReasonV1::InternalFailure => (
                InterchangeImportRefusalCategoryV1::DocumentAdmissionFailed,
                InterchangeImportRecoveryV1::RetryOrReportProblem,
            ),
            InterchangeImportRefusalReasonV1::RevisionMismatch
            | InterchangeImportRefusalReasonV1::DigestMismatch
            | InterchangeImportRefusalReasonV1::LiveReceiptStale
            | InterchangeImportRefusalReasonV1::LiveReceiptUnavailable => (
                InterchangeImportRefusalCategoryV1::StaleDocument,
                InterchangeImportRecoveryV1::ReopenOrRetry,
            ),
            InterchangeImportRefusalReasonV1::ChemistryRuntimeUnavailable => (
                InterchangeImportRefusalCategoryV1::ChemistryUnavailable,
                InterchangeImportRecoveryV1::InstallChemistryRuntime,
            ),
        };
        Self {
            category,
            reason,
            recovery,
        }
    }
    #[must_use]
    pub const fn category(self) -> InterchangeImportRefusalCategoryV1 {
        self.category
    }
    #[must_use]
    pub const fn reason(self) -> InterchangeImportRefusalReasonV1 {
        self.reason
    }
    #[must_use]
    pub const fn recovery(self) -> InterchangeImportRecoveryV1 {
        self.recovery
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_joins_the_owning_crates_and_refuses_unknown_aliases() {
        InterchangeFormatRegistryV1::validate_exact_join().expect("exact interchange join");
        let descriptor =
            InterchangeFormatRegistryV1::lookup_input_alias("cml2").expect("CML2 alias");
        assert_eq!(
            descriptor.profile_id(),
            CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1
        );
        let refusal =
            InterchangeFormatRegistryV1::lookup_input_alias("xml").expect_err("unknown alias");
        assert_eq!(
            refusal.reason(),
            InterchangeImportRefusalReasonV1::FormatAliasUnsupported
        );
        assert_eq!(
            refusal.category(),
            InterchangeImportRefusalCategoryV1::ConversionUnsupported
        );
        let suffix = InterchangeFormatRegistryV1::lookup_input_suffix(".sd").expect("SDF suffix");
        assert_eq!(suffix.decoder(), InterchangeDecoderKeyV1::Sdf);
    }

    #[test]
    fn cdxml_descriptor_uses_the_chemistry_source_byte_limit() {
        let descriptor =
            InterchangeFormatRegistryV1::lookup_input_alias("cdxml").expect("CDXML descriptor");
        assert_eq!(
            descriptor.limits().max_source_bytes(),
            CDXML_SIMPLE_MOLECULE_IMPORT_MAX_SOURCE_BYTES_V1
        );
    }

    #[test]
    fn every_enabled_descriptor_has_one_closed_decoder_and_complete_transport_policy() {
        for descriptor in InterchangeFormatRegistryV1::descriptors() {
            assert!(!descriptor.input_aliases().is_empty());
            assert!(!descriptor.input_suffixes().is_empty());
            assert!(!descriptor.profile_id().is_empty());
            assert!(descriptor.limits().max_source_bytes() > 0);
            assert!(descriptor.limits().max_response_bytes() > 0);
            assert!(matches!(
                descriptor.decoder(),
                InterchangeDecoderKeyV1::CmlSimpleMolecule
                    | InterchangeDecoderKeyV1::CdxmlSimpleMolecule
                    | InterchangeDecoderKeyV1::Sdf
            ));
        }
    }

    #[test]
    fn refusal_dto_serializes_only_the_typed_recovery_triple() {
        let refusal =
            InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::XmlDepthLimit);
        assert_eq!(
            serde_json::to_value(refusal).expect("refusal serializes"),
            serde_json::json!({
                "category": "resource_limit",
                "reason": "xml_depth_limit",
                "recovery": "reduce_input",
            })
        );
    }

    #[test]
    fn local_document_registry_keeps_routes_and_refusals_closed() {
        LocalDocumentIngressRegistryV1::validate_exact_join().expect("local registry joins CML");
        assert_eq!(
            LocalDocumentIngressRegistryV1::lookup_suffix(".cml")
                .expect("CML route")
                .direction(),
            LocalDocumentIngressDirectionV1::NewDocumentOnly
        );
        let cdxml = LocalDocumentIngressRegistryV1::lookup_suffix(".cdxml").expect("CDXML route");
        assert_eq!(
            cdxml.decoder(),
            LocalDocumentIngressDecoderV1::CdxmlSimpleMolecule
        );
        assert_eq!(cdxml.profile_id(), CDXML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1);
    }
}

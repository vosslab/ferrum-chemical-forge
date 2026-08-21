//! M2a's closed CML/CML2 interchange registry and transport foundations.
//!
//! This module intentionally has no XML parser, document transaction, CLI
//! command, PyO3 receipt, or Qt dependency.  It freezes the values those later
//! layers must join rather than allowing each adapter to maintain its own CML
//! table.

use ferrum_chemistry::{CML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1, SDF_MAX_INPUT_BYTES};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Exact CML profile selected by the static format descriptor.
pub const CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1: &str = CML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1;
/// Exact API format identifier selected by the static format descriptor.
pub const CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1: &str = "cml_simple_molecule_import_v1";
/// Exact API format identifier selected by the SDF V1 descriptor.
pub const SDF_IMPORT_FORMAT_V1: &str = "sdf_v1";
/// Exact SDF profile selected by the static format descriptor.
pub const SDF_IMPORT_PROFILE_V1: &str = "sdf_v1";
const CML_IMPORT_MAX_SOURCE_BYTES_V1: usize = 1_048_576;
const INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1: usize = 1_048_576;

/// Closed import direction advertised by a Ferrum interchange descriptor.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterchangeDirectionV1 {
    DocumentImportNew,
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
    Sdf,
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

/// One static, import-only descriptor consumed by future CLI and Qt surfaces.
#[derive(Debug, JsonSchema, Serialize)]
pub struct InterchangeFormatDescriptorV1 {
    display_name: &'static str,
    format_id: &'static str,
    profile_id: &'static str,
    input_aliases: &'static [&'static str],
    input_suffixes: &'static [&'static str],
    directions: &'static [InterchangeDirectionV1],
    output_suffixes: &'static [&'static str],
    compression: InterchangeCompressionPolicyV1,
    semantic_loss_policy: InterchangeSemanticLossPolicyV1,
    decoder: InterchangeDecoderKeyV1,
    limits: InterchangeImportLimitsV1,
}

impl InterchangeFormatDescriptorV1 {
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
    pub const fn directions(&self) -> &'static [InterchangeDirectionV1] {
        self.directions
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
    #[must_use]
    pub const fn limits(&self) -> InterchangeImportLimitsV1 {
        self.limits
    }
}

const CML_EXPECTED_INPUT_ALIASES_V1: [&str; 3] = ["cml", "cml1", "cml2"];
const CML_EXPECTED_INPUT_SUFFIXES_V1: [&str; 1] = [".cml"];
const CML_EXPECTED_OUTPUT_SUFFIXES_V1: [&str; 0] = [];
const CML_DIRECTIONS_V1: [InterchangeDirectionV1; 1] = [InterchangeDirectionV1::DocumentImportNew];
const CML_DESCRIPTOR_V1: InterchangeFormatDescriptorV1 = InterchangeFormatDescriptorV1 {
    display_name: "Chemical Markup Language (CML/CML2)",
    format_id: CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1,
    profile_id: CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1,
    input_aliases: &CML_EXPECTED_INPUT_ALIASES_V1,
    input_suffixes: &CML_EXPECTED_INPUT_SUFFIXES_V1,
    directions: &CML_DIRECTIONS_V1,
    output_suffixes: &CML_EXPECTED_OUTPUT_SUFFIXES_V1,
    compression: InterchangeCompressionPolicyV1::Forbidden,
    semantic_loss_policy: InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics,
    decoder: InterchangeDecoderKeyV1::CmlSimpleMolecule,
    limits: InterchangeImportLimitsV1::new(
        CML_IMPORT_MAX_SOURCE_BYTES_V1,
        INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1,
    ),
};
const SDF_EXPECTED_INPUT_ALIASES_V1: [&str; 2] = ["sdf", "sd"];
const SDF_EXPECTED_INPUT_SUFFIXES_V1: [&str; 2] = [".sdf", ".sd"];
const SDF_DESCRIPTOR_V1: InterchangeFormatDescriptorV1 = InterchangeFormatDescriptorV1 {
    display_name: "Structure Data File (SDF)",
    format_id: SDF_IMPORT_FORMAT_V1,
    profile_id: SDF_IMPORT_PROFILE_V1,
    input_aliases: &SDF_EXPECTED_INPUT_ALIASES_V1,
    input_suffixes: &SDF_EXPECTED_INPUT_SUFFIXES_V1,
    directions: &CML_DIRECTIONS_V1,
    output_suffixes: &CML_EXPECTED_OUTPUT_SUFFIXES_V1,
    compression: InterchangeCompressionPolicyV1::Forbidden,
    semantic_loss_policy: InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics,
    decoder: InterchangeDecoderKeyV1::Sdf,
    limits: InterchangeImportLimitsV1::new(
        SDF_MAX_INPUT_BYTES,
        INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1,
    ),
};

/// The sole static API-owned interchange registry for M2a.
pub struct InterchangeFormatRegistryV1;

impl InterchangeFormatRegistryV1 {
    /// Return every enabled descriptor in deterministic API order.
    #[must_use]
    pub const fn descriptors() -> &'static [InterchangeFormatDescriptorV1] {
        &[CML_DESCRIPTOR_V1, SDF_DESCRIPTOR_V1]
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
        let directions = [InterchangeDirectionV1::DocumentImportNew];
        if CML_DESCRIPTOR_V1.display_name == "Chemical Markup Language (CML/CML2)"
            && CML_DESCRIPTOR_V1.format_id == CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1
            && CML_DESCRIPTOR_V1.profile_id == CML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1
            && CML_DESCRIPTOR_V1.input_aliases == CML_EXPECTED_INPUT_ALIASES_V1
            && CML_DESCRIPTOR_V1.input_suffixes == CML_EXPECTED_INPUT_SUFFIXES_V1
            && CML_DESCRIPTOR_V1.directions == directions
            && CML_DESCRIPTOR_V1.output_suffixes == CML_EXPECTED_OUTPUT_SUFFIXES_V1
            && SDF_DESCRIPTOR_V1.format_id == SDF_IMPORT_FORMAT_V1
            && SDF_DESCRIPTOR_V1.profile_id == SDF_IMPORT_PROFILE_V1
            && SDF_DESCRIPTOR_V1.input_aliases == SDF_EXPECTED_INPUT_ALIASES_V1
            && SDF_DESCRIPTOR_V1.input_suffixes == SDF_EXPECTED_INPUT_SUFFIXES_V1
            && SDF_DESCRIPTOR_V1.directions == directions
            && SDF_DESCRIPTOR_V1.output_suffixes == CML_EXPECTED_OUTPUT_SUFFIXES_V1
            && CML_DESCRIPTOR_V1.compression == InterchangeCompressionPolicyV1::Forbidden
            && SDF_DESCRIPTOR_V1.compression == InterchangeCompressionPolicyV1::Forbidden
            && CML_DESCRIPTOR_V1.semantic_loss_policy
                == InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics
            && SDF_DESCRIPTOR_V1.semantic_loss_policy
                == InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics
            && CML_DESCRIPTOR_V1.decoder == InterchangeDecoderKeyV1::CmlSimpleMolecule
            && SDF_DESCRIPTOR_V1.decoder == InterchangeDecoderKeyV1::Sdf
            && CML_DESCRIPTOR_V1.limits.max_source_bytes == CML_IMPORT_MAX_SOURCE_BYTES_V1
            && SDF_DESCRIPTOR_V1.limits.max_source_bytes == SDF_MAX_INPUT_BYTES
            && CML_DESCRIPTOR_V1.limits.max_response_bytes
                == INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1
            && SDF_DESCRIPTOR_V1.limits.max_response_bytes
                == INTERCHANGE_IMPORT_MAX_RESPONSE_BYTES_V1
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

/// Every M2a failure and resource reason, with no free-text variant.
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
        InterchangeFormatRegistryV1::validate_exact_join().expect("exact M2a join");
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
    fn every_enabled_descriptor_has_one_closed_decoder_and_complete_transport_policy() {
        for descriptor in InterchangeFormatRegistryV1::descriptors() {
            assert!(!descriptor.input_aliases().is_empty());
            assert!(!descriptor.input_suffixes().is_empty());
            assert!(!descriptor.profile_id().is_empty());
            assert!(descriptor.limits().max_source_bytes() > 0);
            assert!(descriptor.limits().max_response_bytes() > 0);
            assert!(matches!(
                descriptor.decoder(),
                InterchangeDecoderKeyV1::CmlSimpleMolecule | InterchangeDecoderKeyV1::Sdf
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
}

//! M2a's closed CML/CML2 interchange registry and transport foundations.
//!
//! This module intentionally has no XML parser, document transaction, CLI
//! command, PyO3 receipt, or Qt dependency.  It freezes the values those later
//! layers must join rather than allowing each adapter to maintain its own CML
//! table.

use ferrum_chemistry::CML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Exact CML profile selected by the static format descriptor.
pub const CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1: &str = CML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1;
/// Exact API format identifier selected by the static format descriptor.
pub const CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1: &str = "cml_simple_molecule_import_v1";
/// Frozen maximum for M2a.3's one canonical CML import response envelope.
///
/// M2a.0 retains this value for the later exact-envelope admission boundary. It
/// does not serialize or publish a response envelope itself.
pub const CML_IMPORT_RESPONSE_BUDGET_BYTES_V1: usize = 1_048_576;

/// Closed import direction advertised by a Ferrum interchange descriptor.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InterchangeDirectionV1 {
    DocumentImportNew,
}

/// Closed compression policy.  CML import never implicitly decompresses input.
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

/// One static, import-only descriptor consumed by future CLI and Qt surfaces.
#[derive(Debug, JsonSchema, Serialize)]
pub struct InterchangeFormatDescriptorV1 {
    format_id: &'static str,
    profile_id: &'static str,
    input_aliases: &'static [&'static str],
    input_suffixes: &'static [&'static str],
    directions: &'static [InterchangeDirectionV1],
    output_suffixes: &'static [&'static str],
    compression: InterchangeCompressionPolicyV1,
    semantic_loss_policy: InterchangeSemanticLossPolicyV1,
}

impl InterchangeFormatDescriptorV1 {
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
}

const CML_EXPECTED_INPUT_ALIASES_V1: [&str; 3] = ["cml", "cml1", "cml2"];
const CML_EXPECTED_INPUT_SUFFIXES_V1: [&str; 1] = [".cml"];
const CML_EXPECTED_OUTPUT_SUFFIXES_V1: [&str; 0] = [];
const CML_DIRECTIONS_V1: [InterchangeDirectionV1; 1] = [InterchangeDirectionV1::DocumentImportNew];
const CML_DESCRIPTOR_V1: InterchangeFormatDescriptorV1 = InterchangeFormatDescriptorV1 {
    format_id: CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1,
    profile_id: CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1,
    input_aliases: &CML_EXPECTED_INPUT_ALIASES_V1,
    input_suffixes: &CML_EXPECTED_INPUT_SUFFIXES_V1,
    directions: &CML_DIRECTIONS_V1,
    output_suffixes: &CML_EXPECTED_OUTPUT_SUFFIXES_V1,
    compression: InterchangeCompressionPolicyV1::Forbidden,
    semantic_loss_policy: InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics,
};

/// The sole static API-owned interchange registry for M2a.
pub struct InterchangeFormatRegistryV1;

impl InterchangeFormatRegistryV1 {
    /// Return every enabled descriptor in deterministic API order.
    #[must_use]
    pub const fn descriptors() -> &'static [InterchangeFormatDescriptorV1] {
        &[CML_DESCRIPTOR_V1]
    }

    /// Resolve one exact lower-case input alias without guessing or suffix fallback.
    pub fn lookup_input_alias(
        alias: &str,
    ) -> Result<&'static InterchangeFormatDescriptorV1, CmlImportRefusalV1> {
        if CML_DESCRIPTOR_V1.input_aliases.contains(&alias) {
            Ok(&CML_DESCRIPTOR_V1)
        } else {
            Err(CmlImportRefusalV1::for_reason(
                CmlImportRefusalReasonV1::FormatAliasUnsupported,
            ))
        }
    }

    /// Prove the API descriptor exactly joins the chemistry profile and document targets.
    pub fn validate_exact_join() -> Result<(), CmlImportRefusalV1> {
        let descriptor = &CML_DESCRIPTOR_V1;
        let directions = [InterchangeDirectionV1::DocumentImportNew];
        if descriptor.format_id == CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1
            && descriptor.profile_id == CML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1
            && descriptor.input_aliases == CML_EXPECTED_INPUT_ALIASES_V1
            && descriptor.input_suffixes == CML_EXPECTED_INPUT_SUFFIXES_V1
            && descriptor.directions == directions
            && descriptor.output_suffixes == CML_EXPECTED_OUTPUT_SUFFIXES_V1
            && descriptor.compression == InterchangeCompressionPolicyV1::Forbidden
            && descriptor.semantic_loss_policy
                == InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics
        {
            Ok(())
        } else {
            Err(CmlImportRefusalV1::for_reason(
                CmlImportRefusalReasonV1::InternalFailure,
            ))
        }
    }
}

/// Frozen ingress and response bounds for M2a.  It has no caller-settable fields.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
pub struct CmlIngressBudgetV1 {
    raw_utf8_input_bytes: usize,
    decoded_xml_text_bytes: usize,
    xml_declaration_bytes: usize,
    comment_bytes: usize,
    processing_instruction_bytes: usize,
    xml_elements: usize,
    xml_depth: usize,
    attributes_per_element: usize,
    attribute_value_bytes: usize,
    source_records: usize,
    atoms_per_record: usize,
    atoms_total: usize,
    bonds_per_record: usize,
    bonds_total: usize,
    source_id_map_entries: usize,
    scalar_identifier_bytes: usize,
    candidate_cdml_bytes: usize,
    response_bytes: usize,
}

impl CmlIngressBudgetV1 {
    #[must_use]
    pub const fn frozen() -> Self {
        Self {
            raw_utf8_input_bytes: 1_048_576,
            decoded_xml_text_bytes: 1_048_576,
            xml_declaration_bytes: 256,
            comment_bytes: 65_536,
            processing_instruction_bytes: 8_192,
            xml_elements: 50_000,
            xml_depth: 8,
            attributes_per_element: 8,
            attribute_value_bytes: 256,
            source_records: 1_024,
            atoms_per_record: 10_000,
            atoms_total: 100_000,
            bonds_per_record: 20_000,
            bonds_total: 200_000,
            source_id_map_entries: 101_024,
            scalar_identifier_bytes: 128,
            candidate_cdml_bytes: 8_388_608,
            response_bytes: CML_IMPORT_RESPONSE_BUDGET_BYTES_V1,
        }
    }
    #[must_use]
    pub const fn raw_utf8_input_bytes(self) -> usize {
        self.raw_utf8_input_bytes
    }
    #[must_use]
    pub const fn decoded_xml_text_bytes(self) -> usize {
        self.decoded_xml_text_bytes
    }
    #[must_use]
    pub const fn xml_declaration_bytes(self) -> usize {
        self.xml_declaration_bytes
    }
    #[must_use]
    pub const fn comment_bytes(self) -> usize {
        self.comment_bytes
    }
    #[must_use]
    pub const fn processing_instruction_bytes(self) -> usize {
        self.processing_instruction_bytes
    }
    #[must_use]
    pub const fn xml_elements(self) -> usize {
        self.xml_elements
    }
    #[must_use]
    pub const fn xml_depth(self) -> usize {
        self.xml_depth
    }
    #[must_use]
    pub const fn attributes_per_element(self) -> usize {
        self.attributes_per_element
    }
    #[must_use]
    pub const fn attribute_value_bytes(self) -> usize {
        self.attribute_value_bytes
    }
    #[must_use]
    pub const fn source_records(self) -> usize {
        self.source_records
    }
    #[must_use]
    pub const fn atoms_per_record(self) -> usize {
        self.atoms_per_record
    }
    #[must_use]
    pub const fn atoms_total(self) -> usize {
        self.atoms_total
    }
    #[must_use]
    pub const fn bonds_per_record(self) -> usize {
        self.bonds_per_record
    }
    #[must_use]
    pub const fn bonds_total(self) -> usize {
        self.bonds_total
    }
    #[must_use]
    pub const fn source_id_map_entries(self) -> usize {
        self.source_id_map_entries
    }
    #[must_use]
    pub const fn scalar_identifier_bytes(self) -> usize {
        self.scalar_identifier_bytes
    }
    #[must_use]
    pub const fn candidate_cdml_bytes(self) -> usize {
        self.candidate_cdml_bytes
    }
    #[must_use]
    pub const fn response_bytes(self) -> usize {
        self.response_bytes
    }
}

/// Closed refusal categories for CML import.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CmlImportRefusalCategoryV1 {
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
pub enum CmlImportRecoveryV1 {
    ChooseSupportedCml,
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
pub enum CmlImportRefusalReasonV1 {
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
pub struct CmlImportRefusalV1 {
    category: CmlImportRefusalCategoryV1,
    reason: CmlImportRefusalReasonV1,
    recovery: CmlImportRecoveryV1,
}

impl CmlImportRefusalV1 {
    #[must_use]
    pub const fn for_reason(reason: CmlImportRefusalReasonV1) -> Self {
        let (category, recovery) = match reason {
            CmlImportRefusalReasonV1::InvalidUtf8
            | CmlImportRefusalReasonV1::InvalidXml
            | CmlImportRefusalReasonV1::InvalidXmlDeclaration
            | CmlImportRefusalReasonV1::UnexpectedXmlText
            | CmlImportRefusalReasonV1::UnexpectedXmlNode
            | CmlImportRefusalReasonV1::InvalidScalar
            | CmlImportRefusalReasonV1::InvalidCoordinate
            | CmlImportRefusalReasonV1::CoordinateNotFinite
            | CmlImportRefusalReasonV1::CoordinateOutOfRange
            | CmlImportRefusalReasonV1::DuplicateSourceId
            | CmlImportRefusalReasonV1::DuplicateAtomId
            | CmlImportRefusalReasonV1::DanglingBond
            | CmlImportRefusalReasonV1::SelfBond
            | CmlImportRefusalReasonV1::DuplicateBond
            | CmlImportRefusalReasonV1::InvalidGraph
            | CmlImportRefusalReasonV1::EmptyDocument => (
                CmlImportRefusalCategoryV1::ConversionFailed,
                CmlImportRecoveryV1::ChooseSupportedCml,
            ),
            CmlImportRefusalReasonV1::NamespaceUnsupported
            | CmlImportRefusalReasonV1::RootUnsupported
            | CmlImportRefusalReasonV1::ProfileMismatch
            | CmlImportRefusalReasonV1::AttributeUnsupported
            | CmlImportRefusalReasonV1::ArrayAttributeUnsupported
            | CmlImportRefusalReasonV1::UnrepresentedSemanticFact
            | CmlImportRefusalReasonV1::DtdForbidden
            | CmlImportRefusalReasonV1::EntityForbidden
            | CmlImportRefusalReasonV1::ExternalResourceForbidden
            | CmlImportRefusalReasonV1::XincludeForbidden
            | CmlImportRefusalReasonV1::StylesheetForbidden
            | CmlImportRefusalReasonV1::CompressionForbidden
            | CmlImportRefusalReasonV1::FormatAliasUnsupported
            | CmlImportRefusalReasonV1::DirectionUnsupported => (
                CmlImportRefusalCategoryV1::ConversionUnsupported,
                CmlImportRecoveryV1::RemoveUnsupportedFeatures,
            ),
            CmlImportRefusalReasonV1::InputBytesLimit
            | CmlImportRefusalReasonV1::XmlTextBytesLimit
            | CmlImportRefusalReasonV1::XmlDeclarationLimit
            | CmlImportRefusalReasonV1::CommentBytesLimit
            | CmlImportRefusalReasonV1::PiBytesLimit
            | CmlImportRefusalReasonV1::XmlElementLimit
            | CmlImportRefusalReasonV1::XmlDepthLimit
            | CmlImportRefusalReasonV1::XmlAttributeLimit
            | CmlImportRefusalReasonV1::AttributeValueLimit
            | CmlImportRefusalReasonV1::RecordLimit
            | CmlImportRefusalReasonV1::AtomsPerRecordLimit
            | CmlImportRefusalReasonV1::AtomLimit
            | CmlImportRefusalReasonV1::BondsPerRecordLimit
            | CmlImportRefusalReasonV1::BondLimit
            | CmlImportRefusalReasonV1::SourceIdMapLimit
            | CmlImportRefusalReasonV1::IdentifierBytesLimit
            | CmlImportRefusalReasonV1::CandidateBytesLimit
            | CmlImportRefusalReasonV1::ResponseBytesLimit => (
                CmlImportRefusalCategoryV1::ResourceLimit,
                CmlImportRecoveryV1::ReduceInput,
            ),
            CmlImportRefusalReasonV1::CandidateValidationFailed
            | CmlImportRefusalReasonV1::SerializationFailed
            | CmlImportRefusalReasonV1::InternalFailure => (
                CmlImportRefusalCategoryV1::DocumentAdmissionFailed,
                CmlImportRecoveryV1::RetryOrReportProblem,
            ),
            CmlImportRefusalReasonV1::RevisionMismatch
            | CmlImportRefusalReasonV1::DigestMismatch
            | CmlImportRefusalReasonV1::LiveReceiptStale
            | CmlImportRefusalReasonV1::LiveReceiptUnavailable => (
                CmlImportRefusalCategoryV1::StaleDocument,
                CmlImportRecoveryV1::ReopenOrRetry,
            ),
            CmlImportRefusalReasonV1::ChemistryRuntimeUnavailable => (
                CmlImportRefusalCategoryV1::ChemistryUnavailable,
                CmlImportRecoveryV1::InstallChemistryRuntime,
            ),
        };
        Self {
            category,
            reason,
            recovery,
        }
    }
    #[must_use]
    pub const fn category(self) -> CmlImportRefusalCategoryV1 {
        self.category
    }
    #[must_use]
    pub const fn reason(self) -> CmlImportRefusalReasonV1 {
        self.reason
    }
    #[must_use]
    pub const fn recovery(self) -> CmlImportRecoveryV1 {
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
        assert_eq!(descriptor.profile_id(), CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1);
        let refusal =
            InterchangeFormatRegistryV1::lookup_input_alias("xml").expect_err("unknown alias");
        assert_eq!(
            refusal.reason(),
            CmlImportRefusalReasonV1::FormatAliasUnsupported
        );
        assert_eq!(
            refusal.category(),
            CmlImportRefusalCategoryV1::ConversionUnsupported
        );
    }

    #[test]
    fn frozen_budget_preserves_cross_limit_ordering() {
        let budget = CmlIngressBudgetV1::frozen();
        assert!(budget.raw_utf8_input_bytes() <= budget.decoded_xml_text_bytes());
        assert!(budget.response_bytes() <= budget.candidate_cdml_bytes());
        assert!(budget.source_records() <= budget.source_id_map_entries());
    }

    #[test]
    fn refusal_dto_serializes_only_the_typed_recovery_triple() {
        let refusal = CmlImportRefusalV1::for_reason(CmlImportRefusalReasonV1::XmlDepthLimit);
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

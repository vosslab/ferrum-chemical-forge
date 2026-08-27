//! Closed conversion-capability resolver shared by every Ferrum adapter.
//!
//! The input and output registries retain their domain-specific descriptor
//! facts.  This facade is the only join point for conversion aliases, suffixes,
//! codec selection, and execution requirements, so CLI and future discovery
//! routes cannot accumulate private format tables.

use ferrum_chemistry::{InterchangeFormatV1, SDF_MAX_INPUT_BYTES};

use crate::protocol::InspectGraphNormalizationV1;
use crate::protocol::{InspectGraphFactCoverageStatusV1, InspectGraphFactCoverageV1};
use crate::{
    ConversionExecutionProfileV1, ConversionInputProfileV1, ConversionOutputDescriptorV1,
    ConversionOutputRegistryV1, InterchangeCompressionPolicyV1, InterchangeFormatDescriptorV1,
    InterchangeFormatRegistryV1, InterchangeImportRefusalReasonV1, InterchangeImportRefusalV1,
    InterchangeOperationRefusalV1, InterchangeOperationV1, InterchangeRuntimeRequirementV1,
    InterchangeSemanticLossPolicyV1,
};

const NATIVE_RECORD_CONVERSION_PROFILE_V1: ConversionInputProfileV1 = ConversionInputProfileV1::new(
    SDF_MAX_INPUT_BYTES,
    InterchangeRuntimeRequirementV1::RuntimeRequired,
);
const NATIVE_RECORD_CONVERSION_PROFILE_ID_V1: &str = "native_record_conversion_v1";
const CHEMISTRY_CONVERT_OPERATIONS_V1: [InterchangeOperationV1; 1] =
    [InterchangeOperationV1::ChemistryConvert];
const NATIVE_RECORD_COMPRESSION_POLICY_V1: InterchangeCompressionPolicyV1 =
    InterchangeCompressionPolicyV1::Forbidden;
const NATIVE_RECORD_SEMANTIC_LOSS_POLICY_V1: InterchangeSemanticLossPolicyV1 =
    InterchangeSemanticLossPolicyV1::RejectUnrepresentedSemantics;

/// Static conversion-input facts that were historically owned only by the CLI.
#[derive(Clone, Copy, Debug)]
pub struct NativeConversionInputDescriptorV1 {
    canonical_name: &'static str,
    display_name: &'static str,
    format_id: &'static str,
    profile_id: &'static str,
    aliases: &'static [&'static str],
    suffixes: &'static [&'static str],
    protocol_format: InterchangeFormatV1,
    conversion_profile: ConversionInputProfileV1,
    operations: &'static [InterchangeOperationV1],
    compression_policy: InterchangeCompressionPolicyV1,
    semantic_loss_policy: InterchangeSemanticLossPolicyV1,
}

impl NativeConversionInputDescriptorV1 {
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        self.canonical_name
    }

    #[must_use]
    pub const fn display_name(self) -> &'static str {
        self.display_name
    }

    #[must_use]
    pub const fn format_id(self) -> &'static str {
        self.format_id
    }

    #[must_use]
    pub const fn profile_id(self) -> &'static str {
        self.profile_id
    }

    #[must_use]
    pub const fn aliases(self) -> &'static [&'static str] {
        self.aliases
    }

    #[must_use]
    pub const fn suffixes(self) -> &'static [&'static str] {
        self.suffixes
    }

    #[must_use]
    pub const fn protocol_format(self) -> InterchangeFormatV1 {
        self.protocol_format
    }

    #[must_use]
    pub const fn conversion_profile(self) -> ConversionInputProfileV1 {
        self.conversion_profile
    }

    #[must_use]
    pub const fn operations(self) -> &'static [InterchangeOperationV1] {
        self.operations
    }

    #[must_use]
    pub const fn compression_policy(self) -> InterchangeCompressionPolicyV1 {
        self.compression_policy
    }

    #[must_use]
    pub const fn semantic_loss_policy(self) -> InterchangeSemanticLossPolicyV1 {
        self.semantic_loss_policy
    }
}

const SMILES_ALIASES_V1: [&str; 1] = ["smiles"];
const SMILES_SUFFIXES_V1: [&str; 2] = [".smi", ".smiles"];
const INCHI_STANDARD_ALIASES_V1: [&str; 1] = ["inchi_standard"];
const INCHI_STANDARD_SUFFIXES_V1: [&str; 1] = [".inchi"];
const INCHI_FIXED_HYDROGEN_ALIASES_V1: [&str; 1] = ["inchi_fixed_h"];
const EMPTY_SUFFIXES_V1: [&str; 0] = [];
const MOLBLOCK_V2000_ALIASES_V1: [&str; 1] = ["molblock_v2000"];
const MOLBLOCK_V2000_SUFFIXES_V1: [&str; 2] = [".mol", ".molblock"];
const MOLBLOCK_V3000_ALIASES_V1: [&str; 1] = ["molblock_v3000"];
const SDF_V3000_ALIASES_V1: [&str; 1] = ["sdf_v3000"];
const CDML_ALIASES_V1: [&str; 1] = ["cdml"];
const CDML_SUFFIXES_V1: [&str; 1] = [".cdml"];
/// Conversion accepts this explicit spelling in addition to the local-ingress
/// SDF aliases. It belongs to the one resolver capability, so discovery and
/// parsing share the same facts.
const SDF_V2000_CAPABILITY_ALIASES_V1: [&str; 3] = ["sdf", "sd", "sdf_v2000"];

const NATIVE_INPUT_DESCRIPTORS_V1: [NativeConversionInputDescriptorV1; 7] = [
    NativeConversionInputDescriptorV1 {
        canonical_name: "smiles",
        display_name: "smiles",
        format_id: "smiles_v1",
        profile_id: NATIVE_RECORD_CONVERSION_PROFILE_ID_V1,
        aliases: &SMILES_ALIASES_V1,
        suffixes: &SMILES_SUFFIXES_V1,
        protocol_format: InterchangeFormatV1::Smiles,
        conversion_profile: NATIVE_RECORD_CONVERSION_PROFILE_V1,
        operations: &CHEMISTRY_CONVERT_OPERATIONS_V1,
        compression_policy: NATIVE_RECORD_COMPRESSION_POLICY_V1,
        semantic_loss_policy: NATIVE_RECORD_SEMANTIC_LOSS_POLICY_V1,
    },
    NativeConversionInputDescriptorV1 {
        canonical_name: "inchi_standard",
        display_name: "inchi_standard",
        format_id: "inchi_standard_v1",
        profile_id: NATIVE_RECORD_CONVERSION_PROFILE_ID_V1,
        aliases: &INCHI_STANDARD_ALIASES_V1,
        suffixes: &INCHI_STANDARD_SUFFIXES_V1,
        protocol_format: InterchangeFormatV1::InchiStandard,
        conversion_profile: NATIVE_RECORD_CONVERSION_PROFILE_V1,
        operations: &CHEMISTRY_CONVERT_OPERATIONS_V1,
        compression_policy: NATIVE_RECORD_COMPRESSION_POLICY_V1,
        semantic_loss_policy: NATIVE_RECORD_SEMANTIC_LOSS_POLICY_V1,
    },
    NativeConversionInputDescriptorV1 {
        canonical_name: "inchi_fixed_h",
        display_name: "inchi_fixed_h",
        format_id: "inchi_fixed_h_v1",
        profile_id: NATIVE_RECORD_CONVERSION_PROFILE_ID_V1,
        aliases: &INCHI_FIXED_HYDROGEN_ALIASES_V1,
        suffixes: &EMPTY_SUFFIXES_V1,
        protocol_format: InterchangeFormatV1::InchiFixedHydrogen,
        conversion_profile: NATIVE_RECORD_CONVERSION_PROFILE_V1,
        operations: &CHEMISTRY_CONVERT_OPERATIONS_V1,
        compression_policy: NATIVE_RECORD_COMPRESSION_POLICY_V1,
        semantic_loss_policy: NATIVE_RECORD_SEMANTIC_LOSS_POLICY_V1,
    },
    NativeConversionInputDescriptorV1 {
        canonical_name: "molblock_v2000",
        display_name: "molblock_v2000",
        format_id: "molblock_v2000_v1",
        profile_id: NATIVE_RECORD_CONVERSION_PROFILE_ID_V1,
        aliases: &MOLBLOCK_V2000_ALIASES_V1,
        suffixes: &MOLBLOCK_V2000_SUFFIXES_V1,
        protocol_format: InterchangeFormatV1::MolblockV2000,
        conversion_profile: NATIVE_RECORD_CONVERSION_PROFILE_V1,
        operations: &CHEMISTRY_CONVERT_OPERATIONS_V1,
        compression_policy: NATIVE_RECORD_COMPRESSION_POLICY_V1,
        semantic_loss_policy: NATIVE_RECORD_SEMANTIC_LOSS_POLICY_V1,
    },
    NativeConversionInputDescriptorV1 {
        canonical_name: "molblock_v3000",
        display_name: "molblock_v3000",
        format_id: "molblock_v3000_v1",
        profile_id: NATIVE_RECORD_CONVERSION_PROFILE_ID_V1,
        aliases: &MOLBLOCK_V3000_ALIASES_V1,
        suffixes: &EMPTY_SUFFIXES_V1,
        protocol_format: InterchangeFormatV1::MolblockV3000,
        conversion_profile: NATIVE_RECORD_CONVERSION_PROFILE_V1,
        operations: &CHEMISTRY_CONVERT_OPERATIONS_V1,
        compression_policy: NATIVE_RECORD_COMPRESSION_POLICY_V1,
        semantic_loss_policy: NATIVE_RECORD_SEMANTIC_LOSS_POLICY_V1,
    },
    NativeConversionInputDescriptorV1 {
        canonical_name: "sdf_v3000",
        display_name: "sdf_v3000",
        format_id: "sdf_v3000_v1",
        profile_id: NATIVE_RECORD_CONVERSION_PROFILE_ID_V1,
        aliases: &SDF_V3000_ALIASES_V1,
        suffixes: &EMPTY_SUFFIXES_V1,
        protocol_format: InterchangeFormatV1::SdfV3000,
        conversion_profile: NATIVE_RECORD_CONVERSION_PROFILE_V1,
        operations: &CHEMISTRY_CONVERT_OPERATIONS_V1,
        compression_policy: NATIVE_RECORD_COMPRESSION_POLICY_V1,
        semantic_loss_policy: NATIVE_RECORD_SEMANTIC_LOSS_POLICY_V1,
    },
    NativeConversionInputDescriptorV1 {
        canonical_name: "cdml",
        display_name: "cdml",
        format_id: "cdml_v1",
        profile_id: NATIVE_RECORD_CONVERSION_PROFILE_ID_V1,
        aliases: &CDML_ALIASES_V1,
        suffixes: &CDML_SUFFIXES_V1,
        protocol_format: InterchangeFormatV1::Cdml,
        conversion_profile: NATIVE_RECORD_CONVERSION_PROFILE_V1,
        operations: &CHEMISTRY_CONVERT_OPERATIONS_V1,
        compression_policy: NATIVE_RECORD_COMPRESSION_POLICY_V1,
        semantic_loss_policy: NATIVE_RECORD_SEMANTIC_LOSS_POLICY_V1,
    },
];

/// Borrowed input capability resolved by the API-owned conversion facade.
#[derive(Clone, Copy, Debug)]
pub enum ConversionInputCapabilityV1 {
    Interchange(&'static InterchangeFormatDescriptorV1),
    Native(&'static NativeConversionInputDescriptorV1),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterchangeGraphInspectionProfileV1 {
    CmlSimpleMolecule,
    SdfNativeSemantic,
}

/// One immutable decoder route selected by an inspection descriptor.
///
/// Adding a profile requires an exhaustive routing decision here and in the
/// protocol executor; coverage and normalization remain descriptor facts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InterchangeGraphInspectionRouteV1 {
    CmlSimpleMolecule,
    SdfNativeSemantic,
}

impl InterchangeGraphInspectionProfileV1 {
    /// The bounded complete CLI response limit for this profile.
    ///
    /// This safely covers the admitted count-only summary and bounded record
    /// metadata projection without exposing a partial response.
    pub const MAX_RESPONSE_BYTES: usize = 1_048_576;

    #[must_use]
    pub const fn max_response_bytes(self) -> usize {
        Self::MAX_RESPONSE_BYTES
    }

    #[must_use]
    pub const fn max_source_bytes(self) -> usize {
        SDF_MAX_INPUT_BYTES
    }

    #[must_use]
    pub const fn format_id(self) -> &'static str {
        match self {
            Self::CmlSimpleMolecule => "cml_simple_molecule_import_v1",
            Self::SdfNativeSemantic => "sdf_v2000",
        }
    }

    #[must_use]
    pub const fn profile_id(self) -> &'static str {
        match self {
            Self::CmlSimpleMolecule => "cml_simple_molecule_inspection_v1",
            Self::SdfNativeSemantic => "sdf_native_semantic_inspection_v1",
        }
    }

    #[must_use]
    pub const fn runtime_requirement(self) -> InterchangeRuntimeRequirementV1 {
        match self {
            Self::CmlSimpleMolecule => InterchangeRuntimeRequirementV1::RuntimeFree,
            Self::SdfNativeSemantic => InterchangeRuntimeRequirementV1::RuntimeRequired,
        }
    }

    #[must_use]
    pub const fn route(self) -> InterchangeGraphInspectionRouteV1 {
        match self {
            Self::CmlSimpleMolecule => InterchangeGraphInspectionRouteV1::CmlSimpleMolecule,
            Self::SdfNativeSemantic => InterchangeGraphInspectionRouteV1::SdfNativeSemantic,
        }
    }

    /// Return the complete, profile-owned source-fact disclosure.
    #[must_use]
    pub const fn fact_coverage(self) -> InspectGraphFactCoverageV1 {
        use InspectGraphFactCoverageStatusV1::{Known, UnknownWhenOmitted, Unsupported};
        match self {
            Self::CmlSimpleMolecule => InspectGraphFactCoverageV1 {
                source_record_ordering: Known,
                atom_count: Known,
                bond_count: Known,
                atom_source_id: Known,
                element: Known,
                coordinates: Known,
                bond_endpoints: Known,
                bond_order: Known,
                source_molecule_id: UnknownWhenOmitted,
                formal_charge: UnknownWhenOmitted,
                isotope: UnknownWhenOmitted,
                bond_source_id: Unsupported,
                bond_stereo_direction: Unsupported,
                radicals: Unsupported,
                atom_labels_properties: Unsupported,
                reaction_atom_maps: Unsupported,
                record_source_id: UnknownWhenOmitted,
                record_title: Unsupported,
                property_count: Unsupported,
                aromaticity: Unsupported,
                stereo: Unsupported,
            },
            Self::SdfNativeSemantic => InspectGraphFactCoverageV1 {
                source_record_ordering: Known,
                atom_count: Known,
                bond_count: Known,
                atom_source_id: Unsupported,
                element: Known,
                coordinates: Known,
                bond_endpoints: Known,
                bond_order: Known,
                source_molecule_id: Unsupported,
                formal_charge: Known,
                isotope: UnknownWhenOmitted,
                bond_source_id: Unsupported,
                bond_stereo_direction: Known,
                radicals: Unsupported,
                atom_labels_properties: Unsupported,
                reaction_atom_maps: Unsupported,
                record_source_id: Unsupported,
                record_title: Known,
                property_count: Known,
                aromaticity: Known,
                stereo: Known,
            },
        }
    }

    /// Return the complete, profile-owned decoded-semantic normalization disclosure.
    #[must_use]
    pub fn normalization(self) -> InspectGraphNormalizationV1 {
        match self {
            Self::CmlSimpleMolecule => InspectGraphNormalizationV1 {
                source_coordinate_space: "cml_y_down".to_owned(),
                graph_normalization: "closed_cml_profile".to_owned(),
                aromaticity: "unsupported".to_owned(),
                stereo: "unsupported".to_owned(),
                raw_source_fidelity: "not_claimed".to_owned(),
            },
            Self::SdfNativeSemantic => InspectGraphNormalizationV1 {
                source_coordinate_space: "native_decoded_2d".to_owned(),
                graph_normalization: "native_normalized".to_owned(),
                aromaticity: "native_normalized".to_owned(),
                stereo: "native_normalized".to_owned(),
                raw_source_fidelity: "not_claimed".to_owned(),
            },
        }
    }
}

impl ConversionInputCapabilityV1 {
    #[must_use]
    pub const fn graph_inspection_profile(self) -> Option<InterchangeGraphInspectionProfileV1> {
        match self {
            Self::Interchange(descriptor) => descriptor.graph_inspection_profile(),
            Self::Native(_) => None,
        }
    }
    #[must_use]
    pub const fn protocol_format(self) -> InterchangeFormatV1 {
        match self {
            Self::Interchange(descriptor) => match descriptor.decoder() {
                crate::InterchangeDecoderKeyV1::CmlSimpleMolecule => {
                    InterchangeFormatV1::CmlSimpleMolecule
                }
                crate::InterchangeDecoderKeyV1::CdxmlSimpleMolecule => {
                    InterchangeFormatV1::CdxmlSimpleMolecule
                }
                crate::InterchangeDecoderKeyV1::Sdf => InterchangeFormatV1::SdfV2000,
            },
            Self::Native(descriptor) => descriptor.protocol_format(),
        }
    }

    #[must_use]
    pub const fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Interchange(descriptor) => interchange_capability_aliases(descriptor),
            Self::Native(descriptor) => descriptor.aliases(),
        }
    }

    #[must_use]
    pub const fn suffixes(self) -> &'static [&'static str] {
        match self {
            Self::Interchange(descriptor) => descriptor.input_suffixes(),
            Self::Native(descriptor) => descriptor.suffixes(),
        }
    }

    /// Return the stable discovery identity for this admitted input format.
    #[must_use]
    pub const fn format_id(self) -> &'static str {
        match self {
            Self::Interchange(descriptor) => descriptor.format_id(),
            Self::Native(descriptor) => descriptor.format_id(),
        }
    }

    /// Return the versioned execution-profile identity for this input format.
    #[must_use]
    pub const fn profile_id(self) -> &'static str {
        match self {
            Self::Interchange(descriptor) => descriptor.profile_id(),
            Self::Native(descriptor) => descriptor.profile_id(),
        }
    }

    #[must_use]
    pub const fn conversion_profile(self) -> Option<ConversionInputProfileV1> {
        match self {
            Self::Interchange(descriptor) => descriptor.conversion_profile(),
            Self::Native(descriptor) => Some(descriptor.conversion_profile()),
        }
    }

    /// Return every operation admitted for this input.
    #[must_use]
    pub const fn operations(self) -> &'static [InterchangeOperationV1] {
        match self {
            Self::Interchange(descriptor) => descriptor.operations(),
            Self::Native(descriptor) => descriptor.operations(),
        }
    }

    /// Return the input compression policy without adapter-local defaults.
    #[must_use]
    pub const fn compression_policy(self) -> InterchangeCompressionPolicyV1 {
        match self {
            Self::Interchange(descriptor) => descriptor.compression(),
            Self::Native(descriptor) => descriptor.compression_policy(),
        }
    }

    /// Return the input semantic-loss policy without adapter-local defaults.
    #[must_use]
    pub const fn semantic_loss_policy(self) -> InterchangeSemanticLossPolicyV1 {
        match self {
            Self::Interchange(descriptor) => descriptor.semantic_loss_policy(),
            Self::Native(descriptor) => descriptor.semantic_loss_policy(),
        }
    }

    /// Return the actual source bound used by conversion transport.
    #[must_use]
    pub const fn max_source_bytes(self) -> usize {
        match self {
            Self::Interchange(descriptor) => descriptor.limits().max_source_bytes(),
            Self::Native(descriptor) => descriptor.conversion_profile().max_source_bytes(),
        }
    }

    /// Return the input-owned response-envelope bound when one exists.
    ///
    /// Interchange import has an admitted JSON response envelope. Native record
    /// conversion writes its selected output directly, so a response limit is
    /// not an input-owned policy fact and is represented explicitly as `None`.
    #[must_use]
    pub const fn max_response_bytes(self) -> Option<usize> {
        match self {
            Self::Interchange(descriptor) => Some(descriptor.limits().max_response_bytes()),
            Self::Native(_) => None,
        }
    }

    /// Return the runtime requirement selected by the input profile.
    #[must_use]
    pub const fn runtime_requirement(self) -> Option<InterchangeRuntimeRequirementV1> {
        match self.conversion_profile() {
            Some(profile) => Some(profile.runtime_requirement()),
            None => None,
        }
    }

    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Interchange(descriptor) => descriptor.canonical_name(),
            Self::Native(descriptor) => descriptor.canonical_name(),
        }
    }

    /// Return the descriptor-owned human display identity for this input.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Interchange(descriptor) => descriptor.display_name(),
            Self::Native(descriptor) => descriptor.display_name(),
        }
    }
}

/// Return the conversion-facing aliases for one interchange descriptor.
///
/// Local document ingress keeps its narrower SDF vocabulary in its own data
/// owner. This resolver extends only the SDF conversion capability with the
/// existing CLI spelling, making it enumerable without changing ingress.
const fn interchange_capability_aliases(
    descriptor: &'static InterchangeFormatDescriptorV1,
) -> &'static [&'static str] {
    match descriptor.decoder() {
        crate::InterchangeDecoderKeyV1::CmlSimpleMolecule
        | crate::InterchangeDecoderKeyV1::CdxmlSimpleMolecule => descriptor.input_aliases(),
        crate::InterchangeDecoderKeyV1::Sdf => &SDF_V2000_CAPABILITY_ALIASES_V1,
    }
}

/// Sole API authority for conversion input/output resolution and execution policy.
pub struct InterchangeCapabilityResolverV1;

impl InterchangeCapabilityResolverV1 {
    #[must_use]
    pub const fn native_input_descriptors() -> &'static [NativeConversionInputDescriptorV1] {
        &NATIVE_INPUT_DESCRIPTORS_V1
    }

    /// Borrow every admitted conversion input in deterministic API order.
    pub fn input_capabilities() -> impl Iterator<Item = ConversionInputCapabilityV1> {
        InterchangeFormatRegistryV1::descriptors()
            .iter()
            .map(ConversionInputCapabilityV1::Interchange)
            .chain(
                NATIVE_INPUT_DESCRIPTORS_V1
                    .iter()
                    .map(ConversionInputCapabilityV1::Native),
            )
    }

    /// Borrow every admitted conversion output in deterministic API order.
    #[must_use]
    pub const fn output_descriptors() -> &'static [ConversionOutputDescriptorV1] {
        ConversionOutputRegistryV1::descriptors()
    }

    /// Resolve one exact lower-case public conversion input alias.
    pub fn lookup_input_alias(
        alias: &str,
    ) -> Result<ConversionInputCapabilityV1, InterchangeImportRefusalV1> {
        Self::input_capabilities()
            .find(|descriptor| descriptor.aliases().contains(&alias))
            .ok_or_else(|| {
                InterchangeImportRefusalV1::for_reason(
                    InterchangeImportRefusalReasonV1::FormatAliasUnsupported,
                )
            })
    }

    /// Resolve one exact lower-case filename suffix.
    pub fn lookup_input_suffix(
        suffix: &str,
    ) -> Result<ConversionInputCapabilityV1, InterchangeImportRefusalV1> {
        if let Ok(descriptor) = InterchangeFormatRegistryV1::lookup_input_suffix(suffix) {
            return Ok(ConversionInputCapabilityV1::Interchange(descriptor));
        }
        NATIVE_INPUT_DESCRIPTORS_V1
            .iter()
            .find(|descriptor| descriptor.suffixes().contains(&suffix))
            .map(ConversionInputCapabilityV1::Native)
            .ok_or_else(|| {
                InterchangeImportRefusalV1::for_reason(
                    InterchangeImportRefusalReasonV1::FormatAliasUnsupported,
                )
            })
    }

    /// Resolve the unique admitted input descriptor for a protocol format.
    #[must_use]
    pub fn lookup_input_format(format: InterchangeFormatV1) -> Option<ConversionInputCapabilityV1> {
        Self::input_capabilities().find(|descriptor| descriptor.protocol_format() == format)
    }

    /// Resolve a known input only when it explicitly admits the requested operation.
    pub fn lookup_input_for_operation(
        format: InterchangeFormatV1,
        operation: InterchangeOperationV1,
    ) -> Result<ConversionInputCapabilityV1, InterchangeOperationRefusalV1> {
        let input =
            Self::lookup_input_format(format).expect("closed CLI input format is registered");
        if input.operations().contains(&operation) {
            Ok(input)
        } else {
            Err(InterchangeOperationRefusalV1::new(
                operation,
                input.operations(),
            ))
        }
    }

    /// Resolve one exact lower-case public conversion output alias.
    #[must_use]
    pub fn lookup_output_alias(alias: &str) -> Option<&'static ConversionOutputDescriptorV1> {
        ConversionOutputRegistryV1::lookup_alias(alias)
    }

    /// Resolve the conversion output descriptor for one protocol format.
    ///
    /// The output registry remains the descriptor-data owner; CLI transport
    /// reaches it only through this resolver-owned conversion join.
    #[must_use]
    pub const fn lookup_output_format(
        format: InterchangeFormatV1,
    ) -> Option<&'static ConversionOutputDescriptorV1> {
        ConversionOutputRegistryV1::lookup_protocol_format(format)
    }

    /// Join an admitted input and output into the execution policy used by CLI transport.
    #[must_use]
    pub const fn resolve_execution_profile(
        input: ConversionInputCapabilityV1,
        output: &'static ConversionOutputDescriptorV1,
    ) -> ConversionExecutionProfileV1 {
        ConversionExecutionProfileV1::join(
            input
                .conversion_profile()
                .expect("conversion eligibility selects a conversion profile"),
            output.runtime_requirement(),
        )
    }

    /// Validate that every public conversion spelling reaches one descriptor and policy.
    pub fn validate_exact_join() -> Result<(), InterchangeImportRefusalV1> {
        InterchangeFormatRegistryV1::validate_exact_join()?;
        ConversionOutputRegistryV1::validate_exact_join().map_err(|_| {
            InterchangeImportRefusalV1::for_reason(
                InterchangeImportRefusalReasonV1::InternalFailure,
            )
        })?;
        for input in Self::input_capabilities() {
            let format = input.protocol_format();
            let has_exact_input = Self::input_capabilities()
                .filter(|candidate| candidate.protocol_format() == format)
                .count()
                == 1;
            let output_count = Self::output_descriptors()
                .iter()
                .filter(|candidate| candidate.target().protocol_format() == format)
                .count();
            let output_contract_is_valid = output_count <= 1;
            let aliases_are_unique = !input.aliases().is_empty()
                && input.aliases().iter().all(|alias| {
                    Self::input_capabilities()
                        .filter(|candidate| candidate.aliases().contains(alias))
                        .count()
                        == 1
                });
            let suffixes_are_unique = input.suffixes().iter().all(|suffix| {
                Self::input_capabilities()
                    .filter(|candidate| candidate.suffixes().contains(suffix))
                    .count()
                    == 1
            });
            let has_unique_identity_profile = !input.format_id().is_empty()
                && !input.profile_id().is_empty()
                && !input.canonical_name().is_empty()
                && !input.display_name().is_empty()
                && Self::input_capabilities()
                    .filter(|candidate| {
                        candidate.format_id() == input.format_id()
                            && candidate.profile_id() == input.profile_id()
                    })
                    .count()
                    == 1;
            if !(has_exact_input
                && output_contract_is_valid
                && aliases_are_unique
                && suffixes_are_unique
                && has_unique_identity_profile)
            {
                return Err(InterchangeImportRefusalV1::for_reason(
                    InterchangeImportRefusalReasonV1::InternalFailure,
                ));
            }
        }
        for output in Self::output_descriptors() {
            let format = output.target().protocol_format();
            let has_exact_output = Self::output_descriptors()
                .iter()
                .filter(|candidate| candidate.target().protocol_format() == format)
                .count()
                == 1;
            let input_count = Self::input_capabilities()
                .filter(|candidate| candidate.protocol_format() == format)
                .count();
            if !(has_exact_output && input_count == 1) {
                return Err(InterchangeImportRefusalV1::for_reason(
                    InterchangeImportRefusalReasonV1::InternalFailure,
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ferrum_chemistry::InterchangeFormatV1;

    use super::{ConversionInputCapabilityV1, InterchangeCapabilityResolverV1};
    use crate::InterchangeOperationV1;

    #[test]
    fn every_current_public_input_and_output_route_has_one_descriptor_and_policy() {
        assert_eq!(
            InterchangeCapabilityResolverV1::validate_exact_join(),
            Ok(())
        );
        for input in InterchangeCapabilityResolverV1::input_capabilities() {
            for alias in input.aliases() {
                assert_eq!(
                    InterchangeCapabilityResolverV1::input_capabilities()
                        .filter(|candidate| candidate.aliases().contains(alias))
                        .count(),
                    1,
                    "{alias} is advertised by one input capability"
                );
                assert_eq!(
                    InterchangeCapabilityResolverV1::lookup_input_alias(alias)
                        .expect("current public input alias")
                        .protocol_format(),
                    input.protocol_format()
                );
            }
            for suffix in input.suffixes() {
                assert_eq!(
                    InterchangeCapabilityResolverV1::input_capabilities()
                        .filter(|candidate| candidate.suffixes().contains(suffix))
                        .count(),
                    1,
                    "{suffix} is advertised by one input capability"
                );
                assert_eq!(
                    InterchangeCapabilityResolverV1::lookup_input_suffix(suffix)
                        .expect("current public input suffix")
                        .protocol_format(),
                    input.protocol_format()
                );
            }
        }
        for output in InterchangeCapabilityResolverV1::output_descriptors() {
            for alias in output.aliases() {
                assert_eq!(
                    InterchangeCapabilityResolverV1::lookup_output_alias(alias)
                        .expect("current public output alias")
                        .format_id(),
                    output.format_id()
                );
            }
        }
        let sdf = InterchangeCapabilityResolverV1::lookup_input_alias("sdf_v2000")
            .expect("SDF V2000 capability");
        assert_eq!(sdf.protocol_format(), InterchangeFormatV1::SdfV2000);
        assert!(sdf.aliases().contains(&"sdf_v2000"));
    }

    #[test]
    fn operation_eligibility_keeps_cdxml_out_of_conversion_without_narrowing_cml_or_sdf() {
        for format in [
            InterchangeFormatV1::CmlSimpleMolecule,
            InterchangeFormatV1::SdfV2000,
        ] {
            assert!(
                InterchangeCapabilityResolverV1::lookup_input_for_operation(
                    format,
                    InterchangeOperationV1::ChemistryConvert,
                )
                .is_ok()
            );
        }
        let refusal = InterchangeCapabilityResolverV1::lookup_input_for_operation(
            InterchangeFormatV1::CdxmlSimpleMolecule,
            InterchangeOperationV1::ChemistryConvert,
        )
        .expect_err("CDXML is document-import-only");
        assert_eq!(
            refusal.requested_operation(),
            InterchangeOperationV1::ChemistryConvert
        );
        assert_eq!(
            refusal.supported_operations(),
            &[InterchangeOperationV1::DocumentImportNew]
        );
    }

    #[test]
    fn native_and_cml_suffixes_share_the_resolver() {
        assert_eq!(
            InterchangeCapabilityResolverV1::lookup_input_suffix(".smi")
                .expect("SMILES suffix")
                .protocol_format(),
            InterchangeFormatV1::Smiles
        );
        assert_eq!(
            InterchangeCapabilityResolverV1::lookup_input_suffix(".cml")
                .expect("CML suffix")
                .protocol_format(),
            InterchangeFormatV1::CmlSimpleMolecule
        );
    }

    #[test]
    fn cdxml_is_a_document_import_only_capability() {
        let from_alias = InterchangeCapabilityResolverV1::lookup_input_alias("cdxml")
            .expect("CDXML alias resolves through the shared resolver");
        let cdxml = InterchangeCapabilityResolverV1::lookup_input_suffix(".cdxml")
            .expect("CDXML suffix resolves through the shared resolver");
        assert_eq!(
            cdxml.protocol_format(),
            InterchangeFormatV1::CdxmlSimpleMolecule
        );
        assert_eq!(from_alias.protocol_format(), cdxml.protocol_format());
        assert_eq!(cdxml.conversion_profile(), None);
        assert!(
            InterchangeCapabilityResolverV1::lookup_output_format(cdxml.protocol_format())
                .is_none()
        );
    }

    #[test]
    fn every_input_capability_exposes_a_complete_and_valid_discovery_contract() {
        for capability in InterchangeCapabilityResolverV1::input_capabilities() {
            assert!(
                !capability.format_id().is_empty(),
                "every capability has a stable format identity"
            );
            assert!(
                !capability.profile_id().is_empty(),
                "every capability has a versioned profile identity"
            );
            assert_eq!(
                InterchangeCapabilityResolverV1::input_capabilities()
                    .filter(|candidate| {
                        candidate.format_id() == capability.format_id()
                            && candidate.profile_id() == capability.profile_id()
                    })
                    .count(),
                1,
                "{} has a unique identity/profile pair",
                capability.canonical_name()
            );
            assert!(
                !capability.canonical_name().is_empty(),
                "every capability has a canonical name"
            );
            assert!(
                !capability.aliases().is_empty(),
                "{} has at least one public alias",
                capability.canonical_name()
            );
            assert!(
                !capability.operations().is_empty(),
                "{} declares an admitted operation",
                capability.canonical_name()
            );
            assert!(
                capability.max_source_bytes() > 0,
                "{} has a usable source limit",
                capability.canonical_name()
            );
            assert!(
                capability.aliases().iter().all(|alias| !alias.is_empty()),
                "{} has valid aliases",
                capability.canonical_name()
            );
            assert!(
                capability
                    .suffixes()
                    .iter()
                    .all(|suffix| suffix.starts_with('.')),
                "{} has valid suffixes",
                capability.canonical_name()
            );
            match capability {
                ConversionInputCapabilityV1::Interchange(descriptor) => {
                    assert_eq!(
                        capability.max_response_bytes(),
                        Some(descriptor.limits().max_response_bytes()),
                        "{} preserves its input response policy",
                        capability.canonical_name()
                    );
                }
                ConversionInputCapabilityV1::Native(_) => {
                    assert_eq!(
                        capability.max_response_bytes(),
                        None,
                        "{} explicitly has no input-owned response envelope",
                        capability.canonical_name()
                    );
                }
            }
        }
    }

    #[test]
    fn resolver_declares_runtime_free_cml_and_runtime_backed_sdf_inspection_profiles() {
        let cml = InterchangeCapabilityResolverV1::lookup_input_alias("cml2")
            .expect("CML2 resolves through the shared resolver");
        let sdf = InterchangeCapabilityResolverV1::lookup_input_alias("sdf")
            .expect("SDF resolves through the shared resolver");
        assert_eq!(
            cml.graph_inspection_profile(),
            Some(super::InterchangeGraphInspectionProfileV1::CmlSimpleMolecule)
        );
        assert_eq!(
            sdf.graph_inspection_profile(),
            Some(super::InterchangeGraphInspectionProfileV1::SdfNativeSemantic)
        );
        let cml_profile = cml.graph_inspection_profile().expect("CML profile");
        let sdf_profile = sdf.graph_inspection_profile().expect("SDF profile");
        assert_eq!(
            cml_profile.runtime_requirement(),
            super::InterchangeRuntimeRequirementV1::RuntimeFree
        );
        assert_eq!(
            sdf_profile.runtime_requirement(),
            super::InterchangeRuntimeRequirementV1::RuntimeRequired
        );
        assert_eq!(
            sdf_profile.fact_coverage().record_source_id,
            crate::protocol::InspectGraphFactCoverageStatusV1::Unsupported
        );
        assert_eq!(
            sdf_profile.fact_coverage().formal_charge,
            crate::protocol::InspectGraphFactCoverageStatusV1::Known
        );
        assert_eq!(
            sdf_profile.fact_coverage().isotope,
            crate::protocol::InspectGraphFactCoverageStatusV1::UnknownWhenOmitted
        );
    }
}

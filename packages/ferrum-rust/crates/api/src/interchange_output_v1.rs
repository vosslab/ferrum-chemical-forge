//! Closed conversion-output capability registry.
//!
//! Import and local-document ingress remain owned by
//! `interchange_import_v1`. This module describes only molecular conversion
//! targets, so CLI adapters can resolve public names without deriving output
//! behavior from an ingress descriptor.

use ferrum_chemistry::InterchangeFormatV1;

use crate::InterchangeRuntimeRequirementV1;

/// Stable API key for the canonical CML2 conversion output capability.
pub const CML_SIMPLE_MOLECULE_OUTPUT_FORMAT_V1: &str = "cml_simple_molecule_output_v1";
/// Stable API profile for canonical CML2 conversion output.
pub const CML_SIMPLE_MOLECULE_OUTPUT_PROFILE_V1: &str =
    "ferrum-cml-simple-molecule-output-profile-v1";

/// The execution domain for one public conversion target.
///
/// CDML is Ferrum's canonical document syntax. Other targets are molecular
/// record codecs owned by `ferrum-chemistry`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConversionOutputTargetV1 {
    /// Emit Ferrum's canonical local document syntax.
    CanonicalCdml,
    /// Emit one molecular record codec through the chemistry runtime.
    ChemistryRecordCodec(InterchangeFormatV1),
}

impl ConversionOutputTargetV1 {
    #[must_use]
    pub const fn protocol_format(self) -> InterchangeFormatV1 {
        match self {
            Self::CanonicalCdml => InterchangeFormatV1::Cdml,
            Self::ChemistryRecordCodec(format) => format,
        }
    }
}

/// Public metadata for one conversion target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConversionOutputDescriptorV1 {
    canonical_name: &'static str,
    display_name: &'static str,
    format_id: &'static str,
    profile_id: &'static str,
    aliases: &'static [&'static str],
    output_suffix: &'static str,
    target: ConversionOutputTargetV1,
    runtime_requirement: InterchangeRuntimeRequirementV1,
}

impl ConversionOutputDescriptorV1 {
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
    pub const fn output_suffix(self) -> &'static str {
        self.output_suffix
    }

    #[must_use]
    pub const fn target(self) -> ConversionOutputTargetV1 {
        self.target
    }

    #[must_use]
    pub const fn runtime_requirement(self) -> InterchangeRuntimeRequirementV1 {
        self.runtime_requirement
    }
}

const SMILES_ALIASES_V1: [&str; 1] = ["smiles"];
const INCHI_STANDARD_ALIASES_V1: [&str; 1] = ["inchi_standard"];
const INCHI_FIXED_HYDROGEN_ALIASES_V1: [&str; 1] = ["inchi_fixed_h"];
const MOLBLOCK_V2000_ALIASES_V1: [&str; 1] = ["molblock_v2000"];
const MOLBLOCK_V3000_ALIASES_V1: [&str; 1] = ["molblock_v3000"];
const SDF_V2000_ALIASES_V1: [&str; 1] = ["sdf_v2000"];
const SDF_V3000_ALIASES_V1: [&str; 1] = ["sdf_v3000"];
const CDML_ALIASES_V1: [&str; 1] = ["cdml"];
const CML_SIMPLE_MOLECULE_ALIASES_V1: [&str; 2] = ["cml", "cml2"];

const SMILES_DESCRIPTOR_V1: ConversionOutputDescriptorV1 = ConversionOutputDescriptorV1 {
    canonical_name: "smiles",
    display_name: "SMILES",
    format_id: "smiles_v1",
    profile_id: "smiles_v1",
    aliases: &SMILES_ALIASES_V1,
    output_suffix: ".smi",
    target: ConversionOutputTargetV1::ChemistryRecordCodec(InterchangeFormatV1::Smiles),
    runtime_requirement: InterchangeRuntimeRequirementV1::RuntimeRequired,
};
const INCHI_STANDARD_DESCRIPTOR_V1: ConversionOutputDescriptorV1 = ConversionOutputDescriptorV1 {
    canonical_name: "inchi_standard",
    display_name: "Standard InChI",
    format_id: "inchi_standard_v1",
    profile_id: "inchi_standard_v1",
    aliases: &INCHI_STANDARD_ALIASES_V1,
    output_suffix: ".inchi",
    target: ConversionOutputTargetV1::ChemistryRecordCodec(InterchangeFormatV1::InchiStandard),
    runtime_requirement: InterchangeRuntimeRequirementV1::RuntimeRequired,
};
const INCHI_FIXED_HYDROGEN_DESCRIPTOR_V1: ConversionOutputDescriptorV1 =
    ConversionOutputDescriptorV1 {
        canonical_name: "inchi_fixed_h",
        display_name: "Fixed-Hydrogen InChI",
        format_id: "inchi_fixed_h_v1",
        profile_id: "inchi_fixed_h_v1",
        aliases: &INCHI_FIXED_HYDROGEN_ALIASES_V1,
        output_suffix: ".inchi",
        target: ConversionOutputTargetV1::ChemistryRecordCodec(
            InterchangeFormatV1::InchiFixedHydrogen,
        ),
        runtime_requirement: InterchangeRuntimeRequirementV1::RuntimeRequired,
    };
const MOLBLOCK_V2000_DESCRIPTOR_V1: ConversionOutputDescriptorV1 = ConversionOutputDescriptorV1 {
    canonical_name: "molblock_v2000",
    display_name: "MDL Molfile V2000",
    format_id: "molblock_v2000_v1",
    profile_id: "molblock_v2000_v1",
    aliases: &MOLBLOCK_V2000_ALIASES_V1,
    output_suffix: ".mol",
    target: ConversionOutputTargetV1::ChemistryRecordCodec(InterchangeFormatV1::MolblockV2000),
    runtime_requirement: InterchangeRuntimeRequirementV1::RuntimeRequired,
};
const MOLBLOCK_V3000_DESCRIPTOR_V1: ConversionOutputDescriptorV1 = ConversionOutputDescriptorV1 {
    canonical_name: "molblock_v3000",
    display_name: "MDL Molfile V3000",
    format_id: "molblock_v3000_v1",
    profile_id: "molblock_v3000_v1",
    aliases: &MOLBLOCK_V3000_ALIASES_V1,
    output_suffix: ".mol",
    target: ConversionOutputTargetV1::ChemistryRecordCodec(InterchangeFormatV1::MolblockV3000),
    runtime_requirement: InterchangeRuntimeRequirementV1::RuntimeRequired,
};
const SDF_V2000_DESCRIPTOR_V1: ConversionOutputDescriptorV1 = ConversionOutputDescriptorV1 {
    canonical_name: "sdf_v2000",
    display_name: "Structure Data File V2000",
    format_id: "sdf_v2000_v1",
    profile_id: "sdf_v2000_v1",
    aliases: &SDF_V2000_ALIASES_V1,
    output_suffix: ".sdf",
    target: ConversionOutputTargetV1::ChemistryRecordCodec(InterchangeFormatV1::SdfV2000),
    runtime_requirement: InterchangeRuntimeRequirementV1::RuntimeRequired,
};
const SDF_V3000_DESCRIPTOR_V1: ConversionOutputDescriptorV1 = ConversionOutputDescriptorV1 {
    canonical_name: "sdf_v3000",
    display_name: "Structure Data File V3000",
    format_id: "sdf_v3000_v1",
    profile_id: "sdf_v3000_v1",
    aliases: &SDF_V3000_ALIASES_V1,
    output_suffix: ".sdf",
    target: ConversionOutputTargetV1::ChemistryRecordCodec(InterchangeFormatV1::SdfV3000),
    runtime_requirement: InterchangeRuntimeRequirementV1::RuntimeRequired,
};
const CDML_DESCRIPTOR_V1: ConversionOutputDescriptorV1 = ConversionOutputDescriptorV1 {
    canonical_name: "cdml",
    display_name: "Ferrum Chemical Document Markup Language (CDML)",
    format_id: "cdml_document_v1",
    profile_id: "ferrum-cdml-document-output-profile-v1",
    aliases: &CDML_ALIASES_V1,
    output_suffix: ".cdml",
    target: ConversionOutputTargetV1::CanonicalCdml,
    runtime_requirement: InterchangeRuntimeRequirementV1::RuntimeFree,
};
const CML_SIMPLE_MOLECULE_DESCRIPTOR_V1: ConversionOutputDescriptorV1 =
    ConversionOutputDescriptorV1 {
        canonical_name: "cml",
        display_name: "Chemical Markup Language (CML2)",
        format_id: CML_SIMPLE_MOLECULE_OUTPUT_FORMAT_V1,
        profile_id: CML_SIMPLE_MOLECULE_OUTPUT_PROFILE_V1,
        aliases: &CML_SIMPLE_MOLECULE_ALIASES_V1,
        output_suffix: ".cml",
        target: ConversionOutputTargetV1::ChemistryRecordCodec(
            InterchangeFormatV1::CmlSimpleMolecule,
        ),
        runtime_requirement: InterchangeRuntimeRequirementV1::RuntimeFree,
    };

const CONVERSION_OUTPUT_DESCRIPTORS_V1: [ConversionOutputDescriptorV1; 9] = [
    SMILES_DESCRIPTOR_V1,
    INCHI_STANDARD_DESCRIPTOR_V1,
    INCHI_FIXED_HYDROGEN_DESCRIPTOR_V1,
    MOLBLOCK_V2000_DESCRIPTOR_V1,
    MOLBLOCK_V3000_DESCRIPTOR_V1,
    SDF_V2000_DESCRIPTOR_V1,
    SDF_V3000_DESCRIPTOR_V1,
    CDML_DESCRIPTOR_V1,
    CML_SIMPLE_MOLECULE_DESCRIPTOR_V1,
];

/// API-owned lookup surface for every supported molecular conversion output.
pub struct ConversionOutputRegistryV1;

impl ConversionOutputRegistryV1 {
    #[must_use]
    pub const fn descriptors() -> &'static [ConversionOutputDescriptorV1] {
        &CONVERSION_OUTPUT_DESCRIPTORS_V1
    }

    /// Resolve one exact lower-case CLI output alias.
    #[must_use]
    pub fn lookup_alias(alias: &str) -> Option<&'static ConversionOutputDescriptorV1> {
        Self::descriptors()
            .iter()
            .find(|descriptor| descriptor.aliases().contains(&alias))
    }

    /// Return the output capability for one closed chemistry format.
    ///
    /// This exhaustive match is the exact API-to-chemistry join. Adding a
    /// chemistry variant requires selecting it here, so it cannot drift into
    /// an unregistered conversion target.
    #[must_use]
    pub const fn lookup_chemistry_format(
        format: InterchangeFormatV1,
    ) -> Option<&'static ConversionOutputDescriptorV1> {
        match format {
            InterchangeFormatV1::Smiles => Some(&SMILES_DESCRIPTOR_V1),
            InterchangeFormatV1::InchiStandard => Some(&INCHI_STANDARD_DESCRIPTOR_V1),
            InterchangeFormatV1::InchiFixedHydrogen => Some(&INCHI_FIXED_HYDROGEN_DESCRIPTOR_V1),
            InterchangeFormatV1::MolblockV2000 => Some(&MOLBLOCK_V2000_DESCRIPTOR_V1),
            InterchangeFormatV1::MolblockV3000 => Some(&MOLBLOCK_V3000_DESCRIPTOR_V1),
            InterchangeFormatV1::SdfV2000 => Some(&SDF_V2000_DESCRIPTOR_V1),
            InterchangeFormatV1::SdfV3000 => Some(&SDF_V3000_DESCRIPTOR_V1),
            InterchangeFormatV1::Cdml => None,
            InterchangeFormatV1::CmlSimpleMolecule => Some(&CML_SIMPLE_MOLECULE_DESCRIPTOR_V1),
        }
    }

    /// Return the conversion target for one protocol interchange format.
    #[must_use]
    pub const fn lookup_protocol_format(
        format: InterchangeFormatV1,
    ) -> Option<&'static ConversionOutputDescriptorV1> {
        match format {
            InterchangeFormatV1::Cdml => Some(&CDML_DESCRIPTOR_V1),
            format => Self::lookup_chemistry_format(format),
        }
    }

    /// Verify that public names and codec keys form one collision-free join.
    pub fn validate_exact_join() -> Result<(), ConversionOutputRegistryRefusalV1> {
        for descriptor in Self::descriptors() {
            if descriptor.format_id().is_empty()
                || descriptor.profile_id().is_empty()
                || descriptor.canonical_name().is_empty()
                || descriptor.display_name().is_empty()
                || descriptor.output_suffix().is_empty()
                || descriptor.aliases().is_empty()
                || Self::lookup_protocol_format(descriptor.target().protocol_format())
                    != Some(descriptor)
                || matches!(
                    descriptor.target(),
                    ConversionOutputTargetV1::CanonicalCdml
                        | ConversionOutputTargetV1::ChemistryRecordCodec(
                            InterchangeFormatV1::CmlSimpleMolecule
                        )
                ) != (descriptor.runtime_requirement()
                    == InterchangeRuntimeRequirementV1::RuntimeFree)
            {
                return Err(ConversionOutputRegistryRefusalV1);
            }
            for alias in descriptor.aliases() {
                if Self::lookup_alias(alias) != Some(descriptor) {
                    return Err(ConversionOutputRegistryRefusalV1);
                }
            }
        }
        for (index, descriptor) in Self::descriptors().iter().enumerate() {
            for other in &Self::descriptors()[index + 1..] {
                if descriptor.format_id() == other.format_id()
                    || descriptor.profile_id() == other.profile_id()
                    || descriptor.target() == other.target()
                    || descriptor
                        .aliases()
                        .iter()
                        .any(|alias| other.aliases().contains(alias))
                {
                    return Err(ConversionOutputRegistryRefusalV1);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) const NON_CANONICAL_FIRST_ALIAS_OUTPUT_DESCRIPTOR_V1: ConversionOutputDescriptorV1 =
    ConversionOutputDescriptorV1 {
        canonical_name: "canonical",
        display_name: "Canonical descriptor",
        format_id: "canonical_test_v1",
        profile_id: "canonical_test_v1",
        aliases: &["compat", "canonical"],
        output_suffix: ".canonical",
        target: ConversionOutputTargetV1::CanonicalCdml,
        runtime_requirement: InterchangeRuntimeRequirementV1::RuntimeFree,
    };

/// Stable refusal returned only when the static registry violates its own join.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConversionOutputRegistryRefusalV1;

#[cfg(test)]
mod tests {
    use ferrum_chemistry::InterchangeFormatV1;

    use super::{
        CML_SIMPLE_MOLECULE_OUTPUT_FORMAT_V1, CML_SIMPLE_MOLECULE_OUTPUT_PROFILE_V1,
        ConversionOutputRegistryV1, ConversionOutputTargetV1,
    };
    use crate::InterchangeRuntimeRequirementV1;

    #[test]
    fn cml_aliases_resolve_to_the_canonical_cml2_output_capability() {
        let cml = ConversionOutputRegistryV1::lookup_alias("cml").expect("CML output");
        let cml2 = ConversionOutputRegistryV1::lookup_alias("cml2").expect("CML2 output");

        assert_eq!(cml, cml2);
        assert_eq!(cml.format_id(), CML_SIMPLE_MOLECULE_OUTPUT_FORMAT_V1);
        assert_eq!(cml.profile_id(), CML_SIMPLE_MOLECULE_OUTPUT_PROFILE_V1);
        assert_eq!(cml.output_suffix(), ".cml");
        assert_eq!(
            cml.target(),
            ConversionOutputTargetV1::ChemistryRecordCodec(InterchangeFormatV1::CmlSimpleMolecule)
        );
        assert!(ConversionOutputRegistryV1::lookup_alias("cml1").is_none());
    }

    #[test]
    fn canonical_cdml_and_record_codecs_are_distinct_output_targets() {
        let cdml = ConversionOutputRegistryV1::lookup_alias("cdml").expect("CDML output");

        assert_eq!(cdml.target(), ConversionOutputTargetV1::CanonicalCdml);
        assert_eq!(cdml.target().protocol_format(), InterchangeFormatV1::Cdml);
        assert_eq!(
            ConversionOutputRegistryV1::lookup_chemistry_format(InterchangeFormatV1::Cdml),
            None
        );
        assert_eq!(
            ConversionOutputRegistryV1::lookup_protocol_format(InterchangeFormatV1::Cdml),
            Some(cdml)
        );
        assert_eq!(ConversionOutputRegistryV1::validate_exact_join(), Ok(()));
    }

    #[test]
    fn output_runtime_requirements_remain_closed_facts() {
        assert_eq!(
            ConversionOutputRegistryV1::lookup_alias("cdml")
                .expect("CDML output")
                .runtime_requirement(),
            InterchangeRuntimeRequirementV1::RuntimeFree
        );
    }
}

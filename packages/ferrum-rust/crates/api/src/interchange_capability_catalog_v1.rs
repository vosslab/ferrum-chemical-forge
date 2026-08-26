//! Versioned public snapshot of Ferrum's admitted interchange capabilities.
//!
//! The resolver remains the owner of joins and descriptor lookup. This module
//! owns only the serializable discovery projection consumed by CLI adapters.

use ferrum_chemistry::InterchangeFormatV1;
use serde::Serialize;
use thiserror::Error;

use crate::{
    ConversionInputCapabilityV1, ConversionOutputDescriptorV1, InterchangeCapabilityResolverV1,
    InterchangeCompressionPolicyV1, InterchangeDirectionV1, InterchangeImportRefusalV1,
    InterchangeRuntimeRequirementV1, InterchangeSemanticLossPolicyV1,
};

/// Stable schema discriminator for the interchange-capability catalog.
pub const INTERCHANGE_CAPABILITY_CATALOG_SCHEMA_V1: &str = "ferrum-interchange-capabilities-v1";

/// Failure while constructing the complete, versioned capability catalog.
#[derive(Debug, Error)]
pub enum InterchangeCapabilityCatalogErrorV1 {
    /// Resolver-owned input/output descriptors did not form one exact join.
    #[error("configuration: interchange capability resolver exact join failed: {0:?}")]
    ExactJoin(InterchangeImportRefusalV1),
    /// One admitted input has no matching output descriptor.
    #[error("configuration: no output descriptor matches protocol format {protocol_format:?}")]
    MissingOutput {
        /// Protocol format declared by the unmatched input descriptor.
        protocol_format: InterchangeFormatV1,
    },
}

/// Versioned, runtime-free discovery response for every admitted conversion format.
#[derive(Debug, Serialize)]
pub struct InterchangeCapabilityCatalogV1 {
    schema: &'static str,
    capabilities: Vec<InterchangeCapabilityV1>,
}

impl InterchangeCapabilityCatalogV1 {
    /// Build the complete catalog in the resolver's deterministic input order.
    pub fn snapshot() -> Result<Self, InterchangeCapabilityCatalogErrorV1> {
        InterchangeCapabilityResolverV1::validate_exact_join()
            .map_err(InterchangeCapabilityCatalogErrorV1::ExactJoin)?;
        let capabilities = InterchangeCapabilityResolverV1::input_capabilities()
            .map(|input| {
                InterchangeCapabilityV1::from_join(
                    input,
                    InterchangeCapabilityResolverV1::lookup_output_format(input.protocol_format()),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            schema: INTERCHANGE_CAPABILITY_CATALOG_SCHEMA_V1,
            capabilities,
        })
    }

    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    #[must_use]
    pub fn capabilities(&self) -> &[InterchangeCapabilityV1] {
        &self.capabilities
    }
}

/// One resolver join with its independently identified input and output facts.
#[derive(Debug, Serialize)]
pub struct InterchangeCapabilityV1 {
    protocol_format: InterchangeFormatV1,
    input: InterchangeCapabilityInputV1,
    output: InterchangeCapabilityOutputV1,
}

impl InterchangeCapabilityV1 {
    fn from_join(
        input: ConversionInputCapabilityV1,
        output: Option<&ConversionOutputDescriptorV1>,
    ) -> Result<Self, InterchangeCapabilityCatalogErrorV1> {
        let protocol_format = input.protocol_format();
        let output =
            output.ok_or(InterchangeCapabilityCatalogErrorV1::MissingOutput { protocol_format })?;
        Ok(Self {
            protocol_format,
            input: InterchangeCapabilityInputV1::from_input(input),
            output: InterchangeCapabilityOutputV1::from_output(output),
        })
    }

    #[must_use]
    pub const fn protocol_format(&self) -> InterchangeFormatV1 {
        self.protocol_format
    }

    #[must_use]
    pub const fn input(&self) -> &InterchangeCapabilityInputV1 {
        &self.input
    }

    #[must_use]
    pub const fn output(&self) -> &InterchangeCapabilityOutputV1 {
        &self.output
    }
}

/// Resolver-provided facts applicable while reading one interchange source.
#[derive(Debug, Serialize)]
pub struct InterchangeCapabilityInputV1 {
    canonical_name: &'static str,
    display_name: &'static str,
    format_id: &'static str,
    profile_id: &'static str,
    aliases: Vec<&'static str>,
    suffixes: Vec<&'static str>,
    directions: Vec<InterchangeDirectionV1>,
    max_source_bytes: usize,
    max_response_bytes: Option<usize>,
    compression: InterchangeCompressionPolicyV1,
    semantic_loss_policy: InterchangeSemanticLossPolicyV1,
    runtime_requirement: InterchangeRuntimeRequirementV1,
}

impl InterchangeCapabilityInputV1 {
    fn from_input(input: ConversionInputCapabilityV1) -> Self {
        Self {
            canonical_name: input.canonical_name(),
            display_name: input.display_name(),
            format_id: input.format_id(),
            profile_id: input.profile_id(),
            aliases: input.aliases().to_vec(),
            suffixes: input.suffixes().to_vec(),
            directions: input.directions().to_vec(),
            max_source_bytes: input.max_source_bytes(),
            max_response_bytes: input.max_response_bytes(),
            compression: input.compression_policy(),
            semantic_loss_policy: input.semantic_loss_policy(),
            runtime_requirement: input.runtime_requirement(),
        }
    }

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
    pub fn aliases(&self) -> &[&'static str] {
        &self.aliases
    }

    #[must_use]
    pub fn suffixes(&self) -> &[&'static str] {
        &self.suffixes
    }

    #[must_use]
    pub const fn runtime_requirement(&self) -> InterchangeRuntimeRequirementV1 {
        self.runtime_requirement
    }
}

/// Resolver-provided facts applicable while writing one interchange target.
#[derive(Debug, Serialize)]
pub struct InterchangeCapabilityOutputV1 {
    canonical_name: &'static str,
    display_name: &'static str,
    format_id: &'static str,
    profile_id: &'static str,
    aliases: Vec<&'static str>,
    suffix: &'static str,
    runtime_requirement: InterchangeRuntimeRequirementV1,
}

impl InterchangeCapabilityOutputV1 {
    fn from_output(output: &ConversionOutputDescriptorV1) -> Self {
        Self {
            canonical_name: output.canonical_name(),
            display_name: output.display_name(),
            format_id: output.format_id(),
            profile_id: output.profile_id(),
            aliases: output.aliases().to_vec(),
            suffix: output.output_suffix(),
            runtime_requirement: output.runtime_requirement(),
        }
    }

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
    pub fn aliases(&self) -> &[&'static str] {
        &self.aliases
    }

    #[must_use]
    pub const fn suffix(&self) -> &'static str {
        self.suffix
    }

    #[must_use]
    pub const fn runtime_requirement(&self) -> InterchangeRuntimeRequirementV1 {
        self.runtime_requirement
    }
}

#[cfg(test)]
mod tests {
    use super::{
        INTERCHANGE_CAPABILITY_CATALOG_SCHEMA_V1, InterchangeCapabilityCatalogErrorV1,
        InterchangeCapabilityCatalogV1, InterchangeCapabilityV1,
    };
    use crate::{
        InterchangeCapabilityOutputV1, InterchangeCapabilityResolverV1,
        interchange_output_v1::NON_CANONICAL_FIRST_ALIAS_OUTPUT_DESCRIPTOR_V1,
    };

    #[test]
    fn snapshot_preserves_resolver_joins_and_public_input_membership() {
        let catalog = InterchangeCapabilityCatalogV1::snapshot().expect("current catalog joins");
        assert_eq!(catalog.schema(), INTERCHANGE_CAPABILITY_CATALOG_SCHEMA_V1);
        for input in InterchangeCapabilityResolverV1::input_capabilities() {
            let capability = catalog
                .capabilities()
                .iter()
                .find(|candidate| {
                    let facts = candidate.input();
                    facts.format_id() == input.format_id()
                        && facts.profile_id() == input.profile_id()
                })
                .expect("every resolver input appears in the catalog");
            let facts = capability.input();
            assert_eq!(capability.protocol_format(), input.protocol_format());
            assert_eq!(facts.canonical_name(), input.canonical_name());
            assert_eq!(facts.display_name(), input.display_name());
            assert!(
                input
                    .aliases()
                    .iter()
                    .all(|alias| facts.aliases().contains(alias))
            );
            assert!(
                input
                    .suffixes()
                    .iter()
                    .all(|suffix| facts.suffixes().contains(suffix))
            );
        }
        for output in InterchangeCapabilityResolverV1::output_descriptors() {
            let matching_capabilities = catalog
                .capabilities()
                .iter()
                .filter(|candidate| {
                    let facts = candidate.output();
                    facts.format_id() == output.format_id()
                        && facts.profile_id() == output.profile_id()
                })
                .count();
            assert_eq!(
                matching_capabilities, 1,
                "every resolver output appears in the catalog exactly once"
            );
        }
    }

    #[test]
    fn output_canonical_identity_is_descriptor_owned_not_alias_ordered() {
        let descriptor = NON_CANONICAL_FIRST_ALIAS_OUTPUT_DESCRIPTOR_V1;
        let output = InterchangeCapabilityOutputV1::from_output(&descriptor);

        assert_eq!(descriptor.aliases()[0], "compat");
        assert_eq!(descriptor.canonical_name(), "canonical");
        assert_eq!(output.canonical_name(), descriptor.canonical_name());
        assert_eq!(output.display_name(), descriptor.display_name());
    }

    #[test]
    fn missing_output_descriptor_returns_a_typed_catalog_error() {
        let input = InterchangeCapabilityResolverV1::input_capabilities()
            .next()
            .expect("current resolver has one input capability");

        let error = InterchangeCapabilityV1::from_join(input, None)
            .expect_err("an unmatched input descriptor is not a catalog capability");

        assert!(matches!(
            error,
            InterchangeCapabilityCatalogErrorV1::MissingOutput { protocol_format }
                if protocol_format == input.protocol_format()
        ));
    }

    #[test]
    fn snapshot_preserves_independent_cml_sdf_and_cdml_identities() {
        let catalog = serde_json::to_value(
            InterchangeCapabilityCatalogV1::snapshot().expect("current catalog joins"),
        )
        .expect("catalog serializes");
        for (alias, input_format_id, input_profile_id, output_format_id, output_profile_id) in [
            (
                "cml",
                "cml_simple_molecule_import_v1",
                "ferrum-cml-simple-molecule-import-profile-v1",
                "cml_simple_molecule_output_v1",
                "ferrum-cml-simple-molecule-output-profile-v1",
            ),
            ("sdf", "sdf_v1", "sdf_v1", "sdf_v2000_v1", "sdf_v2000_v1"),
            (
                "cdml",
                "cdml_v1",
                "native_record_conversion_v1",
                "cdml_document_v1",
                "ferrum-cdml-document-output-profile-v1",
            ),
        ] {
            let capability = catalog["capabilities"]
                .as_array()
                .expect("catalog capabilities")
                .iter()
                .find(|candidate| {
                    candidate["input"]["aliases"]
                        .as_array()
                        .is_some_and(|aliases| aliases.iter().any(|value| value == alias))
                })
                .expect("representative capability");
            assert_eq!(capability["input"]["format_id"], input_format_id);
            assert_eq!(capability["input"]["profile_id"], input_profile_id);
            assert_eq!(capability["output"]["format_id"], output_format_id);
            assert_eq!(capability["output"]["profile_id"], output_profile_id);
            assert_ne!(
                capability["input"]["format_id"],
                capability["output"]["format_id"]
            );
            assert_ne!(
                capability["input"]["profile_id"],
                capability["output"]["profile_id"]
            );
        }
    }
}

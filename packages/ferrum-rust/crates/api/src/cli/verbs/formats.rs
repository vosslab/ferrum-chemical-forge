//! Runtime-free rendering of the API-owned interchange capability catalog.

use std::io::Write;

use super::{VerbCliError, write_pretty};
use crate::{InterchangeCapabilityCatalogV1, InterchangeRuntimeRequirementV1};

/// Write the complete resolver snapshot as text or versioned JSON.
pub(crate) fn run(json: bool, stdout: &mut dyn Write) -> Result<(), VerbCliError> {
    let catalog = InterchangeCapabilityCatalogV1::snapshot()?;
    if json {
        return write_pretty(&catalog, stdout);
    }
    for capability in catalog.capabilities() {
        let input = capability.input();
        let output = capability.output();
        writeln!(
            stdout,
            "protocol={:?}: input canonical={} display={} format_id={} profile_id={} aliases=[{}] suffixes=[{}] runtime={} -> output canonical={} display={} format_id={} profile_id={} aliases=[{}] suffix={} runtime={}",
            capability.protocol_format(),
            input.canonical_name(),
            input.display_name(),
            input.format_id(),
            input.profile_id(),
            input.aliases().join(","),
            input.suffixes().join(","),
            runtime_label(input.runtime_requirement()),
            output.canonical_name(),
            output.display_name(),
            output.format_id(),
            output.profile_id(),
            output.aliases().join(","),
            output.suffix(),
            runtime_label(output.runtime_requirement()),
        )
        .map_err(|source| VerbCliError::Write {
            output: "standard output".to_owned(),
            source,
        })?;
    }
    Ok(())
}

const fn runtime_label(requirement: InterchangeRuntimeRequirementV1) -> &'static str {
    match requirement {
        InterchangeRuntimeRequirementV1::RuntimeFree => "runtime_free",
        InterchangeRuntimeRequirementV1::RuntimeRequired => "runtime_required",
    }
}

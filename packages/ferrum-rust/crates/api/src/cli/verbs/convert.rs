//! `ferrum convert` presentation over `chemistry.convert`.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use super::{VerbCliError, execute, publish_or_write, read_text, write_json, write_refusal};
use crate::InterchangeCapabilityResolverV1;
use crate::cli::{InterchangeInputFormat, interchange_input_format_from_protocol_format};
use crate::protocol::{
    ChemistryConvertInputV1, ChemistryConvertRequestV1, OperationProtocolEnvelopeV1,
    OperationProtocolOperationV1, OperationProtocolOutcomeV1,
};

/// Accepted presentation arguments for one conversion request.
pub(crate) struct ConvertOptions {
    pub(crate) input: PathBuf,
    pub(crate) output: Option<PathBuf>,
    pub(crate) input_format: Option<InterchangeInputFormat>,
    pub(crate) output_format: String,
    pub(crate) json: bool,
}

pub(crate) fn run(
    options: ConvertOptions,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let input_format = options
        .input_format
        .or_else(|| infer_input_format(&options.input))
        .ok_or(VerbCliError::MissingInterchangeInputFormat)?;
    let input = match InterchangeCapabilityResolverV1::lookup_input_for_operation(
        input_format.into(),
        crate::InterchangeOperationV1::ChemistryConvert,
    ) {
        Ok(input) => input,
        Err(refusal) => return write_refusal(refusal.recovery_message(), stderr),
    };
    let output = output_format(&options.output_format)?;
    let execution_profile =
        InterchangeCapabilityResolverV1::resolve_execution_profile(input, output);
    run_with_executor(
        options,
        input_format,
        output.target().protocol_format(),
        execution_profile,
        stdin,
        stdout,
        stderr,
        execute,
    )
}

#[cfg(test)]
pub(crate) fn run_with_runtime_for_test<R: crate::protocol::runtime::ChemistryRuntimeV1>(
    options: ConvertOptions,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    runtime: &R,
) -> Result<(), VerbCliError> {
    let input_format = options
        .input_format
        .or_else(|| infer_input_format(&options.input))
        .ok_or(VerbCliError::MissingInterchangeInputFormat)?;
    let input = match InterchangeCapabilityResolverV1::lookup_input_for_operation(
        input_format.into(),
        crate::InterchangeOperationV1::ChemistryConvert,
    ) {
        Ok(input) => input,
        Err(refusal) => return write_refusal(refusal.recovery_message(), stderr),
    };
    let output = output_format(&options.output_format)?;
    let execution_profile =
        InterchangeCapabilityResolverV1::resolve_execution_profile(input, output);
    run_with_executor(
        options,
        input_format,
        output.target().protocol_format(),
        execution_profile,
        stdin,
        stdout,
        stderr,
        |operation| super::execute_with_runtime_for_test(operation, runtime),
    )
}

fn run_with_executor(
    options: ConvertOptions,
    input_format: InterchangeInputFormat,
    output_format: ferrum_document::InterchangeFormatV1,
    execution_profile: crate::ConversionExecutionProfileV1,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    executor: impl FnOnce(
        OperationProtocolOperationV1,
    ) -> Result<OperationProtocolEnvelopeV1, VerbCliError>,
) -> Result<(), VerbCliError> {
    let source = read_text(&options.input, stdin, execution_profile.max_source_bytes())?;
    let envelope = executor(OperationProtocolOperationV1::ChemistryConvert(
        ChemistryConvertRequestV1 {
            input: ChemistryConvertInputV1 {
                format: input_format.into(),
                text: source.text,
            },
            output_format,
        },
    ))?;
    if options.json {
        return write_json(&envelope, stdout);
    }
    match envelope {
        OperationProtocolEnvelopeV1::Success(response) => match response.outcome {
            OperationProtocolOutcomeV1::ChemistryConvert { text, .. } => publish_or_write(
                options.output.as_deref(),
                text.into_bytes(),
                source.retained_source,
                stdout,
                stderr,
            ),
            _ => Err(VerbCliError::UnexpectedOutcome),
        },
        OperationProtocolEnvelopeV1::Error(response) => {
            write_refusal(&response.error.message, stderr)
        }
    }
}

fn output_format(
    alias: &str,
) -> Result<&'static crate::ConversionOutputDescriptorV1, VerbCliError> {
    InterchangeCapabilityResolverV1::lookup_output_alias(alias)
        .ok_or_else(|| VerbCliError::UnsupportedConversionOutput(alias.to_owned()))
}

fn infer_input_format(input: &Path) -> Option<InterchangeInputFormat> {
    if crate::transport::streams::is_standard_stream(input) {
        return None;
    }
    let extension = input.extension()?.to_str()?.to_ascii_lowercase();
    let suffix = format!(".{extension}");
    InterchangeCapabilityResolverV1::lookup_input_suffix(&suffix)
        .ok()
        .map(|descriptor| {
            interchange_input_format_from_protocol_format(descriptor.protocol_format())
        })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::cli::InterchangeInputFormat;
    use crate::cli::commands::InterchangeFormat;

    use super::{ConvertOptions, infer_input_format, run};

    #[test]
    fn common_extensions_map_to_closed_protocol_names() {
        assert_eq!(
            infer_input_format(Path::new("molecule.smi")),
            Some(InterchangeInputFormat::Native(InterchangeFormat::Smiles))
        );
        assert_eq!(
            infer_input_format(Path::new("drawing.cdml")),
            Some(InterchangeInputFormat::Native(InterchangeFormat::Cdml))
        );
        assert_eq!(
            infer_input_format(Path::new("molecule.cml")),
            Some(InterchangeInputFormat::CmlSimpleMolecule)
        );
        assert_eq!(infer_input_format(Path::new("-")), None);
    }

    fn cdxml_conversion_options(input: PathBuf, explicit: bool) -> ConvertOptions {
        ConvertOptions {
            input,
            output: None,
            input_format: explicit.then_some(InterchangeInputFormat::CdxmlSimpleMolecule),
            output_format: "cml".to_owned(),
            json: false,
        }
    }

    #[test]
    fn explicit_cdxml_conversion_refuses_before_reading_the_source() {
        let mut stdin = std::io::empty();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = run(
            cdxml_conversion_options(PathBuf::from("missing-explicit.cdxml"), true),
            &mut stdin,
            &mut stdout,
            &mut stderr,
        )
        .expect_err("document-only input cannot reach a file read");
        assert!(error.was_emitted_to_stream());
        let diagnostic = String::from_utf8(stderr).expect("ASCII diagnostic");
        assert!(diagnostic.contains("ferrum open"));
        assert!(!diagnostic.contains("could not read"));
    }

    #[test]
    fn inferred_cdxml_conversion_refuses_before_reading_the_source() {
        let mut stdin = std::io::empty();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = run(
            cdxml_conversion_options(PathBuf::from("missing-inferred.cdxml"), false),
            &mut stdin,
            &mut stdout,
            &mut stderr,
        )
        .expect_err("document-only inferred input cannot reach a file read");
        assert!(error.was_emitted_to_stream());
        let diagnostic = String::from_utf8(stderr).expect("ASCII diagnostic");
        assert!(diagnostic.contains("ferrum open"));
        assert!(!diagnostic.contains("could not read"));
    }
}

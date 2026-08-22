//! `ferrum convert` presentation over `chemistry.convert`.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::cli::{InterchangeFormat, InterchangeInputFormat};
use crate::protocol::{
    ChemistryConvertInputV1, ChemistryConvertRequestV1, OperationProtocolEnvelopeV1,
    OperationProtocolOperationV1, OperationProtocolOutcomeV1,
};
use ferrum_document::INTERCHANGE_MAX_TEXT_BYTES_V1;

use super::{VerbCliError, execute, publish_or_write, read_text, write_json, write_refusal};

/// Accepted presentation arguments for one conversion request.
pub(crate) struct ConvertOptions {
    pub(crate) input: PathBuf,
    pub(crate) output: Option<PathBuf>,
    pub(crate) input_format: Option<InterchangeInputFormat>,
    pub(crate) output_format: InterchangeFormat,
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
    run_with_executor(options, input_format, stdin, stdout, stderr, execute)
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
    run_with_executor(options, input_format, stdin, stdout, stderr, |operation| {
        super::execute_with_runtime_for_test(operation, runtime)
    })
}

fn run_with_executor(
    options: ConvertOptions,
    input_format: InterchangeInputFormat,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    executor: impl FnOnce(
        OperationProtocolOperationV1,
    ) -> Result<OperationProtocolEnvelopeV1, VerbCliError>,
) -> Result<(), VerbCliError> {
    let source = read_text(&options.input, stdin, input_limit(input_format))?;
    let envelope = executor(OperationProtocolOperationV1::ChemistryConvert(
        ChemistryConvertRequestV1 {
            input: ChemistryConvertInputV1 {
                format: input_format.into(),
                text: source.text,
            },
            output_format: options.output_format.into(),
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

fn input_limit(format: InterchangeInputFormat) -> usize {
    match format {
        InterchangeInputFormat::CmlSimpleMolecule => {
            crate::interchange_import_v1::InterchangeFormatRegistryV1::lookup_input_alias("cml")
                .expect("CML conversion alias is registry-owned")
                .limits()
                .max_source_bytes()
        }
        InterchangeInputFormat::Native(_) => INTERCHANGE_MAX_TEXT_BYTES_V1,
    }
}

fn infer_input_format(input: &Path) -> Option<InterchangeInputFormat> {
    if crate::transport::streams::is_standard_stream(input) {
        return None;
    }
    let extension = input.extension()?.to_str()?.to_ascii_lowercase();
    let registry_suffix = format!(".{extension}");
    if let Ok(descriptor) =
        crate::interchange_import_v1::InterchangeFormatRegistryV1::lookup_input_suffix(
            &registry_suffix,
        )
    {
        return Some(crate::cli::interchange_input_format_from_descriptor(
            descriptor,
        ));
    }
    match extension.as_str() {
        "cdml" => Some(InterchangeInputFormat::Native(InterchangeFormat::Cdml)),
        "smi" | "smiles" => Some(InterchangeInputFormat::Native(InterchangeFormat::Smiles)),
        "inchi" => Some(InterchangeInputFormat::Native(
            InterchangeFormat::InchiStandard,
        )),
        "mol" | "molblock" => Some(InterchangeInputFormat::Native(
            InterchangeFormat::MolblockV2000,
        )),
        "sdf" => Some(InterchangeInputFormat::Native(InterchangeFormat::SdfV2000)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::cli::{InterchangeFormat, InterchangeInputFormat};

    use super::infer_input_format;

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
}

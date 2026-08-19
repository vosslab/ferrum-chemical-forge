//! `ferrum convert` presentation over `chemistry.convert`.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::cli::InterchangeFormat;
use ferrum_document::InterchangeFormatV1;

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
    pub(crate) input_format: Option<InterchangeFormat>,
    pub(crate) output_format: InterchangeFormat,
    pub(crate) json: bool,
}

pub(crate) fn run(
    options: ConvertOptions,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let format = options
        .input_format
        .map(Into::into)
        .or_else(|| infer_input_format(&options.input))
        .ok_or(VerbCliError::MissingInterchangeInputFormat)?;
    let source = read_text(&options.input, stdin, INTERCHANGE_MAX_TEXT_BYTES_V1)?;
    let envelope = execute(OperationProtocolOperationV1::ChemistryConvert(
        ChemistryConvertRequestV1 {
            input: ChemistryConvertInputV1 {
                format,
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

fn infer_input_format(input: &Path) -> Option<InterchangeFormatV1> {
    if crate::transport::streams::is_standard_stream(input) {
        return None;
    }
    let extension = input.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "cdml" => Some(InterchangeFormatV1::Cdml),
        "smi" | "smiles" => Some(InterchangeFormatV1::Smiles),
        "inchi" => Some(InterchangeFormatV1::InchiStandard),
        "mol" | "molblock" => Some(InterchangeFormatV1::MolblockV2000),
        "sdf" => Some(InterchangeFormatV1::SdfV2000),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use ferrum_document::InterchangeFormatV1;

    use super::infer_input_format;

    #[test]
    fn common_extensions_map_to_closed_protocol_names() {
        assert_eq!(
            infer_input_format(Path::new("molecule.smi")),
            Some(InterchangeFormatV1::Smiles)
        );
        assert_eq!(
            infer_input_format(Path::new("drawing.cdml")),
            Some(InterchangeFormatV1::Cdml)
        );
        assert_eq!(infer_input_format(Path::new("-")), None);
    }
}

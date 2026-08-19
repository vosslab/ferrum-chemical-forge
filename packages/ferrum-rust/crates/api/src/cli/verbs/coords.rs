//! `ferrum coords` presentation over `document.generate_coordinates`.

use std::io::{Read, Write};
use std::path::Path;

use crate::protocol::{
    DocumentGenerateCoordinatesRequestV1, OperationProtocolEnvelopeV1,
    OperationProtocolOperationV1, OperationProtocolOutcomeV1,
};

use super::{VerbCliError, execute, publish_or_write, read_document, write_json, write_refusal};

pub(crate) fn run(
    input: &Path,
    output: Option<&Path>,
    json: bool,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let source = read_document(input, stdin)?;
    let envelope = execute(OperationProtocolOperationV1::GenerateCoordinates(
        DocumentGenerateCoordinatesRequestV1 {
            document: source.document,
        },
    ))?;
    if json {
        return write_json(&envelope, stdout);
    }
    match envelope {
        OperationProtocolEnvelopeV1::Success(response) => match response.outcome {
            OperationProtocolOutcomeV1::GenerateCoordinates { document, .. } => publish_or_write(
                output,
                document.into_bytes(),
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

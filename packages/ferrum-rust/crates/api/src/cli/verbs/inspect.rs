//! `ferrum inspect` presentation over `document.inspect`.

use std::io::{Read, Write};
use std::path::Path;

use crate::protocol::{
    DocumentInspectRequestV1, OperationProtocolEnvelopeV1, OperationProtocolOperationV1,
    OperationProtocolOutcomeV1,
};

use super::{VerbCliError, execute, read_document, write_json, write_pretty, write_refusal};

pub(crate) fn run(
    input: &Path,
    json: bool,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let document = read_document(input, stdin)?;
    let envelope = execute(OperationProtocolOperationV1::Inspect(
        DocumentInspectRequestV1 {
            document: document.document,
        },
    ))?;
    if json {
        return write_json(&envelope, stdout);
    }
    match envelope {
        OperationProtocolEnvelopeV1::Success(response) => match response.outcome {
            OperationProtocolOutcomeV1::Inspect { report, .. } => write_pretty(&report, stdout),
            _ => Err(VerbCliError::UnexpectedOutcome),
        },
        OperationProtocolEnvelopeV1::Error(response) => {
            write_refusal(&response.error.message, stderr)
        }
    }
}

//! `ferrum validate` presentation over `document.validate`.

use std::io::{Read, Write};
use std::path::Path;

use crate::protocol::{
    DocumentValidateRequestV1, OperationProtocolEnvelopeV1, OperationProtocolOperationV1,
    OperationProtocolOutcomeV1, ProtocolValidationLevelV1,
};

use super::{VerbCliError, execute, read_document, write_json, write_pretty, write_refusal};
use crate::cli::ValidationLevel;

pub(crate) fn run(
    input: &Path,
    level: ValidationLevel,
    json: bool,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let document = read_document(input, stdin)?;
    let protocol_level = match level {
        ValidationLevel::Structural => ProtocolValidationLevelV1::Structural,
        ValidationLevel::Typed => ProtocolValidationLevelV1::Typed,
    };
    let envelope = execute(OperationProtocolOperationV1::Validate(
        DocumentValidateRequestV1 {
            document: document.document,
            level: protocol_level,
        },
    ))?;
    if json {
        return write_json(&envelope, stdout);
    }
    match envelope {
        OperationProtocolEnvelopeV1::Success(response) => match response.outcome {
            OperationProtocolOutcomeV1::Validate { report, .. } => write_pretty(&report, stdout),
            _ => Err(VerbCliError::UnexpectedOutcome),
        },
        OperationProtocolEnvelopeV1::Error(response) => {
            write_refusal(&response.error.message, stderr)
        }
    }
}

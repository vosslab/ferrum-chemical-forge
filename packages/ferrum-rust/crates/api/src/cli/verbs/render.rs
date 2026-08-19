//! `ferrum render` presentation over `document.render_artifact`.

use std::io::{Read, Write};
use std::path::Path;

use base64::Engine;

use crate::protocol::{
    DocumentRenderArtifactRequestV1, OperationProtocolEnvelopeV1, OperationProtocolOperationV1,
    OperationProtocolOutcomeV1, ProtocolArtifactFormatV1,
};

use super::{VerbCliError, execute, publish_or_write, read_document, write_json, write_refusal};
use crate::cli::ArtifactOutputFormat;

pub(crate) fn run(
    input: &Path,
    output: Option<&Path>,
    output_format: Option<ArtifactOutputFormat>,
    json: bool,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let format = resolve_format(output, output_format)?;
    let document = read_document(input, stdin)?;
    let envelope = execute(OperationProtocolOperationV1::RenderArtifact(
        DocumentRenderArtifactRequestV1 {
            document: document.document,
            format,
        },
    ))?;
    if json {
        return write_json(&envelope, stdout);
    }
    match envelope {
        OperationProtocolEnvelopeV1::Success(response) => match response.outcome {
            OperationProtocolOutcomeV1::RenderArtifact {
                artifact_base64, ..
            } => {
                let bytes = base64::engine::general_purpose::STANDARD.decode(artifact_base64)?;
                publish_or_write(output, bytes, document.retained_source, stdout, stderr)
            }
            _ => Err(VerbCliError::UnexpectedOutcome),
        },
        OperationProtocolEnvelopeV1::Error(response) => {
            write_refusal(&response.error.message, stderr)
        }
    }
}

fn resolve_format(
    output: Option<&Path>,
    selected: Option<ArtifactOutputFormat>,
) -> Result<ProtocolArtifactFormatV1, VerbCliError> {
    if let Some(selected) = selected {
        return Ok(selected.into());
    }
    let extension = output
        .filter(|path| !crate::transport::streams::is_standard_stream(path))
        .and_then(Path::extension)
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    match extension.as_deref() {
        Some("svg") => Ok(ProtocolArtifactFormatV1::Svg),
        Some("pdf") => Ok(ProtocolArtifactFormatV1::Pdf),
        Some("png") => Ok(ProtocolArtifactFormatV1::PngOnePixelPerPointTransparent),
        _ => Err(VerbCliError::MissingArtifactFormat),
    }
}

impl From<ArtifactOutputFormat> for ProtocolArtifactFormatV1 {
    fn from(value: ArtifactOutputFormat) -> Self {
        match value {
            ArtifactOutputFormat::Svg => Self::Svg,
            ArtifactOutputFormat::Pdf => Self::Pdf,
            ArtifactOutputFormat::Png => Self::PngOnePixelPerPointTransparent,
        }
    }
}

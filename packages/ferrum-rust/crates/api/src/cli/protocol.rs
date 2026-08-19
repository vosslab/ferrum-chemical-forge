//! Frozen command-line presentation for the stateless operation protocol V1.

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use ferrum_document::artifact_publication_v1::{
    ArtifactPublicationErrorV1, ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1,
    RetainedSourceFileGuardV1, publish_artifact_v1, retain_regular_source_file_v1,
};
use thiserror::Error;

use crate::cli::engine_bundle;
use crate::protocol::{
    OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1, OperationProtocolEnvelopeV1,
    OperationProtocolInputErrorV1, OperationProtocolOperationV1, OperationProtocolRequestV1,
    execute_operation_v1, execute_operation_with_runtime_v1,
    generated_operation_protocol_schema_v1,
};
use crate::transport::streams::is_standard_stream;

/// Print the generated schema without a banner or secondary report.
pub(crate) fn write_protocol_schema(stdout: &mut dyn Write) -> Result<(), ProtocolCliError> {
    let json = serde_json::to_vec(&generated_operation_protocol_schema_v1())?;
    write_stdout(&json, stdout)
}

/// Execute one request and emit one completed protocol envelope.
pub(crate) fn run_protocol(
    input: &Path,
    output: Option<&Path>,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), ProtocolCliError> {
    let (request, retained_source) = read_request(input, stdin)?;
    let envelope = execute_with_available_runtime(&request).map_err(protocol_input_error)?;
    let mut response = serde_json::to_vec(&envelope)?;
    response.push(b'\n');

    match output {
        None => write_stdout(&response, stdout),
        Some(destination) => publish_response(destination, response, retained_source, stderr),
    }
}

/// Inject the fixed-root engine capability only for operations that require it.
///
/// Invalid request JSON remains the protocol executor's responsibility. If a
/// valid chemistry request has no installed trusted bundle, deliberately use
/// the default runtime so it yields its normal typed `chemistry_unavailable`
/// response instead of turning an admitted request into a CLI transport error.
fn execute_with_available_runtime(
    request: &str,
) -> Result<OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1> {
    let requires_chemistry = serde_json::from_str::<OperationProtocolRequestV1>(request)
        .map(|request| {
            matches!(
                request.operation,
                OperationProtocolOperationV1::ChemistryConvert(_)
                    | OperationProtocolOperationV1::GenerateCoordinates(_)
            )
        })
        .unwrap_or(false);
    if !requires_chemistry {
        return execute_operation_v1(request);
    }
    match engine_bundle::active_runtime() {
        Ok(runtime) => execute_operation_with_runtime_v1(request, &runtime),
        Err(_) => execute_operation_v1(request),
    }
}

fn read_request(
    input: &Path,
    stdin: &mut dyn Read,
) -> Result<(String, Option<RetainedSourceFileGuardV1>), ProtocolCliError> {
    if is_standard_stream(input) {
        let request = read_request_bytes(stdin, "standard input".to_owned())?;
        return Ok((request, None));
    }

    let label = input.display().to_string();
    let mut file = File::open(input).map_err(|source| ProtocolCliError::Input {
        input: label.clone(),
        source,
    })?;
    let retained_source = retain_regular_source_file_v1(file.try_clone().map_err(|source| {
        ProtocolCliError::Input {
            input: label.clone(),
            source,
        }
    })?)
    .map_err(|source| ProtocolCliError::Input {
        input: label.clone(),
        source: io::Error::other(source),
    })?;
    let request = read_request_bytes(&mut file, label)?;
    Ok((request, Some(retained_source)))
}

fn read_request_bytes(reader: &mut dyn Read, input: String) -> Result<String, ProtocolCliError> {
    let limit = OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1;
    let take_limit = u64::try_from(limit)
        .expect("Ferrum protocol request byte limit fits u64")
        .saturating_add(1);
    let mut bytes = Vec::new();
    reader
        .take(take_limit)
        .read_to_end(&mut bytes)
        .map_err(|source| ProtocolCliError::Input {
            input: input.clone(),
            source,
        })?;
    if bytes.len() > limit {
        return Err(ProtocolCliError::RequestLimit(
            OperationProtocolInputErrorV1::ResourceLimit {
                limit,
                observed: bytes.len(),
            },
        ));
    }
    String::from_utf8(bytes).map_err(|source| ProtocolCliError::InvalidUtf8 { input, source })
}

fn protocol_input_error(error: OperationProtocolInputErrorV1) -> ProtocolCliError {
    match error {
        OperationProtocolInputErrorV1::InvalidJson(source) => {
            ProtocolCliError::InvalidRequest(source)
        }
        OperationProtocolInputErrorV1::ResourceLimit { limit, observed } => {
            ProtocolCliError::RequestLimit(OperationProtocolInputErrorV1::ResourceLimit {
                limit,
                observed,
            })
        }
    }
}

fn publish_response(
    destination: &Path,
    response: Vec<u8>,
    retained_source: Option<RetainedSourceFileGuardV1>,
    stderr: &mut dyn Write,
) -> Result<(), ProtocolCliError> {
    let mut request = ArtifactPublicationRequestV1::new(destination.to_path_buf(), response);
    if let Some(source) = retained_source {
        request = request.with_retained_source(source);
    }
    match publish_artifact_v1(request) {
        Ok(ArtifactPublicationOutcomeV1::ConfirmedDurable(_)) => Ok(()),
        Ok(ArtifactPublicationOutcomeV1::DirectoryEntryUnconfirmed(_)) => stderr
            .write_all(
                b"ferrum: warning: publication completed, but directory-entry durability could not be confirmed\n",
            )
            .map_err(|source| ProtocolCliError::Write {
                output: "standard error".to_owned(),
                source,
            }),
        Err(source) => Err(ProtocolCliError::Publication(source)),
    }
}

fn write_stdout(bytes: &[u8], stdout: &mut dyn Write) -> Result<(), ProtocolCliError> {
    stdout
        .write_all(bytes)
        .map_err(|source| ProtocolCliError::Write {
            output: "standard output".to_owned(),
            source,
        })
}

/// Failure before a complete protocol response was emitted or safely published.
#[derive(Debug, Error)]
pub enum ProtocolCliError {
    /// The named request source could not provide UTF-8 input.
    #[error("input: could not read {input}: {source}")]
    Input {
        /// User-visible input label.
        input: String,
        /// Underlying source failure.
        #[source]
        source: io::Error,
    },
    /// The request did not reach a protocol envelope.
    #[error("input: {0}")]
    InvalidRequest(serde_json::Error),
    /// The complete request exceeded the protocol's derived ingress limit.
    #[error("resource_limit: {0}")]
    RequestLimit(OperationProtocolInputErrorV1),
    /// Request bytes were not valid UTF-8 after they passed the byte limit.
    #[error("input: could not decode {input} as UTF-8: {source}")]
    InvalidUtf8 {
        /// User-visible input label.
        input: String,
        /// UTF-8 decoding failure.
        #[source]
        source: std::string::FromUtf8Error,
    },
    /// The completed envelope could not be encoded as JSON.
    #[error("processing: could not encode protocol response: {0}")]
    Json(#[from] serde_json::Error),
    /// A diagnostic or standard-output write failed.
    #[error("publication: could not write {output}: {source}")]
    Write {
        /// User-visible output label.
        output: String,
        /// Underlying write failure.
        #[source]
        source: io::Error,
    },
    /// The safe publisher declined or could not confirm the requested replacement.
    #[error("publication: {0}")]
    Publication(ArtifactPublicationErrorV1),
}

impl ProtocolCliError {
    /// Return the documented process status for this completed-or-unfinished operation.
    #[must_use]
    pub const fn exit_status(&self) -> u8 {
        match self {
            Self::Publication(ArtifactPublicationErrorV1::PossiblyPublished { .. }) => 3,
            Self::Input { .. }
            | Self::InvalidRequest(_)
            | Self::RequestLimit(_)
            | Self::InvalidUtf8 { .. }
            | Self::Json(_)
            | Self::Write { .. }
            | Self::Publication(_) => 1,
        }
    }
}

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
    OperationProtocolErrorCategoryV1, OperationProtocolErrorResponseV1, OperationProtocolErrorV1,
    OperationProtocolInputErrorV1, OperationProtocolOperationV1, OperationProtocolRequestV1,
    ProtocolErrorSchemaV1, ProtocolOperationKindV1, execute_operation_v1,
    execute_operation_with_runtime_v1, generated_operation_protocol_schema_v1,
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
    let mut response = crate::protocol::canonical_protocol_envelope_json_v1(&envelope)?;
    response.push(b'\n');
    emit_protocol_envelope(output, response, retained_source, stdout, stderr, &envelope)
}

/// Execute one named document request after proving its decoded operation matches the route.
pub(crate) fn run_named_document_protocol(
    expected_operation: ProtocolOperationKindV1,
    input: &Path,
    output: Option<&Path>,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), ProtocolCliError> {
    let (request, retained_source) = read_request(input, stdin)?;
    let envelope = execute_named_document_request(expected_operation, &request)
        .map_err(protocol_input_error)?;
    let mut response = crate::protocol::canonical_protocol_envelope_json_v1(&envelope)?;
    response.push(b'\n');
    emit_protocol_envelope(output, response, retained_source, stdout, stderr, &envelope)
}

/// Deliver one complete envelope, then classify the process outcome from its typed result.
///
/// The JSON envelope is the complete public response even when it contains a refusal. The
/// human-oriented verb layer owns the shared classification policy, so named and generic
/// protocol routes cannot silently diverge on a new error category.
fn emit_protocol_envelope(
    output: Option<&Path>,
    response: Vec<u8>,
    retained_source: Option<RetainedSourceFileGuardV1>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    envelope: &OperationProtocolEnvelopeV1,
) -> Result<(), ProtocolCliError> {
    match output {
        None => write_stdout(&response, stdout),
        Some(destination) => publish_response(destination, response, retained_source, stderr),
    }?;
    classify_emitted_protocol_envelope(envelope)
}

/// Translate the shared envelope classifier into the protocol transport's emitted outcome.
fn classify_emitted_protocol_envelope(
    envelope: &OperationProtocolEnvelopeV1,
) -> Result<(), ProtocolCliError> {
    crate::cli::verbs::classify_emitted_protocol_envelope(envelope)
        .map_err(|_| ProtocolCliError::CompletedUnsuccessfulOutcome)
}

/// Reject a decoded operation before execution when its named route differs.
fn execute_named_document_request(
    expected_operation: ProtocolOperationKindV1,
    request_json: &str,
) -> Result<OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1> {
    let request = serde_json::from_str::<OperationProtocolRequestV1>(request_json)?;
    let actual_operation = request.operation.kind();
    if actual_operation != expected_operation {
        return Ok(OperationProtocolEnvelopeV1::Error(
            OperationProtocolErrorResponseV1 {
                schema: ProtocolErrorSchemaV1::V1,
                request_id: Some(request.request_id),
                error: OperationProtocolErrorV1 {
                    category: OperationProtocolErrorCategoryV1::InvalidRequest,
                    operation: Some(actual_operation),
                    message: "named document command does not match the request operation"
                        .to_owned(),
                    resource_limit: None,
                    presentation_author_refusal: None,
                    catalog_placement_refusal: None,
                    reaction_refusal: None,
                    compact_group_materialization_refusal: None,
                    compact_group_attachment_refusal: None,
                    document_molecule_export_refusal: None,
                },
            },
        ));
    }
    execute_with_available_runtime(request_json)
}

/// Execute one protocol request with a controlled Rust-only chemistry runtime.
///
/// This exists only to prove the named CLI protocol serialization against a
/// deterministic typed engine. Production CLI dispatch continues to obtain its
/// capability exclusively from the installed engine-bundle locator.
#[cfg(test)]
pub(crate) fn run_protocol_with_runtime_for_test<
    R: crate::protocol::runtime::ChemistryRuntimeV1,
>(
    input: &Path,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    runtime: &R,
) -> Result<(), ProtocolCliError> {
    let (request, _) = read_request(input, stdin)?;
    let envelope =
        execute_operation_with_runtime_v1(&request, runtime).map_err(protocol_input_error)?;
    let mut response = crate::protocol::canonical_protocol_envelope_json_v1(&envelope)?;
    response.push(b'\n');
    write_stdout(&response, stdout)
}

/// Test-only named-command transport with a reduced SMARTS response budget.
#[cfg(test)]
pub(crate) fn run_protocol_with_runtime_and_smarts_response_limit_for_test<
    R: crate::protocol::runtime::ChemistryRuntimeV1,
>(
    input: &Path,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    runtime: &R,
    response_limit: usize,
) -> Result<(), ProtocolCliError> {
    let (request, _) = read_request(input, stdin)?;
    let envelope =
        crate::protocol::execute_operation_with_runtime_and_smarts_response_limit_for_test(
            &request,
            runtime,
            response_limit,
        )
        .map_err(protocol_input_error)?;
    let mut response = crate::protocol::canonical_protocol_envelope_json_v1(&envelope)?;
    response.push(b'\n');
    write_stdout(&response, stdout)
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
                    | OperationProtocolOperationV1::DocumentMoleculeReport(_)
                    | OperationProtocolOperationV1::DocumentMoleculeExport(_)
                    | OperationProtocolOperationV1::DocumentSmartsQuery(_)
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
    let mut request =
        ArtifactPublicationRequestV1::new(destination.to_path_buf(), response).create_new();
    if let Some(source) = retained_source {
        request = request.with_retained_source(source);
    }
    match publish_artifact_v1(request) {
        Ok(ArtifactPublicationOutcomeV1::ConfirmedDurable(_)) => Ok(()),
        Ok(ArtifactPublicationOutcomeV1::DirectoryEntryUnconfirmed(_)) => {
            stderr
                .write_all(
                    b"ferrum: warning: publication completed, but directory-entry durability could not be confirmed\n",
                )
                .map_err(|source| ProtocolCliError::Write {
                    output: "standard error".to_owned(),
                    source,
                })?;
            Err(ProtocolCliError::DirectoryEntryUnconfirmed)
        }
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
    /// A typed unsuccessful protocol envelope was already emitted to its documented stream.
    #[error("processing: completed operation reported an unsuccessful typed outcome")]
    CompletedUnsuccessfulOutcome,
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
    /// The response was renamed but directory-entry durability was unavailable.
    #[error(
        "publication: response may have been published, but directory-entry durability could not be confirmed"
    )]
    DirectoryEntryUnconfirmed,
}

impl ProtocolCliError {
    /// Whether this error's complete user-facing outcome was already emitted.
    #[must_use]
    pub const fn was_emitted_to_stream(&self) -> bool {
        matches!(self, Self::CompletedUnsuccessfulOutcome)
    }

    /// Return the documented process status for this completed-or-unfinished operation.
    #[must_use]
    pub const fn exit_status(&self) -> u8 {
        match self {
            Self::Publication(ArtifactPublicationErrorV1::PossiblyPublished { .. }) => 3,
            Self::DirectoryEntryUnconfirmed => 3,
            Self::CompletedUnsuccessfulOutcome
            | Self::Input { .. }
            | Self::InvalidRequest(_)
            | Self::RequestLimit(_)
            | Self::InvalidUtf8 { .. }
            | Self::Json(_)
            | Self::Write { .. }
            | Self::Publication(_) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use ferrum_document::DOCUMENT_MOLECULE_EXPORT_TEXT_UTF8_BYTES;

    use super::{ProtocolCliError, run_named_document_protocol};
    use crate::protocol::{
        MAX_REQUEST_ID_UTF8_BYTES_V1, OPERATION_PROTOCOL_RESPONSE_UTF8_BYTES_V1,
        ProtocolOperationKindV1,
    };

    #[test]
    fn directory_entry_unconfirmed_is_a_possibly_published_exit() {
        assert_eq!(ProtocolCliError::DirectoryEntryUnconfirmed.exit_status(), 3);
    }

    #[test]
    fn named_export_refusal_is_emitted_then_returns_unsuccessful_outcome() {
        let request = br#"{
			"schema":"ferrum-operation-request-v1",
			"request_id":"named-export-refusal",
			"operation":{
				"kind":"document.molecule.export.v1",
				"document":{"cdml":"not CDML","expected_revision":0,"expected_digest_hex":"0000000000000000000000000000000000000000000000000000000000000000"},
				"molecule_id":"ferrum-document-object-v1/00112233445566778899aabbccddeeff",
				"format":"canonical_smiles"
			}
		}"#;
        let mut stdin = request.as_slice();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let error = run_named_document_protocol(
            ProtocolOperationKindV1::DocumentMoleculeExport,
            Path::new("-"),
            None,
            &mut stdin,
            &mut stdout,
            &mut stderr,
        )
        .expect_err("typed export refusal must produce a nonzero process outcome");

        assert_eq!(error.exit_status(), 1);
        assert!(error.was_emitted_to_stream());
        assert!(stderr.is_empty());
        let envelope: serde_json::Value =
            serde_json::from_slice(&stdout).expect("named export must emit canonical JSON");
        assert_eq!(
            envelope["error"]["operation"],
            "document.molecule.export.v1"
        );
        assert_eq!(
            envelope["error"]["document_molecule_export_refusal"]["category"],
            "snapshot_not_admitted"
        );
    }

    #[test]
    fn named_routes_classify_a_completed_typed_refusal_generically() {
        let request = br#"{
			"schema":"ferrum-operation-request-v1",
			"request_id":"wrong-named-route",
			"operation":{"kind":"catalog.list.v1"}
		}"#;
        let mut stdin = request.as_slice();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        let error = run_named_document_protocol(
            ProtocolOperationKindV1::DocumentMoleculeExport,
            Path::new("-"),
            None,
            &mut stdin,
            &mut stdout,
            &mut stderr,
        )
        .expect_err("route mismatch must produce a nonzero process outcome");

        assert!(error.was_emitted_to_stream());
        assert!(stderr.is_empty());
        let envelope: serde_json::Value =
            serde_json::from_slice(&stdout).expect("mismatch must retain the typed envelope");
        assert_eq!(envelope["error"]["category"], "invalid_request");
    }

    #[test]
    fn export_text_ceiling_fits_the_shared_response_budget_after_json_escaping() {
        let success_frame = serde_json::json!({
            "schema": "ferrum-operation-response-v1",
            "request_id": "\0".repeat(MAX_REQUEST_ID_UTF8_BYTES_V1),
            "outcome": {
                "kind": "document.molecule.export.v1",
                "export": {
                    "source_revision": u64::MAX,
                    "source_digest_hex": "f".repeat(64),
                    "molecule_id": "ferrum-document-object-v1/00112233445566778899aabbccddeeff",
                    "format": "inchi_fixed_hydrogen",
                    "text": ""
                }
            }
        });
        let refusal_frame = serde_json::json!({
            "schema": "ferrum-operation-error-v1",
            "request_id": "\0".repeat(MAX_REQUEST_ID_UTF8_BYTES_V1),
            "error": {
                "category": "operation_refused",
                "operation": "document_molecule_export",
                "message": "document_molecule_export_refused",
                "resource_limit": null,
                "presentation_author_refusal": null,
                "catalog_placement_refusal": null,
                "reaction_refusal": null,
                "compact_group_materialization_refusal": null,
                "compact_group_attachment_refusal": null,
                "document_molecule_export_refusal": {
                    "category": "representation_unsupported",
                    "recovery": "choose_supported_representation"
                }
            }
        });
        let text_json_escape_bytes = DOCUMENT_MOLECULE_EXPORT_TEXT_UTF8_BYTES
            .checked_mul(6)
            .expect("export text limit escape expansion must fit usize");
        let success_frame_bytes =
            serde_json::to_vec(&success_frame).expect("closed success frame must serialize");
        let refusal_frame_bytes =
            serde_json::to_vec(&refusal_frame).expect("closed refusal frame must serialize");
        let worst_case_success = success_frame_bytes
            .len()
            .checked_add(text_json_escape_bytes)
            .expect("complete escaped success envelope must fit usize");
        assert!(
            worst_case_success <= OPERATION_PROTOCOL_RESPONSE_UTF8_BYTES_V1,
            "worst-case escaped export envelope must fit the shared response budget"
        );
        assert!(
            refusal_frame_bytes.len() <= OPERATION_PROTOCOL_RESPONSE_UTF8_BYTES_V1,
            "worst-case provenance-bearing export refusal must fit the shared response budget"
        );
    }
}

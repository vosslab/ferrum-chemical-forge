//! Human-oriented CLI verbs layered over the frozen operation protocol.

pub(crate) mod convert;
pub(crate) mod coords;
pub(crate) mod document_export_sdf;
pub(crate) mod formats;
pub(crate) mod haworth;
mod input;
pub(crate) mod inspect;
pub(crate) mod inspect_graph;
pub(crate) mod open;
pub(crate) mod render;
pub(crate) mod rewrite;
pub(crate) mod validate;

use std::io::{self, Write};
use std::path::Path;

use ferrum_document::artifact_publication_v1::{
    ArtifactPublicationErrorV1, ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1,
    RetainedSourceFileGuardV1, publish_artifact_v1,
};
use ferrum_document::{DocumentIngressErrorV1, DocumentSessionError};
use thiserror::Error;

use crate::InterchangeCapabilityResolverV1;
use crate::cli::engine_bundle;
use crate::protocol::{
    OperationProtocolEnvelopeV1, OperationProtocolInputErrorV1, OperationProtocolOperationV1,
    OperationProtocolRequestV1, ProtocolRequestSchemaV1, execute_operation_v1,
    execute_operation_with_runtime_v1,
};

pub(crate) use input::{read_document, read_text};

pub(crate) fn execute(
    operation: OperationProtocolOperationV1,
) -> Result<OperationProtocolEnvelopeV1, VerbCliError> {
    let request = OperationProtocolRequestV1 {
        schema: ProtocolRequestSchemaV1::V1,
        request_id: "ferrum-cli".to_owned(),
        operation,
    };
    let json = serde_json::to_string(&request)?;
    let envelope = if operation_requires_chemistry(&request.operation) {
        match engine_bundle::active_runtime() {
            Ok(runtime) => execute_operation_with_runtime_v1(&json, &runtime)?,
            // The no-runtime executor intentionally turns this into a typed,
            // completed chemistry_unavailable response rather than a transport
            // failure.  It also keeps the first four verbs independent of an
            // optional native bundle.
            Err(_) => execute_operation_v1(&json)?,
        }
    } else {
        execute_operation_v1(&json)?
    };
    Ok(envelope)
}

#[cfg(test)]
pub(crate) fn execute_with_runtime_for_test<R: crate::protocol::runtime::ChemistryRuntimeV1>(
    operation: OperationProtocolOperationV1,
    runtime: &R,
) -> Result<OperationProtocolEnvelopeV1, VerbCliError> {
    let request = OperationProtocolRequestV1 {
        schema: ProtocolRequestSchemaV1::V1,
        request_id: "ferrum-cli".to_owned(),
        operation,
    };
    let json = serde_json::to_string(&request)?;
    Ok(execute_operation_with_runtime_v1(&json, runtime)?)
}

fn operation_requires_chemistry(operation: &OperationProtocolOperationV1) -> bool {
    match operation {
        OperationProtocolOperationV1::ChemistryConvert(request) => {
            let input = InterchangeCapabilityResolverV1::lookup_input_format(request.input.format);
            let output =
                InterchangeCapabilityResolverV1::lookup_output_format(request.output_format);
            input.zip(output).is_none_or(|(input, output)| {
                InterchangeCapabilityResolverV1::resolve_execution_profile(input, output)
                    .requires_chemistry_runtime()
            })
        }
        OperationProtocolOperationV1::InspectInterchangeGraph(request) => {
            match InterchangeCapabilityResolverV1::lookup_input_format(request.input.format)
                .and_then(|capability| capability.graph_inspection_profile())
            {
                Some(crate::InterchangeGraphInspectionProfileV1::CmlSimpleMolecule) | None => false,
                Some(crate::InterchangeGraphInspectionProfileV1::SdfNativeSemantic) => true,
            }
        }
        OperationProtocolOperationV1::GenerateCoordinates(_)
        | OperationProtocolOperationV1::DocumentMoleculeReport(_) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use ferrum_document::InterchangeFormatV1;

    use super::operation_requires_chemistry;
    use crate::protocol::{
        ChemistryConvertInputV1, ChemistryConvertRequestV1, InspectInterchangeGraphInputV1,
        InspectInterchangeGraphRequestV1, OperationProtocolOperationV1,
    };

    fn conversion_operation(
        input_format: InterchangeFormatV1,
        output_format: InterchangeFormatV1,
    ) -> OperationProtocolOperationV1 {
        OperationProtocolOperationV1::ChemistryConvert(ChemistryConvertRequestV1 {
            input: ChemistryConvertInputV1 {
                format: input_format,
                text: String::new(),
            },
            output_format,
        })
    }

    #[test]
    fn conversion_runtime_routing_uses_the_capability_join() {
        assert!(!operation_requires_chemistry(&conversion_operation(
            InterchangeFormatV1::CmlSimpleMolecule,
            InterchangeFormatV1::CmlSimpleMolecule,
        )));
        assert!(operation_requires_chemistry(&conversion_operation(
            InterchangeFormatV1::CmlSimpleMolecule,
            InterchangeFormatV1::Smiles,
        )));
    }

    #[test]
    fn cml_graph_inspection_explicitly_uses_the_runtime_free_route() {
        let operation = OperationProtocolOperationV1::InspectInterchangeGraph(
            InspectInterchangeGraphRequestV1 {
                input: InspectInterchangeGraphInputV1 {
                    format: InterchangeFormatV1::CmlSimpleMolecule,
                    text: String::new(),
                },
            },
        );
        assert!(!operation_requires_chemistry(&operation));
    }
}

pub(crate) fn write_json(
    envelope: &OperationProtocolEnvelopeV1,
    stdout: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let mut bytes = serde_json::to_vec(envelope)?;
    bytes.push(b'\n');
    write_stdout(&bytes, stdout)?;
    classify_emitted_protocol_envelope(envelope)
}

/// Return the process outcome after a complete protocol envelope reaches stdout.
///
/// Canonical and human-oriented writers share this boundary so an emitted typed
/// refusal remains the sole payload while still producing a nonzero process status.
pub(crate) fn classify_emitted_protocol_envelope(
    envelope: &OperationProtocolEnvelopeV1,
) -> Result<(), VerbCliError> {
    if matches!(envelope, OperationProtocolEnvelopeV1::Error(_)) {
        return Err(VerbCliError::CompletedUnsuccessfulOutcome);
    }
    Ok(())
}

pub(crate) fn write_pretty<T: serde::Serialize>(
    value: &T,
    stdout: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    write_stdout(&bytes, stdout)
}

pub(crate) fn write_refusal(message: &str, stderr: &mut dyn Write) -> Result<(), VerbCliError> {
    let diagnostic = format!("ferrum: {message}\n");
    stderr
        .write_all(diagnostic.as_bytes())
        .map_err(|source| VerbCliError::Write {
            output: "standard error".to_owned(),
            source,
        })?;
    Err(VerbCliError::CompletedUnsuccessfulOutcome)
}

pub(crate) fn publish_or_write(
    output: Option<&Path>,
    bytes: Vec<u8>,
    retained_source: Option<RetainedSourceFileGuardV1>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let Some(destination) =
        output.filter(|path| !crate::transport::streams::is_standard_stream(path))
    else {
        return write_stdout(&bytes, stdout);
    };
    let mut request = ArtifactPublicationRequestV1::new(destination.to_path_buf(), bytes);
    if let Some(source) = retained_source {
        request = request.with_retained_source(source);
    }
    match publish_artifact_v1(request)? {
        ArtifactPublicationOutcomeV1::ConfirmedDurable(_) => Ok(()),
        ArtifactPublicationOutcomeV1::DirectoryEntryUnconfirmed(_) => stderr
            .write_all(
                b"ferrum: warning: output was written, but directory-entry durability could not be confirmed\n",
            )
            .map_err(|source| VerbCliError::Write {
                output: "standard error".to_owned(),
                source,
            }),
    }
}

pub(crate) fn write_stdout(bytes: &[u8], stdout: &mut dyn Write) -> Result<(), VerbCliError> {
    stdout
        .write_all(bytes)
        .map_err(|source| VerbCliError::Write {
            output: "standard output".to_owned(),
            source,
        })
}

#[derive(Debug, Error)]
pub enum VerbCliError {
    #[error("input: could not decode {input} as UTF-8: {source}")]
    InvalidUtf8 {
        /// User-facing source label.
        input: String,
        /// UTF-8 decoding failure.
        #[source]
        source: std::string::FromUtf8Error,
    },
    /// A named non-CDML source could not be read safely.
    #[error("input: could not read {input}: {source}")]
    Input {
        /// User-facing source label.
        input: String,
        /// Underlying I/O failure.
        #[source]
        source: io::Error,
    },
    /// A bounded interchange source exceeded its transport allocation limit.
    #[error("input: {input} exceeds the {limit}-byte interchange limit")]
    InputTooLarge {
        /// User-facing source label.
        input: String,
        /// Maximum accepted bytes.
        limit: usize,
    },
    /// Structural SMILES did not satisfy the closed detached Haworth profile.
    #[error("input: {0}")]
    HaworthInput(String),
    /// The checked Haworth receipt could not be lowered or serialized.
    #[error("processing: Haworth SVG rendering failed: {0}")]
    HaworthRender(String),
    /// A named source was not valid UTF-8.
    /// The document could not be admitted through the local V1 profile.
    #[error("input: {0}")]
    Document(#[from] DocumentIngressErrorV1),
    /// The admitted document could not produce an owned structural snapshot.
    #[error("input: could not snapshot admitted document: {0}")]
    Snapshot(#[from] DocumentSessionError),
    /// The typed request or response could not cross the JSON protocol boundary.
    #[error("processing: {0}")]
    Json(#[from] serde_json::Error),
    /// The internal request could not reach a completed envelope.
    #[error("processing: {0}")]
    ProtocolInput(#[from] OperationProtocolInputErrorV1),
    /// The selected output format could not be inferred.
    #[error("usage: choose --to svg, --to pdf, or --to png when output has no known extension")]
    MissingArtifactFormat,
    /// The source extension did not map to one closed interchange format.
    #[error(
        "usage: choose --from one of the documented closed formats when input has no known extension"
    )]
    MissingInterchangeInputFormat,
    /// The conversion-output registry has no descriptor for the requested alias.
    #[error("usage: unsupported conversion output format: {0}")]
    UnsupportedConversionOutput(String),
    /// The API-owned interchange capability catalog was internally inconsistent.
    #[error("configuration: {0}")]
    InterchangeCapabilityCatalog(#[from] crate::InterchangeCapabilityCatalogErrorV1),
    /// The descriptor-dispatched interchange import completed with a typed refusal.
    #[error("input: interchange import refused: {0:?}")]
    InterchangeImportRefusal(crate::InterchangeImportRefusalV1),
    /// The protocol returned a different successful operation than the verb requested.
    #[error("processing: protocol returned an unexpected operation result")]
    UnexpectedOutcome,
    /// A typed unsuccessful protocol outcome was already emitted to its documented stream.
    #[error("processing: completed operation reported an unsuccessful typed outcome")]
    CompletedUnsuccessfulOutcome,
    /// An internal render completion was not valid standard base64.
    #[error("processing: protocol returned invalid artifact encoding: {0}")]
    ArtifactEncoding(#[from] base64::DecodeError),
    /// A standard-stream or diagnostic write failed.
    #[error("publication: could not write {output}: {source}")]
    Write {
        /// User-facing stream label.
        output: String,
        /// Underlying I/O failure.
        #[source]
        source: io::Error,
    },
    /// Safe named publication failed or could not be confirmed.
    #[error("publication: {0}")]
    Publication(#[from] ArtifactPublicationErrorV1),
    /// The executable-relative local chemistry runtime was unavailable.
    #[error("processing: local Ferrum chemistry runtime is unavailable")]
    ChemistryUnavailable,
    /// The Rust-owned multi-root document SDF operation refused the request.
    #[error("processing: document SDF export refused: {0}")]
    DocumentMoleculesSdf(#[from] ferrum_document::DocumentMoleculesSdfErrorV2),
}

impl VerbCliError {
    /// Whether this error's complete user-facing outcome was already emitted.
    #[must_use]
    pub const fn was_emitted_to_stream(&self) -> bool {
        matches!(self, Self::CompletedUnsuccessfulOutcome)
    }

    #[must_use]
    pub const fn exit_status(&self) -> u8 {
        match self {
            Self::Publication(ArtifactPublicationErrorV1::PossiblyPublished { .. }) => 3,
            Self::Document(_)
            | Self::Snapshot(_)
            | Self::Input { .. }
            | Self::InputTooLarge { .. }
            | Self::HaworthInput(_)
            | Self::HaworthRender(_)
            | Self::InvalidUtf8 { .. }
            | Self::Json(_)
            | Self::ProtocolInput(_)
            | Self::MissingArtifactFormat
            | Self::MissingInterchangeInputFormat
            | Self::UnsupportedConversionOutput(_)
            | Self::InterchangeCapabilityCatalog(_)
            | Self::InterchangeImportRefusal(_)
            | Self::UnexpectedOutcome
            | Self::CompletedUnsuccessfulOutcome
            | Self::ArtifactEncoding(_)
            | Self::Write { .. }
            | Self::Publication(_)
            | Self::ChemistryUnavailable
            | Self::DocumentMoleculesSdf(_) => 1,
        }
    }
}

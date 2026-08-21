//! Descriptor-dispatched CLI import of one new Ferrum document.

use std::io::{Read, Write};
use std::path::Path;

use crate::cli::engine_bundle;
use crate::document_interchange_import_v1::{
    InterchangeSourceInputV1, admit_interchange_source_v1, prepare_interchange_new_document_v1,
};
use crate::interchange_import_v1::InterchangeFormatDescriptorV1;
use crate::protocol::DocumentInterchangeProvenanceV1;

use super::{VerbCliError, publish_or_write};

pub(crate) fn run(
    input: &Path,
    output: &Path,
    descriptor: &'static InterchangeFormatDescriptorV1,
    json: bool,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let source = if crate::transport::streams::is_standard_stream(input) {
        admit_interchange_source_v1(descriptor, InterchangeSourceInputV1::StandardInput(stdin))
    } else {
        admit_interchange_source_v1(descriptor, InterchangeSourceInputV1::RegularFile(input))
    }
    .map_err(VerbCliError::InterchangeImportRefusal)?;
    let provenance = DocumentInterchangeProvenanceV1 {
        format_id: descriptor.format_id().to_owned(),
        profile_id: descriptor.profile_id().to_owned(),
        source_kind: source.source_kind(),
    };
    let prepared = match engine_bundle::active_runtime() {
        Ok(runtime) => {
            prepare_interchange_new_document_v1(descriptor, &source, &runtime, provenance)
        }
        Err(_) => prepare_interchange_new_document_v1(
            descriptor,
            &source,
            &crate::protocol::runtime::NoChemistryRuntimeV1,
            provenance,
        ),
    }
    .map_err(VerbCliError::InterchangeImportRefusal)?;
    let summary = prepared.summary().clone();
    let response = serde_json::to_vec(&summary).map_err(|_| VerbCliError::UnexpectedOutcome)?;
    if response.len() > descriptor.limits().max_response_bytes() {
        return Err(VerbCliError::InterchangeImportRefusal(
            crate::InterchangeImportRefusalV1::for_reason(
                crate::InterchangeImportRefusalReasonV1::ResponseBytesLimit,
            ),
        ));
    }
    let (session, summary) = prepared
        .commit_and_take_session()
        .map_err(VerbCliError::InterchangeImportRefusal)?;
    let document = session
        .snapshot()
        .map_err(VerbCliError::Snapshot)?
        .cdml()
        .as_bytes()
        .to_vec();
    publish_or_write(
        Some(output),
        document,
        source
            .retained_source()
            .map(|source| source.try_clone())
            .transpose()
            .map_err(|source| VerbCliError::Input {
                input: input.display().to_string(),
                source,
            })?,
        stdout,
        stderr,
    )?;
    if json {
        stdout.write_all(&response).map_err(stdout_error)?;
        stdout.write_all(b"\n").map_err(stdout_error)
    } else {
        writeln!(
            stdout,
            "Opened {}: records={}, atoms={}, bonds={}, revision={}, digest={}, output={}",
            summary.format_id,
            summary.imported_record_count,
            summary.atom_count,
            summary.bond_count,
            summary.document_revision,
            summary.document_digest_hex,
            output.display(),
        )
        .map_err(stdout_error)
    }
}

fn stdout_error(source: std::io::Error) -> VerbCliError {
    VerbCliError::Write {
        output: "standard output".to_owned(),
        source,
    }
}

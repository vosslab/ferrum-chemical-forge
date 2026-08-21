//! CLI workflow for the fixed-target CML new-document operation.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::cml_open_v1::{
    CmlOpenEnvelopeV1, canonical_cml_open_envelope_json_v1, open_cml_new_document_v1,
};
use crate::interchange_import_v1::CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1;
use crate::protocol::DocumentMoleculeInterchangeImportSummaryV1;

use super::{VerbCliError, publish_or_write};

pub(crate) fn run(
    input: &Path,
    output: &Path,
    json: bool,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let source = read_cml_bytes(input, stdin)?;
    let mut completed = open_cml_new_document_v1(&source);
    let envelope = completed.envelope().clone();
    let response = canonical_cml_open_envelope_json_v1(&envelope)
        .map_err(|_| VerbCliError::UnexpectedOutcome)?;
    match envelope {
        CmlOpenEnvelopeV1::Success { summary, .. } => {
            let document = completed
                .take_document_cdml()
                .ok_or(VerbCliError::UnexpectedOutcome)?;
            publish_or_write(Some(output), document.into_bytes(), None, stdout, stderr)?;
            if json {
                stdout
                    .write_all(&response)
                    .map_err(|source| VerbCliError::Write {
                        output: "standard output".to_owned(),
                        source,
                    })?;
                stdout
                    .write_all(b"\n")
                    .map_err(|source| VerbCliError::Write {
                        output: "standard output".to_owned(),
                        source,
                    })
            } else {
                write_open_summary(stdout, output, &summary)
            }
        }
        CmlOpenEnvelopeV1::Refused { refusal, .. } => {
            if json {
                stdout
                    .write_all(&response)
                    .map_err(|source| VerbCliError::Write {
                        output: "standard output".to_owned(),
                        source,
                    })?;
                stdout
                    .write_all(b"\n")
                    .map_err(|source| VerbCliError::Write {
                        output: "standard output".to_owned(),
                        source,
                    })?;
            }
            Err(VerbCliError::CmlOpenRefusal(refusal))
        }
    }
}

fn write_open_summary(
    stdout: &mut dyn Write,
    output: &Path,
    summary: &DocumentMoleculeInterchangeImportSummaryV1,
) -> Result<(), VerbCliError> {
    writeln!(
        stdout,
        "Opened {CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1}: records={}, atoms={}, bonds={}, revision={}, digest={}, output={}",
        summary.inserted_record_count,
        summary.atom_count,
        summary.bond_count,
        summary.document_revision,
        summary.document_digest_hex,
        output.display(),
    )
    .map_err(|source| VerbCliError::Write {
        output: "standard output".to_owned(),
        source,
    })
}

fn read_cml_bytes(input: &Path, stdin: &mut dyn Read) -> Result<Vec<u8>, VerbCliError> {
    let limit = crate::interchange_import_v1::CmlIngressBudgetV1::frozen()
        .raw_utf8_input_bytes()
        .saturating_add(1);
    let mut bytes = Vec::new();
    if crate::transport::streams::is_standard_stream(input) {
        stdin.take(limit as u64).read_to_end(&mut bytes)
    } else {
        File::open(input)
            .map_err(|source| VerbCliError::Input {
                input: input.display().to_string(),
                source,
            })?
            .take(limit as u64)
            .read_to_end(&mut bytes)
    }
    .map_err(|source| VerbCliError::Input {
        input: input.display().to_string(),
        source,
    })?;
    Ok(bytes)
}

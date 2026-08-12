//! Public integration surface and command-line application for Ferrum clients.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use ferrum_document::{
    CoreProjectionError, TypedDocument, TypedDocumentError, TypedRecord, XmlSerializationError,
};
use serde::Serialize;
use thiserror::Error;

const INSPECTION_SCHEMA: &str = "ferrum-cdml-inspection-v1";

/// Ferrum command-line arguments.
#[derive(Debug, Parser)]
#[command(name = "ferrum", version, about = "Ferrum chemical document tools")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Inspect or structurally rewrite a CDML document.
    Cdml {
        #[command(subcommand)]
        command: CdmlCommand,
    },
}

#[derive(Debug, Subcommand)]
enum CdmlCommand {
    /// Print a validated document summary as JSON.
    Inspect {
        /// Input CDML path, or `-` for standard input.
        input: PathBuf,
    },
    /// Parse and structurally re-emit a CDML document.
    Rewrite {
        /// Input CDML path, or `-` for standard input.
        input: PathBuf,
        /// Output CDML path, or `-` for standard output.
        #[arg(short, long)]
        output: PathBuf,
    },
}

/// Stable machine-readable summary emitted by `ferrum cdml inspect`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CdmlInspection {
    schema: &'static str,
    document_version: Option<String>,
    persistent_id_count: usize,
    top_level_record_count: usize,
    typed_record_counts: BTreeMap<&'static str, usize>,
    diagnostic_count: usize,
    molecules: Vec<MoleculeInspection>,
}

/// One molecule's source-order summary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct MoleculeInspection {
    source_id: Option<String>,
    name: Option<String>,
    atom_count: usize,
    group_count: usize,
    text_count: usize,
    query_count: usize,
    bond_count: usize,
}

/// A CDML library operation failed.
#[derive(Debug, Error)]
pub enum CdmlError {
    /// CDML could not be parsed or typed.
    #[error(transparent)]
    Document(#[from] TypedDocumentError),
    /// Typed CDML could not produce a valid core molecule model.
    #[error(transparent)]
    Projection(#[from] CoreProjectionError),
    /// The retained document tree could not be serialized.
    #[error(transparent)]
    Serialization(#[from] XmlSerializationError),
}

/// A command-line operation failed after its arguments were accepted.
#[derive(Debug, Error)]
pub enum CliError {
    /// The requested input could not be read.
    #[error("could not read {input}: {source}")]
    Read {
        /// User-facing input label.
        input: String,
        /// Underlying I/O failure.
        #[source]
        source: io::Error,
    },
    /// CDML processing failed.
    #[error("could not process {input}: {source}")]
    Cdml {
        /// User-facing input label.
        input: String,
        /// Typed CDML failure.
        #[source]
        source: CdmlError,
    },
    /// Inspection JSON could not be encoded.
    #[error("could not encode inspection JSON: {0}")]
    Json(#[from] serde_json::Error),
    /// The requested output could not be written.
    #[error("could not write {output}: {source}")]
    Write {
        /// User-facing output label.
        output: String,
        /// Underlying I/O failure.
        #[source]
        source: io::Error,
    },
}

/// Inspect one CDML document through the typed Ferrum model.
pub fn inspect_cdml(source: &str) -> Result<CdmlInspection, CdmlError> {
    let document = TypedDocument::parse(source)?;
    let projection = document.core_projection()?;
    let mut typed_record_counts = BTreeMap::new();
    let mut diagnostic_count = 0;
    count_typed_records(
        document.root(),
        &mut typed_record_counts,
        &mut diagnostic_count,
    );
    let molecules = projection
        .molecules()
        .iter()
        .map(|molecule| MoleculeInspection {
            source_id: molecule.source_id().map(|value| value.as_str().to_owned()),
            name: molecule.name().map(str::to_owned),
            atom_count: molecule.atoms().len(),
            group_count: molecule.groups().len(),
            text_count: molecule.texts().len(),
            query_count: molecule.queries().len(),
            bond_count: molecule.bonds().len(),
        })
        .collect();
    Ok(CdmlInspection {
        schema: INSPECTION_SCHEMA,
        document_version: projection.document_version().map(str::to_owned),
        persistent_id_count: document.indexed().persistent_id_count(),
        top_level_record_count: document.indexed().records().len(),
        typed_record_counts,
        diagnostic_count,
        molecules,
    })
}

/// Parse and structurally re-emit one CDML document.
pub fn rewrite_cdml(source: &str) -> Result<String, CdmlError> {
    Ok(TypedDocument::parse(source)?.to_xml()?)
}

/// Execute accepted CLI arguments with caller-owned standard streams.
pub fn run(cli: Cli, stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<(), CliError> {
    match cli.command {
        Command::Cdml { command } => match command {
            CdmlCommand::Inspect { input } => {
                let (source, label) = read_input(&input, stdin)?;
                let inspection = inspect_cdml(&source).map_err(|source| CliError::Cdml {
                    input: label,
                    source,
                })?;
                let mut json = serde_json::to_string(&inspection)?;
                json.push('\n');
                write_output(Path::new("-"), json.as_bytes(), stdout)
            }
            CdmlCommand::Rewrite { input, output } => {
                let (source, label) = read_input(&input, stdin)?;
                let rewritten = rewrite_cdml(&source).map_err(|source| CliError::Cdml {
                    input: label,
                    source,
                })?;
                write_output(&output, rewritten.as_bytes(), stdout)
            }
        },
    }
}

fn count_typed_records(
    record: &TypedRecord,
    counts: &mut BTreeMap<&'static str, usize>,
    diagnostics: &mut usize,
) {
    *counts.entry(record.class().name()).or_default() += 1;
    *diagnostics += record.diagnostics().len();
    for child in record.typed_children() {
        count_typed_records(child.record(), counts, diagnostics);
    }
}

fn read_input(path: &Path, stdin: &mut dyn Read) -> Result<(String, String), CliError> {
    let label = stream_label(path, "standard input");
    if path == Path::new("-") {
        let mut source = String::new();
        stdin
            .read_to_string(&mut source)
            .map_err(|source| CliError::Read {
                input: label.clone(),
                source,
            })?;
        Ok((source, label))
    } else {
        fs::read_to_string(path)
            .map(|source| (source, label.clone()))
            .map_err(|source| CliError::Read {
                input: label,
                source,
            })
    }
}

fn write_output(path: &Path, contents: &[u8], stdout: &mut dyn Write) -> Result<(), CliError> {
    let label = stream_label(path, "standard output");
    if path == Path::new("-") {
        stdout
            .write_all(contents)
            .map_err(|source| CliError::Write {
                output: label,
                source,
            })
    } else {
        fs::write(path, contents).map_err(|source| CliError::Write {
            output: label,
            source,
        })
    }
}

fn stream_label(path: &Path, standard_stream: &str) -> String {
    if path == Path::new("-") {
        standard_stream.to_owned()
    } else {
        path.display().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{inspect_cdml, rewrite_cdml};

    const SIMPLE_CDML: &str = r#"<cdml version="0.16"><molecule id="m1"><atom id="a1" name="C"><point x="1" y="2"/></atom></molecule></cdml>"#;

    #[test]
    fn inspection_reports_typed_core_facts() {
        let inspection = inspect_cdml(SIMPLE_CDML).expect("valid CDML inspects");

        assert_eq!(inspection.schema, "ferrum-cdml-inspection-v1");
        assert_eq!(inspection.document_version.as_deref(), Some("0.16"));
        assert_eq!(inspection.persistent_id_count, 2);
        assert_eq!(inspection.top_level_record_count, 1);
        assert_eq!(inspection.typed_record_counts["molecule/atom"], 1);
        assert_eq!(inspection.molecules[0].atom_count, 1);
    }

    #[test]
    fn rewrite_preserves_opaque_payload_structure() {
        let source = r#"<cdml xmlns:q="urn:test"><q:payload id="foreign"><q:item flag="yes"/></q:payload></cdml>"#;
        let rewritten = rewrite_cdml(source).expect("opaque CDML rewrites");
        let inspection = inspect_cdml(&rewritten).expect("rewritten CDML reparses");

        assert_eq!(inspection.persistent_id_count, 1);
        assert!(rewritten.contains("q:payload"));
        assert!(rewritten.contains("flag=\"yes\""));
    }

    #[test]
    fn inspection_rejects_unresolved_core_endpoints() {
        let source = r#"<cdml><molecule><atom id="a1"><point x="0" y="0"/></atom><bond start="a1" end="missing"/></molecule></cdml>"#;
        let error = inspect_cdml(source).expect_err("unresolved endpoint must fail");

        assert!(error.to_string().contains("unknown molecule-local vertex"));
    }
}

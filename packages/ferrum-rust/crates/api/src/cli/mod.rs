use std::io::{Read, Write};

use crate::cli::protocol::{run_protocol, write_protocol_schema};
use crate::cli::verbs::{
    convert, coords, document_export_sdf, haworth, inspect, open, render, rewrite, validate,
};
use crate::interchange_import_v1::{InterchangeFormatDescriptorV1, InterchangeFormatRegistryV1};
use crate::transport::errors::CliError;

pub(crate) mod commands;
pub(crate) mod engine_bundle;
pub(crate) mod protocol;
pub(crate) mod verbs;

pub use commands::Cli;
pub(crate) use commands::{
    ArtifactOutputFormat, Command, DocumentCommand, InterchangeFormat, InterchangeInputFormat,
    NamedDocumentCommand, ProtocolCommand, SdfVersion, ValidationLevel,
    interchange_input_format_from_descriptor,
};

/// Execute accepted CLI arguments with caller-owned standard streams.
pub fn run(
    cli: Cli,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
    match cli.command {
        Command::DocumentMoleculeHydrogenMaterialize { request } => {
            Ok(run_protocol(&request, None, stdin, stdout, stderr)?)
        }
        Command::DocumentAtomOxidationObserve { request } => {
            Ok(run_protocol(&request, None, stdin, stdout, stderr)?)
        }
        Command::Inspect {
            document,
            input_format: _,
            json,
        } => Ok(inspect::run(&document, json, stdin, stdout, stderr)?),
        Command::Validate {
            document,
            input_format: _,
            level,
            json,
        } => Ok(validate::run(
            &document, level, json, stdin, stdout, stderr,
        )?),
        Command::Rewrite {
            document,
            output,
            input_format: _,
            output_format: _,
            json,
        } => Ok(rewrite::run(
            &document,
            output.as_deref(),
            json,
            stdin,
            stdout,
            stderr,
        )?),
        Command::Render {
            document,
            output,
            input_format: _,
            output_format,
            json,
        } => Ok(render::run(
            &document,
            output.as_deref(),
            output_format,
            json,
            stdin,
            stdout,
            stderr,
        )?),
        Command::Convert {
            input,
            output,
            input_format,
            output_format,
            json,
        } => Ok(convert::run(
            convert::ConvertOptions {
                input,
                output,
                input_format,
                output_format,
                json,
            },
            stdin,
            stdout,
            stderr,
        )?),
        Command::Coords {
            document,
            output,
            json,
        } => Ok(coords::run(
            &document,
            output.as_deref(),
            json,
            stdin,
            stdout,
            stderr,
        )?),
        Command::Haworth { smiles, output } => Ok(haworth::run(
            &smiles,
            output.as_deref(),
            stdin,
            stdout,
            stderr,
        )?),
        Command::Open {
            input,
            format,
            output,
            json,
        } => {
            let descriptor = interchange_open_descriptor_for_input(&input, format.as_deref())
                .map_err(crate::cli::verbs::VerbCliError::InterchangeImportRefusal)?;
            Ok(open::run(
                &input, &output, descriptor, json, stdin, stdout, stderr,
            )?)
        }
        Command::Protocol { command } => match command {
            ProtocolCommand::Schema => Ok(write_protocol_schema(stdout)?),
            ProtocolCommand::Run { input, output } => Ok(run_protocol(
                &input,
                output.as_deref(),
                stdin,
                stdout,
                stderr,
            )?),
        },
        Command::Document { command } => match command {
            DocumentCommand::ExportSdf {
                input,
                molecule_ids,
                version,
                output,
            } => Ok(document_export_sdf::run(
                &input,
                &molecule_ids,
                version,
                &output,
                stdin,
                stdout,
                stderr,
            )?),
            DocumentCommand::Command { command } => match command {
                NamedDocumentCommand::CatalogList { input, output }
                | NamedDocumentCommand::CatalogInsert { input, output }
                | NamedDocumentCommand::PresentationAuthor { input, output }
                | NamedDocumentCommand::DocumentCompactGroupMaterialize { input, output }
                | NamedDocumentCommand::DocumentMoleculeSmartsQuery { input, output }
                | NamedDocumentCommand::DocumentMoleculeInterchangeImport { input, output } => Ok(
                    run_protocol(&input, output.as_deref(), stdin, stdout, stderr)?,
                ),
                NamedDocumentCommand::ReactionCreate { input, output } => Ok(run_protocol(
                    &input,
                    output.as_deref(),
                    stdin,
                    stdout,
                    stderr,
                )?),
                NamedDocumentCommand::ReactionList { input, output }
                | NamedDocumentCommand::ReactionObserve { input, output }
                | NamedDocumentCommand::ReactionSelect { input, output }
                | NamedDocumentCommand::ReactionPatchMembership { input, output }
                | NamedDocumentCommand::ReactionDeleteDefinition { input, output }
                | NamedDocumentCommand::ReactionTranslate { input, output } => Ok(run_protocol(
                    &input,
                    output.as_deref(),
                    stdin,
                    stdout,
                    stderr,
                )?),
            },
        },
    }
}

fn interchange_open_descriptor_for_input(
    input: &std::path::Path,
    format: Option<&str>,
) -> Result<&'static InterchangeFormatDescriptorV1, crate::InterchangeImportRefusalV1> {
    if let Some(alias) = format {
        return InterchangeFormatRegistryV1::lookup_input_alias(alias);
    }
    let suffix = input
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .map(|extension| format!(".{extension}"))
        .ok_or_else(|| {
            crate::InterchangeImportRefusalV1::for_reason(
                crate::InterchangeImportRefusalReasonV1::FormatAliasUnsupported,
            )
        })?;
    InterchangeFormatRegistryV1::lookup_input_suffix(&suffix)
}

/// Execute the parsed named SMARTS CLI route with a controlled typed runtime.
///
/// This test-only seam deliberately accepts the same parsed [`Cli`] that the
/// production entry point receives. It is restricted to the named SMARTS
/// command so a controlled runtime can never become a production fallback.
#[cfg(test)]
fn run_with_runtime_for_test<R: crate::protocol::runtime::ChemistryRuntimeV1>(
    cli: Cli,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    runtime: &R,
) -> Result<(), CliError> {
    run_with_runtime_and_smarts_response_limit_for_test(
        cli,
        stdin,
        stdout,
        stderr,
        runtime,
        crate::protocol::DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1,
    )
}

#[cfg(test)]
fn run_with_runtime_and_smarts_response_limit_for_test<
    R: crate::protocol::runtime::ChemistryRuntimeV1,
>(
    cli: Cli,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    runtime: &R,
    response_limit: usize,
) -> Result<(), CliError> {
    match cli.command {
        Command::Convert {
            input,
            output,
            input_format,
            output_format,
            json,
        } => Ok(convert::run_with_runtime_for_test(
            convert::ConvertOptions {
                input,
                output,
                input_format,
                output_format,
                json,
            },
            stdin,
            stdout,
            stderr,
            runtime,
        )?),
        Command::Document {
            command:
                DocumentCommand::Command {
                    command: NamedDocumentCommand::DocumentMoleculeSmartsQuery { input, output },
                },
        } => {
            assert!(
                output.is_none(),
                "controlled named SMARTS test does not publish files"
            );
            let _ = stderr;
            if response_limit == crate::protocol::DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1 {
                Ok(protocol::run_protocol_with_runtime_for_test(
                    &input, stdin, stdout, runtime,
                )?)
            } else {
                Ok(
                    protocol::run_protocol_with_runtime_and_smarts_response_limit_for_test(
                        &input,
                        stdin,
                        stdout,
                        runtime,
                        response_limit,
                    )?,
                )
            }
        }
        _ => panic!("controlled runtime is restricted to chemistry CLI routes"),
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use clap::{CommandFactory, Parser};
    use ferrum_chemistry::{
        ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, SmartsMatchOptions,
        SmartsMatchResult, SmilesMolecule,
    };

    use super::{Cli, interchange_open_descriptor_for_input, run};

    const CDML: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule></cdml>";
    const CML2: &str = r#"<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule><atomArray><atom id="a1" elementType="C" x2="0" y2="0"/><atom id="a2" elementType="O" x2="1" y2="0"/></atomArray><bondArray><bond atomRefs2="a1 a2" order="1"/></bondArray></molecule></cml>"#;

    #[test]
    fn interchange_open_resolves_every_registered_alias_and_suffix() {
        for descriptor in crate::InterchangeFormatRegistryV1::descriptors() {
            let alias = descriptor.input_aliases()[0];
            let from_alias =
                interchange_open_descriptor_for_input(std::path::Path::new("-"), Some(alias))
                    .expect("registered alias should resolve");
            assert_eq!(from_alias.format_id(), descriptor.format_id());

            let path =
                std::path::PathBuf::from(format!("molecule{}", descriptor.input_suffixes()[0]));
            let from_suffix = interchange_open_descriptor_for_input(&path, None)
                .expect("registered suffix should resolve");
            assert_eq!(from_suffix.format_id(), descriptor.format_id());
        }
    }

    #[test]
    fn interchange_open_rejects_unregistered_alias_and_suffix() {
        assert!(
            interchange_open_descriptor_for_input(std::path::Path::new("-"), Some("cml2 "))
                .is_err()
        );
        assert!(
            interchange_open_descriptor_for_input(std::path::Path::new("molecule.xyz"), None)
                .is_err()
        );
    }

    fn run_from_stdin(arguments: &[&str]) -> (Vec<u8>, Vec<u8>) {
        let cli = Cli::try_parse_from(arguments).expect("verb arguments should parse");
        let mut stdin = CDML.as_bytes();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        run(cli, &mut stdin, &mut stdout, &mut stderr).expect("verb should complete");
        (stdout, stderr)
    }

    #[test]
    fn inspect_reads_cdml_from_standard_input() {
        let (stdout, stderr) = run_from_stdin(&["ferrum", "inspect", "-"]);
        let report: serde_json::Value =
            serde_json::from_slice(&stdout).expect("inspection report should be JSON");
        assert_eq!(report["schema"], "ferrum-cdml-inspection-v1");
        assert!(stderr.is_empty());
    }

    #[test]
    fn validate_exposes_the_typed_protocol_operation() {
        let (stdout, stderr) = run_from_stdin(&["ferrum", "validate", "-", "--level", "typed"]);
        let report: serde_json::Value =
            serde_json::from_slice(&stdout).expect("validation report should be JSON");
        assert_eq!(report["schema"], "ferrum-cdml-validation-v1");
        assert!(stderr.is_empty());
    }

    #[test]
    fn rewrite_emits_cdml_to_standard_output() {
        let (stdout, stderr) = run_from_stdin(&["ferrum", "rewrite", "-"]);
        let document = String::from_utf8(stdout).expect("rewritten CDML should be UTF-8");
        assert!(document.starts_with("<cdml xmlns=\"urn:ferrum:cdml\""));
        assert!(stderr.is_empty());
    }

    #[test]
    fn render_emits_the_selected_artifact_to_standard_output() {
        let (stdout, stderr) = run_from_stdin(&["ferrum", "render", "-", "--to", "svg"]);
        assert!(stdout.starts_with(b"<svg"));
        assert!(stderr.is_empty());
    }

    #[test]
    fn engine_verbs_complete_through_the_protocol_envelope() {
        for arguments in [
            vec![
                "ferrum", "convert", "-", "--from", "smiles", "--to", "smiles", "--json",
            ],
            vec!["ferrum", "coords", "-", "--json"],
        ] {
            let cli = Cli::try_parse_from(arguments).expect("engine verb arguments should parse");
            let mut stdin = CDML.as_bytes();
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            run(cli, &mut stdin, &mut stdout, &mut stderr)
                .expect("typed refusal should complete the CLI operation");
            let envelope: serde_json::Value =
                serde_json::from_slice(&stdout).expect("engine verb should return an envelope");
            assert!(
                envelope["schema"].is_string(),
                "the protocol envelope must retain its schema discriminator"
            );
            assert!(stderr.is_empty());
        }
    }

    struct CmlConvertEngine;

    impl ChemEngine for CmlConvertEngine {
        fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "smiles_to_molecule",
            })
        }

        fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "generate_2d_coordinates",
            })
        }

        fn molecule_to_smiles(&self, molecule: &MolGraph) -> Result<String, ChemistryError> {
            assert_eq!(molecule.atoms().len(), 2);
            assert_eq!(molecule.bonds().len(), 1);
            Ok("CO".to_owned())
        }

        fn kekulize(&self, _: &MolGraph, _: KekulizeOptions) -> Result<MolGraph, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "kekulize",
            })
        }
    }

    struct CmlConvertRuntime(CmlConvertEngine);

    impl crate::protocol::runtime::ChemistryRuntimeV1 for CmlConvertRuntime {
        fn with_engine<T>(
            &self,
            operation: impl FnOnce(
                &dyn ChemEngine,
            )
                -> Result<T, crate::protocol::runtime::ChemistryRuntimeErrorV1>,
        ) -> Result<T, crate::protocol::runtime::ChemistryRuntimeErrorV1> {
            operation(&self.0)
        }
    }

    #[test]
    fn convert_accepts_registry_owned_cml2_and_exports_with_the_rust_engine() {
        let cli =
            Cli::try_parse_from(["ferrum", "convert", "-", "--from", "cml2", "--to", "smiles"])
                .expect("CML2 alias should parse through the registry");
        let mut stdin = CML2.as_bytes();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        super::run_with_runtime_for_test(
            cli,
            &mut stdin,
            &mut stdout,
            &mut stderr,
            &CmlConvertRuntime(CmlConvertEngine),
        )
        .expect("CML2 conversion should complete");
        assert_eq!(stdout, b"CO");
        assert!(stderr.is_empty());
    }

    #[test]
    fn convert_refuses_unsupported_cml_before_loading_or_using_the_engine() {
        let cli = Cli::try_parse_from([
            "ferrum", "convert", "-", "--from", "cml", "--to", "smiles", "--json",
        ])
        .expect("CML alias should parse through the registry");
        let mut stdin =
            b"<!DOCTYPE cml><cml xmlns=\"http://www.xml-cml.org/schema/cml2/core\"></cml>"
                .as_slice();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        super::run_with_runtime_for_test(
            cli,
            &mut stdin,
            &mut stdout,
            &mut stderr,
            &CmlConvertRuntime(CmlConvertEngine),
        )
        .expect("typed CML refusal should complete");
        let envelope: serde_json::Value =
            serde_json::from_slice(&stdout).expect("CML refusal is JSON");
        assert_eq!(envelope["error"]["category"], "conversion_unsupported");
        assert_eq!(
            envelope["error"]["message"],
            "interchange_import_refused:DtdForbidden"
        );
        assert!(stderr.is_empty());
    }

    #[test]
    fn named_smarts_query_command_routes_one_complete_protocol_envelope() {
        let session = ferrum_document::DocumentSession::load(CDML).expect("fixture loads");
        let snapshot = session.snapshot().expect("fixture snapshots");
        let digest = snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let request = serde_json::json!({
            "schema": "ferrum-operation-request-v1",
            "request_id": "named-smarts-query",
            "operation": {
                "kind": "document.molecule.smarts.query.v1",
                "document": {"cdml": CDML, "expected_revision": 0, "expected_digest_hex": digest},
                "query": {"kind": "smarts", "value": "[#6]"},
                "limits": {"max_matches_per_molecule": 1, "max_total_matches": 1},
            },
        });
        let cli = Cli::try_parse_from([
            "ferrum",
            "document",
            "command",
            "document.molecule.smarts.query.v1",
            "-",
        ])
        .expect("named SMARTS command parses");
        let input = request.to_string();
        let mut stdin = input.as_bytes();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        run(cli, &mut stdin, &mut stdout, &mut stderr).expect("command returns an envelope");
        let envelope: serde_json::Value = serde_json::from_slice(&stdout).expect("JSON envelope");
        assert_eq!(envelope["request_id"], "named-smarts-query");
        assert!(
            envelope["outcome"]["kind"] == "document.molecule.smarts.query.v1"
                || envelope["error"]["category"] == "chemistry_unavailable"
        );
        assert!(stderr.is_empty());
    }

    struct SelectedSmartsEngine {
        queries: RefCell<Vec<String>>,
    }

    impl ChemEngine for SelectedSmartsEngine {
        fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "smiles",
            })
        }

        fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "coordinates",
            })
        }

        fn smarts_match(
            &self,
            query: &str,
            target: &MolGraph,
            options: SmartsMatchOptions,
        ) -> Result<SmartsMatchResult, ChemistryError> {
            self.queries.borrow_mut().push(query.to_owned());
            SmartsMatchResult::try_from_rows(target, options, vec![vec![0]], true).map_err(|_| {
                ChemistryError::SmartsMatchUnavailable {
                    reason: ferrum_chemistry::SmartsMatchUnavailableReason::MalformedNativeResponse,
                }
            })
        }

        fn molecule_to_smarts(&self, _: &MolGraph) -> Result<String, ChemistryError> {
            Ok("selected-fixture-smarts".to_owned())
        }

        fn kekulize(&self, _: &MolGraph, _: KekulizeOptions) -> Result<MolGraph, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "kekulize",
            })
        }
    }

    struct SelectedSmartsRuntime(SelectedSmartsEngine);

    impl crate::protocol::runtime::ChemistryRuntimeV1 for SelectedSmartsRuntime {
        fn with_engine<T>(
            &self,
            operation: impl FnOnce(
                &dyn ChemEngine,
            )
                -> Result<T, crate::protocol::runtime::ChemistryRuntimeErrorV1>,
        ) -> Result<T, crate::protocol::runtime::ChemistryRuntimeErrorV1> {
            operation(&self.0)
        }
    }

    #[test]
    fn named_smarts_protocol_lowers_selected_molecules_and_emits_bounded_facts() {
        let session = ferrum_document::DocumentSession::load(CDML).expect("fixture loads");
        let snapshot = session.snapshot().expect("fixture snapshots");
        let selected = session
            .observe(0)
            .expect("fixture observation")
            .projection()
            .molecules()[0]
            .id()
            .expect("fixture molecule has a durable identity")
            .as_str()
            .to_owned();
        assert_ne!(
            selected, "m",
            "the named protocol must receive the durable direct-root selector"
        );
        let digest = snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let request = serde_json::json!({
            "schema": "ferrum-operation-request-v1",
            "request_id": "named-selected-smarts-query",
            "operation": {
                "kind": "document.molecule.smarts.query.v1",
                "document": {"cdml": CDML, "expected_revision": 0, "expected_digest_hex": digest},
                "query": {"kind": "selected_molecule", "molecule_id": selected},
                "limits": {"max_matches_per_molecule": 1, "max_total_matches": 1},
            },
        });
        let runtime = SelectedSmartsRuntime(SelectedSmartsEngine {
            queries: RefCell::new(Vec::new()),
        });
        let input = request.to_string();
        let mut stdin = input.as_bytes();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let cli = Cli::try_parse_from([
            "ferrum",
            "document",
            "command",
            "document.molecule.smarts.query.v1",
            "-",
        ])
        .expect("selected SMARTS named command parses");
        super::run_with_runtime_for_test(cli, &mut stdin, &mut stdout, &mut stderr, &runtime)
            .expect("named CLI command completes with controlled chemistry");
        let envelope: serde_json::Value = serde_json::from_slice(&stdout).expect("JSON envelope");
        assert_eq!(envelope["request_id"], "named-selected-smarts-query");
        assert_eq!(
            envelope["outcome"]["kind"], "document.molecule.smarts.query.v1",
            "controlled selected SMARTS protocol refused: {envelope:?}"
        );
        assert_eq!(
            envelope["outcome"]["query"],
            serde_json::json!({
                "schema": "ferrum-document-molecule-smarts-query-v1",
                "traversal": {"kind": "complete"},
                "molecules": [{
                    "source_order": 0,
                    "match_count": 1,
                    "completeness": "truncated",
                }],
            })
        );
        assert_eq!(
            runtime.0.queries.borrow().as_slice(),
            ["selected-fixture-smarts"]
        );
        let serialized = String::from_utf8(stdout).expect("CLI response is UTF-8 JSON");
        for forbidden in [
            "selected-fixture-smarts",
            selected.as_str(),
            CDML,
            "record_id",
            "receipt",
            "adapter",
            "rows",
            "graph",
            "position",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "named CLI response leaked private SMARTS state: {forbidden}"
            );
        }
        assert!(stderr.is_empty());
    }

    #[test]
    fn named_smarts_query_response_admission_is_exact_and_redacted_for_raw_and_selected_forms() {
        let session = ferrum_document::DocumentSession::load(CDML).expect("fixture loads");
        let snapshot = session.snapshot().expect("fixture snapshots");
        let selected = session
            .observe(0)
            .expect("fixture observation")
            .projection()
            .molecules()[0]
            .id()
            .expect("fixture molecule has a durable identity")
            .as_str()
            .to_owned();
        let digest = snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        for query in [
            serde_json::json!({"kind": "smarts", "value": "FERRUM_PRIVATE_RAW_SMARTS"}),
            serde_json::json!({"kind": "selected_molecule", "molecule_id": selected}),
        ] {
            let request = serde_json::json!({
                "schema": "ferrum-operation-request-v1",
                "request_id": "response-admission-correlation",
                "operation": {
                    "kind": "document.molecule.smarts.query.v1",
                    "document": {"cdml": CDML, "expected_revision": 0, "expected_digest_hex": digest},
                    "query": query,
                    "limits": {"max_matches_per_molecule": 1, "max_total_matches": 1},
                },
            });
            let input = request.to_string();
            let cli = || {
                Cli::try_parse_from([
                    "ferrum",
                    "document",
                    "command",
                    "document.molecule.smarts.query.v1",
                    "-",
                ])
                .expect("named SMARTS command parses")
            };
            let runtime = || {
                SelectedSmartsRuntime(SelectedSmartsEngine {
                    queries: RefCell::new(Vec::new()),
                })
            };

            let mut complete_stdin = input.as_bytes();
            let mut complete_stdout = Vec::new();
            let mut complete_stderr = Vec::new();
            let complete_runtime = runtime();
            super::run_with_runtime_and_smarts_response_limit_for_test(
                cli(),
                &mut complete_stdin,
                &mut complete_stdout,
                &mut complete_stderr,
                &complete_runtime,
                usize::MAX,
            )
            .expect("unbounded controlled named SMARTS response completes");
            assert!(complete_stderr.is_empty());
            let canonical_len = complete_stdout.len().checked_sub(1).expect("newline only");
            assert_eq!(complete_stdout.last(), Some(&b'\n'));

            let mut boundary_stdin = input.as_bytes();
            let mut boundary_stdout = Vec::new();
            let mut boundary_stderr = Vec::new();
            let boundary_runtime = runtime();
            super::run_with_runtime_and_smarts_response_limit_for_test(
                cli(),
                &mut boundary_stdin,
                &mut boundary_stdout,
                &mut boundary_stderr,
                &boundary_runtime,
                canonical_len,
            )
            .expect("exact canonical JSON boundary is admitted");
            assert_eq!(boundary_stdout, complete_stdout);
            assert!(boundary_stderr.is_empty());

            let mut over_stdin = input.as_bytes();
            let mut over_stdout = Vec::new();
            let mut over_stderr = Vec::new();
            let over_runtime = runtime();
            super::run_with_runtime_and_smarts_response_limit_for_test(
                cli(),
                &mut over_stdin,
                &mut over_stdout,
                &mut over_stderr,
                &over_runtime,
                canonical_len - 1,
            )
            .expect("over-limit response becomes protocol data");
            assert!(over_stderr.is_empty());
            let rendered = String::from_utf8(over_stdout).expect("UTF-8 JSON response");
            let refusal: serde_json::Value = serde_json::from_str(&rendered).expect("JSON refusal");
            assert_eq!(refusal["request_id"], "response-admission-correlation");
            assert_eq!(refusal["error"]["category"], "resource_limit");
            assert_eq!(
                refusal["error"]["resource_limit"]["reason"], "response_size_exceeded",
                "oversize refusal must retain its resource-limit classification: {refusal:?}"
            );
            assert_eq!(refusal["error"]["message"], "response_size_exceeded");
            assert_eq!(
                refusal["error"]["operation"],
                "document.molecule.smarts.query.v1"
            );
            for forbidden in [
                "FERRUM_PRIVATE_RAW_SMARTS",
                "selected-fixture-smarts",
                CDML,
                "molecules",
                "rows",
                "receipt",
                "record_id",
            ] {
                assert!(
                    !rendered.contains(forbidden),
                    "oversized named SMARTS response leaked {forbidden}"
                );
            }
        }
    }

    #[test]
    fn every_human_verb_help_includes_a_worked_example() {
        let command = Cli::command();
        for (verb, example) in [
            ("inspect", "ferrum inspect drawing.cdml"),
            ("validate", "ferrum validate drawing.cdml --level typed"),
            ("rewrite", "ferrum rewrite drawing.cdml -o cleaned.cdml"),
            ("render", "ferrum render drawing.cdml -o drawing.svg"),
            (
                "convert",
                "ferrum convert aspirin.smi --to sdf_v2000 -o aspirin.sdf",
            ),
            ("coords", "ferrum coords drawing.cdml -o laid-out.cdml"),
        ] {
            let help = command
                .find_subcommand(verb)
                .expect("human verb should exist")
                .clone()
                .render_long_help()
                .to_string();
            assert!(
                help.contains(example),
                "{verb} help should teach one example"
            );
        }
    }
}

//! `ferrum inspect-graph` presentation over the versioned operation protocol.

use std::io::{Read, Write};
use std::path::Path;

use super::{VerbCliError, execute, read_text, write_refusal, write_stdout};
use crate::InterchangeCapabilityResolverV1;
use crate::protocol::{
    InspectGraphFactCoverageStatusV1, InspectInterchangeGraphInputV1,
    InspectInterchangeGraphRequestV1, InspectInterchangeGraphSummaryV1,
    MINIMUM_RESPONSE_SIZE_EXCEEDED_ENVELOPE_BYTES_V1, OperationProtocolEnvelopeV1,
    OperationProtocolOperationV1, OperationProtocolOutcomeV1, ProtocolOperationKindV1,
    canonical_protocol_envelope_json_v1, response_size_exceeded_envelope_v1,
};

pub(crate) fn run(
    input: &Path,
    input_format: &str,
    json: bool,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    run_with_source_reader(
        input,
        input_format,
        json,
        stdin,
        stdout,
        stderr,
        |input, stdin, limit| read_text(input, stdin, limit).map(|source| source.text),
    )
}

fn run_with_source_reader(
    input: &Path,
    input_format: &str,
    json: bool,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    read_source: impl FnOnce(&Path, &mut dyn Read, usize) -> Result<String, VerbCliError>,
) -> Result<(), VerbCliError> {
    let capability = InterchangeCapabilityResolverV1::lookup_input_alias(input_format)
        .map_err(VerbCliError::InterchangeImportRefusal)?;
    let profile = capability.graph_inspection_profile().ok_or_else(|| {
        VerbCliError::InterchangeImportRefusal(crate::InterchangeImportRefusalV1::for_reason(
            crate::InterchangeImportRefusalReasonV1::FormatAliasUnsupported,
        ))
    })?;
    let source = read_source(input, stdin, profile.max_source_bytes())?;
    let envelope = execute(OperationProtocolOperationV1::InspectInterchangeGraph(
        InspectInterchangeGraphRequestV1 {
            input: InspectInterchangeGraphInputV1 {
                format: capability.protocol_format(),
                text: source,
            },
        },
    ))?;
    if json {
        return write_bounded_json(&envelope, profile.max_response_bytes(), stdout);
    }
    match &envelope {
        OperationProtocolEnvelopeV1::Success(response) => match &response.outcome {
            OperationProtocolOutcomeV1::InspectInterchangeGraph { summary } => {
                write_bounded_text(summary, profile.max_response_bytes(), stdout, stderr)
            }
            _ => Err(VerbCliError::UnexpectedOutcome),
        },
        OperationProtocolEnvelopeV1::Error(response) => {
            write_refusal(&response.error.message, stderr)
        }
    }
}

fn write_text(summary: &InspectInterchangeGraphSummaryV1) -> Vec<u8> {
    let mut text = format!(
        "schema: {}\nformat_id: {}\nprofile_id: {}\ngraph meaning: {}\nrecords: {}\natoms: {}\nbonds: {}\n",
        summary.schema,
        summary.format_id,
        summary.profile_id,
        summary.graph_meaning,
        summary.record_count,
        summary.atom_count,
        summary.bond_count,
    );
    for record in &summary.records {
        let source_id = source_molecule_id_text(&record.record_source_id);
        let title = source_molecule_id_text(&record.record_title);
        let properties = source_fact_u32_text(&record.property_count);
        text.push_str(&format!(
            "record {}: source_id={} title={} properties={} atoms={} bonds={}\n",
            record.record_index, source_id, title, properties, record.atom_count, record.bond_count,
        ));
    }
    text.push_str(&format!(
        "coverage: known={}; unknown_when_omitted={}; unsupported={}\n",
        coverage_categories(summary, InspectGraphFactCoverageStatusV1::Known),
        coverage_categories(
            summary,
            InspectGraphFactCoverageStatusV1::UnknownWhenOmitted
        ),
        coverage_categories(summary, InspectGraphFactCoverageStatusV1::Unsupported),
    ));
    text.into_bytes()
}

fn write_bounded_text(
    summary: &InspectInterchangeGraphSummaryV1,
    limit: usize,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let bytes = write_text(summary);
    if bytes.len() <= limit {
        return write_stdout(&bytes, stdout);
    }
    write_refusal("response_size_exceeded", stderr)
}

fn write_bounded_json(
    envelope: &OperationProtocolEnvelopeV1,
    limit: usize,
    stdout: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let mut bytes = canonical_protocol_envelope_json_v1(envelope)?;
    bytes.push(b'\n');
    if bytes.len() <= limit {
        write_stdout(&bytes, stdout)?;
        return super::classify_emitted_protocol_envelope(envelope);
    }
    write_response_size_refusal(envelope, stdout)
}

fn write_response_size_refusal(
    envelope: &OperationProtocolEnvelopeV1,
    stdout: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let refusal = response_size_exceeded_envelope_v1(
        envelope,
        ProtocolOperationKindV1::InspectInterchangeGraph,
    );
    let mut bytes = canonical_protocol_envelope_json_v1(&refusal)?;
    bytes.push(b'\n');
    debug_assert!(
        bytes.len() <= MINIMUM_RESPONSE_SIZE_EXCEEDED_ENVELOPE_BYTES_V1,
        "the declared inspection error-envelope allowance must cover the closed refusal"
    );
    write_stdout(&bytes, stdout)?;
    super::classify_emitted_protocol_envelope(&refusal)
}

fn source_fact_u32_text(source: &crate::protocol::SourceFactV1<u32>) -> String {
    match source {
        crate::protocol::SourceFactV1::Known { value } => format!("known:{value}"),
        crate::protocol::SourceFactV1::Unknown => "unknown".to_owned(),
        crate::protocol::SourceFactV1::Unsupported => "unsupported".to_owned(),
    }
}

fn source_molecule_id_text(source_id: &crate::protocol::SourceFactV1<String>) -> String {
    match source_id {
        crate::protocol::SourceFactV1::Known { value } => format!("known:{value:?}"),
        crate::protocol::SourceFactV1::Unknown => "unknown".to_owned(),
        crate::protocol::SourceFactV1::Unsupported => "unsupported".to_owned(),
    }
}

fn coverage_categories(
    summary: &InspectInterchangeGraphSummaryV1,
    expected: InspectGraphFactCoverageStatusV1,
) -> String {
    let coverage = &summary.declared_fact_coverage;
    [
        ("source_record_ordering", &coverage.source_record_ordering),
        ("atom_count", &coverage.atom_count),
        ("bond_count", &coverage.bond_count),
        ("atom_source_id", &coverage.atom_source_id),
        ("element", &coverage.element),
        ("coordinates", &coverage.coordinates),
        ("bond_endpoints", &coverage.bond_endpoints),
        ("bond_order", &coverage.bond_order),
        ("source_molecule_id", &coverage.source_molecule_id),
        ("formal_charge", &coverage.formal_charge),
        ("isotope", &coverage.isotope),
        ("bond_source_id", &coverage.bond_source_id),
        ("bond_stereo_direction", &coverage.bond_stereo_direction),
        ("radicals", &coverage.radicals),
        ("atom_labels_properties", &coverage.atom_labels_properties),
        ("reaction_atom_maps", &coverage.reaction_atom_maps),
        ("record_source_id", &coverage.record_source_id),
        ("record_title", &coverage.record_title),
        ("property_count", &coverage.property_count),
        ("aromaticity", &coverage.aromaticity),
        ("stereo", &coverage.stereo),
    ]
    .into_iter()
    .filter_map(|(name, status)| (*status == expected).then_some(name))
    .collect::<Vec<_>>()
    .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    const CML: &str = r#"<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule id="m"><atomArray><atom id="a" elementType="C" x2="0" y2="0"/></atomArray></molecule><molecule><atomArray><atom id="b" elementType="O" x2="0" y2="0"/><atom id="c" elementType="H" x2="1" y2="0"/></atomArray><bondArray><bond atomRefs2="b c" order="1"/></bondArray></molecule></cml>"#;

    #[test]
    fn renders_one_shared_outcome_as_text_and_json() {
        let mut text_in = CML.as_bytes();
        let mut text_out = Vec::new();
        let mut text_err = Vec::new();
        run(
            Path::new("-"),
            "cml2",
            false,
            &mut text_in,
            &mut text_out,
            &mut text_err,
        )
        .expect("text");
        let text = String::from_utf8(text_out).expect("UTF-8");
        let mut json_in = CML.as_bytes();
        let mut json_out = Vec::new();
        let mut json_err = Vec::new();
        run(
            Path::new("-"),
            "cml",
            true,
            &mut json_in,
            &mut json_out,
            &mut json_err,
        )
        .expect("JSON");
        let json = serde_json::from_slice::<serde_json::Value>(&json_out).expect("JSON");
        let summary = &json["outcome"]["summary"];
        assert_eq!(json["schema"], "ferrum-operation-response-v1");
        for field in [
            "schema",
            "format_id",
            "profile_id",
            "record_count",
            "atom_count",
            "bond_count",
        ] {
            let value = summary[field]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| summary[field].to_string());
            let text_field = match field {
                "record_count" => "records",
                "atom_count" => "atoms",
                "bond_count" => "bonds",
                _ => field,
            };
            let rendered = format!("{text_field}: {value}");
            assert!(text.contains(&rendered), "text omits {field}: {rendered}");
        }
        assert_eq!(summary["record_count"], 2);
        assert_eq!(summary["atom_count"], 3);
        assert_eq!(summary["bond_count"], 1);
        assert_eq!(summary["records"][0]["record_index"], 0);
        assert_eq!(summary["records"][0]["record_source_id"]["status"], "known");
        assert_eq!(summary["records"][0]["record_source_id"]["value"], "m");
        assert_eq!(summary["records"][1]["record_index"], 1);
        assert_eq!(
            summary["records"][1]["record_source_id"]["status"],
            "unknown"
        );
        for record in summary["records"].as_array().expect("record array") {
            let index = record["record_index"].as_u64().expect("record index");
            let source_id = match record["record_source_id"]["status"]
                .as_str()
                .expect("source-ID status")
            {
                "known" => format!(
                    "known:{:?}",
                    record["record_source_id"]["value"]
                        .as_str()
                        .expect("known source ID")
                ),
                "unknown" => "unknown".to_owned(),
                "unsupported" => "unsupported".to_owned(),
                status => panic!("unexpected source-ID status: {status}"),
            };
            let rendered = format!(
                "record {index}: source_id={source_id} title=unsupported properties=unsupported atoms={} bonds={}",
                record["atom_count"], record["bond_count"],
            );
            assert!(text.contains(&rendered), "text omits record {index}");
        }
        let coverage = text
            .lines()
            .find(|line| line.starts_with("coverage:"))
            .expect("coverage line");
        for (category, status) in summary["declared_fact_coverage"]
            .as_object()
            .expect("coverage object")
        {
            let status_section = coverage
                .split(';')
                .find(|section| section.contains(status.as_str().expect("coverage status")))
                .expect("coverage status section");
            assert!(
                status_section.contains(category),
                "text omits {category} from its {status} coverage section"
            );
        }
    }

    #[test]
    fn source_molecule_id_text_distinguishes_literal_sentinel_ids() {
        use crate::protocol::SourceFactV1;

        assert_eq!(
            source_molecule_id_text(&SourceFactV1::Known {
                value: "unknown".to_owned(),
            }),
            "known:\"unknown\""
        );
        assert_eq!(
            source_molecule_id_text(&SourceFactV1::Known {
                value: "unsupported".to_owned(),
            }),
            "known:\"unsupported\""
        );
        assert_eq!(source_molecule_id_text(&SourceFactV1::Unknown), "unknown");
        assert_eq!(
            source_molecule_id_text(&SourceFactV1::Unsupported),
            "unsupported"
        );
    }

    #[test]
    fn sdf_runtime_refusal_is_the_only_json_stdout_payload() {
        let mut input = "".as_bytes();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = run(
            Path::new("-"),
            "sdf",
            true,
            &mut input,
            &mut stdout,
            &mut stderr,
        )
        .expect_err("SDF refusal");
        assert!(error.was_emitted_to_stream());
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&stdout).expect("JSON")["error"]["category"],
            "chemistry_unavailable"
        );
    }

    #[test]
    fn profile_external_formats_refuse_without_reading_the_source() {
        for input in [Path::new("missing.smi"), Path::new("-")] {
            let mut stdin = "source that must remain unread".as_bytes();
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let error = run_with_source_reader(
                input,
                "smiles",
                true,
                &mut stdin,
                &mut stdout,
                &mut stderr,
                |_, _, _| panic!("unsupported inspection profile read a source"),
            )
            .expect_err("profile-external refusal");
            assert!(matches!(error, VerbCliError::InterchangeImportRefusal(_)));
            assert!(stdout.is_empty());
            assert!(stderr.is_empty());
        }
    }

    #[test]
    fn response_overflow_emits_one_typed_json_refusal_without_success_bytes() {
        struct ShortWriter {
            bytes: Vec<u8>,
        }

        impl Write for ShortWriter {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                let accepted = bytes.len().min(7);
                self.bytes.extend_from_slice(&bytes[..accepted]);
                Ok(accepted)
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let envelope = execute(OperationProtocolOperationV1::InspectInterchangeGraph(
            InspectInterchangeGraphRequestV1 {
                input: InspectInterchangeGraphInputV1 {
                    format: ferrum_chemistry::InterchangeFormatV1::CmlSimpleMolecule,
                    text: CML.to_owned(),
                },
            },
        ))
        .expect("CML envelope");
        let encoded = canonical_protocol_envelope_json_v1(&envelope).expect("JSON envelope");
        let mut stdout = ShortWriter { bytes: Vec::new() };
        let error = write_bounded_json(&envelope, encoded.len(), &mut stdout)
            .expect_err("newline makes the complete response exceed its bound");
        assert!(error.was_emitted_to_stream());
        let refusal: serde_json::Value = serde_json::from_slice(&stdout.bytes).expect("refusal");
        assert_eq!(refusal["error"]["category"], "resource_limit");
        assert_eq!(refusal["error"]["message"], "response_size_exceeded");
    }

    #[test]
    fn short_writing_stdout_receives_the_complete_admitted_success() {
        struct ShortWriter(Vec<u8>);
        impl Write for ShortWriter {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                let accepted = bytes.len().min(3);
                self.0.extend_from_slice(&bytes[..accepted]);
                Ok(accepted)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let envelope = execute(OperationProtocolOperationV1::InspectInterchangeGraph(
            InspectInterchangeGraphRequestV1 {
                input: InspectInterchangeGraphInputV1 {
                    format: ferrum_chemistry::InterchangeFormatV1::CmlSimpleMolecule,
                    text: CML.to_owned(),
                },
            },
        ))
        .expect("CML envelope");
        let mut expected = canonical_protocol_envelope_json_v1(&envelope).expect("JSON");
        expected.push(b'\n');
        let mut stdout = ShortWriter(Vec::new());
        write_bounded_json(&envelope, usize::MAX, &mut stdout).expect("complete success");
        assert_eq!(stdout.0, expected);
    }

    #[test]
    fn text_response_overflow_emits_only_the_human_refusal() {
        let envelope = execute(OperationProtocolOperationV1::InspectInterchangeGraph(
            InspectInterchangeGraphRequestV1 {
                input: InspectInterchangeGraphInputV1 {
                    format: ferrum_chemistry::InterchangeFormatV1::CmlSimpleMolecule,
                    text: CML.to_owned(),
                },
            },
        ))
        .expect("CML envelope");
        let OperationProtocolEnvelopeV1::Success(response) = &envelope else {
            panic!("success envelope expected");
        };
        let OperationProtocolOutcomeV1::InspectInterchangeGraph { summary } = &response.outcome
        else {
            panic!("graph outcome expected");
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = write_bounded_text(summary, 0, &mut stdout, &mut stderr)
            .expect_err("text response overflow");
        assert!(error.was_emitted_to_stream());
        assert!(stdout.is_empty());
        assert!(
            String::from_utf8(stderr)
                .expect("human refusal is UTF-8")
                .contains("response_size_exceeded")
        );
    }

    #[test]
    fn failed_stdout_writer_does_not_append_a_fallback_envelope() {
        struct FailingWriter {
            bytes: Vec<u8>,
            failed: bool,
        }
        impl Write for FailingWriter {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if self.failed {
                    return Err(std::io::Error::other("closed"));
                }
                self.failed = true;
                let accepted = bytes.len().min(5);
                self.bytes.extend_from_slice(&bytes[..accepted]);
                Ok(accepted)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let envelope = execute(OperationProtocolOperationV1::InspectInterchangeGraph(
            InspectInterchangeGraphRequestV1 {
                input: InspectInterchangeGraphInputV1 {
                    format: ferrum_chemistry::InterchangeFormatV1::CmlSimpleMolecule,
                    text: CML.to_owned(),
                },
            },
        ))
        .expect("CML envelope");
        let mut stdout = FailingWriter {
            bytes: Vec::new(),
            failed: false,
        };
        let error =
            write_bounded_json(&envelope, usize::MAX, &mut stdout).expect_err("transport failure");
        assert!(matches!(error, VerbCliError::Write { .. }));
        assert!(
            !stdout
                .bytes
                .windows(b"ferrum-operation-error-v1".len())
                .any(|part| part == b"ferrum-operation-error-v1")
        );
    }
}

use ferrum_chemistry::{
    ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, MoleculeComposition,
    MoleculeCompositionEntry, SmilesMolecule,
};
use ferrum_document::DocumentSession;
use ferrum_domain::{
    MoleculeDiagnosticCodeV1, MoleculeDiagnosticFindingV1, MoleculeDiagnosticLocationV1,
    MoleculeDiagnosticRecoveryV1, MoleculeDiagnosticSeverityV1, NeutralBondCapacityAtomOutcomeV1,
};

use super::super::dto::{
    DocumentMoleculeReportFindingCodeSummaryV1, DocumentMoleculeReportFindingLocationSummaryV1,
    DocumentMoleculeReportFindingRecoverySummaryV1, DocumentMoleculeReportFindingSeveritySummaryV1,
    DocumentMoleculeReportFindingSubjectSummaryV1, ProtocolResourceLimitRecoveryV1,
    ProtocolResourceLimitRefusalV1,
};
use super::super::execution::{
    canonical_protocol_envelope_json_v1,
    execute_operation_with_runtime_and_smarts_response_limit_for_test,
    execute_operation_with_runtime_v1,
};
use super::super::molecule_report_diagnostics_v1::authenticated_report_finding_summary_v1;
use super::super::runtime::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1};
use super::super::{
    DocumentMoleculeReportAggregateOutcomeSummaryV1,
    DocumentMoleculeReportCompositionElementSummaryV1, DocumentMoleculeReportCompositionSummaryV1,
    OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, OperationProtocolEnvelopeV1,
    OperationProtocolErrorCategoryV1, OperationProtocolOutcomeV1,
};
use super::{
    DocumentMoleculeReportAggregateOmissionReasonV1, DocumentMoleculeReportErrorV1,
    DocumentMoleculeReportRequestErrorV1, MAX_MOLECULE_REPORT_SELECTOR_UTF8_BYTES_V1,
    MAX_MOLECULE_REPORT_SELECTORS_V1, ParsedDocumentMoleculeReportRequestV1,
    execute_prepared_document_molecule_report_v1, prepare_document_molecule_report_v1,
    report_summary,
};
use ferrum_document::DocumentBondCapacityOutcomeV1;

struct CompositionEngine;
impl ChemEngine for CompositionEngine {
    fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
        unavailable("smiles_to_molecule")
    }
    fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
        unavailable("generate_2d_coordinates")
    }
    fn molecule_composition(
        &self,
        graph: &MolGraph,
    ) -> Result<MoleculeComposition, ChemistryError> {
        let mut entries = Vec::new();
        for atom in graph.atoms() {
            let key =
                ferrum_chemistry::CompositionElementKey::new(atom.atomic_number(), atom.isotope());
            if let Some((_, count)) = entries.iter_mut().find(|(present, _)| *present == key) {
                *count += 1;
            } else {
                entries.push((key, 1_u64));
            }
        }
        let entries = entries
            .into_iter()
            .map(|(key, count)| {
                MoleculeCompositionEntry::new(
                    key,
                    count,
                    f64::from(key.atomic_number().get()) * count as f64,
                )
                .expect("test entry")
            })
            .collect();
        MoleculeComposition::from_entries(1, 21.0, entries).map_err(|error| {
            ChemistryError::MalformedNativeResponse {
                reason: error.to_string(),
            }
        })
    }
    fn kekulize(&self, _: &MolGraph, _: KekulizeOptions) -> Result<MolGraph, ChemistryError> {
        unavailable("kekulize")
    }
}
fn unavailable<T>(operation: &'static str) -> Result<T, ChemistryError> {
    Err(ChemistryError::OperationUnavailable { operation })
}

struct CompositionRuntime;
impl ChemistryRuntimeV1 for CompositionRuntime {
    fn with_engine<T>(
        &self,
        operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
    ) -> Result<T, ChemistryRuntimeErrorV1> {
        operation(&CompositionEngine)
    }
}
fn observation(source: &str) -> ferrum_document::SessionDocumentObservationV1 {
    DocumentSession::load(source)
        .expect("source loads")
        .observe(0)
        .expect("source projects")
}
fn request(
    observation: &ferrum_document::SessionDocumentObservationV1,
    indices: &[usize],
) -> ParsedDocumentMoleculeReportRequestV1 {
    let ids = indices
        .iter()
        .map(|index| {
            observation.projection().molecules()[*index]
                .id()
                .expect("direct id")
                .clone()
        })
        .collect();
    ParsedDocumentMoleculeReportRequestV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        ids,
    )
    .expect("request")
}

#[test]
fn report_records_follow_document_source_order_not_selector_order() {
    let observation = observation(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"><molecule id=\"first\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"second\"><atom id=\"o\" name=\"O\"><point x=\"1\" y=\"0\"/></atom></molecule></cdml>"
    ));
    let receipt = execute_prepared_document_molecule_report_v1(
        &CompositionEngine,
        prepare_document_molecule_report_v1(&observation, &request(&observation, &[1, 0]))
            .expect("prepare"),
    )
    .expect("execute");
    assert_eq!(
        receipt
            .records()
            .iter()
            .map(|record| record.source().source_id())
            .collect::<Vec<_>>(),
        vec!["first", "second"]
    );
    assert_eq!(
        receipt.records()[0]
            .source()
            .authored_elements()
            .iter()
            .map(|entry| (entry.symbol(), entry.atom_count()))
            .collect::<Vec<_>>(),
        vec![("C", 1)]
    );
    assert_eq!(receipt.records()[0].source().authored_charge(), None);
    assert!(receipt.combined_composition().is_some());
}

#[test]
fn unsupported_composition_is_a_record_finding_and_prevents_partial_combined_value() {
    let observation = observation(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"><molecule id=\"good\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"unsupported\"><atom id=\"n\" name=\"N\" valency=\"3\"><point x=\"1\" y=\"0\"/></atom></molecule></cdml>"
    ));
    let receipt = execute_prepared_document_molecule_report_v1(
        &CompositionEngine,
        prepare_document_molecule_report_v1(&observation, &request(&observation, &[0, 1]))
            .expect("prepare"),
    )
    .expect("execute");
    assert!(receipt.records()[1].composition().is_none());
    assert!(receipt.records()[1].findings().iter().any(|finding| {
        finding.code == DocumentMoleculeReportFindingCodeSummaryV1::UnsupportedAtomFact
    }));
    assert!(receipt.combined_composition().is_none());
    assert!(matches!(
        receipt.document_findings(),
        [finding] if finding.aggregate_omission_reason()
            == DocumentMoleculeReportAggregateOmissionReasonV1::IncompleteRecordComposition
    ));
}

#[test]
fn capacity_outcomes_remain_report_facets() {
    let observation = observation(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"><molecule id=\"within\"><atom id=\"c\" name=\"C\" explicit_hydrogens=\"4\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"exceeds\"><atom id=\"c2\" name=\"C\" explicit_hydrogens=\"4\"><point x=\"1\" y=\"0\"/></atom><atom id=\"o\" name=\"O\"><point x=\"2\" y=\"0\"/></atom><bond id=\"b\" start=\"c2\" end=\"o\" type=\"n1\"/></molecule>",
        "<molecule id=\"unchecked\"><atom id=\"x\" name=\"P\"><point x=\"3\" y=\"0\"/></atom></molecule></cdml>"
    ));
    let receipt = execute_prepared_document_molecule_report_v1(
        &CompositionEngine,
        prepare_document_molecule_report_v1(&observation, &request(&observation, &[0, 1, 2]))
            .expect("prepare"),
    )
    .expect("execute");
    assert!(
        matches!(receipt.records()[0].neutral_bond_capacity(), DocumentBondCapacityOutcomeV1::WithinCapacity { atoms } if matches!(atoms[0].outcome, NeutralBondCapacityAtomOutcomeV1::WithinCapacity { .. }))
    );
    assert!(matches!(
        receipt.records()[1].neutral_bond_capacity(),
        DocumentBondCapacityOutcomeV1::ExceedsCapacity { .. }
    ));
    assert!(matches!(
        receipt.records()[2].neutral_bond_capacity(),
        DocumentBondCapacityOutcomeV1::NotChecked { .. }
    ));
}

#[test]
fn report_findings_follow_the_defined_category_order_with_authenticated_locations() {
    let observation = observation(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
        "<atom id=\"c\" name=\"C\" explicit_hydrogens=\"4\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"o\" name=\"O\"><point x=\"1\" y=\"0\"/></atom>",
        "<text id=\"text\"><point x=\"2\" y=\"0\"/></text>",
        "<compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"3\" y=\"0\"/></compact-group>",
        "<bond id=\"capacity\" start=\"c\" end=\"o\" type=\"n1\"/>",
        "<bond id=\"zero\" start=\"c\" end=\"o\" type=\"n0\"/>",
        "</molecule></cdml>",
    ));
    let receipt = execute_prepared_document_molecule_report_v1(
        &CompositionEngine,
        prepare_document_molecule_report_v1(&observation, &request(&observation, &[0]))
            .expect("prepare"),
    )
    .expect("execute");
    let summary = report_summary(receipt);
    assert_eq!(
        summary.records[0]
            .findings
            .iter()
            .map(|finding| finding.code.as_str())
            .take(4)
            .collect::<Vec<_>>(),
        vec![
            "text_atom_present",
            "neutral_capacity_not_checked",
            "unexpanded_group_present",
            "zero_order_bond",
        ]
    );
    assert_eq!(
        summary.records[0].findings[0].location,
        DocumentMoleculeReportFindingLocationSummaryV1::Vertex {
            identifier: "text".to_owned(),
        }
    );
    assert_eq!(
        summary.records[0].findings[3].location,
        DocumentMoleculeReportFindingLocationSummaryV1::Bond {
            identifier: "zero".to_owned(),
        }
    );
}

#[test]
fn clean_report_serializes_empty_structured_findings() {
    let observation = observation(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    );
    let receipt = execute_prepared_document_molecule_report_v1(
        &CompositionEngine,
        prepare_document_molecule_report_v1(&observation, &request(&observation, &[0]))
            .expect("prepare"),
    )
    .expect("execute");
    let rendered = serde_json::to_value(report_summary(receipt)).expect("DTO serializes");
    assert_eq!(rendered["records"][0]["findings"], serde_json::json!([]));
}

#[test]
fn finding_receipt_maps_complete_closed_domain_facts() {
    let finding = MoleculeDiagnosticFindingV1::new(
        MoleculeDiagnosticSeverityV1::Error,
        MoleculeDiagnosticCodeV1::InvalidElement,
        MoleculeDiagnosticRecoveryV1::CorrectChemicalFacts,
        MoleculeDiagnosticLocationV1::Root,
        Some("unrecognized element symbol"),
    )
    .expect("bounded finding");
    let summary = authenticated_report_finding_summary_v1(None, &finding).expect("root maps");
    assert_eq!(
        (
            summary.severity,
            summary.code,
            summary.recovery,
            summary.detail.as_deref()
        ),
        (
            DocumentMoleculeReportFindingSeveritySummaryV1::Error,
            DocumentMoleculeReportFindingCodeSummaryV1::InvalidElement,
            DocumentMoleculeReportFindingRecoverySummaryV1::CorrectChemicalFacts,
            Some("unrecognized element symbol"),
        )
    );
    assert_eq!(
        summary.location,
        DocumentMoleculeReportFindingLocationSummaryV1::Root
    );
}

#[test]
fn idless_and_untrusted_locations_remain_typed_or_refused() {
    let idless = MoleculeDiagnosticFindingV1::new(
        MoleculeDiagnosticSeverityV1::Warning,
        MoleculeDiagnosticCodeV1::UnsupportedAtomFact,
        MoleculeDiagnosticRecoveryV1::InspectStructure,
        MoleculeDiagnosticLocationV1::Atom {
            source_identifier: None,
        },
        None,
    )
    .expect("idless finding");
    let untrusted = MoleculeDiagnosticFindingV1::new(
        MoleculeDiagnosticSeverityV1::Warning,
        MoleculeDiagnosticCodeV1::UnsupportedAtomFact,
        MoleculeDiagnosticRecoveryV1::InspectStructure,
        MoleculeDiagnosticLocationV1::Atom {
            source_identifier: Some("foreign".to_owned()),
        },
        None,
    )
    .expect("identified finding");
    assert_eq!(
        authenticated_report_finding_summary_v1(None, &idless)
            .expect("idless location lowers")
            .location,
        DocumentMoleculeReportFindingLocationSummaryV1::Unaddressable {
            subject: DocumentMoleculeReportFindingSubjectSummaryV1::Atom,
        }
    );
    assert!(authenticated_report_finding_summary_v1(None, &untrusted).is_err());
}

#[test]
fn receipt_serialization_has_no_legacy_finding_codes() {
    let observation = observation(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    );
    let receipt = execute_prepared_document_molecule_report_v1(
        &CompositionEngine,
        prepare_document_molecule_report_v1(&observation, &request(&observation, &[0]))
            .expect("prepare"),
    )
    .expect("execute");
    let rendered = serde_json::to_value(report_summary(receipt)).expect("DTO serializes");
    assert!(rendered["records"][0].get("finding_codes").is_none());
}

#[test]
fn stale_fence_refuses_the_entire_request() {
    let observation = observation(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"><molecule id=\"m\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    );
    let request = ParsedDocumentMoleculeReportRequestV1::new(
        9,
        *observation.snapshot().digest(),
        vec![
            observation.projection().molecules()[0]
                .id()
                .expect("direct id")
                .clone(),
        ],
    )
    .expect("request");
    assert!(matches!(
        prepare_document_molecule_report_v1(&observation, &request),
        Err(DocumentMoleculeReportErrorV1::Inspection)
    ));
}

#[test]
fn selector_bound_rejects_before_duplicate_analysis() {
    let observation = observation(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"><molecule id=\"m\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    );
    let id = observation.projection().molecules()[0]
        .id()
        .expect("id")
        .clone();
    assert_eq!(
        ParsedDocumentMoleculeReportRequestV1::new(
            0,
            [0; 32],
            vec![id; MAX_MOLECULE_REPORT_SELECTORS_V1 + 1]
        ),
        Err(DocumentMoleculeReportRequestErrorV1::TooManySelectors)
    );
}

#[test]
fn selector_length_refuses_before_document_resolution() {
    let selector = ferrum_document::DocumentObjectIdV1::parse(format!(
        "ferrum-document-object-v1/{}/source/{}",
        "6d6f6c6563756c65",
        "61".repeat(MAX_MOLECULE_REPORT_SELECTOR_UTF8_BYTES_V1),
    ))
    .expect("selector grammar remains valid");
    assert_eq!(
        ParsedDocumentMoleculeReportRequestV1::new(0, [0; 32], vec![selector]),
        Err(DocumentMoleculeReportRequestErrorV1::SelectorTooLong)
    );
}

#[test]
fn one_selected_record_explains_why_combined_composition_is_absent() {
    let observation = observation(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"><molecule id=\"m\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    );
    let receipt = execute_prepared_document_molecule_report_v1(
        &CompositionEngine,
        prepare_document_molecule_report_v1(&observation, &request(&observation, &[0]))
            .expect("prepare"),
    )
    .expect("execute");
    assert!(matches!(
        receipt.document_findings(),
        [finding] if finding.aggregate_omission_reason()
            == DocumentMoleculeReportAggregateOmissionReasonV1::FewerThanTwoSelected
    ));
}

#[test]
fn protocol_maps_literal_isotope_aware_report_facts_without_runtime_detail() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"first\"><atom id=\"c\" name=\"C\" charge=\"1\" isotope=\"13\">",
        "<point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"second\"><atom id=\"o\" name=\"O\" charge=\"1\">",
        "<point x=\"1\" y=\"0\"/></atom></molecule></cdml>"
    );
    let observation = observation(source);
    let digest: String = observation
        .snapshot()
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let ids: Vec<String> = observation
        .projection()
        .molecules()
        .iter()
        .map(|root| root.id().expect("direct root id").as_str().to_owned())
        .collect();
    let request = serde_json::json!({
        "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
        "request_id": "molecule-report",
        "operation": {
            "kind": "document.molecule.report.v1",
            "snapshot": {"cdml": source, "revision": 0, "digest_hex": digest},
            "molecule_ids": ids
        }
    });
    let response = execute_operation_with_runtime_v1(&request.to_string(), &CompositionRuntime)
        .expect("valid protocol JSON");
    let OperationProtocolEnvelopeV1::Success(response) = response else {
        panic!("report should complete with the injected test runtime");
    };
    let OperationProtocolOutcomeV1::DocumentMoleculeReport { report } = response.outcome else {
        panic!("molecule report outcome expected");
    };
    let isotope_carbon = DocumentMoleculeReportCompositionSummaryV1 {
        formula: "[13C]+".to_owned(),
        net_formal_charge: 1,
        average_molecular_weight_da: 6.0,
        monoisotopic_mass_da: 21.0,
        elements: vec![DocumentMoleculeReportCompositionElementSummaryV1 {
            symbol: "C".to_owned(),
            isotope: Some(13),
            atom_count: 1,
            average_mass_contribution_da: 6.0,
            mass_percentage: 100.0,
        }],
    };
    let oxygen = DocumentMoleculeReportCompositionSummaryV1 {
        formula: "O+".to_owned(),
        net_formal_charge: 1,
        average_molecular_weight_da: 8.0,
        monoisotopic_mass_da: 21.0,
        elements: vec![DocumentMoleculeReportCompositionElementSummaryV1 {
            symbol: "O".to_owned(),
            isotope: None,
            atom_count: 1,
            average_mass_contribution_da: 8.0,
            mass_percentage: 100.0,
        }],
    };
    let combined = DocumentMoleculeReportCompositionSummaryV1 {
        formula: "[13C]O+2".to_owned(),
        net_formal_charge: 2,
        average_molecular_weight_da: 14.0,
        monoisotopic_mass_da: 42.0,
        elements: vec![
            DocumentMoleculeReportCompositionElementSummaryV1 {
                symbol: "C".to_owned(),
                isotope: Some(13),
                atom_count: 1,
                average_mass_contribution_da: 6.0,
                mass_percentage: 42.857_142_857_142_854,
            },
            DocumentMoleculeReportCompositionElementSummaryV1 {
                symbol: "O".to_owned(),
                isotope: None,
                atom_count: 1,
                average_mass_contribution_da: 8.0,
                mass_percentage: 57.142_857_142_857_14,
            },
        ],
    };
    assert_eq!(report.records[0].composition, Some(isotope_carbon));
    assert_eq!(report.records[1].composition, Some(oxygen));
    assert_eq!(
        report.aggregate,
        DocumentMoleculeReportAggregateOutcomeSummaryV1::Complete {
            composition: combined,
        }
    );
    let rendered = serde_json::to_string(&report).expect("DTO serializes");
    assert!(!rendered.contains("<cdml xmlns=\"urn:ferrum:cdml\""));
    assert!(!rendered.contains("adapter"));
}

#[test]
fn aggregate_outcome_serializes_closed_branches_and_refuses_impossible_states() {
    let complete = serde_json::json!({
        "kind": "complete",
        "composition": {
            "formula": "[13C]+",
            "net_formal_charge": 1,
            "average_molecular_weight_da": 6.0,
            "monoisotopic_mass_da": 21.0,
            "elements": [{
                "symbol": "C",
                "isotope": 13,
                "atom_count": 1,
                "average_mass_contribution_da": 6.0,
                "mass_percentage": 100.0
            }]
        }
    });
    let omitted = serde_json::json!({
        "kind": "omitted",
        "reason": "incomplete_record_composition",
        "recovery": "choose_supported_representation"
    });
    let complete_outcome =
        serde_json::from_value::<DocumentMoleculeReportAggregateOutcomeSummaryV1>(complete.clone())
            .expect("complete aggregate decodes");
    let omitted_outcome =
        serde_json::from_value::<DocumentMoleculeReportAggregateOutcomeSummaryV1>(omitted.clone())
            .expect("closed omitted aggregate decodes");
    assert_eq!(
        serde_json::to_value(complete_outcome).expect("complete reserializes"),
        complete
    );
    assert_eq!(
        serde_json::to_value(omitted_outcome).expect("omitted reserializes"),
        omitted
    );
    for invalid in [
        serde_json::json!({
            "kind": "complete",
            "composition": {"formula": "C", "net_formal_charge": 0,
                "average_molecular_weight_da": 12.0, "monoisotopic_mass_da": 12.0,
                "elements": []},
            "reason": "fewer_than_two_selected"
        }),
        serde_json::json!({
            "kind": "omitted",
            "reason": "not_a_protocol_reason"
        }),
        serde_json::json!({
            "kind": "omitted",
            "reason": "fewer_than_two_selected",
            "composition": {"formula": "C", "net_formal_charge": 0,
                "average_molecular_weight_da": 12.0, "monoisotopic_mass_da": 12.0,
                "elements": []}
        }),
    ] {
        assert!(
            serde_json::from_value::<DocumentMoleculeReportAggregateOutcomeSummaryV1>(invalid)
                .is_err()
        );
    }
}

#[test]
fn mapper_serializes_both_closed_aggregate_omissions() {
    let one = observation(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"one\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    );
    let one_receipt = execute_prepared_document_molecule_report_v1(
        &CompositionEngine,
        prepare_document_molecule_report_v1(&one, &request(&one, &[0])).expect("one prepares"),
    )
    .expect("one executes");
    let incomplete = observation(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"good\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"unsupported\"><atom id=\"n\" name=\"N\" valency=\"3\"><point x=\"1\" y=\"0\"/></atom></molecule></cdml>"
    ));
    let incomplete_receipt = execute_prepared_document_molecule_report_v1(
        &CompositionEngine,
        prepare_document_molecule_report_v1(&incomplete, &request(&incomplete, &[0, 1]))
            .expect("incomplete prepares"),
    )
    .expect("incomplete executes");
    assert_eq!(
        serde_json::to_value(report_summary(one_receipt)).expect("one serializes")["aggregate"],
        serde_json::json!({
            "kind": "omitted",
            "reason": "fewer_than_two_selected",
            "recovery": "none"
        })
    );
    assert_eq!(
        serde_json::to_value(report_summary(incomplete_receipt)).expect("incomplete serializes")["aggregate"],
        serde_json::json!({
            "kind": "omitted",
            "reason": "incomplete_record_composition",
            "recovery": "choose_supported_representation"
        })
    );
}

#[test]
fn protocol_missing_runtime_is_a_redacted_chemistry_refusal() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";
    let observation = observation(source);
    let digest: String = observation
        .snapshot()
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let id = observation.projection().molecules()[0]
        .id()
        .expect("direct root id")
        .as_str()
        .to_owned();
    let request = serde_json::json!({
        "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
        "request_id": "missing-runtime",
        "operation": {"kind": "document.molecule.report.v1",
            "snapshot": {"cdml": source, "revision": 0, "digest_hex": digest}, "molecule_ids": [id]}
    });
    let response = super::super::execution::execute_operation_v1(&request.to_string())
        .expect("valid protocol JSON");
    let OperationProtocolEnvelopeV1::Error(response) = response else {
        panic!("missing runtime must refuse");
    };
    assert_eq!(
        response.error.category,
        OperationProtocolErrorCategoryV1::ChemistryUnavailable
    );
    assert!(!response.error.message.contains("path"));
}

#[test]
fn detached_protocol_report_preserves_nonzero_snapshot_provenance_and_source_order() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"first\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"second\"><atom id=\"o\" name=\"O\"><point x=\"1\" y=\"0\"/></atom></molecule></cdml>"
    );
    let observation = observation(source);
    let digest: String = observation
        .snapshot()
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let ids: Vec<String> = observation
        .projection()
        .molecules()
        .iter()
        .rev()
        .map(|root| root.id().expect("direct root id").as_str().to_owned())
        .collect();
    let request = serde_json::json!({
        "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
        "request_id": "nonzero-report-snapshot",
        "operation": {
            "kind": "document.molecule.report.v1",
            "snapshot": {"cdml": source, "revision": 7, "digest_hex": digest},
            "molecule_ids": ids,
        }
    });
    let envelope = execute_operation_with_runtime_v1(&request.to_string(), &CompositionRuntime)
        .expect("valid report request");
    let OperationProtocolEnvelopeV1::Success(response) = envelope else {
        panic!("verified snapshot must report");
    };
    let OperationProtocolOutcomeV1::DocumentMoleculeReport { report } = response.outcome else {
        panic!("report outcome expected");
    };
    assert_eq!(report.source_revision, 7);
    assert_eq!(report.source_digest_hex, digest);
    assert_eq!(
        report
            .records
            .iter()
            .map(|record| record.source_id.as_str())
            .collect::<Vec<_>>(),
        vec!["first", "second"]
    );
}

#[test]
fn detached_protocol_report_refuses_a_digest_that_does_not_authenticate_cdml() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";
    let request = serde_json::json!({
        "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
        "request_id": "invalid-report-snapshot",
        "operation": {
            "kind": "document.molecule.report.v1",
            "snapshot": {"cdml": source, "revision": 4, "digest_hex": "00".repeat(32)},
            "molecule_ids": ["m"],
        }
    });
    let envelope = execute_operation_with_runtime_v1(&request.to_string(), &CompositionRuntime)
        .expect("valid protocol JSON");
    let OperationProtocolEnvelopeV1::Error(response) = envelope else {
        panic!("mismatched digest must refuse");
    };
    assert_eq!(
        response.error.category,
        OperationProtocolErrorCategoryV1::DocumentInvalid
    );
    assert_eq!(
        response.error.operation,
        Some(super::super::ProtocolOperationKindV1::DocumentMoleculeReport)
    );
}

#[test]
fn detached_protocol_report_uses_the_shared_final_envelope_budget() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";
    let observation = observation(source);
    let digest: String = observation
        .snapshot()
        .digest()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let molecule_id = observation.projection().molecules()[0]
        .id()
        .expect("direct root id")
        .as_str()
        .to_owned();
    let request = serde_json::json!({
        "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
        "request_id": "bounded-report",
        "operation": {
            "kind": "document.molecule.report.v1",
            "snapshot": {"cdml": source, "revision": 0, "digest_hex": digest},
            "molecule_ids": [molecule_id],
        }
    });
    let limit = 512;
    let envelope = execute_operation_with_runtime_and_smarts_response_limit_for_test(
        &request.to_string(),
        &CompositionRuntime,
        limit,
    )
    .expect("valid protocol JSON");
    let OperationProtocolEnvelopeV1::Error(response) = &envelope else {
        panic!("small shared response budget must refuse the report");
    };
    assert_eq!(
        response.error.category,
        OperationProtocolErrorCategoryV1::ResourceLimit
    );
    assert_eq!(response.error.message, "response_size_exceeded");
    assert!(matches!(
        response.error.resource_limit,
        Some(ProtocolResourceLimitRefusalV1 {
            reason: super::super::ProtocolResourceLimitReasonV1::ResponseSizeExceeded,
            recovery: ProtocolResourceLimitRecoveryV1::ReduceRequestedResult,
        })
    ));
    assert!(
        canonical_protocol_envelope_json_v1(&envelope)
            .expect("final envelope serializes")
            .len()
            <= limit
    );
}

"""Installed-extension behavior for the frozen operation protocol V1."""

import json

import pytest

import ferrum_chem


CDML = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C">'
	'<point x="10" y="20"/></atom></molecule></cdml>'
)


def test_protocol_execution_returns_semantic_success_and_refusal_data() -> None:
	"""One decodable request family returns machine-readable completed envelopes."""
	success = json.loads(ferrum_chem.execute_operation_v1(json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": "inspect-example",
		"operation": {"kind": "document.inspect", "document": CDML},
	})))
	refusal = json.loads(ferrum_chem.execute_operation_v1(json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": "rejected-example",
		"operation": {"kind": "document.inspect", "document": "not CDML"},
	})))

	assert success["request_id"] == "inspect-example" and (
		success["outcome"]["kind"] == "document.inspect"
		and success["outcome"]["report"]["schema"] == "ferrum-cdml-inspection-v1"
	)
	assert refusal["request_id"] == "rejected-example" and (
		refusal["error"]["category"] == "document_admission_failed"
	)


def test_protocol_malformed_json_has_the_public_exception_category() -> None:
	"""Malformed transport has no envelope, so it uses the documented exception."""
	with pytest.raises(ferrum_chem.OperationProtocolErrorV1) as error:
		ferrum_chem.execute_operation_v1("{")

	assert error.value.category == "invalid_json"


def test_molecule_report_schema_exposes_closed_aggregate_outcomes() -> None:
	"""The PyO3 JSON contract makes aggregate completion and omission exclusive."""
	schema = json.loads(ferrum_chem.operation_protocol_schema_v1())
	definitions = schema["$defs"]
	record = definitions["DocumentMoleculeReportRecordSummaryV1"]["properties"]
	report = definitions["DocumentMoleculeReportSummaryV1"]["properties"]
	aggregate = definitions["DocumentMoleculeReportAggregateOutcomeSummaryV1"]
	omission = definitions["DocumentMoleculeReportAggregateOmissionReasonSummaryV1"]
	composition = definitions["DocumentMoleculeReportCompositionSummaryV1"]["properties"]
	element = definitions["DocumentMoleculeReportCompositionElementSummaryV1"]["properties"]

	assert (
		"composition" in record
		and "composition_formula" not in record
		and "aggregate" in report
		and "combined_composition" not in report
		and "aggregate_omission_reason" not in report
		and '"complete"' in json.dumps(aggregate)
		and '"omitted"' in json.dumps(aggregate)
		and omission["enum"] == ["fewer_than_two_selected", "incomplete_record_composition"]
		and {"formula", "net_formal_charge", "average_molecular_weight_da", "monoisotopic_mass_da", "elements"}.issubset(composition)
		and {"symbol", "isotope", "atom_count", "average_mass_contribution_da", "mass_percentage"}.issubset(element)
	)


def test_smarts_query_schema_and_stateless_protocol_keep_live_state_private() -> None:
	"""The named JSON route admits raw and selected queries but returns bounded facts only."""
	schema = json.loads(ferrum_chem.operation_protocol_schema_v1())
	definitions = schema["$defs"]
	request = definitions["DocumentSmartsQueryRequestV1"]["properties"]
	summary = definitions["DocumentSmartsQuerySummaryV1"]["properties"]
	molecule = definitions["DocumentSmartsQueryMoleculeSummaryV1"]["properties"]
	resource_reason = definitions["ProtocolResourceLimitReasonV1"]
	assert {"document", "query", "limits"}.issubset(request)
	assert '"smarts"' in json.dumps(definitions["DocumentSmartsQueryInputV1"])
	assert '"selected_molecule"' in json.dumps(definitions["DocumentSmartsQueryInputV1"])
	assert set(summary) == {"schema", "traversal", "molecules"}
	assert set(molecule) == {"source_order", "match_count", "completeness"}
	assert resource_reason["oneOf"][0]["const"] == "response_size_exceeded"
	private_names = {
		"receipt",
		"paint",
		"query_origin",
		"query_display",
		"graph_position",
		"record_id",
	}
	assert not private_names.intersection(request)
	assert not private_names.intersection(summary)
	assert not private_names.intersection(molecule)

	snapshot = ferrum_chem.DocumentSession.load(CDML).snapshot()
	response = ferrum_chem.execute_operation_v1(json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": "stateless-smarts-redaction",
		"operation": {
			"kind": "document.molecule.smarts.query.v1",
			"document": {"cdml": CDML, "expected_revision": snapshot.revision, "expected_digest_hex": snapshot.digest},
			"query": {"kind": "smarts", "value": "FERRUM_SECRET_SMARTS_TEXT_91"},
			"limits": {"max_matches_per_molecule": 1, "max_total_matches": 1},
		},
	}))
	assert "FERRUM_SECRET_SMARTS_TEXT_91" not in response
	completed = json.loads(response)
	assert completed["request_id"] == "stateless-smarts-redaction"
	assert (completed.get("outcome", {}).get("kind") == "document.molecule.smarts.query.v1"
		or completed.get("error", {}).get("category") == "chemistry_unavailable")


def test_mutating_module_file_cannot_publish_a_loader_path() -> None:
	"""A direct chemistry operation keeps its PyInit runtime after import."""
	original_file = ferrum_chem.__file__
	ferrum_chem.__file__ = "/untrusted/redirected/ferrum_chem.so"
	try:
		molecule = ferrum_chem.parse_smiles("C")
	finally:
		ferrum_chem.__file__ = original_file

	assert molecule.canonical_smiles == "C"

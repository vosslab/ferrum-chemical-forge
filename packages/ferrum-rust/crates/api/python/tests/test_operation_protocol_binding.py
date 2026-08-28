"""Installed-extension behavior for the frozen operation protocol V1."""

import json

import pytest

import ferrum_chem


CDML = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C">'
	'<point x="10" y="20"/></atom></molecule></cdml>'
)

DIAGNOSTIC_CDML = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
	'<atom id="c" name="C"><point x="0" y="0"/></atom>'
	'<atom id="o" name="O"><point x="1" y="0"/></atom>'
	'<text id="text"><point x="2" y="0"/></text>'
	'<compact-group id="group" version="1" catalog-key="methyl" attachment-index="0" '
	'orientation-degrees="0"><point x="3" y="0"/></compact-group>'
	'<bond id="zero" start="c" end="o" type="n0"/>'
	'</molecule></cdml>'
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
	record_required = definitions["DocumentMoleculeReportRecordSummaryV1"]["required"]
	report = definitions["DocumentMoleculeReportSummaryV1"]["properties"]
	aggregate = definitions["DocumentMoleculeReportAggregateOutcomeSummaryV1"]
	omission = definitions["DocumentMoleculeReportAggregateOmissionReasonSummaryV1"]
	composition = definitions["DocumentMoleculeReportCompositionSummaryV1"]["properties"]
	element = definitions["DocumentMoleculeReportCompositionElementSummaryV1"]["properties"]

	assert (
		"composition" in record
		and "identifiers" in record
		and "identifiers" in record_required
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


def test_molecule_report_installed_receipt_preserves_complete_identifiers() -> None:
	"""The installed PyO3 route publishes one complete native identifier bundle."""
	session = ferrum_chem.DocumentSession.load(CDML)
	snapshot = session.snapshot()
	molecule_id = session.observe(snapshot.revision).projection.molecules[0].document_object_id
	completed = json.loads(ferrum_chem.execute_operation_v1(json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": "identifier-receipt-example",
		"operation": {
			"kind": "document.molecule.report.v1",
			"snapshot": {
				"cdml": snapshot.cdml,
				"revision": snapshot.revision,
				"digest_hex": snapshot.digest,
			},
			"molecule_ids": [molecule_id],
		},
	})))
	report = completed["outcome"]["report"]
	identifiers = report["records"][0]["identifiers"]
	after = session.snapshot()

	assert identifiers == {
		"kind": "available",
		"canonical_smiles": "C",
		"standard_inchi": "InChI=1S/CH4/h1H4",
		"standard_inchi_key": "VNWKTOKETHGBQD-UHFFFAOYSA-N",
	}
	assert report["source_revision"] == snapshot.revision and (
		report["source_digest_hex"] == snapshot.digest
		and after.revision == snapshot.revision
		and after.digest == snapshot.digest
	)


def test_molecule_report_serializes_structured_diagnostic_findings() -> None:
	"""One public selected-molecule report preserves typed source diagnostics."""
	session = ferrum_chem.DocumentSession.load(DIAGNOSTIC_CDML)
	snapshot = session.snapshot()
	molecule_id = session.observe(snapshot.revision).projection.molecules[0].document_object_id
	completed = json.loads(ferrum_chem.execute_operation_v1(json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": "diagnostic-location-example",
		"operation": {
			"kind": "document.molecule.report.v1",
			"snapshot": {
				"cdml": snapshot.cdml,
				"revision": snapshot.revision,
				"digest_hex": snapshot.digest,
			},
			"molecule_ids": [molecule_id],
		},
	})))
	assert completed["request_id"] == "diagnostic-location-example"
	assert completed["outcome"]["kind"] == "document.molecule.report.v1"
	report = completed["outcome"]["report"]
	record = report["records"][0]
	findings_by_code = {finding["code"]: finding for finding in record["findings"]}
	text_finding = findings_by_code["text_atom_present"]
	group_finding = findings_by_code["unexpanded_group_present"]
	zero_bond_finding = findings_by_code["zero_order_bond"]

	assert report["source_revision"] == snapshot.revision and report["source_digest_hex"] == snapshot.digest
	assert record["identifiers"] == {
		"kind": "unavailable", "reason": "unsupported_molecule",
	}
	assert "finding_codes" not in record
	assert text_finding["severity"] == "warning"
	assert text_finding["recovery"] == "choose_supported_representation"
	assert text_finding["location"] == {"kind": "vertex", "identifier": "text"}
	assert text_finding["detail"] is None
	assert group_finding["severity"] == "warning"
	assert group_finding["recovery"] == "materialize_compact_group"
	assert group_finding["location"] == {"kind": "vertex", "identifier": "group"}
	assert group_finding["detail"] is None
	assert zero_bond_finding["severity"] == "warning"
	assert zero_bond_finding["recovery"] == "correct_chemical_facts"
	assert zero_bond_finding["location"] == {"kind": "bond", "identifier": "zero"}
	assert zero_bond_finding["detail"] is None
	after = session.snapshot()
	assert after.revision == snapshot.revision and after.digest == snapshot.digest


def test_generic_operation_protocol_preserves_nonzero_oxidation_snapshot_provenance() -> None:
	"""A detached oxidation snapshot retains caller provenance through the generic bridge."""
	session = ferrum_chem.DocumentSession.load(CDML)
	snapshot = session.snapshot()
	projection = session.observe(snapshot.revision).projection
	molecule = projection.molecules[0]
	completed = json.loads(ferrum_chem.execute_operation_v1(json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": "nonzero-oxidation-snapshot",
		"operation": {
			"kind": "document.atom.oxidation.observe.v1",
			"document": {
				"cdml": snapshot.cdml,
				"expected_revision": 7,
				"expected_digest_hex": snapshot.digest,
			},
			"molecule_id": molecule.document_object_id,
			"atom_id": molecule.atoms[0].document_object_id,
		},
	})))

	assert completed["request_id"] == "nonzero-oxidation-snapshot"
	assert completed["outcome"]["kind"] == "document.atom.oxidation.observe.v1"
	observation = completed["outcome"]["observation"]
	assert observation["source_revision"] == 7 and observation["source_digest_hex"] == snapshot.digest
	assert observation["status"] in {"accepted", "unavailable"}


def test_generic_oxidation_protocol_refuses_malformed_and_mismatched_digests() -> None:
	"""Digest validation remains a typed operation-protocol refusal."""
	session = ferrum_chem.DocumentSession.load(CDML)
	snapshot = session.snapshot()
	molecule = session.observe(snapshot.revision).projection.molecules[0]

	def execute(expected_digest_hex: str) -> dict[str, object]:
		response = ferrum_chem.execute_operation_v1(json.dumps({
			"schema": "ferrum-operation-request-v1",
			"request_id": "invalid-oxidation-digest",
			"operation": {
				"kind": "document.atom.oxidation.observe.v1",
				"document": {
					"cdml": snapshot.cdml,
					"expected_revision": 0,
					"expected_digest_hex": expected_digest_hex,
				},
				"molecule_id": molecule.document_object_id,
				"atom_id": molecule.atoms[0].document_object_id,
			},
		}))
		return json.loads(response)

	malformed = execute("not-a-sha-256-digest")
	mismatched_digest = ("0" if snapshot.digest[0] != "0" else "1") + snapshot.digest[1:]
	mismatched = execute(mismatched_digest)

	assert malformed["error"]["category"] == "invalid_request"
	assert mismatched["error"]["category"] == "stale_document"


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
	assert set(molecule) == {"document_paint_order", "match_count", "completeness"}
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
			"document": {"cdml": snapshot.cdml, "expected_revision": snapshot.revision, "expected_digest_hex": snapshot.digest},
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

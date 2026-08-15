"""Installed-extension behavior for the frozen operation protocol V1."""

import json

import pytest

import ferrum_chem


CDML = (
	'<cdml><molecule id="m"><atom id="a" name="C">'
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

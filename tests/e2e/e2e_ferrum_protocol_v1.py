"""Exercise the frozen Ferrum operation protocol through the built CLI."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys


CDML = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C">'
	'<point x="10" y="20"/></atom></molecule></cdml>'
)


class ProtocolE2eError(RuntimeError):
	"""Raised when the public protocol CLI breaks its completed-response contract."""


def run(
		ferrum: Path, *arguments: str, input_text: str = "",
		) -> subprocess.CompletedProcess[str]:
	"""Run the built public executable with one exact standard-input request."""
	return subprocess.run(
		[str(ferrum), *arguments], input=input_text, text=True,
		stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
	)


def one_json_object(output: str, label: str) -> dict[str, object]:
	"""Decode the CLI's one-record stdout contract without preserving JSON bytes."""
	lines = output.splitlines()
	if len(lines) != 1:
		raise ProtocolE2eError(f"{label} must emit one JSON object")
	try:
		value = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise ProtocolE2eError(f"{label} emitted invalid JSON: {error.msg}") from error
	if not isinstance(value, dict):
		raise ProtocolE2eError(f"{label} must emit a JSON object")
	return value


def request(request_id: str, document: str) -> str:
	"""Build one ordinary inspect request with caller-chosen opaque identity."""
	return json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": request_id,
		"operation": {"kind": "document.inspect", "document": document},
	})


def check_schema(ferrum: Path) -> None:
	"""Require a parseable generated schema with all protocol response roots."""
	result = run(ferrum, "protocol", "schema")
	if result.returncode != 0 or result.stderr:
		raise ProtocolE2eError("protocol schema did not complete cleanly")
	schema = one_json_object(result.stdout, "protocol schema")
	roots = schema.get("x-ferrum-roots")
	if not isinstance(roots, dict) or not {
		"request", "success_response", "error_response",
	}.issubset(roots):
		raise ProtocolE2eError("protocol schema lacks its declared request and response roots")


def check_completed_response(
		ferrum: Path, request_id: str, document: str, expected_schema: str,
		expected_exit_status: int,
		) -> dict[str, object]:
	"""Run a decodable request and enforce its stdout/stderr/exit-channel rules."""
	result = run(ferrum, "protocol", "run", "-", input_text=request(request_id, document))
	if result.returncode != expected_exit_status or result.stderr:
		raise ProtocolE2eError(
			"completed protocol request did not keep its expected exit and diagnostics channels"
		)
	response = one_json_object(result.stdout, "protocol run")
	if response.get("schema") != expected_schema or response.get("request_id") != request_id:
		raise ProtocolE2eError("protocol response lost its declared envelope identity")
	return response


def nested_object(response: dict[str, object], name: str) -> dict[str, object]:
	"""Return a required object-valued response member."""
	value = response.get(name)
	if not isinstance(value, dict):
		raise ProtocolE2eError(f"protocol response lacks object {name}")
	return value


def main() -> int:
	"""Run the public V1 schema, success, and typed-refusal workflow."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--ferrum", type=Path, required=True)
	arguments = parser.parse_args()
	ferrum = arguments.ferrum.resolve()
	if not ferrum.is_file():
		raise ProtocolE2eError("--ferrum must name an existing executable")
	check_schema(ferrum)
	success = check_completed_response(
		ferrum, "e2e-success", CDML, "ferrum-operation-response-v1", 0,
	)
	refusal = check_completed_response(
		ferrum, "e2e-refusal", "not CDML", "ferrum-operation-error-v1", 1,
	)
	if nested_object(success, "outcome").get("kind") != "document.inspect":
		raise ProtocolE2eError("protocol inspect success omitted its semantic outcome")
	if nested_object(refusal, "error").get("category") != "document_admission_failed":
		raise ProtocolE2eError("protocol refusal omitted its typed admission category")
	print(json.dumps({"schema": "ferrum-protocol-e2e-v1", "status": "ok"}, sort_keys=True))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except ProtocolE2eError as error:
		print(f"protocol E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)

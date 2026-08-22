"""Exercise the stateless CLI presentation-vector transaction end to end."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess

import ferrum_chem


CDML = '<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C"><point x="10" y="20"/></atom></molecule></cdml>'
EXCLUDED = '<cdml xmlns="urn:ferrum:cdml"><text id="bad"><point x="1" y="2"/><ftext><b>x</b></ftext></text></cdml>'


def request(document: str, revision: int, end_x: float, end_y: float) -> str:
	"""Create one complete stateless vector command request."""
	digest = ferrum_chem.DocumentSession.load(document).snapshot().digest
	return json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": "vector-cli-e2e",
		"operation": {
			"kind": "presentation.vector.create.v1",
			"document": document,
			"expected_revision": revision,
			"expected_digest_hex": digest,
			"vector_kind": "rectangle",
			"start_x": 10.0,
			"start_y": 20.0,
			"end_x": end_x,
			"end_y": end_y,
			"appearance_policy": "effective_drawing_standard",
		},
	})


def invoke(ferrum: Path, payload: str) -> dict[str, object]:
	"""Run the named CLI command and require one clean JSON response."""
	result = subprocess.run(
		[ferrum, "document", "command", "presentation.vector.create.v1", "-"],
		input=payload, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
	)
	if result.returncode != 0 or result.stderr:
		raise RuntimeError(f"vector CLI failed: {result.returncode}: {result.stderr.strip()}")
	response = json.loads(result.stdout)
	if not isinstance(response, dict):
		raise RuntimeError("vector CLI response was not a JSON object")
	return response


def refusal(response: dict[str, object], category: str, recovery: str) -> None:
	"""Require the closed refusal contract instead of a diagnostic-only failure."""
	error = response.get("error")
	if not isinstance(error, dict):
		raise RuntimeError("expected error envelope")
	vector = error.get("presentation_vector_refusal")
	if not isinstance(vector, dict) or vector.get("category") != category or vector.get("recovery") != recovery:
		raise RuntimeError(f"unexpected vector refusal: {response}")


def main() -> None:
	"""Verify success, sequential stateless use, typed refusal, and render preflight refusal."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--ferrum", required=True, type=Path)
	args = parser.parse_args()
	first = invoke(args.ferrum, request(CDML, 0, 40.0, 60.0))
	outcome = first.get("outcome")
	if not isinstance(outcome, dict):
		raise RuntimeError("expected vector success outcome")
	if outcome.get("input_revision") != 0 or outcome.get("committed_revision") != 1 or outcome.get("next_input_expected_revision") != 0:
		raise RuntimeError(f"unexpected stateless revision contract: {outcome}")
	if not isinstance(outcome.get("renderer_observation"), dict):
		raise RuntimeError("missing immutable renderer observation")
	document = outcome.get("document")
	if not isinstance(document, str):
		raise RuntimeError("missing result CDML")
	second = invoke(args.ferrum, request(document, 0, 70.0, 90.0))
	if not isinstance(second.get("outcome"), dict):
		raise RuntimeError("returned CDML was not chainable as fresh stateless input")
	refusal(invoke(args.ferrum, request(CDML, 0, 10.0, 20.0)), "degenerate_geometry", "change_geometry")
	refusal(invoke(args.ferrum, request(EXCLUDED, 0, 40.0, 60.0)), "render_preparation", "document_unchanged")
	print(json.dumps({"schema": "ferrum-presentation-vector-cli-e2e-v1", "status": "ok"}))


if __name__ == "__main__":
	main()

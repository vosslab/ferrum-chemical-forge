"""Exercise public molecule reporting through generic and named CLI transports."""

from __future__ import annotations

# Standard Library
import argparse
import json
from pathlib import Path
import subprocess
import sys


MOLECULE_ID = "ferrum-document-object-v1/00000000000000000000000000000031"
ATOM_ID = "ferrum-document-object-v1/00000000000000000000000000000032"
CDML = (
	'<cdml xmlns="urn:ferrum:cdml" xmlns:object="urn:ferrum:document-object:v1">'
	f'<molecule id="source" object:id="{MOLECULE_ID}">'
	f'<atom id="carbon" object:id="{ATOM_ID}" name="C"><point x="0" y="0"/></atom>'
	'</molecule></cdml>'
)


class DocumentMoleculeReportCliE2eError(RuntimeError):
	"""Raised when the public molecule-report CLI contract breaks."""


#============================================
def parse_arguments() -> argparse.Namespace:
	"""Parse the public CLI E2E arguments."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--ferrum", type=Path, required=True)
	return parser.parse_args()


#============================================
def run(
	ferrum: Path,
	*arguments: str,
	input_text: str,
) -> subprocess.CompletedProcess[str]:
	"""Run one public CLI transport with one exact serialized request."""
	return subprocess.run(
		[str(ferrum), *arguments],
		input=input_text,
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
		check=False,
	)


#============================================
def one_envelope(result: subprocess.CompletedProcess[str], label: str) -> dict[str, object]:
	"""Decode the sole standard-output protocol envelope without retaining JSON bytes."""
	if result.stderr:
		raise DocumentMoleculeReportCliE2eError(
			f"{label} wrote a diagnostic: {result.stderr.strip()}"
		)
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise DocumentMoleculeReportCliE2eError(f"{label} did not emit one protocol envelope")
	try:
		value = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise DocumentMoleculeReportCliE2eError(
			f"{label} emitted invalid JSON: {error.msg}"
		) from error
	if not isinstance(value, dict):
		raise DocumentMoleculeReportCliE2eError(f"{label} did not emit a JSON object")
	return value


#============================================
def request(request_id: str, operation: dict[str, object]) -> str:
	"""Build one ordinary frozen V1 operation request."""
	return json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": request_id,
		"operation": operation,
	})


#============================================
def document_fence(ferrum: Path) -> dict[str, object]:
	"""Authenticate the inline document and return its public immutable fence."""
	result = run(
		ferrum,
		"protocol",
		"run",
		"-",
		input_text=request("document-molecule-report-inspect", {
			"kind": "document.inspect",
			"document": CDML,
		}),
	)
	if result.returncode != 0:
		raise DocumentMoleculeReportCliE2eError("document inspection did not complete")
	envelope = one_envelope(result, "document inspection")
	outcome = envelope.get("outcome")
	if not isinstance(outcome, dict) or outcome.get("kind") != "document.inspect":
		raise DocumentMoleculeReportCliE2eError("document inspection omitted its outcome")
	fence = outcome.get("document_fence")
	if (
		not isinstance(fence, dict)
		or not isinstance(fence.get("expected_revision"), int)
		or not isinstance(fence.get("expected_digest_hex"), str)
		or not fence["expected_digest_hex"]
	):
		raise DocumentMoleculeReportCliE2eError("document inspection omitted a usable fence")
	return fence


#============================================
def report_operation(fence: dict[str, object], digest_hex: str) -> dict[str, object]:
	"""Build the one fenced single-molecule report request from inline CDML."""
	return {
		"kind": "document.molecule.report.v1",
		"snapshot": {
			"cdml": CDML,
			"revision": fence["expected_revision"],
			"digest_hex": digest_hex,
		},
		"molecule_ids": [MOLECULE_ID],
	}


#============================================
def report_outcome(
	envelope: dict[str, object],
	request_id: str,
	fence: dict[str, object],
) -> None:
	"""Require stable selected-root and composition facts from one successful report."""
	if (
		envelope.get("schema") != "ferrum-operation-response-v1"
		or envelope.get("request_id") != request_id
	):
		raise DocumentMoleculeReportCliE2eError("report lost its public envelope identity")
	outcome = envelope.get("outcome")
	if not isinstance(outcome, dict) or outcome.get("kind") != "document.molecule.report.v1":
		raise DocumentMoleculeReportCliE2eError("report omitted its successful outcome")
	report = outcome.get("report")
	if (
		not isinstance(report, dict)
		or report.get("schema") != "ferrum-document-molecule-report-v1"
		or report.get("source_revision") != fence["expected_revision"]
		or report.get("source_digest_hex") != fence["expected_digest_hex"]
	):
		raise DocumentMoleculeReportCliE2eError("report omitted its frozen source facts")
	records = report.get("records")
	if not isinstance(records, list):
		raise DocumentMoleculeReportCliE2eError("report omitted selected molecule records")
	for record in records:
		if isinstance(record, dict) and record.get("molecule_id") == MOLECULE_ID:
			composition = record.get("composition")
			if isinstance(composition, dict) and composition.get("formula") == "CH4":
				return
	raise DocumentMoleculeReportCliE2eError(
		"report omitted the selected durable molecule's methane formula"
	)


#============================================
def changed_digest(digest_hex: str) -> str:
	"""Return another valid digest-shaped value without trusting a generated digest."""
	if not digest_hex:
		raise DocumentMoleculeReportCliE2eError("inspection returned an empty digest")
	return ("0" if digest_hex[-1] != "0" else "1") + digest_hex[1:]


#============================================
def refusal_envelope(envelope: dict[str, object], request_id: str) -> None:
	"""Require the established completed frozen-snapshot digest refusal surface."""
	error = envelope.get("error")
	if (
		envelope.get("schema") != "ferrum-operation-error-v1"
		or envelope.get("request_id") != request_id
		or "outcome" in envelope
		or not isinstance(error, dict)
		or error.get("operation") != "document.molecule.report.v1"
		or error.get("category") != "document_invalid"
	):
		raise DocumentMoleculeReportCliE2eError(
			"digest mismatch did not retain the typed frozen-snapshot refusal"
		)


#============================================
def run_scenario(ferrum: Path) -> None:
	"""Exercise equivalent generic and named report responses and digest refusals."""
	fence = document_fence(ferrum)
	success_request_id = "document-molecule-report-success"
	success = request(
		success_request_id,
		report_operation(fence, fence["expected_digest_hex"]),
	)
	generic = run(ferrum, "protocol", "run", "-", input_text=success)
	named = run(
		ferrum,
		"document",
		"command",
		"document.molecule.report.v1",
		"-",
		input_text=success,
	)
	if generic.returncode != 0 or named.returncode != 0:
		raise DocumentMoleculeReportCliE2eError("successful report did not complete")
	generic_envelope = one_envelope(generic, "generic report")
	named_envelope = one_envelope(named, "named report")
	report_outcome(generic_envelope, success_request_id, fence)
	report_outcome(named_envelope, success_request_id, fence)
	if generic_envelope != named_envelope:
		raise DocumentMoleculeReportCliE2eError("report transports emitted different decoded envelopes")

	refusal_request_id = "document-molecule-report-digest-mismatch"
	refusal = request(
		refusal_request_id,
		report_operation(fence, changed_digest(fence["expected_digest_hex"])),
	)
	generic = run(ferrum, "protocol", "run", "-", input_text=refusal)
	named = run(
		ferrum,
		"document",
		"command",
		"document.molecule.report.v1",
		"-",
		input_text=refusal,
	)
	if generic.returncode != 0 or named.returncode != 0:
		raise DocumentMoleculeReportCliE2eError("digest-mismatch refusal did not complete")
	generic_envelope = one_envelope(generic, "generic digest refusal")
	named_envelope = one_envelope(named, "named digest refusal")
	refusal_envelope(generic_envelope, refusal_request_id)
	refusal_envelope(named_envelope, refusal_request_id)
	if generic_envelope != named_envelope:
		raise DocumentMoleculeReportCliE2eError("refusal transports emitted different decoded envelopes")


#============================================
def main() -> int:
	"""Run the public molecule-report CLI E2E."""
	arguments = parse_arguments()
	ferrum = arguments.ferrum.resolve()
	if not ferrum.is_file():
		raise DocumentMoleculeReportCliE2eError("--ferrum must name an existing executable")
	run_scenario(ferrum)
	print(json.dumps({
		"schema": "ferrum-document-molecule-report-cli-e2e-v1",
		"status": "ok",
	}, sort_keys=True))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except DocumentMoleculeReportCliE2eError as error:
		print(f"document molecule report CLI E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)

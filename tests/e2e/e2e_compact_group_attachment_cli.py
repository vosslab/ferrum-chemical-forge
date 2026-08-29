"""Exercise public fenced compact-group attachment through both CLI transports."""

from __future__ import annotations

# Standard Library
import sys
import json
import argparse
from pathlib import Path
import subprocess


MOLECULE_ID = "ferrum-document-object-v1/00000000000000000000000000000021"
ANCHOR_ATOM_ID = "ferrum-document-object-v1/00000000000000000000000000000022"
CDML = (
	'<cdml xmlns="urn:ferrum:cdml" xmlns:object="urn:ferrum:document-object:v1">'
	f'<molecule id="source" object:id="{MOLECULE_ID}">'
	f'<atom id="anchor" object:id="{ANCHOR_ATOM_ID}" name="C">'
	'<point x="0" y="0"/></atom></molecule></cdml>'
)


class CompactGroupAttachmentCliE2eError(RuntimeError):
	"""Raised when the public compact-group attachment CLI contract breaks."""


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
	"""Run one public CLI transport with an exact protocol envelope."""
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
	"""Decode the CLI's sole standard-output protocol envelope."""
	if result.stderr:
		raise CompactGroupAttachmentCliE2eError(
			f"{label} wrote a second diagnostic: {result.stderr.strip()}"
		)
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise CompactGroupAttachmentCliE2eError(f"{label} did not emit one protocol envelope")
	try:
		value = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise CompactGroupAttachmentCliE2eError(
			f"{label} emitted invalid JSON: {error.msg}"
		) from error
	if not isinstance(value, dict):
		raise CompactGroupAttachmentCliE2eError(f"{label} did not emit a JSON object")
	return value


#============================================
def request(request_id: str, operation: dict[str, object]) -> str:
	"""Build one ordinary frozen V1 protocol request."""
	return json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": request_id,
		"operation": operation,
	})


#============================================
def document_fence(ferrum: Path, document: str, request_id: str) -> dict[str, object]:
	"""Obtain a reusable mutation fence through public document inspection."""
	result = run(
		ferrum,
		"protocol",
		"run",
		"-",
		input_text=request(request_id, {"kind": "document.inspect", "document": document}),
	)
	if result.returncode != 0:
		raise CompactGroupAttachmentCliE2eError("document inspection did not complete")
	envelope = one_envelope(result, "document inspection")
	outcome = envelope.get("outcome")
	if not isinstance(outcome, dict) or outcome.get("kind") != "document.inspect":
		raise CompactGroupAttachmentCliE2eError("document inspection omitted its outcome")
	fence = outcome.get("document_fence")
	if (
		not isinstance(fence, dict)
		or not isinstance(fence.get("expected_revision"), int)
		or not isinstance(fence.get("expected_digest_hex"), str)
		or not fence["expected_digest_hex"]
	):
		raise CompactGroupAttachmentCliE2eError("document inspection omitted a usable fence")
	return fence


#============================================
def attachment_operation(
	document: str,
	fence: dict[str, object],
	catalog_key: str,
) -> dict[str, object]:
	"""Build one fenced public attachment request using the inline source selectors."""
	return {
		"kind": "document.compact-group.attach.v1",
		"document": {
			"cdml": document,
			"expected_revision": fence["expected_revision"],
			"expected_digest_hex": fence["expected_digest_hex"],
		},
		"molecule_id": MOLECULE_ID,
		"anchor_atom_id": ANCHOR_ATOM_ID,
		"catalog_key": catalog_key,
		"release": {"x": 40.0, "y": 0.0},
	}


#============================================
def committed_attachment(
	envelope: dict[str, object],
	request_id: str,
	source_fence: dict[str, object],
	catalog_key: str,
) -> dict[str, object]:
	"""Require one committed receipt with source selectors and a next fence."""
	if (
		envelope.get("schema") != "ferrum-operation-response-v1"
		or envelope.get("request_id") != request_id
	):
		raise CompactGroupAttachmentCliE2eError("attachment lost its public envelope identity")
	outcome = envelope.get("outcome")
	if not isinstance(outcome, dict) or outcome.get("kind") != "document.compact-group.attach.v1":
		raise CompactGroupAttachmentCliE2eError("attachment omitted its successful outcome")
	receipt = outcome.get("attachment")
	if not isinstance(receipt, dict):
		raise CompactGroupAttachmentCliE2eError("attachment omitted its committed receipt")
	if (
		receipt.get("schema") != "ferrum-document-compact-group-attachment-v1"
		or receipt.get("source_revision") != source_fence["expected_revision"]
		or receipt.get("source_digest_hex") != source_fence["expected_digest_hex"]
		or receipt.get("molecule_id") != MOLECULE_ID
		or receipt.get("anchor_atom_id") != ANCHOR_ATOM_ID
		or receipt.get("catalog_key") != catalog_key
	):
		raise CompactGroupAttachmentCliE2eError("attachment receipt lost its source facts")
	compact_group_id = receipt.get("compact_group_id")
	document = receipt.get("document")
	next_fence = receipt.get("document_fence")
	if (
		not isinstance(compact_group_id, str)
		or not compact_group_id
		or compact_group_id in {MOLECULE_ID, ANCHOR_ATOM_ID}
		or not isinstance(document, str)
		or "<cdml" not in document
		or not isinstance(next_fence, dict)
		or next_fence == source_fence
	):
		raise CompactGroupAttachmentCliE2eError("attachment receipt omitted committed public facts")
	return receipt


#============================================
def attachment_document(receipt: dict[str, object]) -> str:
	"""Return the already-validated committed document from an attachment receipt."""
	document = receipt["document"]
	if not isinstance(document, str):
		raise CompactGroupAttachmentCliE2eError("attachment receipt omitted its committed document")
	return document


#============================================
def attachment_fence(receipt: dict[str, object]) -> dict[str, object]:
	"""Return the already-validated next fence from an attachment receipt."""
	fence = receipt["document_fence"]
	if not isinstance(fence, dict):
		raise CompactGroupAttachmentCliE2eError("attachment receipt omitted its next fence")
	return fence


#============================================
def assert_authenticated_attachment_document(
	ferrum: Path,
	receipt: dict[str, object],
	request_id: str,
	label: str,
) -> None:
	"""Require public inspection to authenticate an attachment's committed document."""
	document = attachment_document(receipt)
	expected_fence = attachment_fence(receipt)
	actual_fence = document_fence(ferrum, document, request_id)
	if actual_fence != expected_fence:
		raise CompactGroupAttachmentCliE2eError(
			f"{label} attachment fence did not authenticate CDML"
		)


#============================================
def assert_typed_stale_refusal(
	ferrum: Path,
	document: str,
	fence: dict[str, object],
	arguments: tuple[str, ...],
	request_id: str,
	label: str,
) -> None:
	"""Require one completed typed stale-fence refusal without a second diagnostic."""
	stale_fence = dict(fence)
	stale_fence["expected_revision"] = fence["expected_revision"] + 1
	result = run(
		ferrum,
		*arguments,
		input_text=request(
			request_id,
			attachment_operation(document, stale_fence, "methyl"),
		),
	)
	if result.returncode != 1:
		raise CompactGroupAttachmentCliE2eError(
			f"{label} stale attachment refusal did not return the typed-failure exit status"
		)
	envelope = one_envelope(result, f"{label} stale attachment refusal")
	error = envelope.get("error")
	if (
		envelope.get("schema") != "ferrum-operation-error-v1"
		or envelope.get("request_id") != request_id
		or "outcome" in envelope
		or not isinstance(error, dict)
		or not isinstance(error.get("compact_group_attachment_refusal"), dict)
	):
		raise CompactGroupAttachmentCliE2eError(
			f"{label} stale attachment lacked a typed error envelope"
		)
	refusal = error["compact_group_attachment_refusal"]
	if (
		refusal.get("category") != "stale_document_fence"
		or refusal.get("recovery") != "refresh_and_retry"
	):
		raise CompactGroupAttachmentCliE2eError(
			f"{label} stale attachment omitted typed recovery facts"
		)


#============================================
def generic_attachment(
	ferrum: Path,
	document: str,
	fence: dict[str, object],
) -> dict[str, object]:
	"""Exercise successful generic transport attachment."""
	request_id = "compact-group-attachment-generic"
	result = run(
		ferrum,
		"protocol",
		"run",
		"-",
		input_text=request(
			request_id,
			attachment_operation(document, fence, "methyl"),
		),
	)
	if result.returncode != 0:
		raise CompactGroupAttachmentCliE2eError("generic attachment did not complete")
	return committed_attachment(
		one_envelope(result, "generic attachment"),
		request_id,
		fence,
		"methyl",
	)


#============================================
def named_attachment(
	ferrum: Path,
	document: str,
	fence: dict[str, object],
) -> dict[str, object]:
	"""Exercise successful named-command attachment."""
	request_id = "compact-group-attachment-named"
	result = run(
		ferrum,
		"document",
		"command",
		"document.compact-group.attach.v1",
		"-",
		input_text=request(
			request_id,
			attachment_operation(document, fence, "methyl"),
		),
	)
	if result.returncode != 0:
		raise CompactGroupAttachmentCliE2eError("named attachment did not complete")
	return committed_attachment(
		one_envelope(result, "named attachment"),
		request_id,
		fence,
		"methyl",
	)


#============================================
def run_scenario(ferrum: Path) -> None:
	"""Exercise both public transports for success and typed stale refusal."""
	document = CDML
	fence = document_fence(ferrum, document, "compact-group-attachment-inspect")
	generic = generic_attachment(ferrum, document, fence)
	assert_authenticated_attachment_document(
		ferrum,
		generic,
		"generic-attachment-inspect",
		"generic",
	)
	named = named_attachment(ferrum, document, fence)
	assert_authenticated_attachment_document(
		ferrum,
		named,
		"named-attachment-inspect",
		"named",
	)
	assert_typed_stale_refusal(
		ferrum,
		document,
		fence,
		("protocol", "run", "-"),
		"compact-group-attachment-generic-stale",
		"generic",
	)
	assert_typed_stale_refusal(
		ferrum,
		document,
		fence,
		("document", "command", "document.compact-group.attach.v1", "-"),
		"compact-group-attachment-named-stale",
		"named",
	)


#============================================
def main() -> int:
	"""Run the public compact-group attachment CLI E2E."""
	arguments = parse_arguments()
	ferrum = arguments.ferrum.resolve()
	if not ferrum.is_file():
		raise CompactGroupAttachmentCliE2eError("--ferrum must name an existing executable")
	run_scenario(ferrum)
	print(json.dumps({
		"schema": "ferrum-compact-group-attachment-cli-e2e-v1",
		"status": "ok",
	}, sort_keys=True))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except CompactGroupAttachmentCliE2eError as error:
		print(f"compact-group attachment CLI E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)

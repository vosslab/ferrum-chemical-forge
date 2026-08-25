"""Exercise one fenced compact-group materialization and its committed follow-up."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys


COMPACT_CDML = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="source-molecule">'
	'<atom id="anchor" name="C"><point x="0" y="0"/></atom>'
	'<compact-group id="source-group" version="1" catalog-key="methyl" '
	'attachment-index="0" orientation-degrees="0"><point x="20" y="0"/></compact-group>'
	'<bond id="outside" start="anchor" end="source-group" type="n1"/>'
	'</molecule></cdml>'
)


class CompactGroupCliE2eError(RuntimeError):
	"""Raised when the public compact-group CLI contract is broken."""


def run(ferrum: Path, *arguments: str, input_text: str) -> dict[str, object]:
	"""Run one public CLI route and decode its sole protocol envelope."""
	result = subprocess.run(
		[str(ferrum), *arguments], input=input_text, text=True,
		stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
	)
	if result.returncode != 0 or result.stderr:
		raise CompactGroupCliE2eError("compact CLI route did not complete cleanly")
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise CompactGroupCliE2eError("compact CLI route did not emit one JSON object")
	value = json.loads(lines[0])
	if not isinstance(value, dict):
		raise CompactGroupCliE2eError("compact CLI route did not emit a JSON object")
	return value


def protocol_request(request_id: str, operation: dict[str, object]) -> str:
	"""Build one exact protocol envelope."""
	return json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": request_id,
		"operation": operation,
	})


def inspect_fence(ferrum: Path, document: str) -> dict[str, object]:
	"""Obtain the normal fenced document snapshot through the public protocol."""
	response = run(ferrum, "protocol", "run", "-", input_text=protocol_request(
		"compact-inspect", {"kind": "document.inspect", "document": document},
	))
	try:
		outcome = response["outcome"]
		fence = outcome["document_fence"]
	except (KeyError, TypeError) as error:
		raise CompactGroupCliE2eError("inspection omitted a usable document fence") from error
	if not isinstance(outcome, dict) or outcome.get("kind") != "document.inspect":
		raise CompactGroupCliE2eError("inspection omitted its exact operation kind")
	if not isinstance(fence, dict):
		raise CompactGroupCliE2eError("inspection fence was not an object")
	return fence


def materialize_operation(fence: dict[str, object]) -> dict[str, object]:
	"""Build the closed materialization request with opaque durable selectors."""
	return {
		"kind": "document.compact-group.materialize.v1",
		"document": {
			"cdml": COMPACT_CDML,
			"expected_revision": fence["expected_revision"],
			"expected_digest_hex": fence["expected_digest_hex"],
		},
		"molecule_id": "source-molecule",
		"compact_group_id": "source-group",
	}


def receipt(response: dict[str, object]) -> dict[str, object]:
	"""Extract one materialization receipt with a reusable committed snapshot."""
	try:
		outcome = response["outcome"]
		value = outcome["materialization"]
	except (KeyError, TypeError) as error:
		raise CompactGroupCliE2eError("materialization omitted its committed receipt") from error
	if not isinstance(outcome, dict) or outcome.get("kind") != "document.compact-group.materialize.v1":
		raise CompactGroupCliE2eError("materialization omitted its exact operation kind")
	if not isinstance(value, dict):
		raise CompactGroupCliE2eError("materialization receipt was not an object")
	if (
		not isinstance(value.get("molecule_id"), str)
		or not value["molecule_id"]
		or not isinstance(value.get("compact_group_id"), str)
		or not value["compact_group_id"]
		or not isinstance(value.get("replacement_focus_atom_id"), str)
		or not value["replacement_focus_atom_id"]
		or not isinstance(value.get("document"), str)
	):
		raise CompactGroupCliE2eError("materialization receipt omitted committed public facts")
	fence = value.get("document_fence")
	if (
		not isinstance(fence, dict)
		or not isinstance(fence.get("expected_revision"), int)
		or not isinstance(fence.get("expected_digest_hex"), str)
		or not fence["expected_digest_hex"]
	):
		raise CompactGroupCliE2eError("materialization receipt omitted its reusable document fence")
	return value


def main() -> int:
	"""Prove generic materialization returns a usable committed next state."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--ferrum", type=Path, required=True)
	arguments = parser.parse_args()
	ferrum = arguments.ferrum.resolve()
	if not ferrum.is_file():
		raise CompactGroupCliE2eError("--ferrum must name an existing executable")
	fence = inspect_fence(ferrum, COMPACT_CDML)
	request = protocol_request("compact-materialize", materialize_operation(fence))
	materialization = receipt(run(ferrum, "protocol", "run", "-", input_text=request))
	follow_on_fence = inspect_fence(ferrum, materialization["document"])
	if follow_on_fence != materialization["document_fence"]:
		raise CompactGroupCliE2eError(
			"materialization receipt fence did not authenticate its committed document"
		)
	print(json.dumps({"schema": "ferrum-compact-group-cli-e2e-v1", "status": "ok"}, sort_keys=True))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except CompactGroupCliE2eError as error:
		print(f"compact-group CLI E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)

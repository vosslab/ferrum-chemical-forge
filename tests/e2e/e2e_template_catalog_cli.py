"""Exercise one public catalog list-to-insert workflow through the staged CLI."""

from __future__ import annotations

import argparse
import defusedxml.ElementTree
import json
from pathlib import Path
import subprocess
import sys


EMPTY_CDML = '<cdml xmlns="urn:ferrum:cdml"/>'


class CatalogCliE2eError(RuntimeError):
	"""Raised when the public catalog CLI workflow loses a required contract."""


def invoke(ferrum: Path, request_id: str, operation: dict[str, object]) -> dict[str, object]:
	"""Run one operation-protocol request and return its completed JSON response."""
	payload = json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": request_id,
		"operation": operation,
	})
	result = subprocess.run(
		[str(ferrum), "protocol", "run", "-"], input=payload, text=True,
		stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
	)
	if result.returncode != 0 or result.stderr:
		raise CatalogCliE2eError("protocol request did not complete cleanly")
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise CatalogCliE2eError("protocol request did not emit one JSON response")
	try:
		response = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise CatalogCliE2eError(f"protocol request emitted invalid JSON: {error.msg}") from error
	if not isinstance(response, dict) or response.get("request_id") != request_id:
		raise CatalogCliE2eError("protocol response lost its caller identity")
	return response


def outcome(response: dict[str, object], kind: str) -> dict[str, object]:
	"""Return one successful outcome with its expected operation kind."""
	value = response.get("outcome")
	if not isinstance(value, dict) or value.get("kind") != kind:
		raise CatalogCliE2eError(f"protocol response omitted successful {kind} outcome")
	return value


def document_fence(document: str, ferrum: Path, request_id: str) -> dict[str, object]:
	"""Inspect request-owned CDML and return the public mutation fence."""
	inspection = outcome(invoke(
		ferrum, request_id, {"kind": "document.inspect", "document": document},
	), "document.inspect")
	fence = inspection.get("document_fence")
	if (
		not isinstance(fence, dict)
		or not isinstance(fence.get("expected_revision"), int)
		or not isinstance(fence.get("expected_digest_hex"), str)
	):
		raise CatalogCliE2eError("document inspection omitted a usable mutation fence")
	return fence


def public_catalog_summary(entry: object) -> tuple[str, str, str]:
	"""Return the public ID, family, and category ID from one catalog summary."""
	if not isinstance(entry, dict):
		raise CatalogCliE2eError("catalog listing exposed a non-object public entry")
	identifier = entry.get("id")
	family = entry.get("family")
	category = entry.get("category")
	if (
		not isinstance(identifier, str)
		or not identifier
		or not isinstance(family, str)
		or not family
		or not isinstance(category, dict)
		or not isinstance(category.get("id"), str)
		or not category["id"]
	):
		raise CatalogCliE2eError("catalog listing omitted usable public summary facts")
	if "document" in entry or "template_cdml" in entry or "recipe" in entry:
		raise CatalogCliE2eError("catalog summary exposed implementation payload")
	return identifier, family, category["id"]


def selected_catalog_id(ferrum: Path) -> str:
	"""Choose one filtered public catalog summary without freezing its inventory."""
	listing = outcome(invoke(ferrum, "catalog-list", {"kind": "catalog.list.v1"}), "catalog.list.v1")
	entries = listing.get("entries")
	if not isinstance(entries, list):
		raise CatalogCliE2eError("catalog listing omitted its public entries")
	for entry in entries:
		try:
			identifier, family, category_id = public_catalog_summary(entry)
		except CatalogCliE2eError:
			continue
		filtered = outcome(invoke(ferrum, "catalog-list-filtered", {
			"kind": "catalog.list.v1", "family": family, "category": category_id,
		}), "catalog.list.v1")
		filtered_entries = filtered.get("entries")
		if not isinstance(filtered_entries, list):
			raise CatalogCliE2eError("filtered catalog listing omitted its public entries")
		selected_retained = False
		for filtered_entry in filtered_entries:
			filtered_id, filtered_family, filtered_category_id = public_catalog_summary(filtered_entry)
			if filtered_family != family or filtered_category_id != category_id:
				raise CatalogCliE2eError("filtered catalog listing returned an out-of-filter summary")
			if filtered_id == identifier:
				selected_retained = True
		if not selected_retained:
			raise CatalogCliE2eError("filtered catalog listing lost its selected public summary")
		return identifier
	raise CatalogCliE2eError("catalog listing provided no selectable public entry")


def insertion_operation(document: str, fence: dict[str, object], catalog_id: str) -> dict[str, object]:
	"""Build one fence-carrying catalog insertion operation."""
	return {
		"kind": "catalog.insert.v1",
		"document": document,
		"expected_revision": fence["expected_revision"],
		"expected_digest_hex": fence["expected_digest_hex"],
		"catalog_id": catalog_id,
		"anchor_x": 100.0,
		"anchor_y": 50.0,
	}


def main() -> int:
	"""Verify public catalog selection, fenced insertion, and stale-fence refusal."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--ferrum", type=Path, required=True)
	arguments = parser.parse_args()
	ferrum = arguments.ferrum.resolve()
	if not ferrum.is_file():
		raise CatalogCliE2eError("--ferrum must name an existing executable")
	initial_fence = document_fence(EMPTY_CDML, ferrum, "catalog-inspect-initial")
	catalog_id = selected_catalog_id(ferrum)
	inserted = outcome(invoke(
		ferrum, "catalog-insert", insertion_operation(EMPTY_CDML, initial_fence, catalog_id),
	), "catalog.insert.v1")
	created_id = inserted.get("identifier")
	document = inserted.get("document")
	if not isinstance(created_id, str) or not created_id or not isinstance(document, str):
		raise CatalogCliE2eError("catalog insertion omitted its public created ID or document")
	try:
		defusedxml.ElementTree.fromstring(document)
	except defusedxml.ElementTree.ParseError as error:
		raise CatalogCliE2eError(f"catalog insertion returned invalid CDML: {error}") from error
	follow_on_fence = inserted.get("document_fence")
	if (
		not isinstance(follow_on_fence, dict)
		or not isinstance(follow_on_fence.get("expected_revision"), int)
		or not isinstance(follow_on_fence.get("expected_digest_hex"), str)
		or follow_on_fence == initial_fence
	):
		raise CatalogCliE2eError("catalog insertion did not return a usable changed document fence")
	if document_fence(document, ferrum, "catalog-inspect-follow-on") != follow_on_fence:
		raise CatalogCliE2eError("catalog insertion fence disagreed with fresh document inspection")
	stale = insertion_operation(document, follow_on_fence, catalog_id)
	stale["expected_revision"] = follow_on_fence["expected_revision"] + 1
	refusal = invoke(ferrum, "catalog-insert-stale", stale)
	error = refusal.get("error")
	if (
		refusal.get("schema") != "ferrum-operation-error-v1"
		or "outcome" in refusal
		or not isinstance(error, dict)
		or not isinstance(error.get("catalog_placement_refusal"), dict)
	):
		raise CatalogCliE2eError("stale catalog fence was not refused through the typed protocol")
	placement_refusal = error["catalog_placement_refusal"]
	if (
		placement_refusal.get("category") != "stale_snapshot"
		or placement_refusal.get("recovery") != "refresh_and_restart"
	):
		raise CatalogCliE2eError("stale catalog fence omitted its typed retry recovery")
	if document_fence(document, ferrum, "catalog-inspect-after-refusal") != follow_on_fence:
		raise CatalogCliE2eError("stale catalog refusal changed the caller document")
	print(json.dumps({"schema": "ferrum-template-catalog-cli-e2e-v1", "status": "ok"}, sort_keys=True))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except CatalogCliE2eError as error:
		print(f"catalog CLI E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)

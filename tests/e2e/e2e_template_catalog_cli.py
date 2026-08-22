"""Exercise native CLI template catalog list and benzene insertion end to end."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess

import ferrum_chem


EMPTY = "<cdml xmlns='urn:ferrum:cdml'/>"


def digest(document: str) -> str:
	"""Return the canonical Rust snapshot fence digest."""
	return ferrum_chem.DocumentSession.load(document).snapshot().digest


def invoke(ferrum: Path, command: str, payload: dict[str, object]) -> dict[str, object]:
	"""Run one named document command and require a clean JSON response."""
	result = subprocess.run([ferrum, "document", "command", command, "-"], input=json.dumps(payload), text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
	if result.returncode != 0 or result.stderr:
		raise RuntimeError(f"catalog CLI failed: {result.returncode}: {result.stderr.strip()}")
	response = json.loads(result.stdout)
	if not isinstance(response, dict):
		raise RuntimeError("catalog CLI response was not a JSON object")
	return response


def envelope(operation: dict[str, object]) -> dict[str, object]:
	"""Build one complete protocol envelope."""
	return {"schema": "ferrum-operation-request-v1", "request_id": "catalog-cli-e2e", "operation": operation}


def insert(document: str, revision: int, catalog_id: str = "system/rings/benzene") -> dict[str, object]:
	"""Build one fenced benzene insertion request."""
	return envelope({"kind": "catalog.insert.v1", "document": document, "expected_revision": revision, "expected_digest_hex": digest(document), "catalog_id": catalog_id, "anchor_x": 100.0, "anchor_y": 50.0})


def list_catalog(family: str | None = None, category: str | None = None, query: str | None = None) -> dict[str, object]:
	"""Build one immutable-summary catalog listing request."""
	return envelope({"kind": "catalog.list.v1", "family": family, "category": category, "query": query})


def haworth_summary_facts(entries: list[object]) -> dict[str, tuple[object, ...]]:
	"""Return the stable public summary facts for sealed Haworth entries."""
	facts: dict[str, tuple[object, ...]] = {}
	for entry in entries:
		if not isinstance(entry, dict):
			raise RuntimeError(f"catalog entry was not an object: {entry}")
		provenance = entry.get("provenance")
		category = entry.get("category")
		if not isinstance(provenance, dict) or not isinstance(category, dict):
			raise RuntimeError(f"catalog entry omitted public summary facts: {entry}")
		entry_id = entry.get("id")
		if not isinstance(entry_id, str):
			raise RuntimeError(f"catalog entry omitted ID: {entry}")
		facts[entry_id] = (
			entry.get("family"),
			category.get("id"),
			category.get("name"),
			entry.get("name"),
			provenance.get("source_kind"),
			provenance.get("source_id"),
			provenance.get("license_spdx"),
		)
	return facts


EXPECTED_HAWORTH_SUMMARY_FACTS = {
	"biomolecules/carbohydrates/d-glucose/alpha-d-glucopyranose": ("biomolecule", "carbohydrates_d_glucose", "Carbohydrates / D-glucose", "alpha-D-glucopyranose", "curated_ferrum", "ferrum-authored-d-glucose-haworth-depictions-v1", "LGPL-3.0-only"),
	"biomolecules/carbohydrates/d-glucose/beta-d-glucopyranose": ("biomolecule", "carbohydrates_d_glucose", "Carbohydrates / D-glucose", "beta-D-glucopyranose", "curated_ferrum", "ferrum-authored-d-glucose-haworth-depictions-v1", "LGPL-3.0-only"),
	"biomolecules/carbohydrates/d-glucose/alpha-d-glucofuranose": ("biomolecule", "carbohydrates_d_glucose", "Carbohydrates / D-glucose", "alpha-D-glucofuranose", "curated_ferrum", "ferrum-authored-d-glucose-haworth-depictions-v1", "LGPL-3.0-only"),
	"biomolecules/carbohydrates/d-glucose/beta-d-glucofuranose": ("biomolecule", "carbohydrates_d_glucose", "Carbohydrates / D-glucose", "beta-D-glucofuranose", "curated_ferrum", "ferrum-authored-d-glucose-haworth-depictions-v1", "LGPL-3.0-only"),
}


def main() -> None:
	"""Verify list, benzene topology, chainability, and closed insertion refusal."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--ferrum", required=True, type=Path)
	args = parser.parse_args()
	listed = invoke(args.ferrum, "catalog.list.v1", list_catalog())
	entries = listed.get("outcome", {}).get("entries") if isinstance(listed.get("outcome"), dict) else None
	if not isinstance(entries, list) or entries[0].get("id") != "system/rings/benzene":
		raise RuntimeError(f"missing native benzene summary: {listed}")
	if "document" in entries[0] or "template_cdml" in entries[0]:
		raise RuntimeError("catalog list leaked a template payload")
	for payload, expected_ids in [
		(list_catalog(family="system"), ["system/rings/benzene", "system/rings/cyclopropane", "system/rings/cyclobutane", "system/rings/cyclopentane", "system/rings/cyclohexane", "system/heterocycles/thiophene", "system/heterocycles/furan", "system/heterocycles/pyrrole", "system/heterocycles/purine"]),
		(list_catalog(category="rings"), ["system/rings/benzene", "system/rings/cyclopropane", "system/rings/cyclobutane", "system/rings/cyclopentane", "system/rings/cyclohexane"]),
		(list_catalog(query=" SYSTEM/RINGS "), ["system/rings/benzene", "system/rings/cyclopropane", "system/rings/cyclobutane", "system/rings/cyclopentane", "system/rings/cyclohexane"]),
		(list_catalog(category="heterocycles", query="sulfur"), ["system/heterocycles/thiophene"]),
		(list_catalog(family="system", category="rings", query="missing"), []),
		(list_catalog(family="biomolecule", category="rings"), []),
		(list_catalog(family="system", category="carbohydrates_d_glucose"), []),
		(list_catalog(category="missing"), []),
	]:
		filtered = invoke(args.ferrum, "catalog.list.v1", payload)
		outcome = filtered.get("outcome")
		filtered_entries = outcome.get("entries") if isinstance(outcome, dict) else None
		if not isinstance(filtered_entries, list):
			raise RuntimeError(f"catalog list did not return entries: {filtered}")
		actual_ids = [entry.get("id") for entry in filtered_entries if isinstance(entry, dict)]
		if actual_ids != expected_ids:
			raise RuntimeError(f"catalog filters returned {actual_ids}, expected {expected_ids}")
	biomolecules = invoke(args.ferrum, "catalog.list.v1", list_catalog(family="biomolecule"))
	biomolecule_outcome = biomolecules.get("outcome")
	biomolecule_entries = biomolecule_outcome.get("entries") if isinstance(biomolecule_outcome, dict) else None
	if not isinstance(biomolecule_entries, list) or haworth_summary_facts(biomolecule_entries) != EXPECTED_HAWORTH_SUMMARY_FACTS:
		raise RuntimeError(f"sealed Haworth summary slice changed: {biomolecules}")
	beta_haworth = invoke(args.ferrum, "catalog.list.v1", list_catalog(family="biomolecule", category="carbohydrates_d_glucose", query="beta"))
	beta_outcome = beta_haworth.get("outcome")
	beta_entries = beta_outcome.get("entries") if isinstance(beta_outcome, dict) else None
	if not isinstance(beta_entries, list) or haworth_summary_facts(beta_entries) != {key: value for key, value in EXPECTED_HAWORTH_SUMMARY_FACTS.items() if "/beta-" in key}:
		raise RuntimeError(f"mixed Haworth filters changed: {beta_haworth}")
	first = invoke(args.ferrum, "catalog.insert.v1", insert(EMPTY, 0))
	outcome = first.get("outcome")
	if not isinstance(outcome, dict) or outcome.get("committed_revision") != 1:
		raise RuntimeError(f"catalog insertion did not commit once: {first}")
	document = outcome.get("document")
	if not isinstance(document, str) or document.count('name="C"') != 6 or document.count('type="n2"') != 3:
		raise RuntimeError("benzene insertion topology was not canonical")
	second = invoke(args.ferrum, "catalog.insert.v1", insert(document, 0))
	if not isinstance(second.get("outcome"), dict):
		raise RuntimeError("returned document was not chainable stateless input")
	refused = invoke(args.ferrum, "catalog.insert.v1", insert(EMPTY, 0, "missing"))
	error = refused.get("error")
	if not isinstance(error, dict) or error.get("catalog_placement_refusal", {}).get("category") != "unknown_key":
		raise RuntimeError(f"missing typed catalog refusal: {refused}")
	print(json.dumps({"schema": "ferrum-template-catalog-cli-e2e-v1", "status": "ok"}))


if __name__ == "__main__":
	main()

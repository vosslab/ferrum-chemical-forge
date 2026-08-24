"""Installed-wheel contract for generic Rust-owned catalog placement."""

from __future__ import annotations

import pytest

import ferrum_chem


CATALOG_KEY = "system/rings/benzene"


def _place(session: object, key: str, x: float, y: float) -> object:
	"""Submit one closed catalog intent against the current document observation."""
	snapshot = session.snapshot()
	return session.place_catalog_molecule_v1(
		snapshot.revision, snapshot.digest, key, x, y,
	)


def test_catalog_listing_is_immutable_summary_data() -> None:
	"""The public catalog exposes searchable metadata, not recipe payloads."""
	entries = ferrum_chem.list_catalog_v1("system", "rings", "benzene")
	entry = next(entry for entry in entries if entry.key == CATALOG_KEY)

	assert type(entry) is ferrum_chem.CatalogSummaryV1
	assert (entry.key, entry.family, entry.category) == (CATALOG_KEY, "system", "rings")
	assert entry.schema and entry.catalog_version and entry.label and entry.provenance_source
	with pytest.raises(AttributeError):
		entry.label = "Forged"
	for forbidden in ("cdml", "payload", "recipe", "fragment"):
		assert not hasattr(entry, forbidden)


def test_catalog_placement_is_one_generic_semantic_commit() -> None:
	"""Catalog intent commits through the generic result envelope without V2 receipts."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	placed = _place(session, CATALOG_KEY, 120.0, 80.0)

	assert type(placed) is ferrum_chem.CatalogPlacementResultV1
	assert placed.root_identifier.startswith("ferrum-molecule-v1-")
	assert placed.result.observation.snapshot.revision == 1
	assert 'name="Benzene"' in placed.result.observation.snapshot.cdml
	assert len(placed.result.observation.projection.molecules) == 1
	for retired in (
		"begin_catalog_placement_v2",
		"preview_catalog_placement_v2",
		"prepare_catalog_placement_v2",
		"commit_catalog_placement_v2",
	):
		assert not hasattr(session, retired)


def test_catalog_placement_refuses_a_stale_observation_without_mutation() -> None:
	"""The semantic binding compares revision and digest before generic preparation."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	snapshot = session.snapshot()
	_place(session, CATALOG_KEY, 0.0, 0.0)

	with pytest.raises(ferrum_chem.CatalogPlacementError) as captured:
		session.place_catalog_molecule_v1(
			snapshot.revision, snapshot.digest, CATALOG_KEY, 20.0, 20.0,
		)
	assert captured.value.category == "stale_snapshot"
	assert session.snapshot().revision == 1

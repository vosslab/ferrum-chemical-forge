"""Installed-wheel contract for Rust-owned Ferrum template catalog placement."""

from __future__ import annotations

import pytest

import ferrum_chem


CATALOG_KEY = "system/rings/benzene"
HOST_DOCUMENT = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="host"><atom id="host-a" name="C">'
	'<point x="0" y="0"/></atom></molecule></cdml>'
)


def _begin_benzene(session: object) -> object:
	"""Begin one placement from the exact current Rust snapshot fence."""
	snapshot = session.snapshot()
	gesture = session.begin_catalog_placement_v2(
		snapshot.revision, snapshot.digest, CATALOG_KEY,
	)
	return gesture


def _prepare_benzene(session: object, gesture: object) -> object:
	"""Create one opaque renderer-preflight receipt without client CDML."""
	preview = session.preview_catalog_placement_v2(gesture, 120.0, 80.0)
	assert type(preview) is ferrum_chem.CatalogPlacementPreviewV2
	assert preview.overlay.plan.batches
	assert not hasattr(preview, "atom_points")
	assert not hasattr(preview, "bond_segments")
	prepared = session.prepare_catalog_placement_v2(gesture, preview)
	return prepared


def test_catalog_listing_is_immutable_ferrum_metadata_without_template_payload() -> None:
	"""The public catalog is searchable summary data, never recipe or CDML data."""
	entries = ferrum_chem.list_catalog_v1("system", "rings", "benzene")
	entry = next(entry for entry in entries if entry.key == CATALOG_KEY)

	assert type(entry) is ferrum_chem.CatalogSummaryV1
	assert (entry.key, entry.family, entry.category) == (CATALOG_KEY, "system", "rings")
	assert entry.schema and entry.catalog_version and entry.label
	assert entry.provenance_source
	with pytest.raises(AttributeError):
		entry.label = "Forged"
	for forbidden in ("cdml", "payload", "recipe", "fragment"):
		assert not hasattr(entry, forbidden)
	sulfur_entries = ferrum_chem.list_catalog_v1("system", "heterocycles", "sulfur")
	assert any(entry.key == "system/heterocycles/thiophene" for entry in sulfur_entries)


def test_catalog_placement_uses_only_opaque_handles_one_revision_and_one_undo() -> None:
	"""A catalog entry reaches canonical CDML only through Rust-issued capabilities."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	gesture = _begin_benzene(session)
	prepared = _prepare_benzene(session, gesture)

	assert type(gesture) is ferrum_chem.CatalogPlacementGestureV2
	assert type(prepared) is ferrum_chem.CatalogPlacementReceiptV2
	for handle in (gesture, prepared):
		for forbidden in ("cdml", "payload", "recipe", "candidate"):
			assert not hasattr(handle, forbidden)
	with pytest.raises(TypeError):
		session.commit_catalog_placement_v2(gesture)
	for forbidden in ("begin_catalog_placement_v1", "preview_catalog_placement_v1", "prepare_catalog_placement_v1", "commit_catalog_placement_v1"):
		assert not hasattr(session, forbidden)
	assert not hasattr(session, "commit_complete_cdml_transaction_v1")

	commit = session.commit_catalog_placement_v2(prepared)
	assert type(commit) is ferrum_chem.CatalogPlacementCommitV2
	assert commit.identifier
	assert commit.result.observation.snapshot.revision == 1
	assert "<molecule" in commit.result.observation.snapshot.cdml
	assert 'name="Benzene"' in commit.result.observation.snapshot.cdml
	assert len(commit.result.observation.projection.molecules) == 1
	assert session.undo(1).observation.snapshot.revision == 2
	assert not session.observe(2).projection.molecules
	with pytest.raises(ferrum_chem.CatalogPlacementError) as captured:
		session.commit_catalog_placement_v2(prepared)
	assert captured.value.category == "ReplayedGesture"
	assert session.snapshot().revision == 2


def test_catalog_placement_refuses_foreign_and_stale_handles_without_mutation() -> None:
	"""Session origin and revision fencing reject stale or cross-session capabilities."""
	first = ferrum_chem.DocumentSession.load(HOST_DOCUMENT)
	second = ferrum_chem.DocumentSession.load(HOST_DOCUMENT)
	gesture = _begin_benzene(first)
	second_before = second.snapshot()
	with pytest.raises(ferrum_chem.CatalogPlacementError) as captured:
		second.preview_catalog_placement_v2(gesture, 120.0, 80.0)
	assert captured.value.category == "ForeignSession"
	assert second.snapshot().revision == second_before.revision

	first_before = first.snapshot()
	first.submit(
		first_before.revision,
		ferrum_chem.DocumentOperationV1.set_atom_element("host-a", "N"),
	)
	with pytest.raises(ferrum_chem.CatalogPlacementError) as captured:
		first.preview_catalog_placement_v2(gesture, 120.0, 80.0)
	assert captured.value.category == "StaleSnapshot"
	assert first.snapshot().revision == first_before.revision + 1


def test_catalog_placement_skips_an_opaque_descendant_identifier_collision() -> None:
	"""The native allocator reserves descendant declarations, not just root IDs."""
	source = (
		'<cdml xmlns="urn:ferrum:cdml"><molecule id="host"><atom id="host-a" name="C">'
		'<point x="0" y="0"/><opaque id="ferrum-catalog-benzene-1-a1">'
		'<retained/></opaque></atom></molecule></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	prepared = _prepare_benzene(session, _begin_benzene(session))
	commit = session.commit_catalog_placement_v2(prepared)

	assert commit.identifier != "ferrum-catalog-benzene-1"
	assert '<opaque id="ferrum-catalog-benzene-1-a1"' in session.snapshot().cdml
	assert f'id="{commit.identifier}"' in session.snapshot().cdml


def test_catalog_v2_foreign_refusal_keeps_the_receipt_for_an_owner_retry() -> None:
	"""A non-consuming foreign refusal must not turn a valid V2 receipt into replay."""
	owner = ferrum_chem.DocumentSession.load(HOST_DOCUMENT)
	foreign = ferrum_chem.DocumentSession.load(HOST_DOCUMENT)
	prepared = _prepare_benzene(owner, _begin_benzene(owner))

	with pytest.raises(ferrum_chem.CatalogPlacementError) as captured:
		foreign.commit_catalog_placement_v2(prepared)
	assert captured.value.category == "ForeignSession"
	assert foreign.snapshot().revision == 0
	commit = owner.commit_catalog_placement_v2(prepared)
	assert commit.result.observation.snapshot.revision == 1


def test_catalog_v2_preview_is_renderer_owned_frozen_and_last_preview_wins() -> None:
	"""V2 exposes only a renderer plan and one current opaque candidate capability."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	snapshot = session.snapshot()
	gesture = session.begin_catalog_placement_v2(
		snapshot.revision, snapshot.digest, CATALOG_KEY,
	)
	first = session.preview_catalog_placement_v2(gesture, 80.0, 60.0)
	second = session.preview_catalog_placement_v2(gesture, 120.0, 90.0)
	with pytest.raises(ferrum_chem.CatalogPlacementError) as captured:
		session.prepare_catalog_placement_v2(gesture, first)
	assert captured.value.category == "ReplayedGesture"
	assert type(second.overlay.plan).__name__ == "RenderPlanV2"
	assert second.overlay.plan.batches
	for value in (second, second.overlay):
		for forbidden in ("cdml", "candidate", "digest", "identifier"):
			assert not hasattr(value, forbidden)
	with pytest.raises(AttributeError):
		second.overlay.source_order = 99
	prepared = session.prepare_catalog_placement_v2(gesture, second)
	commit = session.commit_catalog_placement_v2(prepared)
	assert commit.result.observation.snapshot.revision == 1
	assert "Benzene" in commit.result.observation.snapshot.cdml


def test_catalog_v2_haworth_preview_uses_renderer_paths_for_directed_stereo() -> None:
	"""A sealed Haworth entry lowers its directed stereo through renderer paths."""
	key = "biomolecules/carbohydrates/d-glucose/alpha-d-glucopyranose"
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	snapshot = session.snapshot()
	gesture = session.begin_catalog_placement_v2(
		snapshot.revision, snapshot.digest, key,
	)
	preview = session.preview_catalog_placement_v2(gesture, 100.0, 100.0)
	operations = [
		operation
		for batch in preview.overlay.plan.batches
		for operation in batch.operations
	]
	assert any(operation.kind == "path" for operation in operations)
	session.release_catalog_placement_preview_v2(preview)
	assert session.snapshot().revision == 0

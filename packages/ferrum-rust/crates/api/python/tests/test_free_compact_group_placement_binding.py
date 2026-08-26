"""Installed private boundary tests for free compact-group placement."""

# PIP3
import pytest

# Local
import ferrum_chem


#============================================
def _snapshot_facts(session: object) -> tuple[int, str, bool]:
	"""Return the public mutation facts relevant to one private refusal."""
	snapshot = session.snapshot()
	return snapshot.revision, snapshot.digest, snapshot.is_dirty


#============================================
def _begin_methyl(session: object) -> object:
	"""Prepare one free methyl placement from the current authoritative fence."""
	snapshot = session.snapshot()
	return session._begin_place_free_compact_group_v1(
		snapshot.revision, snapshot.digest, "methyl", 12.0, -4.0)


#============================================
def test_free_methyl_placement_commits_one_revision_with_durable_projection_id() -> None:
	"""One accepted placement exposes its durable compact-group identity publicly."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	before = session.snapshot()
	pending = _begin_methyl(session)

	assert _snapshot_facts(session) == (before.revision, before.digest, before.is_dirty)
	committed = session._commit_place_free_compact_group_v1(pending)
	observation = committed.observation
	molecule = next(
		item for item in observation.projection.molecules
		if item.document_object_id == committed.molecule_object_id
	)

	assert (committed.revision, committed.digest, committed.is_dirty) == (
		before.revision + 1, observation.snapshot.digest, True)
	assert observation.snapshot.revision == committed.revision
	assert any(
		item.document_object_id == committed.compact_group_object_id and item.catalog_key == "methyl"
		for item in molecule.compact_groups)


#============================================
@pytest.mark.parametrize(
	("catalog_key", "scene_x", "digest", "category"),
	[
		("not-a-key", 12.0, None, ferrum_chem.FreeCompactGroupPlacementCategoryV1.invalid_catalog_key),
		("nitro", 12.0, None, ferrum_chem.FreeCompactGroupPlacementCategoryV1.unsupported_catalog_key),
		("methyl", float("inf"), None, ferrum_chem.FreeCompactGroupPlacementCategoryV1.non_finite_point),
		("methyl", 12.0, "malformed", ferrum_chem.FreeCompactGroupPlacementCategoryV1.invalid_digest),
	],
)
def test_free_placement_input_refusals_are_typed_and_mutation_free(
	catalog_key: str,
	scene_x: float,
	digest: str | None,
	category: object,
) -> None:
	"""Malformed requests stop before candidate allocation or document mutation."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	before = _snapshot_facts(session)
	expected_digest = before[1] if digest is None else digest

	with pytest.raises(ferrum_chem.FreeCompactGroupPlacementError) as refused:
		session._begin_place_free_compact_group_v1(
			before[0], expected_digest, catalog_key, scene_x, -4.0)
	assert refused.value.category == category
	assert _snapshot_facts(session) == before


#============================================
def test_free_placement_pending_lifecycle_is_session_affine_one_use_and_fenced() -> None:
	"""Foreign, consumed, and stale candidates preserve the current durable document."""
	owner = ferrum_chem.DocumentSession.create_empty_document_v1()
	foreign = ferrum_chem.DocumentSession.create_empty_document_v1()
	owner_before = _snapshot_facts(owner)
	foreign_before = _snapshot_facts(foreign)
	pending = _begin_methyl(owner)

	with pytest.raises(ferrum_chem.FreeCompactGroupPlacementError) as foreign_refusal:
		foreign._commit_place_free_compact_group_v1(pending)
	assert foreign_refusal.value.category == (
		ferrum_chem.FreeCompactGroupPlacementCategoryV1.foreign_session)
	assert _snapshot_facts(owner) == owner_before
	assert _snapshot_facts(foreign) == foreign_before

	owner._cancel_place_free_compact_group_v1(pending)
	with pytest.raises(ferrum_chem.FreeCompactGroupPlacementError) as consumed_refusal:
		owner._commit_place_free_compact_group_v1(pending)
	assert consumed_refusal.value.category == ferrum_chem.FreeCompactGroupPlacementCategoryV1.consumed
	assert _snapshot_facts(owner) == owner_before

	stale = _begin_methyl(owner)
	accepted = _begin_methyl(owner)
	owner._commit_place_free_compact_group_v1(accepted)
	stable = _snapshot_facts(owner)
	with pytest.raises(ferrum_chem.RevisionConflictError) as stale_refusal:
		owner._commit_place_free_compact_group_v1(stale)
	assert stale_refusal.value.category == (
		ferrum_chem.FreeCompactGroupPlacementCategoryV1.stale_revision)
	assert _snapshot_facts(owner) == stable

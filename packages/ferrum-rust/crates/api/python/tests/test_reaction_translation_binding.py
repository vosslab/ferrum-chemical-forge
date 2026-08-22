"""Installed-extension contract for opaque Rust reaction translation."""

from __future__ import annotations

import pytest

import ferrum_chem


SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="left"><atom id="left-a" name="C">'
	'<point x="0" y="0"/></atom></molecule>'
	'<molecule id="right"><atom id="right-a" name="O">'
	'<point x="100" y="0"/></atom></molecule>'
	'<arrow id="arrow"><point x="25" y="0"/><point x="75" y="0"/></arrow>'
	'<reaction id="strict"><reactant idref="left"/><product idref="right"/>'
	'<arrow idref="arrow"/></reaction></cdml>'
)


def _selection(session: object) -> object:
	"""Select the exact strict reaction from a Rust-issued current observation."""
	snapshot = session.snapshot()
	observation = session.observe_reaction_list_v1(snapshot.revision, snapshot.digest)
	return session.select_reaction_v1(observation, "strict")


def test_translation_moves_all_members_preserves_references_and_undoes() -> None:
	"""One opaque receipt moves the full aggregate and keeps definition IDREFs intact."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	gesture = session.begin_reaction_translation_v1(_selection(session), 0.0, 0.0)
	preview = session.preview_reaction_translation_v1(gesture, 20.0, 10.0)
	prepared = session.prepare_reaction_translation_v1(gesture, preview)
	accepted = session.commit_reaction_translation_v1(prepared)
	changed = accepted.result.observation.snapshot

	assert accepted.reaction_id == "strict" and changed.revision == 1
	assert all(reference in changed.cdml for reference in (
		'idref="left"', 'idref="right"', 'idref="arrow"',
	))
	assert '<point x="0.706cm" y="0.353cm"' in changed.cdml
	assert session.undo(changed.revision).observation.snapshot.cdml == SOURCE


def test_translation_enforces_foreign_stale_and_replay_refusals_without_extra_mutation() -> None:
	"""Opaque handles reject hostile lifecycle reuse while retaining authoritative state."""
	owner = ferrum_chem.DocumentSession.load(SOURCE)
	foreign = ferrum_chem.DocumentSession.load(SOURCE)
	gesture = owner.begin_reaction_translation_v1(_selection(owner), 0.0, 0.0, True)
	foreign_before = foreign.snapshot()
	with pytest.raises(ferrum_chem.ReactionGestureError) as foreign_error:
		foreign.preview_reaction_translation_v1(gesture, 20.0, 10.0)
	assert foreign_error.value.category is ferrum_chem.ReactionRefusalCategoryV1.foreign_session
	assert foreign.snapshot().digest == foreign_before.digest

	preview = owner.preview_reaction_translation_v1(gesture, 20.0, 10.0)
	prepared = owner.prepare_reaction_translation_v1(gesture, preview)
	accepted = owner.commit_reaction_translation_v1(prepared)
	with pytest.raises(ferrum_chem.ReactionGestureError) as replay_error:
		owner.commit_reaction_translation_v1(prepared)
	assert replay_error.value.category is ferrum_chem.ReactionRefusalCategoryV1.replayed_gesture
	assert owner.snapshot().digest == accepted.result.observation.snapshot.digest

	stale_gesture = owner.begin_reaction_translation_v1(_selection(owner), 0.0, 0.0)
	stale_preview = owner.preview_reaction_translation_v1(stale_gesture, 20.0, 10.0)
	baseline = owner.snapshot()
	owner.submit(
		baseline.revision,
		ferrum_chem.DocumentOperationV1.set_atom_element("left-a", "N"),
	)
	after_source_change = owner.snapshot()
	with pytest.raises(ferrum_chem.ReactionGestureError) as stale_error:
		owner.prepare_reaction_translation_v1(stale_gesture, stale_preview)
	assert stale_error.value.category is ferrum_chem.ReactionRefusalCategoryV1.stale_snapshot
	assert owner.snapshot().digest == after_source_change.digest

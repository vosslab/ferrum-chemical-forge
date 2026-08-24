"""Installed-extension contract for opaque Rust reaction translation."""

from __future__ import annotations

import defusedxml.ElementTree

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


def _point_coordinates(cdml: str) -> list[tuple[str, str]]:
	"""Return canonical coordinate tokens from the public committed snapshot."""
	root = defusedxml.ElementTree.fromstring(cdml)
	return [
		(point.attrib["x"], point.attrib["y"])
		for point in root.findall(".//{urn:ferrum:cdml}point")
	]


def test_translation_resolves_to_generic_transition_preserves_references_and_undoes() -> None:
	"""One semantic gesture resolves into the sole generic transition receipt."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	gesture = session.begin_reaction_translation_v1(_selection(session), 10.0, 20.0)
	request = session.resolve_reaction_translation_v1(gesture, 45.0, 40.0)
	prepared = session.prepare_session_operation_transition_v1(request)
	accepted = session.commit_session_operation_transition_v1(prepared)
	changed = accepted.observation.snapshot

	assert accepted.outcome.kind == "standard" and changed.revision == 1
	assert all(reference in changed.cdml for reference in (
		'idref="left"', 'idref="right"', 'idref="arrow"',
	))
	assert _point_coordinates(changed.cdml) == [
		("35", "20"), ("135", "20"), ("60", "20"), ("110", "20"),
	]
	assert session.undo(changed.revision).observation.snapshot.cdml == SOURCE


def test_translation_enforces_foreign_stale_and_replay_refusals_without_extra_mutation() -> None:
	"""Opaque handles reject hostile lifecycle reuse while retaining authoritative state."""
	owner = ferrum_chem.DocumentSession.load(SOURCE)
	gesture = owner.begin_reaction_translation_v1(_selection(owner), 10.0, 20.0, True)
	request = owner.resolve_reaction_translation_v1(gesture, 45.0, 40.0)
	prepared = owner.prepare_session_operation_transition_v1(request)
	accepted = owner.commit_session_operation_transition_v1(prepared)
	with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
		owner.commit_session_operation_transition_v1(prepared)
	assert owner.snapshot().digest == accepted.observation.snapshot.digest

	stale_gesture = owner.begin_reaction_translation_v1(_selection(owner), 10.0, 20.0)
	baseline = owner.snapshot()
	owner.apply_document_operation_v1(
		baseline.revision,
		ferrum_chem.DocumentOperationV1.set_atom_element("left-a", "N"),
	)
	after_source_change = owner.snapshot()
	with pytest.raises(ferrum_chem.ReactionGestureError) as stale_error:
		owner.resolve_reaction_translation_v1(stale_gesture, 45.0, 40.0)
	assert stale_error.value.category is ferrum_chem.ReactionRefusalCategoryV1.stale_snapshot
	assert owner.snapshot().digest == after_source_change.digest

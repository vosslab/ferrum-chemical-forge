"""Installed-extension proof for Rust-owned V3 direct-bond pointer probes."""

from __future__ import annotations

from pathlib import Path

import ferrum_chem
import pytest


SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml" version="26.08"><molecule id="m">'
	'<atom id="atom-a" name="C"><point x="0" y="0"/></atom>'
	'<atom id="atom-c" name="C"><point x="40" y="0"/></atom>'
	"</molecule></cdml>"
)


def _frame() -> object:
	"""Return an identity public viewport-to-scene transform."""
	return ferrum_chem.DirectBondViewportToSceneV3(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)


def _empty_probe(x: float, y: float) -> object:
	"""Return a public pointer probe that does not name an atom."""
	return ferrum_chem.DirectBondPointerProbeV3(
		x,
		y,
		_frame(),
		ferrum_chem.DirectBondPointerHitStateV3.none,
	)


def _direct_probe(atom_object_id: str) -> object:
	"""Return a public pointer probe naming one durable atom object."""
	return ferrum_chem.DirectBondPointerProbeV3(
		0.0,
		0.0,
		_frame(),
		ferrum_chem.DirectBondPointerHitStateV3.unique_atom,
		atom_object_id,
	)


def _atom_object_id(session: object, source_id: str) -> str:
	"""Return one current atom's public durable document identity."""
	observation = session.observe(session.snapshot().revision)
	for molecule in observation.projection.molecules:
		for atom in molecule.atoms:
			if atom.source_id == source_id:
				assert atom.id is not None
				return atom.id
	raise AssertionError("direct-bond source fixture must retain its atom")


def _molecule_object_id(session: object) -> str:
	"""Return the fixture molecule's public durable document identity."""
	observation = session.observe(session.snapshot().revision)
	molecule = observation.projection.molecules[0]
	assert molecule.id is not None
	return molecule.id


def _direct_bond_facts(result: object) -> object:
	"""Return the direct-bond facts carried by one generic operation result."""
	assert result.outcome.kind == "direct_bond_v1"
	assert result.outcome.direct_bond is not None
	return result.outcome.direct_bond


def _committed_bond(result: object) -> object:
	"""Return the public projection bond named by one generic operation result."""
	facts = _direct_bond_facts(result)
	return _projected_bond(result.observation, facts.bond_identifier)


def _projected_bond(observation: object, bond_identifier: str) -> object:
	"""Return one named bond from a public document observation."""
	for molecule in observation.projection.molecules:
		for bond in molecule.bonds:
			if bond.source_id == bond_identifier:
				return bond
	raise AssertionError("direct-bond commit must retain its projected bond")


def _directed_endpoint_categories(bond: object) -> tuple[str, str]:
	"""Classify directed endpoints against the two source-document atoms."""
	known_categories = {
		"atom-a": "start-existing",
		"atom-c": "end-existing",
	}
	return (
		known_categories.get(bond.start.source_id, "new"),
		known_categories.get(bond.end.source_id, "new"),
	)


def _commit_direct_bond(
		presentation: object,
		start_identifier: str | None,
		end_identifier: str | None,
	) -> tuple[object, object]:
	"""Commit one V3 gesture through the generic prepared-transition lifecycle."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	observation = session.observe(0)
	start = _direct_probe(_atom_object_id(session, start_identifier)) if start_identifier else _empty_probe(-40.0, 0.0)
	end = _direct_probe(_atom_object_id(session, end_identifier)) if end_identifier else _empty_probe(80.0, 0.0)
	gesture = session.begin_direct_bond_gesture_v3(
		observation.snapshot.revision,
		observation.snapshot.digest,
		start,
		presentation,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	request = gesture.resolve_end_v3(session, end)
	prepared = session.prepare_session_operation_transition_v1(request)
	presentation = prepared.presentation_v1()
	assert presentation.precommit_overlay is not None
	result = session.commit_session_operation_transition_v1(prepared)
	return session, result


def test_direct_bond_snap_policy_invalid_configuration_is_value_error() -> None:
	"""V3-shared snap configuration rejects invalid values without gesture taxonomy."""
	with pytest.raises(ValueError):
		ferrum_chem.DirectBondSnapPolicyV1(angle_increment_degrees=1)


@pytest.mark.parametrize(
	("start", "end"),
	[
		("atom-a", "atom-c"),
		("atom-a", None),
		(None, "atom-c"),
		(None, None),
	],
)
def test_direct_bond_v3_all_probe_forms_retain_directed_endpoint_identity(
		start: str | None,
		end: str | None,
	) -> None:
	"""Each public pointer form preserves directed endpoints after one commit."""
	session, result = _commit_direct_bond(
		ferrum_chem.DocumentBondPresentationV1.normal_single, start, end,
	)
	facts = _direct_bond_facts(result)
	bond = _committed_bond(result)

	assert bond.end.source_id == facts.end_atom_identifier
	if start is not None:
		assert bond.start.source_id == start
	if end is not None:
		assert facts.end_atom_identifier == end
	if start is None:
		assert bond.start.source_id != "atom-a"
	assert session.snapshot().digest == result.observation.snapshot.digest


def test_direct_bond_v3_pointer_refusals_are_typed() -> None:
	with pytest.raises(ferrum_chem.DirectBondPointerProbeErrorV3) as malformed:
		ferrum_chem.DirectBondViewportToSceneV3(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
	assert malformed.value.category == ferrum_chem.DirectBondPointerProbeCategoryV3.malformed_transform

	session = ferrum_chem.DocumentSession.load(SOURCE)
	observation = session.observe(0)
	non_atom_probe = _direct_probe(_molecule_object_id(session))
	with pytest.raises(ferrum_chem.DirectBondPointerProbeErrorV3) as unknown:
		session.begin_direct_bond_gesture_v3(
			observation.snapshot.revision,
			observation.snapshot.digest,
			non_atom_probe,
			ferrum_chem.DocumentBondPresentationV1.normal_single,
			"C",
			ferrum_chem.DirectBondSnapPolicyV1(),
		)
	assert unknown.value.category == ferrum_chem.DirectBondPointerProbeCategoryV3.unknown_direct_atom

	ambiguous_probe = ferrum_chem.DirectBondPointerProbeV3(
		20.0,
		0.0,
		_frame(),
		ferrum_chem.DirectBondPointerHitStateV3.ambiguous_atom,
	)
	with pytest.raises(ferrum_chem.DirectBondPointerProbeErrorV3) as ambiguous:
		session.begin_direct_bond_gesture_v3(
			observation.snapshot.revision,
			observation.snapshot.digest,
			ambiguous_probe,
			ferrum_chem.DocumentBondPresentationV1.normal_single,
			"C",
			ferrum_chem.DirectBondSnapPolicyV1(),
		)
	assert ambiguous.value.category == ferrum_chem.DirectBondPointerProbeCategoryV3.ambiguous_atom


@pytest.mark.parametrize(
	"presentation",
	[
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		ferrum_chem.DocumentBondPresentationV1.solid_wedge,
		ferrum_chem.DocumentBondPresentationV1.hashed_wedge,
	],
)
def test_direct_bond_v3_same_atom_refusal_preserves_admission_semantics(
	presentation: object,
) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	observation = session.observe(0)
	before = observation.snapshot
	probe = _direct_probe(_atom_object_id(session, "atom-a"))
	gesture = session.begin_direct_bond_gesture_v3(
		before.revision,
		before.digest,
		probe,
		presentation,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	request = gesture.resolve_end_v3(session, probe)
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.prepare_session_operation_transition_v1(request)
	after = session.observe(0).snapshot
	assert after.revision == before.revision
	assert after.digest == before.digest


def test_direct_bond_v3_preparation_transfers_one_gesture_once() -> None:
	"""One V3 preparation transfers its gesture and commits exactly once."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()
	gesture = session.begin_direct_bond_gesture_v3(
		before.revision,
		before.digest,
		_direct_probe(_atom_object_id(session, "atom-a")),
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	request = gesture.resolve_end_v3(session, _direct_probe(_atom_object_id(session, "atom-c")))
	prepared = session.prepare_session_operation_transition_v1(request)
	after_admissions = session.snapshot()
	assert (after_admissions.revision, after_admissions.digest, after_admissions.cdml) == (
		before.revision,
		before.digest,
		before.cdml,
	)

	with pytest.raises(ferrum_chem.DirectBondGestureError):
		gesture.resolve_end_v3(session, _direct_probe(_atom_object_id(session, "atom-c")))

	result = session.commit_session_operation_transition_v1(prepared)
	assert _direct_bond_facts(result).bond_identifier
	after_first_commit = session.snapshot()
	assert after_first_commit.revision == before.revision + 1
	with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
		session.commit_session_operation_transition_v1(prepared)


def test_direct_bond_v3_foreign_commit_keeps_owner_transition_redeemable() -> None:
	"""A foreign generic commit refusal leaves the owner transition redeemable."""
	owner = ferrum_chem.DocumentSession.load(SOURCE)
	foreign = ferrum_chem.DocumentSession.load(SOURCE)
	before = owner.snapshot()
	gesture = owner.begin_direct_bond_gesture_v3(
		before.revision,
		before.digest,
		_direct_probe(_atom_object_id(owner, "atom-a")),
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	request = gesture.resolve_end_v3(owner, _direct_probe(_atom_object_id(owner, "atom-c")))
	prepared = owner.prepare_session_operation_transition_v1(request)

	with pytest.raises(ferrum_chem.PreparedOperationForeignSessionError):
		foreign.commit_session_operation_transition_v1(prepared)
	assert foreign.snapshot().revision == 0
	assert owner.snapshot().revision == before.revision

	result = owner.commit_session_operation_transition_v1(prepared)
	assert result.observation.snapshot.revision == before.revision + 1


def test_direct_bond_v3_stale_preparation_preserves_intervening_mutation() -> None:
	"""A generic commit fence rejects a transition made stale by another gesture."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()
	gesture = session.begin_direct_bond_gesture_v3(
		before.revision,
		before.digest,
		_direct_probe(_atom_object_id(session, "atom-a")),
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	stale_request = gesture.resolve_end_v3(session, _direct_probe(_atom_object_id(session, "atom-c")))
	stale_prepared = session.prepare_session_operation_transition_v1(stale_request)
	intervening_gesture = session.begin_direct_bond_gesture_v3(
		before.revision,
		before.digest,
		_direct_probe(_atom_object_id(session, "atom-c")),
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	intervening_request = intervening_gesture.resolve_end_v3(session, _empty_probe(80.0, 0.0))
	intervening_prepared = session.prepare_session_operation_transition_v1(intervening_request)
	intervening_result = session.commit_session_operation_transition_v1(intervening_prepared)
	intervening_snapshot = intervening_result.observation.snapshot

	with pytest.raises(ferrum_chem.OperationValidationError):
		session.commit_session_operation_transition_v1(stale_prepared)
	after = session.snapshot()
	assert (after.revision, after.digest, after.cdml) == (
		intervening_snapshot.revision,
		intervening_snapshot.digest,
		intervening_snapshot.cdml,
	)


def test_direct_bond_v3_next_pointer_candidate_prepares_before_prior_commit() -> None:
	"""A displayed candidate leaves the session free for the next pointer candidate."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()
	first_gesture = session.begin_direct_bond_gesture_v3(
		before.revision,
		before.digest,
		_direct_probe(_atom_object_id(session, "atom-a")),
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	first_request = first_gesture.resolve_end_v3(session, _empty_probe(80.0, 0.0))
	first_prepared = session.prepare_session_operation_transition_v1(first_request)
	assert first_prepared.presentation_v1().precommit_overlay is not None

	second_gesture = session.begin_direct_bond_gesture_v3(
		before.revision,
		before.digest,
		_direct_probe(_atom_object_id(session, "atom-a")),
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	second_request = second_gesture.resolve_end_v3(session, _empty_probe(80.0, 0.0))
	second_prepared = session.prepare_session_operation_transition_v1(second_request)
	assert second_prepared.presentation_v1().precommit_overlay is not None


def test_direct_bond_v3_directed_wedge_is_undoable_and_durable(
		tmp_path: Path,
	) -> None:
	"""A directed existing-to-new wedge survives history and reopening."""
	presentation = ferrum_chem.DocumentBondPresentationV1.solid_wedge
	session, result = _commit_direct_bond(presentation, "atom-a", None)
	facts = _direct_bond_facts(result)
	committed_bond = _committed_bond(result)
	changed = result.observation.snapshot
	undone = session.undo(changed.revision).observation.snapshot
	with pytest.raises(AssertionError, match="direct-bond commit must retain"):
		_projected_bond(session.observe(undone.revision), facts.bond_identifier)
	redone = session.redo(undone.revision).observation.snapshot
	path = tmp_path / "directed-wedge.cdml"
	session.save_atomic(path, redone.revision)
	prepared = ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(
		str(path),
	)
	reopened, _observation, _origin, _source_kind = prepared.take_admission_v1()
	redone_bond = _projected_bond(session.observe(redone.revision), facts.bond_identifier)
	reopened_bond = _projected_bond(reopened.observe(0), facts.bond_identifier)

	assert _directed_endpoint_categories(committed_bond) == ("start-existing", "new")
	assert _directed_endpoint_categories(redone_bond) == ("start-existing", "new")
	assert _directed_endpoint_categories(reopened_bond) == ("start-existing", "new")
	assert committed_bond.source_type == redone_bond.source_type == reopened_bond.source_type


def test_direct_bond_v3_typed_probe_and_post_resolution_refusals_are_nonmutating() -> None:
	"""Public pointer and candidate refusals preserve the authoritative snapshot."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()
	with pytest.raises(ferrum_chem.DirectBondPointerProbeErrorV3) as stale_digest:
		session.begin_direct_bond_gesture_v3(
			before.revision,
			"0" * 64,
			_direct_probe(_atom_object_id(session, "atom-a")),
			ferrum_chem.DocumentBondPresentationV1.normal_single,
			"C",
			ferrum_chem.DirectBondSnapPolicyV1(),
		)

	assert stale_digest.value.category == ferrum_chem.DirectBondPointerProbeCategoryV3.stale_digest
	assert (session.snapshot().revision, session.snapshot().digest, session.snapshot().cdml) == (
		before.revision, before.digest, before.cdml,
	)
	_committed_session, _committed = _commit_direct_bond(
		ferrum_chem.DocumentBondPresentationV1.normal_single, "atom-a", "atom-c",
	)
	committed_before = _committed_session.snapshot()
	gesture = _committed_session.begin_direct_bond_gesture_v3(
		committed_before.revision,
		committed_before.digest,
		_direct_probe(_atom_object_id(_committed_session, "atom-a")),
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	request = gesture.resolve_end_v3(
		_committed_session, _direct_probe(_atom_object_id(_committed_session, "atom-c")),
	)
	with pytest.raises(ferrum_chem.OperationValidationError):
		_committed_session.prepare_session_operation_transition_v1(request)
	assert (
		_committed_session.snapshot().revision,
		_committed_session.snapshot().digest,
		_committed_session.snapshot().cdml,
	) == (committed_before.revision, committed_before.digest, committed_before.cdml)

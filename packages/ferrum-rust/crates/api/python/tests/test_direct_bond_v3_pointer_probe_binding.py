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


def _direct_probe(atom_identifier: str) -> object:
	"""Return a public pointer probe naming one existing atom."""
	return ferrum_chem.DirectBondPointerProbeV3(
		0.0,
		0.0,
		_frame(),
		ferrum_chem.DirectBondPointerHitStateV3.unique_atom,
		atom_identifier,
	)


def _committed_bond(commit: object) -> object:
	"""Return the public projection bond named by one direct-bond commit."""
	return _projected_bond(commit.result.observation, commit.bond_identifier)


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
	"""Commit one V3 gesture through the public pointer-probe lifecycle."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	observation = session.observe(0)
	start = _direct_probe(start_identifier) if start_identifier else _empty_probe(-40.0, 0.0)
	end = _direct_probe(end_identifier) if end_identifier else _empty_probe(80.0, 0.0)
	gesture = session.begin_direct_bond_gesture_v3(
		observation.snapshot.revision,
		observation.snapshot.digest,
		start,
		presentation,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	admission = session.admit_direct_bond_candidate_v3(gesture, end)
	commit = session.commit_direct_bond_admission_v3(admission)
	return session, commit


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
	session, committed = _commit_direct_bond(
		ferrum_chem.DocumentBondPresentationV1.normal_single, start, end,
	)
	bond = _committed_bond(committed)

	assert bond.end.source_id == committed.end_atom_identifier
	if start is not None:
		assert bond.start.source_id == start
	if end is not None:
		assert committed.end_atom_identifier == end
	if start is None:
		assert bond.start.source_id != "atom-a"
	assert session.snapshot().digest == committed.result.observation.snapshot.digest


def test_direct_bond_v3_pointer_refusals_are_typed() -> None:
	with pytest.raises(ferrum_chem.DirectBondPointerProbeErrorV3) as malformed:
		ferrum_chem.DirectBondViewportToSceneV3(0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
	assert malformed.value.category == ferrum_chem.DirectBondPointerProbeCategoryV3.malformed_transform

	session = ferrum_chem.DocumentSession.load(SOURCE)
	observation = session.observe(0)
	non_atom_probe = _direct_probe("unknown-source-id")
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
	atom_identifier = observation.projection.molecules[0].atoms[0].source_id
	assert atom_identifier == "atom-a"
	probe = _direct_probe(atom_identifier)
	gesture = session.begin_direct_bond_gesture_v3(
		before.revision,
		before.digest,
		probe,
		presentation,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	with pytest.raises(ferrum_chem.DirectBondAdmissionRefusalV3) as refusal:
		session.admit_direct_bond_candidate_v3(gesture, probe)
	assert refusal.value.category == ferrum_chem.DirectBondAdmissionCategoryV3.self_loop
	assert refusal.value.recovery == ferrum_chem.DirectBondAdmissionRecoveryV3.adjust_endpoint
	after = session.observe(0).snapshot
	assert after.revision == before.revision
	assert after.digest == before.digest


def test_direct_bond_v3_admissions_preflight_and_redeem_once() -> None:
	"""Sibling preflight receipts share one commit-only gesture capability."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()
	gesture = session.begin_direct_bond_gesture_v3(
		before.revision,
		before.digest,
		_direct_probe("atom-a"),
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	first_admission = session.admit_direct_bond_candidate_v3(
		gesture, _direct_probe("atom-c"),
	)
	sibling_admission = session.admit_direct_bond_candidate_v3(
		gesture, _direct_probe("atom-c"),
	)
	after_admissions = session.snapshot()
	assert (after_admissions.revision, after_admissions.digest, after_admissions.cdml) == (
		before.revision,
		before.digest,
		before.cdml,
	)

	session.commit_direct_bond_admission_v3(first_admission)
	after_first_commit = session.snapshot()
	with pytest.raises(ferrum_chem.DirectBondCommitError) as sibling_replay:
		session.commit_direct_bond_admission_v3(sibling_admission)
	assert sibling_replay.value.category == ferrum_chem.DirectBondCommitCategoryV1.replayed_receipt
	assert sibling_replay.value.recovery == ferrum_chem.DirectBondCommitRecoveryV1.refresh_and_restart
	after_sibling_commit = session.snapshot()
	assert (
		after_sibling_commit.revision,
		after_sibling_commit.digest,
		after_sibling_commit.cdml,
	) == (
		after_first_commit.revision,
		after_first_commit.digest,
		after_first_commit.cdml,
	)

	with pytest.raises(ferrum_chem.DirectBondAdmissionRefusalV3) as later_admission:
		session.admit_direct_bond_candidate_v3(gesture, _direct_probe("atom-c"))
	assert later_admission.value.category == ferrum_chem.DirectBondAdmissionCategoryV3.replayed_gesture
	assert later_admission.value.recovery == ferrum_chem.DirectBondAdmissionRecoveryV3.refresh_and_restart


def test_direct_bond_v3_foreign_commit_keeps_owner_receipt_redeemable() -> None:
	"""A foreign commit refusal leaves the V3 admission available to its owner."""
	owner = ferrum_chem.DocumentSession.load(SOURCE)
	foreign = ferrum_chem.DocumentSession.load(SOURCE)
	before = owner.snapshot()
	gesture = owner.begin_direct_bond_gesture_v3(
		before.revision,
		before.digest,
		_direct_probe("atom-a"),
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	admission = owner.admit_direct_bond_candidate_v3(gesture, _direct_probe("atom-c"))

	with pytest.raises(ferrum_chem.DirectBondCommitError) as refusal:
		foreign.commit_direct_bond_admission_v3(admission)
	assert refusal.value.category == ferrum_chem.DirectBondCommitCategoryV1.foreign_session
	assert refusal.value.recovery == ferrum_chem.DirectBondCommitRecoveryV1.refresh_and_restart
	assert foreign.snapshot().revision == 0
	assert owner.snapshot().revision == before.revision

	committed = owner.commit_direct_bond_admission_v3(admission)
	assert committed.result.observation.snapshot.revision == before.revision + 1


@pytest.mark.parametrize(
	("form", "start", "end", "endpoint_categories"),
	[
		("ExistingExisting", "atom-a", "atom-c", ("start-existing", "end-existing")),
		("ExistingNew", "atom-a", None, ("start-existing", "new")),
		("NewExisting", None, "atom-c", ("new", "end-existing")),
		("NewNew", None, None, ("new", "new")),
	],
)
def test_direct_bond_v3_directed_wedges_are_undoable_and_durable_for_all_forms(
		form: str,
		start: str | None,
		end: str | None,
		endpoint_categories: tuple[str, str],
		tmp_path: Path,
	) -> None:
	"""Directed wedges retain all endpoint forms through history and reopening."""
	presentation = ferrum_chem.DocumentBondPresentationV1.solid_wedge
	source_type = "wedge"
	session, committed = _commit_direct_bond(presentation, start, end)
	changed = committed.result.observation.snapshot
	undone = session.undo(changed.revision).observation.snapshot
	redone = session.redo(undone.revision).observation.snapshot
	published = session.save_atomic(tmp_path / f"{form}-{source_type}.cdml", redone.revision)
	prepared = ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(
		str(tmp_path / f"{form}-{source_type}.cdml"),
	)
	reopened, _observation, _origin, _source_kind = prepared.take_admission_v1()
	redone_bond = _projected_bond(session.observe(redone.revision), committed.bond_identifier)
	reopened_bond = _projected_bond(reopened.observe(0), committed.bond_identifier)

	assert committed.bond_identifier not in undone.cdml
	assert (
		redone_bond.source_type,
		_directed_endpoint_categories(redone_bond),
	) == (source_type, endpoint_categories)
	assert (
		reopened_bond.source_type,
		_directed_endpoint_categories(reopened_bond),
	) == (source_type, endpoint_categories)
	assert published.published_snapshot.cdml == redone.cdml


def test_direct_bond_v3_typed_probe_and_post_resolution_refusals_are_nonmutating() -> None:
	"""Public pointer and candidate refusals preserve the authoritative snapshot."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()
	with pytest.raises(ferrum_chem.DirectBondPointerProbeErrorV3) as stale_digest:
		session.begin_direct_bond_gesture_v3(
			before.revision,
			"0" * 64,
			_direct_probe("atom-a"),
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
		_direct_probe("atom-a"),
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	with pytest.raises(ferrum_chem.DirectBondAdmissionRefusalV3) as duplicate:
		_committed_session.admit_direct_bond_candidate_v3(gesture, _direct_probe("atom-c"))

	assert duplicate.value.category == ferrum_chem.DirectBondAdmissionCategoryV3.duplicate_bond
	assert (
		_committed_session.snapshot().revision,
		_committed_session.snapshot().digest,
		_committed_session.snapshot().cdml,
	) == (committed_before.revision, committed_before.digest, committed_before.cdml)

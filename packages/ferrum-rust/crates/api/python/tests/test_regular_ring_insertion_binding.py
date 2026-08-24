"""Installed private binding behavior for detached Rust-owned regular rings."""

import pytest

import ferrum_chem


def _snapshot_facts(snapshot: object) -> tuple[str, int, str]:
	"""Return the durable facts that a refused authoring operation must preserve."""
	return (snapshot.cdml, snapshot.revision, snapshot.digest)


def _is_carbon_single_cycle(molecule: object) -> bool:
	"""Recognize ordinary CDML cycle semantics without depending on generated names."""
	if not molecule.atoms or len(molecule.atoms) != len(molecule.bonds):
		return False
	if any(atom.element != "C" for atom in molecule.atoms):
		return False
	if any(bond.source_type != "n1" for bond in molecule.bonds):
		return False
	degrees = {atom.source_id: 0 for atom in molecule.atoms}
	for bond in molecule.bonds:
		if bond.start.source_id == bond.end.source_id:
			return False
		if bond.start.source_id not in degrees or bond.end.source_id not in degrees:
			return False
		degrees[bond.start.source_id] += 1
		degrees[bond.end.source_id] += 1
	return all(degree == 2 for degree in degrees.values())


def test_regular_ring_generic_operation_commits_one_durable_cycle() -> None:
	"""One regular-ring operation commits through the generic session lifecycle."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	operation = ferrum_chem.DocumentOperationV1.insert_regular_ring_v1(
		6, 13.0, -7.0, 4.0)
	prepared = session.prepare_session_operation_transition_v1(
		operation.transition_request_v1(0))
	result = session.commit_session_operation_transition_v1(prepared)
	molecule = result.observation.projection.molecules[0]

	assert _is_carbon_single_cycle(molecule)
	assert result.outcome.molecule_inserted.molecule_identifier == molecule.source_id


def test_private_regular_ring_refusal_preserves_current_snapshot() -> None:
	"""Invalid intent and stale provenance leave the session at its current result."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	baseline = session.snapshot()
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.insert_regular_ring_v1(
			6, 0.0, 0.0, float("inf"))
	assert _snapshot_facts(session.snapshot()) == _snapshot_facts(baseline)

	stale = session.prepare_session_operation_transition_v1(
		ferrum_chem.DocumentOperationV1.insert_regular_ring_v1(6, 0.0, 0.0, 4.0)
		.transition_request_v1(0))
	accepted = session.prepare_session_operation_transition_v1(
		ferrum_chem.DocumentOperationV1.insert_regular_ring_v1(6, 20.0, 0.0, 4.0)
		.transition_request_v1(0))
	session.commit_session_operation_transition_v1(accepted)
	before_refusal = session.snapshot()
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.commit_session_operation_transition_v1(stale)
	assert _snapshot_facts(session.snapshot()) == _snapshot_facts(before_refusal)

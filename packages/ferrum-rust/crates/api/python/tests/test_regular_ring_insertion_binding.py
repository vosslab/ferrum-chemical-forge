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


def _point_coordinates(points: object) -> list[tuple[float, float, float]]:
	"""Compare copied immutable coordinates, not Python wrapper identity."""
	return [(point.x, point.y, point.z) for point in points]


def test_private_regular_ring_receipt_projects_rust_vertices_and_durable_cycle() -> None:
	"""One private receipt carries the preview geometry that Rust commits as CDML."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	prepared = session.prepare_create_regular_ring_v1(0, 6, 13.0, -7.0, 4.0)
	result = session.commit_create_regular_ring_v1(0, prepared)
	molecule = result.observation.projection.molecules[0]

	assert _is_carbon_single_cycle(molecule)
	assert _point_coordinates(atom.position for atom in molecule.atoms) == _point_coordinates(
		prepared.vertices,
	)


def test_private_regular_ring_refusal_preserves_current_snapshot() -> None:
	"""Invalid intent and stale provenance leave the session at its current result."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	baseline = session.snapshot()
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.prepare_create_regular_ring_v1(0, 6, 0.0, 0.0, float("inf"))
	assert _snapshot_facts(session.snapshot()) == _snapshot_facts(baseline)

	stale = session.prepare_create_regular_ring_v1(0, 6, 0.0, 0.0, 4.0)
	accepted = session.prepare_create_regular_ring_v1(0, 6, 20.0, 0.0, 4.0)
	session.commit_create_regular_ring_v1(0, accepted)
	before_refusal = session.snapshot()
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.commit_create_regular_ring_v1(1, stale)
	assert _snapshot_facts(session.snapshot()) == _snapshot_facts(before_refusal)

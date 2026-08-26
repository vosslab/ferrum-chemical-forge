"""Semantic Python checks for Rust-owned selected-atom rotation."""

# Standard Library
import math

# PIP3 modules
import pytest

import ferrum_chem


SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C"><point x="0" y="0"/></atom>'
	'<atom id="b" name="O"><point x="10" y="0" z="3"/></atom>'
	'<bond id="ab" start="a" end="b" type="n1"/></molecule></cdml>'
)
HALF_AUTHORED_UNIT_POINTS = (0.001 * 72.0 / 2.54) / 2.0


def _target(molecule_id: str, atom_id: str) -> object:
	"""Create one frozen direct-atom selector."""
	return ferrum_chem.DocumentAtomRotationTargetV1.create(molecule_id, atom_id)


def test_atom_rotation_is_one_revisioned_semantic_operation() -> None:
	"""A positive radian angle rotates every selected durable point together."""
	targets = (_target("m", "a"), _target("m", "b"))
	operation = ferrum_chem.DocumentOperationV1.rotate_atoms(
		targets, 0.0, 0.0, math.pi / 2.0,
	)
	session = ferrum_chem.DocumentSession.load(SOURCE)
	rotated = session.apply_document_operation_v1(0, operation).observation
	atoms = rotated.projection.molecules[0].atoms
	assert atoms[1].position.x == pytest.approx(0.0, abs=HALF_AUTHORED_UNIT_POINTS)
	assert atoms[1].position.y == pytest.approx(10.0, abs=HALF_AUTHORED_UNIT_POINTS)
	assert atoms[1].position.z == 3.0
	assert session.undo(1).observation.projection.molecules[0].atoms[1].position.x == 10.0


def test_atom_rotation_factory_and_resolution_fail_without_mutation() -> None:
	"""Exact input and foreign durable live targets reject atomically."""
	target = _target("m", "a")
	with pytest.raises(TypeError):
		ferrum_chem.DocumentOperationV1.rotate_atoms([target], 0, 0, 1)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.rotate_atoms((target,), 0, 0, True)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.rotate_atoms((target, target), 0, 0, 1)

	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()
	projection = session.observe(before.revision).projection
	molecule = projection.molecules[0]
	foreign_session = ferrum_chem.DocumentSession.load(SOURCE.replace('molecule id="m"', 'molecule id="other"'))
	foreign_molecule = foreign_session.observe(0).projection.molecules[0]
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
		session.rotate_live_document_atoms_v1(
			before.revision,
			before.digest,
			((molecule.document_object_id, molecule.atoms[0].document_object_id),
			(foreign_molecule.document_object_id, foreign_molecule.atoms[0].document_object_id)),
			0.0,
			0.0,
			1.0,
		)
	assert caught.value.category == "unknown_document_object"
	assert caught.value.location == "document_object"
	after = session.snapshot()
	assert (after.revision, after.digest) == (before.revision, before.digest)

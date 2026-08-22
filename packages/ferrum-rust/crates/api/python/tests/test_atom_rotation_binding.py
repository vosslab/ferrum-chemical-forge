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
    rotated = session.submit(0, operation).observation
    atoms = rotated.projection.molecules[0].atoms
    assert atoms[1].position.x == pytest.approx(0.0, abs=HALF_AUTHORED_UNIT_POINTS)
    assert atoms[1].position.y == pytest.approx(10.0, abs=HALF_AUTHORED_UNIT_POINTS)
    assert atoms[1].position.z == 3.0
    assert session.undo(1).observation.projection.molecules[0].atoms[1].position.x == 10.0


def test_atom_rotation_factory_and_resolution_fail_without_mutation() -> None:
    """Exact tuple/scalars and molecule ownership reject hostile intent atomically."""
    target = _target("m", "a")
    with pytest.raises(TypeError):
        ferrum_chem.DocumentOperationV1.rotate_atoms([target], 0, 0, 1)
    with pytest.raises(ferrum_chem.OperationValidationError):
        ferrum_chem.DocumentOperationV1.rotate_atoms((target,), 0, 0, True)
    with pytest.raises(ferrum_chem.OperationValidationError):
        ferrum_chem.DocumentOperationV1.rotate_atoms((target, target), 0, 0, 1)

    session = ferrum_chem.DocumentSession.load(SOURCE)
    missing = _target("other", "a")
    operation = ferrum_chem.DocumentOperationV1.rotate_atoms((target, missing), 0, 0, 1)
    before = session.snapshot()
    with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
        session.submit(0, operation)
    assert caught.value.object_id == "a"
    assert "molecule other" in str(caught.value)
    after = session.snapshot()
    assert (after.revision, after.digest) == (before.revision, before.digest)

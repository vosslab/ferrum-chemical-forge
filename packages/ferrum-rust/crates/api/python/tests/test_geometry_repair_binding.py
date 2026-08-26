"""Semantic Python checks for Rust-owned geometry repair."""

# Standard library
import math

# PIP3 modules
import pytest

import ferrum_chem


SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C">'
    '<point x="0.2" y="0.2" z="2"/></atom>'
    '<atom id="b" name="O"><point x="0" y="0"/></atom>'
    '<bond id="ab" start="a" end="b" type="n1"/></molecule></cdml>'
)
HALF_AUTHORED_UNIT_POINTS = (0.001 * 72.0 / 2.54) / 2.0


def _repair_live_molecule(session: object, kind: object, spacing: float) -> object:
    """Repair the current projected molecule through its Rust-issued address."""
    snapshot = session.snapshot()
    molecule = session.observe(snapshot.revision).projection.molecules[0]
    return session.repair_live_document_geometry_v1(
        snapshot.revision, snapshot.digest, (molecule.document_object_id,), kind, spacing,
    ).observation


def test_hex_snap_is_one_revisioned_sparse_repair() -> None:
    """The supported repair delegates to Rust and preserves non-planar facts."""
    session = ferrum_chem.DocumentSession.load(SOURCE)
    repaired = _repair_live_molecule(
        session, ferrum_chem.DocumentGeometryRepairKindV1.snap_to_hex_grid, 1.0,
    )
    atom = repaired.projection.molecules[0].atoms[0]
    assert atom.position.x == pytest.approx(0.0, abs=HALF_AUTHORED_UNIT_POINTS)
    assert atom.position.y == pytest.approx(0.0, abs=HALF_AUTHORED_UNIT_POINTS)
    assert atom.position.z == 2.0
    assert session.undo(1).observation.projection.molecules[0].atoms[0].position.x == 0.2


def test_live_repair_target_failures_preserve_current_snapshot() -> None:
    """Exact immutable IDs, spacing, and durable targets fail closed."""
    kind = ferrum_chem.DocumentGeometryRepairKindV1.snap_to_hex_grid
    session = ferrum_chem.DocumentSession.load(SOURCE)
    before = session.snapshot()
    molecule_id = session.observe(before.revision).projection.molecules[0].document_object_id
    with pytest.raises(TypeError):
        session.repair_live_document_geometry_v1(
            before.revision, before.digest, [molecule_id], kind, 1.0,
        )
    with pytest.raises(ferrum_chem.OperationValidationError):
        session.repair_live_document_geometry_v1(
            before.revision, before.digest, (molecule_id,), kind, True,
        )
    with pytest.raises(ferrum_chem.OperationValidationError):
        session.repair_live_document_geometry_v1(
            before.revision, before.digest, (molecule_id, molecule_id), kind, 1.0,
        )

    with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
        session.repair_live_document_geometry_v1(
            before.revision, before.digest,
            ("ferrum-document-object-v1/00000000000000000000000000000000",), kind, 1.0,
        )
    assert str(caught.value)
    after = session.snapshot()
    assert (after.revision, after.digest) == (before.revision, before.digest)


def test_straighten_bonds_uses_terminal_endpoint_contract() -> None:
    """Two-atom anchoring uses lexical identity and ignores common spacing."""
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="z" name="O">'
        '<point x="0.9659258262890683" y="-0.25881904510252074" z="3"/>'
        '</atom><atom id="a" name="C"><point x="0" y="0"/></atom>'
        '<bond id="az" start="a" end="z" type="n1"/></molecule></cdml>'
    )
    repaired = _repair_live_molecule(
        ferrum_chem.DocumentSession.load(source),
        ferrum_chem.DocumentGeometryRepairKindV1.straighten_bonds, 777.0,
    )
    moved, fixed = repaired.projection.molecules[0].atoms
    assert moved.position.x == pytest.approx(
        math.sqrt(3.0) / 2.0, abs=HALF_AUTHORED_UNIT_POINTS,
    )
    assert moved.position.y == pytest.approx(-0.5, abs=HALF_AUTHORED_UNIT_POINTS)
    assert moved.position.z == 3.0
    assert (fixed.position.x, fixed.position.y) == (0.0, 0.0)


def test_normalize_bond_lengths_uses_explicit_spacing_and_preserves_direction() -> None:
    """The frozen kind reaches the Rust tree planner without frontend geometry."""
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C"><point x="-20" y="0"/>'
        '</atom><atom id="b" name="N"><point x="0" y="0"/></atom>'
        '<atom id="c" name="O"><point x="0" y="30"/></atom>'
        '<bond id="ab" start="a" end="b" type="n1"/>'
        '<bond id="bc" start="b" end="c" type="n1"/></molecule></cdml>'
    )
    repaired = _repair_live_molecule(
        ferrum_chem.DocumentSession.load(source),
        ferrum_chem.DocumentGeometryRepairKindV1.normalize_bond_lengths, 10.0,
    )
    first, root, last = repaired.projection.molecules[0].atoms
    assert first.position.x == pytest.approx(-10.0, abs=HALF_AUTHORED_UNIT_POINTS)
    assert (first.position.y, root.position.x, root.position.y) == (0.0, 0.0, 0.0)
    assert last.position.x == 0.0
    assert last.position.y == pytest.approx(10.0, abs=HALF_AUTHORED_UNIT_POINTS)


def test_normalize_bond_angles_preserves_length_and_authored_child_order() -> None:
    """The frozen kind delegates slot ownership and coordinate work to Rust."""
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="root" name="C"><point x="0" y="0"/>'
        '</atom><atom id="z_first" name="N"><point x="10" y="1" z="3"/></atom>'
        '<atom id="a_second" name="O"><point x="10" y="2"/></atom>'
        '<bond id="z_first_bond" start="root" end="z_first" type="n1"/>'
        '<bond id="a_second_bond" start="root" end="a_second" type="n1"/>'
        '</molecule></cdml>'
    )
    repaired = _repair_live_molecule(
        ferrum_chem.DocumentSession.load(source),
        ferrum_chem.DocumentGeometryRepairKindV1.normalize_bond_angles, 20.0,
    )
    root, first, second = repaired.projection.molecules[0].atoms
    first_distance = math.hypot(10.0, 1.0)
    second_distance = math.hypot(10.0, 2.0)
    assert (first.position.x, first.position.y) == pytest.approx(
        (first_distance, 0.0), abs=HALF_AUTHORED_UNIT_POINTS,
    )
    assert first.position.z == 3.0
    assert (second.position.x, second.position.y) == pytest.approx(
        (second_distance / 2.0, second_distance * math.sqrt(3.0) / 2.0),
        abs=HALF_AUTHORED_UNIT_POINTS,
    )


def test_normalize_rings_preserves_centroid_and_moves_substituent_rigidly() -> None:
    """The frozen ring kind preserves its bounded topology contract."""
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C"><point x="0" y="0"/>'
        '</atom><atom id="b" name="C"><point x="20" y="0"/></atom>'
        '<atom id="c" name="C"><point x="15" y="10"/></atom>'
        '<atom id="d" name="C"><point x="0" y="10"/></atom>'
        '<atom id="side" name="O"><point x="-10" y="10" z="4"/></atom>'
        '<bond id="ab" start="a" end="b" type="n1"/>'
        '<bond id="bc" start="b" end="c" type="n1"/>'
        '<bond id="cd" start="c" end="d" type="n1"/>'
        '<bond id="da" start="d" end="a" type="n1"/>'
        '<bond id="ds" start="d" end="side" type="n1"/></molecule></cdml>'
    )
    session = ferrum_chem.DocumentSession.load(source)
    before = tuple(atom.position for atom in session.observe(0).projection.molecules[0].atoms)
    after = tuple(
        atom.position for atom in _repair_live_molecule(
            session, ferrum_chem.DocumentGeometryRepairKindV1.normalize_rings, 20.0,
        ).projection.molecules[0].atoms
    )
    before_center = tuple(sum(getattr(point, axis) for point in before[:4]) / 4 for axis in ("x", "y"))
    after_center = tuple(sum(getattr(point, axis) for point in after[:4]) / 4 for axis in ("x", "y"))
    assert after_center == pytest.approx(before_center, abs=HALF_AUTHORED_UNIT_POINTS)
    assert after[4].z == 4.0
    assert (
        after[4].x - before[4].x,
        after[4].y - before[4].y,
    ) == pytest.approx(
        (after[3].x - before[3].x, after[3].y - before[3].y),
        abs=HALF_AUTHORED_UNIT_POINTS,
    )

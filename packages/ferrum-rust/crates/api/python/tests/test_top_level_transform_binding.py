"""Semantic Python checks for Rust-owned direct-root transforms."""

# PIP3 modules
import pytest

import ferrum_chem


SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C"><point x="1" y="2"/>'
    '</atom></molecule><plus id="p"><point x="5" y="7"/></plus></cdml>'
)
AUTHORED_HALF_UNIT_POINTS = (0.001 * 72.0 / 2.54) / 2.0


def _selector(identifier: str, kind: object) -> object:
    """Build one frozen exact-kind durable root selector."""
    return ferrum_chem.DocumentTopLevelRootSelectorV1.create(identifier, kind)


def test_scale_and_mirror_share_the_closed_revisioned_operation_boundary() -> None:
    """Affine factories preserve aggregate-pivot semantics and frozen intent."""
    kinds = ferrum_chem.DocumentTopLevelRootKindV1
    targets = (_selector("m", kinds.molecule), _selector("p", kinds.plus))
    session = ferrum_chem.DocumentSession.load(SOURCE)
    scale = ferrum_chem.DocumentOperationV1.scale_top_level_roots(targets, 2, 1.0)
    scaled = session.apply_document_operation_v1(0, scale).observation
    assert scaled.projection.molecules[0].atoms[0].position.x == pytest.approx(
        -1.0, abs=AUTHORED_HALF_UNIT_POINTS,
    )
    assert scaled.projection.presentation_stack.roots[0].plus.anchor.x == pytest.approx(
        7.0, abs=AUTHORED_HALF_UNIT_POINTS,
    )

    mirror = ferrum_chem.DocumentOperationV1.mirror_top_level_roots(
        targets, ferrum_chem.DocumentTopLevelMirrorV1.horizontal,
    )
    mirrored = session.apply_document_operation_v1(1, mirror).observation
    assert mirrored.projection.molecules[0].atoms[0].position.y == pytest.approx(
        7.0, abs=AUTHORED_HALF_UNIT_POINTS,
    )
    assert mirrored.projection.presentation_stack.roots[0].plus.anchor.y == pytest.approx(
        2.0, abs=AUTHORED_HALF_UNIT_POINTS,
    )


def test_rigid_transform_factory_rejects_forged_or_invalid_intent() -> None:
    """Exact tuple, scalar, kind, and alignment grammar fail before mutation."""
    kinds = ferrum_chem.DocumentTopLevelRootKindV1
    selector = _selector("p", kinds.plus)
    with pytest.raises(ferrum_chem.OperationValidationError):
        ferrum_chem.DocumentOperationV1.scale_top_level_roots((selector,), 0, 2)
    with pytest.raises(ferrum_chem.OperationValidationError):
        ferrum_chem.DocumentOperationV1.align_top_level_roots(
            (selector,), ferrum_chem.DocumentTopLevelAlignmentV1.left,
        )
    with pytest.raises(AttributeError):
        selector.root_id = "forged"

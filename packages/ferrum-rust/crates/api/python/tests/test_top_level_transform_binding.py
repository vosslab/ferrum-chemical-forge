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


def test_rigid_translation_is_one_revisioned_operation_across_root_kinds() -> None:
    """One closed operation moves molecule and presentation geometry together."""
    kinds = ferrum_chem.DocumentTopLevelRootKindV1
    targets = (_selector("m", kinds.molecule), _selector("p", kinds.plus))
    operation = ferrum_chem.DocumentOperationV1.translate_top_level_roots(
        targets, 3, -1.0,
    )
    session = ferrum_chem.DocumentSession.load(SOURCE)
    changed = session.submit(0, operation).observation
    assert changed.projection.molecules[0].atoms[0].position.x == pytest.approx(
        4.0, abs=AUTHORED_HALF_UNIT_POINTS,
    )
    assert changed.projection.presentation_stack.roots[0].plus.anchor.x == pytest.approx(
        8.0, abs=AUTHORED_HALF_UNIT_POINTS,
    )
    assert session.undo(1).observation.projection.molecules[0].atoms[0].position.x == 1.0


def test_private_translation_anchor_receipt_is_canonical_and_revision_bound() -> None:
    """The private receipt carries Rust geometry and refuses stale provenance."""
    kinds = ferrum_chem.DocumentTopLevelRootKindV1
    targets = (_selector("p", kinds.plus), _selector("m", kinds.molecule))
    session = ferrum_chem.DocumentSession.load(SOURCE)
    receipt = session.observe_top_level_translation_anchor_v1(0, targets)
    assert (
        (receipt.anchor_x, receipt.anchor_y) == (1.0, 2.0)
        and tuple(selector.root_id for selector in receipt.selectors) == ("m", "p")
        and receipt.source_revision == session.snapshot().revision
        and receipt.source_digest == session.snapshot().digest
    )

    changed = session.submit(
        0,
        ferrum_chem.DocumentOperationV1.translate_top_level_roots(
            receipt.selectors, 3.0, -1.0,
        ),
    )
    with pytest.raises(ferrum_chem.RevisionConflictError):
        session.observe_top_level_translation_anchor_v1(receipt.source_revision, receipt.selectors)
    snapshot = session.snapshot()
    assert (snapshot.revision, snapshot.digest) == (
        changed.observation.snapshot.revision, changed.observation.snapshot.digest,
    )


def test_scale_and_mirror_share_the_closed_revisioned_operation_boundary() -> None:
    """Affine factories preserve aggregate-pivot semantics and frozen intent."""
    kinds = ferrum_chem.DocumentTopLevelRootKindV1
    targets = (_selector("m", kinds.molecule), _selector("p", kinds.plus))
    session = ferrum_chem.DocumentSession.load(SOURCE)
    scale = ferrum_chem.DocumentOperationV1.scale_top_level_roots(targets, 2, 1.0)
    scaled = session.submit(0, scale).observation
    assert scaled.projection.molecules[0].atoms[0].position.x == pytest.approx(
        -1.0, abs=AUTHORED_HALF_UNIT_POINTS,
    )
    assert scaled.projection.presentation_stack.roots[0].plus.anchor.x == pytest.approx(
        7.0, abs=AUTHORED_HALF_UNIT_POINTS,
    )

    mirror = ferrum_chem.DocumentOperationV1.mirror_top_level_roots(
        targets, ferrum_chem.DocumentTopLevelMirrorV1.horizontal,
    )
    mirrored = session.submit(1, mirror).observation
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
    with pytest.raises(TypeError):
        ferrum_chem.DocumentOperationV1.translate_top_level_roots([selector], 1, 2)
    with pytest.raises(ferrum_chem.OperationValidationError):
        ferrum_chem.DocumentOperationV1.translate_top_level_roots((selector,), True, 2)
    with pytest.raises(ferrum_chem.OperationValidationError):
        ferrum_chem.DocumentOperationV1.scale_top_level_roots((selector,), 0, 2)
    with pytest.raises(ferrum_chem.OperationValidationError):
        ferrum_chem.DocumentOperationV1.align_top_level_roots(
            (selector,), ferrum_chem.DocumentTopLevelAlignmentV1.left,
        )
    with pytest.raises(AttributeError):
        selector.root_id = "forged"

    session = ferrum_chem.DocumentSession.load(SOURCE)
    missing = _selector("missing", kinds.plus)
    operation = ferrum_chem.DocumentOperationV1.translate_top_level_roots(
        (missing,), 1, 2,
    )
    before = session.snapshot()
    with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
        session.submit(0, operation)
    assert caught.value.object_id == "missing"
    after = session.snapshot()
    assert (after.revision, after.digest) == (before.revision, before.digest)

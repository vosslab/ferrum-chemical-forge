"""Public factory and lifecycle contract for directed wedge reversal."""

from __future__ import annotations

import pytest

import ferrum_chem


SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
    '<atom id="a" name="C"><point x="1" y="2"/></atom>'
    '<atom id="b" name="O"><point x="3" y="2"/></atom>'
    '<bond id="ab" start="a" end="b" type="w1" retained="yes"/>'
    '</molecule></cdml>'
)


def test_directed_bond_endpoint_reverse_factory_uses_the_generic_session_lifecycle() -> None:
    """The public factory commits one reversible directed-depiction change."""
    session = ferrum_chem.DocumentSession.load(SOURCE)
    before = session.observe(0).projection.molecules[0].bonds[0]
    assert before.source_id == "ab"
    assert before.source_id != before.document_object_id
    original_endpoints = (
        before.start.document_object_id,
        before.end.document_object_id,
    )
    changed = session.apply_document_operation_v1(
        0, ferrum_chem.DocumentOperationV1.reverse_directed_bond_endpoints(source_bond_id="ab"),
    ).observation
    bond = changed.projection.molecules[0].bonds[0]

    assert changed.snapshot.revision == 1
    assert bond.source_id == before.source_id
    assert bond.source_type == "w1"
    assert (bond.start.document_object_id, bond.end.document_object_id) == original_endpoints[::-1]
    assert (
        session.undo(1).observation.projection.molecules[0].bonds[0].start.document_object_id
        == original_endpoints[0]
    )
    assert (
        session.redo(2).observation.projection.molecules[0].bonds[0].start.document_object_id
        == original_endpoints[1]
    )

    with pytest.raises(ferrum_chem.OperationValidationError):
        ferrum_chem.DocumentOperationV1.reverse_directed_bond_endpoints(source_bond_id="")

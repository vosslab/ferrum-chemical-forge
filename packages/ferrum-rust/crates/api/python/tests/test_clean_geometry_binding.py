"""Boundary checks for revision-bound native clean-geometry preparation."""

import pytest

import ferrum_chem


SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml"><molecule id="first"><atom id="a" name="C">'
    '<point x="0" y="0"/></atom><atom id="b" name="N">'
    '<point x="10" y="0"/></atom>'
    '<bond id="ab" start="a" end="b" type="n1"/></molecule>'
    '<molecule id="second"><atom id="c" name="C">'
    '<point x="20" y="0"/></atom><atom id="d" name="O">'
    '<point x="30" y="0"/></atom>'
    '<bond id="cd" start="c" end="d" type="n1"/></molecule></cdml>'
)


def test_clean_geometry_rejects_malformed_requests_before_native_loading() -> None:
    """Closed Python inputs fail without mutation or a packaged RDKit adapter."""
    session = ferrum_chem.DocumentSession.load(SOURCE)
    observation = session.observe(0)
    molecule_ids = tuple(
        molecule.id for molecule in observation.projection.molecules
    )

    class TupleSubclass(tuple):
        pass

    with pytest.raises(TypeError):
        ferrum_chem.prepare_clean_geometry_v1(
            observation, list(molecule_ids), 10.0,
        )
    for invalid_ids in (
        TupleSubclass(molecule_ids),
        (),
        (molecule_ids[0], molecule_ids[0]),
        (molecule_ids[0], 7),
    ):
        with pytest.raises(ferrum_chem.OperationValidationError):
            ferrum_chem.prepare_clean_geometry_v1(
                observation, invalid_ids, 10.0,
            )
    for invalid_spacing in (True, 0.0, float("nan")):
        with pytest.raises(ferrum_chem.OperationValidationError):
            ferrum_chem.prepare_clean_geometry_v1(
                observation, molecule_ids, invalid_spacing,
            )

    retained = session.snapshot()
    assert retained.revision == 0
    assert retained.cdml == SOURCE
    assert ferrum_chem.PreparedCleanGeometryV1.__module__ == "ferrum_chem"

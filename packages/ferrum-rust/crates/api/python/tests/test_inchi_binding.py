"""Installed-extension behavior for the closed ABI-4 InChI boundary."""

import pytest

import ferrum_chem


METHANE = "InChI=1S/CH4/h1H4"


def test_standard_inchi_import_and_key_are_owned_values() -> None:
	molecule = ferrum_chem.parse_inchi(METHANE)

	assert molecule.canonical_smiles == "C"
	assert ferrum_chem.inchi_to_inchi_key(METHANE) == "VNWKTOKETHGBQD-UHFFFAOYSA-N"


def test_inchi_prepares_one_revision_bound_document_insertion() -> None:
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 100.0, 200.0)
	prepared = ferrum_chem.prepare_inchi_molecule_v1(METHANE, placement)
	session = ferrum_chem.DocumentSession.load('<cdml xmlns="urn:ferrum:cdml" version="1.0"/>')
	operation = ferrum_chem.DocumentOperationV1.insert_molecule_v1(prepared)
	request = operation.transition_request_v1(0)
	pending = session.prepare_session_operation_transition_v1(request)
	accepted = session.commit_session_operation_transition_v1(pending)
	molecule = accepted.observation.projection.molecules[0]

	assert prepared.atom_count == 1 and prepared.bond_count == 0
	assert tuple(atom.element for atom in molecule.atoms) == ("C",)


def test_inchi_export_uses_an_exact_closed_mode() -> None:
	molecule = ferrum_chem.parse_smiles("CC=C(N)C")
	standard = ferrum_chem.molecule_to_inchi(molecule, ferrum_chem.InchiModeV1.standard)
	fixed = ferrum_chem.molecule_to_inchi(
		molecule, ferrum_chem.InchiModeV1.fixed_hydrogen,
	)

	assert standard.startswith("InChI=1S/")
	assert fixed.startswith("InChI=1/") and not fixed.startswith("InChI=1S/")


def test_invalid_inchi_fails_before_native_processing() -> None:
	with pytest.raises(ferrum_chem.InvalidInchi):
		ferrum_chem.parse_inchi("methane")

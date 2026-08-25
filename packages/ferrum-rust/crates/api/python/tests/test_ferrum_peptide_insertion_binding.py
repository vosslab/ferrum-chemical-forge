"""Installed-extension contract for the Qt-owned native peptide bridge."""

import pytest

import ferrum_chem


def _placement() -> object:
	"""Return one finite document insertion placement."""
	return ferrum_chem.validate_insertion_placement_v1(40.0, 100.0, 200.0)


@pytest.mark.parametrize("sequence, position, found", [
	("a", 1, "a"),
	("A A", 2, " "),
	("A" + chr(0x00e9), 2, chr(0x00e9)),
])
def test_native_peptide_syntax_is_typed_before_native_loading(
		sequence: str, position: int, found: str) -> None:
	"""Invalid text is rejected without resolving the packaged chemistry library."""
	with pytest.raises(ferrum_chem.FerrumPeptideSyntaxError) as caught:
		ferrum_chem.prepare_ferrum_peptide_insertion_v1(sequence, _placement())

	error = caught.value
	assert error.position == position
	assert error.found == found
	assert error.alphabet == "ACDEFGHIKLMNPQRSTVWY"
	assert isinstance(error.reason, str)


def test_native_peptide_profile_refuses_unsupported_residue_before_native_loading() -> None:
	"""The closed native-17 profile rejects an unsupported standard residue."""
	with pytest.raises(ferrum_chem.UnsupportedFerrumPeptideProfileError) as caught:
		ferrum_chem.prepare_ferrum_peptide_insertion_v1("AH", _placement())

	error = caught.value
	assert error.position == 2
	assert error.residue == "H"
	assert error.profile == "ferrum-native-peptide-structure-v1"
	assert isinstance(error.reason, str)


def test_native_peptide_preparation_returns_a_frozen_document_insertion() -> None:
	"""The accepted Qt bridge result is a detached Ferrum document value."""
	prepared = ferrum_chem.prepare_ferrum_peptide_insertion_v1("ANKLE", _placement())
	assert prepared.atom_count > 5
	assert prepared.bond_count == prepared.atom_count - 1

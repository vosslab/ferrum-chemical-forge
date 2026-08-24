"""Installed-extension contract for strict supported peptide templates."""

import pytest

import ferrum_chem


def _placement() -> object:
	"""Return one finite document insertion placement."""
	return ferrum_chem.validate_insertion_placement_v1(40.0, 100.0, 200.0)


@pytest.mark.parametrize("sequence, position, found", [
	("a", 1, "a"),
	("A A", 2, " "),
	("Aé", 2, "é"),
])
def test_strict_template_syntax_is_typed_before_native_loading(
		sequence: str, position: int, found: str) -> None:
	"""Invalid text is rejected without needing a packaged native library."""
	with pytest.raises(ferrum_chem.PeptideTemplateSyntaxError) as caught:
		ferrum_chem.prepare_supported_peptide_template_molecule_v1(sequence, _placement())

	error = caught.value
	assert error.position == position
	assert error.found == found
	assert error.alphabet == "ACDEFGHIKLMNPQRSTVWY"
	assert isinstance(error.reason, str)


@pytest.mark.parametrize("sequence, residue", [("AH", "H"), ("AP", "P"), ("AW", "W")])
def test_native_profile_is_typed_before_native_loading(sequence: str, residue: str) -> None:
	"""H, P, and W are excluded by the actual native-17 insertion profile."""
	with pytest.raises(ferrum_chem.UnsupportedPeptideTemplateProfileError) as caught:
		ferrum_chem.prepare_supported_peptide_template_molecule_v1(sequence, _placement())

	error = caught.value
	assert error.position == 2 and error.residue == residue
	assert error.profile == "ferrum-native-peptide-template-insertion-v1"
	assert error.supported_alphabet == "ACDEFGIKLMNQRSTVY"
	assert isinstance(error.reason, str)


def test_over_budget_text_is_typed_before_native_loading() -> None:
	"""Raw input bytes are admitted before native library resolution."""
	with pytest.raises(ferrum_chem.PeptideTemplateResourceError) as caught:
		ferrum_chem.prepare_supported_peptide_template_molecule_v1("R" * 33_825, _placement())

	error = caught.value
	assert error.submitted_bytes == 33_825
	assert error.max_submitted_bytes == 33_824
	assert isinstance(error.reason, str)


def test_supported_template_prepares_and_commits_an_ordinary_molecule() -> None:
	"""A native accepted template returns the frozen DTO and commits normally."""
	prepared = ferrum_chem.prepare_supported_peptide_template_molecule_v1("ANKLE", _placement())
	assert isinstance(prepared, ferrum_chem.MoleculeInsertionV1)
	session = ferrum_chem.DocumentSession.load('<cdml xmlns="urn:ferrum:cdml" version="1.0"/>')
	operation = ferrum_chem.DocumentOperationV1.insert_molecule_v1(prepared)
	pending = session.prepare_session_operation_transition_v1(
		operation.transition_request_v1(0))
	accepted = session.commit_session_operation_transition_v1(pending)
	assert len(accepted.observation.projection.molecules) == 1

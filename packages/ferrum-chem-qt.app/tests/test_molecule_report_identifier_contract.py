"""Receipt validation and presentation coverage for Molecule Report identifiers."""

import pytest

# local repo modules
import ferrum_qt.ferrum.molecule_report_identifier_contract


#============================================
def test_available_identifiers_render_the_complete_rust_issued_trio() -> None:
	"""Qt presents the complete native identity bundle without recalculating it."""
	identifiers = {
		"kind": "available",
		"canonical_smiles": "C",
		"standard_inchi": "InChI=1S/CH4/h1H4",
		"standard_inchi_key": "VNWKTOKETHGBQD-UHFFFAOYSA-N",
	}

	assert ferrum_qt.ferrum.molecule_report_identifier_contract.display_lines(identifiers) == [
		"Identifiers:",
		"  Canonical SMILES: C",
		"  Standard InChI: InChI=1S/CH4/h1H4",
		"  Standard InChIKey: VNWKTOKETHGBQD-UHFFFAOYSA-N",
	]


#============================================
def test_unavailable_identifiers_render_one_explicit_reason() -> None:
	"""Qt makes a Rust-owned unavailable result visible without a fallback export."""
	identifiers = {"kind": "unavailable", "reason": "unsupported_molecule"}

	assert ferrum_qt.ferrum.molecule_report_identifier_contract.display_lines(identifiers) == [
		"Identifiers: unavailable (unsupported_molecule)",
	]


#============================================
@pytest.mark.parametrize("identifiers", [
	{"kind": "available", "canonical_smiles": "C"},
	{"kind": "available", "canonical_smiles": "C", "standard_inchi": "I", "standard_inchi_key": "K", "extra": "x"},
	{"kind": "unavailable", "reason": "unrecognized"},
])
def test_identifier_contract_rejects_partial_unknown_and_extra_outcomes(identifiers: dict) -> None:
	"""Receipt admission accepts neither partial identities nor unrecognized variants."""
	assert not ferrum_qt.ferrum.molecule_report_identifier_contract.valid_identifiers(identifiers)

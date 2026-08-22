"""Installed-extension checks for private native molecule information V1."""

import pytest

import ferrum_chem


_SOURCE = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07">
 <molecule id="methane" name="Methane">
  <atom id="c1" name="C"><point x="0" y="0"/></atom>
 </molecule>
 <molecule id="ammonium" name="Ammonium">
  <atom id="n1" name="N" charge="1" explicit_hydrogens="4">
   <point x="2" y="0"/>
  </atom>
 </molecule>
</cdml>
"""


def _observation() -> tuple[object, object]:
	"""Return one session and its frozen initial observation."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	return session, session.observe(0)


def test_private_information_reports_native_records_and_combined_selection() -> None:
	"""Real ABI4 composition retains source facts and combines document order."""
	session, observation = _observation()
	before = session.snapshot()
	ids = tuple(root.id for root in reversed(observation.projection.molecules))

	receipt = ferrum_chem.inspect_document_molecule_information_v1(
		observation, 0, observation.snapshot.digest, ids,
	)

	assert receipt.schema == "ferrum-document-molecule-information-v1"
	assert receipt.source_revision == 0
	assert receipt.source_digest == observation.snapshot.digest
	assert isinstance(receipt.records, tuple)
	assert [record.source_facts.source_id for record in receipt.records] == [
		"methane", "ammonium",
	]
	assert [record.composition.formula for record in receipt.records] == ["CH4", "H4N+"]
	assert receipt.records[1].composition.net_formal_charge == 1
	assert receipt.combined_selection.formula == "CH8N+"
	assert receipt.combined_selection.net_formal_charge == 1
	assert [
		(entry.symbol, entry.isotope, entry.atom_count)
		for entry in receipt.combined_selection.element_counts
	] == [("C", None, 1), ("H", None, 8), ("N", None, 1)]
	assert all(
		entry.average_mass_contribution > 0.0 and entry.percentage > 0.0
		for entry in receipt.combined_selection.mass_percentages
	)
	assert session.snapshot().revision == before.revision
	assert session.snapshot().digest == before.digest
	assert session.snapshot().is_dirty is False


def test_private_information_keeps_isotope_formula_and_has_no_single_aggregate() -> None:
	"""The sealed RDKit path preserves a labelled heavy atom in its formula."""
	source = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07"><molecule id="labelled">
 <atom id="c1" name="C" isotope="13"><point x="0" y="0"/></atom>
</molecule></cdml>
"""
	session = ferrum_chem.DocumentSession.load(source)
	observation = session.observe(0)
	molecule_id = observation.projection.molecules[0].id

	receipt = ferrum_chem.inspect_document_molecule_information_v1(
		observation, 0, observation.snapshot.digest, (molecule_id,),
	)

	assert receipt.records[0].composition.formula == "[13C]H4"
	assert receipt.records[0].composition.element_counts[0].isotope == 13
	assert receipt.combined_selection is None


@pytest.mark.parametrize("selectors", [(), [], ("not-an-object-id",)])
def test_private_information_rejects_invalid_selector_shapes(selectors: object) -> None:
	"""Selection is an exact nonempty tuple of valid durable object strings."""
	_session, observation = _observation()
	with pytest.raises(ferrum_chem.DocumentMoleculeInformationError) as caught:
		ferrum_chem.inspect_document_molecule_information_v1(
			observation, 0, observation.snapshot.digest, selectors,
		)
	assert caught.value.reason


def test_private_information_rejects_duplicates_stale_and_surrogates() -> None:
	"""Input and observation failures remain inside the information contract."""
	_session, observation = _observation()
	molecule_id = observation.projection.molecules[0].id
	with pytest.raises(ferrum_chem.DocumentMoleculeInformationError) as duplicate:
		ferrum_chem.inspect_document_molecule_information_v1(
			observation, 0, observation.snapshot.digest, (molecule_id, molecule_id),
		)
	assert "repeats" in duplicate.value.reason
	with pytest.raises(ferrum_chem.DocumentMoleculeInformationError) as stale:
		ferrum_chem.inspect_document_molecule_information_v1(
			observation, 1, observation.snapshot.digest, (molecule_id,),
		)
	assert "document changed" in stale.value.reason
	with pytest.raises(ferrum_chem.DocumentMoleculeInformationError) as digest:
		ferrum_chem.inspect_document_molecule_information_v1(
			observation, 0, "\ud800", (molecule_id,),
		)
	assert digest.value.reason == "expected digest must be valid UTF-8 text"
	with pytest.raises(ferrum_chem.DocumentMoleculeInformationError) as selector:
		ferrum_chem.inspect_document_molecule_information_v1(
			observation, 0, observation.snapshot.digest, ("\ud800",),
		)
	assert selector.value.reason == "molecule selectors must be valid UTF-8 text"


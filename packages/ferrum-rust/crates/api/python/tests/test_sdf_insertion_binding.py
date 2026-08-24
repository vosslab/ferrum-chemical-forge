"""Public boundary behavior for atomic multi-record SDF insertion."""

# PIP3 modules
import ferrum_chem
import pytest


#============================================
def _two_record_sdf() -> str:
	"""Build two coordinate-bearing records through the packaged native adapter."""
	first = ferrum_chem.prepare_sdf_record(
		ferrum_chem.parse_smiles("CCO"),
		"ethanol input",
		(("SOURCE", "first\nline"),),
	)
	second = ferrum_chem.prepare_sdf_record(
		ferrum_chem.parse_smiles("[Cl-]"), "", (),
	)
	sdf = ferrum_chem.records_to_sdf(
		(first, second), ferrum_chem.MolblockVersionV1.v3000,
	)
	return sdf.replace(
		"$$$$\n",
		">  <SOURCE>\nsecond\n\n$$$$\n",
		1,
	)


#============================================
def test_sdf_batch_is_frozen_ordered_and_one_document_history_step() -> None:
	"""Retain every record/property while committing exactly one session revision."""
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)
	batch = ferrum_chem.prepare_sdf_molecules_v1(_two_record_sdf(), placement)
	session = ferrum_chem.DocumentSession.load(
		"<cdml xmlns='urn:ferrum:cdml'><opaque payload=\"retained\"/></cdml>",
	)
	prepared = session.prepare_admitted_interchange_records_v1(0, batch)
	result = session.commit_admitted_interchange_records_v1(0, prepared)

	assert batch.record_count == 2
	assert tuple(len(record) for record in prepared.atom_identifiers) == (3, 1)
	assert result.observation.snapshot.revision == 1
	assert tuple(
		molecule.name for molecule in result.observation.projection.molecules
	) == ("ethanol input", None)
	assert "payload=\"retained\"" in result.observation.snapshot.cdml
	assert "urn:ferrum-chemical-forge:interchange-import:v1" in result.observation.snapshot.cdml
	assert "534f55524345" in result.observation.snapshot.cdml
	assert session.undo(1).observation.projection.molecules == []
	assert len(session.redo(2).observation.projection.molecules) == 2
	with pytest.raises(AttributeError):
		batch.record_count = 3


#============================================
def test_sdf_text_insertion_requires_text_input() -> None:
	"""Retire descriptor-free file admission from the insertion-only API."""
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 0.0, 0.0)
	with pytest.raises(TypeError):
		ferrum_chem.prepare_sdf_molecules_v1(b"not text", placement)

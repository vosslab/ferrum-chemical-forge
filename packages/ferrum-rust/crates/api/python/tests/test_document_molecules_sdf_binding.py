"""Installed-extension coverage for multi-root immutable SDF receipts."""

# Standard Library
import pathlib

# PIP3 modules
import ferrum_chem
import pytest


SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml">'
	'<molecule id="left" name="left title">'
	'<atom id="left-atom" name="C"><point x="0" y="0"/></atom>'
	'</molecule>'
	'<molecule id="right" name="right title">'
	'<atom id="right-atom" name="O"><point x="10" y="0"/></atom>'
	'</molecule>'
	'</cdml>'
)


#============================================
def _observed_roots() -> tuple[object, tuple[str, str]]:
	"""Load the inline document with its two known selected-root identifiers."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	observation = session.observe(0)
	left, right = observation.projection.molecules
	return observation, (left.document_object_id, right.document_object_id)


#============================================
def test_multi_root_sdf_receipt_uses_rust_canonical_source_order() -> None:
	"""Export reversed selected membership in the document's canonical source order."""
	observation, molecule_ids = _observed_roots()
	receipt = ferrum_chem.export_document_molecules_sdf_v2(
		observation,
		0,
		observation.snapshot.digest,
		tuple(reversed(molecule_ids)),
		ferrum_chem.MolblockVersionV1.v2000,
	)

	assert (receipt.source_revision, receipt.source_digest, receipt.molecule_ids) == (
		observation.snapshot.revision,
		observation.snapshot.digest,
		molecule_ids,
	)
	assert receipt.sdf.count("$$$$\n") == len(molecule_ids)
	records = receipt.sdf.split("$$$$\n")[:-1]
	assert [record.splitlines()[0] for record in records] == ["left title", "right title"]


#============================================
def test_multi_root_sdf_publication_writes_the_immutable_receipt_atomically(
		tmp_path: pathlib.Path,
		) -> None:
	"""Publish one immutable Rust SDF receipt through the public artifact boundary."""
	observation, molecule_ids = _observed_roots()
	receipt = ferrum_chem.export_document_molecules_sdf_v2(
		observation,
		observation.snapshot.revision,
		observation.snapshot.digest,
		molecule_ids,
		ferrum_chem.MolblockVersionV1.v2000,
	)
	destination = tmp_path / "selected.sdf"
	publication = ferrum_chem.publish_document_molecules_sdf_v2(receipt, destination)
	assert destination.read_text(encoding="utf-8") == receipt.sdf
	assert type(publication.directory_entry_confirmed) is bool


#============================================
def test_duplicate_selectors_use_the_typed_sdf_error_contract() -> None:
	"""Reject a duplicate selected root before native SDF export."""
	observation, molecule_ids = _observed_roots()

	with pytest.raises(ferrum_chem.DocumentMoleculesSdfError) as duplicate:
		ferrum_chem.export_document_molecules_sdf_v2(
			observation,
			observation.snapshot.revision,
			observation.snapshot.digest,
			(molecule_ids[0], molecule_ids[0]),
			ferrum_chem.MolblockVersionV1.v3000,
		)
	assert duplicate.value.reason


#============================================
def test_stale_provenance_uses_the_typed_sdf_error_contract() -> None:
	"""Reject a revision that does not match the observed document."""
	observation, molecule_ids = _observed_roots()

	with pytest.raises(ferrum_chem.DocumentMoleculesSdfError) as stale:
		ferrum_chem.export_document_molecules_sdf_v2(
			observation,
			observation.snapshot.revision + 1,
			observation.snapshot.digest,
			molecule_ids,
			ferrum_chem.MolblockVersionV1.v3000,
		)
	assert stale.value.reason


#============================================
@pytest.mark.parametrize(
	("position", "value"),
	[
		(0, object()),
		(1, True),
		(1, -1),
		(1, 2**64),
		(2, object()),
		(3, ()),
		(4, object()),
	],
)
def test_malformed_sdf_export_inputs_use_the_typed_sdf_error_contract(
	position: int,
	value: object,
) -> None:
	"""Reject malformed public values before native SDF export."""
	observation, molecule_ids = _observed_roots()
	arguments: list[object] = [
		observation,
		observation.snapshot.revision,
		observation.snapshot.digest,
		molecule_ids,
		ferrum_chem.MolblockVersionV1.v2000,
	]
	arguments[position] = value

	with pytest.raises(ferrum_chem.DocumentMoleculesSdfError):
		ferrum_chem.export_document_molecules_sdf_v2(*arguments)

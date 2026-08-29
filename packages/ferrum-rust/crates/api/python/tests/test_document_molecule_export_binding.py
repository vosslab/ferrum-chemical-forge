"""Installed-extension checks for the unified selected-root export receipt."""

from pathlib import Path

import ferrum_chem
import pytest


_SOURCE = """
<cdml xmlns="urn:ferrum:cdml" version="1.0"><molecule id="m1">
 <atom id="a1" name="C"><point x="0" y="0"/></atom>
</molecule></cdml>
"""


def _address() -> tuple[object, object, str]:
	"""Return one exact source observation and direct root ID."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation = session.observe(0)
	return session, observation, observation.projection.molecules[0].document_object_id


def test_selected_root_export_uses_one_frozen_receipt_and_publisher(
		tmp_path: Path) -> None:
	"""The closed format algebra preserves provenance and exact emitted bytes."""
	_session, observation, molecule_id = _address()
	receipt = ferrum_chem.export_document_molecule(
		observation, 0, observation.snapshot.digest, molecule_id,
		ferrum_chem.DocumentMoleculeExportFormat.canonical_smiles,
	)
	assert type(receipt) is ferrum_chem.DocumentMoleculeExport
	assert receipt.source_revision == 0
	assert receipt.source_digest == observation.snapshot.digest
	assert receipt.molecule_id == molecule_id
	assert receipt.format is ferrum_chem.DocumentMoleculeExportFormat.canonical_smiles
	assert receipt.text == "C"
	destination = tmp_path / "molecule.smi"
	publication = ferrum_chem.publish_document_molecule_export(receipt, destination)
	assert type(publication) is ferrum_chem.DocumentMoleculeExportPublication
	assert destination.read_text() == receipt.text


#============================================
def test_selected_root_publication_refuses_existing_entries_without_mutating_them(
		tmp_path: Path) -> None:
	"""The installed PyO3 route never replaces existing regular, linked, or symlinked files."""
	_session, observation, molecule_id = _address()
	receipt = ferrum_chem.export_document_molecule(
		observation, 0, observation.snapshot.digest, molecule_id,
		ferrum_chem.DocumentMoleculeExportFormat.canonical_smiles,
	)
	protected = tmp_path / "protected.smi"
	protected.write_text("protected molecule")
	cases = (
		(
			tmp_path / "existing.smi",
			lambda destination: destination.write_text("existing molecule"),
			ferrum_chem.PublicationNotStartedError,
		),
		(
			tmp_path / "symlink.smi",
			lambda destination: destination.symlink_to(protected),
			ferrum_chem.InvalidDestinationError,
		),
		(
			tmp_path / "hardlink.smi",
			lambda destination: destination.hardlink_to(protected),
			ferrum_chem.PublicationNotStartedError,
		),
	)
	for destination, establish, exception_type in cases:
		establish(destination)
		with pytest.raises(exception_type) as caught:
			ferrum_chem.publish_document_molecule_export(receipt, destination)
		assert caught.value.path == str(destination)
		if exception_type is ferrum_chem.PublicationNotStartedError:
			assert "validating the destination before temporary creation" in caught.value.reason
		else:
			assert caught.value.reason == "destination file must not be a symbolic link"
	assert protected.read_text() == "protected molecule"


#============================================
def test_every_selected_export_format_uses_the_same_frozen_receipt_contract() -> None:
	"""All public selected-root formats issue one provenance-carrying receipt type."""
	_session, observation, molecule_id = _address()
	formats = (
		ferrum_chem.DocumentMoleculeExportFormat.molfile_v2000,
		ferrum_chem.DocumentMoleculeExportFormat.molfile_v3000,
		ferrum_chem.DocumentMoleculeExportFormat.sdf_v2000,
		ferrum_chem.DocumentMoleculeExportFormat.sdf_v3000,
		ferrum_chem.DocumentMoleculeExportFormat.canonical_smiles,
		ferrum_chem.DocumentMoleculeExportFormat.inchi_standard,
		ferrum_chem.DocumentMoleculeExportFormat.inchi_fixed_hydrogen,
	)
	for format in formats:
		receipt = ferrum_chem.export_document_molecule(
			observation, observation.snapshot.revision, observation.snapshot.digest,
			molecule_id, format,
		)
		assert type(receipt) is ferrum_chem.DocumentMoleculeExport
		assert (receipt.source_revision, receipt.source_digest, receipt.molecule_id) == (
			observation.snapshot.revision, observation.snapshot.digest, molecule_id,
		)
		assert receipt.format is format
		assert isinstance(receipt.text, str) and receipt.text

"""Installed-extension checks for private canonical document SMILES V1."""

from pathlib import Path

import pytest

import ferrum_chem


_SOURCE = """\
<cdml xmlns="urn:ferrum:cdml" version="1.0"><molecule id="m1">
 <atom id="a1" name="N" charge="1" isotope="15" explicit_hydrogens="3">
  <point x="0" y="0"/>
 </atom>
 <atom id="a2" name="C"><point x="1" y="0"/></atom>
 <bond id="b1" start="a1" end="a2" type="n1"/>
</molecule></cdml>
"""


def _address(source: str = _SOURCE) -> tuple[object, object, str]:
	"""Return one session, frozen observation, and durable direct-root selector."""
	session = ferrum_chem.DocumentSession.load(source)
	observation = session.observe(0)
	return session, observation, observation.projection.molecules[0].id


def test_private_document_smiles_is_canonical_and_provenance_bound() -> None:
	"""The real packaged writer returns one immutable exact-source receipt."""
	session, observation, molecule_id = _address()
	before = session.snapshot()
	receipt = ferrum_chem.export_document_molecule_smiles_v1(
		observation, 0, observation.snapshot.digest, molecule_id,
	)

	assert type(receipt) is ferrum_chem.DocumentMoleculeSmilesV1
	assert receipt.schema == "ferrum-document-molecule-smiles-v1"
	assert receipt.profile == "canonical-isomeric-v1"
	assert receipt.source_revision == 0
	assert receipt.source_digest == observation.snapshot.digest
	assert receipt.molecule_id == molecule_id
	assert receipt.smiles == "C[15NH3+]"
	with pytest.raises(AttributeError):
		receipt.smiles = "changed"
	after = session.snapshot()
	assert (after.revision, after.digest, after.is_dirty) == (
		before.revision, before.digest, before.is_dirty,
	)


def test_private_document_smiles_publishes_exact_receipt_bytes(
		tmp_path: Path,
		) -> None:
	"""The Rust publisher writes one line without changing the source session."""
	session, observation, molecule_id = _address()
	receipt = ferrum_chem.export_document_molecule_smiles_v1(
		observation, 0, observation.snapshot.digest, molecule_id,
	)
	before = session.snapshot()
	destination = tmp_path / "molecule.smi"

	publication = ferrum_chem.publish_document_molecule_smiles_v1(
		receipt, destination,
	)

	assert type(publication) is ferrum_chem.DocumentMoleculeSmilesPublicationV1
	assert type(publication.directory_entry_confirmed) is bool
	assert destination.read_bytes() == b"C[15NH3+]\n"
	after = session.snapshot()
	assert (after.cdml, after.revision, after.digest, after.is_dirty) == (
		before.cdml, before.revision, before.digest, before.is_dirty,
	)


def test_private_document_smiles_refuses_a_symlink_destination(
		tmp_path: Path,
		) -> None:
	"""The private file route preserves an existing target behind a symlink."""
	_session, observation, molecule_id = _address()
	receipt = ferrum_chem.export_document_molecule_smiles_v1(
		observation, 0, observation.snapshot.digest, molecule_id,
	)
	target = tmp_path / "target.smi"
	target.write_bytes(b"preserved\n")
	destination = tmp_path / "linked.smi"
	destination.symlink_to(target)

	with pytest.raises(ferrum_chem.InvalidDestinationError) as rejected:
		ferrum_chem.publish_document_molecule_smiles_v1(receipt, destination)

	assert rejected.value.path == str(destination)
	assert "symbolic link" in rejected.value.reason
	assert target.read_bytes() == b"preserved\n"


def test_private_document_smiles_rejects_source_facts_before_native_export() -> None:
	"""Drawing styles, stale state, and non-root selectors remain typed failures."""
	styled = _SOURCE.replace('type="n1"', 'type="w1"')
	_session, observation, molecule_id = _address(styled)
	with pytest.raises(ferrum_chem.DocumentMoleculeSmilesError) as drawing:
		ferrum_chem.export_document_molecule_smiles_v1(
			observation, 0, observation.snapshot.digest, molecule_id,
		)
	assert "drawing style" in drawing.value.reason

	_session, observation, molecule_id = _address()
	with pytest.raises(ferrum_chem.DocumentMoleculeSmilesError) as stale:
		ferrum_chem.export_document_molecule_smiles_v1(
			observation, 1, observation.snapshot.digest, molecule_id,
		)
	assert "document changed" in stale.value.reason
	atom_id = "ferrum-document-object-v1/6d6f6c6563756c652f61746f6d/source/6131"
	with pytest.raises(ferrum_chem.DocumentMoleculeSmilesError) as root:
		ferrum_chem.export_document_molecule_smiles_v1(
			observation, 0, observation.snapshot.digest, atom_id,
		)
	assert "direct-root molecule" in root.value.reason


def test_private_document_smiles_maps_surrogate_text_to_its_error() -> None:
	"""Python strings that cannot encode as UTF-8 stay in this error contract."""
	_session, observation, molecule_id = _address()
	with pytest.raises(ferrum_chem.DocumentMoleculeSmilesError) as digest:
		ferrum_chem.export_document_molecule_smiles_v1(
			observation, 0, "\ud800", molecule_id,
		)
	assert digest.value.reason == "expected digest must be valid UTF-8 text"
	with pytest.raises(ferrum_chem.DocumentMoleculeSmilesError) as selector:
		ferrum_chem.export_document_molecule_smiles_v1(
			observation, 0, observation.snapshot.digest, "\ud800",
		)
	assert selector.value.reason == "molecule selector must be valid UTF-8 text"

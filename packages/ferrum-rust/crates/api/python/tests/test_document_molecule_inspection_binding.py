"""Installed-extension checks for private durable molecule inspection V1."""

import pytest

import ferrum_chem


_SOURCE = """\
<cdml xmlns="urn:ferrum:cdml" version="1.0"><molecule id="m1" name="Example">
 <atom id="a1" name="O" charge="-1"><point x="10" y="20"/></atom>
 <atom id="a2" name="C" charge="1"><point x="30" y="5"/></atom>
 <bond id="b1" start="a1" end="a2" type="w1"/>
</molecule></cdml>
"""


def _address(source: str = _SOURCE) -> tuple[object, object, str]:
	"""Return one frozen observation, snapshot, and durable root selector."""
	session = ferrum_chem.DocumentSession.load(source)
	observation = session.observe(0)
	return session, observation, observation.projection.molecules[0].document_object_id


def test_private_inspection_is_frozen_and_reports_exact_retained_facts() -> None:
	"""One drawing-style molecule crosses the private source-fact boundary."""
	session, observation, molecule_id = _address()
	before = session.snapshot()
	receipt = ferrum_chem.inspect_document_molecule_v1(
		observation, 0, observation.snapshot.digest, molecule_id,
	)

	assert receipt.schema == "ferrum-document-molecule-inspection-v1"
	assert receipt.source_revision == 0
	assert receipt.source_digest == observation.snapshot.digest
	assert receipt.molecule_id == molecule_id
	assert receipt.source_id == "m1"
	assert receipt.document_paint_order == 0
	assert receipt.authored_name == "Example"
	assert (receipt.atom_count, receipt.bond_count) == (2, 1)
	assert isinstance(receipt.element_inventory, tuple)
	assert [(entry.symbol, entry.atom_count) for entry in receipt.element_inventory] == [
		("C", 1), ("O", 1),
	]
	assert receipt.total_formal_charge == 0
	assert (receipt.bounds.min_x, receipt.bounds.min_y) == (10.0, 5.0)
	assert (receipt.bounds.max_x, receipt.bounds.max_y) == (30.0, 20.0)
	with pytest.raises(AttributeError):
		receipt.source_id = "changed"
	with pytest.raises(AttributeError):
		receipt.element_inventory[0].symbol = "N"
	assert session.snapshot().revision == before.revision
	assert session.snapshot().digest == before.digest
	assert session.snapshot().is_dirty is False


@pytest.mark.parametrize("digest", ["A" * 64, "a" * 63, "g" * 64])
def test_private_inspection_rejects_noncanonical_digests(digest: str) -> None:
	"""Digest input is exactly lowercase 64-hex before inspection starts."""
	_session, observation, molecule_id = _address()
	with pytest.raises(ferrum_chem.DocumentMoleculeInspectionError) as caught:
		ferrum_chem.inspect_document_molecule_v1(observation, 0, digest, molecule_id)
	assert "64 lowercase hexadecimal" in caught.value.reason


def test_private_inspection_maps_surrogate_text_to_dedicated_input_errors() -> None:
	"""Python strings that cannot encode as UTF-8 stay inside this error contract."""
	_session, observation, molecule_id = _address()
	with pytest.raises(ferrum_chem.DocumentMoleculeInspectionError) as digest:
		ferrum_chem.inspect_document_molecule_v1(observation, 0, "\ud800", molecule_id)
	assert digest.value.reason == "expected digest must be valid UTF-8 text"
	with pytest.raises(ferrum_chem.DocumentMoleculeInspectionError) as selector:
		ferrum_chem.inspect_document_molecule_v1(
			observation, 0, observation.snapshot.digest, "\ud800",
		)
	assert selector.value.reason == "molecule selector must be valid UTF-8 text"


def test_private_inspection_maps_object_and_rust_failures_to_its_error() -> None:
	"""Selector, stale, digest, root, and retained-source failures stay typed."""
	session, observation, molecule_id = _address()
	before = session.snapshot()
	with pytest.raises(ferrum_chem.DocumentMoleculeInspectionError) as malformed:
		ferrum_chem.inspect_document_molecule_v1(
			observation, 0, observation.snapshot.digest, "not-an-object-id",
		)
	assert "document object" in malformed.value.reason
	with pytest.raises(ferrum_chem.DocumentMoleculeInspectionError) as stale:
		ferrum_chem.inspect_document_molecule_v1(
			observation, 1, observation.snapshot.digest, molecule_id,
		)
	assert "document changed" in stale.value.reason
	with pytest.raises(ferrum_chem.DocumentMoleculeInspectionError) as digest:
		ferrum_chem.inspect_document_molecule_v1(observation, 0, "0" * 64, molecule_id)
	assert "digest changed" in digest.value.reason
	atom_id = observation.projection.molecules[0].atoms[0].document_object_id
	with pytest.raises(ferrum_chem.DocumentMoleculeInspectionError) as root:
		ferrum_chem.inspect_document_molecule_v1(
			observation, 0, observation.snapshot.digest, atom_id,
		)
	assert "direct-root molecule" in root.value.reason
	invalid_source = """\
<cdml xmlns="urn:ferrum:cdml" version="1.0"><molecule id="m1">
 <atom id="a1" name="Xx"><point x="0" y="0"/></atom>
</molecule></cdml>
"""
	_invalid_session, invalid_observation, invalid_id = _address(invalid_source)
	with pytest.raises(ferrum_chem.DocumentMoleculeInspectionError) as invalid:
		ferrum_chem.inspect_document_molecule_v1(
			invalid_observation, 0, invalid_observation.snapshot.digest, invalid_id,
		)
	assert "invalid element" in invalid.value.reason
	assert session.snapshot().revision == before.revision
	assert session.snapshot().digest == before.digest

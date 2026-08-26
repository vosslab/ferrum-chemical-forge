"""Behavioral checks for exact-revision document-molecule InChI export."""

# Standard Library
from pathlib import Path

# PIP3 modules
import pytest

# local repo modules
import ferrum_chem


_STYLED_SOURCE = """\
<cdml xmlns="urn:ferrum:cdml" version="1.0"><molecule id="m1">
 <atom id="a1" name="C"><point x="10" y="20"/></atom>
 <atom id="a2" name="H"><point x="30" y="20"/></atom>
 <bond id="b1" start="a1" end="a2" type="w1"/>
</molecule></cdml>
"""

_SOURCE = _STYLED_SOURCE.replace('type="w1"', 'type="n1"')


#============================================
def _address(source: str = _SOURCE) -> tuple[object, object, str]:
	"""Return one session, observation, and durable direct-root selector."""
	session = ferrum_chem.DocumentSession.load(source)
	observation = session.observe(0)
	return session, observation, observation.projection.molecules[0].document_object_id


#============================================
def test_unsupported_document_graph_is_rejected_before_packaged_adapter_loading() -> None:
	"""A drawing-only bond style cannot cross FFI or change the source session."""
	session = ferrum_chem.DocumentSession.load(_STYLED_SOURCE)
	observation = session.observe(0)
	molecule_id = observation.projection.molecules[0].document_object_id
	before = session.snapshot()

	with pytest.raises(
		ferrum_chem.UnsupportedDocumentMoleculeInchiError,
		match="cannot cross the native InChI boundary",
	) as captured:
		ferrum_chem.export_document_molecule_inchi_v1(
			observation, molecule_id, ferrum_chem.InchiModeV1.standard,
		)

	assert captured.value.reason == str(captured.value)
	assert session.snapshot().revision == before.revision
	assert session.snapshot().digest == before.digest
	assert session.snapshot().is_dirty is False


#============================================
def test_exact_inchi_receipt_publishes_one_line_without_document_effects(
		tmp_path: Path,
		) -> None:
	"""The real packaged engine and Rust publisher retain one exact receipt."""
	session, observation, molecule_id = _address()
	before = session.snapshot()
	receipt = ferrum_chem.export_document_molecule_inchi_v1(
		observation, molecule_id, ferrum_chem.InchiModeV1.standard,
	)
	destination = tmp_path / "molecule.inchi"

	publication = ferrum_chem.publish_document_molecule_inchi_v1(receipt, destination)

	assert type(publication) is ferrum_chem.DocumentMoleculeInchiPublicationV1
	assert type(publication.directory_entry_confirmed) is bool
	assert receipt.source_revision == before.revision
	assert receipt.source_digest == before.digest
	assert receipt.molecule_id == molecule_id
	assert receipt.mode is ferrum_chem.InchiModeV1.standard
	assert destination.read_bytes() == receipt.inchi.encode("ascii") + b"\n"
	after = session.snapshot()
	assert (after.cdml, after.revision, after.digest, after.is_dirty) == (
		before.cdml, before.revision, before.digest, before.is_dirty,
	)


#============================================
def test_inchi_selector_surrogate_stays_in_the_operation_error_contract() -> None:
	"""An unencodable Python selector never escapes as raw UnicodeEncodeError."""
	_session, observation, _molecule_id = _address()
	with pytest.raises(ferrum_chem.DocumentMoleculeInchiError) as invalid:
		ferrum_chem.export_document_molecule_inchi_v1(
			observation, "\ud800", ferrum_chem.InchiModeV1.standard,
		)
	assert invalid.value.reason == "molecule selector must be valid UTF-8 text"


#============================================

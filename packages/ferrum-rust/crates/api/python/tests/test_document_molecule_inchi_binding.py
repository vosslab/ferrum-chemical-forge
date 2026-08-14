"""Behavioral checks for exact-revision document-molecule InChI export."""

# PIP3 modules
import pytest

# local repo modules
import ferrum_chem


_STYLED_SOURCE = """\
<cdml version="1.0"><molecule id="m1">
 <atom id="a1" name="C"><point x="10" y="20"/></atom>
 <atom id="a2" name="H"><point x="30" y="20"/></atom>
 <bond id="b1" start="a1" end="a2" type="w1"/>
</molecule></cdml>
"""


#============================================
def test_unsupported_document_graph_is_rejected_before_packaged_adapter_loading() -> None:
	"""A drawing-only bond style cannot cross FFI or change the source session."""
	session = ferrum_chem.DocumentSession.load(_STYLED_SOURCE)
	observation = session.observe(0)
	molecule_id = observation.projection.molecules[0].id
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

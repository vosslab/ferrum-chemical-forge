"""Installed-extension checks for private native clipboard fragment extraction."""

import pytest

import ferrum_chem


_SOURCE = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07"><plus id="p"><point x="30" y="40"/></plus>
<molecule id="m" name="chain">
 <atom id="a" name="C"><point x="0" y="0"/></atom>
 <atom id="b" name="N"><point x="10" y="0"/></atom>
 <atom id="c" name="O"><point x="20" y="0"/></atom>
 <bond id="ab" start="a" end="b" type="n1"/>
 <bond id="bc" start="b" end="c" type="n1"/>
</molecule></cdml>
"""


def _observation() -> tuple[object, object]:
	"""Return one session and its immutable revision-zero observation."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	return session, session.observe(0)


def test_private_clipboard_binding_closes_bond_endpoints_without_mutation() -> None:
	"""A durable selected bond returns its exact connected partial molecule."""
	session, observation = _observation()
	before = session.snapshot()
	molecule = observation.projection.molecules[0]
	bond = molecule.bonds[1].id
	receipt = ferrum_chem.extract_document_clipboard_fragment_v1(
		observation, (bond,),
	)

	assert receipt.schema == "ferrum-document-clipboard-fragment-v1"
	assert receipt.source_revision == 0
	assert receipt.source_digest == observation.snapshot.digest
	assert receipt.kind == "structure"
	assert receipt.selected_objects == (bond,)
	assert receipt.copied_roots == (molecule.id,)
	assert receipt.copied_atoms == (molecule.atoms[1].id, molecule.atoms[2].id)
	assert receipt.copied_bonds == (bond,)
	assert 'id="a"' not in receipt.fragment_cdml
	assert 'id="ab"' not in receipt.fragment_cdml
	assert session.snapshot().revision == before.revision
	assert session.snapshot().digest == before.digest
	with pytest.raises(AttributeError):
		receipt.fragment_cdml = "changed"


def test_private_clipboard_binding_canonicalizes_mixed_selection_to_whole_roots() -> None:
	"""Mixed atom/artwork selection copies complete roots in source order."""
	_session, observation = _observation()
	molecule = observation.projection.molecules[0]
	atom = molecule.atoms[0].id
	plus = observation.projection.presentation_stack.roots[0].plus.target.id
	receipt = ferrum_chem.extract_document_clipboard_fragment_v1(
		observation, (atom, plus),
	)

	assert receipt.kind == "top_level"
	assert receipt.selected_objects == (plus, atom)
	assert receipt.copied_roots == (plus, molecule.id)
	assert 'id="a"' in receipt.fragment_cdml
	assert 'id="bc"' in receipt.fragment_cdml


def test_private_clipboard_binding_contains_invalid_python_and_rust_inputs() -> None:
	"""Malformed, surrogate, duplicate, and disconnected selections stay typed."""
	_session, observation = _observation()
	molecule = observation.projection.molecules[0]
	invalid_values = (
		([], "exact tuple"),
		((), "nonempty exact tuple"),
		(("\ud800",), "valid UTF-8 text"),
		(("not-an-object-id",), "document object"),
		((molecule.atoms[0].id, molecule.atoms[0].id), "must be unique"),
		((molecule.atoms[0].id, molecule.atoms[2].id), "must be connected"),
	)
	for selected, message in invalid_values:
		with pytest.raises(ferrum_chem.DocumentClipboardFragmentError) as caught:
			ferrum_chem.extract_document_clipboard_fragment_v1(observation, selected)
		assert message in caught.value.reason


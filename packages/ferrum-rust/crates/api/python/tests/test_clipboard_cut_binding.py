"""Installed-extension behavior checks for private native clipboard Cut."""

import pytest

import ferrum_chem


_SOURCE = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07"><plus id="p"><point x="30" y="40"/></plus>
<molecule id="m"><atom id="a" name="C"><point x="0" y="0"/></atom>
 <atom id="b" name="N"><point x="10" y="0"/></atom>
 <atom id="c" name="O"><point x="20" y="0"/></atom>
 <bond id="ab" start="a" end="b" type="n1"/>
 <bond id="bc" start="b" end="c" type="n1"/>
</molecule></cdml>
"""


def test_private_cut_prepares_fragment_then_commits_one_topology_edit() -> None:
	"""One source-authenticated plan carries Copy content and atomic deletion."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation = session.observe(0)
	selected = observation.projection.molecules[0].atoms[1].id
	plan = ferrum_chem.prepare_document_clipboard_cut_v1(
		observation, (selected,),
	)
	result = session.apply_clipboard_cut_v1(0, observation.snapshot.digest, plan)
	molecule = result.observation.projection.molecules[0]

	assert (
		plan.source_revision, plan.source_digest, plan.selected_objects,
	) == (0, observation.snapshot.digest, (selected,))
	assert (
		result.observation.snapshot.revision,
		tuple(atom.source_id for atom in molecule.atoms),
		len(molecule.bonds),
	) == (1, ("a", "c"), 0)


def test_private_cut_refuses_copy_fallback_with_partial_root_deletion() -> None:
	"""Mixed structure and artwork stays available to Copy but has no Cut meaning."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation = session.observe(0)
	atom = observation.projection.molecules[0].atoms[0].id
	plus = observation.projection.presentation_stack.roots[0].plus.target.id

	with pytest.raises(ferrum_chem.DocumentClipboardCutError) as caught:
		ferrum_chem.prepare_document_clipboard_cut_v1(observation, (atom, plus))

	assert "presentation roots only" in caught.value.reason

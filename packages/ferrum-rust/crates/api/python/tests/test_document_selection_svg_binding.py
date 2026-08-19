"""Installed-extension behavior for private selected-root native SVG."""

import math

import defusedxml.ElementTree
import pytest

import ferrum_chem


_SOURCE = """\
<cdml version="26.07"><plus id="p"><point x="40" y="20"/></plus>
<molecule id="near"><atom id="a" name="C"><point x="10" y="20"/></atom>
 <atom id="b" name="O"><point x="25" y="20"/></atom>
 <bond id="ab" start="a" end="b" type="n1"/></molecule>
<molecule id="far"><atom id="z" name="N"><point x="300" y="20"/></atom></molecule>
</cdml>
"""


def test_private_selected_svg_keeps_complete_roots_and_source_provenance() -> None:
	"""Atom and artwork selection produces two fitted roots without mutation."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation = session.observe(0)
	plus = observation.projection.presentation_stack.roots[0].plus.target.id
	atom = observation.projection.molecules[0].atoms[0].id
	before = session.snapshot()
	receipt = ferrum_chem.render_document_selection_svg_v1(
		observation, (atom, plus),
	)
	root = defusedxml.ElementTree.fromstring(receipt.svg)
	view_box = tuple(float(value) for value in root.attrib["viewBox"].split())

	assert (
		receipt.schema,
		receipt.source_revision,
		receipt.source_digest,
		receipt.selected_objects,
		len(receipt.selected_roots),
	) == (
		"ferrum-document-selection-svg-v1",
		0,
		observation.snapshot.digest,
		(plus, atom),
		2,
	)
	assert (
		all(math.isclose(actual, expected) for actual, expected in zip(
			view_box,
			(
				receipt.viewport.x, receipt.viewport.y,
				receipt.viewport.width, receipt.viewport.height,
			),
			strict=True,
		))
		and receipt.viewport.x + receipt.viewport.width < 100.0
		and session.snapshot().revision == before.revision
		and session.snapshot().digest == before.digest
	)


def test_private_selected_svg_withholds_a_profile_excluded_root() -> None:
	"""A selected root without native depiction returns its private typed reason."""
	session = ferrum_chem.DocumentSession.load(
		'<cdml><text id="t"><point x="10" y="20"/><font family="Arial"/>'
		'<ftext>label</ftext></text></cdml>',
	)
	observation = session.observe(0)
	selected = observation.projection.presentation_stack.roots[0].text.target.id

	with pytest.raises(ferrum_chem.DocumentSelectionSvgError) as caught:
		ferrum_chem.render_document_selection_svg_v1(observation, (selected,))

	assert "excluded by the native render profile" in caught.value.reason

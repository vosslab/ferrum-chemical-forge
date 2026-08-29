"""Installed-extension behavior for private selected-root native SVG."""

import math

import lxml.etree
import pytest

import ferrum_chem


_SOURCE = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07"><plus id="p"><point x="40" y="20"/></plus>
<molecule id="near"><atom id="a" name="C"><point x="10" y="20"/></atom>
 <atom id="b" name="O"><point x="25" y="20"/></atom>
 <bond id="ab" start="a" end="b" type="n1"/></molecule>
<molecule id="far"><atom id="z" name="N"><point x="300" y="20"/></atom></molecule>
</cdml>
"""
_XML_PARSER = lxml.etree.XMLParser(
	load_dtd=False,
	resolve_entities=False,
	no_network=True,
	huge_tree=False,
)


def test_private_selected_svg_keeps_complete_roots_and_source_provenance() -> None:
	"""Atom and artwork selection produces two fitted roots without mutation."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation = session.observe(0)
	plus = observation.projection.presentation_stack.entries[0].plus.target.document_object_id
	atom = observation.projection.molecules[0].atoms[0].document_object_id
	before = session.snapshot()
	receipt = ferrum_chem.render_document_selection_svg_v1(
		observation, (atom, plus),
	)
	root = lxml.etree.fromstring(receipt.svg.encode("utf-8"), parser=_XML_PARSER)
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
		(atom, plus),
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


def test_direct_unknown_font_is_refused_during_document_admission() -> None:
	"""A direct root cannot enter a session with an unsupported authored face."""
	with pytest.raises(ferrum_chem.DocumentLoadError):
		ferrum_chem.DocumentSession.load(
			'<cdml xmlns="urn:ferrum:cdml"><text id="t"><point x="10" y="20"/>'
			'<font family="Arial"/><ftext>label</ftext></text></cdml>',
		)

"""Hydrate presentation and reaction projection facts from CDML."""

# Standard Library
import xml.dom.minidom as dom

# local repo modules
import oasa.cdml_document

import ferrum_qt.bridge.oasa_bridge
import ferrum_qt.io.cdml_xml_helpers
import ferrum_qt.models.document_object


_SUPPORTED_MARK_TYPES = {
	"plus", "minus", "radical", "biradical", "electronpair",
	"dotted_electronpair", "pz_orbital",
}
_QT_OWNED_MARK_TYPES = {"atom_number"}


def _presentation(
		element: dom.Element, supported: bool,
		) -> ferrum_qt.models.document_object.PresentationObject:
	"""Create a presentation DTO while retaining every XML attribute/child."""
	attrs = ferrum_qt.io.cdml_xml_helpers._attributes(element)
	points = [ferrum_qt.io.cdml_xml_helpers._point_values(point) for point in ferrum_qt.io.cdml_xml_helpers._direct_children(element, "point")]
	bounds = ferrum_qt.io.cdml_xml_helpers._bounds_values(element)
	if bounds is None and ferrum_qt.io.cdml_xml_helpers._local_name(element) in ("text", "plus") and points:
		bounds = (points[0][0], points[0][1], 0.0, 0.0)
	font = ferrum_qt.io.cdml_xml_helpers._first_child(element, "font")
	ftext = ferrum_qt.io.cdml_xml_helpers._first_child(element, "ftext")
	formatted_text_runs, display_text = _ftext_projection_values(ftext, attrs)
	return ferrum_qt.models.document_object.PresentationObject(
		kind=ferrum_qt.io.cdml_xml_helpers._local_name(element),
		attributes=attrs,
		points=points, bounds=bounds,
		font_attributes=ferrum_qt.io.cdml_xml_helpers._attributes(font) if font is not None else {},
		xml_ftext=ferrum_qt.io.cdml_xml_helpers._inner_xml(ftext) if ftext is not None else None,
		formatted_text_runs=formatted_text_runs, display_text=display_text,
		raw_xml=ferrum_qt.io.cdml_xml_helpers._raw_xml(element), supported=supported,
	)


#============================================
def _presentation_from_description(
		record: oasa.cdml_document.CDMLPresentationRecord,
		) -> ferrum_qt.models.document_object.PresentationObject:
	"""Create a disposable presentation model from OASA's plain projection facts."""
	# OASA owns the accepted CDML.  Its supported ftext runs are the typed
	# backend projection facts, so reconstruct the frontend-only fragment from
	# them rather than reading or mutating a Qt text item.  Preservation-only
	# ftext remains ``None`` and is display-only by contract.
	xml_ftext = (
		ferrum_qt.bridge.oasa_bridge.encode_authored_ftext_runs(record.ftext_runs)
		if record.ftext_runs is not None else None
	)
	return ferrum_qt.models.document_object.PresentationObject(
		kind=record.kind,
		attributes=dict(record.attributes),
		points=list(record.points),
		bounds=record.bounds,
		xml_ftext=xml_ftext,
		formatted_text_runs=record.ftext_runs,
		display_text=record.display_text,
		font_attributes=dict(record.font_attributes),
		supported=record.disposition in {"editable", "display-only"},
		editable=record.disposition == "editable",
	)


#============================================
def _ftext_projection_values(
		ftext: dom.Element | None, attributes: dict[str, str],
		) -> tuple[tuple[tuple[str, tuple[str, ...]], ...] | None, str]:
	"""Return typed authored runs or preservation-safe display character data."""
	if ftext is None:
		text = attributes.get("text", "")
		return None, text
	character_data = _recursive_character_data(ftext)
	# Editable rich text permits authored character data only. Attributes make the
	# element preservation-only, even when its character data happens to decode.
	if ftext.hasAttributes():
		return None, character_data
	if any(
			child.nodeType not in (child.TEXT_NODE, child.CDATA_SECTION_NODE)
			for child in ftext.childNodes
		):
		return None, character_data
	authored = ferrum_qt.io.cdml_xml_helpers._element_text(ftext)
	runs = ferrum_qt.bridge.oasa_bridge.decode_authored_ftext_runs(authored)
	if runs is None:
		return None, character_data
	display_text = "".join(text for text, _styles in runs)
	return runs, display_text


#============================================
def _recursive_character_data(node: dom.Node) -> str:
	"""Collect rendered character data without assigning meaning to child markup."""
	parts = []
	for child in node.childNodes:
		if child.nodeType in (child.TEXT_NODE, child.CDATA_SECTION_NODE):
			parts.append(child.data)
		elif child.hasChildNodes():
			parts.append(_recursive_character_data(child))
	text = "".join(parts)
	return text


#============================================
def _reaction(element: dom.Element) -> ferrum_qt.models.document_object.ReactionRecord:
	"""Read ordered reaction references without changing their XML order."""
	refs: list[tuple[str, str]] = []
	for child in ferrum_qt.io.cdml_xml_helpers._element_children(element):
		refs.append((ferrum_qt.io.cdml_xml_helpers._local_name(child), child.getAttribute("idref")))
	return ferrum_qt.models.document_object.ReactionRecord(
		refs=refs, raw_xml=ferrum_qt.io.cdml_xml_helpers._raw_xml(element),
	)


#============================================
def _parse_marks(
		document: ferrum_qt.models.document.Document, molecule_el: dom.Element,
		atom_lookup: dict[str, object],
		unsupported: list[ferrum_qt.models.document_object.UnsupportedContent],
		molecule_position: int,
		) -> None:
	"""Read atom marks and keep unsupported mark XML explicitly visible."""
	for atom_position, atom_el in enumerate(ferrum_qt.io.cdml_xml_helpers._direct_children(molecule_el, "atom"), start=1):
		atom_id = atom_el.getAttribute("id")
		atom_model = atom_lookup.get(atom_id)
		matching_mark_counts: dict[str, int] = {}
		for mark_position, mark_el in enumerate(
				ferrum_qt.io.cdml_xml_helpers._direct_core_cdml_children(
					atom_el, "mark",
				),
				start=1,
				):
			attrs = ferrum_qt.io.cdml_xml_helpers._attributes(mark_el)
			mark_type = attrs.get("type", "")
			matching_mark_index = matching_mark_counts.get(mark_type, 0)
			matching_mark_counts[mark_type] = matching_mark_index + 1
			if mark_type in _QT_OWNED_MARK_TYPES:
				continue
			if mark_type not in _SUPPORTED_MARK_TYPES:
				unsupported.append(ferrum_qt.io.cdml_xml_helpers._unsupported(
						mark_el, "unsupported atom mark",
						"/cdml/molecule[%d]/atom[%d]/mark[%d]" % (
								molecule_position, atom_position, mark_position,
								),
						))
				continue
			if atom_model is None:
				unsupported.append(ferrum_qt.io.cdml_xml_helpers._unsupported(
						mark_el, "unsupported atom mark",
						"/cdml/molecule[%d]/atom[%d]/mark[%d]" % (
								molecule_position, atom_position, mark_position,
								),
						))
				continue
			mark = ferrum_qt.models.document_object.AtomMarkModel(
				atom_model=atom_model, attributes=attrs, raw_xml=ferrum_qt.io.cdml_xml_helpers._raw_xml(mark_el),
				matching_mark_index=matching_mark_index,
			)
			document.add_mark(mark, mark_dirty=False)


#============================================

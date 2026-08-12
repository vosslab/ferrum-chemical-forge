"""Low-level CDML XML helpers shared by compatibility hydration stages."""

# Standard Library
import xml.dom.minidom as dom

# local repo modules
import oasa.cdml_document
import oasa.cdml_writer
import oasa.cdml_xml
import oasa.safe_xml

import ferrum_qt.models.document_object


def _unsupported(
		element: dom.Element, reason: str, path: str,
		) -> ferrum_qt.models.document_object.UnsupportedContent:
	"""Create one legacy-isolated warning with its compatibility XML payload."""
	attrs = _attributes(element)
	return ferrum_qt.models.document_object.UnsupportedContent(
		path=path, tag=_local_name(element),
		object_id=attrs.get("id"), reason=reason,
		raw_xml=_raw_xml(element),
	)


#============================================
def _unsupported_from_presentation_issue(
		issue: oasa.cdml_document.CDMLPresentationIssue,
		) -> ferrum_qt.models.document_object.UnsupportedContent:
	"""Create a Qt warning from OASA facts without retaining authoritative XML."""
	return ferrum_qt.models.document_object.UnsupportedContent(
		path=issue.path, tag=issue.tag, object_id=issue.identifier,
		reason=issue.reason, raw_xml="",
	)


#============================================
#============================================
def _all_attributes(element: dom.Element | None) -> dict[str, str]:
	"""Return every XML attribute, including namespace declarations."""
	if element is None:
		return {}
	return {element.attributes.item(index).name: element.attributes.item(index).value
			for index in range(element.attributes.length)}


#============================================
def _attributes(element: dom.Element | None) -> dict[str, str]:
	"""Return model attributes without serializer-only namespace declarations."""
	return {name: value for name, value in _all_attributes(element).items()
			if name != "xmlns" and not name.startswith("xmlns:")}


#============================================
def _element_children(element: dom.Element) -> list[dom.Element]:
	"""Return direct element children, excluding whitespace and text nodes."""
	return [child for child in element.childNodes
			if child.nodeType == child.ELEMENT_NODE]


#============================================
def _direct_children(element: dom.Element, tag: str) -> list[dom.Element]:
	"""Return direct element children whose local CDML tag matches."""
	return [child for child in _element_children(element) if _local_name(child) == tag]


#============================================
def _direct_core_cdml_children(element: dom.Element, tag: str) -> list[dom.Element]:
	"""Return direct editable core CDML children with one semantic name."""
	return [
		child for child in _element_children(element)
		if _local_name(child) == tag and _is_direct_core_cdml_child(child)
	]


#============================================
def _local_name(element: dom.Element) -> str:
	"""Return a CDML element's semantic name independently of its prefix."""
	return element.localName or element.tagName.rsplit(":", maxsplit=1)[-1]


#============================================
def _is_direct_core_cdml_child(element: dom.Element) -> bool:
	"""Return whether one direct root child is editable CDML, not foreign XML."""
	return (
		element.namespaceURI in (None, "", oasa.cdml_document.CDML_NAMESPACE_URI)
		and _local_name(element) in oasa.cdml_xml.CDML_CORE_ELEMENT_NAMES
	)


#============================================
def _first_child(element: dom.Element, tag: str) -> dom.Element | None:
	"""Return the first direct matching CDML child, if present."""
	children = _direct_children(element, tag)
	return children[0] if children else None


#============================================
def _element_text(element: dom.Element) -> str:
	"""Return direct text content without interpreting embedded XML."""
	return "".join(child.data for child in element.childNodes
				if child.nodeType in (child.TEXT_NODE, child.CDATA_SECTION_NODE))


#============================================
def _raw_xml(element: dom.Element) -> str:
	"""Serialize an element with namespace declarations inherited from parents."""
	result = dom.Document()
	copy = result.importNode(element, deep=True)
	for name, value in _in_scope_namespace_attributes(element).items():
		if not copy.hasAttribute(name):
			copy.setAttribute(name, value)
	result.appendChild(copy)
	return copy.toxml()


#============================================
def _inner_xml(element: dom.Element | None) -> str:
	"""Serialize child XML that remains valid outside its original ancestor."""
	if element is None:
		return ""
	return "".join(
			_raw_inner_xml_element(child)
			if child.nodeType == child.ELEMENT_NODE else child.toxml()
			for child in element.childNodes
		)


#============================================
def _raw_inner_xml_element(element: dom.Element) -> str:
	"""Serialize one formatted-text child without adding the CDML default XMLNS."""
	result = dom.Document()
	copy = result.importNode(element, deep=True)
	inherited = _in_scope_namespace_attributes(element)

	# The saved child will be inserted under ``ftext`` in the CDML default
	# namespace.  Adding that inherited declaration to a simple <b> or <i>
	# changes the exposed formatted-text string on every load/save cycle.
	if (copy.hasAttribute("xmlns")
			and copy.getAttribute("xmlns") == inherited.get("xmlns")):
		copy.removeAttribute("xmlns")

	# Prefix declarations remain necessary because the formatted-text fragment is
	# parsed beneath a temporary namespace-free wrapper before it is reinserted.
	for name, value in inherited.items():
		if name == "xmlns" or copy.hasAttribute(name):
			continue
		copy.setAttribute(name, value)
	result.appendChild(copy)
	text = copy.toxml()
	return text


#============================================
def _in_scope_namespace_attributes(element: dom.Element) -> dict[str, str]:
	"""Return the namespace declarations visible from ``element``'s parents."""
	ancestors: list[dom.Element] = []
	parent = element.parentNode
	while isinstance(parent, dom.Element):
		ancestors.append(parent)
		parent = parent.parentNode
	attributes: dict[str, str] = {}
	for ancestor in reversed(ancestors):
		for name, value in _all_attributes(ancestor).items():
			if name == "xmlns" or name.startswith("xmlns:"):
				attributes[name] = value
	return attributes


#============================================
def _import_raw(result: dom.Document, raw: str) -> dom.Element:
	"""Import a raw fragment while removing declarations inherited from output."""
	parsed = oasa.safe_xml.parse_dom_from_string(raw)
	imported = result.importNode(parsed.documentElement, deep=True)
	root = result.documentElement
	root_namespaces = {
			name: value for name, value in _all_attributes(root).items()
			if name == "xmlns" or name.startswith("xmlns:")
		}
	_strip_redundant_namespace_declarations(imported, root_namespaces)
	return imported


#============================================
def _strip_redundant_namespace_declarations(
		element: dom.Element, inherited: dict[str, str],
		) -> None:
	"""Drop synthetic declarations already supplied by the output CDML root."""
	visible = dict(inherited)
	for name, value in list(_all_attributes(element).items()):
		if name != "xmlns" and not name.startswith("xmlns:"):
			continue
		if visible.get(name) == value:
			element.removeAttribute(name)
		else:
			visible[name] = value
	for child in _element_children(element):
		_strip_redundant_namespace_declarations(child, visible)


#============================================
def _replace_inner_xml(result: dom.Document, element: dom.Element, inner: str) -> None:
	"""Replace children from a fragment, retaining only needed namespaces."""
	for child in list(element.childNodes):
		element.removeChild(child)
	if inner:
		wrapper = oasa.safe_xml.parse_dom_from_string("<wrapper>%s</wrapper>" % inner)
		root = result.documentElement
		root_namespaces = {
				name: value for name, value in _all_attributes(root).items()
				if name == "xmlns" or name.startswith("xmlns:")
		}
		for child in wrapper.documentElement.childNodes:
			imported = result.importNode(child, deep=True)
			if isinstance(imported, dom.Element):
				_strip_redundant_namespace_declarations(imported, root_namespaces)
			element.appendChild(imported)


#============================================
def _coord_to_points(value: str | None) -> float:
	"""Convert CDML centimetres to points while retaining raw pixel values."""
	if not value:
		return 0.0
	text = str(value).strip()
	if text.endswith("cm"):
		return float(text[:-2]) * oasa.cdml_writer.POINTS_PER_CM
	if text.endswith("px"):
		text = text[:-2]
	return float(text)


#============================================
def _point_values(element: dom.Element) -> tuple[float, float, float | None]:
	"""Return a CDML point's coordinates converted to scene points."""
	x = _coord_to_points(element.getAttribute("x"))
	y = _coord_to_points(element.getAttribute("y"))
	z_text = element.getAttribute("z")
	return (x, y, _coord_to_points(z_text) if z_text else None)


#============================================
def _bounds_values(
		element: dom.Element,
		) -> tuple[float, float, float, float] | None:
	"""Return CDML bounds as scene x, y, width, and height when complete."""
	if not all(element.hasAttribute(name) for name in ("x1", "y1", "x2", "y2")):
		return None
	x1, y1, x2, y2 = tuple(_coord_to_points(element.getAttribute(name))
			for name in ("x1", "y1", "x2", "y2"))
	return (x1, y1, x2 - x1, y2 - y1)


#============================================
def _px_to_cm_text(value: float | None) -> str:
	"""Convert scene points to canonical CDML centimetre text."""
	points = 0.0 if value is None else float(value)
	return "%.3fcm" % (points / oasa.cdml_writer.POINTS_PER_CM)

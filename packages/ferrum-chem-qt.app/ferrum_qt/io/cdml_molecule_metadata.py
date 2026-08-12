"""Hydrate molecule-attached CDML metadata and retained compatibility content."""

# Standard Library
import xml.dom.minidom as dom

# local repo modules
import oasa.cdml_document

import ferrum_qt.io.cdml_xml_helpers
import ferrum_qt.models.document
import ferrum_qt.models.document_object
import ferrum_qt.models.fragment_model
import ferrum_qt.models.group_model
import ferrum_qt.models.molecule_model


_SUPPORTED_MARK_TYPES = {
	"plus", "minus", "radical", "biradical", "electronpair",
	"dotted_electronpair", "pz_orbital",
}


def _hydrate_atom_mark_observation(
		document: ferrum_qt.models.document.Document, molecule_el: dom.Element,
		atoms_by_source_position: dict[int, object], unsupported: list,
		molecule_position: int, records_by_position: dict,
		) -> None:
	"""Hydrate backend mark facts by their root-scoped source association."""
	for atom_position, atom_el in enumerate(ferrum_qt.io.cdml_xml_helpers._element_children(molecule_el), 1):
		if ferrum_qt.io.cdml_xml_helpers._local_name(atom_el) != "atom":
			continue
		atom_model = atoms_by_source_position.get(atom_position)
		for mark_position, mark_el in enumerate(list(ferrum_qt.io.cdml_xml_helpers._element_children(atom_el)), 1):
			if ferrum_qt.io.cdml_xml_helpers._local_name(mark_el) != "mark":
				continue
			record = records_by_position.get((
				molecule_position, atom_position, mark_position,
			))
			if record is not None and record.mark_type is not None and atom_model is not None:
				mark = ferrum_qt.models.document_object.AtomMarkModel(
					atom_model, {"type": record.mark_type}, raw_xml=None,
					supported=record.mark_type in _SUPPORTED_MARK_TYPES,
					matching_mark_index=record.same_type_ordinal,
					rendering_facts=(
						record.angle_degrees, record.radial_offset_pt, record.size_pt,
						record.draw_circle, record.line_width_pt,
					),
				)
				document.add_mark(mark, mark_dirty=False)
			if record is not None and record.disposition != "editable":
				unsupported.append(ferrum_qt.models.document_object.UnsupportedContent(
					"mark", None,
					"/cdml/molecule[%d]/atom[%d]/mark[%d]" % (
						molecule_position, atom_position, mark_position,
					), record.reason or "atom mark is display-only", "",
				))
			atom_el.removeChild(mark_el)


#============================================
def _load_atom_number_fields(
		mol_model: ferrum_qt.models.molecule_model.MoleculeModel,
		molecule_el: dom.Element,
		) -> None:
	"""Load legacy atom ``number`` and ``show_number`` presentation fields."""
	for atom_el, atom_model in zip(
			ferrum_qt.io.cdml_xml_helpers._direct_children(molecule_el, "atom"), mol_model.atoms,
		):
		number_text = atom_el.getAttribute("number")
		if number_text:
			atom_model.number = int(number_text)
		show_number = atom_el.getAttribute("show_number")
		if show_number:
			atom_model.show_number = show_number in ("yes", "true", "1", "on")


#============================================
def _parse_fragments(
		mol_model: ferrum_qt.models.molecule_model.MoleculeModel,
		molecule_el: dom.Element,
		unsupported: list[ferrum_qt.models.document_object.UnsupportedContent],
		molecule_position: int,
		) -> None:
	"""Read safely representable fragments without evaluating legacy values."""
	for fragment_position, fragment_el in enumerate(
				ferrum_qt.io.cdml_xml_helpers._direct_children(molecule_el, "fragment"), start=1,
		):
		fragment = _read_fragment(fragment_el)
		path = "/cdml/molecule[%d]/fragment[%d]" % (
				molecule_position, fragment_position,
		)
		if fragment is None:
			mol_model.retain_unsupported_fragment_xml(ferrum_qt.io.cdml_xml_helpers._raw_xml(fragment_el))
			unsupported.append(ferrum_qt.io.cdml_xml_helpers._unsupported(
					fragment_el, "fragment has no stable identifier", path,
					))
			continue
		if mol_model.can_add_fragment(fragment):
			mol_model.add_fragment(fragment)
		else:
			# A dangling or duplicate reference is retained as raw XML so it is
			# visible to callers and never becomes partly editable metadata.
			mol_model.retain_unsupported_fragment_xml(ferrum_qt.io.cdml_xml_helpers._raw_xml(fragment_el))
			unsupported.append(ferrum_qt.io.cdml_xml_helpers._unsupported(
					fragment_el, "fragment has unresolved references", path,
					))


#============================================
def _hydrate_fragment_metadata(
		mol_model: ferrum_qt.models.molecule_model.MoleculeModel,
		records: tuple[oasa.cdml_document.CDMLFragmentMetadataRecord, ...] | list,
		) -> None:
	"""Apply only OASA-approved fragment facts to one disposable molecule model."""
	for record in records:
		label = record.display_name or record.fragment_id or "Unnamed fragment"
		if record.disposition == "editable":
			if record.fragment_id is None or record.fragment_type is None:
				mol_model.add_fragment_notice("%s is read-only." % label)
				continue
			fragment = ferrum_qt.models.fragment_model.FragmentModel(
				record.fragment_id, record.fragment_type, label,
				record.atom_ids, record.bond_ids,
			)
			if mol_model.can_add_fragment(fragment):
				mol_model.add_fragment(fragment)
			else:
				mol_model.add_fragment_notice("%s is read-only." % label)
		else:
			reason = record.reason or "fragment metadata is read-only"
			mol_model.add_fragment_notice("%s: %s." % (label, reason.rstrip(".")))


#============================================
def _remove_fragment_source_children(molecule_el: dom.Element) -> None:
	"""Remove all direct fragment lookalikes from synchronized retained source XML."""
	for child in tuple(ferrum_qt.io.cdml_xml_helpers._element_children(molecule_el)):
		if ferrum_qt.io.cdml_xml_helpers._local_name(child) == "fragment":
			molecule_el.removeChild(child)


#============================================
def _hydrate_group_observation(
		mol_model: ferrum_qt.models.molecule_model.MoleculeModel,
		molecule_el: dom.Element, unsupported: list, molecule_position: int,
		records_by_position: dict,
		) -> None:
	"""Hydrate groups from OASA facts using only transient source XML."""
	group_ids = set()
	for group_position, group_el in enumerate(list(ferrum_qt.io.cdml_xml_helpers._element_children(molecule_el)), 1):
		if ferrum_qt.io.cdml_xml_helpers._local_name(group_el) != "group":
			continue
		if group_el.getAttribute("id"):
			group_ids.add(group_el.getAttribute("id"))
		record = records_by_position.get((molecule_position, group_position))
		if record is not None and record.x_pt is not None and record.y_pt is not None:
			font_attributes = tuple((key, value) for key, value in (
				("family", record.font_family),
				("size", None if record.font_size_pt is None else str(record.font_size_pt)),
			) if value is not None)
			group = ferrum_qt.models.group_model.GroupModel(
				record.group_id or "", record.name or "", record.group_type or "", record.pos or "center-first",
				record.x_pt, record.y_pt, (), (), font_attributes, None,
				unsupported_reason=record.reason,
			)
			group.implicit_expandable = record.implicit_expandable
			mol_model.add_group(group)
		if record is not None and record.disposition != "selectable":
			unsupported.append(ferrum_qt.models.document_object.UnsupportedContent(
				"group", None, "/cdml/molecule[%d]/group[%d]" % (
					molecule_position, group_position,
				), record.reason or "group is display-only", "",
			))
		molecule_el.removeChild(group_el)
	for bond_el in tuple(ferrum_qt.io.cdml_xml_helpers._element_children(molecule_el)):
		if ferrum_qt.io.cdml_xml_helpers._local_name(bond_el) == "bond" and (
			bond_el.getAttribute("start") in group_ids or bond_el.getAttribute("end") in group_ids
		):
			molecule_el.removeChild(bond_el)


#============================================
def _parse_groups(
		mol_model: ferrum_qt.models.molecule_model.MoleculeModel,
		molecule_el: dom.Element,
		unsupported: list[ferrum_qt.models.document_object.UnsupportedContent],
		molecule_position: int,
		) -> None:
	"""Load CDML groups structurally while retaining every source XML node.

	The OASA CDML codec omits group vertices.  This frontend layer therefore
	reads only their display/attachment metadata and lets the source-envelope
	merge retain group and incident-bond XML unchanged on save.
	"""
	groups_by_id = {}
	for group_position, group_el in enumerate(ferrum_qt.io.cdml_xml_helpers._direct_children(molecule_el, "group"), start=1):
		group, reason = _read_group(group_el)
		path = "/cdml/molecule[%d]/group[%d]" % (molecule_position, group_position)
		if group is None:
			unsupported.append(ferrum_qt.io.cdml_xml_helpers._unsupported(group_el, reason or "invalid group", path))
			continue
		mol_model.add_group(group)
		groups_by_id[group.group_id] = group
		if not group.supported:
			unsupported.append(ferrum_qt.io.cdml_xml_helpers._unsupported(group_el, group.unsupported_reason or "unsupported group", path))
	attachments = {group_id: [] for group_id in groups_by_id}
	for bond_el in ferrum_qt.io.cdml_xml_helpers._direct_children(molecule_el, "bond"):
		start_id = bond_el.getAttribute("start")
		end_id = bond_el.getAttribute("end")
		for group_id in (start_id, end_id):
			if group_id not in attachments:
				continue
			attachments[group_id].append(ferrum_qt.models.group_model.GroupAttachment(
					bond_id=bond_el.getAttribute("id"), start_id=start_id, end_id=end_id,
					attributes=tuple(sorted(ferrum_qt.io.cdml_xml_helpers._attributes(bond_el).items())), raw_xml=ferrum_qt.io.cdml_xml_helpers._raw_xml(bond_el),
					))
	for group_id, group in groups_by_id.items():
		group.set_attachments(tuple(attachments[group_id]))


#============================================
def _read_group(
		element: dom.Element,
		) -> tuple[ferrum_qt.models.group_model.GroupModel | None, str | None]:
	"""Decode the narrow native-group projection without interpreting chemistry."""
	group_id = element.getAttribute("id")
	if not group_id:
		return None, "group has no stable identifier"
	points = ferrum_qt.io.cdml_xml_helpers._direct_children(element, "point")
	if len(points) != 1:
		return None, "group must have exactly one point"
	point = points[0]
	try:
		x = ferrum_qt.io.cdml_xml_helpers._coord_to_points(point.getAttribute("x"))
		y = ferrum_qt.io.cdml_xml_helpers._coord_to_points(point.getAttribute("y"))
	except ValueError:
		return None, "group point has invalid coordinates"
	group_type = element.getAttribute("group-type")
	unsupported_reason = None
	if group_type not in {"builtin", "implicit", "explicit"}:
		unsupported_reason = "unsupported group-type %r" % (group_type or "missing")
	allowed_children = {"point", "font"}
	if any(ferrum_qt.io.cdml_xml_helpers._local_name(child) not in allowed_children for child in ferrum_qt.io.cdml_xml_helpers._element_children(element)):
		unsupported_reason = "group has retained child content not editable by PySide6"
	font = ferrum_qt.io.cdml_xml_helpers._first_child(element, "font")
	group = ferrum_qt.models.group_model.GroupModel(
			group_id=group_id, name=element.getAttribute("name"), group_type=group_type,
			pos=element.getAttribute("pos") or "center-first", x=x, y=y,
			attributes=tuple(sorted(ferrum_qt.io.cdml_xml_helpers._attributes(element).items())),
			point_attributes=tuple(sorted(ferrum_qt.io.cdml_xml_helpers._attributes(point).items())),
			font_attributes=tuple(sorted(ferrum_qt.io.cdml_xml_helpers._attributes(font).items())) if font is not None else (),
			raw_xml=ferrum_qt.io.cdml_xml_helpers._raw_xml(element), unsupported_reason=unsupported_reason,
			)
	return group, None


#============================================
def _read_fragment(element: dom.Element) -> ferrum_qt.models.fragment_model.FragmentModel | None:
	"""Decode one CDML fragment into stable, text-only metadata."""
	fragment_id = element.getAttribute("id")
	if not fragment_id:
		return None
	properties = []
	for property_el in ferrum_qt.io.cdml_xml_helpers._direct_children(element, "property"):
		properties.append(ferrum_qt.models.fragment_model.FragmentProperty(
				name=property_el.getAttribute("name"),
				value=property_el.getAttribute("value"),
				type_name=property_el.getAttribute("type"),
				attributes=tuple(sorted(ferrum_qt.io.cdml_xml_helpers._attributes(property_el).items())),
				raw_xml=ferrum_qt.io.cdml_xml_helpers._raw_xml(property_el),
				))
	known_children = {"name", "bond", "vertex", "property"}
	unknown = tuple(ferrum_qt.io.cdml_xml_helpers._raw_xml(child) for child in ferrum_qt.io.cdml_xml_helpers._element_children(element)
					if ferrum_qt.io.cdml_xml_helpers._local_name(child) not in known_children)
	name_el = ferrum_qt.io.cdml_xml_helpers._first_child(element, "name")
	name = ferrum_qt.io.cdml_xml_helpers._element_text(name_el) if name_el is not None else ""
	return ferrum_qt.models.fragment_model.FragmentModel(
			fragment_id=fragment_id,
			fragment_type=element.getAttribute("type") or "explicit",
			name=name,
			atom_ids=tuple(vertex.getAttribute("id")
						for vertex in ferrum_qt.io.cdml_xml_helpers._direct_children(element, "vertex")),
			bond_ids=tuple(bond.getAttribute("id")
						for bond in ferrum_qt.io.cdml_xml_helpers._direct_children(element, "bond")),
			properties=tuple(properties),
			attributes=tuple(sorted(ferrum_qt.io.cdml_xml_helpers._attributes(element).items())),
			unknown_children_xml=unknown,
			raw_xml=ferrum_qt.io.cdml_xml_helpers._raw_xml(element),
			)


#============================================
def _report_unrendered_molecule_children(
		molecule_el: dom.Element,
		unsupported: list[ferrum_qt.models.document_object.UnsupportedContent],
		molecule_position: int,
		) -> None:
	"""Report preserved molecule vertices PySide6 cannot yet project."""
	for child_position, child in enumerate(ferrum_qt.io.cdml_xml_helpers._element_children(molecule_el), start=1):
		if ferrum_qt.io.cdml_xml_helpers._local_name(child) in {"atom", "bond", "fragment", "group"}:
			continue
		unsupported.append(ferrum_qt.io.cdml_xml_helpers._unsupported(
				child, "molecule child retained but not rendered by PySide6",
				"/cdml/molecule[%d]/%s[%d]" % (
				molecule_position, ferrum_qt.io.cdml_xml_helpers._local_name(child), child_position,
						),
				))


#============================================
def _bond_has_only_model_atoms(
		bond_el: dom.Element, source_atom_ids: set[str],
		) -> bool:
	"""Return whether a source bond has two OASA-modelled atom endpoints."""
	has_model_atoms = (
			bond_el.getAttribute("start") in source_atom_ids
			and bond_el.getAttribute("end") in source_atom_ids
		)
	return has_model_atoms


#============================================
def _normalize_loaded_source_ids(
		mol_model: ferrum_qt.models.molecule_model.MoleculeModel,
		source: dom.Element,
		) -> None:
	"""Assign local linkage IDs while loading an ID-less legacy molecule.

	Legacy CDML is allowed to omit atom and bond IDs.  Qt needs private IDs to
	link the decoded models with retained source XML, but those IDs are not
	issued by the authoritative backend and therefore cannot name a persistent
	operation target.  Serialization itself is read-only and must instead
	allocate output-only IDs.
	"""
	atom_ids = {child.getAttribute("id") for child in ferrum_qt.io.cdml_xml_helpers._element_children(source)
				if child.getAttribute("id")}
	bond_ids = {child.getAttribute("id") for child in ferrum_qt.io.cdml_xml_helpers._direct_children(source, "bond")
				if child.getAttribute("id")}
	source_atoms = ferrum_qt.io.cdml_xml_helpers._direct_children(source, "atom")
	source_atoms_by_id = {
			atom.getAttribute("id"): atom for atom in source_atoms
			if atom.getAttribute("id")
	}
	for atom_model in mol_model.atoms:
		atom_id = atom_model.atom_id
		source_atom = source_atoms_by_id.get(str(atom_id)) if atom_id else None
		atom_model.bind_backend_durable_id(
			source_atom.getAttribute("id") if source_atom is not None else None,
		)
	for atom_model in mol_model.atoms:
		atom_id = atom_model.atom_id
		source_atom = source_atoms_by_id.get(str(atom_id)) if atom_id else None
		if source_atom is None:
			continue
		atom_ids.add(str(atom_id))

	# Pair only genuinely ID-less loaded atoms by position.  Never zip a newly
	# created atom onto an unmatched, deleted source atom with a real ID.
	idless_source_atoms = [atom for atom in source_atoms
			if not atom.getAttribute("id")]
	idless_current_atoms = [atom for atom in mol_model.atoms
			if atom.atom_id is None]
	for atom_model, source_atom in zip(idless_current_atoms, idless_source_atoms):
		fresh_id = _next_free_id("atom", atom_ids)
		atom_model.atom_id = fresh_id
		source_atom.setAttribute("id", fresh_id)
		atom_ids.add(fresh_id)
	for atom_model in mol_model.atoms:
		if atom_model.atom_id is not None:
			continue
		fresh_id = _next_free_id("atom", atom_ids)
		atom_model.atom_id = fresh_id
		atom_ids.add(fresh_id)

	source_atom_ids = {atom.getAttribute("id") for atom in source_atoms}
	source_bonds = [
			bond for bond in ferrum_qt.io.cdml_xml_helpers._direct_children(source, "bond")
			if _bond_has_only_model_atoms(bond, source_atom_ids)
		]
	source_bonds_by_id = {
			bond.getAttribute("id"): bond for bond in source_bonds
			if bond.getAttribute("id")
	}
	for bond_model in mol_model.bonds:
		bond_id = bond_model.bond_id
		source_bond = source_bonds_by_id.get(str(bond_id)) if bond_id else None
		bond_model.bind_backend_durable_id(
			source_bond.getAttribute("id") if source_bond is not None else None,
		)
	for bond_model in mol_model.bonds:
		bond_id = bond_model.bond_id
		if bond_id and str(bond_id) in source_bonds_by_id:
			bond_ids.add(str(bond_id))
	idless_source_bonds = [bond for bond in source_bonds
			if not bond.getAttribute("id")]
	idless_current_bonds = [bond for bond in mol_model.bonds
			if not bond.bond_id]
	for bond_model, source_bond in zip(idless_current_bonds, idless_source_bonds):
		fresh_id = _next_free_id("bond", bond_ids)
		bond_model.bond_id = fresh_id
		source_bond.setAttribute("id", fresh_id)
		bond_ids.add(fresh_id)
	for bond_model in mol_model.bonds:
		if bond_model.bond_id:
			continue
		fresh_id = _next_free_id("bond", bond_ids)
		bond_model.bond_id = fresh_id
		bond_ids.add(fresh_id)


#============================================
def _next_free_id(prefix: str, reserved: set[str]) -> str:
	"""Return the next unused stable CDML identifier for an edited object."""
	index = 1
	candidate = "%s%d" % (prefix, index)
	while candidate in reserved:
		index += 1
		candidate = "%s%d" % (prefix, index)
	return candidate


#============================================

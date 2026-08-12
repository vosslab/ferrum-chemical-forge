"""Coordinate CDML compatibility and synchronized document hydration."""

# Standard Library
import xml.dom.minidom as dom

# local repo modules
import oasa.cdml_document
import oasa.safe_xml

import ferrum_qt.io.cdml_molecule_hydration
import ferrum_qt.io.cdml_molecule_metadata
import ferrum_qt.io.cdml_presentation_hydration
import ferrum_qt.io.cdml_xml_helpers
import ferrum_qt.models.document
import ferrum_qt.models.document_object


_DRAWING_TAGS = {
	"arrow", "plus", "text", "rect", "oval", "square", "circle",
	"polygon", "polyline",
}
_HEADER_TAGS = {"info", "metadata", "paper", "viewport", "standard"}


def decode_compatibility_cdml_file(
		file_path: str, bond_length_pt: float | None = None,
		preserve_coordinates: bool = True,
		) -> ferrum_qt.models.document.Document:
	"""Load a complete CDML drawing from ``file_path``.

	Native CDML defaults to preserving its scene coordinates.  Callers importing
	into a normalized layout can pass ``preserve_coordinates=False`` and a target
	bond length.
	"""
	with open(file_path, "r", encoding="utf-8") as source:
		return decode_compatibility_cdml_string(
			source.read(), bond_length_pt=bond_length_pt,
			preserve_coordinates=preserve_coordinates,
		)


#============================================
def decode_compatibility_cdml_string(
		text: str, bond_length_pt: float | None = None,
		preserve_coordinates: bool = True,
		) -> ferrum_qt.models.document.Document:
	"""Decode standalone or legacy CDML through the local compatibility route."""
	return _hydrate_cdml_document(
		text, bond_length_pt, preserve_coordinates, True,
		None, None, None, None, None, None, None,
	)


#============================================
def hydrate_synchronized_cdml_document(
		projection_snapshot: oasa.cdml_document.CDMLProjectionSnapshot,
		) -> ferrum_qt.models.document.Document:
	"""Hydrate a Qt document from all facts of one authoritative revision."""
	if type(projection_snapshot) is not oasa.cdml_document.CDMLProjectionSnapshot:
		raise ValueError("synchronized hydration requires one backend projection envelope")
	plan = projection_snapshot.plan
	if type(plan) is not oasa.cdml_document.CDMLProjectionPlan:
		raise ValueError("synchronized hydration requires one backend projection plan")
	if plan.revision != projection_snapshot.snapshot.revision:
		raise ValueError("synchronized projection plan revision must match its snapshot")
	ferrum_qt.io.cdml_projection_staging._require_complete_molecule_render_batches(
		plan.molecule_core_observation,
		plan.molecule_render_observation,
	)
	return _hydrate_cdml_document(
		projection_snapshot.snapshot.cdml, None, True, False,
		plan.presentation_description, plan.paper_layout,
		plan.fragment_metadata, plan.atom_mark_observation,
		plan.group_observation, plan.molecule_core_observation,
		plan.molecule_render_observation,
	)


#============================================
def _hydrate_cdml_document(
		text: str, bond_length_pt: float | None, preserve_coordinates: bool,
		compatibility_mode: bool,
		presentation_description: oasa.cdml_document.CDMLPresentationDescription | None,
		paper_layout: oasa.cdml_document.CDMLPaperLayout | None,
		fragment_metadata: oasa.cdml_document.CDMLFragmentMetadata | None,
		atom_mark_observation: oasa.cdml_document.CDMLAtomMarkObservation | None,
		group_observation: oasa.cdml_document.CDMLGroupObservation | None,
		molecule_core_observation: oasa.cdml_document.CDMLMoleculeCoreObservation | None,
		molecule_render_observation: oasa.cdml_document.CDMLMoleculeRenderObservation | None,
		) -> ferrum_qt.models.document.Document:
	"""Associate source positions with either compatibility or synchronized facts."""
	dom_doc = oasa.safe_xml.parse_dom_from_string(text)
	root = dom_doc.documentElement
	if root is None or ferrum_qt.io.cdml_xml_helpers._local_name(root) != "cdml":
		raise ValueError("CDML document must have a <cdml> root element")

	document = ferrum_qt.models.document.Document()
	header_elements = {tag: [] for tag in _HEADER_TAGS}
	envelope = None
	state = {
		"paper": None,
		"reactions": [],
		"external_data": [],
		"unsupported": [],
		"trailing": [],
	}
	presentation_by_position = (
		{record.source_position: record for record in presentation_description.records}
		if presentation_description is not None else {}
	)
	presentation_issues_by_position = {}
	if presentation_description is not None:
		for issue in presentation_description.issues:
			presentation_issues_by_position.setdefault(issue.source_position, []).append(issue)
	fragment_records_by_molecule = {}
	if fragment_metadata is not None:
		for record in fragment_metadata.records:
			fragment_records_by_molecule.setdefault(
				record.molecule_source_position, [],
			).append(record)
	mark_records_by_position = {}
	if atom_mark_observation is not None:
		for record in atom_mark_observation.records:
			mark_records_by_position[(
				record.molecule_source_position, record.atom_source_position,
				record.mark_source_position,
			)] = record
	group_records_by_position = {}
	if group_observation is not None:
		for record in group_observation.records:
			group_records_by_position[(record.molecule_source_position, record.group_source_position)] = record
	molecule_core_by_position = {}
	render_batches_by_molecule = {}
	if molecule_render_observation is not None:
		for batch in molecule_render_observation.batches:
			render_batches_by_molecule.setdefault(batch.molecule_source_position, []).append(batch)
	molecule_core_issues_by_molecule = {}
	if molecule_core_observation is not None:
		for record in molecule_core_observation.records:
			molecule_core_by_position[record.source_position] = record
		for issue in molecule_core_observation.issues:
			molecule_core_issues_by_molecule.setdefault(issue.molecule_source_position, []).append(issue)

	for child_position, child in enumerate(ferrum_qt.io.cdml_xml_helpers._element_children(root), start=1):
		tag = ferrum_qt.io.cdml_xml_helpers._local_name(child)
		for issue in presentation_issues_by_position.get(child_position, ()):
			state["unsupported"].append(ferrum_qt.io.cdml_xml_helpers._unsupported_from_presentation_issue(issue))
		if tag in _HEADER_TAGS and ferrum_qt.io.cdml_xml_helpers._is_direct_core_cdml_child(child):
			if paper_layout is not None:
				continue
			header = ferrum_qt.io.cdml_xml_helpers._raw_xml(child)
			header_elements[tag].append(header)
			continue
		if tag == "molecule":
			for issue in molecule_core_issues_by_molecule.get(child_position, ()):
				state["unsupported"].append(ferrum_qt.models.document_object.UnsupportedContent(
					issue.kind, None, "/cdml/molecule[%d]/%s[%d]" % (
						child_position, issue.kind, issue.source_position,
					), issue.reason, "",
				))
			core_record = molecule_core_by_position.get(child_position)
			mol_model = (
				ferrum_qt.io.cdml_molecule_hydration._hydrate_molecule_core_observation(core_record)
				if core_record is not None else (
					ferrum_qt.io.cdml_molecule_hydration._decode_compatibility_molecule(child, bond_length_pt, preserve_coordinates)
					if compatibility_mode else None
				)
			)
			if core_record is not None and molecule_render_observation is not None:
				ferrum_qt.io.cdml_molecule_hydration._install_molecule_render_batches(mol_model, render_batches_by_molecule.get(child_position, ()))
			if mol_model is None:
				unsupported = ferrum_qt.io.cdml_xml_helpers._unsupported(
						child, "molecule could not be decoded",
						"/cdml/molecule[%d]" % child_position,
					)
				state["unsupported"].append(unsupported)
				document.add_presentation_object(
						ferrum_qt.io.cdml_presentation_hydration._presentation(child, supported=False), mark_dirty=False,
					)
				continue
			# Legacy CDML can omit atom and bond IDs.  Give every loaded
			# model/source pair a shared ID before it is exposed for editing;
			# matching the remaining nodes by position at a later save would
			# attach deleted nodes' display XML to their survivors.
			# Import into a detached DOM so inherited namespace declarations stay
			# on the retained fragment instead of being stripped against ``root``.
			source_el = ferrum_qt.io.cdml_xml_helpers._import_raw(dom.Document(), ferrum_qt.io.cdml_xml_helpers._raw_xml(child))
			if core_record is None:
				ferrum_qt.io.cdml_molecule_metadata._normalize_loaded_source_ids(mol_model, source_el)
				ferrum_qt.io.cdml_molecule_metadata._load_atom_number_fields(mol_model, source_el)
			if compatibility_mode:
				mol_model.compatibility_source_xml = ferrum_qt.io.cdml_xml_helpers._raw_xml(source_el)
			if fragment_metadata is None:
				ferrum_qt.io.cdml_molecule_metadata._parse_fragments(mol_model, source_el, state["unsupported"], child_position)
			else:
				ferrum_qt.io.cdml_molecule_metadata._hydrate_fragment_metadata(
					mol_model, fragment_records_by_molecule.get(child_position, ()),
				)
			if group_observation is None:
				ferrum_qt.io.cdml_molecule_metadata._parse_groups(mol_model, source_el, state["unsupported"], child_position)
			else:
				ferrum_qt.io.cdml_molecule_metadata._hydrate_group_observation(
					mol_model, source_el, state["unsupported"], child_position,
					group_records_by_position,
				)
			document.add_molecule(mol_model, mark_dirty=False)
			atom_lookup: dict[str, object] = {}
			for atom_model in mol_model.atoms:
				atom_id = atom_model.atom_id
				if atom_id:
					atom_lookup[str(atom_id)] = atom_model
			if atom_mark_observation is None:
				ferrum_qt.io.cdml_presentation_hydration._parse_marks(
						document, source_el, atom_lookup, state["unsupported"], child_position,
						)
			else:
				ferrum_qt.io.cdml_molecule_metadata._hydrate_atom_mark_observation(
					document, source_el, ferrum_qt.io.cdml_molecule_hydration._core_atoms_by_source_position(mol_model),
					state["unsupported"], child_position,
					mark_records_by_position,
				)
			ferrum_qt.io.cdml_molecule_metadata._report_unrendered_molecule_children(
					source_el, state["unsupported"], child_position,
				)
			continue
		if tag in _DRAWING_TAGS:
			if presentation_description is None:
				document.add_presentation_object(
						ferrum_qt.io.cdml_presentation_hydration._presentation(child, supported=True), mark_dirty=False,
					)
			else:
				record = presentation_by_position.get(child_position)
				if record is not None:
					document.add_presentation_object(
							ferrum_qt.io.cdml_presentation_hydration._presentation_from_description(record), mark_dirty=False,
						)
			continue
		if tag == "reaction":
			if paper_layout is not None:
				continue
			state["reactions"].append(ferrum_qt.io.cdml_presentation_hydration._reaction(child))
			continue
		if tag == "external-data":
			if paper_layout is not None:
				continue
			state["external_data"].append(ferrum_qt.io.cdml_xml_helpers._raw_xml(child))
			continue
		if presentation_description is not None:
			continue
		unsupported = ferrum_qt.io.cdml_xml_helpers._unsupported(
				child, "unsupported top-level CDML element",
				"/cdml/%s[%d]" % (tag, child_position),
				)
		state["unsupported"].append(unsupported)
		document.add_presentation_object(
				ferrum_qt.io.cdml_presentation_hydration._presentation(child, supported=False), mark_dirty=False,
				)

	if paper_layout is not None:
		state["paper"] = ferrum_qt.models.document_object.PaperModel(
			attributes=dict(paper_layout.effective_paper_attributes),
			viewport_attributes=dict(paper_layout.viewport_attributes),
		)
		envelope = ferrum_qt.models.document_object.CdmlEnvelope()
	else:
		info_xml = header_elements["info"]
		metadata_xml = header_elements["metadata"]
		standard_xml = header_elements["standard"]
		paper_xml = header_elements["paper"][0] if header_elements["paper"] else None
		viewport_xml = header_elements["viewport"][0] if header_elements["viewport"] else None
		paper_el = ferrum_qt.io.cdml_xml_helpers._import_raw(dom_doc, paper_xml) if paper_xml else None
		viewport_el = ferrum_qt.io.cdml_xml_helpers._import_raw(dom_doc, viewport_xml) if viewport_xml else None
		state["paper"] = ferrum_qt.models.document_object.PaperModel(
			attributes=ferrum_qt.io.cdml_xml_helpers._attributes(paper_el), viewport_attributes=ferrum_qt.io.cdml_xml_helpers._attributes(viewport_el),
			raw_xml=paper_xml, viewport_raw_xml=viewport_xml,
		)
		extra_headers = []
		for tag in ("paper", "viewport"):
			extra_headers.extend(header_elements[tag][1:])
		envelope = ferrum_qt.models.document_object.CdmlEnvelope(
			root_attributes=ferrum_qt.io.cdml_xml_helpers._all_attributes(root),
			info_xml=info_xml, metadata_xml=metadata_xml, standard_xml=standard_xml,
			extra_header_xml=extra_headers, reactions=state["reactions"],
			external_data_xml=state["external_data"], trailing_xml=state["trailing"],
		)
	state["envelope"] = envelope
	document.set_cdml_state(envelope, state["paper"], state["unsupported"])
	document.mark_clean()
	return document


#============================================
#============================================

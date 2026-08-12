"""Hydrate compatibility molecule cores and backend molecule observations."""

# Standard Library
import xml.dom.minidom as dom

# local repo modules
import oasa.cdml_document
import oasa.cdml_writer

import ferrum_qt.bridge.oasa_bridge
import ferrum_qt.io.cdml_xml_helpers
import ferrum_qt.models.molecule_model


def _decode_compatibility_molecule(
		element: dom.Element, bond_length_pt: float | None,
		preserve_coordinates: bool,
		) -> ferrum_qt.models.molecule_model.MoleculeModel | None:
	"""Decode one molecule without splitting a native CDML element."""
	oasa_molecule = oasa.cdml_writer.read_cdml_molecule_element(element)
	if oasa_molecule is None:
		return None
	target = None if preserve_coordinates else bond_length_pt
	mol_model = ferrum_qt.bridge.oasa_bridge.oasa_mol_to_qt_mol(
		oasa_molecule, bond_length_pt=target,
	)
	return mol_model


#============================================
def _hydrate_molecule_core_observation(
		record: oasa.cdml_document.CDMLMoleculeCoreObservationRecord,
		) -> ferrum_qt.models.molecule_model.MoleculeModel:
	"""Build fresh Qt wrappers from backend-owned atom and bond facts only."""
	molecule = ferrum_qt.models.molecule_model.MoleculeModel()
	molecule.mol_id = record.identifier if record.addressable and record.identifier else ""
	molecule.name = record.name or ""
	atoms = {}
	ambiguous_atom_ids = set()
	for record_atom in record.atoms:
		if not record_atom.renderable:
			continue
		atom = molecule.create_atom(record_atom.symbol)
		atom._molecule_core_source_position = record_atom.source_position
		atom.install_projection(
			atom_id=record_atom.identifier,
			symbol=record_atom.symbol,
			charge=record_atom.charge if record_atom.charge is not None else 0,
			valency=(
				record_atom.valency
				if record_atom.valency is not None else atom.valency
			),
			authored_valency=record_atom.valency,
			isotope=record_atom.isotope,
			multiplicity=record_atom.multiplicity if record_atom.multiplicity is not None else 1,
			free_sites=record_atom.free_sites if record_atom.free_sites is not None else 0,
			explicit_hydrogens=(
				record_atom.explicit_hydrogens
				if record_atom.explicit_hydrogens is not None else 0
			),
			x=record_atom.x_pt,
			y=record_atom.y_pt,
			z=record_atom.z_pt if record_atom.z_pt is not None else 0.0,
			show=record_atom.show if record_atom.show is not None else True,
			show_hydrogens=(
				record_atom.show_hydrogens
				if record_atom.show_hydrogens is not None else True
			),
			font_size=record_atom.font_size if record_atom.font_size is not None else 12,
			font_family=record_atom.font_family if record_atom.font_family is not None else "Arial",
			line_color=record_atom.line_color if record_atom.line_color is not None else "#000000",
			number=record_atom.number,
			show_number=record_atom.show_number if record_atom.show_number is not None else True,
			explicit_fields=frozenset(
				name for name, value in (
					("show", record_atom.show),
					("show_hydrogens", record_atom.show_hydrogens),
					("font_size", record_atom.font_size),
					("font_family", record_atom.font_family),
					("line_color", record_atom.line_color),
				) if value is not None
			),
		)
		atom.bind_backend_durable_id(
			record_atom.identifier if record_atom.addressable else None,
		)
		molecule.add_atom(atom)
		if record_atom.identifier is not None:
			if record_atom.identifier in atoms:
				ambiguous_atom_ids.add(record_atom.identifier)
				atoms.pop(record_atom.identifier)
			elif record_atom.identifier not in ambiguous_atom_ids:
				atoms[record_atom.identifier] = atom
	for record_bond in record.bonds:
		if (
			not record_bond.renderable
			or record_bond.start_id not in atoms or record_bond.end_id not in atoms
			or record_bond.order is None or record_bond.bond_type is None
		):
			continue
		bond = molecule.create_bond(record_bond.order, record_bond.bond_type)
		bond._molecule_core_source_position = record_bond.source_position
		bond.install_projection(
			bond_id=record_bond.identifier,
			order=record_bond.order,
			bond_type=record_bond.bond_type,
			aromatic=None,
			line_width=record_bond.line_width,
			bond_width=record_bond.bond_width,
			wedge_width=record_bond.wedge_width,
			double_ratio=record_bond.double_ratio,
			center=record_bond.center,
			auto_sign=record_bond.auto_sign,
			equithick=record_bond.equithick,
			simple_double=record_bond.simple_double,
			line_color=record_bond.line_color,
			wavy_style=record_bond.wavy_style,
			haworth_position=record_bond.haworth_position,
			explicit_fields=record_bond.explicit_fields,
		)
		bond.bind_backend_durable_id(record_bond.identifier if record_bond.addressable else None)
		molecule.add_bond(atoms[record_bond.start_id], atoms[record_bond.end_id], bond)
	return molecule


#============================================
def _install_molecule_render_batches(
		molecule: ferrum_qt.models.molecule_model.MoleculeModel,
		batches: tuple[object, ...] | list[object],
		) -> None:
	"""Attach disposable backend paint facts to their fresh Qt wrappers."""
	for batch in batches:
		candidates = molecule.atoms if batch.kind == "atom" else molecule.bonds
		target = next((candidate for candidate in candidates if getattr(
			candidate, "_molecule_core_source_position", None,
		) == batch.source_position), None)
		if target is not None:
			target._backend_render_batch = batch


#============================================
def _core_atoms_by_source_position(
		molecule: ferrum_qt.models.molecule_model.MoleculeModel,
		) -> dict[int, object]:
	"""Map one synchronized root's observed atom positions to fresh Qt models."""
	result = {}
	for atom in molecule.atoms:
		source_position = getattr(atom, "_molecule_core_source_position", None)
		if type(source_position) is int:
			result[source_position] = atom
	return result


#============================================
def _remove_molecule_core_source_children(molecule_el: dom.Element) -> None:
	"""Drop every direct atom or bond lookalike from synchronized source XML."""
	for child in tuple(ferrum_qt.io.cdml_xml_helpers._element_children(molecule_el)):
		if ferrum_qt.io.cdml_xml_helpers._local_name(child) in {"atom", "bond"}:
			molecule_el.removeChild(child)


#============================================

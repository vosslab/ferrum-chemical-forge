"""Fragment metadata and linear-form actions over the document model."""

import PySide6.QtWidgets

import ferrum_qt.bridge.oasa_bridge
import ferrum_qt.models.document_session
import ferrum_qt.models.fragment_model
import ferrum_qt.undo.commands


# Legacy linear-form fragments use this compact display spacing in scene units.
_LINEAR_FORM_BOND_LENGTH = 10.0
_LINEAR_FORM_PROPERTY_TYPE = "IntType"
_LINEAR_FORM_LABEL_PADDING = 2.0

def _ordered_fragment_selection(app: object) -> tuple[object, list[object], list[object]] | None:
	"""Resolve one selected molecule and its members in canonical model order."""
	atom_items = app.document.selected_atoms
	bond_items = app.document.selected_bonds
	selected_items = [*atom_items, *bond_items]
	molecules = {
		app.document.molecule_for_graphics_item(item)
		for item in selected_items
	}
	if not selected_items or None in molecules or len(molecules) != 1:
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Create Fragment",
			"Select atoms and bonds from exactly one molecule."
		)
		return
	molecule = next(iter(molecules))
	selected_atom_models = {id(item.atom_model) for item in atom_items}
	selected_bond_models = {id(item.bond_model) for item in bond_items}
	bond_models = [
		bond_model for bond_model in molecule.bonds
		if id(bond_model) in selected_bond_models
	]
	if len(bond_models) != len(selected_bond_models):
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Create Fragment", "Selected objects are unavailable in the authoritative document.",
		)
		return
	member_atom_models = set(selected_atom_models)
	for bond_model in bond_models:
		if bond_model.atom1 is not None:
			member_atom_models.add(id(bond_model.atom1))
		if bond_model.atom2 is not None:
			member_atom_models.add(id(bond_model.atom2))
	atom_models = [
		atom_model for atom_model in molecule.atoms
		if id(atom_model) in member_atom_models
	]
	if len(atom_models) != len(member_atom_models):
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Create Fragment", "Selected objects are unavailable in the authoritative document.",
		)
		return
	if not atom_models:
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Create Fragment", "A fragment must contain at least one atom."
		)
		return None
	return molecule, atom_models, bond_models


#============================================
def _capture_fragment_create_submit(
		app: object, origin_session: object,
		) -> tuple[int, object, str, tuple[str, ...], tuple[str, ...]] | None:
	"""Capture one synchronized fragment intent with no projection wrappers."""
	selection = _ordered_fragment_selection(app)
	if selection is None:
		return None
	molecule, atom_models, bond_models = selection
	molecule_id = molecule.mol_id
	atom_ids = tuple(
		atom_model.backend_durable_id for atom_model in atom_models
		if atom_model.backend_durable_id is not None
	)
	bond_ids = tuple(
		bond_model.backend_durable_id for bond_model in bond_models
		if bond_model.backend_durable_id is not None
	)
	if len(atom_ids) != len(atom_models) or len(bond_ids) != len(bond_models) or not molecule_id:
		PySide6.QtWidgets.QMessageBox.warning(
			app, "Create Fragment", "Selected objects are unavailable in the authoritative document.",
		)
		return None
	try:
		submit = app.persistent_operation_capability_for(origin_session)
	except ValueError:
		return None
	return origin_session.backend_snapshot.revision, submit, molecule_id, atom_ids, bond_ids


#============================================
def _create_fragment(app: object) -> None:
	"""Create durable metadata for one selected molecular subgraph.

	Args:
		app: MainWindow instance.
	"""
	origin_session = getattr(app, "_active_session", None)
	captured = None
	if origin_session is not None and origin_session.can_commit_persistent_action:
		captured = _capture_fragment_create_submit(app, origin_session)
		if captured is None:
			return
	else:
		selection = _ordered_fragment_selection(app)
		if selection is None:
			return
		molecule, atom_models, bond_models = selection
	name, accepted = PySide6.QtWidgets.QInputDialog.getText(
		app, "Create Fragment", "Fragment name:"
	)
	if not accepted or not name.strip():
		return
	fragment_type, accepted = PySide6.QtWidgets.QInputDialog.getItem(
		app, "Create Fragment", "Fragment type:",
		["explicit", "implicit"], 0, False,
	)
	if not accepted:
		return
	if captured is not None:
		origin_revision, origin_submit, molecule_id, atom_ids, bond_ids = captured
		request = ferrum_qt.models.document_session.build_fragment_create_request(
			origin_revision, molecule_id, name.strip(), fragment_type, atom_ids, bond_ids,
		)
		outcome = origin_submit(request)
		if outcome.status != "accepted":
			PySide6.QtWidgets.QMessageBox.warning(app, "Create Fragment", outcome.message)
		return
	atom_id_changes, bond_id_changes = app.document.planned_fragment_id_changes(
		molecule,
	)
	fragment = ferrum_qt.models.fragment_model.FragmentModel(
		fragment_id=app.document.unique_cdml_id("fragment"),
		fragment_type=fragment_type,
		name=name.strip(),
		atom_ids=tuple(
			after_id for atom_model, _before_id, after_id in atom_id_changes
			if atom_model in atom_models
		),
		bond_ids=tuple(
			after_id for bond_model, _before_id, after_id in bond_id_changes
			if bond_model in bond_models
		),
	)
	app.document.undo_stack.push(ferrum_qt.undo.commands.AddFragmentCommand(
		molecule, fragment, atom_id_changes, bond_id_changes,
	))


#============================================
def _fragment_choices(app: object) -> tuple[list[tuple[str, str, str, int]], list[str]]:
	"""Read current fragment labels into plain durable dialog data."""
	choices = []
	raw_entries = []
	for molecule_position, molecule in enumerate(app.document.molecules, start=1):
		molecule_label = molecule.name or molecule.mol_id or "Molecule %d" % molecule_position
		for fragment in molecule.fragments:
			label = "%s: %s [%s; %s]" % (
				molecule_label, fragment.name or "unnamed", fragment.fragment_type,
				fragment.fragment_id,
			)
			choices.append((label, molecule.mol_id, fragment.fragment_id, molecule_position - 1))
		for notice in molecule.fragment_notices:
			raw_entries.append("%s: %s" % (molecule_label, notice))
		if molecule.unsupported_fragment_xml:
			raw_entries.append("%s: imported fragment metadata is read-only." % molecule_label)
	return choices, raw_entries


#============================================
def _view_fragments(app: object) -> None:
	"""Display one molecule's fragments and delete editable metadata on request.

	Args:
		app: MainWindow instance.
	"""
	choices, raw_entries = _fragment_choices(app)
	if not choices:
		message = "No editable fragments are defined."
		if raw_entries:
			message += "\n\nRetained imported fragments are read-only:\n%s" % (
				"\n".join(raw_entries),
			)
		PySide6.QtWidgets.QMessageBox.information(app, "View Fragments", message)
		return
	origin_session = getattr(app, "_active_session", None)
	origin_revision = None
	origin_submit = None
	if origin_session is not None and origin_session.can_commit_persistent_action:
		origin_revision = origin_session.backend_snapshot.revision
		try:
			origin_submit = app.persistent_operation_capability_for(origin_session)
		except ValueError:
			return
	choice_map = {
		label: (molecule_id, fragment_id, molecule_position)
		for label, molecule_id, fragment_id, molecule_position in choices
	}
	labels = ["Keep fragments unchanged", *choice_map]
	prompt = "Choose a fragment to delete:"
	if raw_entries:
		prompt += "\n\nRead-only imported fragments:\n%s" % "\n".join(raw_entries)
	choice, accepted = PySide6.QtWidgets.QInputDialog.getItem(
		app, "View Fragments", prompt, labels, 0, False,
	)
	if not accepted or choice == labels[0]:
		return
	molecule_id, fragment_id, molecule_position = choice_map[choice]
	if origin_submit is not None and origin_revision is not None:
		if not molecule_id:
			PySide6.QtWidgets.QMessageBox.warning(
				app, "View Fragments", "The selected molecule is unavailable in the authoritative document.",
			)
			return
		request = ferrum_qt.models.document_session.build_fragment_delete_request(
			origin_revision, molecule_id, fragment_id,
		)
		outcome = origin_submit(request)
		if outcome.status != "accepted":
			PySide6.QtWidgets.QMessageBox.warning(app, "View Fragments", outcome.message)
		return
	if not 0 <= molecule_position < len(app.document.molecules):
		return
	molecule = app.document.molecules[molecule_position]
	if (
			molecule.mol_id != molecule_id
			or not any(fragment.fragment_id == fragment_id for fragment in molecule.fragments)
		):
		return
	app.document.undo_stack.push(ferrum_qt.undo.commands.RemoveFragmentCommand(
		molecule, fragment_id,
	))


#============================================
def _capture_linear_form_submit(
		app: object, origin_session: object,
		) -> tuple[int, object, str, tuple[str, ...]] | None:
	"""Capture one origin-bound linear-form intent without retaining Qt wrappers."""
	atom_items = tuple(app.document.selected_atoms)
	bond_items = tuple(app.document.selected_bonds)
	items = (*atom_items, *bond_items)
	molecules = {
		app.document.molecule_for_graphics_item(item)
		for item in items
	}
	if not items or None in molecules or len(molecules) != 1:
		_linear_warning(app, "Select atoms and bonds from exactly one molecule.")
		return None
	molecule = next(iter(molecules))
	selected_models = {item.atom_model for item in atom_items}
	for item in bond_items:
		if item.bond_model.atom1 is not None:
			selected_models.add(item.bond_model.atom1)
		if item.bond_model.atom2 is not None:
			selected_models.add(item.bond_model.atom2)
	atom_ids = tuple(
		atom.backend_durable_id for atom in molecule.atoms
		if atom in selected_models and atom.backend_durable_id is not None
	)
	if not molecule.mol_id or len(atom_ids) != len(selected_models):
		_linear_warning(app, "Selected atoms are unavailable in the authoritative document.")
		return None
	try:
		submit = app.persistent_operation_capability_for(origin_session)
	except ValueError:
		return None
	return origin_session.backend_snapshot.revision, submit, molecule.mol_id, atom_ids


#============================================
def _convert_to_linear(app: object) -> None:
	"""Convert one selected unbranched component into a linear fragment.

	The legacy action records a ``linear_form`` fragment rather than replacing
	the molecular graph.  Qt keeps that contract, but computes every affected
	coordinate before it pushes a macro so an invalid selection cannot leave a
	partly moved molecule behind.

	Args:
		app: MainWindow instance.
	"""
	origin_session = getattr(app, "_active_session", None)
	if origin_session is not None:
		if not origin_session.can_commit_persistent_action:
			_linear_warning(app, "Document cannot accept a persistent edit.")
			return
		captured = _capture_linear_form_submit(app, origin_session)
		if captured is None:
			return
		origin_revision, submit, molecule_id, atom_ids = captured
		request = ferrum_qt.models.document_session.build_linear_form_convert_request(
			origin_revision, molecule_id, atom_ids,
		)
		outcome = submit(request)
		if outcome.status != "accepted":
			_linear_warning(app, outcome.message)
		return
	selection = _linear_selection(app)
	if selection is None:
		return
	molecule, path, path_bonds = selection
	coordinate_plan = _linear_coordinate_changes(molecule, path, path_bonds)
	if coordinate_plan is None:
		_linear_warning(
			app, "The selected chain has an external component attached to more than one selected atom.",
		)
		return
	atom_changes, bond_length = coordinate_plan
	atom_id_changes, bond_id_changes = app.document.planned_fragment_id_changes(
		molecule,
	)
	if not _linear_id_normalization_is_safe(
		molecule, atom_id_changes, bond_id_changes,
	):
		_linear_warning(
			app, "This conversion would renumber atoms or bonds referenced by an existing fragment.",
		)
		return
	atom_ids = _fragment_ids_for_models(path, atom_id_changes)
	bond_ids = _fragment_ids_for_models(path_bonds, bond_id_changes)
	fragment = ferrum_qt.models.fragment_model.FragmentModel(
		fragment_id=app.document.unique_cdml_id("fragment"),
		fragment_type="linear_form",
		name="",
		atom_ids=atom_ids,
		bond_ids=bond_ids,
		properties=(ferrum_qt.models.fragment_model.FragmentProperty(
				name="bond_length", value=f"{bond_length:.6f}",
				type_name=_LINEAR_FORM_PROPERTY_TYPE,
		),),
	)
	# A macro makes geometry, explicit-hydrogen display, stable IDs, and the
	# fragment metadata one undo/redo operation and one dirty transition.
	undo_stack = app.document.undo_stack
	undo_stack.beginMacro("Convert to Linear Form")
	if atom_changes:
		undo_stack.push(ferrum_qt.undo.commands.TransformGeometryCommand(
				atom_changes, [], "Linear Form Geometry",
		))
	for atom_model in path:
		if not atom_model.show_hydrogens:
			undo_stack.push(ferrum_qt.undo.commands.ChangePropertyCommand(
					atom_model, "show_hydrogens", False, True,
					"Show Linear Form Hydrogens",
			))
	undo_stack.push(ferrum_qt.undo.commands.AddFragmentCommand(
			molecule, fragment, atom_id_changes, bond_id_changes,
			"Create Linear Form Fragment",
	))
	undo_stack.endMacro()
	app.statusBar().showMessage("Converted selection to linear form", 3000)


#============================================
def _linear_warning(app: object, message: str) -> None:
	"""Show one safe, non-mutating conversion rejection message."""
	PySide6.QtWidgets.QMessageBox.warning(app, "Convert to Linear Form", message)


#============================================
def _linear_selection(app: object) -> tuple[object, tuple[object, ...], tuple[object, ...]] | None:
	"""Return one selected path and its induced bonds, or report why not.

	Selected bonds contribute both endpoints, matching the legacy selection
	semantics.  The selected vertices must induce a single unbranched path (or
	a single atom); a ring and a fork cannot safely become a linear formula.
	"""
	atom_items = app.document.selected_atoms
	bond_items = app.document.selected_bonds
	items = [*atom_items, *bond_items]
	molecules = {
		app.document.molecule_for_graphics_item(item)
		for item in items
	}
	if not items or None in molecules or len(molecules) != 1:
		_linear_warning(app, "Select atoms and bonds from exactly one molecule.")
		return None
	molecule = next(iter(molecules))
	selected_atoms = {item.atom_model for item in atom_items}
	for item in bond_items:
		if item.bond_model.atom1 is not None:
			selected_atoms.add(item.bond_model.atom1)
		if item.bond_model.atom2 is not None:
			selected_atoms.add(item.bond_model.atom2)
	if not selected_atoms:
		_linear_warning(app, "Select at least one atom or bond to make a linear form.")
		return None
	induced_bonds = tuple(
		bond for bond in molecule.bonds
		if bond.atom1 in selected_atoms and bond.atom2 in selected_atoms
	)
	neighbors = {atom: [] for atom in selected_atoms}
	for bond in induced_bonds:
		neighbors[bond.atom1].append(bond.atom2)
		neighbors[bond.atom2].append(bond.atom1)
	if any(len(atom_neighbors) > 2 for atom_neighbors in neighbors.values()):
		_linear_warning(app, "The selection is not linear because it contains a branch.")
		return None
	path = _ordered_linear_path(neighbors)
	if path is None:
		_linear_warning(
			app, "The selected atoms must form one connected chain, not a ring or split selection.",
		)
		return None
	path_bonds = _ordered_path_bonds(path, induced_bonds)
	return molecule, path, path_bonds


#============================================
def _ordered_linear_path(neighbors: dict[object, list[object]]) -> tuple[object, ...] | None:
	"""Order a connected path from its leftmost endpoint without mutation."""
	if len(neighbors) == 1:
		path = tuple(neighbors)
		return path
	ends = [atom for atom, atom_neighbors in neighbors.items()
			if len(atom_neighbors) == 1]
	if len(ends) != 2:
		return None
	start = min(ends, key=lambda atom: (atom.x, atom.y, id(atom)))
	path = []
	previous = None
	current = start
	while current is not None:
		path.append(current)
		next_atoms = [atom for atom in neighbors[current] if atom is not previous]
		if len(next_atoms) > 1:
			return None
		previous, current = current, next_atoms[0] if next_atoms else None
	if len(path) != len(neighbors):
		return None
	return tuple(path)


#============================================
def _ordered_path_bonds(
		path: tuple[object, ...], bonds: tuple[object, ...],
		) -> tuple[object, ...]:
	"""Return path edges in the same semantic order as their vertices."""
	ordered = []
	for first, second in zip(path, path[1:]):
		for bond in bonds:
			if {bond.atom1, bond.atom2} == {first, second}:
				ordered.append(bond)
				break
		else:
			raise ValueError("linear path is missing an induced bond")
	result = tuple(ordered)
	return result


#============================================
def _linear_id_normalization_is_safe(
		molecule: object, atom_id_changes: tuple[tuple[object, str, str], ...],
		bond_id_changes: tuple[tuple[object, str, str], ...],
		) -> bool:
	"""Reject ID rewrites that could invalidate pre-existing fragment XML.

	``AddFragmentCommand`` changes atom and bond IDs atomically with the new
	fragment.  Existing editable or losslessly retained fragment metadata may
	refer to those IDs, however, so this action refuses the conversion before
	any command is pushed instead of creating dangling references.
	"""
	id_changes = [*atom_id_changes, *bond_id_changes]
	requires_rewrite = any(before != after for _model, before, after in id_changes)
	if not requires_rewrite:
		return True
	safe = not molecule.fragments and not molecule.unsupported_fragment_xml
	return safe


#============================================
def _linear_coordinate_changes(
		molecule: object, path: tuple[object, ...], path_bonds: tuple[object, ...],
		) -> tuple[list[tuple[object, tuple[float, float], tuple[float, float]]], float] | None:
	"""Plan linear-path and attached-component translations without mutation."""
	path_set = set(path)
	path_bond_set = set(path_bonds)
	start_x, start_y = path[0].x, path[0].y
	bond_length = _linear_label_safe_spacing(path)
	deltas = {
		atom: (
			start_x + index * bond_length - atom.x,
			start_y - atom.y,
		)
		for index, atom in enumerate(path)
	}
	# Every external component can follow exactly one selected anchor.  An
	# external bridge between two selected atoms has no single coherent offset.
	component_offsets: dict[object, tuple[float, float]] = {}
	visited_external = set()
	for anchor in path:
		for bond in molecule.bonds:
			if bond in path_bond_set:
				continue
			other = _other_bond_atom(bond, anchor)
			if other is None or other in path_set:
				continue
			component = _external_component(molecule, other, path_set)
			component_key = frozenset(component)
			if component_key in visited_external:
				if any(component_offsets[atom] != deltas[anchor] for atom in component):
					return None
				continue
			visited_external.add(component_key)
			for atom in component:
				component_offsets[atom] = deltas[anchor]
	changes = []
	for atom in [*path, *component_offsets]:
		before = (atom.x, atom.y)
		dx, dy = deltas[atom] if atom in path_set else component_offsets[atom]
		after = (before[0] + dx, before[1] + dy)
		if after != before:
			changes.append((atom, before, after))
	return changes, bond_length


#============================================
def _linear_label_safe_spacing(path: tuple[object, ...]) -> float:
	"""Return uniform spacing that keeps adjacent rendered atom labels apart."""
	spacing = _LINEAR_FORM_BOND_LENGTH
	for first, second in zip(path, path[1:]):
		_first_left, first_right = _linear_label_bounds(first)
		second_left, _second_right = _linear_label_bounds(second)
		required = first_right - second_left + _LINEAR_FORM_LABEL_PADDING
		if required > spacing:
			spacing = required
	return spacing


#============================================
def _linear_label_bounds(atom_model: object) -> tuple[float, float]:
	"""Measure one atom label's horizontal glyph bounds relative to its atom."""
	return ferrum_qt.bridge.oasa_bridge.legacy_atom_text_bounds(atom_model)


#============================================
def _other_bond_atom(bond: object, atom: object) -> object | None:
	"""Return the opposite endpoint when ``bond`` touches ``atom``."""
	if bond.atom1 is atom:
		return bond.atom2
	if bond.atom2 is atom:
		return bond.atom1
	return None


#============================================
def _external_component(molecule: object, start: object, selected: set[object]) -> set[object]:
	"""Return unselected atoms reachable from one selected-path attachment."""
	component = set()
	pending = [start]
	while pending:
		atom = pending.pop()
		if atom in component or atom in selected:
			continue
		component.add(atom)
		for bond in molecule.bonds:
			other = _other_bond_atom(bond, atom)
			if other is not None and other not in component and other not in selected:
				pending.append(other)
	return component


#============================================
def _fragment_ids_for_models(
		models: tuple[object, ...], id_changes: tuple[tuple[object, str, str], ...],
		) -> tuple[str, ...]:
	"""Return each selected model's planned durable CDML identifier."""
	planned_ids = {model: after for model, _before, after in id_changes}
	ids = tuple(planned_ids[model] for model in models)
	return ids


#============================================

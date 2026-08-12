"""Resolve Qt selections into durable backend keys without model mutation."""

# Standard Library
import dataclasses
import enum

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.canvas.items.atom_item
import ferrum_qt.canvas.items.bond_item
import ferrum_qt.canvas.items.mark_item
import ferrum_qt.canvas.presentation_projection

#============================================
def selected_presentation_stack_root_ids(document: object,
		scene: PySide6.QtWidgets.QGraphicsScene | None) -> tuple[str, ...]:
	"""Return canonical durable roots only for real selected projections.

	Every selected item must be a direct, supported presentation item bound by
	the current projection.  Canonical document order, rather than Qt selection
	or scene order, determines the submitted IDs.
	"""
	selected_items = ferrum_qt.canvas.graphics_retirement.selected_items_from_captured_scene(
		scene,
	)
	selected_models = []
	for item in selected_items:
		model = getattr(item, "document_object_model", None)
		if (
			not getattr(model, "supported", False)
			or not getattr(model, "editable", False)
			or model not in document.presentation_objects
			or model not in document.objects
			or not isinstance(getattr(model, "object_id", None), str)
			or not model.object_id.strip()
			or not ferrum_qt.canvas.presentation_projection.is_bound_presentation_projection(item, model)
		):
			return ()
		selected_models.append(model)
	if not selected_models or len({id(model) for model in selected_models}) != len(selected_models):
		return ()
	selected_ids = {id(model) for model in selected_models}
	root_ids = tuple(
		model.object_id for model in document.objects if id(model) in selected_ids
	)
	if len(root_ids) != len(selected_ids) or len(set(root_ids)) != len(root_ids):
		return ()
	return root_ids


#============================================
def selected_top_level_transform_keys(document: object,
		scene: PySide6.QtWidgets.QGraphicsScene | None,
		) -> tuple[tuple[str, str], ...]:
	"""Return the complete eligible transform selection in document order.

	This bridge resolves selected atom, bond, group, and mark projections to
	their owning direct-root molecule.  Any item outside this exact projection
	invalidates the whole selection rather than silently changing the request.
	"""
	selected_items = ferrum_qt.canvas.graphics_retirement.selected_items_from_captured_scene(
		scene,
	)
	selected_roots = set()
	document_objects = tuple(document.objects)
	for item in selected_items:
		model = getattr(item, "document_object_model", None)
		if model is not None:
			if (
				not getattr(model, "supported", False)
				or not getattr(model, "editable", False)
				or model not in document.presentation_objects
				or model not in document_objects
				or type(getattr(model, "object_id", None)) is not str
				or not model.object_id
				or not ferrum_qt.canvas.presentation_projection.is_bound_presentation_projection(item, model)
			):
				return ()
			selected_roots.add(model)
			continue
		molecule = document.molecule_for_current_projection_item(item)
		if (
			molecule is None or molecule not in document.molecules
			or molecule not in document_objects
			or type(getattr(molecule, "mol_id", None)) is not str
			or not molecule.mol_id
		):
			return ()
		selected_roots.add(molecule)
	if not selected_roots:
		return ()
	keys = []
	for root in document_objects:
		if root not in selected_roots:
			continue
		if root in document.molecules:
			keys.append(("molecule", root.mol_id))
		elif root in document.presentation_objects:
			keys.append(("presentation", root.object_id))
		else:
			return ()
	return tuple(keys) if len(keys) == len(selected_roots) else ()


#============================================
def top_level_presentation_keys_for_items(
		document: object, items: tuple[PySide6.QtWidgets.QGraphicsItem, ...],
		) -> tuple[tuple[str, str], ...]:
	"""Return durable presentation roots for one exact current wrapper tuple.

	EditMode captures responsive drag previews as graphics/model state, then uses
	this bridge only at release to obtain the plain direct-root CDML request.  A
	foreign, retired, unsupported, duplicated, or ID-less wrapper invalidates
	the whole selection rather than allowing a local edit to leak into a
	synchronized session.
	"""
	if type(items) is not tuple or not items:
		return ()
	models = []
	for item in items:
		model = getattr(item, "document_object_model", None)
		if (
			not document.is_current_projection_item(item)
			or not getattr(model, "supported", False)
			or not getattr(model, "editable", False)
			or model not in document.presentation_objects
			or model not in document.objects
			or type(getattr(model, "object_id", None)) is not str
			or not model.object_id
			or not ferrum_qt.canvas.presentation_projection.is_bound_presentation_projection(item, model)
		):
			return ()
		models.append(model)
	if len({id(model) for model in models}) != len(models):
		return ()
	model_ids = {id(model) for model in models}
	keys = tuple(
		("presentation", model.object_id)
		for model in document.objects
		if id(model) in model_ids
	)
	return keys if len(keys) == len(models) and len({key[1] for key in keys}) == len(keys) else ()


#============================================
def selection_translate_targets_for_items(
		document: object, items: tuple[PySide6.QtWidgets.QGraphicsItem, ...],
		) -> tuple[tuple[tuple[str, str], ...], tuple[tuple[str, str], ...]] | None:
	"""Resolve one exact mixed atom/presentation selection in source order.

	The returned values contain only durable backend addresses.  Every supplied
	wrapper must belong to the current projection: atom wrappers resolve through
	the document identity bridge and presentation wrappers through their direct
	document binding.  A mixed operation deliberately has no partial selection
	meaning, so duplicate, foreign, retired, unsupported, or extra wrappers make
	the complete observation ineligible.
	"""
	if type(items) is not tuple or not items:
		return None
	if len({id(item) for item in items}) != len(items):
		return None
	atom_models = set()
	presentation_models = set()
	for item in items:
		if not document.is_current_projection_item(item):
			return None
		if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
			molecule = document.molecule_for_current_projection_item(item)
			model = item.atom_model
			atom_id = getattr(model, "backend_durable_id", None)
			molecule_id = getattr(molecule, "mol_id", None)
			if (
				molecule is None or molecule not in document.molecules
				or model not in molecule.atoms or model in atom_models
				or type(atom_id) is not str or not atom_id.strip()
				or type(molecule_id) is not str or not molecule_id.strip()
			):
				return None
			atom_models.add(model)
			continue
		model = getattr(item, "document_object_model", None)
		if (
			not getattr(model, "supported", False)
			or not getattr(model, "editable", False)
			or model not in document.presentation_objects
			or model not in document.objects
			or model in presentation_models
			or type(getattr(model, "object_id", None)) is not str
			or not model.object_id.strip()
			or not ferrum_qt.canvas.presentation_projection.is_bound_presentation_projection(item, model)
		):
			return None
		presentation_models.add(model)
	if not atom_models or not presentation_models:
		return None
	atom_targets = []
	for molecule in document.molecules:
		if molecule not in document.objects:
			return None
		for atom in molecule.atoms:
			if atom in atom_models:
				atom_targets.append((molecule.mol_id, atom.backend_durable_id))
	presentation_keys = tuple(
		("presentation", model.object_id)
		for model in document.objects if model in presentation_models
	)
	if (
		len(atom_targets) != len(atom_models)
		or len(presentation_keys) != len(presentation_models)
		or len(set(atom_targets)) != len(atom_targets)
		or len({key[1] for key in presentation_keys}) != len(presentation_keys)
	):
		return None
	return tuple(atom_targets), presentation_keys


#============================================
class StructuralSelectionKind(enum.Enum):
	"""Describe how a selected scene set relates to partial structure actions."""
	EXACT = "exact"
	INVALID = "invalid"
	ROOT_OR_MIXED = "root-or-mixed"


@dataclasses.dataclass(frozen=True)
class StructuralSelectionClassification:
	"""Return only immutable structural targets when the selection is exact."""
	kind: StructuralSelectionKind
	targets: tuple[str, tuple[str, ...], tuple[str, ...]] | None = None


#============================================
def classify_structural_selection(
		document: object, items: tuple[PySide6.QtWidgets.QGraphicsItem, ...],
		) -> StructuralSelectionClassification:
	"""Classify exact structural, invalid structural, and root/mixed selection.

	The returned observation contains only immutable durable backend values.
	Every wrapper must be registered in this exact projection and resolve to one
	document-owned molecule.  Atom and bond identifiers are emitted in molecule
	model source order rather than Qt scene or selection order.  A foreign,
	stale, or ID-less structural wrapper is invalid rather than a whole-root
	fallback.  A legitimate presentation-root or mixed selection remains for the
	existing top-level clipboard route.
	"""
	if type(items) is not tuple or not items:
		return StructuralSelectionClassification(StructuralSelectionKind.ROOT_OR_MIXED)
	if len({id(item) for item in items}) != len(items):
		return StructuralSelectionClassification(StructuralSelectionKind.INVALID)
	structural_items = tuple(
		item for item in items
		if isinstance(item, (
			ferrum_qt.canvas.items.atom_item.AtomItem,
			ferrum_qt.canvas.items.bond_item.BondItem,
		))
	)
	if not structural_items:
		return StructuralSelectionClassification(StructuralSelectionKind.ROOT_OR_MIXED)
	molecule = None
	multiple_molecules = False
	molecules_by_id = {}
	atom_models = set()
	bond_models = set()
	for item in structural_items:
		if not document.is_current_projection_item(item):
			return StructuralSelectionClassification(StructuralSelectionKind.INVALID)
		item_molecule = document.molecule_for_current_projection_item(item)
		if item_molecule is None or item_molecule not in document.molecules:
			return StructuralSelectionClassification(StructuralSelectionKind.INVALID)
		if molecule is None:
			molecule = item_molecule
		elif item_molecule is not molecule:
			multiple_molecules = True
		molecule_id = getattr(item_molecule, "mol_id", None)
		if type(molecule_id) is not str or not molecule_id.strip():
			return StructuralSelectionClassification(StructuralSelectionKind.INVALID)
		if molecule_id in molecules_by_id and molecules_by_id[molecule_id] is not item_molecule:
			return StructuralSelectionClassification(StructuralSelectionKind.INVALID)
		molecules_by_id[molecule_id] = item_molecule
		if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
			model = item.atom_model
			durable_id = getattr(model, "backend_durable_id", None)
			if (
				model not in item_molecule.atoms
				or type(durable_id) is not str
				or not durable_id.strip()
				or model in atom_models
			):
				return StructuralSelectionClassification(StructuralSelectionKind.INVALID)
			atom_models.add(model)
		elif isinstance(item, ferrum_qt.canvas.items.bond_item.BondItem):
			model = item.bond_model
			durable_id = getattr(model, "backend_durable_id", None)
			if (
				model not in item_molecule.bonds
				or type(durable_id) is not str
				or not durable_id.strip()
				or model in bond_models
			):
				return StructuralSelectionClassification(StructuralSelectionKind.INVALID)
			bond_models.add(model)
	if molecule is None:
		return StructuralSelectionClassification(StructuralSelectionKind.INVALID)
	if multiple_molecules:
		return StructuralSelectionClassification(StructuralSelectionKind.ROOT_OR_MIXED)
	molecule_id = molecule.mol_id
	atom_ids = tuple(
		atom.backend_durable_id for atom in molecule.atoms if atom in atom_models
	)
	bond_ids = tuple(
		bond.backend_durable_id for bond in molecule.bonds if bond in bond_models
	)
	if (
		len(atom_ids) != len(atom_models)
		or len(bond_ids) != len(bond_models)
		or len(set(atom_ids)) != len(atom_ids)
		or len(set(bond_ids)) != len(bond_ids)
		or set(atom_ids).intersection(bond_ids)
	):
		return StructuralSelectionClassification(StructuralSelectionKind.INVALID)
	if len(structural_items) != len(items):
		return StructuralSelectionClassification(StructuralSelectionKind.ROOT_OR_MIXED)
	return StructuralSelectionClassification(
		StructuralSelectionKind.EXACT, (molecule_id, atom_ids, bond_ids),
	)


#============================================
def structure_delete_targets_for_items(
		document: object, items: tuple[PySide6.QtWidgets.QGraphicsItem, ...],
		) -> tuple[str, tuple[str, ...], tuple[str, ...]] | None:
	"""Return exact structural targets for the existing partial-Delete route."""
	classification = classify_structural_selection(document, items)
	if classification.kind is StructuralSelectionKind.EXACT:
		return classification.targets
	return None


#============================================
def persistent_selection_key(
		item: PySide6.QtWidgets.QGraphicsItem,
		) -> tuple[str, str] | None:
	"""Return the durable CDML identity represented by an item or its parent.

	Selection is presentation state, so a replacement may carry it forward only
	when the item has an identifier that is already persisted in CDML.  Generated
	labels, marks, handles, and anonymous graphics intentionally return ``None``.
	"""
	current = item
	while ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(current):
		model = getattr(current, "document_object_model", None)
		object_id = getattr(model, "object_id", None)
		if object_id:
			return ("presentation", str(object_id))
		atom_model = getattr(current, "atom_model", None)
		if atom_model is not None:
			atom_id = getattr(atom_model, "backend_durable_id", None)
			return ("atom", str(atom_id)) if atom_id else None
		bond_model = getattr(current, "bond_model", None)
		if bond_model is not None:
			bond_id = getattr(bond_model, "backend_durable_id", None)
			return ("bond", str(bond_id)) if bond_id else None
		mark_model = getattr(current, "atom_mark_model", None)
		if mark_model is not None:
			atom_id = getattr(mark_model.atom_model, "backend_durable_id", None)
			return ("atom", str(atom_id)) if atom_id else None
		group_model = getattr(current, "group_model", None)
		group_id = getattr(group_model, "group_id", None)
		if group_id:
			return ("group", str(group_id))
		molecule = getattr(current, "molecule_model", None)
		molecule_id = getattr(molecule, "mol_id", None)
		if molecule_id:
			return ("molecule", str(molecule_id))
		current = ferrum_qt.canvas.graphics_retirement.native_parent_for_item(current)
	return None


#============================================
def atom_mark_delete_target_for_items(
		document: object, items: tuple[PySide6.QtWidgets.QGraphicsItem, ...],
		) -> tuple[str, str, str, int] | None:
	"""Return plain exact-mark deletion intent for one current supported MarkItem."""
	if len(items) != 1:
		return None
	item = items[0]
	if not isinstance(item, ferrum_qt.canvas.items.mark_item.MarkItem):
		return None
	if not document.is_current_projection_item(item):
		return None
	mark_model = getattr(item, "atom_mark_model", None)
	if mark_model is None or not mark_model.supported or mark_model not in document.marks:
		return None
	atom_item = ferrum_qt.canvas.graphics_retirement.native_parent_for_item(item)
	if not isinstance(atom_item, ferrum_qt.canvas.items.atom_item.AtomItem):
		return None
	if getattr(atom_item, "atom_model", None) is not mark_model.atom_model:
		return None
	molecule = document.molecule_for_graphics_item(atom_item)
	molecule_id = getattr(molecule, "mol_id", None)
	atom_id = getattr(mark_model.atom_model, "backend_durable_id", None)
	mark_type = mark_model.mark_type
	matching_mark_index = mark_model.matching_mark_index
	if (
		type(molecule_id) is not str or not molecule_id
		or type(atom_id) is not str or not atom_id
		or type(mark_type) is not str or not mark_type
		or type(matching_mark_index) is not int or matching_mark_index < 0
		):
		return None
	return molecule_id, atom_id, mark_type, matching_mark_index


#============================================
def select_projected_persistent_keys(
		scene: PySide6.QtWidgets.QGraphicsScene,
		keys: frozenset[tuple[str, str]],
		) -> None:
	"""Restore durable selections only to their canonical projection owners.

	An anonymous MarkItem inherits its atom's durable selection key for ordinary
	interactive correlation, but it is not a durable selection owner.  Selecting
	it here would make one backend atom selection reappear as two Qt selections
	after canonical reprojection.
	"""
	for item in scene.items():
		if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(item):
			continue
		if getattr(item, "atom_mark_model", None) is not None:
			continue
		molecule = getattr(item, "molecule_model", None)
		molecule_id = getattr(molecule, "mol_id", None)
		molecule_key = ("molecule", str(molecule_id)) if molecule_id else None
		if persistent_selection_key(item) in keys or molecule_key in keys:
			item.setSelected(True)


#============================================

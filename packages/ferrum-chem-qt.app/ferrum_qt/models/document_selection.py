"""Selection and live-projection helpers for the Document facade."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.models.atom_model
import ferrum_qt.models.bond_model
import ferrum_qt.models.document_object
import ferrum_qt.models.molecule_model


#============================================
class DocumentSelection:
	def set_scene(self, scene: PySide6.QtWidgets.QGraphicsScene | None) -> None:
		"""Wire the scene for selection change forwarding.

		Connects the scene's selectionChanged signal so Document can
		re-emit it as ``selection_changed`` for menu predicates and
		mode state updates.

		Args:
			scene: QGraphicsScene instance (ChemScene).
		"""
		from ferrum_qt.canvas.graphics_retirement import is_valid_native_wrapper
		if is_valid_native_wrapper(self._scene):
			# disconnect old scene
			self._scene.selectionChanged.disconnect(self._on_scene_selection_changed)
		self._scene = scene
		if is_valid_native_wrapper(scene):
			scene.selectionChanged.connect(self._on_scene_selection_changed)

	#============================================
	def is_current_projection_scene(
			self, scene: PySide6.QtWidgets.QGraphicsScene,
			) -> bool:
		"""Return whether ``scene`` remains this document's live projection scene."""
		from ferrum_qt.canvas.graphics_retirement import is_valid_native_wrapper
		return self._scene is scene and is_valid_native_wrapper(scene)

	#============================================
	def register_current_projection_items(
			self, items: tuple[PySide6.QtWidgets.QGraphicsItem, ...],
			) -> None:
		"""Register the exact graphics wrappers installed for this projection.

		A model reference on an arbitrary scene item is presentation metadata, not
		proof of membership.  This registration both distinguishes the current
		projection from lookalike wrappers and owns its Python wrappers until the
		projection is explicitly retired.
		"""
		if self._scene is None:
			raise RuntimeError("Projection item registration requires a live scene")
		self._projection_item_refs = {id(item): item for item in items}

	#============================================
	def is_current_projection_item(
			self, item: PySide6.QtWidgets.QGraphicsItem,
			) -> bool:
		"""Return whether one wrapper belongs to this document's live projection."""
		from ferrum_qt.canvas.graphics_retirement import item_belongs_to_scene
		scene = self._scene
		if scene is None or not self.is_current_projection_scene(scene):
			return False
		item_ref = self._projection_item_refs.get(id(item))
		return item_belongs_to_scene(scene, item) and item_ref is item

	#============================================
	def molecule_for_current_projection_item(
			self, item: PySide6.QtWidgets.QGraphicsItem,
			) -> ferrum_qt.models.molecule_model.MoleculeModel | None:
		"""Resolve one registered current-projection item to its root molecule."""
		if not self.is_current_projection_item(item):
			return None
		return self.molecule_for_graphics_item(item)

	#============================================
	def _on_scene_selection_changed(self) -> None:
		"""Forward scene selection changes as a Document signal."""
		self.selection_changed.emit()

	# ------------------------------------------------------------------
	# Selection queries
	# ------------------------------------------------------------------

	#============================================
	@property
	def selected_atoms(self) -> list:
		"""Return selected AtomItems from the scene.

		Returns:
			List of AtomItem instances currently selected.
		"""
		import ferrum_qt.canvas.items.atom_item
		from ferrum_qt.canvas.graphics_retirement import selected_items_from_captured_scene
		return [item for item in selected_items_from_captured_scene(self._scene)
				if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem)]

	#============================================
	@property
	def selected_bonds(self) -> list:
		"""Return selected BondItems from the scene.

		Returns:
			List of BondItem instances currently selected.
		"""
		import ferrum_qt.canvas.items.bond_item
		from ferrum_qt.canvas.graphics_retirement import selected_items_from_captured_scene
		return [item for item in selected_items_from_captured_scene(self._scene)
				if isinstance(item, ferrum_qt.canvas.items.bond_item.BondItem)]

	#============================================
	@property
	def selected_groups(self) -> list:
		"""Return selected native CDML group items in scene selection order."""
		import ferrum_qt.canvas.items.group_item
		from ferrum_qt.canvas.graphics_retirement import selected_items_from_captured_scene
		return [item for item in selected_items_from_captured_scene(self._scene)
				if isinstance(item, ferrum_qt.canvas.items.group_item.GroupItem)]

	#============================================
	@property
	def groups_selected(self) -> bool:
		"""Whether selection contains at least one group for menu predicates."""
		return bool(self.selected_groups)

	#============================================
	@property
	def selected_mols(self) -> list:
		"""Return MoleculeModels that have at least one selected atom.

		Deduplicates so each molecule appears at most once.

		Returns:
			List of MoleculeModel instances with selected content.
		"""
		seen = set()
		result = []
		for atom_item in self.selected_atoms:
			mol = self._find_molecule_for_atom(atom_item.atom_model)
			if mol is not None and id(mol) not in seen:
				seen.add(id(mol))
				result.append(mol)
		return result

	#============================================
	@property
	def selected_presentation_objects(self) -> list:
		"""Return selected presentation objects in canonical document order.

		The scene is only an interaction surface: its selected-item order and
		z-order must never become the persistence order for CDML objects.

		Returns:
			Selected PresentationObject instances in ``Document.objects`` order.
		"""
		from ferrum_qt.canvas.graphics_retirement import selected_items_from_captured_scene
		selected_ids = {
			id(getattr(item, "document_object_model", None))
			for item in selected_items_from_captured_scene(self._scene)
		}
		return [
			object_model for object_model in self._object_stack
			if (
				isinstance(
					object_model,
					ferrum_qt.models.document_object.PresentationObject,
				)
				and id(object_model) in selected_ids
			)
		]

	#============================================
	@property
	def selected_presentation_stack_root_ids(self) -> tuple[str, ...]:
		"""Return durable selected presentation roots only when every item is valid.

		Stack reordering is deliberately stricter than ordinary presentation
		selection: every selected scene item must directly identify one supported,
		durable presentation model owned by this exact document.  Child graphics,
		molecule projections, marks, foreign wrappers, and ID-less records make the
		whole request ineligible rather than being silently discarded.
		"""
		# The projection boundary owns proof that a graphics item is a real binding,
		# not merely an arbitrary item carrying a lookalike model attribute.
		import ferrum_qt.canvas.document_projection
		return ferrum_qt.canvas.document_projection.selected_presentation_stack_root_ids(
			self, self._scene,
		)

	#============================================
	@property
	def selected_top_level_objects(self) -> list:
		"""Return selected molecules and artwork in canonical document order.

		Atoms, bonds, and atom-attached marks resolve to the molecule that owns
		their model.  Presentation graphics resolve through their explicit
		``document_object_model`` identity.  This keeps commands independent of
		the incidental order returned by ``QGraphicsScene.selectedItems()``.

		Returns:
			Selected molecule and presentation models in ``Document.objects`` order.
		"""
		from ferrum_qt.canvas.graphics_retirement import selected_items_from_captured_scene
		selected_ids = set()
		for item in selected_items_from_captured_scene(self._scene):
			object_model = getattr(item, "document_object_model", None)
			if object_model in self._presentation_objects:
				selected_ids.add(id(object_model))
			molecule = self.molecule_for_graphics_item(item)
			if molecule is not None:
				selected_ids.add(id(molecule))
		return [
			object_model for object_model in self._object_stack
			if id(object_model) in selected_ids
		]

	#============================================
	@property
	def selected_direct_root_molecule_ids(self) -> tuple[str, ...]:
		"""Return one durable molecule ID per supported selected root.

		This is a frontend selection bridge, not a chemistry conversion.  It
		uses the canonical top-level selection resolver so an atom, bond, or
		attached mark selects its owning direct-root molecule.  This root-only
		observation needs the molecule's durable ID, not a durable child ID:
		compatibility-loaded ID-less children can therefore observe their durable
		root without becoming child-addressable.  A presentation selection or
		mixed molecule/presentation selection is deliberately not a usable
		molecule query target.

		Returns:
			Durable direct-root molecule IDs in canonical document order, or an
			empty tuple when the selection is not wholly supported.
		"""
		objects = self.selected_top_level_objects
		if not objects:
			return ()
		if any(
			not isinstance(object_model, ferrum_qt.models.molecule_model.MoleculeModel)
			or not object_model.mol_id
			for object_model in objects
		):
			return ()
		return tuple(object_model.mol_id for object_model in objects)

	#============================================
	@property
	def has_selection(self) -> bool:
		"""Whether any interactive item is selected."""
		from ferrum_qt.canvas.graphics_retirement import selected_items_from_captured_scene
		return bool(selected_items_from_captured_scene(self._scene))

	#============================================
	def selected_to_unique_top_levels(self) -> tuple:
		"""Dedup selected items to their parent containers.

		Port of Tk paper_selection.selected_to_unique_top_levels().
		Maps atoms and bonds to their parent MoleculeModel, removing
		duplicates. Returns (unique_top_levels, is_unique) where
		is_unique is True when each container had at most one selected
		child.

		Returns:
			Tuple of (list of unique top-level objects, bool is_unique).
		"""
		filtrate = []
		unique = True
		seen_ids = set()
		for atom_item in self.selected_atoms:
			mol = self._find_molecule_for_atom(atom_item.atom_model)
			if mol is not None:
				if id(mol) not in seen_ids:
					seen_ids.add(id(mol))
					filtrate.append(mol)
				else:
					unique = False
		for bond_item in self.selected_bonds:
			mol = self._find_molecule_for_bond(bond_item.bond_model)
			if mol is not None:
				if id(mol) not in seen_ids:
					seen_ids.add(id(mol))
					filtrate.append(mol)
				else:
					unique = False
		return (filtrate, unique)

	#============================================
	@property
	def one_mol_selected(self) -> bool:
		"""Whether exactly one molecule has selected content."""
		return len(self.selected_mols) == 1

	#============================================
	def bonds_to_update(self) -> list:
		"""Return bonds adjacent to selected atoms that need redraw.

		Port of Tk paper_selection.bonds_to_update(). Finds bonds
		connected to any selected atom, excluding bonds that are
		themselves selected.

		Returns:
			List of BondModel instances needing update.
		"""
		import ferrum_qt.canvas.items.bond_item
		if self._scene is None:
			return []
		# collect selected atom models
		selected_atom_models = set()
		for atom_item in self.selected_atoms:
			selected_atom_models.add(id(atom_item.atom_model))
		# collect selected bond models to exclude
		selected_bond_models = set()
		for bond_item in self.selected_bonds:
			selected_bond_models.add(id(bond_item.bond_model))
		# find bonds connected to selected atoms but not themselves selected
		result = []
		for item in self._scene.items():
			if not isinstance(item, ferrum_qt.canvas.items.bond_item.BondItem):
				continue
			bm = item.bond_model
			if id(bm) in selected_bond_models:
				continue
			if bm.atom1 is not None and id(bm.atom1) in selected_atom_models:
				result.append(bm)
			elif bm.atom2 is not None and id(bm.atom2) in selected_atom_models:
				result.append(bm)
		return result

	#============================================
	def atoms_to_update(self) -> list:
		"""Return atoms adjacent to selected bonds that need redraw.

		Port of Tk paper_selection.atoms_to_update(). Finds atoms
		connected to any selected bond, excluding atoms that are
		themselves selected.

		Returns:
			List of AtomModel instances needing update.
		"""
		# collect selected atom models to exclude
		selected_atom_models = set()
		for atom_item in self.selected_atoms:
			selected_atom_models.add(id(atom_item.atom_model))
		# find atoms connected to selected bonds but not themselves selected
		seen = set()
		result = []
		for bond_item in self.selected_bonds:
			bm = bond_item.bond_model
			for atom_model in (bm.atom1, bm.atom2):
				if atom_model is None:
					continue
				if id(atom_model) in selected_atom_models:
					continue
				if id(atom_model) not in seen:
					seen.add(id(atom_model))
					result.append(atom_model)
		return result

	#============================================
	def _find_molecule_for_atom(
			self, atom_model: ferrum_qt.models.atom_model.AtomModel,
			) -> ferrum_qt.models.molecule_model.MoleculeModel | None:
		"""Find the MoleculeModel containing a given AtomModel.

		Args:
			atom_model: AtomModel to search for.

		Returns:
			MoleculeModel or None.
		"""
		for mol_model in self._molecules:
			if atom_model in mol_model.atoms:
				return mol_model
		return None

	#============================================
	def _find_molecule_for_bond(
			self, bond_model: ferrum_qt.models.bond_model.BondModel,
			) -> ferrum_qt.models.molecule_model.MoleculeModel | None:
		"""Find the MoleculeModel containing a given BondModel.

		Args:
			bond_model: BondModel to search for.

		Returns:
			MoleculeModel or None.
		"""
		for mol_model in self._molecules:
			if bond_model in mol_model.bonds:
				return mol_model
		return None

	#============================================
	def molecule_for_graphics_item(
			self, item: PySide6.QtWidgets.QGraphicsItem,
			) -> ferrum_qt.models.molecule_model.MoleculeModel | None:
		"""Resolve a graphics item to its document-owned molecule model."""
		molecule = getattr(item, "molecule_model", None)
		if molecule in self._molecules:
			return molecule
		atom_model = getattr(item, "atom_model", None)
		if atom_model is not None:
			molecule = self._find_molecule_for_atom(atom_model)
			if molecule is not None:
				return molecule
		bond_model = getattr(item, "bond_model", None)
		if bond_model is not None:
			molecule = self._find_molecule_for_bond(bond_model)
			if molecule is not None:
				return molecule
		mark_model = getattr(item, "atom_mark_model", None)
		if mark_model is not None:
			return self._find_molecule_for_atom(mark_model.atom_model)
		from ferrum_qt.canvas.graphics_retirement import native_parent_for_item
		parent_item = native_parent_for_item(item)
		if parent_item is not None:
			return self.molecule_for_graphics_item(parent_item)
		return None

	# ------------------------------------------------------------------
	# Mutation
	# ------------------------------------------------------------------

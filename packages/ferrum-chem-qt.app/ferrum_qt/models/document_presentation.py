"""Presentation-stack and graphics-lifetime helpers for Document."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.models.document_object
import ferrum_qt.models.molecule_model


#============================================
class DocumentPresentation:
	def add_molecule(self, mol_model: ferrum_qt.models.molecule_model.MoleculeModel,
					mark_dirty: bool = True, index: int | None = None) -> None:
		"""Add a molecule to the document.

		Args:
			mol_model: MoleculeModel to add.
			mark_dirty: Whether this direct mutation represents a user edit.
			index: Optional top-level stack insertion position.
		"""
		self.insert_molecule(mol_model, index=index, mark_dirty=mark_dirty)

	#============================================
	def insert_molecule(
			self, mol_model: ferrum_qt.models.molecule_model.MoleculeModel,
			index: int | None = None, mark_dirty: bool = True,
			) -> None:
		"""Insert a molecule at a canonical top-level stack position."""
		if mol_model in self._molecules:
			return
		stack_index = self._normalized_insert_index(index)
		self._molecules.append(mol_model)
		self._object_stack.insert(stack_index, mol_model)
		mol_model.setParent(self)
		self.object_added.emit(mol_model)
		if mark_dirty:
			self.mark_dirty()

	#============================================
	def remove_molecule(self, mol_model: ferrum_qt.models.molecule_model.MoleculeModel,
						mark_dirty: bool = True) -> None:
		"""Remove a molecule from the document.

		Args:
			mol_model: MoleculeModel to remove.
			mark_dirty: Whether this direct mutation represents a user edit.

		Raises:
			ValueError: If the molecule is not in the document.
		"""
		self._molecules.remove(mol_model)
		self._object_stack.remove(mol_model)
		mol_model.setParent(None)
		self.object_removed.emit(mol_model)
		if mark_dirty:
			self.mark_dirty()

	#============================================
	def add_presentation_object(
			self,
			object_model: ferrum_qt.models.document_object.PresentationObject,
			mark_dirty: bool = True,
			index: int | None = None,
			) -> None:
		"""Add a non-molecule object to the ordered document stack.

		Args:
			object_model: Presentation object to own.
			mark_dirty: Whether this direct mutation represents a user edit.
			index: Optional top-level stack insertion position.
		"""
		self.insert_presentation_object(
			object_model, index=index, mark_dirty=mark_dirty,
		)

	#============================================
	def insert_presentation_object(
			self,
			object_model: ferrum_qt.models.document_object.PresentationObject,
			index: int | None = None, mark_dirty: bool = True,
			) -> None:
		"""Insert presentation artwork at a canonical stack position."""
		if object_model in self._presentation_objects:
			return
		stack_index = self._normalized_insert_index(index)
		self._presentation_objects.append(object_model)
		self._object_stack.insert(stack_index, object_model)
		object_model.setParent(self)
		self.object_added.emit(object_model)
		if mark_dirty:
			self.mark_dirty()

	#============================================
	def object_index(self, object_model: object) -> int:
		"""Return the identity-based index of one top-level document object.

		Raises:
			ValueError: If ``object_model`` is not owned by this document.
		"""
		for index, current_object in enumerate(self._object_stack):
			if current_object is object_model:
				return index
		raise ValueError("Object is not owned by this document")

	#============================================
	def replace_object_order(self, objects: list, mark_dirty: bool = True) -> None:
		"""Replace the top-level stack after validating exact object identity.

		The supplied sequence must contain every current object exactly once.
		This prevents a reorder command from silently dropping or duplicating a
		model when two distinct QObject wrappers happen to compare alike.
		"""
		if len(objects) != len(self._object_stack):
			raise ValueError("Object order must contain every document object")
		current_ids = {id(object_model) for object_model in self._object_stack}
		proposed_ids = [id(object_model) for object_model in objects]
		if len(set(proposed_ids)) != len(proposed_ids) or set(proposed_ids) != current_ids:
			raise ValueError("Object order must contain each document object once")
		self._object_stack = list(objects)
		self._synchronize_scene_object_stack()
		if mark_dirty:
			self.mark_dirty()

	#============================================
	def _normalized_insert_index(self, index: int | None) -> int:
		"""Return a Python-list insertion index for a top-level object."""
		if index is None:
			return len(self._object_stack)
		if index < 0 or index > len(self._object_stack):
			raise IndexError("Object insertion index is outside the document stack")
		return index

	#============================================
	def _synchronize_scene_object_stack(self) -> None:
		"""Refresh projected z values after a top-level order replacement."""
		if self._scene is None:
			return
		import ferrum_qt.canvas.document_projection
		ferrum_qt.canvas.document_projection.synchronize_document_stack_z_order(
			self, self._scene,
		)

	#============================================
	def remove_presentation_object(
			self,
			object_model: ferrum_qt.models.document_object.PresentationObject,
			mark_dirty: bool = True,
			) -> None:
		"""Remove a non-molecule object from the document stack.

		Args:
			object_model: Presentation object to detach.
			mark_dirty: Whether this direct mutation represents a user edit.

		Raises:
			ValueError: If the object is not in this document.
		"""
		self._presentation_objects.remove(object_model)
		self._object_stack.remove(object_model)
		object_model.setParent(None)
		self.object_removed.emit(object_model)
		if mark_dirty:
			self.mark_dirty()

	#============================================
	def add_mark(
			self,
			mark_model: ferrum_qt.models.document_object.AtomMarkModel,
			mark_dirty: bool = True,
			) -> None:
		"""Add an atom-attached mark to the document.

		Args:
			mark_model: Atom mark model to own.
			mark_dirty: Whether this direct mutation represents a user edit.
		"""
		if mark_model in self._marks:
			return
		self._marks.append(mark_model)
		mark_model.setParent(self)
		self.mark_added.emit(mark_model)
		if mark_dirty:
			self.mark_dirty()

	#============================================
	def remove_mark(
			self,
			mark_model: ferrum_qt.models.document_object.AtomMarkModel,
			mark_dirty: bool = True,
			) -> None:
		"""Remove an atom-attached mark from the document.

		Args:
			mark_model: Atom mark model to detach.
			mark_dirty: Whether this direct mutation represents a user edit.

		Raises:
			ValueError: If the mark is not in this document.
		"""
		self._marks.remove(mark_model)
		mark_model.setParent(None)
		self.mark_removed.emit(mark_model)
		if mark_dirty:
			self.mark_dirty()

	#============================================
	def set_cdml_state(
			self,
			envelope: ferrum_qt.models.document_object.CdmlEnvelope,
			paper: ferrum_qt.models.document_object.PaperModel,
			unsupported_content: list[ferrum_qt.models.document_object.UnsupportedContent],
			) -> None:
		"""Install parsed CDML metadata as a clean document baseline.

		Args:
			envelope: Root, header, reaction, and external-data state.
			paper: Paper and viewport state.
			unsupported_content: Warnings for content without a UI representation.
		"""
		self._cdml_envelope = envelope
		self._paper = paper
		self._unsupported_content = list(unsupported_content)
		self.paper_changed.emit(paper)
		self.mark_clean()

	#============================================
	def replace_paper(self, replacement: ferrum_qt.models.document_object.PaperModel) -> None:
		"""Replace the modeled paper state and notify its scene projection.

		The existing PaperModel remains document-owned so callers that retain the
		model during a dialog or undo command do not acquire a stale object.
		"""
		self._paper.replace(replacement)
		self.paper_changed.emit(self._paper)

	#============================================
	def unique_object_id(self, prefix: str) -> str:
		"""Return the first unused stable top-level CDML identifier."""
		existing_ids = set()
		for object_model in self._object_stack:
			object_id = getattr(object_model, "object_id", None)
			if object_id is None:
				object_id = getattr(object_model, "mol_id", None)
			if object_id:
				existing_ids.add(str(object_id))
		index = 1
		candidate = f"{prefix}-{index}"
		while candidate in existing_ids:
			index += 1
			candidate = f"{prefix}-{index}"
		return candidate

	#============================================
	def clear(self) -> None:
		"""Remove all document-owned state and reset to an empty baseline.

		Graphics projections are deliberately disconnected before their models or
		undo commands can release their final Python references.  This mirrors
		:class:`DocumentSession` teardown while leaving ChemScene's paper and grid
		decorations in place for the next document.
		"""
		first_error = None
		try:
			self._dispose_document_graphics()
		except Exception as exc:
			# Continue severing QObject model ownership.  A graphics callback must
			# never make a partially disposed Document safe to reuse or delete by
			# parent cascade.
			first_error = exc
		for object_model in list(self._object_stack):
			try:
				object_model.setParent(None)
			except Exception as exc:
				if first_error is None:
					first_error = exc
			try:
				self.object_removed.emit(object_model)
			except Exception as exc:
				if first_error is None:
					first_error = exc
		for mark_model in list(self._marks):
			try:
				mark_model.setParent(None)
			except Exception as exc:
				if first_error is None:
					first_error = exc
			try:
				self.mark_removed.emit(mark_model)
			except Exception as exc:
				if first_error is None:
					first_error = exc
		self._molecules.clear()
		self._object_stack.clear()
		self._presentation_objects.clear()
		self._marks.clear()
		self._paper = ferrum_qt.models.document_object.PaperModel()
		self._cdml_envelope = ferrum_qt.models.document_object.CdmlEnvelope()
		self._unsupported_content.clear()
		self._file_path = None
		self._direct_dirty = False
		try:
			self._undo_stack.clear()
			self._undo_stack.setClean()
		except Exception as exc:
			if first_error is None:
				first_error = exc
		self.paper_changed.emit(self._paper)
		self._sync_dirty_state()
		if first_error is not None:
			raise RuntimeError("Document was cleared after a disposal failure") from first_error

	#============================================
	def _dispose_document_graphics(
			self,
			reaper: "ferrum_qt.canvas.graphics_retirement.DetachedGraphicsRetirementReaper | None" = None,
			) -> None:
		"""Disconnect and detach graphics owned by the current document.

		The undo command module imports :mod:`document`, so its helper is imported
		locally after this model is fully initialized.  A document may share a
		ChemScene with persistent paper/grid decorations; only items that expose a
		model from this document are detached here.
		"""
		# A session installs its terminal reaper once, when it adopts this
		# projection.  Callers that clear the document later must keep using that
		# owner rather than silently falling back to the process reaper.
		if reaper is None:
			reaper = self._effective_terminal_graphics_reaper()
		first_error = None
		if self._scene is not None:
			from ferrum_qt.canvas.graphics_retirement import GraphicsRetirementCoordinator
			# Registered wrappers are the exact active projection and its explicit
			# Python-owned lifetime.  Legacy locally constructed Documents may not
			# have crossed the registration boundary, so retain the model-based scan
			# only as their compatibility fallback.
			items = list(self._projection_item_refs.values())
			if not items:
				owned_model_ids = self._owned_graphics_model_ids()
				items = [
					item for item in self._scene.items()
					if self._item_belongs_to_document(item, owned_model_ids)
				]
			coordinator = GraphicsRetirementCoordinator()
			# The live scene owns the applied projection tree.  Detached graphics
			# retained by commands are a separate terminal transition owned by
			# DocumentUndoStack.clear(), so do not walk the undo stack here.
			coordinator.retire_scene_projection_items(self._scene, items, reaper=reaper)
			# The retirement coordinator now owns detached wrappers through its
			# terminal reaper.  Releasing the document's active ownership here makes
			# native destruction an explicit transition rather than GC timing.
			self._projection_item_refs.clear()
			if coordinator.report.callback_errors:
				first_error = coordinator.report.callback_errors[0]
		else:
			from ferrum_qt.canvas.graphics_retirement import GraphicsRetirementCoordinator
			coordinator = GraphicsRetirementCoordinator()
			# With no scene, this method has no current projection roots to retire.
			# The following DocumentUndoStack.clear() owns any detached history tree.
			self._projection_item_refs.clear()
		if first_error is not None:
			raise RuntimeError(
				"Document graphics were detached after a disposal failure",
			) from first_error

	#============================================
	def _owned_graphics_model_ids(self) -> set:
		"""Return identities exposed by current document graphics items."""
		owned_model_ids = {
			id(object_model) for object_model in self._object_stack
		}
		owned_model_ids.update(id(mark_model) for mark_model in self._marks)
		for molecule in self._molecules:
			owned_model_ids.update(id(atom_model) for atom_model in molecule.atoms)
			owned_model_ids.update(id(bond_model) for bond_model in molecule.bonds)
			owned_model_ids.update(id(group_model) for group_model in molecule.groups)
		return owned_model_ids

	#============================================
	def _item_belongs_to_document(
			self, item: PySide6.QtWidgets.QGraphicsItem,
			owned_model_ids: set,
			) -> bool:
		"""Return whether an item exposes a current document model identity."""
		for attribute in (
				"document_object_model", "atom_mark_model", "atom_model",
				"bond_model", "group_model",
			):
			model = getattr(item, attribute, None)
			if model is not None and id(model) in owned_model_ids:
				return True
		return False

	# ------------------------------------------------------------------
	# File info
	# ------------------------------------------------------------------


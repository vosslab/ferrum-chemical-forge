"""Persistent and local deletion behavior for :class:`EditMode`."""

# Standard Library
import importlib

# local repo modules
import ferrum_qt.canvas.document_projection
import ferrum_qt.canvas.items.atom_item
import ferrum_qt.canvas.items.bond_item
import ferrum_qt.canvas.items.mark_item
import ferrum_qt.undo.commands


#============================================
class EditDeleteMixin:
	"""Own Delete routing and local chemistry-aware cleanup."""

	def _delete_selected(self) -> None:
		"""Delete all selected items with chemistry-aware cleanup.

		Port of Tk paper_selection.delete_selected(). Handles:
		- Bonds: remove from graph, then remove orphan atoms (atoms
		  with no remaining bonds and not themselves selected).
		- Atoms: remove from molecule with connected bond cleanup.
		- Molecule integrity: after deletion, check if the parent
		  molecule has become disconnected and split into separate
		  MoleculeModel instances.

		All deletions are grouped in a single undo macro.
		"""
		scene = self._env.scene
		if scene is None:
			return
		selected = scene.selectedItems()
		if not selected:
			return
		if self._submit_top_level_delete(selected):
			return
		if self._submit_atom_mark_delete(selected):
			return
		if self._submit_structure_delete(selected):
			return
		undo_stack = self._env.undo_stack
		if undo_stack is None:
			return
		# Separate chemistry items from document-owned presentation artwork.
		atom_items = []
		bond_items = []
		presentation_items = []
		for item in selected:
			if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
				atom_items.append(item)
			elif isinstance(item, ferrum_qt.canvas.items.bond_item.BondItem):
				bond_items.append(item)
			elif self._is_presentation_item(item):
				presentation_items.append(item)
		if not atom_items and not bond_items and not presentation_items:
			return
		# collect atom model ids for quick lookup
		selected_atom_ids = set(id(ai.atom_model) for ai in atom_items)
		# use a macro to group all deletions
		undo_stack.beginMacro("Delete Selected")
		for item in presentation_items:
			undo_stack.push(
				ferrum_qt.undo.commands.RemovePresentationObjectCommand(
					self._env.document, scene, item.document_object_model, item,
				),
			)
		# remove bonds first
		for bond_item in bond_items:
			mol_model = self._env.find_molecule_for_bond(bond_item.bond_model)
			if mol_model is not None:
				cmd = ferrum_qt.undo.commands.RemoveBondCommand(
					scene, mol_model, bond_item.bond_model, bond_item,
				)
				undo_stack.push(cmd)
		# remove atoms (which also removes their connected bonds)
		for atom_item in atom_items:
			mol_model = self._env.find_molecule_for_atom(atom_item.atom_model)
			if mol_model is not None:
				# find connected bond items not already removed
				connected_bonds = self._env.find_connected_bond_items(atom_item.atom_model)
				cmd = ferrum_qt.undo.commands.RemoveAtomCommand(
					scene, mol_model, atom_item.atom_model,
					atom_item, connected_bonds,
				)
				undo_stack.push(cmd)
		# orphan cleanup: after bond deletion, remove atoms that have
		# no remaining bonds and were not explicitly selected
		orphan_atoms = self._find_orphan_atoms_after_bond_delete(
			bond_items, selected_atom_ids,
		)
		for orphan_item in orphan_atoms:
			mol_model = self._env.find_molecule_for_atom(orphan_item.atom_model)
			if mol_model is not None:
				connected_bonds = self._env.find_connected_bond_items(orphan_item.atom_model)
				cmd = ferrum_qt.undo.commands.RemoveAtomCommand(
					scene, mol_model, orphan_item.atom_model,
					orphan_item, connected_bonds,
				)
				undo_stack.push(cmd)
		undo_stack.endMacro()
		self.status_message.emit("Deleted selected items")

	#============================================
	def _submit_top_level_delete(self, selected: list) -> bool:
		"""Route one complete direct-root selection through its proven authority.

		Partial atom/bond selections intentionally return ``False`` so their
		legacy structural-delete grammar continues to own the interaction.
		"""
		scene = self._env.scene
		document = self._env.document
		if scene is None or document is None:
			return False
		allowed_presentation = {
			"arrow", "plus", "text", "rect", "square", "oval", "circle",
			"polygon", "polyline",
		}
		selected_set = set(selected)
		for item in selected:
			if not document.is_current_projection_item(item):
				return False
			if isinstance(item, (
				ferrum_qt.canvas.items.atom_item.AtomItem,
				ferrum_qt.canvas.items.bond_item.BondItem,
			)):
				continue
			if not self._is_presentation_item(item):
				return False
			model = item.document_object_model
			if (
				not getattr(model, "editable", False)
				or not getattr(model, "object_id", "")
				or getattr(model, "kind", "") not in allowed_presentation
			):
				return False
		root_ids = []
		target_keys = set()
		for object_model in document.selected_top_level_objects:
			molecule_id = getattr(object_model, "mol_id", "")
			if molecule_id:
				primary_items = [
					item for item in scene.items()
					if isinstance(item, (
						ferrum_qt.canvas.items.atom_item.AtomItem,
						ferrum_qt.canvas.items.bond_item.BondItem,
					)) and document.molecule_for_graphics_item(item) is object_model
				]
				if not primary_items or any(item not in selected_set for item in primary_items):
					return False
				root_ids.append(molecule_id)
				target_keys.add(("molecule", molecule_id))
				continue
			object_id = getattr(object_model, "object_id", "")
			if (
				not getattr(object_model, "editable", False)
				or not object_id
				or getattr(object_model, "kind", "") not in allowed_presentation
			):
				return False
			root_ids.append(object_id)
			target_keys.add(("presentation", object_id))
		if not root_ids:
			return False
		# A selection may include every molecule primary item plus a mark.  Marks
		# are view-only children for this operation and need no durable root key.
		covered = set()
		for item in selected:
			if self._is_presentation_item(item):
				covered.add(item)
			elif isinstance(item, (
				ferrum_qt.canvas.items.atom_item.AtomItem,
				ferrum_qt.canvas.items.bond_item.BondItem,
			)):
				covered.add(item)
		if covered != selected_set:
			return False
		context = self._top_level_delete_context
		if context is None:
			return False
		authority_and_revision = context()
		if (
			type(authority_and_revision) is not tuple
			or len(authority_and_revision) != 2
			or authority_and_revision[0] not in ("backend", "local", "unavailable")
		):
			raise ValueError("Top-level Delete context returned an unknown state")
		authority, expected_revision = authority_and_revision
		if authority == "local":
			return False
		if authority == "unavailable":
			if expected_revision is not None:
				raise ValueError("Unavailable top-level Delete must not capture a revision")
			self.status_message.emit("Delete unavailable for this document")
			return True
		if type(expected_revision) is not int or self._persistent_operation is None:
			raise ValueError("Backend top-level Delete requires a captured revision")
		document_session = importlib.import_module("ferrum_qt.models.document_session")
		request = document_session.PersistentOperationRequest(
			"top-level.delete", "Delete",
			(("expected_revision", expected_revision), ("root_ids", tuple(root_ids))),
			frozenset(target_keys),
		)
		outcome = self._persistent_operation(request)
		# A complete supported selection has crossed the authoritative operation
		# boundary. Its accepted, stale, or rejected result is final for this
		# Delete gesture; the local structural grammar applies only to a local
		# authority state or a selection that failed the eligibility gate above.
		self.status_message.emit(outcome.message)
		return True

	#============================================
	def _submit_atom_mark_delete(self, selected: list) -> bool:
		"""Delete one selected current atom mark through backend authority only."""
		has_mark = any(
			isinstance(item, ferrum_qt.canvas.items.mark_item.MarkItem)
			for item in selected
		)
		if not has_mark:
			return False
		context = self._atom_mark_delete_context
		if context is None:
			return False
		authority_and_revision = context()
		if (
			type(authority_and_revision) is not tuple
			or len(authority_and_revision) != 2
			or authority_and_revision[0] not in ("backend", "local", "unavailable")
			):
			raise ValueError("Atom-mark Delete context returned an unknown state")
		authority, expected_revision = authority_and_revision
		if authority == "local":
			return False
		if authority == "unavailable":
			if expected_revision is not None:
				raise ValueError("Unavailable atom-mark Delete must not capture a revision")
			self.status_message.emit("Delete unavailable for this document")
			return True
		document = self._env.document
		if document is None or type(expected_revision) is not int or self._persistent_operation is None:
			return True
		items = tuple(selected)
		target = ferrum_qt.canvas.document_projection.atom_mark_delete_target_for_items(
			document, items,
		)
		# A synchronized mark selection is never allowed to fall into Qt undo.
		# Keep only plain durable intent before projection can retire its wrappers.
		selected.clear()
		del items
		if target is None:
			self.status_message.emit("Delete requires one current supported atom mark")
			return True
		molecule_id, atom_id, mark_type, matching_mark_index = target
		del target
		del document
		document_session = importlib.import_module("ferrum_qt.models.document_session")
		request = document_session.build_atom_mark_request(
			expected_revision, molecule_id, atom_id, "remove", mark_type,
			matching_mark_index,
		)
		outcome = self._persistent_operation(request)
		self.status_message.emit(outcome.message)
		return True

	#============================================
	def _submit_structure_delete(self, selected: list) -> bool:
		"""Route one partial atom/bond selection through its explicit authority."""
		context = self._structure_delete_context
		if context is None:
			return False
		authority_and_revision = context()
		if (
			type(authority_and_revision) is not tuple
			or len(authority_and_revision) != 2
			or authority_and_revision[0] not in ("backend", "local", "unavailable")
		):
			raise ValueError("Structure Delete context returned an unknown state")
		authority, expected_revision = authority_and_revision
		if authority == "local":
			return False
		if authority == "unavailable":
			if expected_revision is not None:
				raise ValueError("Unavailable Structure Delete must not capture a revision")
			self.status_message.emit("Delete unavailable for this document")
			return True
		if type(expected_revision) is not int or self._persistent_operation is None:
			self.status_message.emit("Delete unavailable for this document")
			return True
		document = self._env.document
		if document is None:
			self.status_message.emit("Delete unavailable for this document")
			return True
		items = tuple(selected)
		targets = (
			ferrum_qt.canvas.document_projection.structure_delete_targets_for_items(
				document, items,
			)
		)
		del items
		if targets is None:
			self.status_message.emit(
				"Delete unavailable: select durable atoms or bonds from one molecule",
			)
			return True
		molecule_id, atom_ids, bond_ids = targets
		document_session = importlib.import_module("ferrum_qt.models.document_session")
		request = document_session.build_structure_delete_request(
			expected_revision, molecule_id, atom_ids, bond_ids,
		)
		# Accepted submission may synchronously retire every selected wrapper.
		# Release this frame's list before crossing that replacement boundary.
		selected.clear()
		del targets
		outcome = self._persistent_operation(request)
		self.status_message.emit(outcome.message)
		return True

	#============================================
	def _find_orphan_atoms_after_bond_delete(
			self, deleted_bond_items: list, selected_atom_ids: set,
			) -> list:
		"""Find atoms that became orphans after bond deletion.

		An orphan is an atom that: (1) was an endpoint of a deleted bond,
		(2) was not itself selected for deletion, and (3) has no remaining
		bonds in the scene after the deletion.

		Args:
			deleted_bond_items: List of BondItems that were deleted.
			selected_atom_ids: Set of id() values for explicitly selected atoms.

		Returns:
			List of AtomItem instances that are orphaned.
		"""
		scene = self._env.scene
		if scene is None:
			return []
		# collect candidate atoms from deleted bonds
		candidates = {}
		for bond_item in deleted_bond_items:
			bm = bond_item.bond_model
			for atom_model in (bm.atom1, bm.atom2):
				if atom_model is None:
					continue
				if id(atom_model) in selected_atom_ids:
					continue
				candidates[id(atom_model)] = atom_model
		if not candidates:
			return []
		# find AtomItems for candidates and check if they have remaining bonds
		orphans = []
		for item in scene.items():
			if not isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
				continue
			if id(item.atom_model) not in candidates:
				continue
			# check if this atom has any remaining bonds in the scene
			has_bonds = False
			for other_item in scene.items():
				if not isinstance(other_item, ferrum_qt.canvas.items.bond_item.BondItem):
					continue
				bm = other_item.bond_model
				if bm.atom1 is item.atom_model or bm.atom2 is item.atom_model:
					has_bonds = True
					break
			if not has_bonds:
				orphans.append(item)
		return orphans

	#============================================

"""Main application window for Ferrum-Qt."""

# Standard Library

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.config.geometry_units
import ferrum_qt.config.keybindings
import ferrum_qt.config.preferences
import ferrum_qt.widgets.status_bar
import ferrum_qt.widgets.zoom_controls
import ferrum_qt.widgets.icon_loader
import ferrum_qt.setup.canvas_setup
import ferrum_qt.setup.mode_setup
import ferrum_qt.setup.toolbar_setup
import ferrum_qt.actions.file_actions
import ferrum_qt.actions.options_actions
import ferrum_qt.canvas.document_projection
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.canvas.molecule_projection
import ferrum_qt.io.clipboard_manager
import ferrum_qt.io.import_capabilities
import ferrum_qt.io.user_template_catalog
import ferrum_qt.bridge.user_template_inspection
import ferrum_qt.dialogs.about_dialog
import ferrum_qt.dialogs.preferences_dialog
import ferrum_qt.dialogs.theme_chooser_dialog

import ferrum_qt.window_shared

_PendingSessionDeletion = ferrum_qt.window_shared._PendingSessionDeletion
ShutdownState = ferrum_qt.window_shared.ShutdownState


#============================================
class WindowClipboardMixin:
	"""Cohesive MainWindow behavior with no MainWindow import."""

	def on_cut(self) -> None:
		"""Copy one selection, then delete its durable roots when synchronized."""
		target = self._active_cut_session()
		if target is None:
			return
		if target.legacy_isolated:
			self._cut_legacy_isolated(target)
			return
		self._cut_synchronized(target)
	def _cut_synchronized(
			self, target: ferrum_qt.models.document_session.DocumentSession,
			) -> None:
		"""Copy first, then submit a request frozen from one synchronized tab.

		Clipboard delivery is an application callback boundary.  This method
		captures its complete plain backend request before that boundary and
		releases every Qt projection reference before the eventual submission.
		"""
		document = target.document
		scene = target.scene
		if document is None or scene is None or not document.has_selection:
			return
		structural_targets = self._selected_cut_structural_targets(document, scene)
		if structural_targets is False:
			self.statusBar().showMessage(
				self.tr("Cut selection cannot be committed"), 3000,
			)
			return
		if structural_targets is not None:
			# Structural extraction can accept and reproject synchronously.  Its
			# immutable targets are the only projection-derived values it needs.
			del document
			del scene
			self._cut_synchronized_structure(target, structural_targets)
			return
		targets = self._selected_cut_root_targets(document, scene)
		request = None
		submit = None
		fragment_cdml = None
		if targets is not None and target.can_commit_persistent_action:
			root_ids, target_keys = targets
			try:
				submit = self.persistent_operation_capability_for(target)
			except ValueError:
				submit = None
			if submit is not None:
				revision = target.backend_snapshot.revision
				request = ferrum_qt.models.document_session.PersistentOperationRequest(
					"top-level.delete", "Cut",
					(("expected_revision", revision), ("root_ids", root_ids)), target_keys,
				)
				try:
					fragment = target.extract_top_level_fragment(
						revision, root_ids,
					)
					fragment_cdml = fragment.fragment_cdml
					del fragment
				except ValueError as exc:
					self.statusBar().showMessage(str(exc), 3000)
					return
		# Keep only session-bound callable and immutable plain request data after
		# the callback boundary.  An accepted commit can now replace this entire
		# projection without an old wrapper remaining in this stack frame.
		del document
		del scene
		del targets
		del target
		if fragment_cdml is None:
			self.statusBar().showMessage(
				self.tr("Cut selection cannot be committed"), 3000,
			)
			return
		try:
			self._clipboard_manager.publish_fragment(fragment_cdml)
		except (RuntimeError, TypeError, ValueError) as exc:
			self.statusBar().showMessage(
				self.tr("Could not copy selection; nothing was cut: %s") % exc, 3000,
			)
			return
		if submit is None or request is None:
			self.statusBar().showMessage(
				self.tr("Cut selection cannot be committed"), 3000,
			)
			self._refresh_document_actions()
			return
		# The captured capability is session-bound.  It remains attached to the
		# originating tab across tab activation, while its own liveness predicate
		# rejects disposal, stale projection, and legacy-isolation transitions.
		outcome = submit(request)
		self._show_persistent_action_outcome(outcome)
		self._refresh_document_actions()
	def _selected_cut_structural_targets(
			self, document: object, scene: object,
			) -> tuple[str, tuple[str, ...], tuple[str, ...]] | bool | None:
		"""Resolve an exact direct atom/bond Cut selection, or reject its mixture."""
		items = tuple(scene.selectedItems())
		if not items:
			return None
		classification = ferrum_qt.canvas.document_projection.classify_structural_selection(
			document, items,
		)
		if classification.kind is ferrum_qt.canvas.document_projection.StructuralSelectionKind.EXACT:
			return classification.targets
		if classification.kind is ferrum_qt.canvas.document_projection.StructuralSelectionKind.INVALID:
			return False
		for item in items:
			# A structural wrapper must prove membership before any native model
			# field is observed.  Unsupported marks and structural/presentation
			# mixtures are inert for this bounded partial-Cut grammar.
			if not document.is_current_projection_item(item):
				return False
			if document.molecule_for_current_projection_item(item) is not None:
				return False
		return None
	def _cut_synchronized_structure(
			self, target: ferrum_qt.models.document_session.DocumentSession,
			targets: tuple[str, tuple[str, ...], tuple[str, ...]],
			) -> None:
		"""Extract, publish, then delete one backend-authoritative subgraph."""
		molecule_id, atom_ids, bond_ids = targets
		if not target.can_commit_persistent_action:
			self.statusBar().showMessage(self.tr("Cut selection cannot be committed"), 3000)
			return
		revision = target.backend_snapshot.revision
		try:
			submit = self.persistent_operation_capability_for(target)
			request = ferrum_qt.models.document_session.build_structure_delete_request(
				revision, molecule_id, atom_ids, bond_ids,
			)
			fragment = target.extract_structure_fragment(
				revision, molecule_id, atom_ids, bond_ids,
			)
		except ValueError as exc:
			self.statusBar().showMessage(str(exc), 3000)
			return
		fragment_cdml = fragment.fragment_cdml
		del fragment
		del target
		try:
			self._clipboard_manager.publish_fragment(fragment_cdml)
		except (RuntimeError, TypeError, ValueError) as exc:
			self.statusBar().showMessage(
				self.tr("Could not copy selection; nothing was cut: %s") % exc, 3000,
			)
			return
		outcome = submit(request)
		self._show_persistent_action_outcome(outcome)
		self._refresh_document_actions()
	def _cut_legacy_isolated(
			self, target: ferrum_qt.models.document_session.DocumentSession,
			) -> None:
		"""Run legacy Cut only after its isolated projection proves unchanged."""
		document = target.document
		if document is None or not document.has_selection:
			return
		selected_object_ids = tuple(
			id(object_model) for object_model in document.selected_top_level_objects
		)
		if not selected_object_ids:
			return
		document_identity = id(document)
		persistent_generation = document.persistent_generation
		try:
			count = self._clipboard_manager.copy_selection(document)
		except ValueError as exc:
			self.statusBar().showMessage(str(exc), 3000)
			return
		del document
		if count == 0:
			self.statusBar().showMessage(
				self.tr("Could not copy selection; nothing was cut"), 3000,
			)
			return
		current = self._active_cut_session()
		if (
			current is not target
			or not current.legacy_isolated
			or current.document is None
			or id(current.document) != document_identity
			or current.document.persistent_generation != persistent_generation
			or tuple(
				id(object_model)
				for object_model in current.document.selected_top_level_objects
			) != selected_object_ids
		):
			self.statusBar().showMessage(
				self.tr("Cut no longer applies to this document"), 3000,
			)
			self._refresh_document_actions()
			return
		self._remove_top_level_objects(current.document.selected_top_level_objects)
		self.statusBar().showMessage(self.tr("Cut %d object(s)") % count, 3000)
	def _active_cut_session(self) -> ferrum_qt.models.document_session.DocumentSession | None:
		"""Return the exact live session represented by current Cut aliases."""
		target = self._active_session
		if (
			target is None
			or target.is_disposed
			or target not in self._sessions
			or target.document is not self._document
			or target.scene is not self._scene
			or target.view is not self._view
		):
			return None
		return target
	def _selected_cut_root_targets(
			self, document: object, scene: object,
			) -> tuple[tuple[str, ...], frozenset[tuple[str, str]]] | None:
		"""Capture durable direct roots from one current selected projection.

		This frontend-only bridge resolves atom, bond, and mark hits to their
		owning molecule before producing plain immutable root IDs for OASA.
		Every selected graphics item must prove current document ownership so a
		foreign, stale, unsupported, or ID-less wrapper cannot downgrade a
		synchronized Cut into a local mutation.
		"""
		selected_items = tuple(scene.selectedItems())
		objects = tuple(document.selected_top_level_objects)
		if not selected_items or not objects:
			return None
		selected_model_ids = {id(object_model) for object_model in objects}
		for item in selected_items:
			# Atom, bond, and mark projections may also expose their child model as
			# ``document_object_model``.  Their owning molecule takes precedence;
			# only otherwise-unowned items are presentation-root candidates.
			if not document.is_current_projection_item(item):
				return None
			molecule = document.molecule_for_current_projection_item(item)
			if molecule is not None:
				if id(molecule) not in selected_model_ids:
					return None
				continue
			model = getattr(item, "document_object_model", None)
			if model is not None:
				if (
					id(model) not in selected_model_ids
					or model not in document.presentation_objects
					or not getattr(model, "supported", False)
					or not ferrum_qt.canvas.document_projection.is_bound_presentation_projection(
						item, model,
					)
				):
					return None
				continue
			if model is None:
				return None
		root_ids = []
		target_keys = set()
		for object_model in objects:
			molecule_id = getattr(object_model, "mol_id", "")
			if molecule_id:
				if object_model not in document.molecules:
					return None
				root_ids.append(molecule_id)
				target_keys.add(("molecule", molecule_id))
				continue
			object_id = getattr(object_model, "object_id", "")
			if (
				object_model not in document.presentation_objects
				or not getattr(object_model, "supported", False)
				or not isinstance(object_id, str)
				or not object_id
			):
				return None
			root_ids.append(object_id)
			target_keys.add(("presentation", object_id))
		if len(root_ids) != len(set(root_ids)):
			return None
		return tuple(root_ids), frozenset(target_keys)
	def on_copy(self) -> None:
		"""Copy an exact structural selection or existing selected top-level roots."""
		target = self._active_cut_session()
		if target is not None and not target.legacy_isolated:
			document = target.document
			scene = target.scene
			if document is not None and scene is not None and document.has_selection:
				classification = ferrum_qt.canvas.document_projection.classify_structural_selection(
					document, tuple(scene.selectedItems()),
				)
				if classification.kind is ferrum_qt.canvas.document_projection.StructuralSelectionKind.EXACT:
					targets = classification.targets
					fragment_cdml = self._extract_synchronized_structure_fragment(target, targets)
					del document
					del scene
					del classification
					del targets
					del target
					if fragment_cdml is not None:
						self._publish_structural_copy_fragment(fragment_cdml)
					return
				if classification.kind is ferrum_qt.canvas.document_projection.StructuralSelectionKind.INVALID:
					self.statusBar().showMessage(self.tr("Copy selection cannot be copied"), 3000)
					return
				del classification
				root_targets = self._selected_cut_root_targets(document, scene)
				if root_targets is None:
					self.statusBar().showMessage(self.tr("Copy selection cannot be copied"), 3000)
					return
				root_ids, _target_keys = root_targets
				revision = target.backend_snapshot.revision
				try:
					fragment = target.extract_top_level_fragment(revision, root_ids)
				except ValueError as exc:
					self.statusBar().showMessage(str(exc), 3000)
					return
				fragment_cdml = fragment.fragment_cdml
				del fragment
				del root_targets
				del root_ids
				del _target_keys
				del revision
				del document
				del scene
				del target
				self._publish_synchronized_top_level_fragment(fragment_cdml)
				return
			del document
			del scene
		# The native clipboard can synchronously invoke application callbacks.  Both
		# synchronized root/mixed Copy and legacy-isolated whole-root Copy reach the
		# shared publication path below, so neither may retain its origin session.
		if target is not None:
			del target
		try:
			count = self._clipboard_manager.copy_selection(self._document)
		except ValueError as exc:
			self.statusBar().showMessage(str(exc), 3000)
			return
		if count == 0:
			self.statusBar().showMessage(
				self.tr("Nothing selected to copy"), 3000,
			)
			return
		self.statusBar().showMessage(
			self.tr("Copied %d object(s)") % count, 3000,
		)
	def _publish_synchronized_top_level_fragment(self, fragment_cdml: str) -> None:
		"""Publish OASA-owned direct-root CDML after wrappers leave scope."""
		try:
			self._clipboard_manager.publish_fragment(fragment_cdml)
		except (RuntimeError, TypeError, ValueError) as exc:
			self.statusBar().showMessage(
				self.tr("Could not copy selection: %s") % exc, 3000,
			)
			return
		self.statusBar().showMessage(self.tr("Copied selection"), 3000)
	def _extract_synchronized_structure_fragment(
			self, target: ferrum_qt.models.document_session.DocumentSession,
			targets: tuple[str, tuple[str, ...], tuple[str, ...]] | None,
			) -> str | None:
		"""Extract one read-only authoritative fragment before native publication."""
		if targets is None:
			return None
		molecule_id, atom_ids, bond_ids = targets
		revision = target.backend_snapshot.revision
		try:
			fragment = target.extract_structure_fragment(
				revision, molecule_id, atom_ids, bond_ids,
			)
		except ValueError as exc:
			self.statusBar().showMessage(str(exc), 3000)
			return None
		fragment_cdml = fragment.fragment_cdml
		del fragment
		return fragment_cdml
	def _publish_structural_copy_fragment(self, fragment_cdml: str) -> None:
		"""Publish raw structural CDML after all origin projection state is gone."""
		try:
			self._clipboard_manager.publish_fragment(fragment_cdml)
		except (RuntimeError, TypeError, ValueError) as exc:
			self.statusBar().showMessage(
				self.tr("Could not copy selection: %s") % exc, 3000,
			)
			return
		self.statusBar().showMessage(self.tr("Copied 1 object(s)"), 3000)
	def on_paste(self) -> None:
		"""Submit one raw clipboard fragment to the captured document session."""
		target = self._active_session
		if (
			target is None
			or target.is_disposed
			or target not in self._sessions
			or not target.can_commit_persistent_action
		):
			self.statusBar().showMessage(
				self.tr("Document cannot accept a persistent edit"), 3000,
			)
			self._refresh_document_actions()
			return
		status, fragment_cdml = self._clipboard_manager.read_fragment()
		if status == "no_data":
			self.statusBar().showMessage(
				self.tr("No CDML data on clipboard"), 3000,
			)
			return
		if status == "decode_error":
			self.statusBar().showMessage(
				self.tr("Could not decode clipboard CDML data"), 3000,
			)
			return
		if fragment_cdml is None:
			return
		if (
			target.is_disposed
			or target not in self._sessions
			or not target.can_commit_persistent_action
		):
			self.statusBar().showMessage(
				self.tr("Document cannot accept a persistent edit"), 3000,
			)
			self._refresh_document_actions()
			return
		outcome = target.submit_clipboard_fragment(fragment_cdml)
		self._show_persistent_action_outcome(outcome)
		self._refresh_document_actions()
	def _remove_top_level_objects(self, selected_objects: list) -> None:
		"""Remove complete selected molecules and artwork through undo commands."""
		scene = self._scene
		document = self._document
		document.undo_stack.beginMacro("Cut")
		for object_model in selected_objects:
			if hasattr(object_model, "atoms"):
				self._remove_molecule_with_marks(object_model)
				continue
			graphics_item = self._presentation_item(object_model)
			if graphics_item is not None:
				document.undo_stack.push(
					ferrum_qt.undo.commands.RemovePresentationObjectCommand(
						document, scene, object_model, graphics_item,
					)
				)
		document.undo_stack.endMacro()
	def _remove_molecule_with_marks(self, molecule_model: object) -> None:
		"""Queue atomic removal of one molecule and all atom-attached marks."""
		document = self._document
		scene = self._scene
		for mark_model in document.marks:
			if mark_model.atom_model not in molecule_model.atoms:
				continue
			mark_item = self._mark_item(mark_model)
			parent_atom_item = self._atom_item(mark_model.atom_model)
			if mark_item is None or parent_atom_item is None:
				continue
			document.undo_stack.push(
				ferrum_qt.undo.commands.RemoveAtomMarkCommand(
					document, mark_model, mark_item, parent_atom_item,
				)
			)
		graphics_items = [
			item for item in scene.items()
			if getattr(item, "molecule_model", None) is molecule_model
		]
		document.undo_stack.push(
			ferrum_qt.undo.commands.RemoveMoleculeCommand(
				document, scene, molecule_model, graphics_items,
			)
		)
	def _atom_item(self, atom_model: object) -> PySide6.QtWidgets.QGraphicsItem | None:
		"""Return the active atom projection for one atom model."""
		for item in self._scene.items():
			if getattr(item, "atom_model", None) is atom_model:
				return item
		return None
	def _mark_item(self, mark_model: object) -> PySide6.QtWidgets.QGraphicsItem | None:
		"""Return the active mark projection for one persistent mark model."""
		for item in self._scene.items():
			if getattr(item, "atom_mark_model", None) is mark_model:
				return item
		return None
	def _presentation_item(
			self, object_model: object,
			) -> PySide6.QtWidgets.QGraphicsItem | None:
		"""Return the active projection for one presentation model."""
		for item in self._scene.items():
			if getattr(item, "document_object_model", None) is object_model:
				return item
		return None
	def on_select_all(self) -> None:
		"""Select all interactive items in the scene."""
		import ferrum_qt.canvas.items.atom_item
		import ferrum_qt.canvas.items.bond_item
		for item in self._scene.items():
			if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
				item.setSelected(True)
			elif isinstance(item, ferrum_qt.canvas.items.bond_item.BondItem):
				item.setSelected(True)
			elif getattr(item, "document_object_model", None) in self.document.presentation_objects:
				item.setSelected(True)
	def _delete_selected(self) -> None:
		"""Delete all selected atoms and bonds with undo support."""
		import ferrum_qt.canvas.items.atom_item
		import ferrum_qt.canvas.items.bond_item
		import ferrum_qt.undo.commands
		scene = self._scene
		undo_stack = self._document.undo_stack
		# begin undo macro for compound delete
		undo_stack.beginMacro("Cut")
		# delete selected bonds first
		for bond_item in list(self._document.selected_bonds):
			bond_model = bond_item.bond_model
			mol = self._document._find_molecule_for_bond(bond_model)
			if mol is not None:
				cmd = ferrum_qt.undo.commands.RemoveBondCommand(
					scene, mol, bond_model, bond_item,
				)
				undo_stack.push(cmd)
		# delete selected atoms and their remaining connected bonds
		for atom_item in list(self._document.selected_atoms):
			atom_model = atom_item.atom_model
			mol = self._document._find_molecule_for_atom(atom_model)
			if mol is None:
				continue
			# find connected bond items still in scene
			connected = []
			for item in scene.items():
				if isinstance(
					item, ferrum_qt.canvas.items.bond_item.BondItem
				):
					bm = item.bond_model
					if bm.atom1 is atom_model or bm.atom2 is atom_model:
						connected.append((bm, item))
			cmd = ferrum_qt.undo.commands.RemoveAtomCommand(
				scene, mol, atom_model, atom_item, connected,
			)
			undo_stack.push(cmd)
		undo_stack.endMacro()

"""Edit-mode event coordinator."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.actions.context_menu
import ferrum_qt.canvas.items.atom_item
import ferrum_qt.canvas.items.bond_item
import ferrum_qt.modes.base_mode
import ferrum_qt.modes.edit_delete
import ferrum_qt.modes.edit_drag
import ferrum_qt.modes.edit_item_interaction
import ferrum_qt.modes.edit_selection
import ferrum_qt.undo.commands

# minimum drag distance in pixels before a move begins
_DRAG_THRESHOLD = 3.0
# nudge distance for arrow key movement
_NUDGE_DISTANCE = 2.0


class EditMode(
		ferrum_qt.modes.edit_drag.EditDragMixin,
		ferrum_qt.modes.edit_delete.EditDeleteMixin,
		ferrum_qt.modes.edit_selection.EditSelectionMixin,
		ferrum_qt.modes.base_mode.BaseMode,
		):
	"""Mode for selecting and manipulating existing items.

	Supports click-to-select, shift-click for multi-select, rubber band
	box selection, drag-to-move selected items, and keyboard shortcuts
	for deletion, nudging, and clipboard operations.

	Args:
		view: The ChemView widget that owns this mode.
		parent: Optional parent QObject.
	"""

	#============================================
	def __init__(
			self, view: object,
			parent: PySide6.QtCore.QObject | None = None,
			) -> None:
		"""Initialize the edit mode.

		Args:
			view: The ChemView widget that dispatches events.
			parent: Optional parent QObject.
		"""
		super().__init__(view, parent)
		self._name = "Edit"
		self._cursor = PySide6.QtCore.Qt.CursorShape.ArrowCursor
		# drag state
		self._dragging = False
		self._drag_start = None
		self._drag_last = None
		self._drag_anchor_item = None
		self._drag_anchor_start = None
		self._drag_atom_start_positions = {}
		self._drag_presentation_start_geometry = {}
		# rubber band selection rectangle
		self._rubber_band = None
		self._rubber_band_scene = None
		self._rubber_band_origin = None
		# items being dragged
		self._moved_items = []
		self._persistent_operation = None
		self._atom_translate_operation = None
		self._atom_translate_authority = None
		self._presentation_translate_operation = None
		self._presentation_translate_context = None
		self._selection_translate_operation = None
		self._selection_translate_context = None
		self._top_level_delete_context = None
		self._structure_delete_context = None
		self._atom_mark_delete_context = None
		self._drag_presentation_authority = "local"
		self._drag_presentation_operation = None
		self._drag_presentation_revision = None
		self._drag_presentation_state = "none"
		self._drag_selection_authority = "local"
		self._drag_selection_operation = None
		self._drag_selection_revision = None
		self._drag_selection_state = "none"

	#============================================
	def set_persistent_operation(self, operation: object | None) -> None:
		"""Install the session-owned plain persistent-operation callback."""
		self._persistent_operation = operation if callable(operation) else None

	#============================================
	def set_atom_translate_operation(self, operation: object | None) -> None:
		"""Install the session-owned direct-atom translation callback."""
		if operation is not None and not callable(operation):
			raise TypeError("Atom translation operation must be callable")
		self._atom_translate_operation = operation

	#============================================
	def set_atom_translate_authority(self, authority: object | None) -> None:
		"""Install the frontend-only drag-authority query for this session."""
		if authority is not None and not callable(authority):
			raise TypeError("Atom translation authority must be callable")
		self._atom_translate_authority = authority

	#============================================
	def set_presentation_translate_operation(self, operation: object | None) -> None:
		"""Install the session-owned top-level translation callback."""
		if operation is not None and not callable(operation):
			raise TypeError("Presentation translation operation must be callable")
		self._presentation_translate_operation = operation

	#============================================
	def set_presentation_translate_context(self, context: object | None) -> None:
		"""Install the plain authority/revision query for presentation drags."""
		if context is not None and not callable(context):
			raise TypeError("Presentation translation context must be callable")
		self._presentation_translate_context = context

	#============================================
	def set_selection_translate_operation(self, operation: object | None) -> None:
		"""Install the session-owned mixed-selection translation callback."""
		if operation is not None and not callable(operation):
			raise TypeError("Selection translation operation must be callable")
		self._selection_translate_operation = operation

	#============================================
	def set_selection_translate_context(self, context: object | None) -> None:
		"""Install the plain authority/revision query for mixed selection drags."""
		if context is not None and not callable(context):
			raise TypeError("Selection translation context must be callable")
		self._selection_translate_context = context

	#============================================
	def set_top_level_delete_context(self, context: object | None) -> None:
		"""Install the plain authority/revision query for complete-root Delete."""
		if context is not None and not callable(context):
			raise TypeError("Top-level Delete context must be callable")
		self._top_level_delete_context = context

	#============================================
	def set_structure_delete_context(self, context: object | None) -> None:
		"""Install the plain authority/revision query for partial structure Delete."""
		if context is not None and not callable(context):
			raise TypeError("Structure Delete context must be callable")
		self._structure_delete_context = context

	#============================================
	def set_atom_mark_delete_context(self, context: object | None) -> None:
		"""Install the plain authority/revision query for selected-mark Delete."""
		if context is not None and not callable(context):
			raise TypeError("Atom-mark Delete context must be callable")
		self._atom_mark_delete_context = context

	#============================================
	@property
	def status_hint(self) -> str:
		"""Return edit mode interaction hint for the status bar.

		Returns:
			A short description of available edit interactions.
		"""
		return "Click to select | Drag to move | Shift-click for multi-select"

	# ------------------------------------------------------------------
	# Lifecycle
	# ------------------------------------------------------------------

	#============================================
	def deactivate(self) -> None:
		"""Clean up any drag or rubber band state when leaving edit mode."""
		self._restore_drag_preview()
		self._reset_drag_state()
		super().deactivate()

	# ------------------------------------------------------------------
	# Mouse event handlers
	# ------------------------------------------------------------------

	#============================================
	def mouse_press3(
			self, scene_pos: PySide6.QtCore.QPointF, event: object,
			) -> None:
		"""Handle right-click to show context menu.

		Dispatched via view -> mode_manager -> mouse_press3 for
		Tk parity with mode.mouse_down3().

		Args:
			scene_pos: Position in scene coordinates.
			event: The mouse event.
		"""
		screen_pos = event.globalPosition().toPoint()
		ferrum_qt.actions.context_menu.show_context_menu(
			self._view, scene_pos, screen_pos,
		)

	#============================================
	def mouse_press(
			self, scene_pos: PySide6.QtCore.QPointF, event: object,
			) -> None:
		"""Handle mouse press for selection and drag initiation.

		Click on an item selects it (shift-click adds to selection).
		Click on empty space clears selection and starts rubber band.
		Click on an already-selected item starts a drag operation.

		Args:
			scene_pos: Position in scene coordinates.
			event: The mouse event.
		"""
		item = self._item_at(scene_pos)
		shift_held = bool(event.modifiers() & PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier)
		scene = self._env.scene
		if scene is None:
			return
		if item is not None:
			# clicking on an item
			if item.isSelected() and not shift_held:
				# start dragging the already-selected items
				self._dragging = True
				self._drag_start = scene_pos
				self._drag_last = scene_pos
				self._moved_items = scene.selectedItems()
				self._capture_drag_start_state(clicked_item=item)
			elif shift_held:
				# toggle selection on shift-click
				item.setSelected(not item.isSelected())
			else:
				# clear selection and select this item
				scene.clearSelection()
				item.setSelected(True)
				# prepare for potential drag
				self._dragging = True
				self._drag_start = scene_pos
				self._drag_last = scene_pos
				self._moved_items = [item]
				self._capture_drag_start_state(clicked_item=item)
		else:
			# click on empty space: clear selection and start rubber band
			if not shift_held:
				scene.clearSelection()
			self._rubber_band_origin = scene_pos
			self.status_message.emit("Drag to select area")

	#============================================
	def mouse_move(
			self, scene_pos: PySide6.QtCore.QPointF, event: object,
			) -> None:
		"""Handle mouse move for dragging items or updating rubber band.

		Drags selected items by the delta from the last move position.
		If rubber-banding, updates the selection rectangle.

		Args:
			scene_pos: Position in scene coordinates.
			event: The mouse event.
		"""
		if self._dragging and self._drag_last is not None:
			scene = self._env.scene
			if scene is None:
				return
			# compute delta from last tracked position
			dx = scene_pos.x() - self._drag_last.x()
			dy = scene_pos.y() - self._drag_last.y()
			# axis-lock: Ctrl constrains to vertical, Shift to horizontal
			modifiers = event.modifiers() if event is not None else PySide6.QtCore.Qt.KeyboardModifier.NoModifier
			if modifiers & PySide6.QtCore.Qt.KeyboardModifier.ControlModifier:
				dx = 0
			if modifiers & PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier:
				dy = 0
			# only start moving after exceeding threshold
			total_dx = dx
			total_dy = dy
			if self._drag_start is not None:
				total_dx = scene_pos.x() - self._drag_start.x()
				total_dy = scene_pos.y() - self._drag_start.y()
				if modifiers & PySide6.QtCore.Qt.KeyboardModifier.ControlModifier:
					total_dx = 0
				if modifiers & PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier:
					total_dy = 0
				distance = (total_dx ** 2 + total_dy ** 2) ** 0.5
				if distance < _DRAG_THRESHOLD:
					return
			# Tk parity: move by snapped anchor delta when grid snap is enabled.
			move_dx = dx
			move_dy = dy
			if (
				getattr(scene, "grid_snap_enabled", True)
				and self._drag_anchor_item is not None
				and self._drag_anchor_start is not None
				and hasattr(scene, "snap_to_grid")
			):
				target_x = self._drag_anchor_start[0] + total_dx
				target_y = self._drag_anchor_start[1] + total_dy
				snap_x, snap_y = scene.snap_to_grid(target_x, target_y)
				anchor_model = self._drag_anchor_item.atom_model
				move_dx = snap_x - anchor_model.x
				move_dy = snap_y - anchor_model.y
			# move each selected item
			for item in self._moved_items:
				if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
					model = item.atom_model
					model.x = model.x + move_dx
					model.y = model.y + move_dy
				elif self._is_presentation_item(item):
					self._translate_presentation_model(
						item.document_object_model, move_dx, move_dy,
					)
			self._drag_last = scene_pos
		elif self._rubber_band_origin is not None:
			# update or create the rubber band rectangle
			self._update_rubber_band(scene_pos)

	#============================================
	def mouse_release(
			self, scene_pos: PySide6.QtCore.QPointF, event: object,
			) -> None:
		"""Handle mouse release to finalize moves or rubber band selection.

		Synchronized atom-only, presentation-only, and mixed drags each restore
		their responsive preview before submitting one backend-owned request.
		Explicitly isolated sessions retain their established local undo behavior.
		If rubber-banding, selects items within the rectangle.

		Args:
			scene_pos: Position in scene coordinates.
			event: The mouse event.
		"""
		scene = self._env.scene
		if self._dragging and self._drag_start is not None and scene is not None:
			if self._drag_selection_state != "none":
				authority = self._drag_selection_authority
				if authority != "local":
					# This helper returns only immutable IDs and scalar geometry.  Its
					# graphics wrappers go out of scope before reset and the synchronous
					# session callback can retire and rebuild the projection.
					selection_drag = self._mixed_selection_drag_plan()
					operation = self._drag_selection_operation
					revision = self._drag_selection_revision
					self._restore_drag_preview()
					self._reset_drag_state()
					if authority == "backend" and selection_drag is not None:
						atom_targets, root_keys, delta = selection_drag
						self._submit_selection_drag(
							operation, revision, atom_targets, root_keys, delta,
						)
					elif authority == "backend":
						self.status_message.emit(
							"Move unavailable: mixed selection lacks durable current identity",
						)
					else:
						self.status_message.emit(
							"Move unavailable until the document projection recovers",
						)
					return
			# Local completion may retain wrappers in undo commands.  Build those
			# wrapper-bearing lists only after the synchronized mixed branch has
			# returned or explicitly selected local authority.
			items_and_offsets = self._atom_drag_offsets()
			presentation_changes = self._presentation_drag_changes()
			presentation_drag = self._presentation_only_drag_request(presentation_changes)
			if self._drag_presentation_state != "none" and presentation_changes:
				authority = self._drag_presentation_authority
				if authority != "local":
					operation = self._drag_presentation_operation
					revision = self._drag_presentation_revision
					self._restore_drag_preview()
					self._reset_drag_state()
					if authority == "backend" and presentation_drag is not None:
						root_keys, delta = presentation_drag
						self._submit_presentation_drag(operation, revision, root_keys, delta)
					elif authority == "backend":
						self.status_message.emit(
							"Move unavailable: selected presentation lacks durable current identity",
						)
					else:
						self.status_message.emit(
							"Move unavailable until the document projection recovers",
						)
					return
			atom_only = self._is_atom_only_drag(presentation_changes)
			atom_drag = self._atom_only_drag_request(items_and_offsets, presentation_changes)
			if atom_only and items_and_offsets:
				authority = self._drag_authority()
				if authority != "local":
					# The backend owns an atom-only completion.  Restore the complete
					# preview before a callback can replace its live Qt projection.
					self._restore_drag_preview()
					self._reset_drag_state()
					if authority == "backend" and atom_drag is not None:
						targets, delta = atom_drag
						self._submit_atom_drag(targets, delta)
					elif authority == "backend":
						self.status_message.emit("Move unavailable: selected atom lacks durable identity")
					else:
						self.status_message.emit("Move unavailable until the document projection recovers")
					return
			if items_and_offsets or presentation_changes:
				undo_stack = self._env.undo_stack
				if undo_stack is not None:
					# Keep a mixed molecule/artwork drag as one history action.
					undo_stack.beginMacro("Move Selected")
					if items_and_offsets:
						undo_stack.push(ferrum_qt.undo.commands.MoveAtomsCommand(
							items_and_offsets,
						))
					if presentation_changes:
						undo_stack.push(
							ferrum_qt.undo.commands.MovePresentationObjectsCommand(
								presentation_changes,
							),
						)
					undo_stack.endMacro()
		elif self._rubber_band_origin is not None and scene is not None:
			# select items within the rubber band rectangle
			self._finalize_rubber_band(scene_pos)
		self._reset_drag_state()

	#============================================
	def mouse_double_click(
			self, scene_pos: PySide6.QtCore.QPointF, event: object,
			) -> None:
		"""Route an eligible projected item to its detached public editor.

		Atom and bond actions keep their existing session-bound operation boundary.
		A Text action is re-resolved from the selected durable projection by the
		public action before opening a modal dialog, so no Qt wrapper crosses a
		potential backend reprojection.

		Args:
			scene_pos: Position in scene coordinates.
			event: The mouse event.
		"""
		item = self._item_at(scene_pos)
		if item is None:
			return
		ferrum_qt.modes.edit_item_interaction.open_item_editor(
			item, self._edit_atom_properties, self._edit_bond_properties,
			self._env.scene, self._env.window,
		)

	# ------------------------------------------------------------------
	# Keyboard event handlers
	# ------------------------------------------------------------------

	#============================================
	def key_press(self, event: object) -> None:
		"""Handle key presses for deletion, nudging, and clipboard ops.

		Supported keys:
		- Delete/Backspace: delete selected items
		- Arrow keys: nudge selected items
		- Ctrl+A: select all
		- Escape: clear selection

		Args:
			event: The QKeyEvent.
		"""
		key = event.key()
		modifiers = event.modifiers()
		ctrl = bool(modifiers & PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
		# delete selected items
		if key in (PySide6.QtCore.Qt.Key.Key_Delete, PySide6.QtCore.Qt.Key.Key_Backspace):
			self._delete_selected()
			return
		# select all
		if ctrl and key == PySide6.QtCore.Qt.Key.Key_A:
			self._select_all()
			return
		# escape clears selection
		if key == PySide6.QtCore.Qt.Key.Key_Escape:
			if self._dragging:
				self._restore_drag_preview()
				self._reset_drag_state()
				self.status_message.emit("Move cancelled")
				return
			scene = self._env.scene
			if scene is not None:
				scene.clearSelection()
			return
		# arrow key nudging
		nudge_map = {
			PySide6.QtCore.Qt.Key.Key_Left: (-_NUDGE_DISTANCE, 0),
			PySide6.QtCore.Qt.Key.Key_Right: (_NUDGE_DISTANCE, 0),
			PySide6.QtCore.Qt.Key.Key_Up: (0, -_NUDGE_DISTANCE),
			PySide6.QtCore.Qt.Key.Key_Down: (0, _NUDGE_DISTANCE),
		}
		if key in nudge_map:
			dx, dy = nudge_map[key]
			self._nudge_selected(dx, dy)
			return

	# ------------------------------------------------------------------
	# Action helpers
	# ------------------------------------------------------------------

	#============================================

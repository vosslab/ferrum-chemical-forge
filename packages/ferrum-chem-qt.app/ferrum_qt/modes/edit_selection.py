"""Selection, property editing, and preview helpers for :class:`EditMode`."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.actions.property_editing
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.canvas.items.atom_item
import ferrum_qt.canvas.items.bond_item
from ferrum_qt.canvas.items import render_ops_painter


#============================================
class EditSelectionMixin:
	"""Own transient selection and small edit-surface helpers."""

	def _select_all(self) -> None:
		"""Select all interactive items in the scene."""
		scene = self._env.scene
		if scene is None:
			return
		for item in scene.items():
			if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
				item.setSelected(True)
			elif isinstance(item, ferrum_qt.canvas.items.bond_item.BondItem):
				item.setSelected(True)
			elif self._is_presentation_item(item):
				item.setSelected(True)
		self.status_message.emit("Selected all")

	#============================================
	def _nudge_selected(self, dx: float, dy: float) -> None:
		"""Submit one selected durable-atom nudge without local geometry mutation.

		Args:
			dx: Horizontal offset in scene units.
			dy: Vertical offset in scene units.
		"""
		scene = self._env.scene
		if scene is None:
			return
		targets = []
		for item in scene.selectedItems():
			if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
				atom_model = item.atom_model
				atom_id = atom_model.backend_durable_id
				molecule = getattr(atom_model, "_molecule_model", None)
				molecule_id = getattr(molecule, "mol_id", None)
				if not atom_id or not molecule_id:
					self.status_message.emit(
						"Nudge unavailable: selected atom lacks durable identity",
					)
					return
				targets.append((str(molecule_id), str(atom_id)))
		if not targets:
			return
		if self._atom_translate_operation is None:
			self.status_message.emit("Nudge unavailable for this document")
			return
		outcome = self._atom_translate_operation(tuple(targets), (dx, dy))
		self.status_message.emit(outcome.message)

	# ------------------------------------------------------------------
	# Property editing helpers
	# ------------------------------------------------------------------

	#============================================
	def _edit_atom_properties(
			self, atom_item: ferrum_qt.canvas.items.atom_item.AtomItem,
			) -> None:
		"""Open the atom dialog and submit one revision-bound persistent patch.

		Dialog acceptance submits exactly the revision captured before the modal
		interaction. A changed backend acceptance installs canonical reprojection;
		a canonical no-op keeps projection and history unchanged. A stale submission
		rejects atomically. Intentionally isolated documents retain local undo.

		Args:
			atom_item: The AtomItem to edit.
		"""
		undo_stack = self._env.undo_stack
		if undo_stack is None:
			return
		model = atom_item.atom_model
		# Capture status text before an accepted backend commit replaces this projection.
		symbol = model.symbol
		changed = ferrum_qt.actions.property_editing.edit_atom_properties(
			model, self._view, undo_stack,
		)
		if changed:
			self.status_message.emit(f"Edited atom {symbol}")

	#============================================
	def _edit_bond_properties(
			self, bond_item: ferrum_qt.canvas.items.bond_item.BondItem,
			) -> None:
		"""Open a bond dialog for detached persistent intent.

		An ineligible durable bond ID or unavailable synchronized capability is inert
		before dialog or commit routing. After the dialog captures its target and
		capability, a revision that becomes stale reaches the backend as a typed
		atomic rejection and leaves the authoritative snapshot, projection, history,
		and dirty state unchanged. A changed accepted synchronized edit submits one
		exact-session backend patch and installs its canonical reprojection; backend
		history owns undo and dirty state. A canonical no-op creates no history and
		keeps the installed projection.
		Intentionally isolated documents retain local ChangePropertyCommand undo.

		Args:
			bond_item: The BondItem to edit.
		"""
		undo_stack = self._env.undo_stack
		if undo_stack is None:
			return
		model = bond_item.bond_model
		changed = ferrum_qt.actions.property_editing.edit_bond_properties(
			model, self._view, undo_stack,
		)
		if changed:
			self.status_message.emit("Edited bond properties")

	# ------------------------------------------------------------------
	# Rubber band helpers
	# ------------------------------------------------------------------

	#============================================
	def _update_rubber_band(self, scene_pos: PySide6.QtCore.QPointF) -> None:
		"""Update or create the rubber band selection rectangle.

		Args:
			scene_pos: Current mouse position in scene coordinates.
		"""
		scene = self._env.scene
		if scene is None or self._rubber_band_origin is None:
			return
		# compute the rectangle from origin to current position
		rect = PySide6.QtCore.QRectF(self._rubber_band_origin, scene_pos).normalized()
		if self._rubber_band is None:
			# create a semi-transparent rubber band rectangle
			pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor(render_ops_painter.get_canvas_color("selection")))
			pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
			brush = PySide6.QtGui.QBrush(PySide6.QtGui.QColor(51, 153, 255, 40))
			self._rubber_band = scene.addRect(rect, pen, brush)
			self._rubber_band_scene = scene
		else:
			self._rubber_band.setRect(rect)

	#============================================
	def _finalize_rubber_band(self, scene_pos: PySide6.QtCore.QPointF) -> None:
		"""Select all interactive items within the rubber band rectangle.

		Args:
			scene_pos: Final mouse position in scene coordinates.
		"""
		scene = self._env.scene
		if scene is None or self._rubber_band_origin is None:
			return
		rect = PySide6.QtCore.QRectF(self._rubber_band_origin, scene_pos).normalized()
		# select items within the rectangle
		items_in_rect = scene.items(rect)
		for item in items_in_rect:
			if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
				item.setSelected(True)
			elif isinstance(item, ferrum_qt.canvas.items.bond_item.BondItem):
				item.setSelected(True)
			elif self._is_presentation_item(item):
				item.setSelected(True)

	#============================================
	def _cancel_rubber_band(self) -> None:
		"""Terminally retire the known rubber band before releasing its wrapper."""
		rubber_band = self._rubber_band
		rubber_band_scene = self._rubber_band_scene
		if rubber_band is None:
			return
		try:
			coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
			if rubber_band_scene is None:
				coordinator.retire_detached_projection_items(
					[rubber_band], reaper=self._graphics_retirement_reaper,
				)
			else:
				coordinator.retire_scene_projection_items(
					rubber_band_scene, [rubber_band],
					reaper=self._graphics_retirement_reaper,
				)
			coordinator.raise_if_callback_failed("Edit selection preview retirement failed")
		finally:
			self._rubber_band = None
			self._rubber_band_scene = None

	#============================================
	def _item_at(self, scene_pos: PySide6.QtCore.QPointF) -> object | None:
		"""Return the topmost selectable chemistry or presentation item."""
		item = super()._item_at(scene_pos)
		if item is not None:
			return item
		scene = self._env.scene
		if scene is None:
			return None
		for candidate in scene.items(scene_pos):
			if self._is_presentation_item(candidate):
				return candidate
		return None

	#============================================
	def _is_presentation_item(self, item: object) -> bool:
		"""Whether an item projects a persistent document presentation model."""
		document = self._env.document
		object_model = getattr(item, "document_object_model", None)
		return document is not None and object_model in document.presentation_objects

	#============================================
	def _presentation_geometry(self, object_model: object) -> tuple:
		"""Copy one presentation model's editable points and bounds."""
		return (object_model.points, object_model.bounds)

	#============================================
	def _translate_presentation_model(
			self, object_model: object, dx: float, dy: float,
			) -> None:
		"""Translate points and bounds through the persistent presentation model."""
		points, bounds = self._presentation_geometry(object_model)
		object_model.set_points([
			(x + dx, y + dy, z) for x, y, z in points
		])
		if bounds is not None:
			x, y, width, height = bounds
			object_model.set_bounds((x + dx, y + dy, width, height))

	#============================================
	def _presentation_drag_changes(self) -> list:
		"""Return changed presentation-model geometry in drag-start order."""
		changes = []
		for object_model, before in self._drag_presentation_start_geometry.values():
			after = self._presentation_geometry(object_model)
			if after != before:
				changes.append((object_model, before, after))
		return changes

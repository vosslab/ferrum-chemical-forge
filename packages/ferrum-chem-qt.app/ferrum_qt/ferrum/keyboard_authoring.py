"""Keyboard document-cursor authoring for revision-bound Ferrum tools."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.ferrum.keyboard_canvas
import ferrum_qt.ferrum.line_tool_intent


#============================================
class FerrumKeyboardAuthoringMixin:
	"""Adapt keyboard events to existing atom and bond tool intents.

	The host owns the Rust-backed tab and pointer intent lifetime.  This mixin
	contains only event adaptation and never retains a document model.
	"""

	#============================================
	def _keyboard_canvas_key_event(self, event: PySide6.QtGui.QKeyEvent) -> bool:
		"""Handle cursor authoring without retaining any Python document state."""
		key = event.key()
		if key == PySide6.QtCore.Qt.Key.Key_Escape:
			self._cancel_atom_insertion()
			self._cancel_line_gesture()
			self._refresh_actions()
			tab = self._active_native_tab()
			if tab is not None:
				tab.view.hide_keyboard_cursor()
				tab.view.viewport().setFocus()
			self._synchronize_mode_state()
			self.statusBar().showMessage(
				self.tr("Tool cancelled. Selection and document are unchanged."), 3000,
			)
			return True
		if key in (
				PySide6.QtCore.Qt.Key.Key_Left,
				PySide6.QtCore.Qt.Key.Key_Right,
				PySide6.QtCore.Qt.Key.Key_Up,
				PySide6.QtCore.Qt.Key.Key_Down,
			):
			tab = self._active_native_tab()
			if tab is None or tab.requires_refresh:
				return True
			fine = bool(event.modifiers() & PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier)
			increment = ferrum_qt.ferrum.keyboard_canvas.keyboard_cursor_increment(fine)
			dx = increment if key == PySide6.QtCore.Qt.Key.Key_Right else (
				-increment if key == PySide6.QtCore.Qt.Key.Key_Left else 0.0
			)
			dy = increment if key == PySide6.QtCore.Qt.Key.Key_Down else (
				-increment if key == PySide6.QtCore.Qt.Key.Key_Up else 0.0
			)
			if self._nudge_render_interaction_selection(dx, dy):
				tab.view.viewport().setFocus()
				return True
			point = tab.view.move_keyboard_cursor(float(dx), float(dy))
			precision = "fine " if fine else ""
			self.statusBar().showMessage(self.tr(
				"{0}document cursor: {1:.1f}, {2:.1f}. Press Enter to commit or Esc to cancel."
			).format(precision, point.x(), point.y()), 5000)
			tab.view.viewport().setFocus()
			return True
		if key in (PySide6.QtCore.Qt.Key.Key_Return, PySide6.QtCore.Qt.Key.Key_Enter):
			if self._atom_insertion_intent is not None:
				self._complete_atom_insertion_at_keyboard_cursor()
				return True
			intent = self._line_gesture_intent
			if intent is not None and intent.tool is ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_BOND:
				self._complete_keyboard_bond_endpoint()
				return True
		return False

	#============================================
	def _complete_atom_insertion_at_keyboard_cursor(self) -> None:
		"""Commit the captured atom intent at the visible document cursor."""
		intent = self._atom_insertion_intent
		if intent is None:
			return
		tab = intent.tab
		snapshot = tab.current_snapshot
		if (
				self._active_native_tab() is not tab
				or tab.requires_refresh
				or snapshot.revision != intent.revision
				or snapshot.digest != intent.digest
				):
			self._cancel_atom_insertion(clear_status=False)
			self._synchronize_mode_state()
			tab.view.viewport().setFocus()
			self._show_edit_refusal(self._typed_refusal(
				"use_tool", "stale_tool",
				"The document changed before placement; start Add Atom again.",
			))
			return
		try:
			point = tab.view.show_keyboard_cursor()
			tab.add_atom_at(intent.molecule_object_id, intent.element, float(point.x()), float(point.y()))
		except Exception as exc:
			if isinstance(
				exc,
				ferrum_qt.ferrum.document_tab.
				FerrumNativeDocumentTabUnrenderableMoleculeError,
			):
				self._cancel_atom_insertion(clear_status=False)
			self._refresh_actions()
			self._synchronize_mode_state()
			tab.view.viewport().setFocus()
			self._show_atom_insertion_refusal(exc)
			return
		self._cancel_atom_insertion(clear_status=False)
		self._synchronize_mode_state()
		tab.view.viewport().setFocus()
		self.statusBar().showMessage(self.tr("Added one atom at the document cursor. Arrow keys move the cursor; Esc cancels."), 5000)
		self._refresh_actions()

	#============================================
	def _complete_keyboard_bond_endpoint(self) -> None:
		"""Select then join Rust-owned existing or proposed cursor endpoints."""
		intent = self._line_gesture_intent
		if intent is None or intent.tool is not ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_BOND:
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture(clear_status=False)
			self._synchronize_mode_state()
			intent.tab.view.viewport().setFocus()
			self._show_edit_refusal(self._typed_refusal(
				"use_tool", "stale_tool",
				"The document changed before placement; start Draw Bond again.",
			))
			return
		try:
			point = intent.tab.view.show_keyboard_cursor()
			probe = intent.tab.direct_bond_pointer_probe_at_keyboard_scene_position(
				point,
			)
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError as exc:
			self._cancel_line_gesture(clear_status=False)
			self._refresh_actions()
			self._synchronize_mode_state()
			intent.tab.view.viewport().setFocus()
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return
		if intent.direct_bond_gesture is None:
			try:
				# Freeze the shared next-drawing choice with this V3 keyboard probe.
				drawing = self._drawing_parameters.snapshot()
				gesture = intent.tab.begin_direct_bond_gesture(
					probe, drawing.bond_presentation(),
					intent.tab.view.hex_grid_snap_enabled,
				)
			except Exception as exc:
				self._cancel_line_gesture(clear_status=False)
				self._refresh_actions()
				self._synchronize_mode_state()
				intent.tab.view.viewport().setFocus()
				if not self._is_direct_bond_begin_refusal(exc):
					raise
				self._show_direct_bond_refusal(exc)
				return
			self._line_gesture_intent = dataclasses.replace(
				intent,
				start_scene=point,
				drawing=drawing,
				direct_bond_gesture=gesture,
			)
			intent.tab.view.viewport().setFocus()
			self.statusBar().showMessage(self.tr("Bond start selected. Move to a different endpoint and press Enter; Esc cancels."), 5000)
			return
		gesture = intent.direct_bond_gesture
		if gesture is None:
			self._cancel_line_gesture(clear_status=False)
			self._synchronize_mode_state()
			intent.tab.view.viewport().setFocus()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Draw Bond was cancelled because its native gesture was unavailable. Start Draw Bond again.",
			))
			return
		try:
			# Reuse the pointer path so both input modes display the same copied
			# generic precommit overlay before redeeming the same receipt type.
			self._update_direct_bond_gesture(intent, intent.tab.view.mapFromScene(point))
			current = self._line_gesture_intent
			if current is None or current.prepared_transition is None:
				return
			self._commit_direct_bond_transition(current.tab, current.prepared_transition)
		except Exception:
			self._cancel_line_gesture(clear_status=False)
			self._refresh_actions()
			self._synchronize_mode_state()
			intent.tab.view.viewport().setFocus()
			raise
		self._reset_line_gesture_start()
		self._finish_line_gesture(intent, self.tr("Added one bond. Choose another start atom with Enter, or press Esc."))
		self._synchronize_mode_state()
		intent.tab.view.viewport().setFocus()

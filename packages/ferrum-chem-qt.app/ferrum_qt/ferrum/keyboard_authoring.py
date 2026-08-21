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
		"""Select then join two durable Rust atoms at the document cursor."""
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
			atom_id = intent.tab.durable_atom_at_scene_position(point)
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError as exc:
			# Exact keyboard coordinates can identify more than one durable atom.
			# A keyboard gesture cannot guess between them, so retire it rather than
			# leaving a stale authoring intent active after this typed refusal.
			self._cancel_line_gesture(clear_status=False)
			self._refresh_actions()
			self._synchronize_mode_state()
			intent.tab.view.viewport().setFocus()
			self.statusBar().showMessage(self.tr(
				"Draw Bond cancelled: more than one atom is at the document cursor. "
				"Choose a distinct atom location, then start Draw Bond again."
			), 5000)
			self._show_edit_refusal(self._typed_refusal(
				"edit_document", "unavailable_operation",
				"Draw Bond was not used because the keyboard cursor does not identify "
				"one durable atom: " + str(exc) + ". Choose a distinct atom location, "
				"then start Draw Bond again.",
			))
			return
		if atom_id is None:
			intent.tab.view.viewport().setFocus()
			self.statusBar().showMessage(self.tr("Move the document cursor onto an existing atom, then press Enter."), 5000)
			return
		if intent.start_atom_id is None:
			drawing = self._drawing_parameters.snapshot()
			self._line_gesture_intent = dataclasses.replace(intent, start_atom_id=atom_id, start_scene=intent.tab.durable_atom_scene_position(atom_id), drawing=drawing)
			intent.tab.view.viewport().setFocus()
			self.statusBar().showMessage(self.tr("Bond start selected. Move to a different atom and press Enter; Esc cancels."), 5000)
			return
		if atom_id == intent.start_atom_id:
			intent.tab.view.viewport().setFocus()
			self.statusBar().showMessage(self.tr("Choose a different bond endpoint, or press Esc to cancel Draw Bond."), 5000)
			return
		drawing = intent.drawing or self._drawing_parameters.snapshot()
		try:
			intent.tab.add_bond_between_atoms(intent.start_atom_id, atom_id, drawing.bond_presentation())
		except Exception as exc:
			self._refresh_actions()
			self._synchronize_mode_state()
			intent.tab.view.viewport().setFocus()
			self._show_edit_refusal(self._typed_refusal(
				"edit_document", "unavailable_operation", str(exc),
			))
			return
		self._reset_line_gesture_start()
		self._finish_line_gesture(intent, self.tr("Added one bond. Choose another start atom with Enter, or press Esc."))
		self._synchronize_mode_state()
		intent.tab.view.viewport().setFocus()

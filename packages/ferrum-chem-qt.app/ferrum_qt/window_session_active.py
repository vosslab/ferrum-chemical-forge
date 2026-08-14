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
class WindowSessionActiveMixin:
	"""One cohesive MainWindow session responsibility."""

	def _connect_active_session_signals(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			) -> None:
		"""Connect view, mode, and document signals for the active tab only."""
		session.view.mouse_moved.connect(self._status_bar.update_coords)
		session.view.zoom_changed.connect(
			self._zoom_controls.update_zoom_display
		)
		session.mode_manager.mode_changed.connect(
			self._mode_toolbar.set_active_mode
		)
		session.mode_manager.mode_changed.connect(self._status_bar.update_mode)
		session.mode_manager.mode_changed.connect(self._on_mode_changed)
		for mode in session.mode_manager._modes.values():
			mode.status_message.connect(self._show_mode_message)
		if session.document is not None:
			self._connect_document_signals(session.document)
	def _disconnect_active_session_signals(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			) -> None:
		"""Disconnect every active-only callback before a switch or close."""
		connections = (
			(session.view.mouse_moved, self._status_bar.update_coords),
			(
				session.view.zoom_changed,
				self._zoom_controls.update_zoom_display,
			),
			(
				session.mode_manager.mode_changed,
				self._mode_toolbar.set_active_mode,
			),
			(session.mode_manager.mode_changed, self._status_bar.update_mode),
			(session.mode_manager.mode_changed, self._on_mode_changed),
		)
		for signal, slot in connections:
			try:
				signal.disconnect(slot)
			except (RuntimeError, SystemError, TypeError):
				pass
		for mode in session.mode_manager._modes.values():
			try:
				mode.status_message.disconnect(self._show_mode_message)
			except (RuntimeError, SystemError, TypeError):
				pass
		if session.document is not None:
			self._disconnect_document_signals(session.document)
	def _activate_session(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> None:
		"""Switch all active-document consumers to one existing session."""
		if session.is_disposed or session not in self._sessions:
			return
		previous = self._active_session
		if not session.has_live_projection:
			if previous is session:
				self._bind_property_dock(None)
				current_mode = session.mode_manager.current_mode
				if current_mode is not None:
					current_mode.deactivate()
				self._synchronize_active_session_ui()
				return
			if previous is not None:
				if self._ui_signals_connected:
					self._disconnect_active_session_signals(previous)
				previous_mode = previous.mode_manager.current_mode
				if previous_mode is not None:
					previous_mode.deactivate()
			self._set_active_session_aliases(session)
			self._bind_property_dock(None)
			if self._ui_signals_connected:
				self._connect_active_session_signals(session)
			current_mode = session.mode_manager.current_mode
			if current_mode is not None:
				current_mode.deactivate()
			self._synchronize_active_session_ui()
			return
		if previous is session:
			if hasattr(self, "_property_dock"):
				self._synchronize_active_session_ui()
			return
		try:
			if previous is not None:
				if self._ui_signals_connected:
					self._disconnect_active_session_signals(previous)
				previous_mode = previous.mode_manager.current_mode
				if previous_mode is not None:
					previous_mode.deactivate()
			self._set_active_session_aliases(session)
			self._bind_property_dock(session)
			if self._ui_signals_connected:
				self._connect_active_session_signals(session)
				active_mode = session.mode_manager.current_mode
				if active_mode is not None:
					active_mode.activate()
				self._synchronize_active_session_ui()
		except Exception:
			if previous is not None and previous in self._sessions:
				previous_index = self._tab_widget.indexOf(previous.view)
				self._restore_active_session(previous, previous_index)
			else:
				self._clear_active_session_aliases()
			raise
	def _synchronize_active_session_ui(self, bind_property_dock: bool = True) -> None:
		"""Refresh controls after creating or activating a document session."""
		self._refresh_active_session_controls()
		if self._active_session is None:
			return
		if bind_property_dock:
			self._bind_property_dock(self._active_session)
		else:
			self._bind_property_dock(None)
	def _refresh_active_session_controls(self) -> None:
		"""Refresh active-session controls without changing dock projection ownership."""
		if self._active_session is None:
			return
		mode_name = self._active_mode_name()
		if mode_name is not None:
			self._mode_toolbar.set_active_mode(mode_name)
			self._status_bar.update_mode(mode_name)
			self._on_mode_changed(mode_name)
		self._zoom_controls.update_zoom_display(
			float(self._view.zoom_percent)
		)
		if hasattr(self, "_action_toggle_grid"):
			self._action_toggle_grid.setChecked(self._scene.grid_visible)
		if hasattr(self, "_action_toggle_grid_snap"):
			self._action_toggle_grid_snap.setChecked(
				self._scene.grid_snap_enabled
			)
		self._refresh_document_actions()
	def _active_mode_name(self) -> str | None:
		"""Return the registered name of the active session's current mode."""
		current_mode = self._mode_manager.current_mode
		for name in self._mode_manager.mode_names():
			if self._mode_manager._modes[name] is current_mode:
				return name
		return None
	@PySide6.QtCore.Slot(str)
	def _on_mode_selected(self, mode_name: str) -> None:
		"""Apply a toolbar mode selection to the active session."""
		self._mode_manager.set_mode(mode_name)
	@PySide6.QtCore.Slot(int)
	def _on_zoom_slider_changed(self, percent: int) -> None:
		"""Apply a zoom-slider change to the active session's view."""
		self._view.set_zoom_percent(float(percent))
	@PySide6.QtCore.Slot(int)
	def _on_tab_changed(self, index: int) -> None:
		"""Activate the session owning the selected tab page."""
		if self._tab_change_blocked or index < 0:
			return
		session = self._sessions_by_view.get(self._tab_widget.widget(index))
		if session is not None:
			self._activate_session(session)
	@PySide6.QtCore.Slot(int)
	def _on_tab_close_requested(self, index: int) -> None:
		"""Close the requested tab through its save guard."""
		self.close_session_at(index)
	def _update_session_tab_title(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			title: str,
			) -> None:
		"""Update one registered session's tab through its owned title relay."""
		if session not in self._sessions or session.is_disposed:
			return
		index = self._tab_widget.indexOf(session.view)
		if index >= 0:
			self._tab_widget.setTabText(index, title)
	def _connect_document_signals(
			self, document: ferrum_qt.models.document.Document,
			) -> None:
		"""Bind window callbacks to one active document exactly once."""
		if self._document_signal_source is document:
			return
		if self._document_signal_source is not None:
			self._disconnect_document_signals(self._document_signal_source)
		document.selection_changed.connect(self._property_dock_summary_refresh)
		document.object_added.connect(self._property_dock_summary_refresh)
		document.object_removed.connect(self._property_dock_summary_refresh)
		document.selection_changed.connect(self._update_menu_predicates)
		document.undo_stack.canUndoChanged.connect(
			self._on_document_undo_state_changed
		)
		document.undo_stack.canRedoChanged.connect(
			self._on_document_undo_state_changed
		)
		self._document_signal_source = document
	def _disconnect_document_signals(
			self, document: ferrum_qt.models.document.Document,
			) -> None:
		"""Release callbacks only when this window owns their binding."""
		if self._document_signal_source is not document:
			return
		connections = (
			(document.selection_changed, self._property_dock_summary_refresh),
			(document.object_added, self._property_dock_summary_refresh),
			(document.object_removed, self._property_dock_summary_refresh),
			(document.selection_changed, self._update_menu_predicates),
			(
				document.undo_stack.canUndoChanged,
				self._on_document_undo_state_changed,
			),
			(
				document.undo_stack.canRedoChanged,
				self._on_document_undo_state_changed,
			),
		)
		try:
			for signal, slot in connections:
				signal.disconnect(slot)
		finally:
			self._document_signal_source = None
	def _on_document_undo_state_changed(self, _available: bool) -> None:
		"""Refresh actions after either undo availability signal changes."""
		self._refresh_document_actions()
	def _on_document_modified_changed(self, _dirty: bool) -> None:
		"""Refresh the active tab title after dirty-state transitions."""
		self._update_document_title()
	def _refresh_document_actions(self) -> None:
		"""Synchronize active-document undo/redo and menu state."""
		session = self._active_session
		if self._document is None or session is None:
			self._undo_action.setEnabled(False)
			self._redo_action.setEnabled(False)
			self._update_menu_predicates()
			return
		self._undo_action.setEnabled(self.can_undo())
		self._redo_action.setEnabled(self.can_redo())
		self._update_menu_predicates()
	def can_paste(self) -> bool:
		"""Return whether the active session can accept the current clipboard."""
		session = self._active_session
		return bool(
			session is not None
			and session in self._sessions
			and not session.is_disposed
			and session.can_commit_persistent_action
			and self._clipboard_manager.can_paste()
		)
	def can_undo(self) -> bool:
		"""Return the single active undo capability for menus and shortcuts."""
		return self._legacy_undo_capability("undo")
	def can_redo(self) -> bool:
		"""Return the single active redo capability for menus and shortcuts."""
		return self._legacy_undo_capability("redo")
	def _legacy_undo_capability(self, direction: str) -> bool:
		"""Resolve backend history first, with legacy history only when isolated."""
		session = self._active_session
		if session is None or self._document is None:
			return False
		if session.legacy_isolated:
			stack = self._document.undo_stack
			if stack is None:
				return False
			try:
				available = stack.canUndo() if direction == "undo" else stack.canRedo()
			except RuntimeError:
				return False
			return available
		if not session.has_backend_navigation:
			return False
		available = (
			session.can_undo_backend if direction == "undo"
			else session.can_redo_backend
		)
		return available
	def _update_document_title(self) -> None:
		"""Show the active document name and unsaved marker."""
		if self._active_session is None:
			return
		index = self._tab_widget.indexOf(self._active_session.view)
		if index >= 0:
			self._tab_widget.setTabText(index, self._active_session.title)
	def on_new(self) -> bool:
		"""Public wrapper for toolbar New button."""
		return self._on_new()
	def on_open(self) -> bool:
		"""Public wrapper for toolbar Open button."""
		return self._on_open()
	def on_save(self) -> bool:
		"""Public wrapper for toolbar Save button."""
		return self._on_save()
	def on_undo(self) -> None:
		"""Public wrapper for toolbar Undo button."""
		if self._active_session is None or self._document is None:
			return
		if self._active_session.legacy_isolated:
			self._document.undo_stack.undo()
			return
		if self._active_session.has_backend_navigation:
			if self._active_session.can_undo_backend:
				self._show_persistent_action_outcome(self._active_session.undo_backend())
			else:
				self.statusBar().showMessage("Backend undo is unavailable", 3000)
			self._refresh_document_actions()
			return
		return
	def on_redo(self) -> None:
		"""Public wrapper for toolbar Redo button."""
		if self._active_session is None or self._document is None:
			return
		if self._active_session.legacy_isolated:
			self._document.undo_stack.redo()
			return
		if self._active_session.has_backend_navigation:
			if self._active_session.can_redo_backend:
				self._show_persistent_action_outcome(self._active_session.redo_backend())
			else:
				self.statusBar().showMessage("Backend redo is unavailable", 3000)
			self._refresh_document_actions()
			return
		return
	def _show_persistent_action_outcome(
			self,
			outcome: ferrum_qt.models.document_session.PersistentActionOutcome,
			) -> None:
		"""Display one concise frontend persistent-action result."""
		self.statusBar().showMessage(outcome.message, 3000)
	def discard_legacy_and_retry_projection(
			self,
			session: ferrum_qt.models.document_session.DocumentSession | None = None,
			) -> ferrum_qt.models.document_session.PersistentActionOutcome | None:
		"""Confirm discarding local edits before exact backend-only reprojection."""
		target = self._active_session if session is None else session
		if (
			target is None
			or target.is_disposed
			or target not in self._sessions
			or not target.has_live_projection
			or not target.legacy_isolated
		):
			return None
		answer = PySide6.QtWidgets.QMessageBox.question(
			self,
			self.tr("Discard Qt-local edits?"),
			self.tr(
				"Discard local Qt edits and rebuild this tab from the current "
				"authoritative backend document?",
			),
			PySide6.QtWidgets.QMessageBox.StandardButton.Yes
			| PySide6.QtWidgets.QMessageBox.StandardButton.Cancel,
			PySide6.QtWidgets.QMessageBox.StandardButton.Cancel,
		)
		if answer != PySide6.QtWidgets.QMessageBox.StandardButton.Yes:
			return None
		outcome = target._discard_legacy_and_retry_projection()
		self._show_persistent_action_outcome(outcome)
		self._refresh_document_actions()
		return outcome
	@PySide6.QtCore.Slot(str)
	def _show_mode_message(self, message: str) -> None:
		"""Deliver a mode result to the live window status bar."""
		self.statusBar().showMessage(message, 3000)

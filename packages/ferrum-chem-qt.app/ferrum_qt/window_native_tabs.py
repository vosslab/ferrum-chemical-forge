"""Native Ferrum tab host lifecycle kept separate from legacy sessions."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab


_NATIVE_ALLOWED_ACTION_IDS = frozenset((
	"file.new",
	"file.load",
	"file.save",
	"file.save_as",
	"file.open_native_cdml",
	"edit.undo_native",
	"edit.redo_native",
	"edit.change_element_native",
	"edit.atom_properties_native",
	"edit.atom_number_native",
	"edit.clear_atom_number_native",
	"edit.delete_atom_native",
	"edit.bond_properties_native",
	"edit.delete_bond_native",
	"file.exit",
	"options.theme",
	"help.about",
))


#============================================
class WindowNativeTabsMixin:
	"""Own native tab registration, activation, and terminal disposal.

	Native pages are deliberately not sessions.  The legacy session collections
	remain an authority boundary for OASA-backed tabs until their replacements
	exist as independent Ferrum paths.
	"""

	#============================================
	def _register_native_tab(
			self,
			tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			*, index: int | None = None, activate: bool = True,
			) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
		"""Atomically add one exact Rust-owned page to the common tab widget."""
		if type(tab) is not ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
			raise TypeError("native tab host requires an exact FerrumNativeDocumentTab")
		if tab in self._native_tabs_by_page:
			raise ValueError("Native tab is already registered")
		if self._tab_widget.indexOf(tab) >= 0:
			raise ValueError("Native tab page is already attached")
		if index is None:
			index = self._tab_widget.count()
		if index < 0 or index > self._tab_widget.count():
			raise IndexError("Native tab insertion index is out of range")

		previous_page = self._tab_widget.currentWidget()
		previous_block = self._tab_widget.blockSignals(True)
		previous_tab_change_blocked = self._tab_change_blocked
		self._tab_change_blocked = True
		selection_change_connected = False
		try:
			self._tab_widget.insertTab(index, tab, tab.title)
			self._native_tabs_by_page[tab] = tab
			tab.selection_changed.connect(self._on_native_tab_selection_changed)
			selection_change_connected = True
			if activate:
				self._tab_widget.setCurrentIndex(index)
				self._activate_native_tab(tab)
		except Exception:
			if selection_change_connected:
				try:
					tab.selection_changed.disconnect(self._on_native_tab_selection_changed)
				except (RuntimeError, TypeError):
					pass
			self._native_tabs_by_page.pop(tab, None)
			current_index = self._tab_widget.indexOf(tab)
			if current_index >= 0:
				self._tab_widget.removeTab(current_index)
				tab.hide()
				tab.setParent(None)
			self._restore_page_after_native_registration_failure(previous_page)
			raise
		finally:
			self._tab_change_blocked = previous_tab_change_blocked
			self._tab_widget.blockSignals(previous_block)
		return tab

	#============================================
	def _restore_page_after_native_registration_failure(
			self, previous_page: PySide6.QtWidgets.QWidget | None,
			) -> None:
		"""Restore the exact pre-registration page and its exclusive consumers."""
		if previous_page is None:
			self._restore_native_action_policy()
			self._clear_active_session_aliases()
			return
		previous_index = self._tab_widget.indexOf(previous_page)
		if previous_index < 0:
			raise RuntimeError("Native registration lost its prior tab page")
		self._tab_widget.setCurrentIndex(previous_index)
		previous_native = self._native_tabs_by_page.get(previous_page)
		if previous_native is not None:
			self._activate_native_tab(previous_native)
			return
		self._restore_native_action_policy()
		previous_session = self._sessions_by_view.get(previous_page)
		if previous_session is None:
			raise RuntimeError("Native registration prior page has no tab owner")
		self._activate_session(previous_session)

	#============================================
	def _activate_native_tab(
			self,
			tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Detach legacy consumers before presenting one native-only page."""
		if self._native_tabs_by_page.get(tab) is not tab:
			return
		previous = self._active_session
		if previous is not None:
			if self._ui_signals_connected:
				self._disconnect_active_session_signals(previous)
			current_mode = previous.mode_manager.current_mode
			if current_mode is not None:
				current_mode.deactivate()
		self._bind_property_dock(None)
		self._clear_active_session_aliases()
		self._apply_native_action_policy()
		self._refresh_native_tab_actions(tab)

	#============================================
	@PySide6.QtCore.Slot()
	def _on_native_tab_selection_changed(self) -> None:
		"""Refresh native-only actions after durable selection changes."""
		tab = self._active_native_tab()
		if tab is not None:
			self._refresh_native_tab_actions(tab)

	#============================================
	def _refresh_native_tab_actions(
			self,
			tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Allow the ordinary host to refine its explicitly native actions."""
		refresh = getattr(self, "_refresh_explicit_native_actions", None)
		if refresh is not None:
			refresh(tab)

	#============================================
	def _restore_legacy_after_native_tab(self) -> None:
		"""Restore ordinary action ownership before legacy activation resumes."""
		self._restore_native_action_policy()

	#============================================
	def _apply_native_action_policy(self) -> None:
		"""Disable actions that have no Rust-native operation route yet."""
		if getattr(self, "_neutral_native_shell", False) and not self._sessions:
			self._refresh_neutral_action_policy()
			return
		if self._native_action_enabled_state is not None:
			return
		actions = getattr(self._adapter, "_actions", {})
		self._native_action_enabled_state = {
			action_id: action.isEnabled() for action_id, action in actions.items()
		}
		for action_id, action in actions.items():
			action.setEnabled(action_id in _NATIVE_ALLOWED_ACTION_IDS)
		self._native_widget_enabled_state = {}
		for name in (
			"_mode_toolbar", "_submode_toolbar", "_edit_ribbon_toolbar",
			"_property_dock", "_zoom_controls",
		):
			widget = getattr(self, name, None)
			if widget is not None:
				self._native_widget_enabled_state[name] = widget.isEnabled()
				widget.setEnabled(False)

	#============================================
	def _restore_native_action_policy(self) -> None:
		"""Return legacy actions and controls to their pre-native ownership."""
		if getattr(self, "_neutral_native_shell", False) and not self._sessions:
			self._refresh_neutral_action_policy()
			return
		enabled_state = self._native_action_enabled_state
		if enabled_state is None:
			return
		actions = getattr(self._adapter, "_actions", {})
		for action_id, enabled in enabled_state.items():
			action = actions.get(action_id)
			if action is not None:
				action.setEnabled(enabled)
		self._native_action_enabled_state = None
		for name, enabled in self._native_widget_enabled_state.items():
			widget = getattr(self, name, None)
			if widget is not None:
				widget.setEnabled(enabled)
		self._native_widget_enabled_state = {}

	#============================================
	def _on_tab_changed(self, index: int) -> None:
		"""Dispatch the selected page to native or legacy activation ownership."""
		if self._tab_change_blocked or index < 0:
			return
		page = self._tab_widget.widget(index)
		tab = self._native_tabs_by_page.get(page)
		if tab is not None:
			self._activate_native_tab(tab)
			return
		self._restore_legacy_after_native_tab()
		super()._on_tab_changed(index)

	#============================================
	def _on_tab_close_requested(self, index: int) -> None:
		"""Close native pages through their native dirty guard and disposal path."""
		if index < 0:
			return
		page = self._tab_widget.widget(index)
		if self._native_tabs_by_page.get(page) is not None:
			self._close_native_tab_at(index)
			return
		super()._on_tab_close_requested(index)

	#============================================
	def close_session_at(self, index: int) -> bool:
		"""Close a native page or forward one legacy index to its owner."""
		if index >= 0 and self._native_tabs_by_page.get(self._tab_widget.widget(index)):
			return self._close_native_tab_at(index)
		page = self._tab_widget.widget(index)
		session = self._sessions_by_view.get(page)
		if session is None:
			return False
		return super().close_session_at(self._sessions.index(session))

	#============================================
	def _close_native_tab_at(self, index: int) -> bool:
		"""Guard, detach, and terminally dispose one exact native page."""
		page = self._tab_widget.widget(index)
		tab = self._native_tabs_by_page.get(page)
		if tab is None:
			return False
		if not self._confirm_native_tab_close("closing this tab", tab):
			self._tab_widget.setCurrentIndex(index)
			return False

		was_current = self._tab_widget.currentWidget() is tab
		previous_block = self._tab_widget.blockSignals(True)
		previous_tab_change_blocked = self._tab_change_blocked
		self._tab_change_blocked = True
		try:
			self._tab_widget.removeTab(index)
			self._native_tabs_by_page.pop(tab, None)
			try:
				tab.selection_changed.disconnect(self._on_native_tab_selection_changed)
			except (RuntimeError, TypeError):
				pass
			tab.hide()
			tab.setParent(None)
			if was_current and self._tab_widget.count() > 0:
				self._tab_widget.setCurrentIndex(min(index, self._tab_widget.count() - 1))
		finally:
			self._tab_change_blocked = previous_tab_change_blocked
			self._tab_widget.blockSignals(previous_block)
		tab.dispose()
		if was_current:
			self._activate_selected_page()
		return True

	#============================================
	def _activate_selected_page(self) -> None:
		"""Activate whichever distinct page Qt selected after a native removal."""
		index = self._tab_widget.currentIndex()
		if index >= 0:
			self._on_tab_changed(index)
		else:
			self._clear_active_session_aliases()
			if getattr(self, "_neutral_native_shell", False):
				self._refresh_neutral_action_policy()

	#============================================
	def _confirm_native_tab_close(
			self, operation: str,
			tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			) -> bool:
		"""Ask the injected native guard before discarding a dirty Rust tab."""
		if not tab.is_dirty:
			return True
		guard = self._native_tab_close_guard
		if guard is not None:
			return bool(guard(operation, tab))
		reply = PySide6.QtWidgets.QMessageBox.question(
			self,
			self.tr("Unsaved Changes"),
			self.tr("Discard unsaved Rust changes before %s?") % operation,
			PySide6.QtWidgets.QMessageBox.StandardButton.Discard
			| PySide6.QtWidgets.QMessageBox.StandardButton.Cancel,
			PySide6.QtWidgets.QMessageBox.StandardButton.Cancel,
		)
		return reply == PySide6.QtWidgets.QMessageBox.StandardButton.Discard

	#============================================
	def _confirm_native_tabs_for_shutdown(self) -> bool:
		"""Run the native dirty guard for every page before terminal teardown."""
		for tab in tuple(self._native_tabs_by_page.values()):
			if not self._confirm_native_tab_close("closing Ferrum-Qt", tab):
				index = self._tab_widget.indexOf(tab)
				if index >= 0:
					self._tab_widget.setCurrentIndex(index)
				return False
		return True

	#============================================
	def _dispose_native_tabs_for_shutdown(self) -> None:
		"""Detach and dispose all native pages after shutdown approval only."""
		pages = tuple(self._native_tabs_by_page.values())
		previous_block = self._tab_widget.blockSignals(True)
		previous_tab_change_blocked = self._tab_change_blocked
		self._tab_change_blocked = True
		try:
			for tab in pages:
				index = self._tab_widget.indexOf(tab)
				if index >= 0:
					self._tab_widget.removeTab(index)
				try:
					tab.selection_changed.disconnect(self._on_native_tab_selection_changed)
				except (RuntimeError, TypeError):
					pass
				tab.hide()
				tab.setParent(None)
			self._native_tabs_by_page.clear()
		finally:
			self._tab_change_blocked = previous_tab_change_blocked
			self._tab_widget.blockSignals(previous_block)
		for tab in pages:
			tab.dispose()
		self._restore_native_action_policy()

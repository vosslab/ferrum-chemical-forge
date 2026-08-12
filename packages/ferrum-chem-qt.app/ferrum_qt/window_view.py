"""Main application window for Ferrum-Qt."""

# Standard Library
import os

# PIP3 modules
import yaml
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
class WindowViewMixin:
	"""Cohesive MainWindow behavior with no MainWindow import."""

	def on_zoom_in(self) -> None:
		"""Zoom in on the canvas."""
		self._view.zoom_in()
	def on_zoom_out(self) -> None:
		"""Zoom out on the canvas."""
		self._view.zoom_out()
	def on_reset_zoom(self) -> None:
		"""Reset zoom to 100%."""
		self._view.reset_zoom()
	def on_zoom_to_fit(self) -> None:
		"""Zoom to page (fit paper in viewport)."""
		self._view.zoom_to_fit()
	def on_zoom_to_content(self) -> None:
		"""Zoom to fit all drawn content."""
		self._view.zoom_to_content()
	def on_toggle_grid(self) -> None:
		"""Toggle grid visibility from toolbar."""
		current = self._scene.grid_visible
		self._on_toggle_grid(not current)
		# keep the menu action checkmark in sync
		self._action_toggle_grid.setChecked(not current)
	def on_toggle_grid_snap(self) -> None:
		"""Toggle snap-to-grid from toolbar or command."""
		current = self._scene.grid_snap_enabled
		self._on_toggle_grid_snap(not current)
		# keep the menu action checkmark in sync
		if hasattr(self, "_action_toggle_grid_snap"):
			self._action_toggle_grid_snap.setChecked(not current)
	def _update_menu_predicates(self) -> None:
		"""Re-evaluate enabled_when predicates on all menu actions.

		Called when selection changes, undo/redo state changes, or
		tab switches to keep menu items in sync with document state.
		"""
		if hasattr(self, '_menu_builder') and self._menu_builder is not None:
			self._menu_builder.update_menu_states(self)
	def _on_mode_changed(self, mode_name: str) -> None:
		"""Handle a mode switch by rebuilding the submode ribbon.

		Shows or hides the edit ribbon based on the mode's
		show_edit_pool flag. Rebuilds the submode ribbon with
		the new mode's submode groups.

		Args:
			mode_name: Name of the newly active mode.
		"""
		mode = self._mode_manager.current_mode
		if mode is None:
			return
		# rebuild the submode ribbon for the new mode
		self._submode_ribbon.rebuild(mode_name)
		# keep submode toolbar always visible at a fixed minimum height
		# to prevent layout jumps when switching between modes
		self._submode_toolbar.setVisible(True)
		self._submode_toolbar.setMinimumHeight(32)
		# show/hide the edit ribbon based on mode's show_edit_pool flag
		show_edit = getattr(mode, 'show_edit_pool', False)
		self._edit_ribbon_toolbar.setVisible(show_edit)
	def _on_submode_selected(self, key: str) -> None:
		"""Forward a submode button click to the active mode.

		Args:
			key: The submode key string selected in the ribbon.
		"""
		mode = self._mode_manager.current_mode
		if mode is not None:
			mode.set_submode(key)
	def _on_toggle_grid(self, checked: bool) -> None:
		"""Toggle the grid visibility on the scene.

		Args:
			checked: Whether the grid action is checked.
		"""
		self._scene.set_grid_visible(checked)
		self._prefs.set_value(
			ferrum_qt.config.preferences.Preferences.KEY_GRID_VISIBLE, checked
		)
	def _on_toggle_grid_snap(self, checked: bool) -> None:
		"""Toggle snap-to-grid behavior on the scene.

		Args:
			checked: Whether the snap action is checked.
		"""
		self._scene.set_grid_snap_enabled(checked)
		self._prefs.set_value(
			ferrum_qt.config.preferences.Preferences.KEY_GRID_SNAP_ENABLED,
			checked,
		)
		if checked:
			self.statusBar().showMessage(self.tr("Snap to grid enabled"), 2000)
		else:
			self.statusBar().showMessage(self.tr("Snap to grid disabled"), 2000)
	def _on_toggle_theme(self) -> None:
		"""Toggle between dark and light themes."""
		self._theme_manager.toggle_theme()
	def _on_choose_theme(self) -> None:
		"""Open the theme chooser dialog and apply the selected theme."""
		current = self._theme_manager.current_theme
		chosen = ferrum_qt.dialogs.theme_chooser_dialog.ThemeChooserDialog \
			.choose_theme(self, current)
		# apply only if user selected a different theme
		if chosen is not None and chosen != current:
			self._theme_manager.apply_theme(chosen)
	def _on_theme_changed(self, theme_name: str) -> None:
		"""Handle a theme change by refreshing icons and updating menu text.

		Args:
			theme_name: The new theme name ('dark' or 'light').
		"""
		# update icon_loader theme and clear cache
		ferrum_qt.widgets.icon_loader.set_theme(theme_name)
		ferrum_qt.widgets.icon_loader.reload_icons()

		# refresh mode toolbar icons
		modes_yaml_path = ferrum_qt.setup.mode_setup.get_modes_yaml_path()
		modes_config = {}
		if modes_yaml_path.is_file():
			with open(modes_yaml_path, "r") as fh:
				modes_config = yaml.safe_load(fh) or {}
		modes_defs = modes_config.get("modes", {})
		for name, action in self._mode_toolbar._actions.items():
			# look up the icon name from modes.yaml
			mode_def = modes_defs.get(name, {})
			icon_name = mode_def.get("icon", name)
			icon = ferrum_qt.widgets.icon_loader.get_icon(icon_name)
			self._mode_toolbar.update_action_icon(name, icon)

		# update every session canvas from the YAML theme
		ferrum_qt.themes.theme_loader.clear_cache()
		surround = ferrum_qt.themes.theme_loader.get_canvas_surround(theme_name)
		for session in self._sessions:
			session.view.set_background_color(surround)
			session.scene.apply_theme(theme_name)

		# update chemistry and canvas colors from new theme
		ferrum_qt.setup.canvas_setup._apply_theme_colors(theme_name)

		# refresh submode ribbon icons for new theme
		mode = self._mode_manager.current_mode
		if mode is not None:
			mode_name = mode.name
			# find the registered name for rebuild
			for name in self._mode_manager.mode_names():
				if self._mode_manager._modes[name] is mode:
					mode_name = name
					break
			self._submode_ribbon.rebuild(mode_name)
	def _apply_geometry_preferences(self) -> None:
		"""Apply canonical geometry settings and remove legacy keys."""
		bond_length_pt = ferrum_qt.config.geometry_units.resolve_bond_length_pt(
			self._prefs
		)
		for session in self._sessions:
			session.scene.set_grid_spacing_pt(bond_length_pt)
		self._prefs.remove_value(
			ferrum_qt.config.preferences.Preferences.KEY_BOND_LENGTH
		)
	def _apply_view_preferences(self) -> None:
		"""Apply persisted view toggles (grid visibility and snapping)."""
		grid_visible = bool(self._prefs.value(
			ferrum_qt.config.preferences.Preferences.KEY_GRID_VISIBLE,
			True,
		))
		grid_snap_enabled = bool(self._prefs.value(
			ferrum_qt.config.preferences.Preferences.KEY_GRID_SNAP_ENABLED,
			True,
		))
		for session in self._sessions:
			session.scene.set_grid_visible(grid_visible)
			session.scene.set_grid_snap_enabled(grid_snap_enabled)
		if hasattr(self, "_action_toggle_grid"):
			self._action_toggle_grid.setChecked(grid_visible)
		if hasattr(self, "_action_toggle_grid_snap"):
			self._action_toggle_grid_snap.setChecked(grid_snap_enabled)
	def refresh_recent_files_menu(self) -> None:
		"""Rebuild the Recent files submenu from preferences.

		Clears the existing submenu entries and repopulates from
		the stored recent files list. Each entry shows just the
		filename, with the full path as a tooltip. When the list
		is empty, shows a single disabled placeholder entry.
		"""
		if self._adapter.get_menu_component("Recent files") is None:
			return
		# read the current recent files list
		recent = self._prefs.value(
			ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES
		)
		# QSettings may return a string for single-item lists
		if recent is None:
			recent = []
		elif isinstance(recent, str):
			recent = [recent] if recent else []
		commands = []
		if not recent:
			commands.append((self.tr("(No recent files)"), None, False, None))
		else:
			for file_path in recent:
				# show just the filename as the menu label
				display_name = os.path.basename(file_path)
				def open_recent_file(
						_checked: bool = False, path: str = file_path,
						) -> None:
					"""Open the immutable path bound to one dynamic menu action."""
					self._open_recent_file(path)
				commands.append((display_name, open_recent_file, True, file_path))
		self._adapter.replace_cascade_commands("Recent files", commands)
	def _open_recent_file(self, file_path: str) -> None:
		"""Open a file from the recent files list.

		Verifies the file still exists before attempting to load.

		Args:
			file_path: Absolute path to the file to open.
		"""
		if not os.path.isfile(file_path):
			PySide6.QtWidgets.QMessageBox.warning(
				self, self.tr("File Not Found"),
				self.tr("The file no longer exists:\n%s") % file_path,
			)
			return
		self.open_file_path(file_path)
	def _on_preferences(self) -> None:
		"""Show the preferences dialog."""
		accepted = ferrum_qt.dialogs.preferences_dialog.PreferencesDialog \
			.show_preferences(self)
		if accepted:
			chosen_theme = str(self._prefs.value(
				ferrum_qt.config.preferences.Preferences.KEY_THEME,
				self._theme_manager.current_theme,
			))
			if chosen_theme != self._theme_manager.current_theme:
				self._theme_manager.apply_theme(chosen_theme)
			self._apply_geometry_preferences()
			self._apply_view_preferences()
			self.statusBar().showMessage(
				self.tr(
					"Preferences saved. Display and drawing changes are applied now; "
					"shortcuts are loaded when Ferrum starts."
				),
				5000,
			)
	def _on_about(self) -> None:
		"""Show the About dialog."""
		ferrum_qt.dialogs.about_dialog.AboutDialog.show_about(self)
	def _on_element_changed(self, symbol: str) -> None:
		"""Forward element change from ribbon to active Draw/Atom mode.

		Args:
			symbol: New element symbol.
		"""
		symbol = str(symbol).strip()
		if not symbol:
			return
		mode = self._mode_manager.current_mode
		set_element = getattr(mode, "set_element", None)
		if callable(set_element):
			set_element(symbol)
			return
		if hasattr(mode, 'current_element'):
			try:
				mode.current_element = symbol
			except AttributeError:
				pass
	def _on_bond_order_changed(self, order: int) -> None:
		"""Forward bond order change from ribbon to draw mode.

		Args:
			order: New bond order.
		"""
		mode = self._mode_manager.current_mode
		if hasattr(mode, 'current_bond_order'):
			mode.current_bond_order = order
	def _on_bond_type_changed(self, bond_type: str) -> None:
		"""Forward bond type change from ribbon to draw mode.

		Args:
			bond_type: New bond type character.
		"""
		mode = self._mode_manager.current_mode
		if hasattr(mode, 'current_bond_type'):
			mode.current_bond_type = bond_type
	def restore_geometry(self) -> None:
		"""Restore window geometry from saved preferences.

		Only restores window size and position, not toolbar state,
		because toolbar layout changes between versions would conflict
		with stale saved state.
		"""
		geometry = self._prefs.value(
			ferrum_qt.config.preferences.Preferences.KEY_WINDOW_GEOMETRY
		)
		if geometry is not None:
			self.restoreGeometry(geometry)
	def prepare_application_shutdown(self) -> bool:
		"""Approve and start the one terminal MainWindow retirement sequence.

		Returns:
			True when every live session has entered the terminal reaper path.
			False when a Save, Recovery Export, or Cancel decision keeps the
			window live.

		This is the explicit application-lifetime boundary shared by an ordinary
		window close and an event loop that ends programmatically.  It owns the
		ordered Qt teardown only after the existing close decisions approve it.
		"""
		if self._shutdown_prepared:
			return True
		for session in tuple(self._sessions):
			if not self._confirm_save_if_dirty(
					"closing Ferrum-Qt", session,
			):
				self._select_session(session)
				return False
		self._prefs.set_value(
			ferrum_qt.config.preferences.Preferences.KEY_WINDOW_GEOMETRY,
			self.saveGeometry(),
		)
		self._shutdown_prepared = True
		self._shutdown_state = ShutdownState.DRAINING
		self._stop_import_workers()
		try:
			self._clipboard.dataChanged.disconnect(self._on_clipboard_data_changed)
		except (RuntimeError, TypeError):
			pass
		if self._active_session is not None and self._ui_signals_connected:
			self._disconnect_active_session_signals(self._active_session)
		self._bind_property_dock(None)

		sessions = tuple(self._sessions)
		previous_block = self._tab_widget.blockSignals(True)
		self._tab_change_blocked = True
		try:
			for session in sessions:
				index = self._tab_widget.indexOf(session.view)
				if index >= 0:
					self._detach_tab_page(session, index)
			self._sessions.clear()
			self._sessions_by_view.clear()
			self._clear_active_session_aliases()
		finally:
			self._tab_change_blocked = False
			self._tab_widget.blockSignals(previous_block)

		for session in sessions:
			self._disconnect_session_title(session)
			self._shutdown_sessions_pending_disposal.append(session)
		self._emit_worker_retirement_drained()
		return True
	def closeEvent(self, event: PySide6.QtGui.QCloseEvent) -> None:
		"""Guard unsaved work and tear down Qt callbacks before closing.

		Args:
			event: The close event.
		"""
		if not self.prepare_application_shutdown():
			event.ignore()
			return
		super().closeEvent(event)

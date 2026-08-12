"""Main application window for Ferrum-Qt."""

# Standard Library
import functools

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
class WindowSessionSetupMixin:
	"""One cohesive MainWindow session responsibility."""

	def _connect_session_title(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> None:
		"""Install MainWindow's one title subscription for a registered session."""
		connection_key = id(session)
		connection = self._session_title_connections.get(connection_key)
		if connection is not None and connection[0] is session:
			return
		if connection is not None:
			raise RuntimeError("A different session reused a live title connection key")
		# PySide creates a new Python bound-method wrapper for every attribute
		# lookup. Retain the exact relay passed to connect() so this owner can
		# later disconnect that same subscription.
		slot = functools.partial(self._update_session_tab_title, session)
		session.title_changed.connect(slot)
		self._session_title_connections[connection_key] = (session, slot)

	def _disconnect_session_title(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> None:
		"""Retire MainWindow's title subscription once, before session disposal."""
		connection_key = id(session)
		connection = self._session_title_connections.pop(connection_key, None)
		if connection is None:
			return
		connected_session, slot = connection
		if connected_session is not session:
			raise RuntimeError("Title connection registry no longer owns this session")
		session.title_changed.disconnect(slot)

	def _setup_canvas(self) -> None:
		"""Create the tab host and its initial independent document session."""
		self._tab_widget = PySide6.QtWidgets.QTabWidget(self)
		self._tab_widget.setTabsClosable(True)
		self._tab_widget.setMovable(False)
		self.setCentralWidget(self._tab_widget)
		session = self._create_session(activate=True)
		self._set_active_session_aliases(session)
	def _construct_session(
			self, *,
			file_path: str | None = None, display_name: str | None = None,
			origin_path: str | None = None,
			prepared_native_cdml: (
				ferrum_qt.models.document_session.PreparedNativeCDML | None
			) = None,
			prepared_imported_cdml: (
				ferrum_qt.models.document_session.PreparedImportedCDML | None
			) = None,
			) -> ferrum_qt.models.document_session.DocumentSession:
		"""Build one detached session without changing the live tab graph."""
		return ferrum_qt.models.document_session.DocumentSession(
			parent=self,
			theme_manager=self._theme_manager,
			prefs=self._prefs,
			mode_host=self,
			view_parent=self,
			file_path=file_path,
			display_name=display_name,
			origin_path=origin_path,
			prepared_native_cdml=prepared_native_cdml,
			prepared_imported_cdml=prepared_imported_cdml,
			user_template_catalog=self._user_template_catalog.entries,
		)
	def _register_session(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			*, index: int | None = None, activate: bool = True,
			) -> ferrum_qt.models.document_session.DocumentSession:
		"""Register one viable detached session without disturbing another tab."""
		if session.is_disposed or session in self._sessions:
			raise ValueError("Session must be live and unregistered")
		if session.view in self._sessions_by_view:
			raise ValueError("Session view is already registered")
		if index is None:
			index = self._tab_widget.count()
		if index < 0 or index > len(self._sessions):
			raise IndexError("Session insertion index is out of range")

		previous_session = self._active_session
		previous_index = self._tab_widget.currentIndex()
		title_connected = False
		previous_block = self._tab_widget.blockSignals(True)
		previous_tab_change_blocked = self._tab_change_blocked
		self._tab_change_blocked = True
		try:
			self._sessions.insert(index, session)
			self._sessions_by_view[session.view] = session
			self._tab_widget.insertTab(index, session.view, session.title)
			self._connect_session_title(session)
			title_connected = True
			if activate:
				self._tab_widget.setCurrentIndex(index)
				if self._active_session is not session:
					self._activate_session(session)
			else:
				current_mode = session.mode_manager.current_mode
				if current_mode is not None:
					current_mode.deactivate()
		except Exception:
			self._unregister_session_without_disposal(session, title_connected)
			if previous_session is not None and previous_session in self._sessions:
				self._restore_active_session(previous_session, previous_index)
			elif self._active_session is session:
				self._clear_active_session_aliases()
			self._dispose_session_later(session)
			raise
		finally:
			self._tab_change_blocked = previous_tab_change_blocked
			self._tab_widget.blockSignals(previous_block)
		session.install_projection_lifecycle_port(
			ferrum_qt.models.projection_lifecycle.SessionProjectionLifecyclePort(
				session,
				lambda snapshot: self._replace_session_projection(session, snapshot),
				self._consume_session_projection_notice,
			),
		)
		return session
	def _consume_session_projection_notice(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			result: ferrum_qt.models.projection_lifecycle.ProjectionLifecycleResult,
			) -> None:
		"""Refresh only the emitting active session's disposable UI aliases."""
		if session.is_disposed or session not in self._sessions:
			return
		if session is not self._active_session:
			return
		self._set_active_session_aliases(session)
		self._refresh_active_session_controls()
	def _create_session(
			self, index: int | None = None, activate: bool = True,
			display_name: str | None = None, origin_path: str | None = None,
			) -> ferrum_qt.models.document_session.DocumentSession:
		"""Create, register, and optionally activate one tab session."""
		session = self._construct_session(
			display_name=display_name,
			origin_path=origin_path,
		)
		return self._register_session(session, index=index, activate=activate)
	def _set_active_session_aliases(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> None:
		"""Point compatibility aliases at exactly one active session."""
		self._active_session = session
		self._document = session.document
		self._scene = session.scene
		self._view = session.view
		self._mode_manager = session.mode_manager
	def _clear_active_session_aliases(self) -> None:
		"""Clear compatibility aliases while no live session is active."""
		self._active_session = None
		self._document = None
		self._scene = None
		self._view = None
		self._mode_manager = None
	def _replace_session_projection(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			snapshot: object,
			) -> ferrum_qt.models.projection_lifecycle.ProjectionLifecycleResult:
		"""Rebuild one registered Qt projection from an accepted backend snapshot.

		The session, tab, scene, view, modes, and workers remain in place.  Only
		the document-owned graphics and models are replaced after the exact current
		backend snapshot has prepared successfully.
		"""
		if session.is_disposed or session not in self._sessions:
			return ferrum_qt.models.projection_lifecycle.ProjectionLifecycleResult(
				ferrum_qt.models.projection_lifecycle.ProjectionLifecycleStatus.SESSION_UNAVAILABLE,
				ferrum_qt.models.projection_lifecycle.ProjectionLifecyclePhase.SESSION,
			)
		active = session is self._active_session
		old_document = session.document
		if active:
			if self._ui_signals_connected and old_document is not None:
				self._disconnect_document_signals(old_document)
			self._bind_property_dock(None)
		try:
			replaced = session.replace_projection_from_backend_snapshot(snapshot)
		except Exception:
			if active and old_document is not None and session.document is old_document:
				if self._ui_signals_connected:
					self._connect_document_signals(old_document)
				self._bind_property_dock(session)
			elif active and session.document is None:
				self._set_active_session_aliases(session)
				current_mode = session.mode_manager.current_mode
				if current_mode is not None:
					current_mode.deactivate()
				self._bind_property_dock(None)
			raise
		if active:
			self._set_active_session_aliases(session)
			if replaced.installed and self._ui_signals_connected and session.document is not None:
				self._connect_document_signals(session.document)
			if replaced.installed:
				self._bind_property_dock(session)
			else:
				self._bind_property_dock(None)
				if session.document is None:
					current_mode = session.mode_manager.current_mode
					if current_mode is not None:
						current_mode.deactivate()
		return replaced
	def _setup_mode_system(self) -> None:
		"""Expose the active session's already-owned mode manager."""
		self._mode_manager = self._active_session.mode_manager
	def _setup_menus(self) -> None:
		"""Create the menu bar from YAML menu structure and action registry."""
		from ferrum_qt.actions.action_registry import register_all_actions
		from ferrum_qt.actions.platform_menu import PlatformMenuAdapter
		from ferrum_qt.actions.menu_builder import (
			MenuBuilder,
			preflight_required_menu_actions,
		)
		# register all per-menu action modules
		self._registry = register_all_actions(self)
		# create the Qt menu adapter wrapping QMenuBar
		self._adapter = PlatformMenuAdapter(self)
		# Load the menu definition installed with the Qt package.
		yaml_path = str(ferrum_qt.resource_paths.get_resource_path("menus.yaml"))
		preflight_required_menu_actions(self._registry, yaml_path)
		# build all menus from YAML structure
		self._menu_builder = MenuBuilder(
			yaml_path, self._registry, self._adapter,
		)
		self._menu_builder.build_menus()
		# The Import cascade is driven by the same capability registry as the
		# file chooser and extension router.  Each action delegates to the
		# session-aware loader instead of starting an independent worker.
		for capability in (
				ferrum_qt.io.import_capabilities.worker_import_capabilities()
		):
			self._adapter.add_command_to_cascade(
				"Import",
				self.tr("Import %s..." % capability.label),
				self.tr("Import %s" % capability.description),
				lambda _checked=False, capability=capability: (
					ferrum_qt.actions.file_actions.import_capability(
						self, capability,
					)
				),
				action_key="file.import.%s" % capability.codec_name,
			)
		# populate the Export cascade with export handlers
		export_cascade_label = "Export"
		self._adapter.add_command_to_cascade(
			export_cascade_label, "Export SVG...",
			"Export the current document as SVG",
			self._on_export_svg,
			action_key="file.export_svg",
		)
		self._adapter.add_command_to_cascade(
			export_cascade_label, "Export PNG...",
			"Export the current document as PNG",
			self._on_export_png,
			action_key="file.export_png",
		)
		self._adapter.add_command_to_cascade(
			export_cascade_label, "Export PDF...",
			"Export the current document as PDF",
			self._on_export_pdf,
			action_key="file.export_pdf",
		)
		# retrieve QActions by frozen English key for later enable/disable
		self._action_save = self._adapter.get_action_by_key("file.save")
		self._action_open = self._adapter.get_action_by_key("file.load")
		self._action_new = self._adapter.get_action_by_key("file.new")
		self._action_exit = self._adapter.get_action_by_key("file.exit")
		self._action_undo = self._adapter.get_action_by_key("edit.undo")
		self._action_redo = self._adapter.get_action_by_key("edit.redo")
		self._action_toggle_theme = self._adapter.get_action_by_key(
			"options.theme"
		)
		self._action_about = self._adapter.get_action_by_key("help.about")
		# grid toggle is not in menus.yaml (it is a view feature)
		# create it as a standalone checkable action
		view_menu = self._adapter.get_menu_component("View")
		if view_menu is not None:
			self._adapter.add_separator("View")
			self._action_toggle_grid = self._adapter.add_direct_action(
				"View", self.tr("Toggle &Grid"), "view.toggle_grid",
			)
			self._action_toggle_grid.setCheckable(True)
			self._action_toggle_grid.setChecked(self._scene.grid_visible)
			self._action_toggle_grid.triggered.connect(self._on_toggle_grid)
			self._action_toggle_grid_snap = self._adapter.add_direct_action(
				"View", self.tr("Snap To &Grid"), "view.toggle_grid_snap",
			)
			self._action_toggle_grid_snap.setCheckable(True)
			self._action_toggle_grid_snap.setChecked(
				self._scene.grid_snap_enabled
			)
			self._action_toggle_grid_snap.setShortcut(
				PySide6.QtGui.QKeySequence(self.tr("Shift+Ctrl+G"))
			)
			self._action_toggle_grid_snap.triggered.connect(
				self._on_toggle_grid_snap
			)
		# populate the Recent files cascade from stored preferences
		self.refresh_recent_files_menu()
		# Install one shortcut authority after every menu and direct action exists.
		# Its callbacks resolve the active document session when invoked.
		self._keybinding_manager = ferrum_qt.config.keybindings.KeybindingManager(
			self, self._registry, parent=self,
		)
		self._keybinding_manager.setup_shortcuts()
	def _setup_toolbars(self) -> None:
		"""Create the mode toolbar, submode ribbon, edit ribbon, and docks."""
		widgets = ferrum_qt.setup.toolbar_setup.setup_toolbars(
			self, self._mode_manager, self._document, self._theme_manager,
		)
		self._mode_toolbar = widgets["mode_toolbar"]
		self._submode_ribbon = widgets["submode_ribbon"]
		self._submode_toolbar = widgets["submode_toolbar"]
		self._edit_ribbon = widgets["edit_ribbon"]
		self._edit_ribbon_toolbar = widgets["edit_ribbon_toolbar"]
		self._property_dock = widgets["property_dock"]
		self._undo_action = widgets["undo_action"]
		self._redo_action = widgets["redo_action"]
	def _setup_status_bar(self) -> None:
		"""Create and install the status bar with zoom controls."""
		self._status_bar = ferrum_qt.widgets.status_bar.StatusBar(self)
		self.setStatusBar(self._status_bar)
		# add zoom controls as a permanent widget on the right
		self._zoom_controls = ferrum_qt.widgets.zoom_controls.ZoomControls(self)
		self._status_bar.addPermanentWidget(self._zoom_controls)
	def _connect_signals(self) -> None:
		"""Wire all signals between components."""
		self._tab_widget.currentChanged.connect(self._on_tab_changed)
		self._tab_widget.tabCloseRequested.connect(
			self._on_tab_close_requested
		)

		# Global controls resolve the active session at invocation time.
		self._mode_toolbar.mode_selected.connect(self._on_mode_selected)

		# submode ribbon -> active mode submode selection
		self._submode_ribbon.submode_selected.connect(
			self._on_submode_selected
		)

		# edit ribbon -> draw mode
		self._edit_ribbon.element_changed.connect(self._on_element_changed)
		self._edit_ribbon.bond_order_changed.connect(self._on_bond_order_changed)
		self._edit_ribbon.bond_type_changed.connect(self._on_bond_type_changed)

		# theme changes -> icon refresh and menu text update
		self._theme_manager.theme_changed.connect(self._on_theme_changed)

		# zoom controls -> handler methods
		self._zoom_controls.zoom_in_clicked.connect(self.on_zoom_in)
		self._zoom_controls.zoom_out_clicked.connect(self.on_zoom_out)
		self._zoom_controls.reset_zoom_clicked.connect(self.on_reset_zoom)
		self._zoom_controls.zoom_to_fit_clicked.connect(self.on_zoom_to_fit)
		self._zoom_controls.zoom_to_content_clicked.connect(
			self.on_zoom_to_content
		)
		self._zoom_controls.zoom_slider_changed.connect(
			self._on_zoom_slider_changed
		)
		self._clipboard.dataChanged.connect(self._on_clipboard_data_changed)

		self._ui_signals_connected = True
		self._connect_active_session_signals(self._active_session)
		self._refresh_document_actions()

		# trigger initial mode visibility (submode ribbon + edit ribbon)
		self._synchronize_active_session_ui()
	@PySide6.QtCore.Slot()
	def _on_clipboard_data_changed(self) -> None:
		"""Refresh document actions after a system clipboard transition."""
		if self._shutdown_prepared or not self._ui_signals_connected:
			return
		self._refresh_document_actions()

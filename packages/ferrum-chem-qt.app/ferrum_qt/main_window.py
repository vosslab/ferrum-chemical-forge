"""Ordinary Ferrum application window for Ferrum."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.actions.command_palette
import ferrum_qt.actions.menu_builder
import ferrum_qt.actions.platform_menu
import ferrum_qt.config.keybindings
import ferrum_qt.config.preferences
import ferrum_qt.declarative_resource_preflight
import ferrum_qt.dialogs.about_dialog
import ferrum_qt.ferrum.authoring_ribbon
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.drawing_parameters_client
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.preferences
import ferrum_qt.ferrum.recent_files
import ferrum_qt.ferrum.reaction_composer
import ferrum_qt.ferrum.reaction_inspector
import ferrum_qt.ferrum.smarts_query_dock
import ferrum_qt.ferrum.window_shared_seams
import ferrum_qt.themes.theme_loader
import ferrum_qt.themes.theme_manager
import ferrum_qt.themes.document_display_palette


#============================================
class MainWindow(ferrum_qt.ferrum.main_window.FerrumNativeMainWindow):
	"""Start the product host with one empty Rust-owned document.

	External uncompressed CDML uses the same Rust-owned local V1 profile as the
	native render CLI and never loads document bytes through Python.
	"""

	#============================================
	def __init__(
			self, theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
			parent: PySide6.QtWidgets.QWidget | None = None, *,
			user_template_directory: object = None,
			) -> None:
		"""Build the ordinary native-only host and its initial empty document."""
		if type(theme_manager) is not ferrum_qt.themes.theme_manager.ThemeManager:
			raise TypeError("Ferrum main window requires ThemeManager")
		super().__init__(parent, user_template_directory=user_template_directory)
		self._theme_manager = theme_manager
		self._document_theme_change: ferrum_qt.themes.theme_manager.ThemeChangeV1
		theme_manager.theme_changed.connect(self._apply_document_theme_change)
		self._apply_document_theme_change(
			ferrum_qt.themes.theme_manager.ThemeChangeV1(
				theme_manager.current_theme,
				ferrum_qt.themes.theme_loader.get_document_display_palette(
					theme_manager.current_theme,
				),
			),
		)
		self._drawing_parameters = (
			ferrum_qt.ferrum.drawing_parameters.
			FerrumNativeDrawingParameters.shared_application_model(self._prefs)
		)
		self._set_native_hex_grid_visible(
			ferrum_qt.ferrum.preferences.hex_grid_visible_preference(
				self._prefs,
			),
		)
		self._set_native_hex_grid_snap_enabled(
			ferrum_qt.ferrum.preferences.
			hex_grid_snap_enabled_preference(self._prefs),
		)
		self._shutdown_prepared = False
		self.setWindowTitle(self.tr("Ferrum"))
		self.resize(1280, 800)
		self._action_open = self._open_action
		self._action_new = self._add_new_document_action()
		self._preferences_action = self._add_preferences_action()
		self._next_drawing_action = self._add_next_drawing_action()
		self._about_action = self._add_about_action()
		self._reaction_composer = ferrum_qt.ferrum.reaction_composer.ReactionComposerController(self)
		self._create_reaction_action = self._reaction_composer.install_action()
		self._reaction_inspector = ferrum_qt.ferrum.reaction_inspector.ReactionInspectorController(self)
		self._reaction_inspector_action = self._reaction_inspector.install_action()
		self._smarts_query_action = self._smarts_query_controller.install_action()
		self._command_palette_controller = (
			ferrum_qt.actions.command_palette.CommandPaletteController(
				self, self._action_registry,
			)
		)
		self._command_palette_action = self._add_command_palette_action()
		self._action_registry.register_dynamic_menu(
			"file.recent", self._recent_files_menu,
			"Recent-file actions rebuild from user preferences whenever the menu opens.",
		)
		ferrum_qt.ferrum.window_shared_seams.install_shared_window_seams(
			self, self._action_registry,
		)
		ferrum_qt.declarative_resource_preflight.preflight_window_resources(
			self._action_registry,
		)
		self._declared_menus = ferrum_qt.actions.menu_builder.build_declared_menus(
			self, self._action_registry,
		)
		self._file_menu = self._declared_menus["file"]
		self._edit_menu = self._declared_menus["edit"]
		self._draw_menu = self._declared_menus["draw"]
		self._view_menu = self._declared_menus["view"]
		self._chemistry_menu = self._declared_menus["chemistry"]
		ferrum_qt.actions.platform_menu.apply_platform_menu_roles(self._action_registry)
		self._authoring_ribbon = self._add_authoring_ribbon()
		self._keybinding_manager = ferrum_qt.config.keybindings.KeybindingManager(
			self, self._action_registry,
		)
		self._keybinding_manager.setup_shortcuts()
		self._keybinding_manager.validate_live_shortcuts()
		self._on_new()
		bootstrap = self._active_native_tab()
		if bootstrap is not None:
			bootstrap._mark_initial_placeholder()

	#============================================
	def _add_about_action(self) -> PySide6.QtGui.QAction:
		"""Install the standard application-information route."""
		action = PySide6.QtGui.QAction(self.tr("About Ferrum"), self)
		action.setToolTip(self.tr("Show Ferrum version and license information"))
		action.triggered.connect(
		lambda _checked=False: (
			ferrum_qt.dialogs.about_dialog.AboutDialog.show_about(self)
		),
		)
		action.setMenuRole(PySide6.QtGui.QAction.MenuRole.AboutRole)
		self._register_action("help.about", action)
		return action

	#============================================
	def _add_command_palette_action(self) -> PySide6.QtGui.QAction:
		"""Install the registry-driven keyboard command discovery route."""
		action = PySide6.QtGui.QAction(self.tr("Command Palette..."), self)
		action.setToolTip(self.tr(
			"Search and run available Ferrum commands from the keyboard",
		))
		action.triggered.connect(self._command_palette_controller.open)
		self._register_action("view.command_palette", action)
		return action

	#============================================
	def _add_new_document_action(self) -> PySide6.QtGui.QAction:
		"""Install the window-level Ferrum New action."""
		action = PySide6.QtGui.QAction(self.tr("New"), self)
		action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.New)
		action.setToolTip(self.tr("Create a new empty Rust-owned Ferrum document"))
		action.triggered.connect(self._on_new)
		self._register_action("file.new", action)
		return action

	#============================================
	def _install_native_recent_files_menu(self) -> None:
		"""Create the dynamic File cascade without placing it.

		``menus.yaml`` owns its eventual File-menu position.  Recent-file contents
		remain owned by the preferences-backed controller.
		"""
		self._recent_files_menu = self._native_recent_files.create_menu()

	#============================================
	def _initialize_native_file_menu_clients(self) -> None:
		"""Create ordinary personal menu owners after the Qt window is initialized."""
		self._prefs = ferrum_qt.config.preferences.Preferences.instance()
		self._native_recent_files = (
			ferrum_qt.ferrum.recent_files.FerrumNativeRecentFiles(
				self, self._prefs,
			)
		)

	#============================================
	def _add_authoring_ribbon(
			self,
			) -> ferrum_qt.ferrum.authoring_ribbon.AuthoringRibbon:
		"""Install the one responsive, action-reusing Ferrum authoring surface."""
		standard = PySide6.QtWidgets.QStyle.StandardPixmap
		self._open_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Open)
		self._save_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Save)
		self._undo_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Undo)
		self._redo_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Redo)
		cut_icon = PySide6.QtGui.QIcon.fromTheme("edit-cut")
		if not cut_icon.isNull():
			self._cut_action.setIcon(cut_icon)
		for action, icon_source in (
			(self._action_new, standard.SP_FileIcon),
			(self._open_action, standard.SP_DialogOpenButton),
			(self._save_action, standard.SP_DialogSaveButton),
			(self._undo_action, standard.SP_ArrowBack),
			(self._redo_action, standard.SP_ArrowForward),
			(self._cut_action, standard.SP_TrashIcon),
			(self._copy_action, standard.SP_FileDialogDetailedView),
			(self._paste_action, standard.SP_FileDialogListView),
			(self._zoom_out_action, standard.SP_ArrowDown),
			(self._zoom_100_action, standard.SP_BrowserReload),
			(self._zoom_in_action, standard.SP_ArrowUp),
			(self._show_hex_grid_action, standard.SP_FileDialogContentsView),
			(self._snap_hex_grid_action, standard.SP_DialogApplyButton),
			(self._template_catalog_action, standard.SP_FileDialogNewFolder),
			(self._create_reaction_action, standard.SP_FileDialogDetailedView),
			(self._reaction_inspector_action, standard.SP_FileDialogContentsView),
		):
			if action.icon().isNull():
				action.setIcon(self.style().standardIcon(icon_source))
		ribbon = ferrum_qt.ferrum.authoring_ribbon.AuthoringRibbon(
			self._action_registry, self._window_mode_sync, self._drawing_parameters,
			self._next_drawing_action,
			self._cancel_tool_action, self,
		)
		self.addToolBar(PySide6.QtCore.Qt.ToolBarArea.TopToolBarArea, ribbon)
		return ribbon

	#============================================
	#============================================
	def _on_native_tab_changed(self, index: int) -> None:
		"""Close a composer before another document becomes active."""
		composer = getattr(self, "_reaction_composer", None)
		if composer is not None:
			composer.close()
		inspector = getattr(self, "_reaction_inspector", None)
		if inspector is not None:
			inspector.close()
		super()._on_native_tab_changed(index)

	#============================================
	def _apply_document_theme_change(
			self, change: ferrum_qt.themes.theme_manager.ThemeChangeV1,
			) -> None:
		"""Deliver one typed display palette to every live native document tab."""
		if type(change) is not ferrum_qt.themes.theme_manager.ThemeChangeV1:
			raise TypeError("Ferrum main window requires ThemeChangeV1")
		if type(change.palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
			raise TypeError("Ferrum main window requires a document display palette")
		self._document_theme_change = change
		for tab in self.findChildren(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab):
			tab.apply_theme_change(change)

	#============================================
	def _add_next_drawing_action(self) -> PySide6.QtGui.QAction:
		"""Offer a standard menu and toolbar route to shared drawing preferences."""
		action = PySide6.QtGui.QAction(self.tr("Next Drawing..."), self)
		action.setPriority(PySide6.QtGui.QAction.Priority.LowPriority)
		action.setIcon(self.style().standardIcon(
			PySide6.QtWidgets.QStyle.StandardPixmap.SP_FileDialogDetailedView,
		))
		action.setToolTip(self.tr(
			"Choose the next atom, bond order, and bond presentation for Ferrum drawing",
		))
		action.triggered.connect(self._show_next_drawing_dialog)
		self._register_action("draw.next_drawing", action)
		return action

	#============================================
	def _show_next_drawing_dialog(self) -> None:
		"""Open a compact view of the shared application-owned drawing choices."""
		ferrum_qt.ferrum.drawing_parameters_client \
			.show_native_drawing_parameters_dialog(
				self, self._drawing_parameters, self._cancel_tool_action,
			)

	#============================================
	def _add_preferences_action(self) -> PySide6.QtGui.QAction:
		"""Expose only settings owned by the ordinary application window."""
		return ferrum_qt.ferrum.preferences.install_native_preferences_action(self)

	#============================================
	def _create_empty_native_tab(
			self,
			) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		"""Create a revision-zero Rust document."""
		import ferrum_qt.ferrum.engine as engine
		session = engine.DocumentSession.create_empty_document_v1()
		return ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab.from_session(
			session, self.tr("Untitled"), self._require_document_display_palette(),
		)

	#============================================
	def _require_document_display_palette(
			self,
			) -> ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
		"""Return the validated current display palette before a tab is built."""
		return self._document_theme_change.palette

	#============================================
	def _on_new(self) -> bool:
		"""Add one empty Ferrum document tab."""
		if self._shutdown_prepared:
			return False
		try:
			self._register_native_tab(self._create_empty_native_tab(), activate=True)
		except Exception as exc:
			self.statusBar().showMessage(
				self.tr("Could not create a new Ferrum document: %s") % exc, 5000,
			)
			return False
		return True

	#============================================
	def _register_native_tab(
			self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			*, activate: bool = True,
			) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		"""Keep the common host's activation default for Ferrum callers."""
		registered = super()._register_native_tab(tab, activate=activate)
		registered.apply_theme_change(self._document_theme_change)
		registered.view.viewport().installEventFilter(self)
		return registered

	#============================================
	def _save_native_tab_to_path(
			self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			file_path: str,
			) -> bool:
		"""Promote personal recency only after Rust confirms Ferrum publication."""
		saved = super()._save_native_tab_to_path(tab, file_path)
		if saved and tab.file_path is not None:
			self._native_recent_files.record_confirmed_path(tab.file_path)
		return saved

	#============================================
	def _close_native_tab_at(self, index: int,
			decision: ferrum_qt.ferrum.close_decision.CloseDecision,
			) -> ferrum_qt.ferrum.close_decision.CloseResult:
		"""Apply one explicit close decision to the specified Ferrum page."""
		return self._close_tab_at_with_decision(index, decision)

	#============================================
	def _refresh_actions(self, *_unused: object) -> None:
		"""Coalesce synchronous Qt layout callbacks into one settled action refresh."""
		if getattr(self, "_action_refresh_in_progress", False):
			self._action_refresh_requested = True
			return
		self._action_refresh_in_progress = True
		try:
			while True:
				self._action_refresh_requested = False
				self._refresh_actions_once(*_unused)
				if not self._action_refresh_requested:
					return
		finally:
			self._action_refresh_in_progress = False

	#============================================
	def _refresh_actions_once(self, *_unused: object) -> None:
		"""Refresh every action against one stable tab observation.

		A tab is registered before every optional projection capability has been
		installed.  The ordinary host treats that state as non-editable instead of
		letting an action refresh cross a missing projection attribute.
		"""
		tab = self._active_native_tab()
		controller = getattr(tab, "_controller", None)
		if tab is not None and not hasattr(controller, "projection"):
			for action in self.findChildren(PySide6.QtGui.QAction):
				action.setEnabled(False)
			smarts = getattr(self, "_smarts_query_controller", None)
			if smarts is not None:
				smarts.refresh_action(False, True, False)
			return
		super()._refresh_actions(*_unused)
		if not hasattr(self, "_window_mode_sync"):
			return
		ferrum_qt.ferrum.window_shared_seams.refresh_shared_window_seams(self)
		composer = getattr(self, "_reaction_composer", None)
		action = getattr(self, "_create_reaction_action", None)
		if composer is not None and action is not None:
			composer.refresh_action(action)
		inspector = getattr(self, "_reaction_inspector", None)
		inspector_action = getattr(self, "_reaction_inspector_action", None)
		if inspector is not None and inspector_action is not None:
			inspector.refresh_action(inspector_action)
		smarts = getattr(self, "_smarts_query_controller", None)
		if smarts is not None:
			smarts.refresh_action(
				tab is not None and not tab.is_disposed,
				False if tab is None else tab.requires_refresh,
				bool(getattr(smarts, "_busy", False)),
			)

	#============================================
	def prepare_application_shutdown(self) -> bool:
		"""Close clean Ferrum pages before the generic QObject finalizer runs."""
		if self._shutdown_prepared:
			return True
		if self._cancel_local_document_open_for_close():
			return False
		if self._cancel_molecule_imports_for_close():
			return False
		if any(tab.requires_refresh or tab.is_dirty for tab in self._native_tabs_by_page.values()):
			return False
		for tab in tuple(self._native_tabs_by_page.values()):
			index = self._tab_widget.indexOf(tab)
			if index >= 0:
				self._close_tab_at(index)
		self._shutdown_prepared = not self._native_tabs_by_page
		if self._shutdown_prepared:
			if ferrum_qt.ferrum.preferences \
					.remembered_workspace_preference(self._prefs):
				self._prefs.set_value(
					ferrum_qt.config.preferences.Preferences.KEY_WINDOW_GEOMETRY,
					self.saveGeometry(),
				)
				self._prefs.set_value(
					ferrum_qt.config.preferences.Preferences.KEY_WINDOW_STATE,
					self.saveState(1),
				)
		return self._shutdown_prepared

	#============================================
	def restore_workspace(self) -> None:
		"""Restore application workspace state through the Ferrum view controller."""
		if not ferrum_qt.ferrum.preferences \
				.remembered_workspace_preference(self._prefs):
			return
		geometry = self._prefs.value(
			ferrum_qt.config.preferences.Preferences.KEY_WINDOW_GEOMETRY,
		)
		if geometry is not None:
			self.restoreGeometry(geometry)
		state = self._prefs.value(
			ferrum_qt.config.preferences.Preferences.KEY_WINDOW_STATE,
		)
		if state is not None:
			self.restoreState(state, 1)

	#============================================
	def closeEvent(self, event: PySide6.QtGui.QCloseEvent) -> None:
		"""Use the ordinary native-only shutdown policy for a window close."""
		if not self.prepare_application_shutdown():
			event.ignore()
			return
		event.accept()

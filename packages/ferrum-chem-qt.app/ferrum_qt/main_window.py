"""Ordinary Ferrum application window for Ferrum."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.actions.menu_builder
import ferrum_qt.actions.platform_menu
import ferrum_qt.config.keybindings
import ferrum_qt.config.preferences
import ferrum_qt.dialogs.about_dialog
import ferrum_qt.ferrum.action_toolbar
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.drawing_parameters_client
import ferrum_qt.ferrum.editing_tools_toolbar
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.preferences
import ferrum_qt.ferrum.recent_files
import ferrum_qt.ferrum.window_shared_seams


#============================================
class MainWindow(ferrum_qt.ferrum.main_window.FerrumNativeMainWindow):
	"""Start the product host with one empty Rust-owned document.

	External uncompressed CDML uses the same Rust-owned local V1 profile as the
	native render CLI and never loads document bytes through Python.
	"""

	#============================================
	def __init__(
			self, theme_manager: object,
			parent: PySide6.QtWidgets.QWidget | None = None, *,
			user_template_directory: object = None,
			) -> None:
		"""Build the ordinary native-only host and its initial empty document."""
		super().__init__(parent, user_template_directory=user_template_directory)
		self._theme_manager = theme_manager
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
		self._native_action_toolbar = self._add_native_action_toolbar()
		self._native_editing_tools_toolbar = self._add_native_editing_tools_toolbar()
		self._about_action = self._add_about_action()
		self._action_registry = (
			ferrum_qt.actions.action_registry.register_main_window_actions(self)
		)
		self._action_registry.declare_dynamic_lifecycle(
			"recent-files",
			"Recent-file actions are rebuilt from user preferences whenever the cascade opens.",
		)
		self._declared_menus = ferrum_qt.actions.menu_builder.build_declared_menus(
			self, self._action_registry,
		)
		ferrum_qt.actions.platform_menu.apply_platform_menu_roles(self._action_registry)
		self._keybinding_manager = ferrum_qt.config.keybindings.KeybindingManager(
			self, self._action_registry,
		)
		self._keybinding_manager.setup_shortcuts()
		ferrum_qt.ferrum.window_shared_seams.install_shared_window_seams(
			self, self._action_registry,
		)
		self._on_new()
		bootstrap = self._active_native_tab()
		if bootstrap is not None:
			bootstrap._mark_initial_placeholder()

	#============================================
	def _add_about_action(self) -> PySide6.QtGui.QAction:
		"""Install the standard application-information route."""
		help_menu = self.menuBar().addMenu(self.tr("Help"))
		action = PySide6.QtGui.QAction(self.tr("About Ferrum"), self)
		action.setToolTip(self.tr("Show Ferrum version and license information"))
		action.triggered.connect(
		lambda _checked=False: (
			ferrum_qt.dialogs.about_dialog.AboutDialog.show_about(self)
		),
		)
		action.setMenuRole(PySide6.QtGui.QAction.MenuRole.AboutRole)
		help_menu.addAction(action)
		return action

	#============================================
	def _add_new_document_action(self) -> PySide6.QtGui.QAction:
		"""Install the window-level Ferrum New action."""
		action = PySide6.QtGui.QAction(self.tr("New"), self)
		action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.New)
		action.setToolTip(self.tr("Create a new empty Rust-owned Ferrum document"))
		action.triggered.connect(self._on_new)
		self._file_menu.insertAction(self._open_action, action)
		return action

	#============================================
	def _install_native_recent_files_menu(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Create the File-owned personal cascade before the Ferrum save actions."""
		self._recent_files_menu = self._native_recent_files.install_file_menu(menu)

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
	def _add_native_action_toolbar(
			self,
			) -> ferrum_qt.ferrum.action_toolbar.FerrumNativeActionToolbar:
		"""Expose frequent commands through one responsive shared-action client."""
		standard = PySide6.QtWidgets.QStyle.StandardPixmap
		self._open_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Open)
		self._save_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Save)
		self._undo_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Undo)
		self._redo_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Redo)
		cut_icon = PySide6.QtGui.QIcon.fromTheme("edit-cut")
		if not cut_icon.isNull():
			self._cut_action.setIcon(cut_icon)
		groups = (
			(
				(self._action_new, standard.SP_FileIcon),
				(self._open_action, standard.SP_DialogOpenButton),
				(self._save_action, standard.SP_DialogSaveButton),
			),
			(
				(self._undo_action, standard.SP_ArrowBack),
				(self._redo_action, standard.SP_ArrowForward),
			),
			(
				(self._cut_action, standard.SP_TrashIcon),
				(self._copy_action, standard.SP_FileDialogDetailedView),
				(self._paste_action, standard.SP_FileDialogListView),
			),
			(
				(self._zoom_out_action, standard.SP_ArrowDown),
				(self._zoom_100_action, standard.SP_BrowserReload),
				(self._zoom_in_action, standard.SP_ArrowUp),
			),
			(
				(self._show_hex_grid_action, standard.SP_FileDialogContentsView),
				(self._snap_hex_grid_action, standard.SP_DialogApplyButton),
			),
		)
		return (
			ferrum_qt.ferrum.action_toolbar.
			install_native_action_toolbar(self, groups)
		)

	#============================================
	def _add_native_editing_tools_toolbar(
			self,
			) -> ferrum_qt.ferrum.editing_tools_toolbar.FerrumNativeEditingToolsToolbar:
		"""Expose finished canvas tools through their existing shared actions."""
		tools = (
			(self._add_atom_action, "atom"),
			(self._draw_bond_action, "single"),
			(self._insert_cyclohexane_ring_action, "benzene"),
			(self._insert_haworth_ring_action, "ring"),
			(self._draw_wavy_action, "wavyline"),
			(self._draw_bracket_action, "rectangularbracket"),
			(self._draw_round_bracket_action, "roundbracket"),
			(self._move_atom_action, "edit"),
			(self._rotate_atoms_action, "rotate"),
			(self._translate_roots_action, "bondalign"),
		)
		return (
			ferrum_qt.ferrum.editing_tools_toolbar.
			install_native_editing_tools_toolbar(
				self, tools, self._cancel_tool_action, self._theme_manager,
				self._drawing_parameters, self._next_drawing_action,
			)
		)

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
		self._edit_menu.addAction(action)
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
		menu = self.menuBar().addMenu(self.tr("Options"))
		return ferrum_qt.ferrum.preferences \
			.install_native_preferences_action(self, menu)

	#============================================
	def _create_empty_native_tab(
			self,
			) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		"""Create a revision-zero Rust document."""
		import ferrum_qt.ferrum.engine as engine
		session = engine.DocumentSession.create_empty_document_v1()
		return ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab.from_session(
			session, self.tr("Untitled"),
		)

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
		return super()._register_native_tab(tab, activate=activate)

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
	def _close_native_tab_at(self, index: int) -> bool:
		"""Close one clean Ferrum page and report whether it was disposed."""
		tab = self._native_tabs_by_page.get(self._tab_widget.widget(index))
		if tab is None or tab.requires_refresh or tab.is_dirty:
			return False
		self._close_tab_at(index)
		return tab not in self._native_tabs_by_page

	#============================================
	def _refresh_actions(self, *_unused: object) -> None:
		"""Keep an incomplete detached tab from enabling Ferrum edit commands.

		A tab is registered before every optional projection capability has been
		installed.  The ordinary host treats that state as non-editable instead of
		letting an action refresh cross a missing projection attribute.
		"""
		tab = self._active_native_tab()
		controller = getattr(tab, "_controller", None)
		if tab is not None and not hasattr(controller, "projection"):
			for action in self.findChildren(PySide6.QtGui.QAction):
				action.setEnabled(False)
			return
		super()._refresh_actions(*_unused)
		ferrum_qt.ferrum.window_shared_seams.refresh_shared_window_seams(self)

	#============================================
	def prepare_application_shutdown(self) -> bool:
		"""Retire clean Ferrum pages before the generic QObject finalizer runs."""
		if self._shutdown_prepared:
			return True
		if self._cancel_local_cdml_open_for_close():
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

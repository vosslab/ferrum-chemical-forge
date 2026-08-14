"""Migration-only OASA-backed application window for Ferrum-Qt."""

# Standard Library
import pathlib

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
import ferrum_qt.window_clipboard
import ferrum_qt.window_files
import ferrum_qt.window_native_files
import ferrum_qt.window_native_tabs
import ferrum_qt.window_properties
import ferrum_qt.window_session_active
import ferrum_qt.window_session_lifecycle
import ferrum_qt.window_session_setup
import ferrum_qt.window_sessions
import ferrum_qt.window_shared
import ferrum_qt.window_templates
import ferrum_qt.window_view
import ferrum_qt.native.ferrum_native_atom_element
import ferrum_qt.native.ferrum_native_atom_number
import ferrum_qt.native.ferrum_native_atom_properties
import ferrum_qt.native.ferrum_native_bond_properties
import ferrum_qt.dialogs.atom_dialog

ShutdownState = ferrum_qt.window_shared.ShutdownState


#============================================
class LegacyCompatibilityMainWindow(
		ferrum_qt.window_templates.WindowTemplateMixin,
		ferrum_qt.window_properties.WindowPropertiesMixin,
		ferrum_qt.window_session_setup.WindowSessionSetupMixin,
		ferrum_qt.window_native_files.WindowNativeFileMixin,
		ferrum_qt.window_native_tabs.WindowNativeTabsMixin,
		ferrum_qt.window_session_active.WindowSessionActiveMixin,
		ferrum_qt.window_session_lifecycle.WindowSessionLifecycleMixin,
		ferrum_qt.window_clipboard.WindowClipboardMixin,
		ferrum_qt.window_files.WindowFileMixin,
		ferrum_qt.window_view.WindowViewMixin,
		PySide6.QtWidgets.QMainWindow,
		):
	"""Thin QMainWindow composition facade for Ferrum-Qt controllers."""

	_startup_policy = "legacy"
	_native_cdml_default_open_enabled = False

	worker_retirement_drained = PySide6.QtCore.Signal()

	def __init__(self, theme_manager: object,
			parent: PySide6.QtWidgets.QWidget | None = None, *,
			user_template_directory: str | pathlib.Path | None = None) -> None:
		"""Initialize the main window with all UI components.

		Args:
			theme_manager: ThemeManager instance for theme toggling.
			parent: Optional parent widget.
			user_template_directory: Explicit frontend-owned directory used for
				discovering saved user templates, or None for an empty embedded catalog.
		"""
		super().__init__(parent)
		self._theme_manager = theme_manager
		self._prefs = ferrum_qt.config.preferences.Preferences.instance()
		ferrum_qt.actions.options_actions.apply_saved_logging_level(self._prefs)
		self._shutdown_prepared = False
		self._shutdown_state = ShutdownState.LIVE
		self._ui_signals_connected = False
		# Keep active-document bindings explicit so a projection preparation
		# failure can detach and restore the same document without asking Qt to
		# disconnect callbacks that were never installed.
		self._document_signal_source = None
		# Qt requires this exact callable object to disconnect every document
		# signal that refreshes the dock summary.
		self._property_dock_summary_refresh = None
		self._tab_change_blocked = False
		self._sessions = []
		self._sessions_by_view = {}
		self._native_tabs_by_page = {}
		self._native_tab_close_guard = None
		self._native_action_enabled_state = None
		self._native_widget_enabled_state = {}
		# MainWindow alone owns every session-to-tab title subscription.  Session
		# retirement can follow close, replacement rollback, or full-window
		# shutdown, so all of those paths ask this registry to retire a binding
		# exactly once instead of independently guessing whether Qt still has it.
		self._session_title_connections = {}
		self._pending_session_deletions = {}
		# A destroyed-session callback retries retained terminal graphics once.
		# A transient native failure gets one further ordinary event-loop retry;
		# persistent failures remain explicitly retained for shutdown diagnostics
		# instead of scheduling an unbounded zero-delay loop.
		self._pending_session_graphics_retry_scheduled = False
		self._retired_import_workers = set()
		self._shutdown_sessions_pending_disposal = []
		self._active_session = None
		self._user_template_directory = (
			pathlib.Path(user_template_directory)
			if user_template_directory is not None else None
		)
		self._user_template_catalog = self._scan_user_template_catalog()
		self._clipboard_manager = ferrum_qt.io.clipboard_manager.ClipboardManager()
		self._clipboard = PySide6.QtWidgets.QApplication.clipboard()

		self.setWindowTitle(self.tr("Ferrum-Qt"))
		style = PySide6.QtWidgets.QApplication.style()
		window_icon = style.standardIcon(
			PySide6.QtWidgets.QStyle.StandardPixmap.SP_FileIcon
		)
		if not window_icon.isNull():
			app = PySide6.QtWidgets.QApplication.instance()
			if app is not None:
				app.setWindowIcon(window_icon)
			self.setWindowIcon(window_icon)
		self.resize(1280, 800)

		self._neutral_native_shell = False
		self._setup_canvas()
		self._setup_mode_system()
		self._setup_menus()
		self._setup_toolbars()
		self._setup_status_bar()
		self._connect_signals()
		self._apply_geometry_preferences()
		self._apply_view_preferences()
		self._show_user_template_catalog_status(self._user_template_catalog)

	#============================================
	def _clear_active_session_aliases(self) -> None:
		"""Keep legacy compatibility names explicitly empty in native-first mode."""
		self._active_session = None
		self._document = None
		self._scene = None
		self._view = None
		self._mode_manager = None

	#============================================
	def _bind_property_dock(self, session: object | None) -> None:
		"""Keep the neutral dock detached and delegate complete legacy binding."""
		if self._neutral_native_shell:
			del session
			return
		super()._bind_property_dock(session)

	#============================================
	def _setup_neutral_native_shell(self) -> None:
		"""Build only host, status, and native-safe menu primitives."""
		from ferrum_qt.actions.platform_menu import PlatformMenuAdapter
		from ferrum_qt.widgets.status_bar import StatusBar
		self._tab_widget = PySide6.QtWidgets.QTabWidget(self)
		self._tab_widget.setTabsClosable(True)
		self._tab_widget.setMovable(False)
		self.setCentralWidget(self._tab_widget)
		self._clear_active_session_aliases()
		self._status_bar = StatusBar(self)
		self.setStatusBar(self._status_bar)
		self._adapter = PlatformMenuAdapter(self)
		for name in ("File", "Edit", "Options", "Help"):
			self._adapter.add_menu(name, "")
		self._action_new = self._adapter.add_direct_action("File", self.tr("New"), "file.new")
		self._action_new.triggered.connect(self._on_new)
		self._action_open = self._adapter.add_direct_action("File", self.tr("Open"), "file.load")
		self._action_open.triggered.connect(self._on_open)
		self._action_open.setEnabled(False)
		self._action_open_same_tab = self._adapter.add_direct_action(
			"File", self.tr("Open in Current Tab"), "file.load_same_tab",
		)
		self._action_open_same_tab.triggered.connect(self._on_open_same_tab)
		self._action_open_same_tab.setEnabled(False)
		self._action_save = self._adapter.add_direct_action("File", self.tr("Save"), "file.save")
		self._action_save.triggered.connect(self._on_save)
		self._action_save_as = self._adapter.add_direct_action(
			"File", self.tr("Save As..."), "file.save_as",
		)
		self._action_save_as.triggered.connect(self._on_save_as)
		self._action_exit = self._adapter.add_direct_action("File", self.tr("Quit"), "file.exit")
		self._action_exit.triggered.connect(self.close)
		self._action_toggle_theme = self._adapter.add_direct_action(
			"Options", self.tr("Theme"), "options.theme",
		)
		self._action_toggle_theme.setEnabled(False)
		self._action_about = self._adapter.add_direct_action("Help", self.tr("About"), "help.about")
		self._action_about.triggered.connect(self._on_about)
		self._install_explicit_native_actions()
		self._action_open_native_cdml.setEnabled(False)
		self._tab_widget.currentChanged.connect(self._on_tab_changed)
		self._tab_widget.tabCloseRequested.connect(self._on_tab_close_requested)
		self._host_tab_signals_connected = True
		self._refresh_neutral_action_policy()

	#============================================
	def _refresh_neutral_action_policy(self) -> None:
		"""Keep neutral-shell commands honest about the selected page owner."""
		native_tab = self._active_native_tab()
		self._action_new.setEnabled(not self._shutdown_prepared)
		self._action_open.setEnabled(False)
		self._action_open_same_tab.setEnabled(False)
		self._action_save.setEnabled(native_tab is not None)
		self._action_save_as.setEnabled(native_tab is not None)

	#============================================
	def _ensure_legacy_ui_and_session_capability(self) -> bool:
		"""Reserve the future transactional compatibility route without side effects."""
		return False

	#============================================
	def _on_about(self) -> None:
		"""Show the ordinary application About dialog from the neutral shell."""
		ferrum_qt.dialogs.about_dialog.AboutDialog.show_about(self)

	#============================================
	def prepare_application_shutdown(self) -> bool:
		"""Dispose native pages safely when no legacy session graph exists."""
		if not self._neutral_native_shell:
			return super().prepare_application_shutdown()
		if self._shutdown_prepared:
			return True
		if not self._confirm_native_tabs_for_shutdown():
			return False
		self._shutdown_prepared = True
		self._shutdown_state = ShutdownState.DRAINING
		self._dispose_native_tabs_for_shutdown()
		self._clear_active_session_aliases()
		self._shutdown_state = ShutdownState.READY
		return True

	#============================================
	def closeEvent(self, event: PySide6.QtGui.QCloseEvent) -> None:
		"""Close the neutral host only after its native tabs accept disposal."""
		if not self.prepare_application_shutdown():
			event.ignore()
			return
		super().closeEvent(event)

	#============================================
	def _create_empty_native_tab(
			self,
			) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
		"""Create one detached revision-zero Rust document and its Qt projection."""
		import ferrum_chem
		session = ferrum_chem.DocumentSession.create_empty_document_v1()
		return ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab.from_session(
			session, self.tr("Untitled"),
		)

	#============================================
	def _on_new(self) -> bool:
		"""Open the root-owned document type without cross-root setup."""
		if not self._neutral_native_shell:
			return super()._on_new()
		if self._shutdown_prepared:
			return False
		try:
			tab = self._create_empty_native_tab()
		except Exception as exc:
			self.statusBar().showMessage(
				self.tr("Could not create a new Ferrum document: %s") % exc, 5000,
			)
			return False
		try:
			self._register_native_tab(tab, activate=True)
		except Exception as exc:
			tab.dispose()
			self.statusBar().showMessage(
				self.tr("Could not display a new Ferrum document: %s") % exc, 5000,
			)
			return False
		self._refresh_neutral_action_policy()
		return True

	#============================================
	def _on_open(self) -> bool:
		"""Refuse deferred ordinary Open while the selected page is native."""
		if not self._neutral_native_shell:
			return super()._on_open()
		self.statusBar().showMessage(
			self.tr("Open is deferred until Ferrum-Qt owns an external-input policy."),
			5000,
		)
		return False

	#============================================
	def _on_open_same_tab(self) -> bool:
		"""Refuse deferred replacement before any file admission or legacy startup."""
		if not self._neutral_native_shell:
			return super()._on_open_same_tab()
		self.statusBar().showMessage(
			self.tr("Open in Current Tab is not available for Ferrum-native documents."),
			5000,
		)
		return False

	#============================================
	def open_file_path(self, file_path: str, replace_current: bool = False) -> bool:
		"""Keep first-slice programmatic Open on the same explicit unavailable route."""
		if not self._neutral_native_shell:
			return super().open_file_path(file_path, replace_current)
		del file_path, replace_current
		return self._on_open_same_tab()

	#============================================
	def open_native_cdml_path(self, file_path: str) -> bool:
		"""Refuse unbudgeted external CDML until its admission policy is approved."""
		if not self._neutral_native_shell:
			return super().open_native_cdml_path(file_path)
		del file_path
		self.statusBar().showMessage(
			self.tr("External CDML Open is deferred until its admission policy is approved."),
			5000,
		)
		return False

	#============================================
	def _install_explicit_native_actions(self) -> None:
		"""Add opt-in Rust routes without changing ordinary file-open behavior."""
		self._action_open_native_cdml = self._adapter.add_direct_action(
			"File", self.tr("Open CDML with Ferrum..."), "file.open_native_cdml",
		)
		self._action_open_native_cdml.triggered.connect(self._on_open_native_cdml)
		self._action_undo_native = self._adapter.add_direct_action(
			"Edit", self.tr("Undo with Ferrum"), "edit.undo_native",
		)
		self._action_undo_native.triggered.connect(self._on_undo_with_ferrum)
		self._action_undo_native.setEnabled(False)
		self._action_redo_native = self._adapter.add_direct_action(
			"Edit", self.tr("Redo with Ferrum"), "edit.redo_native",
		)
		self._action_redo_native.triggered.connect(self._on_redo_with_ferrum)
		self._action_redo_native.setEnabled(False)
		self._action_change_element_native = self._adapter.add_direct_action(
			"Edit", self.tr("Change Element with Ferrum"), "edit.change_element_native",
		)
		self._action_change_element_native.triggered.connect(
			self._on_change_element_with_ferrum,
		)
		self._action_change_element_native.setEnabled(False)
		self._action_atom_properties_native = self._adapter.add_direct_action(
			"Edit", self.tr("Edit Atom Properties with Ferrum"),
			"edit.atom_properties_native",
		)
		self._action_atom_properties_native.setToolTip(self.tr(
			"Edit one selected durable atom through one Rust-native operation.",
		))
		self._action_atom_properties_native.triggered.connect(
			self._on_edit_atom_properties_with_ferrum,
		)
		self._action_atom_properties_native.setEnabled(False)
		self._action_atom_number_native = self._adapter.add_direct_action(
			"Edit", self.tr("Set Atom Number with Ferrum..."),
			"edit.atom_number_native",
		)
		self._action_atom_number_native.setToolTip(self.tr(
			"Set one selected durable atom number through one Rust-native operation.",
		))
		self._action_atom_number_native.triggered.connect(
			self._on_set_atom_number_with_ferrum,
		)
		self._action_atom_number_native.setEnabled(False)
		self._action_clear_atom_number_native = self._adapter.add_direct_action(
			"Edit", self.tr("Clear Atom Number with Ferrum"),
			"edit.clear_atom_number_native",
		)
		self._action_clear_atom_number_native.setToolTip(self.tr(
			"Clear one selected durable atom number through one Rust-native operation.",
		))
		self._action_clear_atom_number_native.triggered.connect(
			self._on_clear_atom_number_with_ferrum,
		)
		self._action_clear_atom_number_native.setEnabled(False)
		self._action_delete_atom_native = self._adapter.add_direct_action(
			"Edit", self.tr("Delete Selected Atom with Ferrum"),
			"edit.delete_atom_native",
		)
		self._action_delete_atom_native.setToolTip(self.tr(
			"Delete one selected durable atom and its incident bonds through Rust.",
		))
		self._action_delete_atom_native.triggered.connect(
			self._on_delete_selected_atom_with_ferrum,
		)
		self._action_delete_atom_native.setEnabled(False)
		self._action_bond_properties_native = self._adapter.add_direct_action(
			"Edit", self.tr("Edit Bond Properties with Ferrum"),
			"edit.bond_properties_native",
		)
		self._action_bond_properties_native.setToolTip(self.tr(
			"Edit one selected durable bond through one Rust-native operation.",
		))
		self._action_bond_properties_native.triggered.connect(
			self._on_edit_bond_properties_with_ferrum,
		)
		self._action_bond_properties_native.setEnabled(False)
		self._action_delete_bond_native = self._adapter.add_direct_action(
			"Edit", self.tr("Delete Selected Bond with Ferrum"),
			"edit.delete_bond_native",
		)
		self._action_delete_bond_native.setToolTip(self.tr(
			"Delete one selected durable bond through Rust.",
		))
		self._action_delete_bond_native.triggered.connect(
			self._on_delete_selected_bond_with_ferrum,
		)
		self._action_delete_bond_native.setEnabled(False)

	#============================================
	def _on_open_native_cdml(self) -> bool:
		"""Choose one CDML path for the explicit Rust-native tab route."""
		path = PySide6.QtWidgets.QFileDialog.getOpenFileName(
			self, self.tr("Open CDML with Ferrum"), "", self.tr("Ferrum CDML (*.cdml)"),
		)[0]
		if not path:
			return False
		return self.open_native_cdml_path(path)

	#============================================
	def _on_change_element_with_ferrum(self) -> None:
		"""Submit one selected native atom element through the shared Rust dialog."""
		ferrum_qt.native.ferrum_native_atom_element.run_change_selected_atom_element_dialog(
			self,
		)
		self._refresh_explicit_native_actions(self._active_native_tab())

	#============================================
	def _on_undo_with_ferrum(self) -> None:
		"""Ask the active Rust-native tab to restore its prior document revision."""
		self._run_native_history_action("undo")

	#============================================
	def _on_redo_with_ferrum(self) -> None:
		"""Ask the active Rust-native tab to restore its next document revision."""
		self._run_native_history_action("redo")

	#============================================
	def _run_native_history_action(self, operation: str) -> None:
		"""Run one closed Rust history operation without a legacy-session fallback."""
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			self._refresh_explicit_native_actions(tab)
			return
		try:
			if operation == "undo":
				tab.undo()
			elif operation == "redo":
				tab.redo()
			else:
				raise ValueError("native history operation is not supported")
		except Exception as exc:
			self._refresh_explicit_native_actions(tab)
			self._show_native_file_warning("Native History Unavailable", str(exc))
			return
		self._refresh_explicit_native_actions(self._active_native_tab())

	#============================================
	def _on_edit_atom_properties_with_ferrum(self) -> None:
		"""Apply one accepted AtomDialog patch only through the native tab."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			atom = tab.selected_atom_projection()
			model = (
				ferrum_qt.native.ferrum_native_atom_properties.
				dialog_model_from_projection(atom)
			)
		except Exception as exc:
			self._refresh_explicit_native_actions(tab)
			self._show_native_file_warning("Native Atom Properties Unavailable", str(exc))
			return
		dialog = ferrum_qt.dialogs.atom_dialog.AtomDialog(model, self)
		if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return
		try:
			changes = (
				ferrum_qt.native.ferrum_native_atom_properties.
				property_changes_from_dialog(dialog.changes())
			)
			tab.apply_selected_atom_properties(changes)
		except Exception as exc:
			self._refresh_explicit_native_actions(tab)
			self._show_native_file_warning("Native Atom Properties Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Updated one Rust-native atom."), 5000)
		self._refresh_explicit_native_actions(self._active_native_tab())

	#============================================
	def _on_set_atom_number_with_ferrum(self) -> None:
		"""Assign one selected atom number through the active Rust-native tab."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			atom = tab.selected_atom_projection()
			dialog = ferrum_qt.native.ferrum_native_atom_number.FerrumNativeAtomNumberDialog(
				atom.number, atom.show_number, self,
			)
		except Exception as exc:
			self._refresh_explicit_native_actions(tab)
			self._show_native_file_warning("Native Atom Number Unavailable", str(exc))
			return
		if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return
		try:
			number, show_number = dialog.assignment()
			tab.set_selected_atom_number(number, show_number)
		except Exception as exc:
			self._refresh_explicit_native_actions(tab)
			self._show_native_file_warning("Native Atom Number Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Updated one Rust-native atom number."), 5000)
		self._refresh_explicit_native_actions(self._active_native_tab())

	#============================================
	def _on_clear_atom_number_with_ferrum(self) -> None:
		"""Clear one selected atom number through the active Rust-native tab."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			tab.clear_selected_atom_number()
		except Exception as exc:
			self._refresh_explicit_native_actions(tab)
			self._show_native_file_warning("Native Atom Number Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Cleared one Rust-native atom number."), 5000)
		self._refresh_explicit_native_actions(self._active_native_tab())

	#============================================
	def _on_delete_selected_atom_with_ferrum(self) -> None:
		"""Delete one selected atom only through the active Rust-native tab."""
		tab = self._active_native_tab()
		if tab is None or not tab.has_one_selected_atom():
			self._refresh_explicit_native_actions(tab)
			return
		try:
			tab.delete_selected_atom()
		except Exception as exc:
			self._refresh_explicit_native_actions(tab)
			self._show_native_file_warning("Native Delete Atom Error", str(exc))
			return
		self.statusBar().showMessage(
			self.tr("Deleted one Rust-native atom and its incident bonds."), 5000,
		)
		self._refresh_explicit_native_actions(self._active_native_tab())

	#============================================
	def _on_edit_bond_properties_with_ferrum(self) -> None:
		"""Run the existing closed Rust-native bond-properties dialog route."""
		ferrum_qt.native.ferrum_native_bond_properties.run_bond_properties_dialog(self)
		self._refresh_explicit_native_actions(self._active_native_tab())

	#============================================
	def _on_delete_selected_bond_with_ferrum(self) -> None:
		"""Delete one selected bond only through the active Rust-native tab."""
		tab = self._active_native_tab()
		if tab is None or not tab.has_one_selected_bond():
			self._refresh_explicit_native_actions(tab)
			return
		try:
			tab.delete_selected_bond()
		except Exception as exc:
			self._refresh_explicit_native_actions(tab)
			self._show_native_file_warning("Native Delete Bond Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Deleted one Rust-native bond."), 5000)
		self._refresh_explicit_native_actions(self._active_native_tab())

	#============================================
	def _refresh_explicit_native_actions(self, tab: object | None) -> None:
		"""Keep explicit native actions reachable only on their complete route."""
		history_available = tab is not None and not tab.requires_refresh
		self._action_undo_native.setEnabled(history_available)
		self._action_redo_native.setEnabled(history_available)
		self._action_change_element_native.setEnabled(
			ferrum_qt.native.ferrum_native_atom_element.can_change_selected_atom_element(tab),
		)
		self._action_atom_properties_native.setEnabled(
			ferrum_qt.native.ferrum_native_atom_element.can_change_selected_atom_element(tab),
		)
		self._action_atom_number_native.setEnabled(
			ferrum_qt.native.ferrum_native_atom_element.can_change_selected_atom_element(tab),
		)
		self._action_clear_atom_number_native.setEnabled(
			ferrum_qt.native.ferrum_native_atom_number.can_clear_selected_atom_number(tab),
		)
		self._action_delete_atom_native.setEnabled(
			ferrum_qt.native.ferrum_native_atom_element.can_change_selected_atom_element(tab),
		)
		self._action_bond_properties_native.setEnabled(
			ferrum_qt.native.ferrum_native_bond_properties.
			can_edit_selected_bond_properties(tab),
		)
		self._action_delete_bond_native.setEnabled(
			ferrum_qt.native.ferrum_native_bond_properties.
			can_edit_selected_bond_properties(tab),
		)


#============================================
def drain_pending_session_deletions(
		app: PySide6.QtWidgets.QApplication,
		target_window: object = None,
		max_passes: int = 4,
		) -> bool:
	"""Prove one live window's QObject reaper has released every record."""
	if target_window is None:
		raise ValueError("A MainWindow is required to prove reaper completion")
	while target_window._retired_import_workers:
		loop = PySide6.QtCore.QEventLoop()
		target_window.worker_retirement_drained.connect(loop.quit)
		if target_window._retired_import_workers:
			loop.exec()
		try:
			target_window.worker_retirement_drained.disconnect(loop.quit)
		except (RuntimeError, TypeError):
			pass
	for _pass in range(max_passes):
		target_window._resolve_pending_session_graphics()
		PySide6.QtCore.QCoreApplication.sendPostedEvents(
			None, PySide6.QtCore.QEvent.Type.DeferredDelete,
		)
		app.processEvents()
		if not target_window._pending_session_deletions:
			return True
	return False


#============================================
def delete_qobject_and_wait(
		app: PySide6.QtWidgets.QApplication,
		target: PySide6.QtCore.QObject,
		max_passes: int = 4,
		) -> bool:
	"""Queue one QObject deletion and prove its destroyed signal was delivered."""
	if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(target):
		raise RuntimeError("Cannot retire an already-retired QObject")
	destroyed = []

	#============================================
	def record_destroyed(*_args: object) -> None:
		"""Record either PySide6 destroyed-signal signature."""
		destroyed.append(True)

	target.destroyed.connect(record_destroyed)
	target.deleteLater()
	for _pass in range(max_passes):
		PySide6.QtCore.QCoreApplication.sendPostedEvents(
			None, PySide6.QtCore.QEvent.Type.DeferredDelete,
		)
		app.processEvents()
		if destroyed:
			return True
	return False

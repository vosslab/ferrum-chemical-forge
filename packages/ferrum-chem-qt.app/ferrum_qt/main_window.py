"""Main application window for Ferrum-Qt."""

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
import ferrum_qt.window_properties
import ferrum_qt.window_session_active
import ferrum_qt.window_session_lifecycle
import ferrum_qt.window_session_setup
import ferrum_qt.window_sessions
import ferrum_qt.window_shared
import ferrum_qt.window_templates
import ferrum_qt.window_view

ShutdownState = ferrum_qt.window_shared.ShutdownState


#============================================
class MainWindow(
		ferrum_qt.window_templates.WindowTemplateMixin,
		ferrum_qt.window_properties.WindowPropertiesMixin,
		ferrum_qt.window_session_setup.WindowSessionSetupMixin,
		ferrum_qt.window_session_active.WindowSessionActiveMixin,
		ferrum_qt.window_session_lifecycle.WindowSessionLifecycleMixin,
		ferrum_qt.window_clipboard.WindowClipboardMixin,
		ferrum_qt.window_files.WindowFileMixin,
		ferrum_qt.window_view.WindowViewMixin,
		PySide6.QtWidgets.QMainWindow,
		):
	"""Thin QMainWindow composition facade for Ferrum-Qt controllers."""

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
		# The active document has four window-owned callbacks.  Keep their
		# ownership as explicit state so a projection preparation failure can
		# detach and restore the same document without asking Qt to disconnect
		# callbacks that were never installed.
		self._document_signal_source = None
		self._tab_change_blocked = False
		self._sessions = []
		self._sessions_by_view = {}
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

		# build the UI components
		self._setup_canvas()
		self._setup_mode_system()
		self._setup_menus()
		self._setup_toolbars()
		self._setup_status_bar()
		self._connect_signals()
		self._apply_geometry_preferences()
		self._apply_view_preferences()
		self._show_user_template_catalog_status(self._user_template_catalog)

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
import ferrum_qt.window_session_active
import ferrum_qt.window_session_lifecycle
import ferrum_qt.window_session_setup

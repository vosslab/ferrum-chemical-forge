"""Ordinary OASA-free native-first application window for Ferrum-Qt."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.config.preferences
import ferrum_qt.dialogs.theme_chooser_dialog
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_main_window


#============================================
class MainWindow(ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow):
	"""Start the product host with one empty Rust-owned document.

	The historical OASA session graph lives only in
	``ferrum_qt.legacy.compatibility_main_window``. External uncompressed CDML
	uses the same Rust-owned local V1 profile as the native render CLI and never
	loads document bytes through Python or a compatibility fallback.
	"""

	#============================================
	def __init__(
			self, theme_manager: object,
			parent: PySide6.QtWidgets.QWidget | None = None, *,
			user_template_directory: object = None,
			) -> None:
		"""Build the ordinary native-only host and its initial empty document."""
		del user_template_directory
		super().__init__(parent)
		self._theme_manager = theme_manager
		self._prefs = ferrum_qt.config.preferences.Preferences.instance()
		self._shutdown_prepared = False
		self.setWindowTitle(self.tr("Ferrum-Qt"))
		self.resize(1280, 800)
		self._action_open = self._open_action
		self._action_new = self._add_new_document_action()
		self._theme_action = self._add_theme_action()
		self._on_new()

	#============================================
	def _add_new_document_action(self) -> PySide6.QtGui.QAction:
		"""Install a window-level native New action without a legacy menu owner."""
		action = PySide6.QtGui.QAction(self.tr("New"), self)
		action.triggered.connect(self._on_new)
		self.addAction(action)
		return action

	#============================================
	def _add_theme_action(self) -> PySide6.QtGui.QAction:
		"""Expose the retained application-theme choice without legacy view ownership."""
		menu = self.menuBar().addMenu(self.tr("Options"))
		action = PySide6.QtGui.QAction(self.tr("Theme"), self)
		action.triggered.connect(self._on_choose_theme)
		menu.addAction(action)
		return action

	#============================================
	def _on_choose_theme(self) -> None:
		"""Apply one accepted theme choice through the existing application manager."""
		current = self._theme_manager.current_theme
		chosen = ferrum_qt.dialogs.theme_chooser_dialog.ThemeChooserDialog.choose_theme(
			self, current,
		)
		if chosen is not None and chosen != current:
			self._theme_manager.apply_theme(chosen)

	#============================================
	def _create_empty_native_tab(
			self,
			) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
		"""Create a revision-zero Rust document without a legacy session."""
		import ferrum_chem
		session = ferrum_chem.DocumentSession.create_empty_document_v1()
		return ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab.from_session(
			session, self.tr("Untitled"),
		)

	#============================================
	def _on_new(self) -> bool:
		"""Add one empty Rust-native document tab."""
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
			tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			*, activate: bool = True,
			) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
		"""Keep the common host's activation default for native callers."""
		return super()._register_native_tab(tab, activate=activate)

	#============================================
	def _close_native_tab_at(self, index: int) -> bool:
		"""Close one clean native page and report whether it was disposed."""
		tab = self._native_tabs_by_page.get(self._tab_widget.widget(index))
		if tab is None or tab.requires_refresh or tab.is_dirty:
			return False
		self._close_tab_at(index)
		return tab not in self._native_tabs_by_page

	#============================================
	def _refresh_actions(self, *_unused: object) -> None:
		"""Keep an incomplete detached tab from enabling native edit commands.

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

	#============================================
	def prepare_application_shutdown(self) -> bool:
		"""Retire clean native pages before the generic QObject finalizer runs."""
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
			self._prefs.set_value(
				ferrum_qt.config.preferences.Preferences.KEY_WINDOW_GEOMETRY,
				self.saveGeometry(),
			)
		return self._shutdown_prepared

	#============================================
	def restore_geometry(self) -> None:
		"""Restore ordinary-window geometry without importing a legacy view mixin."""
		geometry = self._prefs.value(
			ferrum_qt.config.preferences.Preferences.KEY_WINDOW_GEOMETRY,
		)
		if geometry is not None:
			self.restoreGeometry(geometry)

	#============================================
	def closeEvent(self, event: PySide6.QtGui.QCloseEvent) -> None:
		"""Use the ordinary native-only shutdown policy for a window close."""
		if not self.prepare_application_shutdown():
			event.ignore()
			return
		event.accept()

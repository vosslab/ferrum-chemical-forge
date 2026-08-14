"""Public behavior coverage for ordinary-window native-first startup."""

# PIP3 modules
import pathlib

import ferrum_chem
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.config.preferences
import ferrum_qt.dialogs.theme_chooser_dialog
import ferrum_qt.main_window
import ferrum_qt.native.ferrum_native_document_tab


#============================================
def _make_window(qapp: PySide6.QtWidgets.QApplication) -> ferrum_qt.main_window.MainWindow:
	"""Create the ordinary neutral host without selecting legacy compatibility."""
	del qapp
	return ferrum_qt.main_window.MainWindow(object())


#============================================
class _ThemeManager:
	"""Record one accepted ordinary-window theme change."""

	def __init__(self) -> None:
		"""Start from the current dark theme without an applied replacement."""
		self.current_theme = "dark"
		self.applied: list[str] = []

	def apply_theme(self, theme: str) -> None:
		"""Record and adopt the selected theme like the real manager."""
		self.applied.append(theme)
		self.current_theme = theme


#============================================
class _PreferencesRecorder:
	"""Capture accepted-shutdown persistence without touching user settings."""

	def __init__(self) -> None:
		"""Start without a saved window geometry."""
		self.values: dict[str, object] = {}

	def set_value(self, key: str, value: object) -> None:
		"""Record one application preference write."""
		self.values[key] = value


#============================================
def _theme_action(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtGui.QAction:
	"""Find the ordinary Options action through the public Qt object tree."""
	return next(
		action
		for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == "Theme"
	)


#============================================
def test_ordinary_startup_creates_a_native_empty_document(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Startup selects a Rust-owned page and exposes bounded Rust CDML Open."""
	window = _make_window(qapp)
	try:
		assert isinstance(
			window._active_native_tab(),
			ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
		)
		assert window._action_open.isEnabled()
	finally:
		window.close()


#============================================
def test_new_document_can_save_and_reopen_through_rust(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A native New page owns the empty baseline through semantic publication."""
	window = _make_window(qapp)
	try:
		tab = window._active_native_tab()
		path = tmp_path.resolve() / "new-document.cdml"
		publication = tab.save_atomic(path)
		reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
		assert publication.outcome.is_confirmed
		assert reopened.snapshot().revision == 0
	finally:
		window.close()


#============================================
def test_closing_the_last_native_page_leaves_a_safe_neutral_host(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Close permits the documented zero-page state without reviving legacy state."""
	window = _make_window(qapp)
	try:
		index = window._tab_widget.currentIndex()
		assert window._close_native_tab_at(index)
		assert window._active_native_tab() is None
	finally:
		window.close()


#============================================
def test_ordinary_theme_action_applies_only_the_accepted_theme(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The public Options action changes UI theme without touching Rust state."""
	del qapp
	theme_manager = _ThemeManager()
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		before = window._active_native_tab().current_snapshot
		def choose_light(_parent: object, current: str) -> str:
			"""Accept exactly one different available theme."""
			assert current == "dark"
			return "light"
		monkeypatch.setattr(
			ferrum_qt.dialogs.theme_chooser_dialog.ThemeChooserDialog,
			"choose_theme", choose_light,
		)
		_theme_action(window).trigger()
		assert theme_manager.applied == ["light"]
		assert window._active_native_tab().current_snapshot == before
	finally:
		window.close()


#============================================
def test_cancelled_ordinary_theme_choice_is_a_no_op(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Cancelling the public Theme action retains the current application theme."""
	del qapp
	theme_manager = _ThemeManager()
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		def cancel_theme(_parent: object, _current: str) -> None:
			"""Return the retained chooser's cancellation result."""
			return None
		monkeypatch.setattr(
			ferrum_qt.dialogs.theme_chooser_dialog.ThemeChooserDialog,
			"choose_theme", cancel_theme,
		)
		_theme_action(window).trigger()
		assert not theme_manager.applied
		assert theme_manager.current_theme == "dark"
	finally:
		window.close()


#============================================
def test_accepted_native_shutdown_persists_geometry_for_restore(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A completed ordinary close writes exactly the geometry restored at startup."""
	window = _make_window(qapp)
	preferences = _PreferencesRecorder()
	window._prefs = preferences
	expected = bytes(window.saveGeometry())
	try:
		assert window.prepare_application_shutdown()
		stored = preferences.values[
			ferrum_qt.config.preferences.Preferences.KEY_WINDOW_GEOMETRY
		]
		assert bytes(stored) == expected
	finally:
		window.close()

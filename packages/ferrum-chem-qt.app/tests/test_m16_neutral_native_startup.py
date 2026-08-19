"""Public behavior coverage for ordinary-window native-first startup."""

# PIP3 modules
import pathlib

import ferrum_chem
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.config.preferences
import ferrum_qt.main_window
import ferrum_qt.ferrum.action_toolbar
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.preferences
import ferrum_qt.ferrum.property_dock


_PROPERTY_CDML = """<cdml version='26.08'>
<molecule id='mol-1'><atom id='atom-c' name='C'><point x='10' y='20'/></atom>
<atom id='atom-o' name='O'><point x='40' y='20'/></atom>
<bond id='bond-co' start='atom-c' end='atom-o' type='n2'/></molecule></cdml>"""


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

	def value(self, key: str, default: object = None) -> object:
		"""Return one recorded setting or its caller-provided fallback."""
		return self.values.get(key, default)

	def remove_value(self, key: str) -> None:
		"""Remove one recorded application setting."""
		self.values.pop(key, None)


#============================================
def _preferences_action(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtGui.QAction:
	"""Find the ordinary Options action through the public Qt object tree."""
	return next(
		action
		for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == "Preferences..."
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
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		)
		assert window._action_open.isEnabled()
	finally:
		window.close()


#============================================
def test_main_toolbar_creates_a_document_and_remains_user_hideable(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The visible toolbar performs a document command and can leave the workspace."""
	window = _make_window(qapp)
	window.show()
	qapp.processEvents()
	try:
		toolbar = window.findChild(
			ferrum_qt.ferrum.action_toolbar.FerrumNativeActionToolbar,
			"native-main-action-toolbar",
		)
		before = window._tab_widget.count()
		toolbar.widgetForAction(window._action_new).click()
		assert window._tab_widget.count() == before + 1
		toggle = toolbar.toggleViewAction()
		toggle.trigger()
		assert not toolbar.isVisible()
	finally:
		window.close()


#============================================
def test_properties_follow_the_selected_atom_across_document_tabs(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The inspector follows durable facts from the user-selected document tab."""
	window = _make_window(qapp)
	first = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_PROPERTY_CDML, "carbon.cdml",
	)
	second = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_PROPERTY_CDML, "oxygen.cdml",
	)
	try:
		window._register_native_tab(first, activate=True)
		first.select_atom("atom-c")
		window._register_native_tab(second, activate=True)
		second.select_atom("atom-o")
		dock = window.findChild(
			ferrum_qt.ferrum.property_dock.FerrumNativePropertyDock,
			"native-properties-dock",
		)
		assert "Element: O" in dock.summary_text
		window._tab_widget.setCurrentWidget(first)
		assert "Element: C" in dock.summary_text
	finally:
		window.close()


#============================================
def test_new_document_can_save_and_reopen_through_rust(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A Ferrum New page owns the empty baseline through semantic publication."""
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
def test_ordinary_preferences_apply_only_application_owned_state(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Accepted Preferences change UI policy without changing the Rust document."""
	del qapp
	theme_manager = _ThemeManager()
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	preferences = _PreferencesRecorder()
	preferences.values.update({
		ferrum_qt.config.preferences.Preferences.KEY_WINDOW_GEOMETRY: b"old geometry",
		ferrum_qt.config.preferences.Preferences.KEY_WINDOW_STATE: b"old state",
	})
	window._prefs = preferences
	try:
		before = window._active_native_tab().current_snapshot
		monkeypatch.setattr(
			ferrum_qt.ferrum.preferences.FerrumNativePreferencesDialog,
			"choose_preferences",
			lambda _parent, _current: (
				ferrum_qt.ferrum.preferences.FerrumNativePreferencesV1(
					"light", False, False,
				)
			),
		)
		_preferences_action(window).trigger()
		assert (
			theme_manager.applied,
			preferences.values[
				ferrum_qt.config.preferences.Preferences.KEY_REMEMBER_WORKSPACE
			],
			preferences.values[
				ferrum_qt.config.preferences.Preferences.KEY_GRID_VISIBLE
			],
			window._active_native_tab().view.hex_grid_visible,
		) == (["light"], False, False, False)
		assert (
			window._active_native_tab().current_snapshot == before
			and ferrum_qt.config.preferences.Preferences.KEY_WINDOW_GEOMETRY
			not in preferences.values
			and ferrum_qt.config.preferences.Preferences.KEY_WINDOW_STATE
			not in preferences.values
		)
	finally:
		window.close()


#============================================
def test_cancelled_ordinary_preferences_are_a_no_op(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Cancelling Preferences preserves application and document state."""
	del qapp
	theme_manager = _ThemeManager()
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	preferences = _PreferencesRecorder()
	window._prefs = preferences
	try:
		before = window._active_native_tab().current_snapshot
		monkeypatch.setattr(
			ferrum_qt.ferrum.preferences.FerrumNativePreferencesDialog,
			"choose_preferences", lambda _parent, _current: None,
		)
		_preferences_action(window).trigger()
		assert not theme_manager.applied and not preferences.values
		assert window._active_native_tab().current_snapshot == before
	finally:
		window.close()


#============================================
def test_accepted_native_shutdown_restores_the_visible_workspace_choice(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Hidden ordinary workspace clients remain hidden in the next window."""
	window = _make_window(qapp)
	preferences = _PreferencesRecorder()
	window._prefs = preferences
	window.show()
	qapp.processEvents()
	window._native_action_toolbar.hide()
	window._native_property_dock.hide()
	editing_tools = next(
		toolbar
		for toolbar in window.findChildren(PySide6.QtWidgets.QToolBar)
		if toolbar.accessibleName() == "Editing tools toolbar"
	)
	editing_tools.toggleViewAction().trigger()
	restored = None
	try:
		assert window.prepare_application_shutdown()
		restored = _make_window(qapp)
		restored._prefs = preferences
		restored.restore_workspace()
		restored_editing_tools = next(
			toolbar
			for toolbar in restored.findChildren(PySide6.QtWidgets.QToolBar)
			if toolbar.accessibleName() == "Editing tools toolbar"
		)
		assert (
			restored._native_action_toolbar.isHidden()
			and restored._native_property_dock.isHidden()
			and restored_editing_tools.isHidden()
		)
	finally:
		window.close()
		if restored is not None:
			restored.close()

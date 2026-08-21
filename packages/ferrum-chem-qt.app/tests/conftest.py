"""Shared native-only pytest setup for ferrum-qt tests."""

# Standard Library
import collections.abc
import os
import sys

_REPO_TESTS_DIR = os.path.abspath(os.path.join(
	os.path.dirname(__file__), "..", "..", "..", "tests",
))
if _REPO_TESTS_DIR not in sys.path:
	sys.path.insert(0, _REPO_TESTS_DIR)

pytest_plugins = ("pytest_kill_after",)
collect_ignore = ["e2e"]

# Qt reads its platform choice when QApplication is initialized. Set the
# deterministic test policy before importing any PySide6 module.
os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
os.environ.setdefault("QT_LOGGING_RULES", "qt.qpa.*=false")

# PIP3 modules
import pytest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
@pytest.fixture(scope="session")
def qapp() -> collections.abc.Iterator[PySide6.QtWidgets.QApplication]:
	"""Return the shared offscreen QApplication for Ferrum Qt behavior tests."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	yield app
	app.clipboard().clear()


#============================================
@pytest.fixture
def theme_manager(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> collections.abc.Iterator[ferrum_qt.themes.theme_manager.ThemeManager]:
	"""Provide an application theme owner without writing personal preferences."""
	palette = qapp.palette()
	stylesheet = qapp.styleSheet()
	manager = ferrum_qt.themes.theme_manager.ThemeManager(qapp)
	monkeypatch.setattr(manager, "_save_preference", lambda _name: None)
	yield manager
	qapp.setPalette(palette)
	qapp.setStyleSheet(stylesheet)


#============================================
@pytest.fixture
def main_window(
		qapp: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		) -> collections.abc.Iterator[ferrum_qt.main_window.MainWindow]:
	"""Provide one ordinary Ferrum window with deterministic cleanup."""
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	yield window
	while window._tab_widget.count():
		window._close_tab_at(0)
	window.close()
	window.deleteLater()
	qapp.processEvents()

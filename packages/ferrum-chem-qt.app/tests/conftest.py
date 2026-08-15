"""Shared native-only pytest setup for ferrum-qt tests."""

# Standard Library
import os
import sys

_REPO_TESTS_DIR = os.path.abspath(os.path.join(
	os.path.dirname(__file__), "..", "..", "..", "tests",
))
if _REPO_TESTS_DIR not in sys.path:
	sys.path.insert(0, _REPO_TESTS_DIR)

pytest_plugins = ("pytest_kill_after",)

# Qt reads its platform choice when QApplication is initialized. Set the
# deterministic test policy before importing any PySide6 module.
os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
os.environ.setdefault("QT_LOGGING_RULES", "qt.qpa.*=false")

# PIP3 modules
import pytest
import PySide6.QtWidgets


#============================================
@pytest.fixture(scope="session")
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Return the shared offscreen QApplication for native Qt behavior tests."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	yield app
	app.clipboard().clear()

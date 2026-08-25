"""Shared launch boundary for Ferrum's direct Qt E2E scripts."""

# Standard Library
import os


#============================================
def select_offscreen_qt_platform() -> None:
	"""Select the test-owned Qt backend before any PySide6 import."""
	os.environ["QT_QPA_PLATFORM"] = "offscreen"

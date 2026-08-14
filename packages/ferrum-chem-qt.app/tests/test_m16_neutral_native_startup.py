"""Public behavior coverage for ordinary-window native-first startup."""

# PIP3 modules
import pathlib

import ferrum_chem
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.native.ferrum_native_document_tab


#============================================
def _make_window(qapp: PySide6.QtWidgets.QApplication) -> ferrum_qt.main_window.MainWindow:
	"""Create the ordinary neutral host without selecting legacy compatibility."""
	del qapp
	return ferrum_qt.main_window.MainWindow(object())


#============================================
def test_ordinary_startup_creates_a_native_empty_document(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Startup selects a Rust-owned page and leaves ordinary Open unavailable."""
	window = _make_window(qapp)
	try:
		assert isinstance(
			window._active_native_tab(),
			ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
		)
		assert not window._action_open.isEnabled()
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

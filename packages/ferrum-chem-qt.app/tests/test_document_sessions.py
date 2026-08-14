"""Native-wrapper teardown regression coverage for document sessions."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.qt_lifecycle
import ferrum_qt.legacy.compatibility_lifecycle

#============================================
def test_closed_tab_releases_its_detached_native_ownership_graph(
		main_window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A removed tab closes and drains its QObject reaper."""
	assert main_window._on_new()
	closed = main_window.close_session_at(1)
	drained = ferrum_qt.legacy.compatibility_lifecycle.drain_pending_session_deletions(
		qapp, main_window,
	)

	assert closed
	assert drained


#============================================
def test_tab_title_binding_retires_with_its_session(
		main_window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A session title reaches its tab and cleanly retires when the tab closes."""
	assert main_window._on_new()
	session = main_window.sessions[1]
	session.title_changed.emit("Named document")

	assert main_window._tab_widget.tabText(1) == "Named document"
	assert main_window.close_session_at(1)
	assert ferrum_qt.legacy.compatibility_lifecycle.drain_pending_session_deletions(qapp, main_window)

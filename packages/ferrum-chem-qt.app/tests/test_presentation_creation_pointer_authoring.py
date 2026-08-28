"""Visible straight-arrow authoring through Ferrum Qt."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.themes.theme_loader
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_EDITABLE_CDML = """<cdml xmlns='urn:ferrum:cdml'>
  <molecule id='mol-1'><atom id='atom-c' name='C'><point x='10' y='20'/></atom></molecule>
</cdml>"""


def _action(window: PySide6.QtWidgets.QMainWindow, text: str) -> PySide6.QtGui.QAction:
	"""Return one visible action by its user-facing label."""
	return next(action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == text)


def _close_window(window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Discard exact test tabs before ordinary window shutdown can prompt."""
	window.cancel_active_pointer_authoring()
	for tab in tuple(window._native_tabs_by_page.values()):
		index = window._tab_widget.indexOf(tab)
		result = window._close_native_tab_at(
			index, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		)
		assert result is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
	assert not window._native_tabs_by_page
	window.close()
	window.deleteLater()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.DeferredDelete)
	qapp.processEvents()


def _scene_point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one scene coordinate through the viewport seam."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


def test_arrow_drag_commits_a_durable_arrow_and_undo_removes_it(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A user can draw an Arrow, then undo the visible document change."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_EDITABLE_CDML, "arrow.cdml", ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		start = _scene_point(tab, 24.0, 30.0)
		end = _scene_point(tab, 124.0, 30.0)
		_action(window, "Draw Arrow").trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()
		assert "<arrow" in tab.current_snapshot.cdml
		assert window._render_interaction_selection is not None
		_action(window, "Select Structure").trigger()
		qapp.processEvents()
		assert window._render_interaction_selection is None
		assert window._render_interaction_selection_item is None
		_action(window, "Undo").trigger()
		qapp.processEvents()
		assert "<arrow" not in tab.current_snapshot.cdml
	finally:
		_close_window(window, qapp)

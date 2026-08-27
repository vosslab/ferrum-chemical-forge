"""Visible Rectangle authoring through Ferrum Qt."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.themes.theme_loader
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_CDML = "<cdml xmlns='urn:ferrum:cdml'><standard line_color='#123456' line_width='3' area_color='#abcdef'/></cdml>"


def _action(window: PySide6.QtWidgets.QMainWindow, text: str) -> PySide6.QtGui.QAction:
	"""Return one visible action by its user-facing label."""
	return next(action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == text)


def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one scene coordinate through the viewport seam."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


def test_rectangle_drag_commits_the_document_style_and_undo_removes_it(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A user can draw a styled Rectangle and undo the document change."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "vectors.cdml", ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		_action(window, "Draw Rectangle").trigger()
		start = _point(tab, 20.0, 20.0)
		end = _point(tab, 60.0, 50.0)
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()
		assert '<rect' in tab.current_snapshot.cdml and 'line_color="#123456"' in tab.current_snapshot.cdml
		_action(window, "Undo").trigger()
		qapp.processEvents()
		assert "<rect" not in tab.current_snapshot.cdml
	finally:
		window.close()
		window.deleteLater()

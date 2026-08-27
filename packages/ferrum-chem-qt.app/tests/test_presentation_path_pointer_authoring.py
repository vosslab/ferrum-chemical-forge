"""Visible multi-point path authoring through Ferrum Qt."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


def _action(window: PySide6.QtWidgets.QMainWindow, text: str) -> PySide6.QtGui.QAction:
	"""Return one visible action by its user-facing label."""
	return next(action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == text)


def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one scene coordinate through the viewport seam."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


def test_polyline_clicks_commit_on_enter_and_undo_removes_it(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A user can finish a Polyline with Enter and undo the document change."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	try:
		window.show()
		qapp.processEvents()
		_action(window, "New").trigger()
		qapp.processEvents()
		tab = window.centralWidget().currentWidget()
		assert tab is not None
		_action(window, "Draw Polyline").trigger()
		for x, y in ((20.0, 20.0), (60.0, 40.0)):
			PySide6.QtTest.QTest.mouseClick(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, x, y),
			)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return)
		qapp.processEvents()
		assert "<polyline" in tab.current_snapshot.cdml
		_action(window, "Undo").trigger()
		qapp.processEvents()
		assert "<polyline" not in tab.current_snapshot.cdml
	finally:
		window.close()
		window.deleteLater()

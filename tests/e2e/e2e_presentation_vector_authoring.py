"""Offscreen Ferrum workflow: author, move, undo, save, and reopen one vector."""

# Standard Library
import json
import pathlib

# local repo modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()


# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import e2e_workspace
import ferrum_chem
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
def _action(window: PySide6.QtWidgets.QMainWindow, text: str) -> PySide6.QtGui.QAction:
	"""Return one visible action by its user-facing label."""
	return next(action for action in window.findChildren(PySide6.QtGui.QAction) if action.text() == text)


#============================================
def _active_canvas_tab(window: PySide6.QtWidgets.QMainWindow) -> object:
	"""Return the selected document canvas from the public central tabs."""
	tab_widget = window.centralWidget()
	if not isinstance(tab_widget, PySide6.QtWidgets.QTabWidget):
		raise RuntimeError("Ferrum window does not expose public document tabs")
	tab = tab_widget.currentWidget()
	if tab is None:
		raise RuntimeError("public New did not select a Ferrum document tab")
	return tab


#============================================
def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one authored scene coordinate into the live canvas viewport."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


#============================================
def _current_cdml(tab: object) -> str:
	"""Read the durable Rust document through the selected canvas observation."""
	return tab.current_document_observation().snapshot.cdml


#============================================
def main() -> int:
	"""Prove one visible vector action survives the normal document workflow."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		with e2e_workspace.E2EWorkspaceLease() as workspace_text:
			workspace = pathlib.Path(workspace_text)
			window.show()
			app.processEvents()
			_action(window, "New").trigger()
			app.processEvents()
			tab = _active_canvas_tab(window)
			start, end = _point(tab, 24.0, 30.0), _point(tab, 124.0, 64.0)
			_action(window, "Draw Line").trigger()
			PySide6.QtTest.QTest.mousePress(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
			)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
			PySide6.QtTest.QTest.mouseRelease(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
			)
			app.processEvents()
			created_cdml = _current_cdml(tab)
			if "<polyline" not in created_cdml:
				raise RuntimeError("Draw Line did not create one durable Rust vector")
			_action(window, "Move Complete Roots").trigger()
			move_start, move_end = _point(tab, 74.0, 47.0), _point(tab, 94.0, 67.0)
			PySide6.QtTest.QTest.mousePress(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, move_start,
			)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), move_end)
			PySide6.QtTest.QTest.mouseRelease(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, move_end,
			)
			app.processEvents()
			if _current_cdml(tab) == created_cdml:
				raise RuntimeError("Move Complete Roots did not translate the authored vector")
			_action(window, "Undo").trigger()
			app.processEvents()
			if _current_cdml(tab) != created_cdml:
				raise RuntimeError("Undo did not restore the authored vector document")
			path = workspace / "vector.cdml"
			if not window.save_active_to_path(str(path)):
				raise RuntimeError("public Save did not publish the authored vector")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if "<polyline" not in reopened.snapshot().cdml:
				raise RuntimeError("Rust reopen did not preserve the authored vector")
			print(json.dumps({"schema": "ferrum-vector-authoring-e2e-v2", "status": "ok"}))
			return 0
	finally:
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	raise SystemExit(main())

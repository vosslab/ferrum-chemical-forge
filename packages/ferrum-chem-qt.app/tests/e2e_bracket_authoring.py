"""Offscreen Ferrum workflow: author, move, undo, save, and reopen brackets."""

import json
import os
import pathlib
import sys

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

import ferrum_chem
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


_CDML = "<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'><molecule id='m'><atom id='a' name='C'><point x='10' y='20'/></atom></molecule></cdml>"


#============================================
def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one authored scene coordinate through the live Qt viewport."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


#============================================
def _reopen_through_native_file_route(window: object, path: pathlib.Path) -> object:
	"""Await the public asynchronous CDML Open route for one saved bracket file."""
	completed = []
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)
	def receive(completed_path: str, success: bool) -> None:
		"""Record the terminal result for the requested saved file."""
		if pathlib.Path(completed_path) == path:
			completed.append(success)
			loop.quit()
	window.local_cdml_open_completed.connect(receive)
	timeout.timeout.connect(loop.quit)
	try:
		if not window.open_file_path(str(path)):
			raise RuntimeError("Ferrum native Open did not accept the saved bracket file")
		timeout.start(10_000)
		loop.exec()
		if completed != [True]:
			raise RuntimeError("Ferrum native Open did not complete the saved bracket route")
		return window._active_native_tab()
	finally:
		timeout.stop()
		window.local_cdml_open_completed.disconnect(receive)


#============================================
def _drag(viewport: object, start: PySide6.QtCore.QPoint,
		end: PySide6.QtCore.QPoint) -> None:
	"""Perform one normal left-button drag through the active viewport."""
	PySide6.QtTest.QTest.mousePress(
		viewport, PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
	)
	PySide6.QtTest.QTest.mouseMove(viewport, end)
	PySide6.QtTest.QTest.mouseRelease(
		viewport, PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
	)


#============================================
def _activate_pointer_tool(action: PySide6.QtGui.QAction,
		app: PySide6.QtWidgets.QApplication) -> None:
	"""Wait until one ribbon action has installed its live viewport owner."""
	action.trigger()
	app.processEvents()
	if not action.isChecked():
		raise RuntimeError("Ferrum pointer tool did not become visibly active")


#============================================
def main() -> int:
	"""Exercise a bracket pair through the native tool, move, Save As, and Open."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "bracket-e2e.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		app.processEvents()
		start = _point(tab, 72.0, 36.0)
		end = _point(tab, 112.0, 96.0)
		before_cancel = tab.current_snapshot.cdml
		_activate_pointer_tool(window._draw_bracket_action, app)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		app.processEvents()
		if (
			tab.current_snapshot.cdml != before_cancel
			or window._draw_bracket_action.isChecked()
		):
			raise RuntimeError("Escape did not cancel Draw Rectangular Bracket without mutation")
		_activate_pointer_tool(window._draw_bracket_action, app)
		_drag(tab.view.viewport(), start, end)
		app.processEvents()
		created = tab.current_snapshot.cdml
		created_stack = tab._document_observation.projection.presentation_stack
		if len(created_stack.bracket_pairs) != 1 or len(created_stack.roots) != 2:
			raise RuntimeError("Draw Rectangular Bracket did not create one complete durable pair")
		selected_targets = tab._controller.projection.selected_durable_targets()
		if [target.identifier for target in selected_targets] != [
			root.polyline.target.id for root in created_stack.roots
		]:
			raise RuntimeError("Draw Rectangular Bracket did not select both durable pair sides")
		_activate_pointer_tool(window._translate_roots_action, app)
		move_start = _point(tab, 72.0, 66.0)
		move_end = _point(tab, 92.0, 84.0)
		_drag(tab.view.viewport(), move_start, move_end)
		app.processEvents()
		if tab.current_snapshot.cdml == created:
			raise RuntimeError("Move Complete Roots did not translate the bracket pair")
		tab.undo()
		app.processEvents()
		if tab.current_snapshot.cdml != created:
			raise RuntimeError("Undo did not restore the authored bracket pair")
		path = pathlib.Path("/private/tmp") / f"ferrum-bracket-e2e-{os.getpid()}.cdml"
		if not window.save_active_to_path(str(path)):
			raise RuntimeError("Ferrum native Save As did not publish the bracket document")
		if tab.file_path != path or tab.is_dirty:
			raise RuntimeError("Ferrum native Save As did not install a clean tab baseline")
		reopened_session = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
		reopened_stack = reopened_session.observe(0).projection.presentation_stack
		if len(reopened_stack.bracket_pairs) != 1 or len(reopened_stack.roots) != 2:
			raise RuntimeError("Rust reopen did not preserve the complete bracket pair")
		reopened = _reopen_through_native_file_route(window, path)
		if reopened is None or len(reopened._document_observation.projection.presentation_stack.bracket_pairs) != 1:
			raise RuntimeError("Ferrum native Open did not reproject the saved bracket pair")
		print(json.dumps({"schema": "ferrum-bracket-authoring-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	sys.exit(main())

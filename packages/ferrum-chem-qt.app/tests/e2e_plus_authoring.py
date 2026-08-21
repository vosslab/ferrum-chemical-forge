"""Offscreen Ferrum workflow: create, move, undo, save, and reopen one Plus."""

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


def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one backend scene coordinate through the active Qt viewport."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


def _reopen_through_native_file_route(window: object, path: pathlib.Path) -> object:
	"""Await the public asynchronous CDML Open route for one saved Plus file."""
	completed = []
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)
	def receive(completed_path: str, success: bool) -> None:
		if pathlib.Path(completed_path) == path:
			completed.append(success)
			loop.quit()
	window.local_document_open_completed.connect(receive)
	timeout.timeout.connect(loop.quit)
	try:
		if not window.open_file_path(str(path)):
			raise RuntimeError("Ferrum native Open did not accept the saved Plus file")
		timeout.start(10_000)
		loop.exec()
		if completed != [True]:
			raise RuntimeError("Ferrum native Open did not complete the saved Plus route")
		return window._active_native_tab()
	finally:
		timeout.stop()
		window.local_document_open_completed.disconnect(receive)


def main() -> int:
	"""Exercise the durable direct Plus authoring path and emit one receipt."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "plus-e2e.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		app.processEvents()
		created_at = _point(tab, 72.0, 36.0)
		window._draw_plus_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, created_at)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, created_at)
		app.processEvents()
		created = tab.current_snapshot.cdml
		if '<plus' not in created or window._render_interaction_selection is None:
			raise RuntimeError("Draw Plus did not create and select one durable Plus")
		window._translate_roots_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 72.0, 36.0))
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), _point(tab, 92.0, 54.0))
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 92.0, 54.0))
		app.processEvents()
		if tab.current_snapshot.cdml == created:
			raise RuntimeError("Move Complete Roots did not translate the created Plus")
		tab.undo()
		app.processEvents()
		if tab.current_snapshot.cdml != created:
			raise RuntimeError("Undo did not restore the pre-translation Plus document")
		path = pathlib.Path("/private/tmp") / f"ferrum-plus-e2e-{os.getpid()}.cdml"
		if not window.save_active_to_path(str(path)):
			raise RuntimeError("Ferrum native Save As did not publish the Plus document")
		if tab.file_path != path or tab.is_dirty:
			raise RuntimeError("Ferrum native Save As did not install a clean tab baseline")
		if '<plus' not in ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8")).snapshot().cdml:
			raise RuntimeError("Rust reopen did not preserve the authored Plus")
		reopened_tab = _reopen_through_native_file_route(window, path)
		if reopened_tab is None or '<plus' not in reopened_tab.current_snapshot.cdml:
			raise RuntimeError("Ferrum native Open did not reproject the saved Plus")
		print(json.dumps({"schema": "ferrum-plus-authoring-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	sys.exit(main())

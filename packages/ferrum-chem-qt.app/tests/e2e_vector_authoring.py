"""Offscreen Ferrum workflow: author, move, undo, save, and reopen vectors."""

# Standard Library
import json
import pathlib
import shutil
import sys
import tempfile

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_chem
import ferrum_qt.ferrum.main_window


def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one finite backend point through the live viewport."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


def _open_and_wait(host: object, path: pathlib.Path) -> None:
	"""Use the public asynchronous native CDML route before authoring."""
	completed = []
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)
	def receive(file_path: str, success: bool) -> None:
		if pathlib.Path(file_path) == path:
			completed.append(success)
			loop.quit()
	host.local_document_open_completed.connect(receive)
	timeout.timeout.connect(loop.quit)
	try:
		if not host.open_file_path(str(path)):
			raise RuntimeError("native asynchronous CDML open did not start")
		timeout.start(5000)
		loop.exec()
	finally:
		timeout.stop()
		host.local_document_open_completed.disconnect(receive)
	if completed != [True]:
		raise RuntimeError("native asynchronous CDML open did not complete successfully")


def main() -> int:
	"""Exercise all five renderer-bridge shapes in a real offscreen window."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	root = pathlib.Path(tempfile.mkdtemp(prefix="ferrum-vector-e2e-", dir=pathlib.Path.cwd()))
	try:
		# An offscreen E2E must surface a typed refusal as a Python failure rather
		# than leave a modal dialog waiting for an unavailable human response.
		window._show_edit_refusal = lambda _request: None
		source_path = root / "ferrum-vector-e2e-source.cdml"
		source_path.write_text("<cdml/>", encoding="utf-8")
		_open_and_wait(window, source_path)
		tab = window._active_native_tab()
		if tab is None:
			raise RuntimeError("native asynchronous CDML open did not install a Ferrum tab")
		window.show()
		app.processEvents()
		for index, action in enumerate(window._draw_vector_actions.values()):
			start, end = _point(tab, 20.0 + 50.0 * index, 20.0), _point(tab, 44.0 + 50.0 * index, 48.0)
			action.trigger()
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
			PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
			app.processEvents()
		created = tab.current_snapshot.cdml
		if not all(token in created for token in ("<polyline", "<rect", "<square", "<oval", "<circle")):
			raise RuntimeError("the five vector tools did not create their durable Rust roots")
		window._translate_roots_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 32.0, 34.0))
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), _point(tab, 48.0, 50.0))
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 48.0, 50.0))
		app.processEvents()
		moved = tab.current_snapshot.cdml
		if moved == created:
			raise RuntimeError("Move Complete Roots did not move the selected vector")
		tab.undo()
		if tab.current_snapshot.cdml != created:
			raise RuntimeError("Undo did not restore the authored vector document")
		path = root / "ferrum-vector-e2e-saved.cdml"
		if not window.save_active_to_path(str(path)):
			raise RuntimeError("native Save As route did not publish the vector document")
		reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
		if "<circle" not in reopened.snapshot().cdml:
			raise RuntimeError("Rust reopen did not preserve the authored vector roots")
		print(json.dumps({"schema": "ferrum-vector-authoring-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()
		shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
	sys.exit(main())

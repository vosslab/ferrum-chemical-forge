"""Offscreen Ferrum workflow: create, move, undo, save, and reopen one Arrow."""

# Standard Library
import json
import pathlib
import sys
import tempfile

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_chem
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1'><atom id='atom-c' name='C'><point x='10' y='20'/></atom></molecule>
</cdml>"""


#============================================
def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map a finite backend scene point to the live viewport."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


#============================================
def main() -> int:
	"""Run the complete arrow-authoring path and publish a compact receipt."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "arrow-e2e.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		app.processEvents()
		start, end = _point(tab, 24.0, 30.0), _point(tab, 124.0, 30.0)
		window._draw_arrow_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		app.processEvents()
		created_cdml = tab.current_snapshot.cdml
		if "<arrow" not in created_cdml or window._render_interaction_selection is None:
			raise RuntimeError("Draw Arrow did not create and select one durable Arrow")
		window._translate_roots_action.trigger()
		move_start, move_end = _point(tab, 74.0, 30.0), _point(tab, 94.0, 48.0)
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, move_start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), move_end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, move_end)
		app.processEvents()
		moved_cdml = tab.current_snapshot.cdml
		if moved_cdml == created_cdml:
			raise RuntimeError("Move Complete Roots did not translate the created Arrow")
		tab.undo()
		app.processEvents()
		if tab.current_snapshot.cdml != created_cdml:
			raise RuntimeError("Undo did not restore the pre-translation Arrow document")
		with tempfile.TemporaryDirectory(prefix="ferrum-arrow-e2e-") as directory:
			path = pathlib.Path(directory) / "arrow.cdml"
			path.write_text(tab.current_snapshot.cdml, encoding="utf-8")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if "<arrow" not in reopened.snapshot().cdml:
				raise RuntimeError("Rust reopen did not preserve the authored Arrow")
		print(json.dumps({"schema": "ferrum-arrow-authoring-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	sys.exit(main())

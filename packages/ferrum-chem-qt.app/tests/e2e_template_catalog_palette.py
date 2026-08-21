"""Offscreen template catalog browse, place, move, undo, save, and reopen."""

import json
import pathlib
import shutil
import sys
import tempfile

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

import ferrum_chem
import ferrum_qt.ferrum.catalog_palette
import ferrum_qt.ferrum.main_window


def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


def main() -> int:
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	root = pathlib.Path(tempfile.mkdtemp(prefix="ferrum-catalog-e2e-", dir=pathlib.Path.cwd()))
	try:
		window._show_edit_refusal = lambda _request: None
		tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab("<cdml/>", "catalog-e2e.cdml")
		window._register_native_tab(tab, activate=True)
		palette = ferrum_qt.ferrum.catalog_palette.FerrumCatalogPalette(window)
		palette.search.setText("benzene")
		app.processEvents()
		if palette.selected_key() != "system/rings/benzene":
			raise RuntimeError("Rust catalog search did not return benzene")
		window.show()
		app.processEvents()
		if not window.start_catalog_placement(palette.selected_key()):
			raise RuntimeError("Ferrum catalog placement did not start")
		anchor = _point(tab, 80.0, 60.0)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), anchor)
		PySide6.QtTest.QTest.mouseClick(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor)
		app.processEvents()
		created = tab.current_snapshot.cdml
		if "Benzene" not in created:
			raise RuntimeError("catalog click did not commit canonical benzene")
		window._translate_roots_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), _point(tab, 100.0, 80.0))
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 100.0, 80.0))
		app.processEvents()
		if tab.current_snapshot.cdml == created:
			raise RuntimeError("Move Complete Roots did not move catalog benzene")
		tab.undo()
		if tab.current_snapshot.cdml != created:
			raise RuntimeError("Undo did not restore placed benzene")
		path = root / "ferrum-catalog-e2e.cdml"
		if not window.save_active_to_path(str(path)):
			raise RuntimeError("Ferrum Save As did not publish catalog benzene")
		if "Benzene" not in ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8")).snapshot().cdml:
			raise RuntimeError("Rust reopen did not preserve catalog benzene")
		print(json.dumps({"schema": "ferrum-template-catalog-qt-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()
		shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
	sys.exit(main())

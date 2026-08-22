"""Installed-wheel V2 Haworth hover preview and commit proof."""

import json
import sys

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window


HAWORTH_KEY = "biomolecules/carbohydrates/d-glucose/alpha-d-glucopyranose"


def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


def main() -> int:
	"""Require V2 renderer paths to drive the public transient Qt item group."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	try:
		window._show_edit_refusal = lambda _request: None
		tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab("<cdml xmlns='urn:ferrum:cdml'/>", "haworth-v2.cdml")
		window._register_native_tab(tab, activate=True)
		window.show()
		app.processEvents()
		if not window.start_catalog_placement(HAWORTH_KEY):
			raise RuntimeError("Haworth V2 placement did not start")
		anchor = _point(tab, 100.0, 100.0)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), anchor)
		app.processEvents()
		intent = window._catalog_placement_intent
		if intent is None or intent.preview is None or intent.item is None:
			raise RuntimeError("Haworth V2 hover did not create a renderer preview item")
		operations = [
			operation
			for batch in intent.preview.overlay.plan.batches
			for operation in batch.operations
		]
		paths = [operation.operation for operation in operations if operation.kind == "path"]
		if len(paths) < 2 or not all(path.fill_paint is not None for path in paths):
			raise RuntimeError("Haworth V2 preview lost renderer-issued directed wedge paths")
		if intent.item.boundingRect().isEmpty():
			raise RuntimeError("Qt preview did not retain a visible projection of renderer batches")
		if tab.current_snapshot.revision != 0:
			raise RuntimeError("Hover preview mutated the Rust document")
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		app.processEvents()
		if window._catalog_placement_intent is not None or tab.current_snapshot.revision != 0:
			raise RuntimeError("Escape did not dispose the V2 preview without mutation")
		if not window.start_catalog_placement(HAWORTH_KEY):
			raise RuntimeError("Haworth V2 placement did not restart after cancellation")
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), anchor)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor,
		)
		app.processEvents()
		cdml = tab.current_snapshot.cdml
		if tab.current_snapshot.revision != 1 or 'type="q1"' not in cdml or 'type="w1"' not in cdml:
			raise RuntimeError("Haworth V2 commit did not preserve canonical q1/w1 depiction")
		print(json.dumps({"schema": "ferrum-catalog-renderer-preview-v2-e2e", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	sys.exit(main())

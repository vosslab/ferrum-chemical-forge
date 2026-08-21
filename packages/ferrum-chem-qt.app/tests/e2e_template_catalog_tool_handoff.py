"""Offscreen catalog-to-tool handoff through direct and compact ribbon actions."""

import json
import sys

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


_CDML = "<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'><molecule id='m'><atom id='a' name='C'><point x='10' y='20'/></atom></molecule></cdml>"


def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one Rust-authored scene coordinate into the current viewport."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


def _preview_catalog(window: object, tab: object, point: PySide6.QtCore.QPoint) -> None:
	"""Start the opaque catalog gesture and require its live Rust preview."""
	if not window.start_catalog_placement("system/rings/benzene"):
		raise RuntimeError("catalog placement did not start")
	PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), point)
	intent = window._catalog_placement_intent
	if intent is None or intent.preview is None or intent.item is None:
		raise RuntimeError("catalog placement did not paint its preview")


def _drag(viewport: PySide6.QtWidgets.QWidget, start: PySide6.QtCore.QPoint,
		end: PySide6.QtCore.QPoint) -> None:
	"""Deliver a complete real pointer gesture to the active tool owner."""
	PySide6.QtTest.QTest.mousePress(
		viewport, PySide6.QtCore.Qt.MouseButton.LeftButton, pos=start,
	)
	PySide6.QtTest.QTest.mouseMove(viewport, end)
	PySide6.QtTest.QTest.mouseRelease(
		viewport, PySide6.QtCore.Qt.MouseButton.LeftButton, pos=end,
	)


def main() -> int:
	"""Prove catalog retirement precedes direct and More Tools filter activation."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "catalog-handoff-e2e.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		app.processEvents()
		plus_point = _point(tab, 72.0, 36.0)

		_preview_catalog(window, tab, plus_point)
		window._draw_plus_action.trigger()
		app.processEvents()
		if window._catalog_placement_intent is not None or window._line_gesture_intent is None:
			raise RuntimeError("direct Draw Plus did not replace catalog capture")
		_drag(tab.view.viewport(), plus_point, plus_point)
		app.processEvents()
		if "<plus" not in tab.current_snapshot.cdml:
			raise RuntimeError("direct Draw Plus did not receive the viewport click")

		window.resize(1024, 800)
		app.processEvents()
		ribbon = window._authoring_ribbon
		more_tools = ribbon._more_tools_button.menu()
		if more_tools is None:
			raise RuntimeError("compact ribbon did not expose More tools")
		_preview_catalog(window, tab, _point(tab, 50.0, 40.0))
		more_tools.actions()[more_tools.actions().index(window._draw_bond_action)].trigger()
		app.processEvents()
		if window._catalog_placement_intent is not None or window._line_gesture_intent is None:
			raise RuntimeError("More Tools Draw Bond did not replace catalog capture")
		before = tab.current_snapshot.revision
		_drag(tab.view.viewport(), _point(tab, 10.0, 20.0), _point(tab, 32.0, 20.0))
		app.processEvents()
		if tab.current_snapshot.revision != before + 1:
			raise RuntimeError("More Tools Draw Bond did not receive the viewport drag")

		window._draw_bond_action.trigger()
		app.processEvents()
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		app.processEvents()
		if window._line_gesture_intent is not None or window._catalog_placement_intent is not None:
			raise RuntimeError("Escape did not cancel only the active incoming tool")
		print(json.dumps({"schema": "ferrum-template-catalog-tool-handoff-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	sys.exit(main())

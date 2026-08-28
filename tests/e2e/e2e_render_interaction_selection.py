"""Exercise P0.2 root selection against the staged offscreen Ferrum runtime."""

# Standard Library
import json
import math
import pathlib

# local repo modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.08'>
<molecule id='left'><atom id='left-c' name='C'><point x='10' y='20'/></atom></molecule>
<plus id='plus-1'><point x='80' y='20'/></plus>
</cdml>"""


#============================================
class RenderInteractionE2eError(RuntimeError):
	"""One failed Rust-owned Qt selection workflow assertion."""


#============================================
def _mixed_positions(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[tuple[float, float], tuple[float, float]]:
	"""Return current backend-owned coordinates for the molecule and plus root."""
	projection = tab.current_document_observation().projection
	atom = projection.molecules[0].atoms[0].position
	plus = projection.presentation_stack.entries[0].plus.anchor
	return (atom.x, atom.y), (plus.x, plus.y)


#============================================
def _view_point(tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		position: tuple[float, float]) -> PySide6.QtCore.QPoint:
	"""Map one current Rust-observed root coordinate to the visible viewport."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(*position))


#============================================
def main() -> int:
	"""Run selection, marquee, move, nudge, undo, save, and Rust reopen."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_CDML, "p0-selection.cdml", window._require_document_display_palette(),
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	app.processEvents()
	try:
		window._translate_roots_action.trigger()
		left_position, plus_position = _mixed_positions(tab)
		left = _view_point(tab, left_position)
		plus = _view_point(tab, plus_position)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, left,
		)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier, plus,
		)
		tab.view.set_hex_grid_snap_enabled(False)
		before_move = tab.current_snapshot
		before_raw = _mixed_positions(tab)
		end = plus + PySide6.QtCore.QPoint(12, 0)
		raw_delta = tab.view.mapToScene(end).x() - tab.view.mapToScene(left).x()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, left,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end, 20)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		if tab.current_snapshot.revision <= before_move.revision:
			raise RenderInteractionE2eError("pointer drag did not commit through Rust")
		after_raw = _mixed_positions(tab)
		if (
			abs(after_raw[0][0] - before_raw[0][0] - raw_delta) > 0.02
			or abs(after_raw[1][0] - before_raw[1][0] - raw_delta) > 0.02
		):
				raise RenderInteractionE2eError("off-grid drag did not retain raw Rust delta")
		upper_left = tab.view.mapFromScene(PySide6.QtCore.QPointF(-10.0, 0.0))
		lower_right = tab.view.mapFromScene(PySide6.QtCore.QPointF(220.0, 80.0))
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, upper_left,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), lower_right, 20)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, lower_right,
		)
		before_nudge = tab.current_snapshot
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Right)
		if tab.current_snapshot.revision <= before_nudge.revision:
				raise RenderInteractionE2eError("keyboard nudge did not commit through Rust")
		left_position, plus_position = _mixed_positions(tab)
		left = _view_point(tab, left_position)
		plus = _view_point(tab, plus_position)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, left,
		)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier, plus,
		)
		tab.view.set_hex_grid_snap_enabled(True)
		before_snap = _mixed_positions(tab)
		end = left + PySide6.QtCore.QPoint(31, 0)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, left,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end, 20)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		after_snap = _mixed_positions(tab)
		atom_delta = (
			after_snap[0][0] - before_snap[0][0], after_snap[0][1] - before_snap[0][1],
		)
		plus_delta = (
			after_snap[1][0] - before_snap[1][0], after_snap[1][1] - before_snap[1][1],
		)
		if (
			math.hypot(*atom_delta) <= 0.01
			or abs(plus_delta[0] - atom_delta[0]) > 0.01
			or abs(plus_delta[1] - atom_delta[1]) > 0.01
		):
			raise RenderInteractionE2eError(
				"on-grid drag did not move both selected roots by one vector"
			)
		snapped_revision = tab.current_snapshot.revision
		if tab.undo().observation.snapshot.revision <= snapped_revision:
			raise RenderInteractionE2eError("undo did not restore the snapped root move")
		path = pathlib.Path("/private/tmp/ferrum-p0-selection-e2e.cdml")
		tab.save_atomic(path)
		reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
		if reopened.snapshot().digest != tab.current_snapshot.digest:
			raise RenderInteractionE2eError("save/reopen changed the Rust document")
		print(json.dumps({"schema": "ferrum-p0-selection-e2e-v1", "status": "ok"}))
		return 0
	finally:
		ferrum_qt_e2e.close_e2e_main_window(window, app)


if __name__ == "__main__":
	raise SystemExit(main())

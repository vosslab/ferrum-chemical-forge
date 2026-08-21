#!/usr/bin/env python3
"""Exercise Ferrum P0.2 Rust-owned root selection through the offscreen Qt app."""

# Standard Library
import json
import math
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window


_CDML = """<cdml version='26.08'>
<molecule id='left'><atom id='left-c' name='C'><point x='10' y='20'/></atom></molecule>
<plus id='plus-1'><point x='80' y='20'/></plus>
</cdml>"""


#============================================
class RenderInteractionE2eError(RuntimeError):
	"""One failed Rust-owned Qt selection workflow assertion."""


#============================================
def _durable_point(tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		key: tuple[str, str]) -> PySide6.QtCore.QPoint:
	"""Return one visible projected point for one durable root member."""
	item = tab._controller.projection.durable_items[key]
	return tab.view.mapFromScene(item.mapToScene(item.shape().boundingRect().center()))


#============================================
def _mixed_positions(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[tuple[float, float], tuple[float, float]]:
	"""Return current backend-owned coordinates for the molecule and plus root."""
	projection = tab.current_document_observation().projection
	atom = projection.molecules[0].atoms[0].position
	plus = projection.presentation_stack.roots[0].plus.anchor
	return (atom.x, atom.y), (plus.x, plus.y)


#============================================
def main() -> int:
	"""Run selection, marquee, move, nudge, undo, save, and Rust reopen."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "p0-selection.cdml")
	window._register_native_tab(tab, activate=True)
	window.show()
	app.processEvents()
	try:
		window._translate_roots_action.trigger()
		left = _durable_point(tab, ("atom", "left-c"))
		plus_key = next(
			key for key in tab._controller.projection.durable_items if key[0] == "plus"
		)
		plus = _durable_point(tab, plus_key)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, left,
		)
		if len(window._render_interaction_selection.roots) != 1:
			raise RenderInteractionE2eError("click did not select one Rust-renderable root")
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier, plus,
		)
		if len(window._render_interaction_selection.roots) != 2:
			raise RenderInteractionE2eError("Shift click did not add the Rust plus root")
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
		if len(window._render_interaction_selection.roots) != 2:
			raise RenderInteractionE2eError("marquee did not select both fully-contained roots")
		before_nudge = tab.current_snapshot
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Right)
		if tab.current_snapshot.revision <= before_nudge.revision:
				raise RenderInteractionE2eError("keyboard nudge did not commit through Rust")
		left = _durable_point(tab, ("atom", "left-c"))
		plus_key = next(
			key for key in tab._controller.projection.durable_items if key[0] == "plus"
		)
		plus = _durable_point(tab, plus_key)
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
			abs(math.hypot(*atom_delta) - 40.0) > 0.01
			or abs(plus_delta[0] - atom_delta[0]) > 0.01
			or abs(plus_delta[1] - atom_delta[1]) > 0.01
		):
			raise RenderInteractionE2eError("on-grid drag did not apply one 40-point hex delta")
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
		for native_tab in tuple(window._native_tabs_by_page.values()):
			native_tab.dispose()
		window.deleteLater()


if __name__ == "__main__":
	raise SystemExit(main())

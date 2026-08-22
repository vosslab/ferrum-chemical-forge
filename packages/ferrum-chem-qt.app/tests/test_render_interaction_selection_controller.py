"""P0.2 Qt behavior for Rust-owned molecule-root interaction."""

# Standard Library
import pathlib

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.main_window


_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.08'>
<molecule id='left'><atom id='left-c' name='C'><point x='10' y='20'/></atom></molecule>
<plus id='plus-1'><point x='80' y='20'/></plus>
</cdml>"""

_EXCLUDED_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.08'>
<molecule id='blocked'><atom id='atom-c' name='C'><point x='10' y='20'/><ftext><b>rich</b></ftext></atom></molecule>
</cdml>"""


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one offscreen Qt application."""
	app = PySide6.QtWidgets.QApplication.instance()
	return app if app is not None else PySide6.QtWidgets.QApplication([])


#============================================
def _durable_viewport_point(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		key: tuple[str, str],
		) -> PySide6.QtCore.QPoint:
	"""Return a visible point over one durable current projection item."""
	item = tab._controller.projection.durable_items[key]
	center = item.shape().boundingRect().center()
	return tab.view.mapFromScene(item.mapToScene(center))


#============================================
def _mixed_positions(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[tuple[float, float], tuple[float, float]]:
	"""Return backend-owned molecule and plus coordinates for one mixed move."""
	projection = tab.current_document_observation().projection
	atom = projection.molecules[0].atoms[0].position
	plus = projection.presentation_stack.roots[0].plus.anchor
	return (atom.x, atom.y), (plus.x, plus.y)


#============================================
def _select_mixed_roots(
		window: ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[PySide6.QtCore.QPoint, PySide6.QtCore.QPoint]:
	"""Select the molecule and plus through the single Rust-owned tool."""
	left = _durable_viewport_point(tab, ("atom", "left-c"))
	plus_key = next(key for key in tab._controller.projection.durable_items if key[0] == "plus")
	plus = _durable_viewport_point(tab, plus_key)
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, left,
	)
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier, plus,
	)
	assert len(window._render_interaction_selection.roots) == 2
	return left, plus


#============================================
def _dispose_test_window(
		window: ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Dispose test-owned tabs without triggering an interactive close route."""
	for index, tab in reversed(tuple(enumerate(window._native_tabs_by_page.values()))):
		tab.dispose()
		window._tab_widget.removeTab(index)
	window.deleteLater()
	qapp.sendPostedEvents(None, PySide6.QtCore.QEvent.Type.DeferredDelete)
	qapp.processEvents()


#============================================
def test_select_marquee_move_nudge_undo_and_save_reopen(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The Qt controller uses opaque Rust selection and translation handles."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "roots.cdml")
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	try:
		window._translate_roots_action.trigger()
		left = _durable_viewport_point(tab, ("atom", "left-c"))
		plus_key = next(
			key for key in tab._controller.projection.durable_items if key[0] == "plus"
		)
		plus = _durable_viewport_point(tab, plus_key)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, left,
		)
		assert len(window._render_interaction_selection.roots) == 1
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier, plus,
		)
		assert len(window._render_interaction_selection.roots) == 2
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			left + PySide6.QtCore.QPoint(-20, -20),
		)
		PySide6.QtTest.QTest.mouseMove(
			tab.view.viewport(), plus + PySide6.QtCore.QPoint(20, 20), 20,
		)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			plus + PySide6.QtCore.QPoint(20, 20),
		)
		assert len(window._render_interaction_selection.roots) == 2
		before = tab.current_snapshot
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Right)
		moved = tab.current_snapshot
		assert moved.revision > before.revision
		undone = tab.undo().observation.snapshot
		assert undone.revision > moved.revision
		path = tmp_path / "p0-selection.cdml"
		tab.save_atomic(path)
		reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
		assert reopened.snapshot().digest == tab.current_snapshot.digest
	finally:
		_dispose_test_window(window, qapp)


#============================================
def test_known_excluded_root_refuses_without_mutation(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The named Rust query distinguishes excluded content from blank canvas."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_EXCLUDED_CDML, "blocked.cdml")
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	try:
		before = tab.current_snapshot
		observation = tab.observe_direct_root_interaction()
		with pytest.raises(ferrum_chem.RenderInteractionError) as caught:
			tab.select_direct_roots(
				observation, None,
				ferrum_qt.ferrum.engine.RenderInteractionQueryV1.root("blocked"),
			)
		request = window._render_interaction_refusal(caught.value)
		assert request is not None
		assert tab.current_snapshot.digest == before.digest
	finally:
		_dispose_test_window(window, qapp)


#============================================
def test_mixed_root_drag_delegates_raw_and_hex_grid_deltas_to_rust(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The view Boolean delegates raw and grid translation to Rust unchanged."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "grid.cdml")
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	try:
		tab.view.set_hex_grid_snap_enabled(False)
		window._translate_roots_action.trigger()
		left, _plus = _select_mixed_roots(window, tab)
		before = _mixed_positions(tab)
		end = left + PySide6.QtCore.QPoint(17, 0)
		raw_delta = tab.view.mapToScene(end).x() - tab.view.mapToScene(left).x()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, left,
		)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		raw = _mixed_positions(tab)
		assert raw[0][0] - before[0][0] == pytest.approx(raw_delta, abs=0.01)
		assert raw[1][0] - before[1][0] == pytest.approx(raw_delta, abs=0.01)
		left, _plus = _select_mixed_roots(window, tab)
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
		snapped = _mixed_positions(tab)
		atom_delta = (
			snapped[0][0] - before_snap[0][0], snapped[0][1] - before_snap[0][1],
		)
		plus_delta = (
			snapped[1][0] - before_snap[1][0], snapped[1][1] - before_snap[1][1],
		)
		assert atom_delta != pytest.approx((0.0, 0.0))
		assert plus_delta == pytest.approx(atom_delta)
		moved_revision = tab.current_snapshot.revision
		assert tab.undo().observation.snapshot.revision > moved_revision
	finally:
		_dispose_test_window(window, qapp)

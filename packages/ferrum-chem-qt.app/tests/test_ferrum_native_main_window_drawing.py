"""Drawing-tool lifecycle coverage for the public Ferrum window seam."""

# Standard Library
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.window_refusals

_BOND_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  <atom id='atom-o' name='O'><point x='40' y='20'/></atom>
</molecule></cdml>"""

_EDITABLE_CDML = """<cdml xmlns='urn:ferrum:cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  </molecule>
</cdml>"""

_MIXED_ROOT_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='13' y='17'/></atom>
</molecule><plus id='plus-1'><point x='73' y='51'/></plus></cdml>"""

_DUPLICATE_MARK_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.07'><molecule id='mol-1'>
  <atom id='atom-c' name='C' charge='2'><point x='10' y='20'/>
    <mark type='plus' x='18' y='28' size='10' data-origin='first'/>
    <mark type='plus' x='20' y='30' size='10' data-origin='second'/>
  </atom>
</molecule></cdml>"""

_AUTHORED_COORDINATE_TOLERANCE = 0.001 * 72.0 / 2.54


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide an offscreen application without importing legacy fixtures."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _atom_viewport_point(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		atom_id: str,
		) -> PySide6.QtCore.QPoint:
	"""Return a stable interior viewport point for one projected durable atom."""
	item = tab._controller.projection.durable_items[("atom", atom_id)]
	shape = item.shape()
	bounds = shape.boundingRect()
	for x_step in range(1, 10):
		for y_step in range(1, 10):
			point = PySide6.QtCore.QPointF(
				bounds.left() + bounds.width() * x_step / 10.0,
				bounds.top() + bounds.height() * y_step / 10.0,
			)
			if shape.contains(point):
				return tab.view.mapFromScene(item.mapToScene(point))
	raise AssertionError("projected atom has no interior hit-test point")


#============================================
def _empty_viewport_point(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> PySide6.QtCore.QPoint:
	"""Return one visible viewport point that does not hit a durable atom."""
	rect = tab.view.viewport().rect().adjusted(12, 12, -12, -12)
	for x_step in range(1, 10):
		for y_step in range(1, 10):
			point = PySide6.QtCore.QPoint(
				rect.left() + rect.width() * x_step // 10,
				rect.top() + rect.height() * y_step // 10,
			)
			if tab.durable_atom_at_viewport_point(point) is None:
				return point
	raise AssertionError("Ferrum viewport has no empty hit-test point")


#============================================
def _select_mixed_complete_roots(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[tuple[str, str], ...]:
	"""Select one complete molecule and one independent presentation root."""
	plus_key = next(
		key for key in tab._controller.projection.durable_items if key[0] == "plus"
	)
	tab._controller.projection.select_durable((("atom", "atom-c"), plus_key))
	return tab.selected_top_level_transform_targets()[1]


#============================================
def _mixed_root_positions(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[tuple[float, float], tuple[float, float]]:
	"""Return authoritative molecule and presentation coordinates after a move."""
	projection = tab.current_document_observation().projection
	atom = projection.molecules[0].atoms[0].position
	plus = projection.presentation_stack.roots[0].plus.anchor
	return (atom.x, atom.y), (plus.x, plus.y)


#============================================
def _click_visible_menu_action(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Activate one labelled command through its visible top-level menu item."""
	menu_bar = window.menuBar()
	for menu_action in menu_bar.actions():
		menu = menu_action.menu()
		if menu is None:
			continue
		for candidate in menu.actions():
			if candidate.text().replace("&", "") != label:
				continue
			PySide6.QtTest.QTest.mouseClick(
				menu_bar, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu_bar.actionGeometry(menu_action).center(),
			)
			qapp.processEvents()
			if not menu.isVisible():
				raise AssertionError(f"Visible menu did not open for {label!r}")
			PySide6.QtTest.QTest.mouseClick(
				menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu.actionGeometry(candidate).center(),
			)
			qapp.processEvents()
			return
	raise AssertionError(f"No visible menu action is labelled {label!r}")


#============================================
def _restore_drawing_parameters(
		window: ferrum_qt.main_window.MainWindow, snapshot: object,
		) -> None:
	"""Restore application-owned choices after a behavior test changes them."""
	window._drawing_parameters.set_element(snapshot.element)
	window._drawing_parameters.set_order_name(snapshot.order_name)


#============================================
def _dispose_test_window(
		window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Retire pointer ownership before deleting a potentially dirty test window."""
	window._cancel_atom_insertion()
	window._cancel_line_gesture()
	window.hide()
	window.deleteLater()
	qapp.sendPostedEvents(None, PySide6.QtCore.QEvent.Type.DeferredDelete)
	qapp.processEvents()


def test_editing_tools_draw_bond_commits_rust_and_escape_preserves_result(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Editing Tools reaches normal Ferrum bond authoring and Escape recovery."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	prior_choices = window._drawing_parameters.snapshot()
	try:
		start = _atom_viewport_point(tab, "atom-c")
		end = _atom_viewport_point(tab, "atom-o")
		assert window._drawing_parameters.set_order_name("single")
		qapp.processEvents()
		_click_visible_menu_action(window, "Draw Bond", qapp)
		assert "Normal single" in window.statusBar().currentMessage()
		assert "drag between atoms or empty canvas locations" in window._draw_bond_action.toolTip()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		selected_bond_id = tab.selected_bond_projection().source_id
		bond = tab.selected_bond_projection()
		assert bond.source_type == "n1"
		assert (bond.start.source_id, bond.end.source_id) == ("atom-c", "atom-o")
		accepted_snapshot = tab.current_snapshot
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape,
		)
		assert tab.current_snapshot == accepted_snapshot
		assert tab.selected_bond_projection().source_id == selected_bond_id
		tab.save_atomic(tmp_path / "drag-bond.cdml")
	finally:
		_restore_drawing_parameters(window, prior_choices)
		_dispose_test_window(window, qapp)


#============================================
def test_editing_tools_cancel_preserves_document_and_selected_atom(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The visible Cancel Tool client preserves current operation and selection."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "cancel-tool.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	prior_choices = None
	try:
		prior_choices = window._drawing_parameters.snapshot()
		tab.select_atom("atom-c")
		before_snapshot = tab.current_snapshot
		before_atom_id = tab.selected_atom_projection().source_id
		window._drawing_parameters.set_element("N")
		window._drawing_parameters.set_order_name("triple")
		_click_visible_menu_action(window, "Draw Bond", qapp)
		_click_visible_menu_action(window, "Cancel Tool", qapp)
		assert tab.current_snapshot == before_snapshot
		assert tab.selected_atom_projection().source_id == before_atom_id
		assert window._drawing_parameters.snapshot() == (
			ferrum_qt.ferrum.drawing_parameters.
			FerrumNativeDrawingParametersSnapshot("N", "triple")
		)
	finally:
		if prior_choices is not None:
			_restore_drawing_parameters(window, prior_choices)
		window.close()
		window.deleteLater()


#============================================
def test_draw_bond_to_empty_space_uses_normal_order(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Empty-space release commits one carbon endpoint with its frozen normal order."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "extend-bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	prior_choices = window._drawing_parameters.snapshot()
	try:
		start = _atom_viewport_point(tab, "atom-c")
		end = _empty_viewport_point(tab)
		window._drawing_parameters.set_order_name("triple")
		_click_visible_menu_action(window, "Draw Bond", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		molecule = tab._document_observation.projection.molecules[0]
		created = next(
			atom for atom in molecule.atoms if atom.source_id not in {"atom-c", "atom-o"}
		)
		assert created.element == "C"
		assert molecule.bonds[0].source_type == "n3"
		tab.save_atomic(tmp_path / "empty-space-bond.cdml")
	finally:
		_restore_drawing_parameters(window, prior_choices)
		window.close()
		window.deleteLater()


#============================================
def test_draw_bond_gesture_freezes_normal_order_at_mouse_press(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""One active drag retains the normal order visible when it began."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "captured-drawing.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	prior_choices = window._drawing_parameters.snapshot()
	try:
		start = _atom_viewport_point(tab, "atom-c")
		end = _empty_viewport_point(tab)
		window._drawing_parameters.set_order_name("triple")
		_click_visible_menu_action(window, "Draw Bond", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		window._drawing_parameters.set_order_name("single")
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		molecule = tab.current_document_observation().projection.molecules[0]
		created = next(
			atom for atom in molecule.atoms if atom.source_id not in {"atom-c", "atom-o"}
		)
		assert created.element == "C"
		assert molecule.bonds[0].source_type == "n3"
	finally:
		_restore_drawing_parameters(window, prior_choices)
		window.close()
		window.deleteLater()


#============================================
def test_move_atom_drag_snaps_the_translated_atom_target(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The move tool applies the shared snap policy after pointer translation."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "move-atom.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	start = _atom_viewport_point(tab, "atom-c")
	end = _empty_viewport_point(tab)
	start_pointer = tab.view.mapToScene(start)
	end_pointer = tab.view.mapToScene(end)
	anchor = tab.durable_atom_scene_position("atom-c")
	expected = tab.view.snap_authored_scene_point(anchor + (end_pointer - start_pointer))
	window._move_atom_action.trigger()
	PySide6.QtTest.QTest.mousePress(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
	)
	assert window._line_gesture_intent.preview is not None
	PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
	PySide6.QtTest.QTest.mouseRelease(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
	)
	atom = tab._document_observation.projection.molecules[0].atoms[0]
	assert (atom.position.x, atom.position.y) == (expected.x(), expected.y())
	tab.undo()
	restored = tab._document_observation.projection.molecules[0].atoms[0].position
	assert (restored.x, restored.y) == (10.0, 20.0)
	window._move_atom_action.trigger()
	tab.save_atomic(tmp_path / "moved-atom.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_move_complete_roots_drag_resolves_one_snapped_rust_anchor_delta(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A visible mixed-root move applies one snapped rigid translation."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_MIXED_ROOT_CDML, "snapped-roots.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	refusals: list[object] = []
	monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
	try:
		durable_selection = _select_mixed_complete_roots(tab)
		window._refresh_actions()
		_click_visible_menu_action(window, "Move Complete Roots", qapp)
		start = _atom_viewport_point(tab, "atom-c")
		end = _empty_viewport_point(tab)
		committed_previews: list[object] = []
		commit = tab.commit_direct_root_translation

		def record_commit(gesture: object, preview: object) -> object:
			committed_previews.append(preview)
			return commit(gesture, preview)

		monkeypatch.setattr(tab, "commit_direct_root_translation", record_commit)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		atom, plus = _mixed_root_positions(tab)
		assert len(committed_previews) == 1
		preview = committed_previews[0]
		assert (
			atom == pytest.approx(
				(13.0 + preview.dx, 17.0 + preview.dy),
				abs=_AUTHORED_COORDINATE_TOLERANCE,
			)
			and plus == pytest.approx(
				(73.0 + preview.dx, 51.0 + preview.dy),
				abs=_AUTHORED_COORDINATE_TOLERANCE,
			)
			and tab.selected_top_level_transform_targets()[1] == durable_selection
		)
		assert refusals == []
	finally:
		_dispose_test_window(window, qapp)


#============================================
def test_move_complete_roots_drag_keeps_the_unsnapped_raw_delta(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Disabling the shared preference retains raw pointer displacement."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_MIXED_ROOT_CDML, "raw-roots.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	try:
		_select_mixed_complete_roots(tab)
		window._refresh_actions()
		tab.view.set_hex_grid_snap_enabled(False)
		_click_visible_menu_action(window, "Move Complete Roots", qapp)
		start = _atom_viewport_point(tab, "atom-c")
		end = _empty_viewport_point(tab)
		start_scene = tab.view.mapToScene(start)
		end_scene = tab.view.mapToScene(end)
		expected_delta = (end_scene.x() - start_scene.x(), end_scene.y() - start_scene.y())
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		atom, plus = _mixed_root_positions(tab)
		assert (
			atom == pytest.approx(
				(13.0 + expected_delta[0], 17.0 + expected_delta[1]),
				abs=_AUTHORED_COORDINATE_TOLERANCE,
			)
			and plus == pytest.approx(
				(73.0 + expected_delta[0], 51.0 + expected_delta[1]),
				abs=_AUTHORED_COORDINATE_TOLERANCE,
			)
		)
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_remove_atom_mark_chooser_uses_exact_duplicate_ordinal(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The chooser removes the selected duplicate without string-derived mutation."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_DUPLICATE_MARK_CDML, "duplicate-marks.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("atom-c")
	qapp.processEvents()
	assert window._remove_atom_mark_action.isEnabled()

	def choose_second(_parent: object, _title: str, _label: str,
			items: tuple[str, ...], _current: int, _editable: bool) -> tuple[str, bool]:
		"""Select the second source-ordered plus mark from the explicit chooser."""
		return items[1], True

	monkeypatch.setattr(PySide6.QtWidgets.QInputDialog, "getItem", choose_second)
	window._remove_atom_mark_action.trigger()
	atom = tab.selected_atom_projection()
	assert atom.formal_charge == 1 and len(atom.marks) == 1
	assert atom.marks[0].same_type_ordinal == 0
	assert "data-origin=\"first\"" in tab.current_snapshot.cdml
	assert "data-origin=\"second\"" not in tab.current_snapshot.cdml
	assert "Removed one Ferrum atom mark." in window.statusBar().currentMessage()
	tab.undo()
	assert len(tab.selected_atom_projection().marks) == 2
	tab.save_atomic(tmp_path / "duplicate-marks.cdml")
	window.close()
	window.deleteLater()

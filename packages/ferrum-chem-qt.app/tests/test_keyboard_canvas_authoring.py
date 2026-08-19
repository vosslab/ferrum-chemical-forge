"""Keyboard-only document-cursor authoring tests for the Ferrum canvas."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.keyboard_canvas
import ferrum_qt.main_window


_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  </molecule>
</cdml>"""


#============================================
def _window_with_tab(
		qapp: PySide6.QtWidgets.QApplication,
		) -> tuple[ferrum_qt.main_window.MainWindow, object]:
	"""Return one shown product window with an editable deterministic document."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "keyboard.cdml")
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	tab.view.set_hex_grid_snap_enabled(False)
	return window, tab


#============================================
def test_keyboard_canvas_cursor_declares_grid_and_shift_fine_increment() -> None:
	"""The cursor's movement contract is finite, explicit, and independently testable."""
	assert ferrum_qt.ferrum.keyboard_canvas.keyboard_cursor_increment(False) == 40.0
	assert ferrum_qt.ferrum.keyboard_canvas.keyboard_cursor_increment(True) == 10.0
	assert ferrum_qt.ferrum.keyboard_canvas.KeyboardCanvasCursor(1.0, 2.0).moved(
		3.0, -4.0,
	) == ferrum_qt.ferrum.keyboard_canvas.KeyboardCanvasCursor(4.0, -2.0)


#============================================
def test_keyboard_only_atom_bond_escape_and_focus_flow(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""QTest keys create atom/bond, retain canvas focus, and cancel without mutation."""
	window, tab = _window_with_tab(qapp)
	try:
		atom_point = PySide6.QtCore.QPointF(90.0, 80.0)
		tab.view.set_keyboard_cursor_scene(atom_point)
		PySide6.QtTest.QTest.keyClick(
			window, PySide6.QtCore.Qt.Key.Key_8,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier,
		)
		qapp.processEvents()
		assert tab.view.viewport().hasFocus()
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Right,
		)
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Down,
			PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier,
		)
		cursor = tab.view.show_keyboard_cursor()
		assert (cursor.x(), cursor.y()) == (130.0, 90.0)
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return,
		)
		qapp.processEvents()
		created = tab.selected_atom_projection()
		assert created is not None and created.position.x == 130.0
		assert tab.view.viewport().hasFocus()

		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(10.0, 20.0))
		PySide6.QtTest.QTest.keyClick(
			window, PySide6.QtCore.Qt.Key.Key_2,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier,
		)
		qapp.processEvents()
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return,
		)
		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(
			created.position.x, created.position.y,
		))
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return,
		)
		qapp.processEvents()
		assert len(tab.current_document_observation().projection.molecules[0].bonds) == 1
		assert tab.view.viewport().hasFocus()

		before_revision = tab.current_snapshot.revision
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape,
		)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before_revision
		assert not window._draw_bond_action.isChecked()
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_keyboard_atom_stale_recovery_preserves_document_and_canvas_focus(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""A stale Enter declines safely and leaves the keyboard recovery surface focused."""
	window, tab = _window_with_tab(qapp)
	warnings = []
	try:
		monkeypatch.setattr(
			window, "_show_edit_refusal",
			lambda request, _details=None: warnings.append(request),
		)
		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(90.0, 80.0))
		PySide6.QtTest.QTest.keyClick(
			window, PySide6.QtCore.Qt.Key.Key_8,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier,
		)
		tab.select_atom("atom-c")
		tab.change_selected_atom_element("N")
		before = tuple(
			atom.source_id
			for atom in tab.current_document_observation().projection.molecules[0].atoms
		)
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return,
		)
		qapp.processEvents()
		after = tuple(
			atom.source_id
			for atom in tab.current_document_observation().projection.molecules[0].atoms
		)
		assert after == before
		assert warnings[-1].outcome.value == "stale_tool"
		assert tab.view.viewport().hasFocus()
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_escape_keeps_live_mode_chrome_and_tool_intent_synchronized(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Atom and Bond cancellation/reselection have one visible inactive state."""
	window, tab = _window_with_tab(qapp)
	try:
		for key, expected_mode, action_name in (
				(PySide6.QtCore.Qt.Key.Key_8, "atom", "_add_atom_action"),
				(PySide6.QtCore.Qt.Key.Key_2, "draw", "_draw_bond_action"),
				):
			PySide6.QtTest.QTest.keyClick(
				window, key, PySide6.QtCore.Qt.KeyboardModifier.ControlModifier,
			)
			qapp.processEvents()
			assert window._mode_manager.active_mode_id.value == expected_mode
			assert getattr(window, action_name).isChecked()
			assert window._shared_mode_toolbar._mode_actions[expected_mode].isChecked()
			PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
			qapp.processEvents()
			assert window._mode_manager.active_mode_id is None
			assert not getattr(window, action_name).isChecked()
			assert not window._shared_mode_toolbar._mode_actions[expected_mode].isChecked()
			assert "None" in window.statusBar()._mode_label.text()
			PySide6.QtTest.QTest.keyClick(
				window, key, PySide6.QtCore.Qt.KeyboardModifier.ControlModifier,
			)
			qapp.processEvents()
			assert window._mode_manager.active_mode_id.value == expected_mode
	finally:
		window.close()
		window.deleteLater()

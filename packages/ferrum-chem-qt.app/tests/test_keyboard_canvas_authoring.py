"""Keyboard-only document-cursor authoring tests for the Ferrum canvas."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.keyboard_canvas
import ferrum_qt.main_window


_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  </molecule>
</cdml>"""

_COLLIDING_ATOM_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
    <atom id='atom-n' name='N'><point x='10' y='20'/></atom>
  </molecule>
</cdml>"""

_UNRENDERABLE_LABEL_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <standard font_family='helvetica'/>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/><ftext>C&lt;sub&gt;2&lt;/sub&gt;</ftext></atom>
  </molecule>
</cdml>"""


#============================================
def _window_with_tab(
		qapp: PySide6.QtWidgets.QApplication, cdml: str = _CDML,
		) -> tuple[ferrum_qt.main_window.MainWindow, object]:
	"""Return one shown product window with an editable deterministic document."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(cdml, "keyboard.cdml")
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	tab.view.set_hex_grid_snap_enabled(False)
	return window, tab


#============================================
def _select_normal_carbon_bond(window: object) -> object:
	"""Use the fixed isolated profile expected by keyboard bond authoring."""
	prior = window._drawing_parameters.snapshot()
	window._drawing_parameters.set_element("C")
	window._drawing_parameters.set_order_name("single")
	window._drawing_parameters.set_presentation_name("normal")
	return prior


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
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""QTest keys create atom/bond, retain canvas focus, and cancel without mutation."""
	window, tab = _window_with_tab(qapp)
	prior_choices = _select_normal_carbon_bond(window)
	try:
		bond_operations = []
		add_bond = tab.add_bond_between_atoms
		monkeypatch.setattr(
			tab, "add_bond_between_atoms",
			lambda start, end, presentation: (
				bond_operations.append((start, end, presentation))
				or add_bond(start, end, presentation)
			),
		)
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
		created = tab.selected_atom_projection()
		assert (
			created is not None
			and created.source_id.startswith("ferrum-atom-v1-")
			and (created.position.x, created.position.y) == (130.0, 90.0)
		)
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
		assert window._line_gesture_intent.start_atom_id == "atom-c"
		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(
			created.position.x, created.position.y,
		))
		before_bond_revision = tab.current_snapshot.revision
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return,
		)
		qapp.processEvents()
		bond = tab.current_document_observation().projection.molecules[0].bonds[0]
		assert (bond.source_type, tab.current_snapshot.revision) == ("n1", before_bond_revision + 1)
		assert [(start, end) for start, end, _presentation in bond_operations] == [
			("atom-c", created.source_id),
		]
		window._undo_action.trigger()
		qapp.processEvents()
		assert not tab.current_document_observation().projection.molecules[0].bonds
		window._redo_action.trigger()
		qapp.processEvents()
		assert tab.current_document_observation().projection.molecules[0].bonds[0].source_id == bond.source_id
		assert tab.view.viewport().hasFocus()

		before_revision = tab.current_snapshot.revision
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape,
		)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before_revision
		assert not window._draw_bond_action.isChecked()
	finally:
		window._drawing_parameters.set_element(prior_choices.element)
		window._drawing_parameters.set_order_name(prior_choices.order_name)
		window._drawing_parameters.set_presentation_name(prior_choices.presentation_name)
		window.close()
		window.deleteLater()


#============================================
def test_add_atom_refuses_an_unrenderable_molecule_without_arming_a_tool(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""No click intent is created when Rust has no canvas plan for the molecule."""
	window, tab = _window_with_tab(qapp, _UNRENDERABLE_LABEL_CDML)
	refusals = []
	try:
		monkeypatch.setattr(
			window, "_show_edit_refusal", lambda request, _details=None: refusals.append(request),
		)
		before = tab.current_snapshot
		PySide6.QtTest.QTest.keyClick(
			window, PySide6.QtCore.Qt.Key.Key_8,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier,
		)
		qapp.processEvents()
		assert window._atom_insertion_intent is None
		assert not window._add_atom_action.isChecked()
		assert tab.current_snapshot.revision == before.revision
		assert tab.current_snapshot.cdml == before.cdml
		presentation = ferrum_qt.dialogs.refusal_presenter.present_refusal(refusals[-1])
		assert refusals[-1].outcome.value == "unrenderable_molecule"
		assert presentation.what_happened == "Ferrum did not change the drawing."
		assert "Choose another visible molecule" in presentation.what_next
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
def test_keyboard_bond_collision_refuses_terminally_without_mutation(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A colliding cursor position cannot leave Draw Bond armed or guess an atom."""
	window, tab = _window_with_tab(qapp, _COLLIDING_ATOM_CDML)
	refusals = []
	try:
		monkeypatch.setattr(
			window, "_show_edit_refusal", lambda request: refusals.append(request),
		)
		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(10.0, 20.0))
		before = tab.current_snapshot
		PySide6.QtTest.QTest.keyClick(
			window, PySide6.QtCore.Qt.Key.Key_2,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier,
		)
		qapp.processEvents()
		assert window._line_gesture_intent is not None and window._draw_bond_action.isChecked()

		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return,
		)
		qapp.processEvents()

		assert tab.current_snapshot.revision == before.revision
		assert tab.current_snapshot.cdml == before.cdml
		assert window._line_gesture_intent is None
		assert not window._draw_bond_action.isChecked()
		assert window._mode_manager.active_mode_id is None
		assert tab.view.viewport().hasFocus()
		assert len(refusals) == 1
		assert refusals[-1].context.value == "edit_document"
		assert refusals[-1].outcome.value == "unavailable_operation"
		assert "more than one durable atom" in (refusals[-1].technical_details or "")
		assert "Choose a distinct atom location" in window.statusBar().currentMessage()
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

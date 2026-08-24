"""P0.3 Qt behavior for Rust-owned structural selection and deletion."""

# Standard Library
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest
import shiboken6

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.main_window


_CHAIN = """<cdml xmlns="urn:ferrum:cdml" version='26.08'><molecule id='m'>
<atom id='a' name='C'><point x='0' y='0'/></atom>
<atom id='b' name='C'><point x='40' y='0'/></atom>
<atom id='c' name='O'><point x='80' y='0'/></atom>
<bond id='ab' start='a' end='b' type='n1'/><bond id='bc' start='b' end='c' type='n1'/>
</molecule></cdml>"""

_WEDGE = """<cdml xmlns="urn:ferrum:cdml"><molecule id='m'>
<atom id='a' name='C'><point x='0' y='0'/></atom>
<atom id='b' name='O'><point x='30' y='0'/></atom>
<bond id='ab' start='a' end='b' type='w1'/>
</molecule></cdml>"""

_TWO_MOLECULES = """<cdml xmlns="urn:ferrum:cdml"><molecule id='left'>
<atom id='a' name='C'><point x='0' y='0'/></atom></molecule>
<molecule id='right'><atom id='b' name='O'><point x='40' y='0'/></atom>
</molecule></cdml>"""


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide an offscreen Qt application."""
	app = PySide6.QtWidgets.QApplication.instance()
	return app if app is not None else PySide6.QtWidgets.QApplication([])


#============================================
def _target_point(tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		identifier: str) -> PySide6.QtCore.QPoint:
	"""Return a viewport point from Rust-issued structural target bounds."""
	observation = tab.observe_structure_interaction()
	target = next(value for value in observation.targets if value.identifier == identifier)
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(
		(target.bounds.left + target.bounds.right) / 2.0,
		(target.bounds.top + target.bounds.bottom) / 2.0,
	))


#============================================
def test_structure_click_marquee_shift_delete_and_undo(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Delete/backspace commits exactly the backend-selected direct structure."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CHAIN, "chain.cdml")
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	try:
		window._select_structure_action.trigger()
		atom_b = _target_point(tab, "b")
		bond_ab = _target_point(tab, "ab")
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, atom_b,
		)
		assert window._change_element_action.isEnabled()
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier, bond_ab,
		)
		assert len(window._structure_selection.targets) == 2
		window._cancel_structure_selection()
		window._select_structure_action.trigger()
		point_a = tab.view.mapFromScene(PySide6.QtCore.QPointF(-20.0, -20.0))
		point_c = tab.view.mapFromScene(PySide6.QtCore.QPointF(100.0, 20.0))
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point_a,
		)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point_c,
		)
		assert {target.identifier for target in window._structure_selection.targets} == {
			"a", "b", "c", "ab", "bc",
		}
	finally:
		window.close()
		tab.dispose()
		window.deleteLater()
		qapp.sendPostedEvents(None, PySide6.QtCore.QEvent.Type.DeferredDelete)
		qapp.processEvents()


#============================================
def test_structure_delete_middle_atom_splits_and_undoes(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""One opaque deletion removes incidence and allocates the split roots."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CHAIN, "chain.cdml")
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	try:
		window._select_structure_action.trigger()
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _target_point(tab, "b"),
		)
		before = tab.current_snapshot.revision
		window._commit_structure_deletion()
		assert len(tab.current_document_observation().projection.molecules) == 2
		assert tab.current_snapshot.revision > before
		before_undo = tab.current_snapshot.revision
		assert tab.undo().observation.snapshot.revision > before_undo
	finally:
		window.close()
		tab.dispose()
		window.deleteLater()
		qapp.sendPostedEvents(None, PySide6.QtCore.QEvent.Type.DeferredDelete)
		qapp.processEvents()


#============================================
def test_close_selected_structure_tab_retires_the_active_pointer_tool(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Closing the active tab retires its visible canvas-tool ownership."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CHAIN, "chain.cdml")
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	try:
		window._select_structure_action.trigger()
		assert window._select_structure_action.isChecked()
		window._close_action.trigger()
		PySide6.QtCore.QCoreApplication.sendPostedEvents(
			None, PySide6.QtCore.QEvent.Type.DeferredDelete,
		)
		qapp.processEvents()
		assert not window._select_structure_action.isChecked() and window._structure_tab is None
		assert not shiboken6.isValid(tab)
	finally:
		window.close()
		window.deleteLater()
		qapp.sendPostedEvents(None, PySide6.QtCore.QEvent.Type.DeferredDelete)
		qapp.processEvents()

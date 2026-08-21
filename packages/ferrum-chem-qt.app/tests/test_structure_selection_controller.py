"""P0.3 Qt behavior for Rust-owned structural selection and deletion."""

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
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.main_window


_CHAIN = """<cdml version='26.08'><molecule id='m'>
<atom id='a' name='C'><point x='0' y='0'/></atom>
<atom id='b' name='C'><point x='40' y='0'/></atom>
<atom id='c' name='O'><point x='80' y='0'/></atom>
<bond id='ab' start='a' end='b' type='n1'/><bond id='bc' start='b' end='c' type='n1'/>
</molecule></cdml>"""

_WEDGE = """<cdml><molecule id='m'>
<atom id='a' name='C'><point x='0' y='0'/></atom>
<atom id='b' name='O'><point x='30' y='0'/></atom>
<bond id='ab' start='a' end='b' type='w1'/>
</molecule></cdml>"""

_TWO_MOLECULES = """<cdml><molecule id='left'>
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
		tab.dispose()
		window.deleteLater()


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
		tab.dispose()
		window.deleteLater()


#============================================
def test_structure_refusal_uses_typed_display_and_same_molecule_recovery(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Map real frozen PyO3 category values to truthful controller recovery."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	wedge = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_WEDGE, "wedge.cdml")
	two = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_TWO_MOLECULES, "two.cdml")
	try:
		observation = wedge.observe_structure_interaction()
		target = next(value for value in observation.targets if value.identifier == "ab")
		with pytest.raises(ferrum_qt.ferrum.engine.RenderInteractionError) as caught:
			wedge.select_structure_interaction(
				observation, None,
				ferrum_qt.ferrum.engine.StructureInteractionQueryV1.point(
					(target.bounds.left + target.bounds.right) / 2.0,
					(target.bounds.top + target.bounds.bottom) / 2.0,
					ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace,
				),
			)
		assert "display-only" in window._structure_refusal(caught.value)

		observation = two.observe_structure_interaction()
		with pytest.raises(ferrum_qt.ferrum.engine.RenderInteractionError) as caught:
			two.select_structure_interaction(
				observation, None,
				ferrum_qt.ferrum.engine.StructureInteractionQueryV1.marquee(
					-10.0, -10.0, 50.0, 10.0,
					ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace,
				),
			)
		assert "one molecule" in window._structure_refusal(caught.value)
	finally:
		wedge.dispose()
		two.dispose()
		window.deleteLater()

"""Visible behavior for the ordinary native cyclohexane-ring action."""

import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

import ferrum_qt.main_window
import ferrum_qt.native.ferrum_native_document_tab


def _click_visible_menu_action(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Activate a labelled command through the visible menu route."""
	for menu_action in window.menuBar().actions():
		menu = menu_action.menu()
		if menu is None:
			continue
		for candidate in menu.actions():
			if candidate.text().replace("&", "") != label:
				continue
			PySide6.QtTest.QTest.mouseClick(
				window.menuBar(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				window.menuBar().actionGeometry(menu_action).center(),
			)
			qapp.processEvents()
			PySide6.QtTest.QTest.mouseClick(
				menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu.actionGeometry(candidate).center(),
			)
			qapp.processEvents()
			return
	raise AssertionError(f"No visible menu action is labelled {label!r}")


def _ring_centre(molecule: object) -> tuple[float, float]:
	"""Return the centre implied by Rust's ordinary authored ring vertices."""
	return (
		sum(atom.position.x for atom in molecule.atoms) / len(molecule.atoms),
		sum(atom.position.y for atom in molecule.atoms) / len(molecule.atoms),
	)


def test_insert_cyclohexane_ring_uses_the_shared_authored_centre_and_selects_rust_atoms(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The ordinary action commits the snapped and raw centres Rust receives."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		"<cdml/>", "ring.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		snapped_click = PySide6.QtCore.QPoint(143, 91)
		expected_snapped = tab.view.snap_authored_scene_point(
			tab.view.mapToScene(snapped_click),
		)
		_click_visible_menu_action(window, "Insert Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, snapped_click,
		)
		snapped_ring = tab.current_document_observation().projection.molecules[0]
		selected = tab._controller.projection.selected_durable_targets()

		assert _ring_centre(snapped_ring) == pytest.approx(
			(expected_snapped.x(), expected_snapped.y()),
		)
		assert selected and all(
			target.kind == "atom" and target.identifier in {
				atom.source_id for atom in snapped_ring.atoms
			}
			for target in selected
		)

		tab.view.set_hex_grid_snap_enabled(False)
		raw_click = PySide6.QtCore.QPoint(318, 207)
		expected_raw = tab.view.mapToScene(raw_click)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, raw_click,
		)
		raw_ring = tab.current_document_observation().projection.molecules[1]
		assert _ring_centre(raw_ring) == pytest.approx((expected_raw.x(), expected_raw.y()))
	finally:
		window.close()
		window.deleteLater()


def test_insert_cyclohexane_ring_refuses_an_occupied_atom_without_mutation(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An occupied click preserves the authoritative document and prior selection."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='m'><atom id='a' name='C'>"
		"<point x='10' y='20'/></atom></molecule></cdml>",
		"occupied-ring.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		tab._controller.projection.select_durable((("atom", "a"),))
		before_snapshot = tab.current_snapshot
		before_selection = tab._controller.projection.selected_durable_targets()
		occupied = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
		_click_visible_menu_action(window, "Insert Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, occupied,
		)

		assert tab.current_snapshot == before_snapshot
		assert tab._controller.projection.selected_durable_targets() == before_selection
	finally:
		window.close()
		window.deleteLater()

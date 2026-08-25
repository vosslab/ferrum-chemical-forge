"""Visible behavior for the ordinary Ferrum cyclohexane-ring action."""

import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

import tests.ferrum_native_menu_actions
import ferrum_qt.main_window
import ferrum_qt.ferrum.document_tab


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
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "ring.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		snapped_click = PySide6.QtCore.QPoint(143, 91)
		expected_snapped = tab.view.snap_authored_scene_point(
			tab.view.mapToScene(snapped_click),
		)
		tests.ferrum_native_menu_actions.click_visible_menu_action(window, "Insert Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, snapped_click,
		)
		snapped_ring = tab.current_document_observation().projection.molecules[0]
		assert _ring_centre(snapped_ring) == pytest.approx(
			(expected_snapped.x(), expected_snapped.y()),
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
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='m'><atom id='a' name='C'>"
		"<point x='10' y='20'/></atom></molecule></cdml>",
		"occupied-ring.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		before_snapshot = tab.current_snapshot
		occupied = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
		tests.ferrum_native_menu_actions.click_visible_menu_action(window, "Insert Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, occupied,
		)

		assert tab.current_snapshot == before_snapshot
	finally:
		window.close()
		window.deleteLater()


def test_attach_cyclohexane_ring_drag_commits_and_escape_retires_pending_receipt(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The attach command is independent, paint-only until its one release commit."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='m'><atom id='a' name='C'>"
		"<point x='10' y='20'/></atom></molecule></cdml>", "attach-ring.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		anchor = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
		before = tab.current_snapshot
		tests.ferrum_native_menu_actions.click_visible_menu_action(window, "Attach Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), anchor + PySide6.QtCore.QPoint(80, 0))
		qapp.processEvents()
		assert tab.current_snapshot == before
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		qapp.processEvents()
		assert tab.current_snapshot == before

		tests.ferrum_native_menu_actions.click_visible_menu_action(window, "Attach Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), anchor + PySide6.QtCore.QPoint(80, 0))
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor + PySide6.QtCore.QPoint(80, 0),
		)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before.revision + 1
		assert len(tab.current_document_observation().projection.molecules[0].atoms) == 6
		undone = tab.undo().observation.snapshot
		assert undone.cdml == before.cdml
		assert not tab.is_dirty
	finally:
		window.close()
		window.deleteLater()

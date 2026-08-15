"""Visible native point-authoring behavior independent of the main window shell."""

import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

import ferrum_qt.main_window
import ferrum_qt.native.ferrum_native_document_tab


_EDITABLE_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  </molecule>
</cdml>"""


def _click_visible_menu_action(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Activate one labelled native command through the visible menu route."""
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


def test_add_atom_action_maps_one_view_click_to_the_rust_scene_point(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The public action captures a shared snapped point and selects Rust's atom."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	prior_choices = None
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		prior_choices = window._drawing_parameters.snapshot()
		window._drawing_parameters.set_element("O")
		click = PySide6.QtCore.QPoint(40, 55)
		expected = tab.view.snap_authored_scene_point(tab.view.mapToScene(click))
		_click_visible_menu_action(window, "Add Atom at Point", qapp)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, click,
		)
		selected = tab.selected_atom_projection()
		created = next(
			atom for atom in tab.current_document_observation().projection.molecules[0].atoms
			if atom.source_id == selected.source_id
		)
		assert (created.element, created.position.x, created.position.y) == (
			"O", expected.x(), expected.y(),
		)
	finally:
		if prior_choices is not None:
			window._drawing_parameters.set_element(prior_choices.element)
			window._drawing_parameters.set_order_name(prior_choices.order_name)
			window._drawing_parameters.set_presentation_name(prior_choices.presentation_name)
		window.close()
		window.deleteLater()


def test_unchecked_snap_control_keeps_new_atom_at_the_click_position(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Disabling point snapping preserves an ordinary authored click position."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "unsnapped-atom.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		click = PySide6.QtCore.QPoint(40, 55)
		expected = tab.view.mapToScene(click)
		tab.view.set_hex_grid_snap_enabled(False)
		_click_visible_menu_action(window, "Add Atom at Point", qapp)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, click,
		)
		selected = tab.selected_atom_projection()
		created = next(
			atom for atom in tab.current_document_observation().projection.molecules[0].atoms
			if atom.source_id == selected.source_id
		)
		assert (created.position.x, created.position.y) == (expected.x(), expected.y())
	finally:
		window.close()
		window.deleteLater()

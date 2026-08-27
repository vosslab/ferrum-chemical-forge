"""Durable selection-context action contracts for the native Ferrum canvas."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.themes.theme_loader
import ferrum_qt.actions.context_menu
import ferrum_qt.declarative_resources
import ferrum_qt.ferrum.document_tab


#============================================
#============================================
def _selected_atom_tab(
		qapp: PySide6.QtWidgets.QApplication, window: object, name: str,
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Return one active tab with its visible atom selected through canvas input."""
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='mol-1'><atom id='a1' "
		"name='C'><point x='10' y='10'/></atom></molecule></cdml>", name,
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	assert window._window_mode_sync.select_action(window._select_structure_action)
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 10.0)),
	)
	qapp.processEvents()
	return tab


#============================================
def _atoms_remain(tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab) -> bool:
	"""Return whether the test document still has a Rust-projected atom."""
	projection = tab.current_document_observation().projection
	return any(molecule.atoms for molecule in projection.molecules)


#============================================
def _open_context_menu(
		qapp: PySide6.QtWidgets.QApplication,
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		invocation: str,
		) -> PySide6.QtWidgets.QMenu:
	"""Open the selected canvas context client through one public input route."""
	viewport = tab.view.viewport()
	if invocation == "Menu":
		PySide6.QtTest.QTest.keyClick(viewport, PySide6.QtCore.Qt.Key.Key_Menu)
	elif invocation == "Shift+F10":
		PySide6.QtTest.QTest.keyClick(
			viewport,
			PySide6.QtCore.Qt.Key.Key_F10,
			PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier,
		)
	else:
		PySide6.QtTest.QTest.mouseClick(
			viewport,
			PySide6.QtCore.Qt.MouseButton.RightButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 10.0)),
		)
	qapp.processEvents()
	menu = PySide6.QtWidgets.QApplication.activePopupWidget()
	assert isinstance(menu, PySide6.QtWidgets.QMenu)
	return menu


#============================================
def test_context_menu_reuses_enabled_registry_actions_in_yaml_group_order(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Selected-structure clients retain action identity and YAML ordering."""
	tab = _selected_atom_tab(qapp, main_window, "context-order.cdml")
	registry = main_window._action_registry
	accessible_name, action_groups = ferrum_qt.declarative_resources.load_context_menu_placement(
		registry,
	)
	filtered = tuple(
		registry.get_qt_action(action_id)
		for group in action_groups
		for action_id in group
		if registry.get_qt_action(action_id).isEnabled()
	)
	menu = ferrum_qt.actions.context_menu.build_context_menu(
		tab.view.viewport(), registry, action_groups, accessible_name,
	)
	assert menu is not None
	menu_actions = tuple(action for action in menu.actions() if not action.isSeparator())
	assert tuple(id(action) for action in menu_actions) == tuple(id(action) for action in filtered)
	menu.deleteLater()


#============================================
@pytest.mark.parametrize("invocation", ("Menu", "Shift+F10", "right button", "Delete", "Backspace"))
def test_context_action_and_normalized_delete_keys_remove_the_same_selection(
		qapp: PySide6.QtWidgets.QApplication, main_window: object, invocation: str,
		) -> None:
	"""Context activation and both delete keys converge on Rust selection deletion."""
	tab = _selected_atom_tab(qapp, main_window, f"selection-{invocation}.cdml")
	if invocation in {"Menu", "Shift+F10", "right button"}:
		registry = main_window._action_registry
		menu = _open_context_menu(qapp, tab, invocation)
		delete_action = registry.get_qt_action("edit.delete_selection")
		assert delete_action in menu.actions()
		delete_action.trigger()
	else:
		key = getattr(PySide6.QtCore.Qt.Key, f"Key_{invocation}")
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), key)
	qapp.processEvents()
	assert not _atoms_remain(tab)


#============================================
@pytest.mark.parametrize("invocation", ("Menu", "Shift+F10"))
def test_keyboard_context_menu_close_restores_native_viewport_focus(
		qapp: PySide6.QtWidgets.QApplication, main_window: object, invocation: str,
		) -> None:
	"""Closing each keyboard context route deterministically returns focus to the canvas."""
	tab = _selected_atom_tab(qapp, main_window, f"context-focus-{invocation}.cdml")
	viewport = tab.view.viewport()
	menu = _open_context_menu(qapp, tab, invocation)
	PySide6.QtTest.QTest.keyClick(menu, PySide6.QtCore.Qt.Key.Key_Escape)
	qapp.processEvents()
	assert viewport.hasFocus()

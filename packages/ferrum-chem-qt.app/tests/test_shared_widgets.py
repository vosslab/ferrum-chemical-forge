"""Widget-level behavior tests for the backend-neutral Ferrum widget seams."""

# Standard Library
import types

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.widgets.periodic_table
import ferrum_qt.widgets.property_dock
import ferrum_qt.widgets.status_bar
import ferrum_qt.widgets.zoom_controls


#============================================
def _registry(parent: PySide6.QtWidgets.QWidget) -> object:
	"""Return live action clients sufficient for isolated widget testing."""
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	for action_id in (
		"view.zoom_in", "view.zoom_out",
			"view.reset_zoom", "view.zoom_page", "view.zoom_content",
			"edit.atom.properties", "edit.bond.properties",
		):
		action = PySide6.QtGui.QAction(action_id, parent)
		registry.register(ferrum_qt.actions.action_registry.MenuAction(
			action_id, action_id, action_id, None, action.trigger, action.isEnabled,
			"Widget test action",
		))
		registry.bind_qt_action(action_id, action)
	return registry


#============================================
def test_status_bar_preserves_context_after_transient_message(qapp: object) -> None:
	"""Transient feedback yields back to keyboard-workflow guidance."""
	del qapp
	bar = ferrum_qt.widgets.status_bar.StatusBar()
	bar.set_context_message("Draw a bond")
	bar.show_message("Bond created", 0)
	assert bar.visible_message == "Bond created"
	bar.clearMessage()
	assert bar.visible_message == "Draw a bond"
	bar.update_coords(2.5, -3.0)
	assert bar.findChild(PySide6.QtWidgets.QLabel, "cursor-coordinates").text() == "X: 2.5  Y: -3.0"


#============================================
def test_zoom_controls_reuse_actions_and_project_observed_zoom(qapp: object) -> None:
	"""A widget trigger activates the original QAction rather than a copy."""
	del qapp
	parent = PySide6.QtWidgets.QWidget()
	registry = _registry(parent)
	called: list[bool] = []
	zoom_in = registry.get_qt_action("view.zoom_in")
	zoom_in.triggered.connect(lambda: called.append(True))
	controls = ferrum_qt.widgets.zoom_controls.ZoomControls(registry, parent)
	button = next(button for button in controls.findChildren(PySide6.QtWidgets.QToolButton)
		if button.accessibleName() == "Zoom in")
	button.click()
	controls.update_zoom_display(153.2)
	assert called == [True]
	assert controls.findChild(PySide6.QtWidgets.QSlider).value() == 153


#============================================
def test_property_dock_projects_only_observation_dto(qapp: object) -> None:
	"""Selection display uses projection fields without holding a tab object."""
	del qapp
	parent = PySide6.QtWidgets.QMainWindow()
	registry = _registry(parent)
	dock = ferrum_qt.widgets.property_dock.PropertyDock(registry, parent)
	atom = types.SimpleNamespace(source_id="a1", element="O", formal_charge=-1,
		position=types.SimpleNamespace(x=1.0, y=2.0))
	document = types.SimpleNamespace(molecules=(types.SimpleNamespace(atoms=(atom,), bonds=()),),
		presentation_stack=types.SimpleNamespace(roots=(), issues=()), issues=())
	observation = types.SimpleNamespace(document=document,
		selection=(types.SimpleNamespace(kind="atom", identifier="a1"),))
	dock.refresh(observation)
	assert "Element: O" in dock.summary_text
	atom_button = next(button for button in dock.findChildren(PySide6.QtWidgets.QToolButton)
		if button.defaultAction() is registry.get_qt_action("edit.atom.properties"))
	assert not atom_button.isHidden()


#============================================
def test_periodic_table_emits_symbol_from_keyboard_reachable_button(qapp: object) -> None:
	"""The chooser returns UI intent without invoking chemistry behavior."""
	del qapp
	dialog = ferrum_qt.widgets.periodic_table.PeriodicTablePopup()
	selected: list[str] = []
	dialog.element_selected.connect(selected.append)
	button = dialog.findChild(PySide6.QtWidgets.QPushButton, "element-O")
	assert button.accessibleName() == "O, Oxygen"
	PySide6.QtTest.QTest.mouseClick(button, PySide6.QtCore.Qt.MouseButton.LeftButton)
	assert selected == ["O"]

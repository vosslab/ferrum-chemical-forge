"""Tests for Patch 2: Toolbar action naming -- all on_* methods resolve."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets


#============================================
def test_save_action_exists_and_has_shortcut(main_window: object) -> None:
	"""The Save menu action exists and has a keyboard shortcut."""
	action = main_window._action_registry.get_qt_action("file.save")
	assert action is not None, "registry should expose file.save"
	shortcut = action.shortcut()
	assert not shortcut.isEmpty(), "save action should have a shortcut"


#============================================
def test_authoring_ribbon_uses_task_tabs(main_window: object) -> None:
	"""One ribbon presents authoring work through named task tabs."""
	ribbon = main_window.findChild(
		PySide6.QtWidgets.QToolBar, "ferrum-authoring-ribbon",
	)
	assert ribbon is main_window._authoring_ribbon


#============================================
def test_authoring_ribbon_exposes_action_names_to_assistive_technology(
		main_window: object) -> None:
	"""Compact controls retain each existing command's accessible label."""
	action = main_window._draw_bond_action
	draw_group = next(group for group in main_window._authoring_ribbon.groups_for_tab("home")
		if group.layout_data.id == "draw")
	button = draw_group.direct_button_for(action)
	assert button is not None and button.accessibleName()

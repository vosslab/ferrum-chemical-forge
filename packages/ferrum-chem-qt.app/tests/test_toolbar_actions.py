"""Tests for Patch 2: Toolbar action naming -- all on_* methods resolve."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets


# Stable action IDs presented by the ordinary toolbar.
_TOOLBAR_ACTION_IDS = (
	"file.new", "file.open", "file.save",
	"edit.undo", "edit.redo", "edit.cut", "edit.copy", "edit.paste",
	"view.zoom_in", "view.zoom_out", "view.reset_zoom", "view.toggle_grid",
)


#============================================
def test_all_toolbar_actions_resolve(main_window: object) -> None:
	"""All toolbar commands resolve to existing live actions."""
	for action_id in _TOOLBAR_ACTION_IDS:
		action = main_window._action_registry.get_qt_action(action_id)
		assert action is not None, f"Toolbar action {action_id!r} should exist"
		assert callable(action.trigger), f"Toolbar action {action_id!r} should trigger"


#============================================
def test_save_action_exists_and_has_shortcut(main_window: object) -> None:
	"""The Save menu action exists and has a keyboard shortcut."""
	action = main_window._action_registry.get_qt_action("file.save")
	assert action is not None, "registry should expose file.save"
	assert hasattr(main_window, "_on_save"), "should have _on_save method"
	assert callable(main_window._on_save), "_on_save should be callable"
	# verify shortcut is set
	shortcut = action.shortcut()
	assert not shortcut.isEmpty(), "save action should have a shortcut"


#============================================
def test_authoring_ribbon_uses_compact_icon_controls(main_window: object) -> None:
	"""One ribbon keeps direct action clients dense without a third toolbar row."""
	ribbon = main_window.findChild(
		PySide6.QtWidgets.QToolBar, "ferrum-authoring-ribbon",
	)
	assert ribbon is main_window._authoring_ribbon
	assert len(main_window.findChildren(PySide6.QtWidgets.QToolBar)) == 1


#============================================
def test_authoring_ribbon_exposes_action_names_to_assistive_technology(
		main_window: object) -> None:
	"""Compact controls retain each existing command's accessible label."""
	buttons = [
		button for button in main_window._authoring_ribbon.findChildren(
			PySide6.QtWidgets.QToolButton,
		)
		if button.defaultAction() is not None
	]
	assert buttons, "ordinary ribbon should expose action buttons"
	assert all(button.accessibleName() for button in buttons)

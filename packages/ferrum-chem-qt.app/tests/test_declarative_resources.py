"""Tests for Ferrum's static menu and drawing-tool declarations."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.declarative_resources


_ACTION_ATTRIBUTES = (
	"_action_new", "_open_action", "_save_action", "_save_as_action",
	"_close_action", "_quit_action", "_undo_action", "_redo_action",
	"_cut_action", "_copy_action", "_paste_action", "_zoom_in_action",
	"_zoom_out_action", "_zoom_100_action", "_show_hex_grid_action",
	"_snap_hex_grid_action", "_add_atom_action", "_draw_bond_action",
	"_cancel_tool_action", "_preferences_action", "_about_action",
)


#============================================
def _registry() -> tuple[
		PySide6.QtWidgets.QMainWindow,
		ferrum_qt.actions.action_registry.ActionRegistry,
		]:
	"""Return a registry populated with the window's existing QActions."""
	window = PySide6.QtWidgets.QMainWindow()
	for attribute in _ACTION_ATTRIBUTES:
		setattr(window, attribute, PySide6.QtGui.QAction(attribute, window))
	return window, ferrum_qt.actions.action_registry.register_main_window_actions(window)


#============================================
def test_declarative_resources_resolve_to_current_action_vocabulary(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Every shipped nonoptional declaration names a current Ferrum action."""
	del qapp
	window, registry = _registry()
	ferrum_qt.declarative_resources.preflight_declarative_resources(registry)
	window.deleteLater()


#============================================
def test_menu_preflight_rejects_duplicate_menu_ids() -> None:
	"""Menu IDs are unique even before menu construction is wired."""
	data = {
		"menus": [
			{
				"name": "file", "label_key": "File", "help_key": "Files",
				"side": "left", "items": [{"action": "file.new"}],
			},
			{
				"name": "file", "label_key": "File", "help_key": "Files",
				"side": "left", "items": [{"action": "file.open"}],
			},
		],
	}
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="Duplicate menu ID",
		):
		ferrum_qt.declarative_resources._validate_menu_declarations(
			data, frozenset({"file.new", "file.open"}),
		)


#============================================
def test_menu_preflight_rejects_duplicate_action_ids() -> None:
	"""A static command appears once in the declarative menu hierarchy."""
	data = {
		"menus": [
			{
				"name": "file", "label_key": "File", "help_key": "Files",
				"side": "left", "items": [{"action": "file.new"}],
			},
			{
				"name": "tools", "label_key": "Tools", "help_key": "Tools",
				"side": "left", "items": [{"action": "file.new"}],
			},
		],
	}
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="Duplicate declared menu action ID",
		):
		ferrum_qt.declarative_resources._validate_menu_declarations(
			data, frozenset({"file.new"}),
		)


#============================================
def test_mode_preflight_rejects_unsupported_tool_action() -> None:
	"""Mode declarations cannot promise a tool absent from Ferrum's vocabulary."""
	data = {
		"toolbar_order": ["atom"],
		"modes": {
			"atom": {
				"label_key": "Atom", "help_key": "Draw atoms",
				"action": "mode.unavailable",
			},
		},
	}
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="unsupported tool action",
		):
		ferrum_qt.declarative_resources._validate_mode_declarations(
			data, frozenset({"mode.unavailable"}),
		)

"""Public portability contracts for the converged Ferrum frontend seams."""

# Standard Library
import dataclasses
import inspect

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.actions.context_menu
import ferrum_qt.actions.menu_builder
import ferrum_qt.actions.platform_menu
import ferrum_qt.config.keybindings
import ferrum_qt.declarative_resources
import ferrum_qt.dialogs.preferences_dialog
import ferrum_qt.dialogs.theme_chooser_dialog
import ferrum_qt.modes.base_mode
import ferrum_qt.modes.mode_manager
import ferrum_qt.widgets.mode_toolbar
import ferrum_qt.widgets.periodic_table
import ferrum_qt.widgets.property_dock
import ferrum_qt.widgets.status_bar
import ferrum_qt.widgets.zoom_controls


#============================================
def _parameter_names(callable_object: object) -> tuple[str, ...]:
	"""Return the public parameter names of one callable contract."""
	return tuple(inspect.signature(callable_object).parameters)


#============================================
def test_declarative_resources_publish_supported_menu_and_mode_ids() -> None:
	"""Resources retain the portable hierarchy while naming Ferrum commands."""
	menus = ferrum_qt.declarative_resources.load_menu_declarations()["menus"]
	modes = ferrum_qt.declarative_resources.load_mode_declarations()
	assert tuple(menu["name"] for menu in menus) == (
		"file", "edit", "view", "tools", "options", "help",
	)
	assert {
		item["action"]
		for menu in menus
		for item in menu["items"]
		if "action" in item
	} <= {
		"file.new", "file.open", "file.save", "file.save_as", "file.close",
		"file.quit", "edit.undo", "edit.redo", "edit.cut", "edit.copy",
		"edit.paste", "view.zoom_in", "view.zoom_out", "view.reset_zoom",
		"view.toggle_grid", "view.toggle_grid_snap", "mode.atom", "mode.draw",
		"tool.cancel", "options.preferences", "help.about",
	}
	assert tuple(modes["toolbar_order"]) == ("atom", "draw", "---", "cancel")
	assert {
		mode["action"] for mode in modes["modes"].values()
	} == {"mode.atom", "mode.draw", "tool.cancel"}


#============================================
def test_action_menu_and_keybinding_seams_keep_stable_public_shapes() -> None:
	"""Menu clients reuse declared IDs and binding APIs stay intentionally small."""
	assert _parameter_names(
		ferrum_qt.actions.action_registry.ActionRegistry.register,
	) == ("self", "action")
	assert _parameter_names(
		ferrum_qt.actions.menu_builder.build_declared_menus,
	) == ("window", "registry")
	assert _parameter_names(
		ferrum_qt.actions.context_menu.build_context_menu,
	) == ("parent", "registry", "action_ids")
	assert _parameter_names(
		ferrum_qt.actions.platform_menu.apply_platform_menu_roles,
	) == ("registry",)
	assert _parameter_names(
		ferrum_qt.config.keybindings.KeybindingManager.set_binding,
	) == ("self", "action_id", "text")
	assert _parameter_names(
		ferrum_qt.config.keybindings.KeybindingManager.reset_defaults,
	) == ("self",)
	assert set(ferrum_qt.config.keybindings.DEFAULT_KEYBINDINGS) >= {
		"mode.atom", "mode.draw", "tool.cancel",
	}


#============================================
def test_mode_seam_publishes_ids_and_document_free_lifecycle_contract() -> None:
	"""Mode APIs carry normalized intent, not a Python document/session model."""
	assert tuple(ferrum_qt.modes.base_mode.ModeId) == (
		ferrum_qt.modes.base_mode.ModeId.ATOM,
		ferrum_qt.modes.base_mode.ModeId.DRAW,
		ferrum_qt.modes.base_mode.ModeId.EDIT,
		ferrum_qt.modes.base_mode.ModeId.ARROW,
		ferrum_qt.modes.base_mode.ModeId.VECTOR,
		ferrum_qt.modes.base_mode.ModeId.BRACKET,
	)
	assert tuple(field.name for field in dataclasses.fields(
		ferrum_qt.modes.base_mode.ModeContext,
	)) == ("observation", "dispatch_context")
	assert _parameter_names(
		ferrum_qt.modes.mode_manager.ModeManager.activate,
	) == ("self", "mode_id", "context")
	assert _parameter_names(
		ferrum_qt.modes.mode_manager.ModeManager.handle_pointer,
	) == ("self", "pointer", "context")


#============================================
def test_shared_widget_and_dialog_clients_expose_focused_boundaries() -> None:
	"""View clients and dialogs accept projection or intent, never ownership."""
	assert _parameter_names(ferrum_qt.widgets.mode_toolbar.ModeToolbar) == (
		"registry", "parent", "compact_breakpoint",
	)
	assert _parameter_names(ferrum_qt.widgets.property_dock.PropertyDock) == (
		"registry", "parent",
	)
	assert _parameter_names(ferrum_qt.widgets.status_bar.StatusBar) == ("parent",)
	assert _parameter_names(ferrum_qt.widgets.zoom_controls.ZoomControls) == (
		"registry", "parent",
	)
	assert _parameter_names(ferrum_qt.widgets.periodic_table.PeriodicTablePopup) == (
		"parent",
	)
	assert tuple(field.name for field in dataclasses.fields(
		ferrum_qt.dialogs.preferences_dialog.PreferencesDialogResult,
	)) == (
		"theme", "remember_workspace", "show_hex_grid",
		"snap_authored_points_to_hex_grid",
	)
	assert _parameter_names(
		ferrum_qt.dialogs.preferences_dialog.PreferencesDialog.choose_preferences,
	) == ("current", "parent")
	assert _parameter_names(
		ferrum_qt.dialogs.theme_chooser_dialog.ThemeChooserDialog.choose_theme,
	) == ("current_theme", "parent")

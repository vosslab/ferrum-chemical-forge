"""Public portability contracts for the converged Ferrum frontend seams."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.actions.context_menu
import ferrum_qt.actions.menu_builder
import ferrum_qt.actions.platform_menu
import ferrum_qt.config.keybindings
import ferrum_qt.declarative_resource_loader
import ferrum_qt.declarative_resource_preflight
import ferrum_qt.dialogs.preferences_dialog
import ferrum_qt.dialogs.theme_chooser_dialog
import ferrum_qt.modes.base_mode
import ferrum_qt.modes.mode_manager
import ferrum_qt.widgets.periodic_table
import ferrum_qt.widgets.property_dock
import ferrum_qt.widgets.status_bar
import ferrum_qt.widgets.zoom_controls


#============================================
def test_action_menu_and_keybinding_seams_keep_stable_public_shapes() -> None:
	"""Menu clients bind existing actions through the explicit registry seam."""
	assert callable(ferrum_qt.actions.action_registry.ActionRegistry.register_existing)
	assert callable(ferrum_qt.actions.menu_builder.build_declared_menus)
	assert callable(ferrum_qt.declarative_resource_loader.load_packaged_yaml)
	assert callable(ferrum_qt.declarative_resource_preflight.preflight_window_resources)


#============================================
def test_real_window_places_reused_draw_actions_once_in_declarative_menu_tree(
		main_window: object,
		) -> None:
	"""Draw's canonical menu clients are the same actions used by Ferrum tools."""
	draw_menu = main_window._declared_menus["draw"]
	registry = main_window._action_registry
	bond_action = registry.get_qt_action("draw.bond")
	ring_action = registry.get_qt_action("draw.ring.regular.c6")
	transform_action = registry.get_qt_action("draw.transform.roots.align_left")
	child_menus = draw_menu.findChildren(PySide6.QtWidgets.QMenu)
	menus = [draw_menu, *child_menus]
	assert sum(bond_action in menu.actions() for menu in menus) == 1
	assert sum(ring_action in menu.actions() for menu in menus) == 1
	assert sum(transform_action in menu.actions() for menu in menus) == 1
	ribbon_button = next(
		button for button in main_window._authoring_ribbon.findChildren(
			PySide6.QtWidgets.QToolButton,
		)
		if button.defaultAction() is bond_action
	)
	assert ribbon_button.isCheckable() is bond_action.isCheckable()


#============================================
def test_mode_seam_keeps_document_free_lifecycle_contract() -> None:
	"""Mode APIs carry normalized intent, not a Python document/session model."""
	context = ferrum_qt.modes.base_mode.ModeContext(None, object())
	manager = ferrum_qt.modes.mode_manager.ModeManager(lambda _context, _intent: None)
	manager.activate(ferrum_qt.modes.base_mode.ModeId.DRAW, context)
	assert manager.active_mode_id is ferrum_qt.modes.base_mode.ModeId.DRAW


#============================================
def test_shared_widget_and_dialog_clients_expose_focused_boundaries() -> None:
	"""View clients and dialogs accept projection or intent, never ownership."""
	assert callable(ferrum_qt.widgets.property_dock.PropertyDock)
	assert callable(ferrum_qt.widgets.status_bar.StatusBar)
	assert callable(ferrum_qt.widgets.zoom_controls.ZoomControls)
	assert callable(ferrum_qt.widgets.periodic_table.PeriodicTablePopup)
	assert callable(ferrum_qt.dialogs.preferences_dialog.PreferencesDialog.choose_preferences)
	assert callable(ferrum_qt.dialogs.theme_chooser_dialog.ThemeChooserDialog.choose_theme)

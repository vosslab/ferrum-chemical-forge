"""Semantic behavior coverage for Ferrum's grouped labelled authoring ribbon."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.declarative_resources
import ferrum_qt.ferrum.authoring_ribbon_layout
import ferrum_qt.ribbon_contract
import ferrum_qt.widgets.ribbon_group


#============================================
def test_authoring_ribbon_uses_named_task_groups_and_live_actions(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Reactions begins with icon-first notation tools backed by registry actions."""
	main_window.show()
	qapp.processEvents()
	ribbon = main_window._authoring_ribbon
	group = _group(ribbon, "reactions", "reaction_notation")
	action = main_window._action_registry.get_qt_action("draw.arrow")
	button = group.direct_button_for(action)
	assert button.defaultAction() is action
	assert button.toolButtonStyle() is PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonIconOnly
	assert button.accessibleDescription()


#============================================
def test_ribbon_header_reuses_accessible_icon_bearing_registry_actions(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Persistent header clients expose the same preflighted QAction identities."""
	main_window.show()
	qapp.processEvents()
	ribbon = main_window._authoring_ribbon
	save_action = main_window._action_registry.get_qt_action("file.save")
	command_palette_action = main_window._action_registry.get_qt_action("view.command_palette")
	header_buttons = ribbon._header.findChildren(PySide6.QtWidgets.QToolButton)
	save_button = next(button for button in header_buttons
		if button.defaultAction() is save_action)
	command_palette_button = next(button for button in header_buttons
		if button.defaultAction() is command_palette_action)
	assert not save_action.icon().isNull()
	assert save_button.property("ribbonHeaderRole") == "quick"
	assert save_button.accessibleName() == save_action.text()
	assert save_button.focusPolicy() is PySide6.QtCore.Qt.FocusPolicy.StrongFocus
	assert command_palette_button.property("ribbonHeaderRole") == "global"
	assert command_palette_button.accessibleDescription()
	assert ribbon.current_tab_id() == "reactions"


#============================================
def test_ribbon_resource_icons_refresh_on_live_theme_change(
		main_window: object, theme_manager: object,
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A theme transition refreshes icons on the shared live command actions."""
	bond_action = main_window._action_registry.get_qt_action("draw.bond")
	light_icon_key = bond_action.icon().cacheKey()
	theme_manager.apply_transient_theme("dark")
	qapp.processEvents()
	assert not bond_action.icon().isNull()
	assert bond_action.icon().cacheKey() != light_icon_key


#============================================
def test_ribbon_layout_resolves_existing_action_identity(qapp: object) -> None:
	"""Layout declarations resolve the supplied registry QAction without a copy."""
	parent = PySide6.QtWidgets.QWidget()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	action = _register_action(registry, parent, "draw.bond")
	_register_action(registry, parent, "file.new")
	_register_action(registry, parent, "view.command_palette")
	layout = ferrum_qt.ferrum.authoring_ribbon_layout._resolve_layout({
		"quick_access": ["file.new"],
		"global_actions": ["view.command_palette"],
		"tabs": [{
		"id": "home", "label_key": "Home", "groups": [{
			"id": "draw", "label_key": "Draw", "overflow_label_key": "More drawing tools",
			"accent": "drawing",
			"entries": [{
				"action": "draw.bond", "role": "primary", "priority": "normal",
				"presentation": "compact",
			}],
		}],
	}]}, registry)
	home_tab = next(tab for tab in layout.tabs if tab.id == "home")
	draw_group = next(group for group in home_tab.groups if group.id == "draw")
	entry = next(entry for entry in draw_group.entries if entry.action_id == "draw.bond")
	assert entry.action is action
	assert (entry.role, entry.priority, entry.presentation) == ("primary", "normal", "compact")


#============================================
def test_ribbon_layout_rejects_unknown_required_action(qapp: object) -> None:
	"""A bad layout fails before a ribbon widget can mutate the visible window."""
	parent = PySide6.QtWidgets.QWidget()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	_register_action(registry, parent, "file.new")
	_register_action(registry, parent, "view.command_palette")
	with pytest.raises(ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="unbound QAction 'draw.unknown'"):
		ferrum_qt.ferrum.authoring_ribbon_layout._resolve_layout({
			"quick_access": ["file.new"],
			"global_actions": ["view.command_palette"],
			"tabs": [{
			"id": "home", "label_key": "Home", "groups": [{
				"id": "draw", "label_key": "Draw", "overflow_label_key": "More drawing tools",
				"accent": "drawing",
				"entries": [{
					"action": "draw.unknown", "role": "primary", "priority": "required",
					"presentation": "compact",
				}],
			}],
		}]}, registry)


#============================================
def test_group_local_more_reuses_disabled_checked_action(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A supporting action retains owner state in its direct and More-menu clients."""
	parent = PySide6.QtWidgets.QWidget()
	action = PySide6.QtGui.QAction("Supporting command", parent)
	action.setToolTip("Use the supporting command")
	action.setCheckable(True)
	entry = ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry(
		"draw.supporting", "supporting", "normal", "compact", action,
	)
	layout = ferrum_qt.ferrum.authoring_ribbon_layout.RibbonGroupLayout(
		"draw", "Draw", "More drawing tools", "drawing", (entry,),
	)
	group = ferrum_qt.widgets.ribbon_group.RibbonGroup(layout, parent)
	group.resize(20, 100)
	parent.show()
	qapp.processEvents()
	action.setChecked(True)
	action.setEnabled(False)
	qapp.processEvents()
	button = group.direct_button_for(action)
	assert button.defaultAction() is action
	assert not button.isEnabled()
	group.set_display_state(ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.COLLAPSED)
	assert action in group._more_button.menu().actions()


#============================================
def test_group_states_keep_each_original_action_reachable_once(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Expanded, compact, and collapsed routes preserve registry QAction identity."""
	parent = PySide6.QtWidgets.QWidget()
	actions = tuple(PySide6.QtGui.QAction(f"Command {index}", parent) for index in range(3))
	entries = (
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry(
			"draw.primary", "primary", "required", "standard", actions[0],
		),
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry(
			"draw.supporting_one", "supporting", "normal", "compact", actions[1],
		),
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry(
			"draw.supporting_two", "supporting", "normal", "compact", actions[2],
		),
	)
	group = ferrum_qt.widgets.ribbon_group.RibbonGroup(
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonGroupLayout(
			"draw", "Draw", "More drawing tools", "drawing", entries,
		), parent,
	)
	group.resize(500, 100)
	group.show()
	parent.show()
	try:
		for state in ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState:
			group.set_display_state(state)
			qapp.processEvents()
			assert group.visible_actions() == actions
			assert len(set(group.visible_actions())) == len(actions)
		assert group.width_for(ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.EXPANDED) > (
			group.width_for(ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.COMPACT)
		) > group.width_for(ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.COLLAPSED)
	finally:
		parent.close()


#============================================
def test_group_controls_follow_declared_presentations_in_a_two_row_grid(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Presentation controls geometry independently from role and overflow priority."""
	parent = PySide6.QtWidgets.QWidget()
	actions = tuple(PySide6.QtGui.QAction(label, parent) for label in (
		"Primary command", "Short", "A longer supporting command", "Last command",
	))
	entries = (
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry(
			"draw.command_0", "primary", "required", "compact", actions[0],
		),
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry(
			"draw.command_1", "supporting", "normal", "compact", actions[1],
		),
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry(
			"draw.command_2", "supporting", "normal", "standard", actions[2],
		),
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry(
			"draw.command_3", "supporting", "normal", "large", actions[3],
		),
	)
	group = ferrum_qt.widgets.ribbon_group.RibbonGroup(
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonGroupLayout(
			"draw", "Draw", "More drawing tools", "drawing", entries,
		), parent,
	)
	parent.show()
	group.show()
	qapp.processEvents()
	try:
		compact = group.direct_button_for(actions[0])
		standard = group.direct_button_for(actions[2])
		large = group.direct_button_for(actions[3])
		assert compact.size().width() == compact.size().height()
		assert standard.width() > compact.width() and standard.height() < compact.height()
		assert large.height() > compact.height()
		assert compact.toolButtonStyle() is PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonIconOnly
		assert standard.toolButtonStyle() is PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextBesideIcon
		assert large.toolButtonStyle() is PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextUnderIcon
		assert group._action_layout.rowCount() == 2
		group.set_display_state(ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.COLLAPSED)
		assert group._more_button.text() == ""
		assert group._more_button.accessibleName() == "More drawing tools"
	finally:
		parent.close()


#============================================
def test_narrow_task_page_reduces_groups_without_losing_commands(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A constrained real tab routes commands through its deterministic compact clients."""
	main_window.show()
	ribbon = main_window._authoring_ribbon
	ribbon.select_tab("structure")
	page = ribbon._pages.currentWidget()
	page.resize(420, page.sizeHint().height())
	page.reallocate()
	qapp.processEvents()
	groups = ribbon.groups_for_tab("structure")
	assert any(group.display_state is not (
		ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.EXPANDED
	) for group in groups)
	for group in groups:
		assert group.visible_actions() == tuple(entry.action for entry in group.layout_data.entries)


#============================================
def test_group_focus_moves_to_exposed_popup_when_direct_client_hides(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A resize-state transition keeps keyboard focus on an exposed group client."""
	parent = PySide6.QtWidgets.QWidget()
	primary = PySide6.QtGui.QAction("Primary", parent)
	supporting = PySide6.QtGui.QAction("Supporting", parent)
	entries = (
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry(
			"draw.primary", "primary", "required", "compact", primary,
		),
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry(
			"draw.supporting", "supporting", "normal", "compact", supporting,
		),
	)
	group = ferrum_qt.widgets.ribbon_group.RibbonGroup(
		ferrum_qt.ferrum.authoring_ribbon_layout.RibbonGroupLayout(
			"draw", "Draw", "More drawing tools", "drawing", entries,
		), parent,
	)
	group.resize(400, 100)
	group.show()
	parent.show()
	try:
		supporting_button = group.direct_button_for(supporting)
		primary_button = group.direct_button_for(primary)
		assert supporting_button is not None and primary_button is not None
		supporting_button.setFocus()
		qapp.processEvents()
		group.set_display_state(ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.COMPACT)
		assert group._more_button.hasFocus()
		primary_button.setFocus()
		qapp.processEvents()
		group.set_display_state(ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.COLLAPSED)
		assert group._more_button.hasFocus()
	finally:
		parent.close()


#============================================
def test_ribbon_rejects_duplicate_same_tab_action_placement(qapp: object) -> None:
	"""One tab cannot create competing direct clients for the same action."""
	parent = PySide6.QtWidgets.QWidget()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	_register_action(registry, parent, "draw.bond")
	_register_action(registry, parent, "file.new")
	_register_action(registry, parent, "view.command_palette")
	data = {
		"quick_access": ["file.new"],
		"global_actions": ["view.command_palette"],
		"tabs": [{"id": "home", "label_key": "Home", "groups": [
		{"id": "one", "label_key": "One", "overflow_label_key": "More one", "entries": [
			{"action": "draw.bond", "role": "primary", "priority": "required", "presentation": "compact"},
		], "accent": "drawing"},
		{"id": "two", "label_key": "Two", "overflow_label_key": "More two", "entries": [
			{"action": "draw.bond", "role": "supporting", "priority": "normal", "presentation": "compact"},
		], "accent": "drawing"},
	]}]}
	with pytest.raises(ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="Duplicate ribbon action 'draw.bond'"):
		ferrum_qt.ferrum.authoring_ribbon_layout._resolve_layout(data, registry)
	parent.close()
	parent.deleteLater()


#============================================
def test_bond_survives_tab_switch_then_escape_cancels_owner_tool(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A real ribbon tab switch does not interfere with owner-managed cancellation."""
	main_window.show()
	qapp.processEvents()
	bond = main_window._draw_bond_action
	bond.trigger()
	qapp.processEvents()
	ribbon = main_window._authoring_ribbon
	ribbon.select_tab("structure")
	qapp.processEvents()
	assert bond.isChecked()
	PySide6.QtTest.QTest.keyClick(main_window, PySide6.QtCore.Qt.Key.Key_Escape)
	qapp.processEvents()
	assert not bond.isChecked()
	assert main_window._window_mode_sync.active_state.mode_id is None


#============================================
def test_attached_compact_group_action_uses_declared_draw_and_structure_clients(
		main_window: object) -> None:
	"""The public compact-group route reuses its registered action in both clients."""
	action = main_window._attach_compact_group_action
	assert (
		action is main_window._action_registry.get_qt_action("chemistry.compact_group.attach")
		and action in main_window._draw_menu.actions()
	)
	group = _group(main_window._authoring_ribbon, "structure", "groups_templates")
	assert group.direct_button_for(action).defaultAction() is action
#============================================
def _group(ribbon: object, tab_id: str, group_id: str) -> object:
	"""Return one YAML-identified visible group without a widget-tree snapshot."""
	return next(group for group in ribbon.groups_for_tab(tab_id)
		if group.layout_data.id == group_id)


#============================================
def _register_action(registry: object, parent: PySide6.QtWidgets.QWidget,
		action_id: str) -> PySide6.QtGui.QAction:
	"""Add one complete live QAction to an isolated ActionRegistry."""
	action = PySide6.QtGui.QAction(action_id, parent)
	action.setToolTip(action_id)
	registry.register_existing(action_id, action, shortcut_exemption_reason="Test action")
	return action

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
import ferrum_qt.widgets.ribbon_group


#============================================
def test_authoring_ribbon_uses_named_task_groups_and_live_actions(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Home presents recognisable task groups backed by the existing registry actions."""
	main_window.show()
	qapp.processEvents()
	ribbon = main_window._authoring_ribbon
	draw_group = _group(ribbon, "home", "draw")
	assert draw_group.accessibleName()
	assert draw_group.direct_button_for(main_window._draw_bond_action).defaultAction() is (
		main_window._draw_bond_action
	)
	assert draw_group.direct_button_for(main_window._draw_bond_action).text()


#============================================
def test_ribbon_layout_resolves_existing_action_identity(qapp: object) -> None:
	"""Layout declarations resolve the supplied registry QAction without a copy."""
	parent = PySide6.QtWidgets.QWidget()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	action = _register_action(registry, parent, "draw.bond")
	layout = ferrum_qt.ferrum.authoring_ribbon_layout._resolve_layout({"tabs": [{
		"id": "home", "label_key": "Home", "groups": [{
			"id": "draw", "label_key": "Draw", "overflow_label_key": "More drawing tools",
			"entries": [{"action": "draw.bond", "role": "primary", "priority": "required"}],
		}],
	}]}, registry)
	home_tab = next(tab for tab in layout if tab.id == "home")
	draw_group = next(group for group in home_tab.groups if group.id == "draw")
	entry = next(entry for entry in draw_group.entries if entry.action_id == "draw.bond")
	assert entry.action is action


#============================================
def test_ribbon_layout_rejects_unknown_required_action(qapp: object) -> None:
	"""A bad layout fails before a ribbon widget can mutate the visible window."""
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	with pytest.raises(ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="unbound QAction 'draw.unknown'"):
		ferrum_qt.ferrum.authoring_ribbon_layout._resolve_layout({"tabs": [{
			"id": "home", "label_key": "Home", "groups": [{
				"id": "draw", "label_key": "Draw", "overflow_label_key": "More drawing tools",
				"entries": [{"action": "draw.unknown", "role": "primary", "priority": "required"}],
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
		"draw.supporting", "supporting", "normal", action,
	)
	layout = ferrum_qt.ferrum.authoring_ribbon_layout.RibbonGroupLayout(
		"draw", "Draw", "More drawing tools", (entry,),
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
	assert action in group._more_button.menu().actions()


#============================================
def test_ribbon_rejects_duplicate_same_tab_action_placement(qapp: object) -> None:
	"""One tab cannot create competing direct clients for the same action."""
	parent = PySide6.QtWidgets.QWidget()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	_register_action(registry, parent, "draw.bond")
	data = {"tabs": [{"id": "home", "label_key": "Home", "groups": [
		{"id": "one", "label_key": "One", "overflow_label_key": "More one", "entries": [
			{"action": "draw.bond", "role": "primary", "priority": "required"},
		]},
		{"id": "two", "label_key": "Two", "overflow_label_key": "More two", "entries": [
			{"action": "draw.bond", "role": "supporting", "priority": "normal"},
		]},
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
	structure_group = _group(ribbon, "structure", "rings")
	ribbon._tabs.setCurrentWidget(structure_group.parentWidget())
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

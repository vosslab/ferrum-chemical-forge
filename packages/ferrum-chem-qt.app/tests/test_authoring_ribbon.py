"""Focused behavior coverage for the compact Ferrum authoring ribbon."""

# Standard Library
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.io.user_template_catalog


_USER_TEMPLATE = (
	'<cdml xmlns="urn:ferrum:cdml" version="26.07"><standard line_width="9"/><paper id="paper"/>\n'
	'<molecule id="source" name="Ribbon template">\n'
	' <atom id="a" name="C"><point x="0" y="0"/></atom>\n'
	' <atom id="b" name="O"><point x="10" y="0"/></atom>\n'
	' <bond id="ab" start="a" end="b" type="n1"/>\n'
	'</molecule></cdml>\n'
)


#============================================
def test_authoring_ribbon_uses_two_rows_and_reuses_live_actions(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Desktop ribbon has two purposeful rows backed by the window actions."""
	main_window.resize(1280, 800)
	main_window.show()
	qapp.processEvents()
	ribbon = main_window._authoring_ribbon
	assert ribbon._content.layout().count() == 2
	assert ribbon._command_buttons[0].defaultAction() is main_window._action_new
	assert ribbon._content.layout().count() == 2
	assert ribbon._command_row.geometry().bottom() < ribbon._tool_row.geometry().top()
	assert ribbon._drawing_parameters_client.isHidden()
	assert not ribbon._more_button.isVisible()
	assert not ribbon._more_tools_button.isVisible()
	assert _button_for_action(ribbon, main_window._show_hex_grid_action).isVisible()
	assert _button_for_action(ribbon, main_window._snap_hex_grid_action).isVisible()
	extension = ribbon.findChild(PySide6.QtWidgets.QToolButton, "qt_toolbar_ext_button")
	assert extension is None or extension.isHidden()


#============================================
def test_authoring_ribbon_checked_tool_changes_context(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Real action switches leave the incoming bracket gesture armed."""
	main_window.resize(1280, 800)
	main_window.show()
	qapp.processEvents()
	ribbon = main_window._authoring_ribbon
	main_window._draw_bond_action.trigger()
	qapp.processEvents()
	assert main_window._line_gesture_intent is not None
	assert ribbon._drawing_parameters_client.isVisible()
	main_window._draw_bracket_action.trigger()
	qapp.processEvents()
	assert main_window._draw_bracket_action.isChecked()
	assert not main_window._draw_bond_action.isChecked()
	assert main_window._line_gesture_intent is not None
	PySide6.QtTest.QTest.keyClick(
		main_window._line_gesture_intent.viewport,
		PySide6.QtCore.Qt.Key.Key_Escape,
	)
	qapp.processEvents()
	assert main_window._line_gesture_intent is None
	assert not main_window._draw_bracket_action.isChecked()


#============================================
def test_authoring_ribbon_programmatic_tool_state_is_exclusive(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Programmatic mode synchronization cannot leave conflicting tool clients."""
	ribbon = main_window._authoring_ribbon
	main_window._draw_bond_action.setChecked(True)
	qapp.processEvents()
	main_window._draw_round_bracket_action.setChecked(True)
	qapp.processEvents()
	assert main_window._draw_round_bracket_action.isChecked()
	assert not main_window._draw_bond_action.isChecked()
	assert ribbon._tool_action_group.checkedAction() is main_window._draw_round_bracket_action
	main_window._draw_round_bracket_action.setChecked(False)
	qapp.processEvents()
	assert ribbon._tool_action_group.checkedAction() is None


#============================================
def test_authoring_ribbon_has_named_more_route_and_accessible_controls(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Narrow width keeps each compact command and tool reachable by name."""
	main_window.resize(1024, 800)
	main_window.show()
	qapp.processEvents()
	ribbon = main_window._authoring_ribbon
	assert ribbon._more_button.isVisible()
	assert ribbon._more_button.menu() is not None
	assert ribbon._more_button.menu().actions()[0] is main_window._cut_action
	assert ribbon._more_button.accessibleName()
	assert ribbon._more_tools_button.isVisible()
	assert ribbon._more_tools_button.accessibleName() == "More tools"
	assert ribbon._more_tools_button.menu() is not None
	more_tools = ribbon._more_tools_button.menu().actions()
	assert main_window._select_structure_action in more_tools
	assert all(action in more_tools for action in main_window._draw_vector_actions.values())
	assert main_window._place_user_template_action in more_tools
	assert more_tools[0] is main_window._add_atom_action
	assert _button_for_action(ribbon, main_window._select_structure_action).isHidden()
	assert _button_for_action(ribbon, main_window._place_user_template_action).isHidden()
	assert _button_for_action(ribbon, main_window._show_hex_grid_action).isVisible()
	assert _button_for_action(ribbon, main_window._snap_hex_grid_action).isVisible()
	extension = ribbon.findChild(PySide6.QtWidgets.QToolButton, "qt_toolbar_ext_button")
	assert extension is None or extension.isHidden()
	assert all(button.accessibleName() for button in ribbon._tool_buttons)


#============================================
def test_authoring_ribbon_assigns_visible_icons_to_selection_and_vector_actions(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Every direct compact action client retains purposeful Ferrum artwork."""
	main_window.resize(1280, 800)
	main_window.show()
	qapp.processEvents()
	ribbon = main_window._authoring_ribbon
	actions = (
		main_window._select_structure_action,
		*main_window._draw_vector_actions.values(),
	)
	for action in actions:
		button = _button_for_action(ribbon, action)
		assert not action.icon().isNull()
		assert button.defaultAction() is action
		assert button.accessibleName() == action.text()
		assert button.toolTip() == action.toolTip()


#============================================
def test_authoring_ribbon_uses_compact_accessible_bond_default_instruction(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Narrow contextual defaults preserve their full instruction for assistive UI."""
	main_window.resize(1024, 800)
	main_window.show()
	main_window._draw_bond_action.trigger()
	qapp.processEvents()
	instruction = main_window._authoring_ribbon._context_instruction
	assert instruction.text() == "Next atom/bond defaults."
	assert instruction.accessibleDescription() == (
		"Drawing defaults apply to the next atom or bond."
	)
	assert instruction.width() >= instruction.fontMetrics().horizontalAdvance(
		instruction.text(),
	)


#============================================
def test_authoring_ribbon_more_tools_reuses_the_live_tool_lifecycle(
		main_window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A compact-menu action switches the live canvas gesture, not just its mark."""
	main_window.resize(1024, 800)
	main_window.show()
	qapp.processEvents()
	ribbon = main_window._authoring_ribbon
	main_window._draw_bond_action.trigger()
	qapp.processEvents()
	more_tools = ribbon._more_tools_button.menu()
	assert more_tools is not None
	more_tools.actions().index(main_window._draw_bracket_action)
	main_window._draw_bracket_action.trigger()
	qapp.processEvents()
	assert main_window._draw_bracket_action.isChecked()
	assert main_window._line_gesture_intent is not None
	PySide6.QtTest.QTest.keyClick(
		main_window._line_gesture_intent.viewport,
		PySide6.QtCore.Qt.Key.Key_Escape,
	)
	qapp.processEvents()
	assert main_window._line_gesture_intent is None


#============================================
def test_authoring_ribbon_retires_template_owner_before_direct_and_more_tools(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path) -> None:
	"""Template placement cannot consume canvas events after another tool wins."""
	directory = tmp_path / "templates"
	directory.mkdir()
	(directory / "ribbon-template.cdml").write_text(_USER_TEMPLATE, encoding="utf-8")
	# Keep the interaction test free of the catalog refresh failure dialog while
	# retaining the production Rust-backed catalog admission path.
	main_window._user_template_catalog = (
		ferrum_qt.io.user_template_catalog.scan_user_template_catalog(directory)
	)
	entry = main_window.user_template_catalog.entries[0]
	main_window.resize(1280, 800)
	main_window.show()
	qapp.processEvents()

	# A direct Draw Bond selection must terminally retire the template owner.
	assert main_window.start_user_template_placement(entry.catalog_key)
	main_window._draw_bond_action.trigger()
	qapp.processEvents()
	assert main_window._user_template_placement_intent is None
	assert main_window._line_gesture_intent is not None
	PySide6.QtTest.QTest.keyClick(
		main_window._line_gesture_intent.viewport, PySide6.QtCore.Qt.Key.Key_Escape,
	)
	qapp.processEvents()
	assert main_window._line_gesture_intent is None

	# The shared More Tools QAction must make the same handoff before an Arrow drag.
	main_window.resize(1024, 800)
	qapp.processEvents()
	ribbon = main_window._authoring_ribbon
	more_tools = ribbon._more_tools_button.menu()
	assert more_tools is not None
	assert main_window.start_user_template_placement(entry.catalog_key)
	more_tools.actions()[more_tools.actions().index(main_window._draw_arrow_action)].trigger()
	qapp.processEvents()
	assert main_window._user_template_placement_intent is None
	intent = main_window._line_gesture_intent
	assert intent is not None
	PySide6.QtTest.QTest.keyClick(intent.viewport, PySide6.QtCore.Qt.Key.Key_Escape)
	qapp.processEvents()
	assert main_window._line_gesture_intent is None
	assert main_window._user_template_placement_intent is None


#============================================
def _button_for_action(
		ribbon: PySide6.QtWidgets.QToolBar, action: PySide6.QtGui.QAction,
		) -> PySide6.QtWidgets.QToolButton:
	"""Return the direct ribbon client for one existing live action."""
	for button in ribbon.findChildren(PySide6.QtWidgets.QToolButton):
		if button.defaultAction() is action:
			return button
	raise AssertionError(f"Ribbon has no direct client for {action.text()!r}")

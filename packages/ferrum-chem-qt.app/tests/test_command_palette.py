"""Focused behavior coverage for the registry-backed Ferrum command palette."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.actions.command_palette
import ferrum_qt.declarative_resources
import ferrum_qt.main_window


#============================================
def _action(
		registry: ferrum_qt.actions.action_registry.ActionRegistry,
		parent: PySide6.QtWidgets.QWidget, action_id: str, label: str,
		help_text: str, enabled: bool = True,
		) -> PySide6.QtGui.QAction:
	"""Register one feature-owned QAction with ordinary visible metadata."""
	action = PySide6.QtGui.QAction(label, parent)
	action.setToolTip(help_text)
	action.setEnabled(enabled)
	registry.register_existing(
		action_id, action,
		shortcut_exemption_reason="The command is reachable by its labelled menu.",
	)
	return action


#============================================
def _reaction_placements() -> dict[str, tuple[str, ...]]:
	"""Return one parsed miniature declaration projection for palette coverage."""
	menu_data = {
		"contexts": [{
			"id": "selected_structure", "accessible_name": "Selected structure actions",
			"groups": [{"id": "actions", "actions": ["chemistry.reaction.create"]}],
		}],
		"menus": [{
			"id": "chemistry", "label_key": "Chemistry", "help_key": "Chemistry commands",
			"items": [{"section": {
				"id": "reactions", "label_key": "Reactions",
				"items": [{"action": "chemistry.reaction.create"}],
			}}],
		}],
	}
	ribbon_data = {
		"quick_access": ["file.new"],
		"global_actions": ["view.command_palette"],
		"tabs": [{
		"id": "reactions", "label_key": "Reactions",
		"groups": [{
			"id": "structure", "label_key": "Reaction structure",
			"overflow_label_key": "More reaction commands",
			"accent": "reaction",
			"entries": [{
				"action": "chemistry.reaction.create", "role": "primary", "priority": "required",
			}],
		}],
	}]}
	projection = ferrum_qt.declarative_resources._build_action_placement_projection(
		menu_data, ribbon_data, frozenset({
			"chemistry.reaction.create", "file.new", "view.command_palette",
		}),
	)
	return dict(projection)


#============================================
def test_palette_searches_live_label_help_and_action_id_deterministically(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Visible results come from immutable, ordered live registry views only."""
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	inspector = _action(
		registry, window, "reaction.inspector", "Reaction Inspector",
		"Inspect the selected reaction definition",
	)
	create = _action(
		registry, window, "reaction.create", "Create Reaction...",
		"Create a reaction from selected molecular roots",
	)
	views = registry.live_action_views()
	assert tuple(view.qt_action for view in views) == (create, inspector)
	controller = ferrum_qt.actions.command_palette.CommandPaletteController(
		window, registry, action_placements={},
	)
	try:
		window.show()
		controller.open()
		qapp.processEvents()
		controller.dialog.search_field.setText("selected reaction")
		qapp.processEvents()
		assert controller.dialog.result_list.currentItem().text() == "Reaction Inspector"
		controller.dialog.search_field.setText("reaction.create")
		qapp.processEvents()
		assert controller.dialog.result_list.currentItem().text() == "Create Reaction..."
	finally:
		controller.dialog.close()
		window.close()
		window.deleteLater()


#============================================
def test_disabled_palette_command_remains_visible_and_never_triggers(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An unavailable live QAction remains discoverable without side effects."""
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	action = _action(
		registry, window, "reaction.delete", "Delete Reaction",
		"Delete the selected reaction definition", enabled=False,
	)
	triggered: list[bool] = []
	action.triggered.connect(lambda: triggered.append(True))
	controller = ferrum_qt.actions.command_palette.CommandPaletteController(
		window, registry, action_placements={},
	)
	try:
		window.show()
		controller.open()
		qapp.processEvents()
		controller.dialog.search_field.setText("delete reaction")
		qapp.processEvents()
		assert controller.dialog.result_list.currentItem().text() == "Delete Reaction - Unavailable"
		PySide6.QtTest.QTest.keyClick(
			controller.dialog.search_field, PySide6.QtCore.Qt.Key.Key_Return,
		)
		qapp.processEvents()
		assert triggered == []
		assert controller.dialog.isVisible()
		assert controller.dialog.status_label.text() == "Delete Reaction is currently unavailable."
	finally:
		controller.dialog.close()
		window.close()
		window.deleteLater()


#============================================
def test_keyboard_activation_closes_palette_and_triggers_registered_action_once(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Arrows select a command without editing its query and Enter activates it."""
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	create = _action(
		registry, window, "reaction.create", "Create Reaction...",
		"Create a reaction from selected molecular roots",
	)
	inspect = _action(
		registry, window, "reaction.inspect", "Inspect Reaction",
		"Inspect the selected reaction definition",
	)
	triggered: list[bool] = []
	create.triggered.connect(lambda: triggered.append(False))
	inspect.triggered.connect(lambda: triggered.append(True))
	controller = ferrum_qt.actions.command_palette.CommandPaletteController(
		window, registry, action_placements={},
	)
	try:
		window.show()
		window.setFocus()
		controller.open()
		qapp.processEvents()
		assert controller.dialog.search_field.hasFocus()
		controller.dialog.search_field.setText("reaction")
		qapp.processEvents()
		PySide6.QtTest.QTest.keyClick(
			controller.dialog.search_field, PySide6.QtCore.Qt.Key.Key_Down,
		)
		qapp.processEvents()
		assert controller.dialog.search_field.text() == "reaction"
		assert controller.dialog.result_list.currentItem().text() == "Inspect Reaction"
		PySide6.QtTest.QTest.keyClick(
			controller.dialog.search_field, PySide6.QtCore.Qt.Key.Key_Up,
			PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier,
		)
		qapp.processEvents()
		assert controller.dialog.result_list.currentItem().text() == "Inspect Reaction"
		PySide6.QtTest.QTest.keyClick(
			controller.dialog.search_field, PySide6.QtCore.Qt.Key.Key_Up,
		)
		qapp.processEvents()
		assert controller.dialog.result_list.currentItem().text() == "Create Reaction..."
		PySide6.QtTest.QTest.keyClick(
			controller.dialog.search_field, PySide6.QtCore.Qt.Key.Key_Down,
		)
		PySide6.QtTest.QTest.keyClick(
			controller.dialog.search_field, PySide6.QtCore.Qt.Key.Key_Return,
		)
		qapp.processEvents()
		assert triggered == [True]
		assert not controller.dialog.isVisible()
		assert window.focusWidget() is not controller.dialog.search_field
	finally:
		controller.dialog.close()
		window.close()
		window.deleteLater()


#============================================
def test_palette_rechecks_an_action_disabled_before_keyboard_activation(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A live disablement keeps the palette open and explains the refusal."""
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	action = _action(
		registry, window, "reaction.create", "Create Reaction...",
		"Create a reaction from selected molecular roots",
	)
	triggered: list[bool] = []
	action.triggered.connect(lambda: triggered.append(True))
	controller = ferrum_qt.actions.command_palette.CommandPaletteController(
		window, registry, action_placements={},
	)
	try:
		window.show()
		controller.open()
		qapp.processEvents()
		action.setEnabled(False)
		PySide6.QtTest.QTest.keyClick(
			controller.dialog.search_field, PySide6.QtCore.Qt.Key.Key_Return,
		)
		qapp.processEvents()
		item = controller.dialog.result_list.currentItem()
		assert triggered == []
		assert controller.dialog.isVisible()
		assert controller.dialog.status_label.text() == "Create Reaction... is currently unavailable."
		assert item.data(PySide6.QtCore.Qt.ItemDataRole.AccessibleDescriptionRole) == (
			"This command is currently unavailable."
		)
	finally:
		controller.dialog.close()
		window.close()
		window.deleteLater()


#============================================
def test_escape_restores_focus_to_the_actual_invoking_child(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Escape closes the palette and returns focus to its invoking child widget."""
	window = PySide6.QtWidgets.QMainWindow()
	invoking_child = PySide6.QtWidgets.QLineEdit(window)
	window.setCentralWidget(invoking_child)
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	controller = ferrum_qt.actions.command_palette.CommandPaletteController(
		window, registry, action_placements={},
	)
	try:
		window.show()
		invoking_child.setFocus()
		qapp.processEvents()
		assert window.focusWidget() is invoking_child
		controller.open()
		qapp.processEvents()
		PySide6.QtTest.QTest.keyClick(
			controller.dialog.search_field, PySide6.QtCore.Qt.Key.Key_Escape,
		)
		qapp.processEvents()
		assert not controller.dialog.isVisible()
		assert window.focusWidget() is invoking_child
	finally:
		controller.dialog.close()
		window.close()
		window.deleteLater()


#============================================
def test_palette_ranks_direct_reaction_matches_and_preserves_stable_ties(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Direct label and ID matches lead incidental help text in registry tie order."""
	del qapp
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	create = _action(
		registry, window, "chemistry.reaction.create", "Create", "Create a molecule",
	)
	inspect = _action(
		registry, window, "chemistry.reaction.inspect", "Inspect", "Inspect a molecule",
	)
	about = _action(
		registry, window, "help.about", "About Ferrum", "Read about reaction tools",
	)
	try:
		ranked = ferrum_qt.actions.command_palette.ranked_matching_views(
			"reaction", registry.live_action_views(),
		)
		assert tuple(view.qt_action for view in ranked) == (create, inspect, about)
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_palette_ranks_a_one_shot_catalog_iterable(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The catalog adapter materializes its advertised generic iterable once."""
	del qapp
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	create = _action(
		registry, window, "chemistry.reaction.create", "Create Reaction",
		"Create a reaction from selected molecular roots",
	)
	inspect = _action(
		registry, window, "chemistry.reaction.inspect", "Inspect Reaction",
		"Inspect the selected reaction definition",
	)
	try:
		catalog = ferrum_qt.actions.command_catalog.live_command_catalog(registry, {})
		ranked = ferrum_qt.actions.command_palette.ranked_matching_entries(
			"reaction", (entry for entry in catalog),
		)
		assert tuple(entry.qt_action for entry in ranked) == (create, inspect)
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_palette_renders_declared_reaction_breadcrumb_for_visible_accessibility(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A parsed primary menu location remains visible and accessible per result."""
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	_action(
		registry, window, "chemistry.reaction.create", "Create Reaction...",
		"Create a reaction from selected molecular roots",
	)
	controller = ferrum_qt.actions.command_palette.CommandPaletteController(
		window, registry, action_placements=_reaction_placements(),
	)
	try:
		window.show()
		controller.open()
		qapp.processEvents()
		item = controller.dialog.result_list.currentItem()
		assert "Chemistry > Reactions" in item.text()
		assert "Chemistry > Reactions" in item.data(
			PySide6.QtCore.Qt.ItemDataRole.AccessibleTextRole,
		)
	finally:
		controller.dialog.close()
		window.close()
		window.deleteLater()


#============================================
def test_palette_refreshes_with_current_resources_and_registered_dynamic_menu(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		) -> None:
	"""Deferred dock retirement leaves every ordinary menu action resolvable."""
	controller = main_window._command_palette_controller
	assert "file.recent" in main_window._action_registry.dynamic_menu_ids()
	try:
		main_window.show()
		qapp.sendPostedEvents(None, PySide6.QtCore.QEvent.Type.DeferredDelete)
		qapp.processEvents()
		assert main_window._action_registry.get_qt_action(
			"view.properties.toggle",
		) is main_window._property_dock_toggle_action
		controller.open()
		qapp.processEvents()
		controller.refresh()
		assert controller.dialog.isVisible()
	finally:
		controller.dialog.close()

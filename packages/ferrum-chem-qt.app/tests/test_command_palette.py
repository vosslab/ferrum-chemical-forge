"""Focused behavior coverage for the registry-backed Ferrum command palette."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.actions.command_palette


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
	controller = ferrum_qt.actions.command_palette.CommandPaletteController(window, registry)
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
	controller = ferrum_qt.actions.command_palette.CommandPaletteController(window, registry)
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
	controller = ferrum_qt.actions.command_palette.CommandPaletteController(window, registry)
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
	controller = ferrum_qt.actions.command_palette.CommandPaletteController(window, registry)
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
	controller = ferrum_qt.actions.command_palette.CommandPaletteController(window, registry)
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

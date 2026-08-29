"""Focused behavior coverage for Ferrum's metadata-derived Command Reference."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.actions.command_catalog
import ferrum_qt.actions.command_reference


#============================================
def _action(
		registry: ferrum_qt.actions.action_registry.ActionRegistry,
		parent: PySide6.QtWidgets.QWidget, action_id: str, label: str,
		help_text: str, *, enabled: bool = True, shortcut: str = "Ctrl+R",
		) -> PySide6.QtGui.QAction:
	"""Register one feature-owned command with current Qt presentation facts."""
	action = PySide6.QtGui.QAction(label, parent)
	action.setToolTip(help_text)
	action.setEnabled(enabled)
	action.setShortcut(shortcut)
	registry.register_existing(action_id, action)
	return action


#============================================
def test_live_catalog_uses_current_qt_facts_and_omits_destroyed_actions(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The one immutable projection has no declaration or stale-client fallback."""
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	parent = PySide6.QtWidgets.QWidget()
	action = _action(
		registry, parent, "help.reference", "Reference", "Read command help",
		enabled=False, shortcut="Alt+R",
	)
	placements = {"help.reference": ("Help", "Reference")}
	entry, = ferrum_qt.actions.command_catalog.live_command_catalog(registry, placements)
	assert entry.qt_action is action
	assert entry.label == "Reference"
	assert entry.help_text == "Read command help"
	assert entry.shortcut == action.shortcut().toString(
		PySide6.QtGui.QKeySequence.SequenceFormat.NativeText,
	)
	assert entry.placement == ("Help", "Reference")
	assert not entry.enabled
	assert entry.availability_description == "This command is currently unavailable."
	parent.deleteLater()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.DeferredDelete,
	)
	qapp.processEvents()
	assert ferrum_qt.actions.command_catalog.live_command_catalog(registry, placements) == ()


#============================================
def test_reference_filters_without_activation_and_restores_invoking_focus(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The modeless reference supports keyboard discovery without command side effects."""
	window = PySide6.QtWidgets.QMainWindow()
	invoking_child = PySide6.QtWidgets.QLineEdit(window)
	window.setCentralWidget(invoking_child)
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	action = _action(
		registry, window, "chemistry.inspect", "Inspect Molecule",
		"Show molecule diagnostics", enabled=False, shortcut="Ctrl+I",
	)
	triggered: list[bool] = []
	action.triggered.connect(lambda: triggered.append(True))
	controller = ferrum_qt.actions.command_reference.CommandReferenceController(
		window, registry, {"chemistry.inspect": ("Chemistry", "Analysis")},
	)
	try:
		window.show()
		invoking_child.setFocus()
		qapp.processEvents()
		controller.open()
		qapp.processEvents()
		assert controller.dialog.search_field.hasFocus()
		item = controller.dialog.result_list.currentItem()
		assert "Inspect Molecule" in item.text()
		assert "Shortcut: {0}".format(action.shortcut().toString(
			PySide6.QtGui.QKeySequence.SequenceFormat.NativeText,
		)) in item.text()
		assert "Chemistry > Analysis" in item.text()
		assert "unavailable" in item.text().casefold()
		assert "unavailable" in item.data(
			PySide6.QtCore.Qt.ItemDataRole.AccessibleDescriptionRole,
		).casefold()
		controller.dialog.search_field.setText("diagnostics")
		qapp.processEvents()
		assert controller.dialog.result_list.count() == 1
		PySide6.QtTest.QTest.keyClick(
			controller.dialog.search_field, PySide6.QtCore.Qt.Key.Key_Tab,
		)
		qapp.processEvents()
		assert controller.dialog.focusWidget() is controller.dialog.result_list
		controller.dialog.search_field.setFocus()
		controller.dialog.search_field.setText("nothing here")
		qapp.processEvents()
		assert controller.dialog.result_list.count() == 0
		assert "No commands match" in controller.dialog.status_label.text()
		PySide6.QtTest.QTest.keyClick(
			controller.dialog.search_field, PySide6.QtCore.Qt.Key.Key_Escape,
		)
		qapp.processEvents()
		assert triggered == []
		assert not controller.dialog.isVisible()
		assert window.focusWidget() is invoking_child
	finally:
		controller.dialog.close()
		window.close()
		window.deleteLater()


#============================================
def test_reference_reopen_reads_a_reassigned_live_shortcut(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Shortcut display comes from the live QAction rather than cached defaults."""
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	action = _action(
		registry, window, "help.reference", "Reference", "Read command help",
		shortcut="Ctrl+R",
	)
	controller = ferrum_qt.actions.command_reference.CommandReferenceController(
		window, registry, {"help.reference": ("Help",)},
	)
	try:
		window.show()
		controller.open()
		qapp.processEvents()
		assert "Shortcut: {0}".format(action.shortcut().toString(
			PySide6.QtGui.QKeySequence.SequenceFormat.NativeText,
		)) in controller.dialog.result_list.currentItem().text()
		controller.dialog.close()
		action.setShortcut("Alt+R")
		controller.open()
		qapp.processEvents()
		shortcut = action.shortcut().toString(
			PySide6.QtGui.QKeySequence.SequenceFormat.NativeText,
		)
		assert "Shortcut: {0}".format(shortcut) in controller.dialog.result_list.currentItem().text()
		controller.dialog.search_field.setText(shortcut)
		qapp.processEvents()
		assert any(
				"Reference" in controller.dialog.result_list.item(index).text()
				and not controller.dialog.result_list.item(index).isHidden()
				for index in range(controller.dialog.result_list.count())
				)
	finally:
		controller.dialog.close()
		window.close()
		window.deleteLater()

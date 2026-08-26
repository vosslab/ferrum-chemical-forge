"""Exercise public atom-oxidation observation in the staged offscreen Qt runtime."""

# Standard Library
import collections.abc
import json

# local repo modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_WATER_CDML = """<cdml xmlns="urn:ferrum:cdml" version="1.0">
<molecule id="water">
<atom id="oxygen" name="O" charge="0" explicit_hydrogens="0"><point x="0" y="0"/></atom>
<atom id="hydrogen-a" name="H" charge="0" explicit_hydrogens="0"><point x="40" y="0"/></atom>
<atom id="hydrogen-b" name="H" charge="0" explicit_hydrogens="0"><point x="-40" y="0"/></atom>
<bond id="bond-a" start="oxygen" end="hydrogen-a" type="n1"/>
<bond id="bond-b" start="oxygen" end="hydrogen-b" type="n1"/>
</molecule></cdml>"""

_EXCLUDED_CDML = """<cdml xmlns="urn:ferrum:cdml" version="1.0">
<molecule id="fluoride"><atom id="fluorine" name="F" charge="0" explicit_hydrogens="0"><point x="0" y="0"/></atom></molecule>
</cdml>"""


#============================================
class AtomOxidationE2eError(RuntimeError):
	"""One failed public Qt atom-oxidation workflow assertion."""


#============================================
def _wait_for(predicate: collections.abc.Callable[[], bool], description: str) -> None:
	"""Run the Qt event loop until one semantic condition completes or times out."""
	loop = PySide6.QtCore.QEventLoop()
	timer = PySide6.QtCore.QTimer()
	timer.setInterval(10)
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)
	state = {"complete": False, "timed_out": False}

	def check() -> None:
		if predicate():
			state["complete"] = True
			loop.quit()

	def expire() -> None:
		state["timed_out"] = True
		loop.quit()

	timer.timeout.connect(check)
	timeout.timeout.connect(expire)
	check()
	if not state["complete"]:
		timer.start()
		timeout.start(5000)
		loop.exec()
	timer.stop()
	if not state["complete"]:
		raise AtomOxidationE2eError("timed out waiting for {0}".format(description))


#============================================
def _select_structure_action(window: ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		) -> PySide6.QtGui.QAction:
	"""Return the public Edit -> Select Structure action by its visible label."""
	for action in window.findChildren(PySide6.QtGui.QAction):
		if action.text().replace("&", "") == "Select Structure":
			return action
	raise AtomOxidationE2eError("main window did not expose Edit -> Select Structure action")


#============================================
def _action(window: ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		label: str) -> PySide6.QtGui.QAction:
	"""Return one public action by its visible label."""
	for action in window.findChildren(PySide6.QtGui.QAction):
		if action.text().replace("&", "") == label:
			return action
	raise AtomOxidationE2eError("main window did not expose action {0!r}".format(label))


#============================================
def _document_tabs(window: ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		) -> PySide6.QtWidgets.QTabWidget:
	"""Return the public document-tab control."""
	tabs = window.centralWidget()
	if not isinstance(tabs, PySide6.QtWidgets.QTabWidget):
		raise AtomOxidationE2eError("main window did not expose public document tabs")
	return tabs


#============================================
def _activate_tab(window: ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab) -> None:
	"""Activate one existing source through its visible tab-bar control."""
	tabs = _document_tabs(window)
	index = tabs.indexOf(tab)
	if index < 0:
		raise AtomOxidationE2eError("source tab is absent from the public tab control")
	bar = tabs.tabBar()
	PySide6.QtTest.QTest.mouseClick(
		bar, PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, bar.tabRect(index).center(),
	)
	_wait_for(lambda: tabs.currentWidget() is tab, "the requested source tab to become active")


#============================================
def _select_atom(tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		scene_point: PySide6.QtCore.QPointF, select_action: PySide6.QtGui.QAction,
		action: PySide6.QtGui.QAction) -> None:
	"""Select one authored atom through the focused public viewport gesture."""
	if not select_action.isEnabled():
		raise AtomOxidationE2eError("Edit -> Select Structure action was disabled")
	if not select_action.isCheckable():
		raise AtomOxidationE2eError("Edit -> Select Structure action did not expose active state")
	if not select_action.isChecked():
		select_action.trigger()
	_wait_for(select_action.isChecked, "Edit -> Select Structure mode to become active")
	viewport = tab.view.viewport()
	viewport.setFocus()
	PySide6.QtTest.QTest.mouseClick(
		viewport, PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, tab.view.mapFromScene(scene_point),
	)
	_wait_for(action.isEnabled, "the Atom Oxidation State action to become available")


#============================================
def _visible_result_dialog(window: ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		) -> PySide6.QtWidgets.QDialog | None:
	"""Return the visible result dialog while retired history dialogs remain children."""
	for dialog in window.findChildren(PySide6.QtWidgets.QDialog, "atom-oxidation-dialog"):
		if dialog.isVisible():
			return dialog
	return None


#============================================
def _observe(window: ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		scene_point: PySide6.QtCore.QPointF) -> PySide6.QtWidgets.QDialog:
	"""Select one atom and invoke the visible public oxidation QAction."""
	action = _action(window, "Atom Oxidation State...")
	_select_action = _select_structure_action(window)
	_select_atom(tab, scene_point, _select_action, action)
	action.trigger()
	_wait_for(
		lambda: _visible_result_dialog(window) is not None,
		"the modeless Atom Oxidation State result dialog",
	)
	dialog = _visible_result_dialog(window)
	if dialog is None:
		raise AtomOxidationE2eError("visible atom oxidation dialog disappeared")
	if dialog.isModal():
		raise AtomOxidationE2eError("atom oxidation result is modal")
	return dialog


#============================================
def _dialog_details(dialog: PySide6.QtWidgets.QDialog) -> str:
	"""Return the user-visible result text from the named result field."""
	details = dialog.findChild(PySide6.QtWidgets.QPlainTextEdit, "atom-oxidation-details")
	if details is None:
		raise AtomOxidationE2eError("atom oxidation dialog has no result details field")
	return details.toPlainText()


#============================================
def _dialog_source_status(dialog: PySide6.QtWidgets.QDialog) -> str:
	"""Return the visible source-fence status from the named field."""
	status = dialog.findChild(PySide6.QtWidgets.QLabel, "atom-oxidation-source-status")
	if status is None:
		raise AtomOxidationE2eError("atom oxidation dialog has no source status field")
	return status.text()


#============================================
def _rerun_button(dialog: PySide6.QtWidgets.QDialog) -> PySide6.QtWidgets.QPushButton:
	"""Return the named public source-bound rerun control."""
	button = dialog.findChild(PySide6.QtWidgets.QPushButton, "atom-oxidation-run-again")
	if button is None:
		raise AtomOxidationE2eError("atom oxidation dialog has no Run Again control")
	return button


#============================================
def main() -> int:
	"""Observe accepted water and a genuine excluded element through real Qt actions."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	water = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_WATER_CDML, "water.cdml")
	window._register_native_tab(water, activate=True)
	window.show()
	app.processEvents()
	try:
		water_before = water.current_snapshot
		accepted_dialog = _observe(window, water, PySide6.QtCore.QPointF(0.0, 0.0))
		accepted_details = _dialog_details(accepted_dialog)
		if "Oxidation state: -2" not in accepted_details:
			raise AtomOxidationE2eError("water oxygen result did not expose Rust-issued -2")
		if "Convention: formal-electron-assignment-hcno-v1" not in accepted_details:
			raise AtomOxidationE2eError("water result did not expose the stable convention")
		if water.current_snapshot != water_before:
			raise AtomOxidationE2eError("accepted observation changed the water document")

		water_cdml_before_edit = water.current_snapshot.cdml
		add_atom = _action(window, "Add Atom at Point")
		if not add_atom.isEnabled():
			raise AtomOxidationE2eError("Add Atom at Point was disabled for the water source")
		add_atom.trigger()
		PySide6.QtTest.QTest.mouseClick(
			water.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			water.view.mapFromScene(PySide6.QtCore.QPointF(100.0, 100.0)),
		)
		_wait_for(
			lambda: water.current_snapshot.cdml != water_cdml_before_edit,
			"the visible Add Atom edit to change the water source",
		)
		_wait_for(
			lambda: _dialog_source_status(accepted_dialog)
			== "This result is from an earlier document revision.",
			"the retained water result to become historical after its source edit",
		)
		if _dialog_details(accepted_dialog) != accepted_details:
			raise AtomOxidationE2eError("historical water result did not retain its visible details")
		_action(window, "Undo").trigger()
		_wait_for(
			lambda: water.current_snapshot.cdml == water_cdml_before_edit,
			"Undo to restore the clean water source",
		)

		_action(window, "New").trigger()
		_wait_for(
			lambda: _document_tabs(window).currentWidget() is not water,
			"a second real document tab to become active",
		)
		rerun = _rerun_button(accepted_dialog)
		_wait_for(lambda: not rerun.isEnabled(), "Run Again to disable away from its source")
		if "Return to this result's source document" not in rerun.accessibleDescription():
			raise AtomOxidationE2eError("disabled Run Again did not explain its source binding")

		_activate_tab(window, water)
		atom_oxidation = _action(window, "Atom Oxidation State...")
		_select_atom(
			water, PySide6.QtCore.QPointF(0.0, 0.0), _select_structure_action(window),
			atom_oxidation,
		)
		_wait_for(rerun.isEnabled, "Run Again to enable for the selected original source")
		PySide6.QtTest.QTest.mouseClick(
			rerun, PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		)
		_wait_for(
			lambda: _visible_result_dialog(window) is not None
			and _visible_result_dialog(window) is not accepted_dialog,
			"a refreshed Atom Oxidation State result from the original source",
		)
		refreshed_dialog = _visible_result_dialog(window)
		if refreshed_dialog is None or "Oxidation state: -2" not in _dialog_details(refreshed_dialog):
			raise AtomOxidationE2eError("Run Again did not return the water-source oxidation result")

		_action(window, "Close Tab").trigger()
		_wait_for(
			lambda: _document_tabs(window).indexOf(water) < 0,
			"the clean original source tab to close through the public tab UI",
		)
		_wait_for(
			lambda: not refreshed_dialog.isVisible(),
			"the original source's modeless result dialog to retire",
		)

		excluded = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
			_EXCLUDED_CDML, "fluoride.cdml",
		)
		window._register_native_tab(excluded, activate=True)
		app.processEvents()
		excluded_before = excluded.current_snapshot
		unavailable_dialog = _observe(window, excluded, PySide6.QtCore.QPointF(0.0, 0.0))
		unavailable_details = _dialog_details(unavailable_dialog)
		if "Reason: element_outside_profile" not in unavailable_details:
			raise AtomOxidationE2eError("excluded element did not expose its closed unavailable reason")
		if "Recovery: Use a fully materialized H/C/N/O molecule." not in unavailable_details:
			raise AtomOxidationE2eError("excluded element did not expose its closed recovery")
		if excluded.current_snapshot != excluded_before:
			raise AtomOxidationE2eError("unavailable observation changed the excluded document")
		unavailable_dialog.close()
		_wait_for(lambda: not unavailable_dialog.isVisible(), "the unavailable result dialog to close")
		print(json.dumps({"schema": "ferrum-m4-atom-oxidation-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		app.processEvents()


if __name__ == "__main__":
	raise SystemExit(main())

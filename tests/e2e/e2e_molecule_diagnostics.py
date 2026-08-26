"""Check one unexpanded compact group through Ferrum's public diagnostics dialog."""

# Standard Library
import json

# local repo modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()


# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
# This guard only releases a stalled nested event loop with an actionable
# failure. It is deadlock protection rather than a product timing requirement.
DIALOG_WAIT_DEADLOCK_GUARD_MILLISECONDS = 15000


#============================================
class MoleculeDiagnosticsE2eError(RuntimeError):
	"""One failed public Check Structure workflow assertion."""


#============================================
def _exposed_menu_action(window: PySide6.QtWidgets.QMainWindow,
		menu_label: str, action_label: str) -> PySide6.QtGui.QAction:
	"""Return one visible enabled public menu action by its displayed text."""
	menu_bar = window.menuBar()
	menu_action = next(
		action for action in menu_bar.actions()
		if action.text().replace("&", "") == menu_label
	)
	if not menu_bar.isVisible() or not menu_action.isVisible():
		raise MoleculeDiagnosticsE2eError(
			f"{menu_label} menu was not publicly exposed",
		)
	menu = menu_action.menu()
	if menu is None:
		raise MoleculeDiagnosticsE2eError(
			f"Ferrum did not expose the {menu_label} menu",
		)
	action = next(
		candidate for candidate in menu.actions()
		if candidate.text().replace("&", "") == action_label
	)
	if not action.isVisible() or not action.isEnabled():
		raise MoleculeDiagnosticsE2eError(
			f"{menu_label} -> {action_label} was not publicly available",
		)
	return action


#============================================
def _trigger_exposed_menu_action(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, menu_label: str, action_label: str) -> None:
	"""Trigger one visible enabled public menu action by its displayed text."""
	action = _exposed_menu_action(window, menu_label, action_label)
	action.trigger()
	app.processEvents()


#============================================
def _ensure_exposed_menu_action_checked(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, menu_label: str, action_label: str) -> None:
	"""Activate one public checkable action while preserving an active tool."""
	action = _exposed_menu_action(window, menu_label, action_label)
	if not action.isCheckable():
		raise MoleculeDiagnosticsE2eError(
			f"{menu_label} -> {action_label} was not a checkable public tool",
		)
	if not action.isChecked():
		action.trigger()
		app.processEvents()


#============================================
def _canvas(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtWidgets.QGraphicsView:
	"""Return Ferrum's visible, accessibly named drawing canvas."""
	return next(
		view for view in window.findChildren(PySide6.QtWidgets.QGraphicsView)
		if view.isVisible() and view.accessibleName() == "Ferrum drawing canvas"
	)


#============================================
def _select_next_atom_carbon(app: PySide6.QtWidgets.QApplication) -> None:
	"""Enter C through the public Next Drawing dialog."""
	dialog = app.activeModalWidget()
	if (
		not isinstance(dialog, PySide6.QtWidgets.QDialog)
		or not dialog.isVisible()
		or dialog.accessibleName() != "Next Drawing"
	):
		raise MoleculeDiagnosticsE2eError(
			"Edit -> Next Drawing did not open its visible public dialog",
		)
	combo = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QComboBox)
		if widget.isVisible() and widget.accessibleName() == "Next atom"
	)
	editor = combo.lineEdit()
	if editor is None:
		raise MoleculeDiagnosticsE2eError(
			"Next atom did not expose its visible text editor",
		)
	PySide6.QtTest.QTest.mouseClick(editor, PySide6.QtCore.Qt.MouseButton.LeftButton)
	PySide6.QtTest.QTest.keyClick(editor, PySide6.QtCore.Qt.Key.Key_A,
		PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
	PySide6.QtTest.QTest.keyClicks(editor, "C")
	PySide6.QtTest.QTest.keyClick(editor, PySide6.QtCore.Qt.Key.Key_Return)
	app.processEvents()
	if combo.currentText() != "C":
		raise MoleculeDiagnosticsE2eError(
			"the visible Next atom control did not retain C",
		)
	button_box = next(
		box for box in dialog.findChildren(PySide6.QtWidgets.QDialogButtonBox)
		if box.isVisible()
	)
	accept_button = next(
		button for button in button_box.buttons()
		if button_box.standardButton(button) in (
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok,
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Close,
		)
		and button.isVisible() and button.isEnabled()
	)
	accept_button.click()


#============================================
def _choose_next_atom_carbon(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication) -> None:
	"""Set C through the public Edit -> Next Drawing workflow."""
	PySide6.QtCore.QTimer.singleShot(0, lambda: _select_next_atom_carbon(app))
	_trigger_exposed_menu_action(window, app, "Edit", "Next Drawing...")


#============================================
def _choose_me(app: PySide6.QtWidgets.QApplication) -> None:
	"""Choose the visible Me option and confirm the accessible attach dialog."""
	dialog = app.activeModalWidget()
	if (
		not isinstance(dialog, PySide6.QtWidgets.QDialog)
		or not dialog.isVisible()
		or dialog.accessibleName() != "Attach compact group to selected atom"
	):
		raise MoleculeDiagnosticsE2eError(
			"Attach Compact Group did not open its accessible chooser",
		)
	choice = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QComboBox)
		if widget.isVisible() and widget.accessibleName() == "Compact group"
	)
	choice_index = choice.findText("Me", PySide6.QtCore.Qt.MatchFlag.MatchExactly)
	if choice_index < 0:
		raise MoleculeDiagnosticsE2eError(
			"the visible compact-group chooser did not offer Me",
		)
	choice.setCurrentIndex(choice_index)
	confirm = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QPushButton)
		if widget.isVisible() and widget.text() == "Attach to Selected Atom"
	)
	PySide6.QtTest.QTest.mouseClick(confirm, PySide6.QtCore.Qt.MouseButton.LeftButton)
	app.processEvents()
	if dialog.isVisible():
		raise MoleculeDiagnosticsE2eError(
			"Attach to Selected Atom did not accept the Me chooser",
		)


#============================================
class _AccessibleDialogObserver(PySide6.QtCore.QObject):
	"""Observe one named dialog and release a nested E2E event loop safely."""

	def __init__(self, app: PySide6.QtWidgets.QApplication,
			dialog_name: str, require_modeless: bool) -> None:
		super().__init__(app)
		self.app = app
		self.dialog_name = dialog_name
		self.require_modeless = require_modeless
		self.completion_loop = PySide6.QtCore.QEventLoop()
		self.dialog: PySide6.QtWidgets.QDialog | None = None
		self.observed_text = ""
		self.failure = ""
		self.rejected: list[PySide6.QtWidgets.QDialog] = []
		app.installEventFilter(self)

	def close(self) -> None:
		"""Remove the observer after its single public dialog interaction."""
		self.app.removeEventFilter(self)

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Capture the dialog by public accessible name when it becomes visible."""
		if (
			event.type() != PySide6.QtCore.QEvent.Type.Show
			or not isinstance(watched, PySide6.QtWidgets.QDialog)
			or watched.accessibleName() != self.dialog_name
		):
			return False
		if self.require_modeless and watched.isModal():
			self.failure = f"{self.dialog_name} was modal instead of modeless"
			self._schedule_rejection(watched)
			self.completion_loop.quit()
			return False
		if self.dialog is None:
			self.dialog = watched
			PySide6.QtCore.QTimer.singleShot(0, self._receive_visible_text)
		return False

	def _schedule_rejection(self, dialog: PySide6.QtWidgets.QDialog) -> None:
		"""Reject once so an unexpected modal cannot deadlock the E2E."""
		if any(existing is dialog for existing in self.rejected):
			return
		self.rejected.append(dialog)
		PySide6.QtCore.QTimer.singleShot(0, dialog.reject)

	def _receive_visible_text(self) -> None:
		"""Receive readable public text from the named dialog after it is shown."""
		if self.dialog is None:
			return
		text_widgets = [
			widget for widget in self.dialog.findChildren(PySide6.QtWidgets.QPlainTextEdit)
			if widget.isVisible() and widget.isReadOnly()
		]
		text_widgets.extend(
			widget for widget in self.dialog.findChildren(PySide6.QtWidgets.QTextEdit)
			if widget.isVisible() and widget.isReadOnly()
		)
		label_text = [
			widget.text() for widget in self.dialog.findChildren(PySide6.QtWidgets.QLabel)
			if widget.isVisible() and widget.text()
		]
		self.observed_text = "\n".join(
			[widget.toPlainText() for widget in text_widgets] + label_text,
		)
		if self.observed_text:
			self.completion_loop.quit()

	def _dialog_wait_deadlock(self) -> None:
		"""Release a stalled dialog phase with useful state for the failure report."""
		if not self.observed_text and not self.failure:
			self.failure = (
				f"{self.dialog_name} did not expose visible readable text before "
				"the E2E liveness guard"
			)
		active_modal = self.app.activeModalWidget()
		if isinstance(active_modal, PySide6.QtWidgets.QDialog):
			self._schedule_rejection(active_modal)
		self.completion_loop.quit()

	def await_text(self) -> str:
		"""Wait for dialog text with a scoped deadlock guard, never a timing assertion."""
		guard = PySide6.QtCore.QTimer(self)
		guard.setSingleShot(True)
		guard.timeout.connect(self._dialog_wait_deadlock)
		guard.start(DIALOG_WAIT_DEADLOCK_GUARD_MILLISECONDS)
		if not self.observed_text and not self.failure:
			self.completion_loop.exec()
		guard.stop()
		if self.failure:
			raise MoleculeDiagnosticsE2eError(self.failure)
		if not self.observed_text:
			raise MoleculeDiagnosticsE2eError(
				f"{self.dialog_name} did not expose visible readable text",
			)
		return self.observed_text


#============================================
def _close_visible_dialog(dialog: PySide6.QtWidgets.QDialog,
		app: PySide6.QtWidgets.QApplication, dialog_name: str) -> None:
	"""Close one dialog through its visible public Close control."""
	close_button = next(
		(
			button for button in dialog.findChildren(PySide6.QtWidgets.QPushButton)
			if button.isVisible() and button.isEnabled() and button.text() == "Close"
		),
		None,
	)
	if close_button is None:
		raise MoleculeDiagnosticsE2eError(
			f"{dialog_name} did not expose a visible enabled public Close control",
		)
	PySide6.QtTest.QTest.mouseClick(close_button,
		PySide6.QtCore.Qt.MouseButton.LeftButton)
	app.processEvents()
	if dialog.isVisible():
		raise MoleculeDiagnosticsE2eError(
			f"the visible {dialog_name} dialog did not close through its public Close control",
		)


#============================================
def _select_visible_structure_finding(dialog: PySide6.QtWidgets.QDialog,
		app: PySide6.QtWidgets.QApplication, code: str) -> str:
	"""Select one public finding row and return its visible details and guidance."""
	tree = next((
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QTreeWidget)
		if widget.isVisible() and widget.accessibleName() == "Structure findings"
	), None)
	if tree is None:
		raise MoleculeDiagnosticsE2eError(
			"Check Structure did not expose its accessible findings tree",
		)
	item = next((
		tree.topLevelItem(index) for index in range(tree.topLevelItemCount())
		if tree.topLevelItem(index).text(1) == code
	), None)
	if item is None:
		observed = tuple(
			tree.topLevelItem(index).text(1) for index in range(tree.topLevelItemCount())
		)
		raise MoleculeDiagnosticsE2eError(
			f"Check Structure did not expose finding row {code!r}; observed {observed!r}",
		)
	tree.setCurrentItem(item)
	tree.scrollToItem(item)
	app.processEvents()
	details = next((
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QPlainTextEdit)
		if widget.isVisible() and widget.isReadOnly()
		and widget.accessibleName() == "Selected finding details"
	), None)
	if details is None:
		raise MoleculeDiagnosticsE2eError(
			"Check Structure did not expose accessible selected-finding details",
		)
	labels = (
		widget.text() for widget in dialog.findChildren(PySide6.QtWidgets.QLabel)
		if widget.isVisible() and widget.text()
	)
	return "\n".join((details.toPlainText(), *labels))


#============================================
def _require_diagnostics_contract(details: str) -> None:
	"""Require the public non-mutating finding and recovery guidance semantics."""
	normalized = details.lower()
	if "unexpanded_group_present" not in normalized:
		raise MoleculeDiagnosticsE2eError(
			"Check Structure did not expose the Rust-owned unexpanded_group_present finding; "
			f"observed accessible dialog text: {details[:2000]!r}",
		)
	if "materialize" not in normalized:
		raise MoleculeDiagnosticsE2eError(
			"Check Structure did not expose materialization as the public recovery; "
			f"observed accessible dialog text: {details[:2000]!r}",
		)
	if not (
		"read-only" in normalized
		or "no change" in normalized
		or "does not change" in normalized
		or "does not modify" in normalized
	):
		raise MoleculeDiagnosticsE2eError(
			"Check Structure did not state that diagnostics leave the molecule unchanged; "
			f"observed accessible dialog text: {details[:2000]!r}",
		)


#============================================
def main() -> int:
	"""Prove diagnostics preserve an attached Me until public materialization."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		window.show()
		app.processEvents()
		_trigger_exposed_menu_action(window, app, "File", "New")
		canvas = _canvas(window)
		_choose_next_atom_carbon(window, app)
		first_carbon = PySide6.QtCore.QPointF(40.0, 40.0)
		second_carbon = PySide6.QtCore.QPointF(100.0, 40.0)
		group_anchor = PySide6.QtCore.QPointF(40.0, 125.0)
		_trigger_exposed_menu_action(window, app, "Edit", "Draw Bond")
		PySide6.QtTest.QTest.mousePress(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			canvas.mapFromScene(first_carbon),
		)
		PySide6.QtTest.QTest.mouseMove(
			canvas.viewport(), canvas.mapFromScene(second_carbon),
		)
		PySide6.QtTest.QTest.mouseRelease(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			canvas.mapFromScene(second_carbon),
		)
		app.processEvents()
		_ensure_exposed_menu_action_checked(window, app, "Edit", "Select Structure")
		PySide6.QtTest.QTest.mouseClick(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			canvas.mapFromScene(first_carbon),
		)
		app.processEvents()
		_trigger_exposed_menu_action(window, app, "Chemistry", "Attach Compact Group...")
		_choose_me(app)
		PySide6.QtTest.QTest.mouseRelease(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			canvas.mapFromScene(group_anchor),
		)
		app.processEvents()
		_ensure_exposed_menu_action_checked(window, app, "Edit", "Select Structure")
		PySide6.QtTest.QTest.mouseClick(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			canvas.mapFromScene(first_carbon),
		)
		app.processEvents()
		diagnostics_observer = _AccessibleDialogObserver(app, "Check Structure", True)
		try:
			_trigger_exposed_menu_action(window, app, "Chemistry", "Check Structure...")
			diagnostics_observer.await_text()
			if diagnostics_observer.dialog is None:
				raise MoleculeDiagnosticsE2eError(
					"Check Structure did not expose its accessible modeless dialog",
				)
			diagnostics = _select_visible_structure_finding(
				diagnostics_observer.dialog, app, "unexpanded_group_present",
			)
			_require_diagnostics_contract(diagnostics)
			_close_visible_dialog(diagnostics_observer.dialog, app, "Check Structure")
		finally:
			diagnostics_observer.close()
		PySide6.QtTest.QTest.mouseClick(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			canvas.mapFromScene(group_anchor),
		)
		app.processEvents()
		_trigger_exposed_menu_action(window, app, "Chemistry",
			"Materialize Selected Compact Group")
		report_observer = _AccessibleDialogObserver(app, "Molecule Report", False)
		try:
			_trigger_exposed_menu_action(window, app, "Chemistry", "Molecule Report...")
			report = report_observer.await_text()
			if "Formula: C3H8" not in report:
				raise MoleculeDiagnosticsE2eError(
					"materializing the diagnosed Me did not expose propane's public formula; "
					f"observed accessible dialog text: {report[:2000]!r}",
				)
			if report_observer.dialog is None:
				raise MoleculeDiagnosticsE2eError(
					"Molecule Report did not expose its accessible public dialog",
				)
			_close_visible_dialog(report_observer.dialog, app, "Molecule Report")
		finally:
			report_observer.close()
		print(json.dumps({"schema": "ferrum-molecule-diagnostics-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	raise SystemExit(main())

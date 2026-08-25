"""Author attached Me through the public canvas, then materialize its ordinary atoms."""

# Standard Library
import json

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
class CompactGroupAuthorToMaterializeE2eError(RuntimeError):
	"""One failed public attached-Me authoring and materialization assertion."""


#============================================
def _trigger_exposed_menu_action(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, menu_label: str, action_label: str) -> None:
	"""Trigger one visible enabled public menu action by its displayed text."""
	menu_bar = window.menuBar()
	menu_action = next(
		action for action in menu_bar.actions()
		if action.text().replace("&", "") == menu_label
	)
	if not menu_bar.isVisible() or not menu_action.isVisible():
		raise CompactGroupAuthorToMaterializeE2eError(
			"{0} menu was not publicly exposed".format(menu_label),
		)
	menu = menu_action.menu()
	if menu is None:
		raise CompactGroupAuthorToMaterializeE2eError(
			"Ferrum did not expose the {0} menu".format(menu_label),
		)
	action = next(
		candidate for candidate in menu.actions()
		if candidate.text().replace("&", "") == action_label
	)
	if not action.isVisible() or not action.isEnabled():
		raise CompactGroupAuthorToMaterializeE2eError(
			"{0} -> {1} was not publicly available".format(menu_label, action_label),
		)
	action.trigger()
	app.processEvents()


#============================================
def _canvas(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtWidgets.QGraphicsView:
	"""Return Ferrum's visible, accessibly named drawing canvas."""
	canvas = next(
		view for view in window.findChildren(PySide6.QtWidgets.QGraphicsView)
		if view.isVisible() and view.accessibleName() == "Ferrum drawing canvas"
	)
	return canvas


#============================================
def _select_next_atom_carbon(app: PySide6.QtWidgets.QApplication) -> None:
	"""Enter C through the public Next Drawing dialog."""
	dialog = app.activeModalWidget()
	if (
		not isinstance(dialog, PySide6.QtWidgets.QDialog)
		or not dialog.isVisible()
		or dialog.accessibleName() != "Next Drawing"
	):
		raise CompactGroupAuthorToMaterializeE2eError(
			"Edit -> Next Drawing did not open its visible public dialog",
		)
	combo = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QComboBox)
		if widget.isVisible() and widget.accessibleName() == "Next atom"
	)
	editor = combo.lineEdit()
	if editor is None:
		raise CompactGroupAuthorToMaterializeE2eError(
			"Next atom did not expose its visible text editor",
		)
	PySide6.QtTest.QTest.mouseClick(editor, PySide6.QtCore.Qt.MouseButton.LeftButton)
	PySide6.QtTest.QTest.keyClick(editor, PySide6.QtCore.Qt.Key.Key_A,
		PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
	PySide6.QtTest.QTest.keyClicks(editor, "C")
	PySide6.QtTest.QTest.keyClick(editor, PySide6.QtCore.Qt.Key.Key_Return)
	app.processEvents()
	if combo.currentText() != "C":
			raise CompactGroupAuthorToMaterializeE2eError(
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
		raise CompactGroupAuthorToMaterializeE2eError(
			"Attach Compact Group did not open its accessible chooser",
		)
	choice = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QLabel)
		if widget.isVisible() and widget.accessibleName() == "Compact group Me"
	)
	confirm = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QPushButton)
		if widget.isVisible() and widget.text() == "Attach to Selected Atom"
	)
	PySide6.QtTest.QTest.mouseClick(choice, PySide6.QtCore.Qt.MouseButton.LeftButton)
	PySide6.QtTest.QTest.mouseClick(confirm, PySide6.QtCore.Qt.MouseButton.LeftButton)
	app.processEvents()
	if dialog.isVisible():
		raise CompactGroupAuthorToMaterializeE2eError(
			"Attach to Selected Atom did not accept the Me chooser",
		)


#============================================
class _MoleculeReportObserver(PySide6.QtCore.QObject):
	"""Receive one public modeless report dialog and its completed detail text."""

	def __init__(self, app: PySide6.QtWidgets.QApplication) -> None:
		super().__init__(app)
		self.completion_loop = PySide6.QtCore.QEventLoop()
		self.details: PySide6.QtWidgets.QPlainTextEdit | None = None
		self.observed_text = ""
		app.installEventFilter(self)

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Attach only when the public report dialog becomes visible."""
		if (
			event.type() == PySide6.QtCore.QEvent.Type.Show
			and isinstance(watched, PySide6.QtWidgets.QDialog)
			and watched.accessibleName() == "Molecule Report"
		):
			details = next(
				(
					widget for widget in watched.findChildren(
						PySide6.QtWidgets.QPlainTextEdit,
					)
					if widget.isVisible()
					and widget.isReadOnly()
					and widget.accessibleName() == "Selected molecule report details"
				),
				None,
			)
			if details is not None and self.details is None:
				self.details = details
				details.textChanged.connect(self._receive_completed_details)
				self._receive_completed_details()
		return False

	def _receive_completed_details(self) -> None:
		"""Quit only after the visible public details editor has report text."""
		if self.details is None:
			return
		self.observed_text = self.details.toPlainText()
		if self.observed_text:
			self.completion_loop.quit()

	def await_completed_details(self) -> str:
		"""Wait for the public report dialog to deliver its visible details."""
		if not self.observed_text:
			self.completion_loop.exec()
		if not self.observed_text:
			raise CompactGroupAuthorToMaterializeE2eError(
				"Chemistry -> Molecule Report did not complete its visible public details; "
				"observed accessible dialog text: {0!r}".format(self.observed_text[:2000]),
			)
		return self.observed_text


#============================================
def main() -> int:
	"""Prove the visible attached-Me authoring path materializes to methyl carbon."""
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
		_trigger_exposed_menu_action(window, app, "Edit", "Select Structure")
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
		_trigger_exposed_menu_action(window, app, "Edit", "Select Structure")
		PySide6.QtTest.QTest.mouseClick(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			canvas.mapFromScene(group_anchor),
		)
		app.processEvents()
		_trigger_exposed_menu_action(window, app, "Chemistry", "Materialize Selected Compact Group")
		report_observer = _MoleculeReportObserver(app)
		_trigger_exposed_menu_action(window, app, "Chemistry", "Molecule Report...")
		details = report_observer.await_completed_details()
		if "Formula: C3H8" not in details:
			raise CompactGroupAuthorToMaterializeE2eError(
				"materialized Me did not expose propane's public selected-molecule formula; "
				"observed accessible dialog text: {0!r}".format(details[:2000]),
			)
		print(json.dumps({"schema": "ferrum-compact-group-author-to-materialize-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	raise SystemExit(main())

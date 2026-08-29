"""Delete one publicly authored Me group, then restore it with public Undo."""

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
class CompactGroupDeleteE2eError(RuntimeError):
	"""One failed public compact-group deletion assertion."""


#============================================
def _trigger_exposed_menu_action(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, menu_label: str, action_label: str) -> None:
	"""Trigger one visible enabled public menu action by its displayed text."""
	menu_action = next(
		action for action in window.menuBar().actions()
		if action.text().replace("&", "") == menu_label
	)
	menu = menu_action.menu()
	if not window.menuBar().isVisible() or not menu_action.isVisible() or menu is None:
		raise CompactGroupDeleteE2eError(
			"Ferrum did not publicly expose the {0} menu".format(menu_label),
		)
	action = next(
		candidate for candidate in menu.actions()
		if candidate.text().replace("&", "") == action_label
	)
	if not action.isVisible() or not action.isEnabled():
		raise CompactGroupDeleteE2eError(
			"{0} -> {1} was not publicly available".format(menu_label, action_label),
		)
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
		raise CompactGroupDeleteE2eError(
			"Draw > Drawing setup > Next Drawing did not open its dialog",
		)
	combo = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QComboBox)
		if widget.isVisible() and widget.accessibleName() == "Next atom"
	)
	editor = combo.lineEdit()
	if editor is None:
		raise CompactGroupDeleteE2eError("Next atom did not expose its text editor")
	PySide6.QtTest.QTest.mouseClick(editor, PySide6.QtCore.Qt.MouseButton.LeftButton)
	PySide6.QtTest.QTest.keyClick(editor, PySide6.QtCore.Qt.Key.Key_A,
		PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
	PySide6.QtTest.QTest.keyClicks(editor, "C")
	PySide6.QtTest.QTest.keyClick(editor, PySide6.QtCore.Qt.Key.Key_Return)
	app.processEvents()
	if combo.currentText() != "C":
		raise CompactGroupDeleteE2eError("the visible Next atom control did not retain C")
	button_box = next(
		box for box in dialog.findChildren(PySide6.QtWidgets.QDialogButtonBox)
		if box.isVisible()
	)
	next(
		button for button in button_box.buttons()
		if button_box.standardButton(button) in (
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok,
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Close,
		)
		and button.isVisible() and button.isEnabled()
	).click()


#============================================
def _choose_next_atom_carbon(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication) -> None:
	"""Set C through the public Draw > Drawing setup > Next Drawing workflow."""
	PySide6.QtCore.QTimer.singleShot(0, lambda: _select_next_atom_carbon(app))
	_trigger_exposed_menu_action(window, app, "Draw", "Next Drawing...")


#============================================
def _choose_me(app: PySide6.QtWidgets.QApplication) -> None:
	"""Choose Me and accept the public compact-group chooser."""
	dialog = app.activeModalWidget()
	if (
		not isinstance(dialog, PySide6.QtWidgets.QDialog)
		or not dialog.isVisible()
		or dialog.accessibleName() != "Attach compact group to selected atom"
	):
		raise CompactGroupDeleteE2eError(
			"Attach Compact Group did not open its accessible chooser",
		)
	choice = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QComboBox)
		if widget.isVisible() and widget.accessibleName() == "Compact group"
	)
	choice_index = choice.findText("Me", PySide6.QtCore.Qt.MatchFlag.MatchExactly)
	if choice_index < 0:
		raise CompactGroupDeleteE2eError(
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
		raise CompactGroupDeleteE2eError("Attach to Selected Atom did not accept Me")


#============================================
class _MoleculeReportObserver(PySide6.QtCore.QObject):
	"""Receive one public Molecule Report dialog and its visible detail text."""

	def __init__(self, app: PySide6.QtWidgets.QApplication) -> None:
		super().__init__(app)
		self.completion_loop = PySide6.QtCore.QEventLoop()
		self.dialog: PySide6.QtWidgets.QDialog | None = None
		self.details: PySide6.QtWidgets.QPlainTextEdit | None = None
		self.observed_text = ""
		app.installEventFilter(self)

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Observe only the visible public Molecule Report dialog."""
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
					if widget.isVisible() and widget.isReadOnly()
					and widget.accessibleName() == "Selected molecule report details"
				),
				None,
			)
			if details is not None and self.details is None:
				self.dialog = watched
				self.details = details
				details.textChanged.connect(self._receive_completed_details)
				self._receive_completed_details()
		return False

	def _receive_completed_details(self) -> None:
		"""Quit once visible public report details arrive."""
		if self.details is None:
			return
		self.observed_text = self.details.toPlainText()
		if self.observed_text:
			self.completion_loop.quit()

	def await_completed_details(self) -> str:
		"""Wait for public report delivery without an arbitrary time limit."""
		if not self.observed_text:
			self.completion_loop.exec()
		if not self.observed_text:
			raise CompactGroupDeleteE2eError("Molecule Report did not display details")
		if self.dialog is not None:
			self.dialog.close()
		return self.observed_text


#============================================
def _select_scene_point(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, canvas: PySide6.QtWidgets.QGraphicsView,
		point: PySide6.QtCore.QPointF) -> None:
	"""Select one visible canvas target through the public structure tool."""
	menu_action = next(
		action for action in window.menuBar().actions()
		if action.text().replace("&", "") == "Draw"
	)
	menu = menu_action.menu()
	if menu is None:
		raise CompactGroupDeleteE2eError("Ferrum did not expose the Draw menu")
	select_action = next(
		action for action in menu.actions()
		if action.text().replace("&", "") == "Select Structure"
	)
	if not select_action.isVisible() or not select_action.isEnabled() or (
		not select_action.isCheckable()
	):
		raise CompactGroupDeleteE2eError(
			"Draw -> Select Structure was not an available canvas tool",
		)
	if not select_action.isChecked():
		select_action.trigger()
		app.processEvents()
	PySide6.QtTest.QTest.mouseClick(
		canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(point),
	)
	app.processEvents()


#============================================
def _molecule_report(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication) -> str:
	"""Open the public report and return its visible semantic details."""
	observer = _MoleculeReportObserver(app)
	_trigger_exposed_menu_action(window, app, "Chemistry", "Molecule Report...")
	return observer.await_completed_details()


#============================================
def main() -> int:
	"""Prove public compact-group deletion receipt and Undo restoration."""
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
		_trigger_exposed_menu_action(window, app, "Draw", "Draw Bond")
		PySide6.QtTest.QTest.mousePress(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(first_carbon),
		)
		PySide6.QtTest.QTest.mouseMove(canvas.viewport(), canvas.mapFromScene(second_carbon))
		PySide6.QtTest.QTest.mouseRelease(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(second_carbon),
		)
		app.processEvents()
		_select_scene_point(window, app, canvas, first_carbon)
		_trigger_exposed_menu_action(window, app, "Draw", "Attach Compact Group...")
		_choose_me(app)
		PySide6.QtTest.QTest.mouseRelease(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(group_anchor),
		)
		app.processEvents()
		_select_scene_point(window, app, canvas, group_anchor)
		PySide6.QtTest.QTest.keyClick(canvas.viewport(), PySide6.QtCore.Qt.Key.Key_Delete)
		app.processEvents()
		deleted_receipt = window.statusBar().currentMessage()
		if deleted_receipt != "Deleted 0 atoms, 1 bonds, and 1 compact groups through Rust.":
			raise CompactGroupDeleteE2eError(
				"Delete did not display the authoritative compact-group receipt; observed "
				"status: {0!r}".format(deleted_receipt),
			)
		_select_scene_point(window, app, canvas, first_carbon)
		deleted_details = _molecule_report(window, app)
		if "Formula: C2H6" not in deleted_details:
			raise CompactGroupDeleteE2eError(
				"deleting Me did not expose ethane through Molecule Report; observed "
				"details: {0!r}".format(deleted_details[:2000]),
			)
		_trigger_exposed_menu_action(window, app, "Edit", "Undo")
		_select_scene_point(window, app, canvas, group_anchor)
		_trigger_exposed_menu_action(window, app, "Chemistry", "Materialize Selected Compact Group")
		restored_details = _molecule_report(window, app)
		if "Formula: C3H8" not in restored_details:
			raise CompactGroupDeleteE2eError(
				"Undo did not restore the public Me behavior; observed details: {0!r}".format(
					restored_details[:2000],
				),
			)
		print(json.dumps({"schema": "ferrum-compact-group-delete-e2e-v1", "status": "ok"}))
		return 0
	finally:
		ferrum_qt_e2e.close_e2e_main_window(window, app)


if __name__ == "__main__":
	raise SystemExit(main())

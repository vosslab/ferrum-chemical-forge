"""Recover from an unavailable attached-Me anchor through public Qt controls."""

# Standard Library
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
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
REFUSAL_TITLE = "Action Not Available"
REFUSAL_BODY = "Me cannot attach to the selected atom. Select another atom and try again."
CHOOSER_NAME = "Attach compact group to selected atom"
# This E2E-only guard turns a missing asynchronous public report completion into
# a diagnostic failure. It protects the runner from deadlock; it is not a
# product performance requirement or a timing assertion.
REPORT_WAIT_DEADLOCK_GUARD_MILLISECONDS = 15000


#============================================
class CompactGroupUnavailableAnchorRecoveryE2eError(RuntimeError):
	"""One failed public unavailable-anchor recovery assertion."""


#============================================
def _exposed_menu_action(window: PySide6.QtWidgets.QMainWindow,
		menu_path: tuple[str, ...], action_label: str) -> PySide6.QtGui.QAction:
	"""Return one visible public action by its displayed menu path and text."""
	menu_bar = window.menuBar()
	menu_path_text = " > ".join(menu_path)
	menu_action = next(
		action for action in menu_bar.actions()
		if action.text().replace("&", "") == menu_path[0]
	)
	menu = menu_action.menu()
	if not menu_bar.isVisible() or not menu_action.isVisible() or menu is None:
		raise CompactGroupUnavailableAnchorRecoveryE2eError(
			"Ferrum did not publicly expose the {0} menu".format(menu_path[0]),
		)
	action = next(
		candidate for candidate in menu.actions()
		if candidate.text().replace("&", "") == action_label
	)
	if not action.isVisible():
		raise CompactGroupUnavailableAnchorRecoveryE2eError(
			"{0} -> {1} was not visibly exposed".format(menu_path_text, action_label),
		)
	return action


#============================================
def _trigger_exposed_menu_action(window: PySide6.QtWidgets.QMainWindow,
			app: PySide6.QtWidgets.QApplication, menu_path: tuple[str, ...],
			action_label: str) -> None:
	"""Trigger one visible enabled public menu action by its displayed text."""
	menu_path_text = " > ".join(menu_path)
	action = _exposed_menu_action(window, menu_path, action_label)
	if not action.isEnabled():
		raise CompactGroupUnavailableAnchorRecoveryE2eError(
			"{0} -> {1} was not publicly available".format(menu_path_text, action_label),
		)
	action.trigger()
	app.processEvents()


#============================================
def _activate_exposed_tool_action(window: PySide6.QtWidgets.QMainWindow,
			app: PySide6.QtWidgets.QApplication, menu_path: tuple[str, ...],
			action_label: str) -> None:
	"""Ensure one visible checkable canvas tool owns the next public gesture."""
	menu_path_text = " > ".join(menu_path)
	action = _exposed_menu_action(window, menu_path, action_label)
	if not action.isEnabled() or not action.isCheckable():
		raise CompactGroupUnavailableAnchorRecoveryE2eError(
			"{0} -> {1} was not an available canvas tool".format(menu_path_text, action_label),
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
def _set_next_atom(app: PySide6.QtWidgets.QApplication, element: str) -> None:
	"""Set one visible Next Drawing atom choice."""
	dialog = app.activeModalWidget()
	if (
		not isinstance(dialog, PySide6.QtWidgets.QDialog)
		or not dialog.isVisible()
		or dialog.accessibleName() != "Next Drawing"
	):
		raise CompactGroupUnavailableAnchorRecoveryE2eError(
			"Draw > Drawing setup > Next Drawing did not open its visible public dialog",
		)
	combo = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QComboBox)
		if widget.isVisible() and widget.accessibleName() == "Next atom"
	)
	editor = combo.lineEdit()
	if editor is None:
		raise CompactGroupUnavailableAnchorRecoveryE2eError(
			"Next atom did not expose its visible text editor",
		)
	PySide6.QtTest.QTest.mouseClick(editor, PySide6.QtCore.Qt.MouseButton.LeftButton)
	PySide6.QtTest.QTest.keyClick(editor, PySide6.QtCore.Qt.Key.Key_A,
		PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
	PySide6.QtTest.QTest.keyClicks(editor, element)
	PySide6.QtTest.QTest.keyClick(editor, PySide6.QtCore.Qt.Key.Key_Return)
	app.processEvents()
	if combo.currentText() != element:
		raise CompactGroupUnavailableAnchorRecoveryE2eError(
			"the visible Next atom control did not retain {0}".format(element),
		)
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
def _choose_next_atom(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, element: str) -> None:
	"""Set one atom through the public Draw menu workflow."""
	PySide6.QtCore.QTimer.singleShot(0, lambda: _set_next_atom(app, element))
	_trigger_exposed_menu_action(window, app, ("Draw",), "Next Drawing...")


#============================================
def _draw_single_bond(canvas: PySide6.QtWidgets.QGraphicsView,
		app: PySide6.QtWidgets.QApplication, start: PySide6.QtCore.QPointF,
		end: PySide6.QtCore.QPointF) -> None:
	"""Complete one visible single-bond drag from a public canvas point."""
	PySide6.QtTest.QTest.mousePress(
		canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(start),
	)
	PySide6.QtTest.QTest.mouseMove(canvas.viewport(), canvas.mapFromScene(end))
	PySide6.QtTest.QTest.mouseRelease(
		canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(end),
	)
	app.processEvents()


#============================================
def _select_scene_point(window: PySide6.QtWidgets.QMainWindow,
				app: PySide6.QtWidgets.QApplication, canvas: PySide6.QtWidgets.QGraphicsView,
				point: PySide6.QtCore.QPointF) -> None:
	"""Select one visible canvas target through the public Draw menu."""
	_activate_exposed_tool_action(
		window, app, ("Draw",), "Select Structure",
	)
	PySide6.QtTest.QTest.mouseClick(
		canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(point),
	)
	app.processEvents()


#============================================
def _visible_dialog_text(dialog: PySide6.QtWidgets.QDialog) -> tuple[str, ...]:
	"""Return visible public text from one dialog without private inspection."""
	return tuple(
		text for text in (
			dialog.accessibleName(), dialog.windowTitle(),
			*(widget.text() for widget in dialog.findChildren(PySide6.QtWidgets.QLabel)
				if widget.isVisible()),
		) if text
	)


#============================================
class _AttachDialogObserver(PySide6.QtCore.QObject):
	"""Observe public attachment outcomes and promptly reject unexpected modals."""

	def __init__(self, app: PySide6.QtWidgets.QApplication,
			expect_refusal: bool, second_activation: PySide6.QtGui.QAction | None = None) -> None:
		super().__init__(app)
		self.app = app
		self.expect_refusal = expect_refusal
		self.second_activation = second_activation
		self.chooser: PySide6.QtWidgets.QDialog | None = None
		self.refusal: PySide6.QtWidgets.QMessageBox | None = None
		self.refusal_visible_text: tuple[str, ...] = ()
		self.dismissal_scheduled: list[PySide6.QtWidgets.QMessageBox] = []
		self.dismissal_completed: list[PySide6.QtWidgets.QMessageBox] = []
		self.rejection_scheduled: list[PySide6.QtWidgets.QDialog] = []
		self.failure = ""
		app.installEventFilter(self)

	def close(self) -> None:
		"""Stop observing once this public attachment interaction has completed."""
		self.app.removeEventFilter(self)

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Capture accessible chooser/refusal identity and public visible text."""
		if (
			event.type() != PySide6.QtCore.QEvent.Type.Show
			or not isinstance(watched, PySide6.QtWidgets.QDialog)
		):
			return False
		if watched.accessibleName() == CHOOSER_NAME:
			if self.expect_refusal:
				self.failure = "Attach Compact Group opened a chooser for saturated CH4"
			elif self.chooser is None:
				self.chooser = watched
				if self.second_activation is not None:
					PySide6.QtCore.QTimer.singleShot(0, self.second_activation.trigger)
			else:
				self.failure = "the queued second activation replaced the original chooser"
		elif isinstance(watched, PySide6.QtWidgets.QMessageBox):
			if not self.expect_refusal:
				if not self.failure:
					self.failure = (
						"Attach Compact Group opened an unexpected public refusal: {0!r}"
					).format(_visible_dialog_text(watched))
			elif self.refusal is None:
				self.refusal = watched
			else:
				self.failure = "Attach Compact Group opened more than one public refusal"
			self._schedule_refusal_dismissal(watched)
		else:
			if not self.failure:
				self.failure = "Attach Compact Group opened an unexpected public modal: {0!r}".format(
					_visible_dialog_text(watched),
				)
			self._schedule_dialog_rejection(watched)
		return False

	def _schedule_dialog_rejection(self, dialog: PySide6.QtWidgets.QDialog) -> None:
		"""Queue one public rejection so an unexpected modal cannot stall the E2E."""
		if any(scheduled is dialog for scheduled in self.rejection_scheduled):
			return
		self.rejection_scheduled.append(dialog)
		PySide6.QtCore.QTimer.singleShot(0, dialog.reject)

	def _schedule_refusal_dismissal(self, dialog: PySide6.QtWidgets.QMessageBox) -> None:
		"""Schedule exactly one public dismissal for each observed refusal dialog."""
		if any(scheduled is dialog for scheduled in self.dismissal_scheduled):
			return
		self.dismissal_scheduled.append(dialog)
		PySide6.QtCore.QTimer.singleShot(0, lambda: self._dismiss_refusal(dialog))

	def _dismiss_refusal(self, dialog: PySide6.QtWidgets.QMessageBox) -> None:
		"""Observe then click a refusal's public OK button in its modal event loop."""
		visible_text = _visible_dialog_text(dialog)
		if dialog is self.refusal:
			self.refusal_visible_text = visible_text
		if not self.expect_refusal:
			if not self.failure:
				self.failure = "Attach Compact Group opened an unexpected public refusal: {0!r}".format(
					visible_text,
				)
		elif dialog is self.refusal and dialog.accessibleName() != REFUSAL_TITLE:
			if not self.failure:
				self.failure = (
					"Attach Compact Group refusal accessibility identity mismatch: expected {0!r}; "
					"observed {1!r} with visible text {2!r}"
				).format(REFUSAL_TITLE, dialog.accessibleName(), visible_text)
		elif dialog is self.refusal and not any(
				REFUSAL_BODY in text for text in visible_text
				):
			if not self.failure:
				self.failure = (
					"Attach Compact Group refusal body mismatch: expected {0!r}; "
					"observed visible text {1!r}"
				).format(REFUSAL_BODY, visible_text)
		button = dialog.button(PySide6.QtWidgets.QMessageBox.StandardButton.Ok)
		if button is None or not button.isVisible() or not button.isEnabled():
			if not self.failure:
				self.failure = "the standard attachment refusal did not expose an enabled OK button"
			dialog.reject()
			return
		PySide6.QtTest.QTest.mouseClick(button, PySide6.QtCore.Qt.MouseButton.LeftButton)
		self.app.processEvents()
		if dialog.isVisible():
			if not self.failure:
				self.failure = "the public attachment-refusal OK button did not dismiss its dialog"
			return
		self.dismissal_completed.append(dialog)

	def require_refusal(self) -> PySide6.QtWidgets.QMessageBox:
		"""Return the observed standard accessible refusal or fail promptly."""
		if self.failure:
			raise CompactGroupUnavailableAnchorRecoveryE2eError(self.failure)
		if self.refusal is None:
			raise CompactGroupUnavailableAnchorRecoveryE2eError(
				"Attach Compact Group did not show the standard accessible refusal",
			)
		if not any(scheduled is self.refusal for scheduled in self.dismissal_scheduled):
			raise CompactGroupUnavailableAnchorRecoveryE2eError(
				"the standard accessible refusal did not schedule its public dismissal",
			)
		if not any(completed is self.refusal for completed in self.dismissal_completed):
			raise CompactGroupUnavailableAnchorRecoveryE2eError(
				"the standard accessible refusal did not complete its public dismissal",
			)
		return self.refusal

	def require_chooser(self) -> PySide6.QtWidgets.QDialog:
		"""Return the original chooser after queued events complete."""
		if self.failure:
			raise CompactGroupUnavailableAnchorRecoveryE2eError(self.failure)
		if self.chooser is None:
			raise CompactGroupUnavailableAnchorRecoveryE2eError(
				"Attach Compact Group did not open its accessible chooser",
			)
		return self.chooser


#============================================
def _choose_me(dialog: PySide6.QtWidgets.QDialog,
		app: PySide6.QtWidgets.QApplication) -> None:
	"""Choose Me and accept the original accessible compact-group chooser."""
	if not dialog.isVisible() or dialog.accessibleName() != CHOOSER_NAME:
		raise CompactGroupUnavailableAnchorRecoveryE2eError(
			"the original accessible compact-group chooser did not remain open",
		)
	choice = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QComboBox)
		if widget.isVisible() and widget.accessibleName() == "Compact group"
	)
	choice_index = choice.findText("Me", PySide6.QtCore.Qt.MatchFlag.MatchExactly)
	if choice_index < 0:
		raise CompactGroupUnavailableAnchorRecoveryE2eError(
			"the visible compact-group chooser did not offer Me",
		)
	choice.setCurrentIndex(choice_index)
	confirm = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QPushButton)
		if widget.isVisible() and widget.accessibleName() == "Attach to Selected Atom"
	)
	PySide6.QtTest.QTest.mouseClick(confirm, PySide6.QtCore.Qt.MouseButton.LeftButton)
	app.processEvents()
	if dialog.isVisible():
		raise CompactGroupUnavailableAnchorRecoveryE2eError(
			"Attach to Selected Atom did not accept the Me chooser",
		)


#============================================
class _MoleculeReportObserver(PySide6.QtCore.QObject):
	"""Receive one public Molecule Report dialog and fail on a public refusal."""

	def __init__(self, app: PySide6.QtWidgets.QApplication) -> None:
		super().__init__(app)
		self.app = app
		self.completion_loop = PySide6.QtCore.QEventLoop()
		self.dialog: PySide6.QtWidgets.QDialog | None = None
		self.details: PySide6.QtWidgets.QPlainTextEdit | None = None
		self.observed_text = ""
		self.failure = ""
		self.closed = False
		self.closed_dialogs: list[PySide6.QtWidgets.QDialog] = []
		self.deadlock_guard = PySide6.QtCore.QTimer(self)
		self.deadlock_guard.setSingleShot(True)
		self.deadlock_guard.timeout.connect(self._fail_deadlock_guard)
		app.installEventFilter(self)

	def close(self) -> None:
		"""Release the event filter and report guard on every observer exit path."""
		if self.closed:
			return
		self.closed = True
		self.deadlock_guard.stop()
		self.app.removeEventFilter(self)

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Observe the report, failing immediately when another public modal appears."""
		if (
			event.type() != PySide6.QtCore.QEvent.Type.Show
			or not isinstance(watched, PySide6.QtWidgets.QDialog)
		):
			return False
		if watched.accessibleName() == "Molecule Report":
			details = next(
			(
				widget for widget in watched.findChildren(PySide6.QtWidgets.QPlainTextEdit)
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
		else:
			self.failure = "Molecule Report opened an unexpected public modal: {0!r}".format(
				_visible_dialog_text(watched),
			)
			self._schedule_dialog_close(watched)
			self.completion_loop.quit()
		return False

	def _schedule_dialog_close(self, dialog: PySide6.QtWidgets.QDialog) -> None:
		"""Queue closure of an unexpected modal so its synchronous loop returns."""
		if any(closed is dialog for closed in self.closed_dialogs):
			return
		self.closed_dialogs.append(dialog)
		PySide6.QtCore.QTimer.singleShot(0, dialog.reject)

	def _fail_deadlock_guard(self) -> None:
		"""Fail the asynchronous report phase rather than leave the E2E waiting forever."""
		if not self.observed_text and not self.failure:
			self.failure = (
				"Molecule Report did not complete visible semantic details before the "
				"E2E deadlock guard expired"
			)
			self.completion_loop.quit()

	def _receive_completed_details(self) -> None:
		"""Quit only after the visible public report details have text."""
		if self.details is None:
			return
		self.observed_text = self.details.toPlainText()
		if self.observed_text:
			self.completion_loop.quit()
			if self.dialog is not None:
				self.dialog.accept()

	def await_completed_details(self) -> str:
		"""Return public details, with a deadlock guard for this asynchronous phase."""
		self.deadlock_guard.start(REPORT_WAIT_DEADLOCK_GUARD_MILLISECONDS)
		try:
			if not self.observed_text and not self.failure:
				self.completion_loop.exec()
			if self.failure:
				raise CompactGroupUnavailableAnchorRecoveryE2eError(self.failure)
			if not self.observed_text:
				raise CompactGroupUnavailableAnchorRecoveryE2eError(
					"Molecule Report did not display visible semantic details",
				)
			return self.observed_text
		finally:
			if self.dialog is not None and self.dialog.isVisible():
				self.dialog.close()
			self.close()


#============================================
def _molecule_report(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication) -> str:
	"""Open Molecule Report and return its visible semantic details."""
	observer = _MoleculeReportObserver(app)
	try:
		_trigger_exposed_menu_action(
			window, app, ("Chemistry",), "Molecule Report...",
		)
		return observer.await_completed_details()
	finally:
		observer.close()


#============================================
def main() -> int:
	"""Prove refused saturated-anchor attachment and in-document eligible recovery."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		window.show()
		app.processEvents()
		_trigger_exposed_menu_action(window, app, ("File",), "New")
		canvas = _canvas(window)
		methane_center = PySide6.QtCore.QPointF(90.0, 90.0)
		temporary_carbon = PySide6.QtCore.QPointF(155.0, 90.0)
		hydrogen_points = (
			PySide6.QtCore.QPointF(90.0, 35.0),
			PySide6.QtCore.QPointF(35.0, 90.0),
			PySide6.QtCore.QPointF(145.0, 90.0),
			PySide6.QtCore.QPointF(90.0, 145.0),
		)
		eligible_first_carbon = PySide6.QtCore.QPointF(210.0, 55.0)
		eligible_second_carbon = PySide6.QtCore.QPointF(270.0, 55.0)
		group_anchor = PySide6.QtCore.QPointF(210.0, 140.0)

		_choose_next_atom(window, app, "C")
		_activate_exposed_tool_action(window, app, ("Draw",), "Draw Bond")
		_draw_single_bond(canvas, app, methane_center, temporary_carbon)
		_select_scene_point(window, app, canvas, temporary_carbon)
		PySide6.QtTest.QTest.keyClick(canvas.viewport(), PySide6.QtCore.Qt.Key.Key_Delete)
		app.processEvents()
		_choose_next_atom(window, app, "H")
		_activate_exposed_tool_action(window, app, ("Draw",), "Draw Bond")
		for hydrogen in hydrogen_points:
			_draw_single_bond(canvas, app, methane_center, hydrogen)
		_choose_next_atom(window, app, "C")
		_activate_exposed_tool_action(window, app, ("Draw",), "Draw Bond")
		_draw_single_bond(canvas, app, eligible_first_carbon, eligible_second_carbon)

		_select_scene_point(window, app, canvas, methane_center)
		attach_action = _exposed_menu_action(
			window, ("Draw",), "Attach Compact Group...",
		)
		if not attach_action.isEnabled():
			raise CompactGroupUnavailableAnchorRecoveryE2eError(
				"Attach Compact Group was not enabled for the selected saturated carbon",
			)
		refusal_observer = _AttachDialogObserver(app, expect_refusal=True)
		attach_action.trigger()
		app.processEvents()
		refusal_observer.require_refusal()
		refusal_observer.close()
		_select_scene_point(window, app, canvas, eligible_first_carbon)
		attach_action = _exposed_menu_action(
			window, ("Draw",), "Attach Compact Group...",
		)
		if not attach_action.isEnabled():
			raise CompactGroupUnavailableAnchorRecoveryE2eError(
				"Attach Compact Group did not recover for the selected eligible atom: {0}".format(
					attach_action.statusTip(),
				),
			)
		chooser_observer = _AttachDialogObserver(
			app, expect_refusal=False, second_activation=attach_action,
		)
		attach_action.trigger()
		app.processEvents()
		_choose_me(chooser_observer.require_chooser(), app)
		if chooser_observer.failure:
			raise CompactGroupUnavailableAnchorRecoveryE2eError(chooser_observer.failure)
		chooser_observer.close()
		PySide6.QtTest.QTest.mouseRelease(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(group_anchor),
		)
		app.processEvents()
		_select_scene_point(window, app, canvas, group_anchor)
		_trigger_exposed_menu_action(
			window, app, ("Chemistry",), "Materialize Selected Compact Group",
		)
		propane_details = _molecule_report(window, app)
		if "Formula: C3H8" not in propane_details:
			raise CompactGroupUnavailableAnchorRecoveryE2eError(
				"the eligible recovery did not materialize to propane; observed details: {0!r}".format(
					propane_details[:2000],
				),
			)
		print(json.dumps({
			"schema": "ferrum-compact-group-unavailable-anchor-recovery-e2e-v1",
			"status": "ok",
		}))
		return 0
	finally:
		ferrum_qt_e2e.close_e2e_main_window(window, app)


if __name__ == "__main__":
	raise SystemExit(main())

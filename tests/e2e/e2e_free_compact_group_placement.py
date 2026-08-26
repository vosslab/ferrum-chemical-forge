#!/usr/bin/env python3
"""Place free Me through Qt, then materialize it through the public workflow."""

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
# This E2E-only guard turns a missing asynchronous report completion into a
# diagnostic failure. It prevents an unrecoverable modal wait; it is not a
# product performance requirement or timing assertion.
REPORT_WAIT_DEADLOCK_GUARD_MILLISECONDS = 15000


#============================================
class FreeCompactGroupPlacementE2eError(RuntimeError):
	"""One failed public free-Me placement workflow assertion."""


#============================================
def _exposed_action(window: PySide6.QtWidgets.QMainWindow, menu_label: str,
		action_label: str) -> PySide6.QtGui.QAction:
	"""Return one visible enabled public action by its displayed menu labels."""
	menu_action = next(
		action for action in window.menuBar().actions()
		if action.text().replace("&", "") == menu_label
	)
	menu = menu_action.menu()
	if not window.menuBar().isVisible() or not menu_action.isVisible() or menu is None:
		raise FreeCompactGroupPlacementE2eError(
			f"Ferrum did not publicly expose the {menu_label} menu",
		)
	action = next(
		candidate for candidate in menu.actions()
		if candidate.text().replace("&", "") == action_label
	)
	if not action.isVisible() or not action.isEnabled():
		raise FreeCompactGroupPlacementE2eError(
			f"{menu_label} -> {action_label} was not publicly available",
		)
	return action


#============================================
def _trigger_action(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, menu_label: str, action_label: str) -> None:
	"""Trigger one exposed public action and deliver its queued Qt work."""
	_exposed_action(window, menu_label, action_label).trigger()
	app.processEvents()


#============================================
def _canvas(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtWidgets.QGraphicsView:
	"""Return Ferrum's visible, accessibly named drawing canvas."""
	return next(
		view for view in window.findChildren(PySide6.QtWidgets.QGraphicsView)
		if view.isVisible() and view.accessibleName() == "Ferrum drawing canvas"
	)


#============================================
def _choose_free_me(app: PySide6.QtWidgets.QApplication) -> None:
	"""Confirm the accessible Me-only chooser using its public controls."""
	dialog = app.activeModalWidget()
	if (
		not isinstance(dialog, PySide6.QtWidgets.QDialog)
		or not dialog.isVisible()
		or dialog.accessibleName() != "Place compact group on canvas"
	):
		raise FreeCompactGroupPlacementE2eError(
			"Place Compact Group did not open its accessible chooser",
		)
	me_option = next(
		(
			widget for widget in dialog.findChildren(PySide6.QtWidgets.QLabel)
			if widget.isVisible() and widget.accessibleName() == "Compact group Me"
		),
		None,
	)
	if me_option is None:
		raise FreeCompactGroupPlacementE2eError(
			"the visible free compact-group chooser did not offer Me",
		)
	confirm = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QPushButton)
		if widget.isVisible() and widget.accessibleName() == "Place on Canvas"
	)
	PySide6.QtTest.QTest.mouseClick(confirm, PySide6.QtCore.Qt.MouseButton.LeftButton)
	app.processEvents()
	if dialog.isVisible():
		raise FreeCompactGroupPlacementE2eError(
			"Place on Canvas did not accept the Me chooser",
		)


#============================================
def _visible_dialog_text(dialog: PySide6.QtWidgets.QDialog) -> tuple[str, ...]:
	"""Return visible user-facing text from one dialog for failure evidence."""
	return tuple(
		text for text in (
			dialog.accessibleName(), dialog.windowTitle(),
			*(widget.text() for widget in dialog.findChildren(PySide6.QtWidgets.QLabel)
				if widget.isVisible()),
		) if text
	)


#============================================
class _MoleculeReportObserver(PySide6.QtCore.QObject):
	"""Collect one visible report and reject unexpected modals without a hang."""

	def __init__(self, app: PySide6.QtWidgets.QApplication) -> None:
		super().__init__(app)
		self.app = app
		self.completion_loop = PySide6.QtCore.QEventLoop()
		self.report: PySide6.QtWidgets.QDialog | None = None
		self.details: PySide6.QtWidgets.QPlainTextEdit | None = None
		self.report_text = ""
		self.failure = ""
		self.rejected: list[PySide6.QtWidgets.QDialog] = []
		app.installEventFilter(self)

	def close(self) -> None:
		"""Remove the observer after this one public report interaction."""
		self.app.removeEventFilter(self)

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Receive the public report dialog or schedule a safe diagnostic dismissal."""
		if (
			event.type() != PySide6.QtCore.QEvent.Type.Show
			or not isinstance(watched, PySide6.QtWidgets.QDialog)
		):
			return False
		if watched.accessibleName() != "Molecule Report":
			if not self.failure:
				self.failure = (
					f"Chemistry -> Molecule Report opened an unexpected modal: "
					f"{_visible_dialog_text(watched)!r}"
				)
			self._schedule_rejection(watched)
			return False
		details = next(
			(
				widget for widget in watched.findChildren(PySide6.QtWidgets.QPlainTextEdit)
				if widget.isVisible() and widget.isReadOnly()
				and widget.accessibleName() == "Selected molecule report details"
			),
			None,
		)
		if details is None:
			self.failure = "Molecule Report did not expose its visible details control"
			self._schedule_rejection(watched)
		elif self.details is None:
			self.report = watched
			self.details = details
			details.textChanged.connect(self._receive_report_text)
			self._receive_report_text()
		return False

	def _schedule_rejection(self, dialog: PySide6.QtWidgets.QDialog) -> None:
		"""Queue one public rejection so a wrong modal cannot block CI."""
		if any(existing is dialog for existing in self.rejected):
			return
		self.rejected.append(dialog)
		PySide6.QtCore.QTimer.singleShot(0, dialog.reject)

	def _receive_report_text(self) -> None:
		"""Complete once the visible report field receives content."""
		if self.details is None:
			return
		self.report_text = self.details.toPlainText()
		if self.report_text:
			self.completion_loop.quit()

	def _report_wait_deadlock(self) -> None:
		"""End the event loop with evidence if the report delivery stalls."""
		if not self.report_text and not self.failure:
			self.failure = "Molecule Report did not complete its visible details before the liveness guard"
		self.completion_loop.quit()

	def await_report_text(self) -> str:
		"""Wait for visible report content with a liveness-only escape hatch."""
		guard = PySide6.QtCore.QTimer(self)
		guard.setSingleShot(True)
		guard.timeout.connect(self._report_wait_deadlock)
		guard.start(REPORT_WAIT_DEADLOCK_GUARD_MILLISECONDS)
		if not self.report_text and not self.failure:
			self.completion_loop.exec()
		guard.stop()
		if self.failure:
			raise FreeCompactGroupPlacementE2eError(self.failure)
		if not self.report_text:
			raise FreeCompactGroupPlacementE2eError(
				"Molecule Report did not display public details",
			)
		return self.report_text


#============================================
def main() -> int:
	"""Prove blank-document free-Me placement, materialization, and report output."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		window.show()
		app.processEvents()
		_trigger_action(window, app, "File", "New")
		canvas = _canvas(window)
		_trigger_action(window, app, "Draw", "Place Compact Group...")
		_choose_free_me(app)
		placement_point = PySide6.QtCore.QPointF(80.0, 80.0)
		PySide6.QtTest.QTest.mouseRelease(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			canvas.mapFromScene(placement_point),
		)
		app.processEvents()
		_exposed_action(window, "Chemistry", "Materialize Selected Compact Group")
		_trigger_action(window, app, "Chemistry", "Materialize Selected Compact Group")
		observer = _MoleculeReportObserver(app)
		try:
			_trigger_action(window, app, "Chemistry", "Molecule Report...")
			report_text = observer.await_report_text()
		finally:
			observer.close()
		if "Authored graph: 1 atoms, 0 bonds" not in report_text:
			raise FreeCompactGroupPlacementE2eError(
				"materialized free Me did not expose its one-atom replacement graph; "
				f"observed report: {report_text[:2000]!r}",
			)
		if "Formula: CH4" not in report_text:
			raise FreeCompactGroupPlacementE2eError(
				f"materialized free Me did not expose Formula: CH4; observed report: "
				f"{report_text[:2000]!r}",
			)
		print(json.dumps({"schema": "ferrum-free-compact-group-placement-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	raise SystemExit(main())

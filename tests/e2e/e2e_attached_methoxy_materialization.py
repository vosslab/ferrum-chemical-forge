#!/usr/bin/env python3
"""Author and materialize attached OMe through Ferrum's public Qt workflow."""

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
# This E2E-only guard releases a stalled report wait with diagnostics. It is
# liveness protection, not a product responsiveness requirement.
REPORT_WAIT_DEADLOCK_GUARD_MILLISECONDS = 15000
CHOOSER_NAME = "Attach compact group to selected atom"


#============================================
class AttachedMethoxyMaterializationE2eError(RuntimeError):
	"""One failed public attached-OMe authoring and materialization assertion."""


#============================================
def _exposed_action(window: PySide6.QtWidgets.QMainWindow, menu_label: str,
		action_label: str) -> PySide6.QtGui.QAction:
	"""Return one visible, enabled public menu action by displayed labels."""
	menu_bar = window.menuBar()
	menu_action = next(
		action for action in menu_bar.actions()
		if action.text().replace("&", "") == menu_label
	)
	menu = menu_action.menu()
	if not menu_bar.isVisible() or not menu_action.isVisible() or menu is None:
		raise AttachedMethoxyMaterializationE2eError(
			f"Ferrum did not publicly expose the {menu_label} menu",
		)
	action = next(
		candidate for candidate in menu.actions()
		if candidate.text().replace("&", "") == action_label
	)
	if not action.isVisible() or not action.isEnabled():
		raise AttachedMethoxyMaterializationE2eError(
			f"{menu_label} -> {action_label} was not publicly available",
		)
	return action


#============================================
def _trigger_action(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, menu_label: str, action_label: str) -> None:
	"""Trigger one public action and deliver its queued Qt work."""
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
def _set_next_atom_carbon(app: PySide6.QtWidgets.QApplication) -> None:
	"""Set C through the visible public Next Drawing dialog."""
	dialog = app.activeModalWidget()
	if (
		not isinstance(dialog, PySide6.QtWidgets.QDialog)
		or not dialog.isVisible()
		or dialog.accessibleName() != "Next Drawing"
	):
		raise AttachedMethoxyMaterializationE2eError(
			"Draw > Drawing setup > Next Drawing did not open its visible public dialog",
		)
	choice = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QComboBox)
		if widget.isVisible() and widget.accessibleName() == "Next atom"
	)
	editor = choice.lineEdit()
	if editor is None:
		raise AttachedMethoxyMaterializationE2eError(
			"Next atom did not expose its visible text editor",
		)
	PySide6.QtTest.QTest.mouseClick(editor, PySide6.QtCore.Qt.MouseButton.LeftButton)
	PySide6.QtTest.QTest.keyClick(editor, PySide6.QtCore.Qt.Key.Key_A,
		PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
	PySide6.QtTest.QTest.keyClicks(editor, "C")
	PySide6.QtTest.QTest.keyClick(editor, PySide6.QtCore.Qt.Key.Key_Return)
	if choice.currentText() != "C":
		raise AttachedMethoxyMaterializationE2eError(
			"the visible Next atom control did not retain C",
		)
	button_box = next(
		box for box in dialog.findChildren(PySide6.QtWidgets.QDialogButtonBox)
		if box.isVisible()
	)
	next(
		button for button in button_box.buttons()
		if button_box.standardButton(button)
		in (PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok,
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Close)
		and button.isVisible() and button.isEnabled()
	).click()


#============================================
def _choose_next_atom_carbon(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication) -> None:
	"""Choose C through the public Draw > Drawing setup > Next Drawing workflow."""
	PySide6.QtCore.QTimer.singleShot(0, lambda: _set_next_atom_carbon(app))
	_trigger_action(window, app, "Draw", "Next Drawing...")


#============================================
def _select_scene_point(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, canvas: PySide6.QtWidgets.QGraphicsView,
		point: PySide6.QtCore.QPointF) -> None:
	"""Select one visible canvas target through the public selection tool."""
	_trigger_action(window, app, "Draw", "Select Structure")
	PySide6.QtTest.QTest.mouseClick(
		canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(point),
	)
	app.processEvents()


#============================================
def _choose_ome(app: PySide6.QtWidgets.QApplication) -> None:
	"""Choose the visible OMe choice without assuming chooser position or size."""
	dialog = app.activeModalWidget()
	if (
		not isinstance(dialog, PySide6.QtWidgets.QDialog)
		or not dialog.isVisible()
		or dialog.accessibleName() != CHOOSER_NAME
	):
		raise AttachedMethoxyMaterializationE2eError(
			"Attach Compact Group did not open its accessible chooser",
		)
	choice = next(
		(
			widget for widget in dialog.findChildren(PySide6.QtWidgets.QComboBox)
			if widget.isVisible() and widget.accessibleName() == "Compact group"
		),
		None,
	)
	if choice is None:
		raise AttachedMethoxyMaterializationE2eError(
			"the Rust-projected compact-group chooser did not expose its visible selector",
		)
	choice_index = choice.findText("OMe", PySide6.QtCore.Qt.MatchFlag.MatchExactly)
	if choice_index < 0:
		raise AttachedMethoxyMaterializationE2eError(
			"the Rust-projected compact-group chooser did not visibly offer OMe",
		)
	choice.setCurrentIndex(choice_index)
	confirm = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QPushButton)
		if widget.isVisible() and widget.accessibleName() == "Attach to Selected Atom"
	)
	PySide6.QtTest.QTest.mouseClick(confirm, PySide6.QtCore.Qt.MouseButton.LeftButton)
	app.processEvents()
	if dialog.isVisible():
		raise AttachedMethoxyMaterializationE2eError(
			"Attach to Selected Atom did not accept the OMe chooser",
		)


#============================================
def _visible_dialog_text(dialog: PySide6.QtWidgets.QDialog) -> tuple[str, ...]:
	"""Return public dialog text for an actionable liveness failure."""
	return tuple(
		text for text in (
			dialog.accessibleName(), dialog.windowTitle(),
			*(widget.text() for widget in dialog.findChildren(PySide6.QtWidgets.QLabel)
				if widget.isVisible()),
		) if text
	)


#============================================
class _MoleculeReportObserver(PySide6.QtCore.QObject):
	"""Collect one public report and diagnose an unexpected modal deadlock."""

	def __init__(self, app: PySide6.QtWidgets.QApplication) -> None:
		super().__init__(app)
		self.app = app
		self.completion_loop = PySide6.QtCore.QEventLoop()
		self.details: PySide6.QtWidgets.QPlainTextEdit | None = None
		self.report_text = ""
		self.failure = ""
		self.rejected: list[PySide6.QtWidgets.QDialog] = []
		app.installEventFilter(self)

	def close(self) -> None:
		"""Remove the observer after the one report interaction completes."""
		self.app.removeEventFilter(self)

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Receive the expected report or queue public rejection for another modal."""
		if (
			event.type() != PySide6.QtCore.QEvent.Type.Show
			or not isinstance(watched, PySide6.QtWidgets.QDialog)
		):
			return False
		if watched.accessibleName() != "Molecule Report":
			if not self.failure:
				self.failure = (
					"Molecule Report opened an unexpected public modal: "
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
			self.details = details
			details.textChanged.connect(self._receive_report_text)
			self._receive_report_text()
		return False

	def _schedule_rejection(self, dialog: PySide6.QtWidgets.QDialog) -> None:
		"""Queue one public rejection so an unexpected modal cannot block the E2E."""
		if any(existing is dialog for existing in self.rejected):
			return
		self.rejected.append(dialog)
		PySide6.QtCore.QTimer.singleShot(0, dialog.reject)

	def _receive_report_text(self) -> None:
		"""Complete after the visible report details contain text."""
		if self.details is None:
			return
		self.report_text = self.details.toPlainText()
		if self.report_text:
			self.completion_loop.quit()

	def _report_wait_deadlock(self) -> None:
		"""End a stalled report phase with evidence rather than hanging the runner."""
		if not self.report_text and not self.failure:
			self.failure = "Molecule Report did not complete its visible details before the liveness guard"
		self.completion_loop.quit()

	def await_report_text(self) -> str:
		"""Wait for public details with a liveness-only escape hatch."""
		guard = PySide6.QtCore.QTimer(self)
		guard.setSingleShot(True)
		guard.timeout.connect(self._report_wait_deadlock)
		guard.start(REPORT_WAIT_DEADLOCK_GUARD_MILLISECONDS)
		if not self.report_text and not self.failure:
			self.completion_loop.exec()
		guard.stop()
		if self.failure:
			raise AttachedMethoxyMaterializationE2eError(self.failure)
		if not self.report_text:
			raise AttachedMethoxyMaterializationE2eError(
				"Molecule Report did not display public details",
			)
		return self.report_text


#============================================
def _molecule_report(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication) -> str:
	"""Open Molecule Report and return its visible semantic details."""
	observer = _MoleculeReportObserver(app)
	try:
		_trigger_action(window, app, "Chemistry", "Molecule Report...")
		return observer.await_report_text()
	finally:
		observer.close()


#============================================
def main() -> int:
	"""Prove public one-carbon-to-methoxy materialization behavior."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		window.show()
		app.processEvents()
		_trigger_action(window, app, "File", "New")
		canvas = _canvas(window)
		carbon_anchor = PySide6.QtCore.QPointF(70.0, 70.0)
		bond_end = PySide6.QtCore.QPointF(130.0, 70.0)
		group_anchor = PySide6.QtCore.QPointF(70.0, 145.0)
		_choose_next_atom_carbon(window, app)
		_trigger_action(window, app, "Draw", "Draw Bond")
		PySide6.QtTest.QTest.mousePress(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(carbon_anchor),
		)
		PySide6.QtTest.QTest.mouseMove(
			canvas.viewport(), canvas.mapFromScene(bond_end),
		)
		PySide6.QtTest.QTest.mouseRelease(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(bond_end),
		)
		app.processEvents()
		_select_scene_point(window, app, canvas, carbon_anchor)
		_trigger_action(window, app, "Draw", "Attach Compact Group...")
		_choose_ome(app)
		PySide6.QtTest.QTest.mouseRelease(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(group_anchor),
		)
		app.processEvents()
		_select_scene_point(window, app, canvas, group_anchor)
		_trigger_action(window, app, "Chemistry", "Materialize Selected Compact Group")
		report_text = _molecule_report(window, app)
		for required_fact in (
			"Authored graph: 4 atoms, 3 bonds",
			"Authored elements: C: 3, O: 1",
			"Formula: C3H8O",
			"Net formal charge: +0",
		):
			if required_fact not in report_text:
				raise AttachedMethoxyMaterializationE2eError(
					f"materialized OMe did not expose {required_fact!r}; observed report: "
					f"{report_text[:2000]!r}",
				)
		print(json.dumps({"schema": "ferrum-attached-methoxy-materialization-e2e-v1", "status": "ok"}))
		return 0
	finally:
		ferrum_qt_e2e.close_e2e_main_window(window, app)


if __name__ == "__main__":
	raise SystemExit(main())

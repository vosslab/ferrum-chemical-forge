#!/usr/bin/env python3
"""Exercise the complete public Ferrum reaction authoring workflow."""

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
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_LIVENESS_GUARD_MILLISECONDS = 10000


#============================================
class ReactionWorkflowE2eError(RuntimeError):
	"""One failed public reaction-authoring workflow assertion."""


#============================================
def _action(window: PySide6.QtWidgets.QMainWindow, text: str) -> PySide6.QtGui.QAction:
	"""Return one enabled public menu or ribbon action by its displayed label."""
	try:
		return next(action for action in window.findChildren(PySide6.QtGui.QAction)
			if action.text().replace("&", "") == text)
	except StopIteration as exc:
		raise ReactionWorkflowE2eError(f"Ferrum did not expose public action {text!r}") from exc


#============================================
def _trigger(window: PySide6.QtWidgets.QMainWindow, app: PySide6.QtWidgets.QApplication,
		text: str) -> None:
	"""Trigger one public action and dispatch its queued Qt work."""
	action = _action(window, text)
	if not action.isEnabled():
		raise ReactionWorkflowE2eError(f"public action {text!r} was unavailable")
	action.trigger()
	app.processEvents()


#============================================
def _canvas(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtWidgets.QGraphicsView:
	"""Return Ferrum's public drawing canvas."""
	try:
		return next(view for view in window.findChildren(PySide6.QtWidgets.QGraphicsView)
			if view.isVisible() and view.accessibleName() == "Ferrum drawing canvas")
	except StopIteration as exc:
		raise ReactionWorkflowE2eError("Ferrum did not expose the drawing canvas") from exc


#============================================
def _active_tab(window: PySide6.QtWidgets.QMainWindow) -> object:
	"""Return the public document selected by the tab host."""
	host = window.centralWidget()
	if not isinstance(host, PySide6.QtWidgets.QTabWidget) or host.currentWidget() is None:
		raise ReactionWorkflowE2eError("public New did not select a Ferrum document")
	return host.currentWidget()


#============================================
class _UnexpectedModalObserver(PySide6.QtCore.QObject):
	"""Record and dismiss only unexpected modal blockers during this workflow."""

	def __init__(self, app: PySide6.QtWidgets.QApplication) -> None:
		super().__init__(app)
		self._app = app
		self.observations: list[str] = []
		self._scheduled: list[PySide6.QtWidgets.QDialog] = []
		app.installEventFilter(self)

	def close(self) -> None:
		"""Detach this E2E-owned liveness observer."""
		self._app.removeEventFilter(self)

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Capture a public diagnostic before releasing an unexpected modal loop."""
		if (
			event.type() != PySide6.QtCore.QEvent.Type.Show
			or not isinstance(watched, PySide6.QtWidgets.QDialog)
			or not watched.isModal()
		):
			return False
		self.observations.append(
			f"title={watched.windowTitle()!r}; accessible_name={watched.accessibleName()!r}",
		)
		if not any(dialog is watched for dialog in self._scheduled):
			self._scheduled.append(watched)
			PySide6.QtCore.QTimer.singleShot(0, watched.reject)
		return False

	def raise_if_observed(self, phase: str) -> None:
		"""Fail with public modal facts after one semantic phase."""
		if self.observations:
			raise ReactionWorkflowE2eError(
			f"{phase} opened an unexpected public modal: {self.observations!r}",
			)

	def discard_expected(self, accessible_name: str) -> None:
		"""Retain liveness evidence only for modals outside this expected public step."""
		self.observations = [description for description in self.observations
			if f"accessible_name={accessible_name!r}" not in description]


#============================================
def _await(app: PySide6.QtWidgets.QApplication, observer: _UnexpectedModalObserver,
		predicate: collections.abc.Callable[[], bool], phase: str) -> None:
	"""Await semantic completion with a harness liveness escape, not a speed gate."""
	if predicate():
		observer.raise_if_observed(phase)
		return
	loop = PySide6.QtCore.QEventLoop()
	poll = PySide6.QtCore.QTimer()
	poll.setInterval(10)
	guard = PySide6.QtCore.QTimer()
	guard.setSingleShot(True)

	def check() -> None:
		if observer.observations or predicate():
			loop.quit()

	poll.timeout.connect(check)
	guard.timeout.connect(loop.quit)
	poll.start()
	guard.start(_LIVENESS_GUARD_MILLISECONDS)
	loop.exec()
	poll.stop()
	guard.stop()
	observer.raise_if_observed(phase)
	if not predicate():
		raise ReactionWorkflowE2eError(
			f"{phase} did not reach its semantic completion before the E2E liveness guard",
		)


#============================================
def _click_canvas(canvas: PySide6.QtWidgets.QGraphicsView, point: PySide6.QtCore.QPointF,
		modifier: PySide6.QtCore.Qt.KeyboardModifier = PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		) -> None:
	"""Click one visible scene point using ordinary Qt pointer input."""
	PySide6.QtTest.QTest.mouseClick(
		canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, modifier,
		canvas.mapFromScene(point),
	)


def _reaction(tab: object) -> object:
	"""Return the sole reaction from Ferrum's supported Rust-issued observation."""
	reactions = tab.observe_reaction_list().reactions
	if len(reactions) != 1:
		raise ReactionWorkflowE2eError(
			f"expected one Rust-issued reaction after authoring, observed {len(reactions)}",
		)
	return reactions[0]


#============================================
def _roles(reaction: object) -> dict[str, str]:
	"""Map durable member identities to their Rust-issued semantic roles."""
	return {member.document_object_id: member.role for member in reaction.members}


#============================================
def _authoring_state(tab: object) -> str:
	"""Describe current public root kinds when authoring cannot complete."""
	projection = tab.current_document_observation().projection
	return f"molecules={len(projection.molecules)}; roots={[root.kind for root in projection.direct_roots]!r}"


#============================================
def _set_checked(list_widget: PySide6.QtWidgets.QListWidget, index: int,
		checked: bool) -> None:
	"""Set one visible reaction role through its rendered checkbox item."""
	item = list_widget.item(index)
	item.setCheckState(
		PySide6.QtCore.Qt.CheckState.Checked if checked
		else PySide6.QtCore.Qt.CheckState.Unchecked,
	)


#============================================
def _accept_role_edit(app: PySide6.QtWidgets.QApplication) -> None:
	"""Swap the two molecule roles through the accessible Edit Roles dialog."""
	dialog = app.activeModalWidget()
	if not isinstance(dialog, PySide6.QtWidgets.QDialog) or dialog.accessibleName() != "Edit Reaction":
		raise ReactionWorkflowE2eError("Edit Roles did not open its accessible reaction editor")
	reactants = next(widget for widget in dialog.findChildren(PySide6.QtWidgets.QListWidget)
		if widget.accessibleName() == "Reactants")
	products = next(widget for widget in dialog.findChildren(PySide6.QtWidgets.QListWidget)
		if widget.accessibleName() == "Products")
	if reactants.count() != 2 or products.count() != 2:
		raise ReactionWorkflowE2eError("Edit Roles did not expose both authored molecule roots")
	_set_checked(reactants, 1, True)
	_set_checked(products, 1, True)
	button_box = next(widget for widget in dialog.findChildren(PySide6.QtWidgets.QDialogButtonBox)
		if widget.isVisible())
	button = button_box.button(PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok)
	if button is None or not button.isEnabled():
		raise ReactionWorkflowE2eError("Edit Roles did not expose an enabled confirmation")
	button.click()
	app.processEvents()


#============================================
def _confirm_reaction_deletion(app: PySide6.QtWidgets.QApplication) -> None:
	"""Accept the accessible definition-only deletion confirmation."""
	dialog = app.activeModalWidget()
	if (
		not isinstance(dialog, PySide6.QtWidgets.QDialog)
		or dialog.accessibleName() != "Delete Reaction Definition"
	):
		raise ReactionWorkflowE2eError("Delete Definition did not open its accessible confirmation")
	button = next(widget for widget in dialog.findChildren(PySide6.QtWidgets.QPushButton)
		if widget.isVisible() and widget.accessibleName() == "Delete reaction definition")
	button.click()
	app.processEvents()


#============================================
def main() -> int:
	"""Run create, inspect, role edit, movement, and definition deletion through Qt."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	observer = _UnexpectedModalObserver(app)
	try:
		window.show()
		app.processEvents()
		_trigger(window, app, "New")
		tab = _active_tab(window)
		canvas = _canvas(window)
		reactant = PySide6.QtCore.QPointF(40.0, 40.0)
		product = PySide6.QtCore.QPointF(200.0, 40.0)
		arrow_start = PySide6.QtCore.QPointF(70.0, 110.0)
		arrow_end = PySide6.QtCore.QPointF(170.0, 110.0)
		_trigger(window, app, "Insert Cyclohexane Ring")
		_click_canvas(canvas, reactant)
		_await(
			app, observer,
			lambda: len(tab.current_document_observation().projection.molecules) == 1,
			"inserting the first public molecule",
		)
		PySide6.QtTest.QTest.keyClick(canvas.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		app.processEvents()
		_trigger(window, app, "Insert Cyclohexane Ring")
		_click_canvas(canvas, product)
		_await(
			app, observer,
			lambda: len(tab.current_document_observation().projection.molecules) == 2,
			"inserting the second public molecule",
		)
		_trigger(window, app, "Draw Arrow")
		PySide6.QtTest.QTest.mousePress(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(arrow_start),
		)
		PySide6.QtTest.QTest.mouseMove(canvas.viewport(), canvas.mapFromScene(arrow_end))
		PySide6.QtTest.QTest.mouseRelease(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, canvas.mapFromScene(arrow_end),
		)
		try:
			_await(
				app, observer,
				lambda: (
					len(tab.current_document_observation().projection.molecules) == 2
					and any(
						root.kind == "arrow"
						for root in tab.current_document_observation().projection.direct_roots
					)
				),
				"authoring reaction members",
			)
		except ReactionWorkflowE2eError as exc:
			raise ReactionWorkflowE2eError(f"{exc}; observed {_authoring_state(tab)}") from exc
		_trigger(window, app, "Move Complete Roots")
		for index, molecule in enumerate(tab.current_document_observation().projection.molecules):
			atom = molecule.atoms[0]
			_click_canvas(
				canvas, PySide6.QtCore.QPointF(atom.position.x, atom.position.y),
				(
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier if index == 0
					else PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier
				),
			)
		_click_canvas(
			canvas, PySide6.QtCore.QPointF(120.0, 110.0),
			PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier,
		)
		app.processEvents()
		_trigger(window, app, "Create Reaction...")
		composer = next(widget for widget in window.findChildren(PySide6.QtWidgets.QWidget)
			if widget.isVisible() and widget.accessibleName() == "Define Reaction")
		reactants = next(widget for widget in composer.findChildren(PySide6.QtWidgets.QListWidget)
			if widget.accessibleName() == "Reactants")
		products = next(widget for widget in composer.findChildren(PySide6.QtWidgets.QListWidget)
			if widget.accessibleName() == "Products")
		arrows = next(widget for widget in composer.findChildren(PySide6.QtWidgets.QListWidget)
			if widget.accessibleName() == "Arrow")
		if reactants.count() != 2 or products.count() != 2 or arrows.count() != 1:
			raise ReactionWorkflowE2eError(
				"Create Reaction did not expose all selected member roles; observed "
				f"reactants={reactants.count()}, products={products.count()}, arrows={arrows.count()}",
			)
		_set_checked(reactants, 0, True)
		_set_checked(products, 1, True)
		_set_checked(arrows, 0, True)
		create = next(widget for widget in composer.findChildren(PySide6.QtWidgets.QPushButton)
			if widget.accessibleName() == "Create Reaction")
		if not create.isEnabled():
			raise ReactionWorkflowE2eError("Create Reaction stayed disabled for complete visible roles")
		create.click()
		_await(app, observer, lambda: len(tab.observe_reaction_list().reactions) == 1,
			"Create Reaction")
		created = _reaction(tab)
		if not created.strict or sorted(_roles(created).values()) != ["arrow", "product", "reactant"]:
			raise ReactionWorkflowE2eError("Create Reaction did not publish a strict typed reaction")
		member_ids = frozenset(_roles(created))
		root_ids_before_delete = frozenset(
			root.document_object_id for root in tab.current_document_observation().projection.direct_roots
		)
		if not member_ids <= root_ids_before_delete:
			raise ReactionWorkflowE2eError("reaction members were not durable document roots")
		_trigger(window, app, "Reaction Inspector")
		inspector = next(widget for widget in window.findChildren(PySide6.QtWidgets.QWidget)
			if widget.isVisible() and widget.objectName() == "reaction-inspector-dock")
		details = next(widget for widget in inspector.findChildren(PySide6.QtWidgets.QWidget)
			if widget.isVisible() and widget.accessibleName() == "Reaction details")
		details_text = details.toPlainText()
		if "Members" not in details_text or "Validation: Strict" not in details_text:
			raise ReactionWorkflowE2eError(
				"Reaction Inspector did not display the strict Rust-issued reaction state",
			)
		highlight = next(widget for widget in inspector.findChildren(PySide6.QtWidgets.QPushButton)
			if widget.accessibleName() == "Highlight Members")
		highlight.click()
		app.processEvents()
		if "Highlighted all Rust-issued members" not in window.statusBar().currentMessage():
			raise ReactionWorkflowE2eError("Reaction Inspector did not confirm member highlighting")
		edit_roles = next(widget for widget in inspector.findChildren(PySide6.QtWidgets.QPushButton)
			if widget.accessibleName() == "Edit Roles...")
		created_roles = _roles(created)
		PySide6.QtCore.QTimer.singleShot(0, lambda: _accept_role_edit(app))
		edit_roles.click()
		observer.discard_expected("Edit Reaction")
		_await(app, observer, lambda: _roles(_reaction(tab)) != created_roles, "Edit Roles")
		edited = _reaction(tab)
		if sorted(_roles(edited).values()) != ["arrow", "product", "reactant"]:
			raise ReactionWorkflowE2eError("Edit Roles did not retain a complete typed reaction")
		before_nudge_revision = tab.current_document_observation().snapshot.revision
		nudge_right = next(widget for widget in inspector.findChildren(PySide6.QtWidgets.QPushButton)
			if widget.accessibleName() == "Nudge Right")
		nudge_right.click()
		_await(
			app, observer,
			lambda: tab.current_document_observation().snapshot.revision > before_nudge_revision,
			"Nudge Right",
		)
		nudged = _reaction(tab)
		if not nudged.strict or frozenset(_roles(nudged)) != member_ids:
			raise ReactionWorkflowE2eError("Nudge did not retain the same strict reaction members")
		delete = next(widget for widget in inspector.findChildren(PySide6.QtWidgets.QPushButton)
			if widget.accessibleName() == "Delete Definition...")
		PySide6.QtCore.QTimer.singleShot(0, lambda: _confirm_reaction_deletion(app))
		delete.click()
		observer.discard_expected("Delete Reaction Definition")
		_await(app, observer, lambda: not tab.observe_reaction_list().reactions,
			"Delete Definition")
		root_ids_after_delete = frozenset(
			root.document_object_id for root in tab.current_document_observation().projection.direct_roots
		)
		if root_ids_after_delete != root_ids_before_delete:
			raise ReactionWorkflowE2eError(
			"Delete Definition changed reaction member roots instead of only its definition",
		)
		print(json.dumps({"schema": "ferrum-reaction-workflow-e2e-v1", "status": "ok"}))
		return 0
	finally:
		observer.close()
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	raise SystemExit(main())

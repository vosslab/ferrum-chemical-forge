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
		self._expected_accessible_names: set[str] = set()
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
		if watched.accessibleName() in self._expected_accessible_names:
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

	def expect(self, accessible_name: str) -> None:
		"""Allow one named public modal to complete its intended user workflow."""
		self._expected_accessible_names.add(accessible_name)

	def discard_expected(self, accessible_name: str) -> None:
		"""Resume unexpected-modal detection after one expected public step."""
		self._expected_accessible_names.discard(accessible_name)
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


def _reaction(tab: object, member_ids: frozenset[str]) -> object | None:
	"""Return the authored reaction identified by its durable member roots."""
	return next((
		reaction for reaction in tab.observe_reaction_list().reactions
		if frozenset(member.document_object_id for member in reaction.members) == member_ids
	), None)


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
def _molecule_ids(tab: object) -> frozenset[str]:
	"""Return Rust-issued durable identities for the current molecule roots."""
	return frozenset(
		molecule.document_object_id
		for molecule in tab.current_document_observation().projection.molecules
	)


#============================================
def _created_molecule_id(tab: object, previous_ids: frozenset[str], phase: str) -> str:
	"""Return the one durable molecule identity created by a visible authoring step."""
	created_ids = _molecule_ids(tab) - previous_ids
	try:
		created_id = next(iter(created_ids))
	except StopIteration as exc:
		raise ReactionWorkflowE2eError(
			f"{phase} did not add a durable molecule root",
		) from exc
	if any(candidate != created_id for candidate in created_ids):
		raise ReactionWorkflowE2eError(
			f"{phase} added multiple durable molecule roots: {sorted(created_ids)!r}",
		)
	return created_id


#============================================
def _authored_molecule_point(tab: object, molecule_id: str,
		authored_point: PySide6.QtCore.QPointF) -> PySide6.QtCore.QPointF:
	"""Return Rust-issued geometry for the known molecule atom nearest its authoring click."""
	molecule = next((
		item for item in tab.current_document_observation().projection.molecules
		if item.document_object_id == molecule_id
	), None)
	if molecule is None:
		raise ReactionWorkflowE2eError(
			f"Rust projection no longer contains authored molecule {molecule_id!r}",
		)
	try:
		atom = min(
			molecule.atoms,
			key=lambda item: (
				(item.position.x - authored_point.x()) ** 2
				+ (item.position.y - authored_point.y()) ** 2,
				item.document_object_id,
			),
		)
	except ValueError as exc:
		raise ReactionWorkflowE2eError(
			f"Rust projection omitted atoms for authored molecule {molecule_id!r}",
		) from exc
	return PySide6.QtCore.QPointF(atom.position.x, atom.position.y)


#============================================
def _set_checked(list_widget: PySide6.QtWidgets.QListWidget, document_object_id: str,
		checked: bool) -> None:
	"""Set one visible durable reaction root through its rendered checkbox item."""
	item = next((
		candidate for index in range(list_widget.count())
		for candidate in [list_widget.item(index)]
		if candidate.data(PySide6.QtCore.Qt.ItemDataRole.UserRole) == document_object_id
	), None)
	if item is None:
		raise ReactionWorkflowE2eError(
			f"reaction role editor did not expose durable root {document_object_id!r}",
		)
	item.setCheckState(
		PySide6.QtCore.Qt.CheckState.Checked if checked
		else PySide6.QtCore.Qt.CheckState.Unchecked,
	)


#============================================
def _accept_role_edit(app: PySide6.QtWidgets.QApplication, reactant_id: str,
		product_id: str) -> None:
	"""Swap the two molecule roles through the accessible Edit Roles dialog."""
	dialog = app.activeModalWidget()
	if not isinstance(dialog, PySide6.QtWidgets.QDialog) or dialog.accessibleName() != "Edit Reaction":
		raise ReactionWorkflowE2eError("Edit Roles did not open its accessible reaction editor")
	reactants = next(widget for widget in dialog.findChildren(PySide6.QtWidgets.QListWidget)
		if widget.accessibleName() == "Reactants")
	products = next(widget for widget in dialog.findChildren(PySide6.QtWidgets.QListWidget)
		if widget.accessibleName() == "Products")
	_set_checked(reactants, product_id, True)
	_set_checked(products, reactant_id, True)
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
		molecule_ids_before_reactant = _molecule_ids(tab)
		_trigger(window, app, "Cyclopentane (C5)")
		_click_canvas(canvas, reactant)
		_await(
			app, observer,
			lambda: bool(_molecule_ids(tab) - molecule_ids_before_reactant),
			"inserting the first public molecule",
		)
		reactant_id = _created_molecule_id(
			tab, molecule_ids_before_reactant, "inserting the first public molecule",
		)
		PySide6.QtTest.QTest.keyClick(canvas.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		app.processEvents()
		molecule_ids_before_product = _molecule_ids(tab)
		_trigger(window, app, "Insert Cyclohexane Ring")
		_click_canvas(canvas, product)
		_await(
			app, observer,
			lambda: bool(_molecule_ids(tab) - molecule_ids_before_product),
			"inserting the second public molecule",
		)
		product_id = _created_molecule_id(
			tab, molecule_ids_before_product, "inserting the second public molecule",
		)
		root_ids_before_arrow = frozenset(
			root.document_object_id
			for root in tab.current_document_observation().projection.direct_roots
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
					any(
						root.kind == "arrow" and root.document_object_id not in root_ids_before_arrow
						for root in tab.current_document_observation().projection.direct_roots
					)
				),
				"authoring reaction members",
			)
		except ReactionWorkflowE2eError as exc:
			raise ReactionWorkflowE2eError(f"{exc}; observed {_authoring_state(tab)}") from exc
		arrow_id = next(
			root.document_object_id
			for root in tab.current_document_observation().projection.direct_roots
			if root.kind == "arrow" and root.document_object_id not in root_ids_before_arrow
		)
		member_ids = frozenset((reactant_id, product_id, arrow_id))
		_trigger(window, app, "Move Complete Roots")
		for document_object_id, authored_point, modifier in (
			(reactant_id, reactant, PySide6.QtCore.Qt.KeyboardModifier.NoModifier),
			(product_id, product, PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier),
		):
			_click_canvas(
				canvas, _authored_molecule_point(tab, document_object_id, authored_point), modifier,
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
		_set_checked(reactants, reactant_id, True)
		_set_checked(products, product_id, True)
		_set_checked(arrows, arrow_id, True)
		create = next(widget for widget in composer.findChildren(PySide6.QtWidgets.QPushButton)
			if widget.accessibleName() == "Create Reaction")
		if not create.isEnabled():
			raise ReactionWorkflowE2eError("Create Reaction stayed disabled for complete visible roles")
		create.click()
		_await(app, observer, lambda: _reaction(tab, member_ids) is not None, "Create Reaction")
		created = _reaction(tab, member_ids)
		if created is None or not created.strict or _roles(created) != {
			reactant_id: "reactant", product_id: "product", arrow_id: "arrow",
		}:
			raise ReactionWorkflowE2eError("Create Reaction did not publish a strict typed reaction")
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
		observer.expect("Edit Reaction")
		PySide6.QtCore.QTimer.singleShot(
			0, lambda: _accept_role_edit(app, reactant_id, product_id),
		)
		edit_roles.click()
		observer.discard_expected("Edit Reaction")
		try:
			_await(
				app, observer,
				lambda: (
					(reaction := _reaction(tab, member_ids)) is not None
					and _roles(reaction) != created_roles
				),
				"Edit Roles",
			)
		except ReactionWorkflowE2eError as exc:
			edited_reaction = _reaction(tab, member_ids)
			raise ReactionWorkflowE2eError(
				f"{exc}; role_membership_changed={edited_reaction is not None and _roles(edited_reaction) != created_roles}; "
				f"status={window.statusBar().currentMessage()!r}",
			) from exc
		edited = _reaction(tab, member_ids)
		if edited is None or _roles(edited) != {
			product_id: "reactant", reactant_id: "product", arrow_id: "arrow",
		}:
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
		nudged = _reaction(tab, member_ids)
		if nudged is None or not nudged.strict or frozenset(_roles(nudged)) != member_ids:
			raise ReactionWorkflowE2eError("Nudge did not retain the same strict reaction members")
		delete = next(widget for widget in inspector.findChildren(PySide6.QtWidgets.QPushButton)
			if widget.accessibleName() == "Delete Definition...")
		observer.expect("Delete Reaction Definition")
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

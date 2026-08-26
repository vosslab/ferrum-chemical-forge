"""Offscreen Ferrum workflow: create, move, undo, save, and reopen one Arrow."""

# Standard Library
import collections.abc
import json
import pathlib
import subprocess  # nosec B404 - fixed-argv local staged CLI invocation below.
import sys

# local repo modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()


# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import defusedxml.ElementTree

# local repo modules
import file_utils
import e2e_workspace
import ferrum_chem
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_REPOSITORY_ROOT = pathlib.Path(file_utils.get_repo_root())
_LIVENESS_GUARD_MILLISECONDS = 10000


#============================================
def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map a finite backend scene point to the live viewport."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


#============================================
def _action(window: PySide6.QtWidgets.QMainWindow, text: str) -> PySide6.QtGui.QAction:
	"""Find one public action by its visible user-facing label."""
	return next(action for action in window.findChildren(PySide6.QtGui.QAction) if action.text() == text)


#============================================
def _active_canvas_tab(window: PySide6.QtWidgets.QMainWindow) -> object:
	"""Return the publicly selected Ferrum canvas page."""
	tab_widget = window.centralWidget()
	if not isinstance(tab_widget, PySide6.QtWidgets.QTabWidget):
		raise RuntimeError("Ferrum window does not expose its public document tabs")
	tab = tab_widget.currentWidget()
	if tab is None:
		raise RuntimeError("public New did not select a Ferrum document tab")
	return tab


#============================================
def _current_cdml(tab: object) -> str:
	"""Return the current Rust snapshot through the tab's public observation."""
	return tab.current_document_observation().snapshot.cdml


#============================================
def _render_reopened_document(
		ferrum: pathlib.Path, document: pathlib.Path, format_name: str,
		) -> pathlib.Path:
	"""Render one saved document through one native artifact profile."""
	artifact = document.with_suffix(f".{format_name}")
	result = subprocess.run(
		(str(ferrum), "render", str(document), "--to", format_name,
		"--output", str(artifact)), capture_output=True, check=False,  # nosec B603 - fixed argv, shell=False.
	)
	if result.returncode != 0:
		raise RuntimeError(f"native {format_name} render failed: {result.stderr.decode()}")
	if not artifact.is_file() or artifact.stat().st_size == 0:
		raise RuntimeError(f"native {format_name} render did not publish an artifact")
	return artifact


#============================================
def _has_typed_arrow(cdml: str, arrow_type: str) -> bool:
	"""Report whether saved CDML retains one requested typed arrow root."""
	root = defusedxml.ElementTree.fromstring(cdml)
	return any(
		child.tag == "{urn:ferrum:cdml}arrow" and child.attrib["type"] == arrow_type
		for child in root
	)


#============================================
def _svg_is_parseable(svg: pathlib.Path) -> bool:
	"""Report whether native SVG export remains a parseable SVG document."""
	root = defusedxml.ElementTree.parse(svg).getroot()
	return root.tag == "{http://www.w3.org/2000/svg}svg"


#============================================
class ArrowAuthoringE2eError(RuntimeError):
	"""One failed public Qt arrow-authoring workflow assertion."""


#============================================
class _UnexpectedModalObserver(PySide6.QtCore.QObject):
	"""Record and dismiss public modals that would otherwise stall this E2E."""

	def __init__(self, app: PySide6.QtWidgets.QApplication) -> None:
		super().__init__(app)
		self.app = app
		self.dialogs: list[PySide6.QtWidgets.QDialog] = []
		self.observations: list[str] = []
		app.installEventFilter(self)

	def close(self) -> None:
		"""Remove this test-scoped QApplication dialog observer."""
		self.app.removeEventFilter(self)

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Record every shown modal and queue rejection for nested Qt loops."""
		if (
			event.type() != PySide6.QtCore.QEvent.Type.Show
			or not isinstance(watched, PySide6.QtWidgets.QDialog)
			or not watched.isModal()
		):
			return False
		self.observations.append(self._describe(watched))
		self._schedule_rejection(watched)
		return False

	def _describe(self, dialog: PySide6.QtWidgets.QDialog) -> str:
		"""Return public title, visible body, and accessibility text for one modal."""
		body_text: list[str] = []
		if isinstance(dialog, PySide6.QtWidgets.QMessageBox):
			body_text.extend((dialog.text(), dialog.informativeText(), dialog.detailedText()))
		for label in dialog.findChildren(PySide6.QtWidgets.QLabel):
			if label.isVisible() and label.text():
				body_text.append(label.text())
		visible_body = " | ".join(dict.fromkeys(text for text in body_text if text))
		return (
			f"title={dialog.windowTitle()!r}; accessible_name={dialog.accessibleName()!r}; "
			f"accessible_description={dialog.accessibleDescription()!r}; body={visible_body!r}"
		)

	def _schedule_rejection(self, dialog: PySide6.QtWidgets.QDialog) -> None:
		"""Queue one rejection so this unexpected modal cannot block the E2E."""
		if any(existing is dialog for existing in self.dialogs):
			return
		self.dialogs.append(dialog)
		PySide6.QtCore.QTimer.singleShot(0, dialog.reject)

	def raise_if_observed(self, phase: str) -> None:
		"""Raise a failure with the preserved public diagnostic for this phase."""
		if not self.observations:
			return
		observed = "\n".join(f"- {description}" for description in self.observations)
		raise ArrowAuthoringE2eError(
			f"{phase} opened an unexpected public modal during valid arrow authoring:\n{observed}"
		)


#============================================
def _await_public_phase_completion(app: PySide6.QtWidgets.QApplication,
		observer: _UnexpectedModalObserver, predicate: collections.abc.Callable[[], bool],
		description: str) -> None:
	"""Wait for semantic completion, with a liveness escape rather than a speed claim."""
	loop = PySide6.QtCore.QEventLoop()
	poll = PySide6.QtCore.QTimer()
	poll.setInterval(10)
	guard = PySide6.QtCore.QTimer()
	guard.setSingleShot(True)
	state = {"complete": False}

	def check_completion() -> None:
		if observer.observations:
			loop.quit()
		elif predicate():
			state["complete"] = True
			loop.quit()

	def release_liveness_stall() -> None:
		active_modal = app.activeModalWidget()
		if isinstance(active_modal, PySide6.QtWidgets.QDialog):
			observer._schedule_rejection(active_modal)
		loop.quit()

	poll.timeout.connect(check_completion)
	guard.timeout.connect(release_liveness_stall)
	check_completion()
	if not state["complete"] and not observer.observations:
		poll.start()
		guard.start(_LIVENESS_GUARD_MILLISECONDS)
		loop.exec()
	poll.stop()
	guard.stop()
	observer.raise_if_observed(description)
	if not state["complete"]:
		raise ArrowAuthoringE2eError(
			f"{description} did not reach semantic completion before the E2E liveness guard"
		)


#============================================
def main() -> int:
	"""Run the complete arrow-authoring path and publish a compact receipt."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	modal_observer = _UnexpectedModalObserver(app)
	tab: object | None = None
	try:
		with e2e_workspace.E2EWorkspaceLease() as workspace_text:
			output_root = pathlib.Path(workspace_text)
			window.show()
			app.processEvents()
			modal_observer.raise_if_observed("showing the Ferrum window")
			_action(window, "New").trigger()
			app.processEvents()
			modal_observer.raise_if_observed("creating a new Ferrum document")
			tab = _active_canvas_tab(window)
			normal_start, normal_end = _point(tab, 24.0, 30.0), _point(tab, 124.0, 30.0)
			_action(window, "Draw Arrow").trigger()
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, normal_start)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), normal_end)
			PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, normal_end)
			_await_public_phase_completion(
				app, modal_observer, lambda: "<arrow" in _current_cdml(tab), "Draw Arrow",
			)
			created_cdml = _current_cdml(tab)
			_action(window, "Move Complete Roots").trigger()
			move_start, move_end = _point(tab, 74.0, 30.0), _point(tab, 94.0, 48.0)
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, move_start)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), move_end)
			PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, move_end)
			_await_public_phase_completion(
				app, modal_observer, lambda: _current_cdml(tab) != created_cdml,
				"Move Complete Roots",
			)
			_action(window, "Undo").trigger()
			_await_public_phase_completion(
				app, modal_observer, lambda: _current_cdml(tab) == created_cdml, "Undo",
			)
			path = output_root / "arrow.cdml"
			saved = window.save_active_to_path(str(path))
			modal_observer.raise_if_observed("saving the authored Arrow")
			if not saved:
				raise RuntimeError("public Save did not publish the authored Arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if "<arrow" not in reopened.snapshot().cdml:
				raise RuntimeError("Rust reopen did not preserve the authored Arrow")
			start, control, end = (
				_point(tab, 24.0, 100.0), _point(tab, 72.0, 135.0), _point(tab, 124.0, 100.0),
			)
			_action(window, "Draw Curved Equilibrium Arrow").trigger()
			before_equilibrium = _current_cdml(tab)
			for point in (start, control, end):
				PySide6.QtTest.QTest.mouseClick(
					tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
				)
			_await_public_phase_completion(
				app, modal_observer, lambda: _current_cdml(tab) != before_equilibrium,
				"Draw Curved Equilibrium Arrow",
			)
			saved = window.save_active_to_path(str(path))
			modal_observer.raise_if_observed("saving the curved equilibrium arrow")
			if not saved:
				raise RuntimeError("public Save did not publish the curved equilibrium arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			ferrum = _REPOSITORY_ROOT / "build" / "bin" / "ferrum"
			curved_equilibrium_svg = _render_reopened_document(ferrum, path, "svg")
			if not _svg_is_parseable(curved_equilibrium_svg):
				raise RuntimeError("native SVG export is not a parseable SVG document")
			_action(window, "Draw Curved Retro Arrow").trigger()
			before_retro = _current_cdml(tab)
			for point in (start, control, end):
				PySide6.QtTest.QTest.mouseClick(
					tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
				)
			_await_public_phase_completion(
				app, modal_observer, lambda: _current_cdml(tab) != before_retro,
				"Draw Curved Retro Arrow",
			)
			saved = window.save_active_to_path(str(path))
			modal_observer.raise_if_observed("saving the curved retro arrow")
			if not saved:
				raise RuntimeError("public Save did not publish the curved retro arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if not _has_typed_arrow(reopened.snapshot().cdml, "retro"):
				raise RuntimeError("Rust reopen did not retain a typed retro-arrow root")
			_render_reopened_document(ferrum, path, "pdf")
			_render_reopened_document(ferrum, path, "png")
			_action(window, "Draw Curved Electron Arrow").trigger()
			before_electron = _current_cdml(tab)
			for point in (start, control, end):
				PySide6.QtTest.QTest.mouseClick(
					tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
				)
			_await_public_phase_completion(
				app, modal_observer, lambda: _current_cdml(tab) != before_electron,
				"Draw Curved Electron Arrow",
			)
			saved = window.save_active_to_path(str(path))
			modal_observer.raise_if_observed("saving the curved electron arrow")
			if not saved:
				raise RuntimeError("public Save did not publish the curved electron arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if not _has_typed_arrow(reopened.snapshot().cdml, "electron"):
				raise RuntimeError("Rust reopen did not retain a typed electron-arrow root")
			_action(window, "Draw Curved Reaction Arrow").trigger()
			before_reaction = _current_cdml(tab)
			for point in (start, control, end):
				PySide6.QtTest.QTest.mouseClick(
					tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
				)
			_await_public_phase_completion(
				app, modal_observer, lambda: _current_cdml(tab) != before_reaction,
				"Draw Curved Reaction Arrow",
			)
			saved = window.save_active_to_path(str(path))
			modal_observer.raise_if_observed("saving the curved reaction arrow")
			if not saved:
				raise RuntimeError("public Save did not publish the curved reaction arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if not _has_typed_arrow(reopened.snapshot().cdml, "curved-normal"):
				raise RuntimeError("Rust reopen did not retain a typed curved-normal reaction-arrow root")
			curved_normal_svg = _render_reopened_document(ferrum, path, "svg")
			if not _svg_is_parseable(curved_normal_svg):
				raise RuntimeError("native SVG export is not a parseable SVG document")
			modal_observer.raise_if_observed("completing arrow authoring")
			print(json.dumps({"schema": "ferrum-arrow-authoring-e2e-v1", "status": "ok"}))
			return 0
	finally:
		# Retire test-owned UI directly so an earlier E2E failure cannot open a
		# dirty-document refusal dialog and hide its actual exception offscreen.
		if tab is not None:
			tab.dispose()
		window.deleteLater()
		modal_observer.close()


if __name__ == "__main__":
	sys.exit(main())

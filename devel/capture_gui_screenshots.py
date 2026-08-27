#!/usr/bin/env python3
"""Capture thirteen real, completed Ferrum Qt documentation scenes outside the test suite."""

# Standard Library
import argparse
import collections.abc
import dataclasses
import pathlib
import shutil
import subprocess
import sys
import tempfile

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
OUTPUT_DIRECTORY = REPO_ROOT / "docs" / "screenshots"
WINDOW_SIZE = PySide6.QtCore.QSize(1440, 900)
CAPTURE_TITLE_PREFIX = "Ferrum GUI Tour"
_EMPTY_CDML = "<cdml xmlns='urn:ferrum:cdml' version='26.08'/>"
_CARBON_CDML = """<cdml xmlns='urn:ferrum:cdml' version='26.08'>
<molecule id='demo-molecule'>
  <atom id='carbon' name='C'><point x='300' y='360'/></atom>
</molecule>
</cdml>"""
_PAIR_CDML = """<cdml xmlns='urn:ferrum:cdml' version='26.08'>
<molecule id='demo-molecule' name='Carbonyl fragment'>
  <atom id='carbon' name='C'><point x='300' y='360'/></atom>
  <atom id='oxygen' name='O'><point x='520' y='360'/></atom>
  <bond id='carbonyl' start='carbon' end='oxygen' type='n2'/>
</molecule>
</cdml>"""
_CDXML = (
	'<?xml version="1.0" encoding="UTF-8"?>'
	'<!DOCTYPE CDXML SYSTEM "https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd">'
	'<CDXML CreationProgram="ChemDraw 23.0"><page HeightPages="1">'
	'<fragment id="source-fragment"><n id="source-carbon" p="300 360"/>'
	'<n id="source-oxygen" p="520 360" Element="8"/>'
	'<b id="source-carbonyl" B="source-carbon" E="source-oxygen" Order="2"/>'
	'</fragment></page></CDXML>'
)
_CATALOG_QUERY = "furan"
_OVERLAY_BACKGROUNDS: dict[int, PySide6.QtGui.QPixmap] = {}


#============================================
class CaptureError(RuntimeError):
	"""A scene did not reach its documented, observable ready state."""


#============================================
@dataclasses.dataclass(frozen=True)
class Scene:
	"""One named screenshot and its completed-state authoring workflow."""

	name: str
	caption: str
	create: collections.abc.Callable[
		[
			PySide6.QtWidgets.QApplication,
			ferrum_qt.themes.theme_manager.ThemeManager,
			pathlib.Path,
		],
		PySide6.QtWidgets.QMainWindow,
	]
	post_prepare: collections.abc.Callable[
		[PySide6.QtWidgets.QMainWindow, PySide6.QtWidgets.QApplication], None
	] | None = None
	overlay_capture: collections.abc.Callable[
		[PySide6.QtWidgets.QMainWindow, pathlib.Path], None
	] | None = None


#============================================
def _normalized_action_text(action: PySide6.QtGui.QAction) -> str:
	"""Return the visible command label independent of mnemonic ampersands."""
	return action.text().replace("&", "")


#============================================
def _find_action(window: PySide6.QtWidgets.QMainWindow, label: str) -> PySide6.QtGui.QAction:
	"""Return one actual Ferrum command action by its visible label."""
	for action in window.findChildren(PySide6.QtGui.QAction):
		if _normalized_action_text(action) == label:
			return action
	raise CaptureError(f"Ferrum command is unavailable: {label}")


#============================================
def _activate_command(
		window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication,
		label: str,
		) -> None:
	"""Activate one visible Ferrum command through its public QAction."""
	action = _find_action(window, label)
	action.trigger()
	application.processEvents()


#============================================
def _active_tab(window: PySide6.QtWidgets.QMainWindow) -> object:
	"""Return the visible active document tab through the public central widget."""
	tabs = window.centralWidget()
	if not isinstance(tabs, PySide6.QtWidgets.QTabWidget):
		raise CaptureError("Ferrum did not expose a tabbed document workspace")
	tab = tabs.currentWidget()
	if tab is None or not getattr(tab, "isVisible")():
		raise CaptureError("Ferrum has no visible active document")
	return tab


#============================================
def _canvas(tab: object) -> PySide6.QtWidgets.QGraphicsView:
	"""Return Ferrum's visible canvas without synthesizing drawing state."""
	canvas = getattr(tab, "view", None)
	if not isinstance(canvas, PySide6.QtWidgets.QGraphicsView) or not canvas.isVisible():
		raise CaptureError("Ferrum drawing canvas is unavailable")
	return canvas


#============================================
def _documentation_frame(canvas: PySide6.QtWidgets.QGraphicsView) -> None:
	"""Frame completed scene geometry without changing the active display palette."""
	scene = canvas.scene()
	if scene is None:
		raise CaptureError("Ferrum drawing canvas has no scene to frame")
	content: PySide6.QtCore.QRectF | None = None
	for item in scene.items():
		if item.zValue() < 0.0:
			continue
		bounds = item.sceneBoundingRect()
		if bounds.isEmpty():
			continue
		content = bounds if content is None else content.united(bounds)
	if content is None or content.isNull() or content.isEmpty():
		raise CaptureError("Ferrum scene has no completed content to frame")
	width = max(content.width() * 1.7, 360.0)
	height = max(content.height() * 1.7, 260.0, width / 1.45)
	frame = PySide6.QtCore.QRectF(
		content.center().x() - width / 2.0,
		content.center().y() - height / 2.0,
		width,
		height,
	)
	canvas.setBackgroundBrush(PySide6.QtGui.QColor("#f6f2e9"))
	canvas.fitInView(frame, PySide6.QtCore.Qt.AspectRatioMode.KeepAspectRatio)


#============================================
def _prepare_documentation_capture(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Retire transient editor state and frame the durable result before capture."""
	_activate_command(window, application, "Select Structure")
	canvas = _canvas(_active_tab(window))
	_click(canvas, PySide6.QtCore.QPoint(20, 20))
	application.processEvents()
	_documentation_frame(canvas)
	application.processEvents()


#============================================
def _scene_point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map an ordinary authored scene point onto the visible drawing viewport."""
	return _canvas(tab).mapFromScene(PySide6.QtCore.QPointF(x, y))


#============================================
def _write_source(workspace: pathlib.Path, name: str, suffix: str, source: str) -> pathlib.Path:
	"""Write one bounded local interchange source inside the scene workspace."""
	path = workspace / f"{name}.{suffix}"
	path.write_text(source, encoding="utf-8")
	return path


#============================================
def _wait_for_open(
		window: PySide6.QtWidgets.QMainWindow, path: pathlib.Path,
		) -> None:
	"""Open one ordinary document and await Ferrum's declared completion signal once."""
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)
	outcome: list[bool] = []

	def finish(success: bool) -> None:
		"""Record the one semantic Open outcome and end the bounded local wait."""
		if not outcome:
			outcome.append(success)
			loop.quit()

	window.local_document_open_queue_drained.connect(finish)
	timeout.timeout.connect(lambda: finish(False))
	timeout.start(10000)
	PySide6.QtCore.QTimer.singleShot(0, lambda: window.open_file_path(str(path)))
	loop.exec()
	timeout.stop()
	window.local_document_open_queue_drained.disconnect(finish)
	if outcome != [True]:
		raise CaptureError(f"Ferrum did not open the authored document: {path.name}")


#============================================
def _window(
		application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path,
		source: str = _CARBON_CDML, source_suffix: str = "cdml", use_templates: bool = False,
		) -> PySide6.QtWidgets.QMainWindow:
	"""Create one visible fixed-size Ferrum workspace and open its bounded source."""
	if use_templates:
		window = ferrum_qt.main_window.MainWindow(
			theme_manager, user_template_directory=workspace / "templates",
		)
	else:
		window = ferrum_qt.main_window.MainWindow(theme_manager)
	window.resize(WINDOW_SIZE)
	window.show()
	application.processEvents()
	path = _write_source(workspace, "scene", source_suffix, source)
	_wait_for_open(window, path)
	application.processEvents()
	if not window.isVisible() or not _canvas(_active_tab(window)).isVisible():
		raise CaptureError("Ferrum window did not become visibly ready")
	return window


#============================================
def _document_revision(tab: object) -> int:
	"""Return the current public document revision for a completed-state check."""
	return int(tab.current_snapshot.revision)


#============================================
def _atom_count(tab: object) -> int:
	"""Count the current public projected atoms after an authored operation."""
	return sum(len(molecule.atoms) for molecule in tab.current_document_observation().projection.molecules)


#============================================
def _drag(canvas: PySide6.QtWidgets.QGraphicsView, start: PySide6.QtCore.QPoint,
		end: PySide6.QtCore.QPoint) -> None:
	"""Commit one real visible press-drag-release canvas gesture."""
	PySide6.QtTest.QTest.mousePress(
		canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
	)
	PySide6.QtTest.QTest.mouseMove(canvas.viewport(), end)
	PySide6.QtTest.QTest.mouseRelease(
		canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
	)


#============================================
def _click(canvas: PySide6.QtWidgets.QGraphicsView, point: PySide6.QtCore.QPoint) -> None:
	"""Send one ordinary visible canvas click."""
	PySide6.QtTest.QTest.mouseClick(
		canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
	)


#============================================
def _workspace_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Show an editable ordinary Ferrum chemical-document workspace."""
	window = _window(application, theme_manager, workspace, _PAIR_CDML)
	if _atom_count(_active_tab(window)) != 2:
		raise CaptureError("workspace scene did not render its two authored atoms")
	return window


#============================================
def _cdxml_open_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Open bounded CDXML through Ferrum's Rust-owned local ingress route."""
	window = _window(application, theme_manager, workspace, _CDXML, "cdxml")
	tab = _active_tab(window)
	molecules = tab.current_document_observation().projection.molecules
	origin_token = getattr(tab, "local_document_origin_token", None)
	source_description = tab.local_document_source_description
	if (
			getattr(tab, "file_path", None) is not None
			or origin_token is None
			or source_description is None
			or "imported ChemDraw XML document" not in source_description
			or len(molecules) != 1
			or tuple(atom.element for atom in molecules[0].atoms) != ("C", "O")
			or len(molecules[0].bonds) != 1
			):
		raise CaptureError("bounded CDXML did not become an editable CDXML-origin document")
	return window


#============================================
def _atom_authoring_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Create an atom through the command and visible canvas."""
	window = _window(application, theme_manager, workspace, _PAIR_CDML)
	tab = _active_tab(window)
	before = _document_revision(tab)
	before_atoms = _atom_count(tab)
	_activate_command(window, application, "Add Atom at Point")
	_click(_canvas(tab), _scene_point(tab, 500.0, 470.0))
	application.processEvents()
	if _document_revision(tab) <= before or _atom_count(tab) != before_atoms + 1:
		raise CaptureError("Add Atom at Point did not create a durable atom")
	return window


#============================================
def _direct_bond_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Draw one ordinary bond between two authored atoms."""
	window = _window(application, theme_manager, workspace, _PAIR_CDML.replace("  <bond id='carbonyl' start='carbon' end='oxygen' type='n2'/>\n", ""))
	tab = _active_tab(window)
	before = _document_revision(tab)
	_activate_command(window, application, "Draw Bond")
	_drag(_canvas(tab), _scene_point(tab, 300.0, 360.0), _scene_point(tab, 520.0, 360.0))
	application.processEvents()
	if _document_revision(tab) <= before or "<bond" not in tab.current_snapshot.cdml:
		raise CaptureError("Draw Bond did not create a durable Rust bond")
	return window


#============================================
def _inserted_cyclohexane_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Insert a detached cyclohexane ring at a visible empty location."""
	window = _window(application, theme_manager, workspace)
	tab = _active_tab(window)
	before = _atom_count(tab)
	_activate_command(window, application, "Insert Cyclohexane Ring")
	_click(_canvas(tab), _scene_point(tab, 650.0, 360.0))
	application.processEvents()
	if _atom_count(tab) != before + 6:
		raise CaptureError("Insert Cyclohexane Ring did not create six visible atoms")
	return window


#============================================
def _attached_cyclohexane_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Attach a cyclohexane ring to an eligible authored carbon."""
	window = _window(
		application, theme_manager, workspace,
		_PAIR_CDML.replace("type='n2'", "type='n1'"),
	)
	tab = _active_tab(window)
	before = _atom_count(tab)
	before_revision = _document_revision(tab)
	_activate_command(window, application, "Attach Cyclohexane Ring")
	anchor = _scene_point(tab, 300.0, 360.0)
	refusals: list[str] = []

	def reject_unexpected_refusal() -> None:
		"""Retire an unexpected modal so the capture fails with its visible reason."""
		modal = application.activeModalWidget()
		if isinstance(modal, PySide6.QtWidgets.QMessageBox) and modal.isVisible():
			refusals.append(" ".join(filter(None, (modal.text(), modal.informativeText()))))
			modal.reject()

	PySide6.QtCore.QTimer.singleShot(100, reject_unexpected_refusal)
	_drag(_canvas(tab), anchor, anchor + PySide6.QtCore.QPoint(80, 0))
	application.processEvents()
	if refusals:
		raise CaptureError(f"Attach Cyclohexane Ring was refused: {refusals[0]}")
	projection = tab.current_document_observation().projection
	if (
			_document_revision(tab) <= before_revision
			or _atom_count(tab) != before + 5
			or len(projection.molecules) != 1
			or not any(atom.element == "O" for atom in projection.molecules[0].atoms)
			):
		raise CaptureError("Attach Cyclohexane Ring did not complete its six-atom ring")
	return window


#============================================
def _accept_item_dialog(application: PySide6.QtWidgets.QApplication, value: str) -> None:
	"""Choose one visible public Qt list-dialog value during an interaction action."""
	for widget in application.topLevelWidgets():
		if isinstance(widget, PySide6.QtWidgets.QInputDialog) and widget.isVisible():
			widget.setTextValue(value)
			widget.accept()
			return
	raise CaptureError("Ferrum did not show its expected item-choice dialog")


#============================================
def _template_catalog_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Place one Rust-owned oxygen-ring template through the public palette."""
	window = _window(application, theme_manager, workspace, _PAIR_CDML)
	tab = _active_tab(window)
	before_revision = _document_revision(tab)
	before_molecule_count = len(tab.current_document_observation().projection.molecules)
	PySide6.QtCore.QTimer.singleShot(
		0, lambda: _accept_catalog_result(application, _CATALOG_QUERY),
	)
	_activate_command(window, application, "Insert Template...")
	_click(_canvas(tab), _scene_point(tab, 460.0, 540.0))
	application.processEvents()
	projection = tab.current_document_observation().projection
	placed_atoms = tuple(atom.element for atom in projection.molecules[-1].atoms)
	if (
			_document_revision(tab) <= before_revision
			or len(projection.molecules) != before_molecule_count + 1
			or sorted(placed_atoms) != ["C", "C", "C", "C", "O"]
			):
		raise CaptureError("Insert Template did not place the selected Furan ring")
	return window


#============================================
def _retire_template_selection(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Clear placement feedback so the completed template remains legible."""
	tab = _active_tab(window)
	_activate_command(window, application, "Select Structure")
	canvas = _canvas(tab)
	blank = _scene_point(tab, 50.0, 50.0)
	PySide6.QtTest.QTest.mouseMove(canvas.viewport(), blank)
	_click(canvas, blank)
	application.processEvents()
	if tab.selected_structure_targets() or canvas.scene().selectedItems():
		raise CaptureError("Ferrum retained template placement selection after blank-canvas selection")


#============================================
def _accept_catalog_result(
		application: PySide6.QtWidgets.QApplication, query: str,
		) -> None:
	"""Search and accept one enabled Rust catalog result through public controls."""
	dialog = application.activeModalWidget()
	if (
			not isinstance(dialog, PySide6.QtWidgets.QDialog)
			or not dialog.isVisible()
			or dialog.accessibleName() != "Ferrum template palette"
			):
		raise CaptureError("Ferrum did not show its expected template catalog")
	search = next(
		control for control in dialog.findChildren(PySide6.QtWidgets.QLineEdit)
		if control.accessibleName() == "Search templates"
	)
	results = next(
		control for control in dialog.findChildren(PySide6.QtWidgets.QListWidget)
		if control.accessibleName() == "Ferrum template results"
	)
	search.setText(query)
	application.processEvents()
	selected = results.currentItem()
	if (
			selected is None
			or not selected.flags() & PySide6.QtCore.Qt.ItemFlag.ItemIsEnabled
			or query not in selected.text().casefold()
			or not selected.data(PySide6.QtCore.Qt.ItemDataRole.UserRole)
			):
		raise CaptureError("Ferrum catalog search did not select the requested Rust entry")
	place = next(
		button for button in dialog.findChildren(PySide6.QtWidgets.QPushButton)
		if button.text() == "Place on Canvas"
	)
	if not place.isVisible() or not place.isEnabled():
		raise CaptureError("Ferrum catalog did not enable the selected template")
	place.click()


#============================================
def _selected_atom_edit_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Select a carbon and complete the visible Change Element workflow."""
	window = _window(application, theme_manager, workspace, _CARBON_CDML)
	tab = _active_tab(window)
	_activate_command(window, application, "Select Structure")
	_click(_canvas(tab), _scene_point(tab, 300.0, 360.0))
	application.processEvents()
	if not _find_action(window, "Change Element").isEnabled():
		raise CaptureError("Select Structure did not enable the selected-atom command")
	PySide6.QtCore.QTimer.singleShot(0, lambda: _accept_item_dialog(application, "N"))
	_activate_command(window, application, "Change Element")
	application.processEvents()
	selected = tab.selected_atom_projection()
	if selected.element != "N":
		raise CaptureError("Change Element did not complete the selected-atom edit")
	return window


#============================================
def _smarts_result_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Run a visible SMARTS query and retain its completed result status."""
	window = _window(application, theme_manager, workspace, _CARBON_CDML)
	_activate_command(window, application, "SMARTS Query...")
	dock = window.findChild(PySide6.QtWidgets.QDockWidget, "smarts-query-dock")
	if dock is None or not dock.isVisible():
		raise CaptureError("SMARTS Query did not show its dock")
	query = dock.findChild(PySide6.QtWidgets.QLineEdit, "smarts-query-input")
	find = dock.findChild(PySide6.QtWidgets.QPushButton, "smarts-query-find")
	status = dock.findChild(PySide6.QtWidgets.QLabel, "smarts-query-status")
	if query is None or find is None or status is None:
		raise CaptureError("SMARTS Query dock lacks its visible controls")
	query.setText("[C]")
	PySide6.QtTest.QTest.mouseClick(find, PySide6.QtCore.Qt.MouseButton.LeftButton)
	application.processEvents()
	application.processEvents()
	if "Found 1 matches" not in status.text():
		raise CaptureError("SMARTS Query did not produce its completed match status")
	return window


#============================================
def _reaction_arrow_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Draw and commit a completed normal reaction arrow."""
	window = _window(application, theme_manager, workspace)
	tab = _active_tab(window)
	_activate_command(window, application, "Draw Arrow")
	_drag(_canvas(tab), _scene_point(tab, 300.0, 360.0), _scene_point(tab, 620.0, 360.0))
	application.processEvents()
	if "<arrow" not in tab.current_snapshot.cdml:
		raise CaptureError("Draw Arrow did not create a durable reaction arrow")
	return window


#============================================
def _presentation_vector_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Draw and commit a completed renderer-preflighted presentation vector."""
	window = _window(application, theme_manager, workspace)
	tab = _active_tab(window)
	_activate_command(window, application, "Draw Line")
	_drag(_canvas(tab), _scene_point(tab, 300.0, 300.0), _scene_point(tab, 620.0, 470.0))
	application.processEvents()
	if "<polyline" not in tab.current_snapshot.cdml:
		raise CaptureError("Draw Line did not create a durable presentation vector")
	return window


#============================================
def _capture_with_qt(window: PySide6.QtWidgets.QMainWindow, output: pathlib.Path) -> None:
	"""Capture the same visible top-level Ferrum window without Screen Recording access."""
	handle = window.windowHandle()
	if handle is None or handle.screen() is None:
		raise CaptureError("Ferrum window has no screen for the Qt capture fallback")
	pixmap = handle.screen().grabWindow(window.winId())
	if pixmap.isNull():
		pixmap = window.grab()
	if pixmap.isNull() or not pixmap.save(str(output), "PNG"):
		raise CaptureError("Qt could not capture the visible Ferrum window")


#============================================
def _save_dialog_over_window(window: PySide6.QtWidgets.QMainWindow,
		dialog: PySide6.QtWidgets.QDialog, output: pathlib.Path,
		background: PySide6.QtGui.QPixmap | None = None,
		) -> None:
	"""Capture one real visible child dialog over the complete Ferrum application surface."""
	if not dialog.isVisible():
		raise CaptureError("Ferrum dialog overlay is not visible for capture")
	position = dialog.frameGeometry().topLeft() - window.frameGeometry().topLeft()
	if background is None:
		dialog.hide()
		PySide6.QtWidgets.QApplication.processEvents()
		pixmap = window.grab()
		dialog.show()
		dialog.raise_()
		PySide6.QtWidgets.QApplication.processEvents()
	else:
		pixmap = background.copy()
	dialog_pixmap = dialog.grab()
	painter = PySide6.QtGui.QPainter(pixmap)
	painter.drawPixmap(position, dialog_pixmap)
	painter.end()
	if pixmap.isNull() or not pixmap.save(str(output), "PNG"):
		raise CaptureError("Qt could not capture the visible Ferrum dialog overlay")


#============================================
def _capture_template_catalog_with_qt(window: PySide6.QtWidgets.QMainWindow,
		output: pathlib.Path) -> None:
	"""Capture Rust catalog provenance beside its completed template placement."""
	captured = []
	background = window.grab()

	def capture_catalog() -> None:
		for widget in PySide6.QtWidgets.QApplication.topLevelWidgets():
			if isinstance(widget, PySide6.QtWidgets.QDialog) and (
					widget.isVisible() and widget.accessibleName() == "Ferrum template palette"
					):
				search = next(
					control for control in widget.findChildren(PySide6.QtWidgets.QLineEdit)
					if control.accessibleName() == "Search templates"
				)
				results = next(
					control for control in widget.findChildren(PySide6.QtWidgets.QListWidget)
					if control.accessibleName() == "Ferrum template results"
				)
				family = next(
					control for control in widget.findChildren(PySide6.QtWidgets.QComboBox)
					if control.accessibleName() == "Template family"
				)
				category = next(
					control for control in widget.findChildren(PySide6.QtWidgets.QComboBox)
					if control.accessibleName() == "Template category"
				)
				search.setText(_CATALOG_QUERY)
				if results.currentItem() is None or not results.currentItem().data(
						PySide6.QtCore.Qt.ItemDataRole.UserRole
						):
					raise CaptureError("Ferrum catalog search did not select a real Rust entry")
				if _CATALOG_QUERY not in results.currentItem().text().casefold():
					raise CaptureError("Ferrum catalog did not retain the placed template selection")
				details = tuple(
					label.text() for label in widget.findChildren(PySide6.QtWidgets.QLabel)
					if label.text() and " | " in label.text()
				)
				if family.currentText() == "" or category.currentText() == "" or not details:
					raise CaptureError("Ferrum catalog did not expose family, category, and provenance")
				_save_dialog_over_window(window, widget, output, background)
				captured.append(True)
				widget.reject()
				return
		raise CaptureError("Ferrum did not show its expected template catalog")

	PySide6.QtCore.QTimer.singleShot(0, capture_catalog)
	_find_action(window, "Insert Template...").trigger()
	if captured != [True]:
		raise CaptureError("Ferrum did not capture the selected template catalog")


#============================================
def _rearm_atom_authoring(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Restore the visible Add Atom command after its completed authored result is framed."""
	_activate_command(window, application, "Add Atom at Point")
	if not _find_action(window, "Add Atom at Point").isChecked():
		raise CaptureError("Add Atom at Point did not visibly rearm after authoring")


#============================================
def _reselect_edited_nitrogen(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Restore the named visible nitrogen selection after neutral scene framing."""
	tab = _active_tab(window)
	_activate_command(window, application, "Select Structure")
	_click(_canvas(tab), _scene_point(tab, 300.0, 360.0))
	application.processEvents()
	if not tab.has_one_selected_atom() or tab.selected_atom_projection().element != "N":
		raise CaptureError("Ferrum did not visibly reselect the edited nitrogen")


#============================================
def _retire_presentation_selection(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Use normal blank-canvas selection to retire direct-root presentation feedback."""
	tab = _active_tab(window)
	_activate_command(window, application, "Select Structure")
	canvas = _canvas(tab)
	blank = _scene_point(tab, 50.0, 50.0)
	PySide6.QtTest.QTest.mouseMove(canvas.viewport(), blank)
	_click(canvas, blank)
	application.processEvents()
	if (
		tab.has_one_selected_arrow()
		or tab.has_one_selected_plus()
		or canvas.scene().selectedItems()
		):
		raise CaptureError("Ferrum retained a visible presentation selection after blank-canvas selection")


#============================================
def _verify_cdxml_after_prepare(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Require the imported ChemDraw document to remain current after framing."""
	application.processEvents()
	tab = _active_tab(window)
	molecules = tab.current_document_observation().projection.molecules
	if (
		len(molecules) != 1
		or tuple(atom.element for atom in molecules[0].atoms) != ("C", "O")
		or len(molecules[0].bonds) != 1
		):
		raise CaptureError("bounded CDXML lost its imported structure before capture")
	if "Opening drawing" in window.statusBar().currentMessage():
		raise CaptureError("bounded CDXML still reports an in-progress Open before capture")


#============================================
def _find_visible_widget(
		window: PySide6.QtWidgets.QMainWindow, widget_type: type[PySide6.QtWidgets.QWidget],
		accessible_name: str,
		) -> PySide6.QtWidgets.QWidget:
	"""Return one visible public Ferrum widget by its accessible name."""
	for widget in window.findChildren(widget_type):
		if widget.isVisible() and widget.accessibleName() == accessible_name:
			return widget
	raise CaptureError(f"Ferrum control is unavailable: {accessible_name}")


#============================================
def _view_controls_after_prepare(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Use the visible status-bar client to fit a completed document to its content."""
	reset = _find_visible_widget(window, PySide6.QtWidgets.QToolButton, "Reset zoom to 100%")
	content = _find_visible_widget(window, PySide6.QtWidgets.QToolButton, "Zoom to Content")
	slider = _find_visible_widget(window, PySide6.QtWidgets.QSlider, "Zoom percentage slider")
	if not isinstance(reset, PySide6.QtWidgets.QToolButton):
		raise CaptureError("Ferrum reset zoom control has the wrong widget type")
	if not isinstance(content, PySide6.QtWidgets.QToolButton):
		raise CaptureError("Ferrum content zoom control has the wrong widget type")
	if not isinstance(slider, PySide6.QtWidgets.QSlider):
		raise CaptureError("Ferrum zoom percentage control has the wrong widget type")
	PySide6.QtTest.QTest.mouseClick(reset, PySide6.QtCore.Qt.MouseButton.LeftButton)
	application.processEvents()
	if not slider.isEnabled() or slider.value() != 100:
		raise CaptureError("Reset zoom did not expose a 100% status-bar value")
	PySide6.QtTest.QTest.mouseClick(content, PySide6.QtCore.Qt.MouseButton.LeftButton)
	application.processEvents()
	if not slider.isEnabled() or slider.value() == 100:
		raise CaptureError("Zoom to Content did not update the visible status-bar zoom value")


#============================================
def _command_palette_after_prepare(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Open the current live registry palette over an explicitly unselected workspace."""
	_retire_presentation_selection(window, application)
	_OVERLAY_BACKGROUNDS[id(window)] = window.grab()
	_activate_command(window, application, "Command Palette...")
	dialog = window.findChild(PySide6.QtWidgets.QDialog, "command-palette-dialog")
	if dialog is None or not dialog.isVisible():
		raise CaptureError("Command Palette did not open its visible search dialog")
	search = dialog.findChild(PySide6.QtWidgets.QLineEdit, "command-palette-search")
	results = dialog.findChild(PySide6.QtWidgets.QListWidget, "command-palette-results")
	if search is None or results is None:
		raise CaptureError("Command Palette lacks its visible query and result controls")
	search.setText("reaction")
	application.processEvents()
	labels = tuple(results.item(index).text() for index in range(results.count()))
	if not any(label.startswith("Create Reaction...") for label in labels):
		raise CaptureError("Command Palette did not discover the live Create Reaction command")
	if not any(label.startswith("Reaction Inspector") for label in labels):
		raise CaptureError("Command Palette did not discover the live Reaction Inspector command")


#============================================
def _capture_command_palette_with_qt(window: PySide6.QtWidgets.QMainWindow,
		output: pathlib.Path) -> None:
	"""Capture the real visible palette overlay above the completed reaction state."""
	dialog = window.findChild(PySide6.QtWidgets.QDialog, "command-palette-dialog")
	if dialog is None or not dialog.isVisible():
		raise CaptureError("Ferrum Command Palette overlay is unavailable for capture")
	background = _OVERLAY_BACKGROUNDS.pop(id(window), None)
	if background is None:
		raise CaptureError("Ferrum Command Palette lacks its completed workspace surface")
	_save_dialog_over_window(window, dialog, output, background)


SCENES = (
	Scene("workspace", "Editable carbonyl workspace", _workspace_scene),
	Scene(
		"atom_authoring", "Add atom at point", _atom_authoring_scene,
		post_prepare=_rearm_atom_authoring,
	),
	Scene("direct_bond", "Draw direct bond", _direct_bond_scene),
	Scene("inserted_cyclohexane", "Insert cyclohexane ring", _inserted_cyclohexane_scene),
	Scene("attached_cyclohexane", "Attach cyclohexane ring", _attached_cyclohexane_scene),
	Scene(
		"template_catalog", "Browse Rust-owned template catalog", _template_catalog_scene,
		post_prepare=_retire_template_selection,
		overlay_capture=_capture_template_catalog_with_qt,
	),
	Scene(
		"selected_atom_edit", "Change selected carbon to nitrogen", _selected_atom_edit_scene,
		post_prepare=_reselect_edited_nitrogen,
	),
	Scene("smarts_result", "Find carbon SMARTS match", _smarts_result_scene),
	Scene(
		"reaction_arrow", "Draw durable reaction arrow", _reaction_arrow_scene,
		post_prepare=_retire_presentation_selection,
	),
	Scene(
		"presentation_vector", "Draw durable presentation vector", _presentation_vector_scene,
		post_prepare=_retire_presentation_selection,
	),
	Scene(
		"cdxml_open", "Open bounded ChemDraw XML", _cdxml_open_scene,
		post_prepare=_verify_cdxml_after_prepare,
	),
	Scene(
		"view_controls", "Fit document content with status-bar view controls", _workspace_scene,
		post_prepare=_view_controls_after_prepare,
	),
	Scene(
		"command_palette_reaction", "Discover reaction commands from the live palette",
		_reaction_arrow_scene, post_prepare=_command_palette_after_prepare,
		overlay_capture=_capture_command_palette_with_qt,
	),
)
SCENE_NAMES = tuple(scene.name for scene in SCENES)


#============================================
def _capture_with_easy_screenshot(window: PySide6.QtWidgets.QMainWindow,
		output: pathlib.Path) -> bool:
	"""Attempt the optional macOS window backend for this exact titled Ferrum window."""
	command = shutil.which("screenshot")
	if command is None:
		return False
	result = subprocess.run(
		[command, "-A", PySide6.QtWidgets.QApplication.applicationName(),
			"-t", window.windowTitle(), "-f", str(output)],
		capture_output=True, text=True, check=False,
	)
	return result.returncode == 0 and output.is_file() and output.stat().st_size > 0


#============================================
def _verify_full_window_capture_surface(
		window: PySide6.QtWidgets.QMainWindow, output: pathlib.Path,
		) -> None:
	"""Require the documented 16:10 window, ribbon, status bar, and PNG geometry."""
	if window.size() != WINDOW_SIZE:
		raise CaptureError(
			f"Ferrum capture window is {window.width()}x{window.height()}, "
			f"not the required full-window {WINDOW_SIZE.width()}x{WINDOW_SIZE.height()} surface"
		)
	ribbon = window.findChild(PySide6.QtWidgets.QToolBar, "ferrum-authoring-ribbon")
	if ribbon is None or not ribbon.isVisible():
		raise CaptureError("Ferrum capture requires the visible authoring ribbon")
	status_bar = window.statusBar()
	if not status_bar.isVisible():
		raise CaptureError("Ferrum capture requires the visible status bar")
	image = PySide6.QtGui.QImage(str(output))
	if image.isNull() or image.width() < 200 or image.height() < 200:
		raise CaptureError("capture output is not a usable window PNG")
	if image.width() * WINDOW_SIZE.height() != image.height() * WINDOW_SIZE.width():
		raise CaptureError(
			f"capture backend produced {image.width()}x{image.height()}, not the required "
			"16:10 full Ferrum window. The backend likely included window decoration or "
			"cropped the application; use the Qt backend or configure the window capture "
			"backend to capture only the Ferrum application surface."
		)


#============================================
def _capture(window: PySide6.QtWidgets.QMainWindow, output: pathlib.Path,
		backend: str,
		overlay_capture: collections.abc.Callable[
			[PySide6.QtWidgets.QMainWindow, pathlib.Path], None
		] | None = None,
		) -> str:
	"""Capture one completed full-window scene and verify its documented surface."""
	if overlay_capture is not None:
		overlay_capture(window, output)
		used = "qt"
	elif backend == "easy-screenshot":
		if not _capture_with_easy_screenshot(window, output):
			raise CaptureError("easy-screenshot could not capture the titled Ferrum window")
		used = backend
	elif backend == "qt":
		_capture_with_qt(window, output)
		used = backend
	elif _capture_with_easy_screenshot(window, output):
		used = "easy-screenshot"
	else:
		_capture_with_qt(window, output)
		used = "qt"
	_verify_full_window_capture_surface(window, output)
	return used


#============================================
def _close_window(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Retire one disposable scene window after its staged capture completes."""
	window.close()
	window.deleteLater()
	application.processEvents()


#============================================
def _publish(staged_directory: pathlib.Path, scene_names: tuple[str, ...]) -> None:
	"""Publish verified screenshot names, preserving unaffected tour assets."""
	OUTPUT_DIRECTORY.mkdir(parents=True, exist_ok=True)
	for name in scene_names:
		staged = staged_directory / f"{name}.png"
		destination = OUTPUT_DIRECTORY / staged.name
		shutil.copyfile(staged, destination)


#============================================
def _parse_args() -> argparse.Namespace:
	"""Read the bounded local-capture backend choice."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument(
		"--backend", choices=("auto", "easy-screenshot", "qt"), default="auto",
		help="capture backend; auto prefers easy-screenshot then uses Qt QScreen",
	)
	parser.add_argument(
		"--scene", choices=SCENE_NAMES,
		help="capture and publish one named scene without replacing the other tour PNGs",
	)
	parser.add_argument("--list", action="store_true", help="print scene names without launching Ferrum")
	return parser.parse_args()


#============================================
def main() -> int:
	"""Create each real GUI state, stage its evidence, then publish the full verified set."""
	args = _parse_args()
	if args.list:
		print("\n".join(SCENE_NAMES))
		return 0
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication(sys.argv[:1])
	application.setApplicationName("Ferrum")
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(application)
	theme_manager.restore_theme()
	scenes = tuple(scene for scene in SCENES if args.scene is None or scene.name == args.scene)
	backends: set[str] = set()
	with tempfile.TemporaryDirectory(prefix="ferrum_gui_screenshots_") as temporary:
		staged = pathlib.Path(temporary)
		for scene in scenes:
			print(f"=== Ferrum GUI scene: {scene.name} (create) ===", flush=True)
			workspace = staged / scene.name
			workspace.mkdir()
			window = scene.create(application, theme_manager, workspace)
			window.setWindowTitle(f"{CAPTURE_TITLE_PREFIX}: {scene.caption}")
			show_grid = _find_action(window, "Show Hex Grid")
			if show_grid.isChecked():
				show_grid.trigger()
			_prepare_documentation_capture(window, application)
			if scene.post_prepare is not None:
				scene.post_prepare(window, application)
			output = staged / f"{scene.name}.png"
			print(f"=== Ferrum GUI scene: {scene.name} (capture) ===", flush=True)
			try:
				backends.add(_capture(window, output, args.backend, scene.overlay_capture))
			finally:
				_close_window(window, application)
			print(f"=== Ferrum GUI scene: {scene.name} (staged) ===", flush=True)
		_publish(staged, tuple(scene.name for scene in scenes))
	print(f"Captured {len(scenes)} Ferrum GUI tour PNGs with {', '.join(sorted(backends))}.")
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except CaptureError as error:
		print(f"GUI screenshot capture error: {error}", file=sys.stderr)
		raise SystemExit(1) from error

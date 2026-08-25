#!/usr/bin/env python3
"""Capture ten real, completed Ferrum Qt documentation scenes outside the test suite."""

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


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
OUTPUT_DIRECTORY = REPO_ROOT / "docs" / "screenshots"
WINDOW_SIZE = PySide6.QtCore.QSize(1440, 900)
CAPTURE_TITLE_PREFIX = "Ferrum GUI Tour"
SCENE_NAMES = (
	"workspace",
	"atom_authoring",
	"direct_bond",
	"inserted_cyclohexane",
	"attached_cyclohexane",
	"template_catalog",
	"selected_atom_edit",
	"smarts_result",
	"reaction_arrow",
	"presentation_vector",
)

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
_TEMPLATE_CDML = """<cdml xmlns='urn:ferrum:cdml' version='26.08'>
<molecule id='template-molecule' name='Reusable oxygen-ring template'>
  <atom id='template-carbon-one' name='C'><point x='0' y='0'/></atom>
  <atom id='template-carbon-two' name='C'><point x='40' y='0'/></atom>
  <atom id='template-oxygen' name='O'><point x='60' y='30'/></atom>
  <atom id='template-carbon-three' name='C'><point x='40' y='60'/></atom>
  <atom id='template-carbon-four' name='C'><point x='0' y='60'/></atom>
  <atom id='template-carbon-five' name='C'><point x='-20' y='30'/></atom>
  <bond id='template-bond-one' start='template-carbon-one' end='template-carbon-two' type='n1'/>
  <bond id='template-bond-two' start='template-carbon-two' end='template-oxygen' type='n1'/>
  <bond id='template-bond-three' start='template-oxygen' end='template-carbon-three' type='n1'/>
  <bond id='template-bond-four' start='template-carbon-three' end='template-carbon-four' type='n1'/>
  <bond id='template-bond-five' start='template-carbon-four' end='template-carbon-five' type='n1'/>
  <bond id='template-bond-six' start='template-carbon-five' end='template-carbon-one' type='n1'/>
</molecule>
</cdml>"""


#============================================
class CaptureError(RuntimeError):
	"""A scene did not reach its documented, observable ready state."""


#============================================
@dataclasses.dataclass(frozen=True)
class Scene:
	"""One named screenshot and its completed-state authoring workflow."""

	name: str
	create: collections.abc.Callable[
		[PySide6.QtWidgets.QApplication, pathlib.Path], PySide6.QtWidgets.QMainWindow
	]
	show_template_chooser: bool = False


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
	"""Frame completed scene content legibly on a high-contrast documentation canvas."""
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
	for item in scene.items():
		if type(item).__name__ != "FerrumPaperItem":
			continue
		if isinstance(item, PySide6.QtWidgets.QGraphicsRectItem):
			item.setBrush(PySide6.QtGui.QColor("#f6f2e9"))
			item.setPen(PySide6.QtGui.QPen(PySide6.QtGui.QColor("#ddd5c7")))
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
	scene = canvas.scene()
	if scene is not None:
		scene.clearSelection()
		items = scene.items()
		has_durable_drawing = any(
			type(item).__name__ in {"ArrowProjectionItem", "PolylineProjectionItem"}
			for item in items
		)
		if has_durable_drawing:
			for item in items:
				name = type(item).__name__
				if name == "QGraphicsRectItem" or (
						name == "QGraphicsPathItem" and item.zValue() >= 1_000_000.0
						):
					item.setVisible(False)
	application.processEvents()
	_documentation_frame(canvas)
	application.processEvents()


#============================================
def _scene_point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map an ordinary authored scene point onto the visible drawing viewport."""
	return _canvas(tab).mapFromScene(PySide6.QtCore.QPointF(x, y))


#============================================
def _write_document(workspace: pathlib.Path, name: str, cdml: str) -> pathlib.Path:
	"""Write one ordinary authored CDML document inside the scene workspace."""
	path = workspace / f"{name}.cdml"
	path.write_text(cdml, encoding="utf-8")
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
		application: PySide6.QtWidgets.QApplication, workspace: pathlib.Path,
		cdml: str = _CARBON_CDML, use_templates: bool = False,
		) -> PySide6.QtWidgets.QMainWindow:
	"""Create one visible fixed-size Ferrum workspace and open its authored input."""
	if use_templates:
		window = ferrum_qt.main_window.MainWindow(
			object(), user_template_directory=workspace / "templates",
		)
	else:
		window = ferrum_qt.main_window.MainWindow(object())
	window.resize(WINDOW_SIZE)
	window.show()
	application.processEvents()
	path = _write_document(workspace, "scene", cdml)
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
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Show an editable ordinary Ferrum chemical-document workspace."""
	window = _window(application, workspace, _PAIR_CDML)
	if _atom_count(_active_tab(window)) != 2:
		raise CaptureError("workspace scene did not render its two authored atoms")
	return window


#============================================
def _atom_authoring_scene(application: PySide6.QtWidgets.QApplication,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Create an atom through the command and visible canvas."""
	window = _window(application, workspace)
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
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Draw one ordinary bond between two authored atoms."""
	window = _window(application, workspace, _PAIR_CDML.replace("  <bond id='carbonyl' start='carbon' end='oxygen' type='n2'/>\n", ""))
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
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Insert a detached cyclohexane ring at a visible empty location."""
	window = _window(application, workspace)
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
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Attach a cyclohexane ring to an eligible authored carbon."""
	window = _window(application, workspace, _CARBON_CDML)
	tab = _active_tab(window)
	before = _atom_count(tab)
	before_revision = _document_revision(tab)
	_activate_command(window, application, "Attach Cyclohexane Ring")
	anchor = _scene_point(tab, 300.0, 360.0)
	_drag(_canvas(tab), anchor, anchor + PySide6.QtCore.QPoint(80, 0))
	application.processEvents()
	if _document_revision(tab) <= before_revision or _atom_count(tab) != before + 5:
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
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Place a reusable authored template through Ferrum's visible template command."""
	template_directory = workspace / "templates"
	template_directory.mkdir()
	(template_directory / "reusable_pair.cdml").write_text(_TEMPLATE_CDML, encoding="utf-8")
	window = _window(application, workspace, use_templates=True)
	tab = _active_tab(window)
	before = _atom_count(tab)
	PySide6.QtCore.QTimer.singleShot(
		0, lambda: _accept_item_dialog(application, "Reusable oxygen-ring template"),
	)
	_activate_command(window, application, "Place User Template...")
	_click(_canvas(tab), _scene_point(tab, 600.0, 400.0))
	application.processEvents()
	if _atom_count(tab) != before + 6:
		raise CaptureError("Place User Template did not install the reusable authored molecule")
	return window


#============================================
def _selected_atom_edit_scene(application: PySide6.QtWidgets.QApplication,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Select a carbon and complete the visible Change Element workflow."""
	window = _window(application, workspace, _CARBON_CDML)
	tab = _active_tab(window)
	_activate_command(window, application, "Select Structure")
	_click(_canvas(tab), _scene_point(tab, 300.0, 360.0))
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
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Run a visible SMARTS query and retain its completed result status."""
	window = _window(application, workspace, _CARBON_CDML)
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
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Draw and commit a completed normal reaction arrow."""
	window = _window(application, workspace)
	tab = _active_tab(window)
	_activate_command(window, application, "Draw Arrow")
	_drag(_canvas(tab), _scene_point(tab, 300.0, 360.0), _scene_point(tab, 620.0, 360.0))
	application.processEvents()
	if "<arrow" not in tab.current_snapshot.cdml:
		raise CaptureError("Draw Arrow did not create a durable reaction arrow")
	return window


#============================================
def _presentation_vector_scene(application: PySide6.QtWidgets.QApplication,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Draw and commit a completed renderer-preflighted presentation vector."""
	window = _window(application, workspace)
	tab = _active_tab(window)
	_activate_command(window, application, "Draw Line")
	_drag(_canvas(tab), _scene_point(tab, 300.0, 300.0), _scene_point(tab, 620.0, 470.0))
	application.processEvents()
	if "<polyline" not in tab.current_snapshot.cdml:
		raise CaptureError("Draw Line did not create a durable presentation vector")
	return window


SCENES = (
	Scene("workspace", _workspace_scene),
	Scene("atom_authoring", _atom_authoring_scene),
	Scene("direct_bond", _direct_bond_scene),
	Scene("inserted_cyclohexane", _inserted_cyclohexane_scene),
	Scene("attached_cyclohexane", _attached_cyclohexane_scene),
	Scene("template_catalog", _template_catalog_scene, True),
	Scene("selected_atom_edit", _selected_atom_edit_scene),
	Scene("smarts_result", _smarts_result_scene),
	Scene("reaction_arrow", _reaction_arrow_scene),
	Scene("presentation_vector", _presentation_vector_scene),
)


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
def _capture_template_chooser_with_qt(window: PySide6.QtWidgets.QMainWindow,
		output: pathlib.Path) -> None:
	"""Capture a named visible template chooser and its completed placement together."""
	label = "Reusable oxygen-ring template"
	captured = []

	def capture_chooser() -> None:
		for widget in PySide6.QtWidgets.QApplication.topLevelWidgets():
			if isinstance(widget, PySide6.QtWidgets.QInputDialog) and widget.isVisible():
				widget.setTextValue(label)
				pixmap = window.grab()
				dialog_pixmap = widget.grab()
				position = widget.frameGeometry().topLeft() - window.frameGeometry().topLeft()
				painter = PySide6.QtGui.QPainter(pixmap)
				painter.drawPixmap(position, dialog_pixmap)
				painter.end()
				if pixmap.isNull() or not pixmap.save(str(output), "PNG"):
					raise CaptureError("Qt could not capture the visible template chooser")
				captured.append(True)
				widget.reject()
				return
		raise CaptureError("Ferrum did not show its expected template chooser")

	PySide6.QtCore.QTimer.singleShot(0, capture_chooser)
	_find_action(window, "Place User Template...").trigger()
	if captured != [True]:
		raise CaptureError("Ferrum did not capture the selected template chooser")


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
		backend: str, show_template_chooser: bool = False) -> str:
	"""Capture one completed full-window scene and verify its documented surface."""
	if show_template_chooser:
		_capture_template_chooser_with_qt(window, output)
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
	scenes = tuple(scene for scene in SCENES if args.scene is None or scene.name == args.scene)
	backends: set[str] = set()
	with tempfile.TemporaryDirectory(prefix="ferrum_gui_screenshots_") as temporary:
		staged = pathlib.Path(temporary)
		for scene in scenes:
			workspace = staged / scene.name
			workspace.mkdir()
			window = scene.create(application, workspace)
			window.setWindowTitle(f"{CAPTURE_TITLE_PREFIX}: {scene.name.replace('_', ' ')}")
			show_grid = _find_action(window, "Show Hex Grid")
			if show_grid.isChecked():
				show_grid.trigger()
			_prepare_documentation_capture(window, application)
			output = staged / f"{scene.name}.png"
			try:
				backends.add(_capture(window, output, args.backend, scene.show_template_chooser))
			finally:
				_close_window(window, application)
		_publish(staged, tuple(scene.name for scene in scenes))
	print(f"Captured {len(scenes)} Ferrum GUI tour PNGs with {', '.join(sorted(backends))}.")
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except CaptureError as error:
		print(f"GUI screenshot capture error: {error}", file=sys.stderr)
		raise SystemExit(1) from error

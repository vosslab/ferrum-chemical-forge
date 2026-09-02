#!/usr/bin/env python3
"""Capture fourteen real, completed Ferrum Qt documentation scenes outside the test suite."""

# Standard Library
import argparse
import collections.abc
import pathlib
import shutil
import sys
import tempfile

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.themes.theme_manager

from documentation_biomolecule_geometry import (
	assert_dspc_geometry, distearoylphosphatidylcholine_source, dna_base_pair_source,
)
from documentation_biomolecule_sources import SUCROSE_CDML as _SUCROSE_CDML
from ferrum_qt.documentation_capture_models import (
	CATALOG_QUERY as _CATALOG_QUERY, CARBON_CDML as _CARBON_CDML,
	CDXML as _CDXML, DOCUMENTATION_PROPERTY_DOCK_WIDTH as _DOCUMENTATION_PROPERTY_DOCK_WIDTH,
	EMPTY_CDML as _EMPTY_CDML, PAIR_CDML as _PAIR_CDML, Scene,
)
from ferrum_qt.documentation_capture_surfaces import (
	CaptureError, capture_with_easy_screenshot as _capture_with_easy_screenshot,
	capture_with_qt as _capture_with_qt,
	save_dialog_over_window as _save_dialog_over_window,
	verify_full_window_capture_surface as _verify_full_window_capture_surface,
)


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
OUTPUT_DIRECTORY = REPO_ROOT / "docs" / "screenshots"
WINDOW_SIZE = PySide6.QtCore.QSize(1440, 900)
CAPTURE_TITLE_PREFIX = "Ferrum GUI Tour"
_PRE_DIALOG_SURFACES: dict[int, PySide6.QtGui.QPixmap] = {}
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
def _documentation_frame(tab: object) -> None:
	"""Frame document roots closely enough that completed chemistry stays legible."""
	content_bounds = getattr(tab, "document_content_bounds", None)
	if not callable(content_bounds):
		raise CaptureError("Ferrum tab does not expose document content bounds")
	content = content_bounds()
	if not isinstance(content, PySide6.QtCore.QRectF) or content.isNull() or content.isEmpty():
		raise CaptureError("Ferrum document has no completed content to frame")
	canvas = _canvas(tab)
	# The public Content control uses these exact document-root bounds.  Add only a
	# modest presentation margin, which keeps ordinary bonds and rings readable in
	# the complete window instead of accidentally framing the paper or renderer aids.
	width = max(content.width() * 1.35, 70.0)
	height = max(content.height() * 1.35, 54.0, width / 2.0)
	frame = PySide6.QtCore.QRectF(
		content.center().x() - width / 2.0,
		content.center().y() - height / 2.0,
		width,
		height,
	)
	canvas.setBackgroundBrush(PySide6.QtGui.QColor("#f6f2e9"))
	fit = getattr(canvas, "fit_display_bounds", None)
	canvas.resetTransform()
	if not callable(fit) or fit(frame) is not True:
		raise CaptureError("Ferrum drawing canvas could not frame completed content")

#============================================
def _set_documentation_zoom(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Use Ferrum's visible status-bar zoom client for readable full-window evidence."""
	slider = _find_visible_widget(window, PySide6.QtWidgets.QSlider, "Zoom percentage slider")
	if not isinstance(slider, PySide6.QtWidgets.QSlider) or not slider.isEnabled():
		raise CaptureError("Ferrum documentation capture cannot reach the visible zoom client")
	bounds = _active_tab(window).document_content_bounds()
	target = 150 if bounds is not None and bounds.height() > 320.0 else 230
	slider.setValue(target)
	application.processEvents()
	if slider.value() != target:
		raise CaptureError("Ferrum documentation capture did not retain the requested zoom")

#============================================
def _prepare_documentation_capture(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Retire transient editor state and frame the durable result before capture."""
	_activate_command(window, application, "Select Structure")
	tab = _active_tab(window)
	canvas = _canvas(tab)
	_click(canvas, PySide6.QtCore.QPoint(20, 20))
	application.processEvents()
	hide_keyboard_cursor = getattr(canvas, "hide_keyboard_cursor", None)
	if not callable(hide_keyboard_cursor):
		raise CaptureError("Ferrum drawing canvas cannot retire its keyboard cursor")
	hide_keyboard_cursor()
	_documentation_frame(tab)
	application.processEvents()

#============================================
def _arrange_documentation_docks(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Give the temporary capture-only Properties dock a readable title bar."""
	dock = getattr(window, "_native_property_dock", None)
	if not isinstance(dock, PySide6.QtWidgets.QDockWidget):
		raise CaptureError("Ferrum documentation capture requires the Properties dock")
	# The ordinary user's saved layout is never changed: every scene owns a fresh
	# disposable window.  A fixed 190 px dock leaves the full title visible while
	# still preserving a broad editable document surface at 1440 x 900.
	dock.show()
	dock.setMinimumWidth(_DOCUMENTATION_PROPERTY_DOCK_WIDTH)
	window.resizeDocks(
		[dock], [_DOCUMENTATION_PROPERTY_DOCK_WIDTH],
		PySide6.QtCore.Qt.Orientation.Horizontal,
	)
	application.processEvents()
	option = PySide6.QtWidgets.QStyleOptionDockWidget()
	dock.initStyleOption(option)
	title_rect = dock.style().subElementRect(
		PySide6.QtWidgets.QStyle.SubElement.SE_DockWidgetTitleBarText, option, dock,
	)
	title_width = dock.fontMetrics().horizontalAdvance(dock.windowTitle())
	if title_rect.width() < title_width:
		raise CaptureError(
			"Ferrum documentation Properties title is clipped in the fixed capture layout",
		)

#============================================
def _scene_point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map an ordinary authored scene point onto the visible drawing viewport."""
	return _canvas(tab).mapFromScene(PySide6.QtCore.QPointF(x, y))

#============================================
def _write_source(workspace: pathlib.Path, name: str, suffix: str, source: str) -> pathlib.Path:
	"""Write one bounded local interchange source inside the scene workspace."""
	# ASVS 5.3.2: names and suffixes come only from the closed scene registry;
	# no user-controlled filename participates in capture path construction.
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
	"""Show an editable stereochemical sucrose drawing in the Ferrum workspace."""
	window = _window(application, theme_manager, workspace, _SUCROSE_CDML)
	if _atom_count(_active_tab(window)) != 23:
		raise CaptureError("workspace scene did not render the complete sucrose drawing")
	return window

#============================================
def _pentapeptide_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Import Ala-Asn-Lys-Leu-Glu through Ferrum's native peptide worker."""
	window = _window(application, theme_manager, workspace, _EMPTY_CDML)
	tab = _active_tab(window)
	canvas = _canvas(tab)
	canvas.fit_display_bounds(canvas.scene().sceneRect())
	application.processEvents()
	before_atoms = _atom_count(tab)
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)
	receipts: list[object] = []

	def finish(receipt: object | None = None) -> None:
		"""Retain the semantic installation receipt and end the bounded wait."""
		if receipt is not None:
			receipts.append(receipt)
		loop.quit()

	window.document_installation_completed.connect(finish)
	timeout.timeout.connect(finish)
	timeout.start(10000)
	if window.start_supported_peptide_import("ANKLE") is not True:
		timeout.stop()
		window.document_installation_completed.disconnect(finish)
		raise CaptureError("Ferrum did not start the supported pentapeptide import")
	loop.exec()
	timeout.stop()
	window.document_installation_completed.disconnect(finish)
	if (
		len(receipts) != 1
		or getattr(receipts[0], "installation_kind", None) != "peptide_sequence_import"
		or _atom_count(tab) <= before_atoms
		or len(tab.current_document_observation().projection.molecules) != 1
		):
		raise CaptureError(
			f"Ferrum did not install the complete ANKLE pentapeptide: "
			f"receipts={len(receipts)} atoms={_atom_count(tab)} "
			f"molecules={len(tab.current_document_observation().projection.molecules)}"
		)
	interaction = tab.observe_direct_root_interaction()
	selection = tab.select_direct_roots(
		interaction, None, engine.RenderInteractionQueryV1.root(
			interaction.roots[0].document_object_id,
		),
	)
	move = tab.translate_direct_root_selection_from_origin(
		selection, -1670.0, -1080.0, engine.RenderInteractionSnapV1.free(),
	)
	application.processEvents()
	if move.changed is not True:
		raise CaptureError("Ferrum did not move the imported pentapeptide onto the visible page")
	PySide6.QtTest.QTest.qWait(100)
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
			or tuple(atom.element for atom in molecules[0].atoms) != ("C", "O", "N", "F")
			or tuple(bond.source_type for bond in molecules[0].bonds) != ("s1", "b1", "d1")
			):
		raise CaptureError("styled CDXML did not become an editable CDXML-origin document")
	return window

#============================================
def _atom_authoring_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Create an atom beside sucrose through the command and visible canvas."""
	window = _window(application, theme_manager, workspace, _SUCROSE_CDML)
	tab = _active_tab(window)
	before = _document_revision(tab)
	before_atoms = _atom_count(tab)
	_activate_command(window, application, "Add Atom at Point")
	_click(_canvas(tab), _scene_point(tab, 575.0, 520.0))
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
	_drag(_canvas(tab), _scene_point(tab, 300.0, 360.0), _scene_point(tab, 340.0, 360.0))
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
	_click(_canvas(tab), _scene_point(tab, 450.0, 360.0))
	application.processEvents()
	if _atom_count(tab) != before + 6:
		raise CaptureError("Insert Cyclohexane Ring did not create six visible atoms")
	return window

#============================================
def _attached_cyclohexane_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Attach a fully renderable cyclohexane ring away from the neighboring oxygen."""
	window = _window(
		application, theme_manager, workspace,
		_PAIR_CDML.replace("type='n2'", "type='n1'"),
	)
	tab = _active_tab(window)
	before = _atom_count(tab)
	before_revision = _document_revision(tab)
	before_molecule = tab.current_document_observation().projection.molecules[0]
	host_carbon_id = next(
		atom.document_object_id for atom in before_molecule.atoms if atom.element == "C"
	)
	host_oxygen_id = next(
		atom.document_object_id for atom in before_molecule.atoms if atom.element == "O"
	)
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
	# The initial C--O fragment leaves its left side open.  A leftward release is an
	# ordinary valid user choice and gives the attached ring enough space to retain
	# every authored bond, including the original exterior C--O bond.
	_drag(_canvas(tab), anchor, anchor - PySide6.QtCore.QPoint(80, 0))
	application.processEvents()
	if refusals:
		raise CaptureError(f"Attach Cyclohexane Ring was refused: {refusals[0]}")
	observation = tab.current_document_observation()
	projection = observation.projection
	render_observation = tab._render_observation
	render_projection = tab._controller.projection
	if render_projection is None:
		raise CaptureError("Attach Cyclohexane Ring did not retain its installed render projection")
	molecule = projection.molecules[0]
	bond_ids = {bond.document_object_id for bond in molecule.bonds}
	bond_graphics = tuple(
		item for item, target in render_projection.item_targets.items()
		if target.document_object_id in bond_ids
	)
	exterior_bond = next(
		bond for bond in molecule.bonds
		if {bond.start.document_object_id, bond.end.document_object_id}
		== {host_carbon_id, host_oxygen_id}
	)
	if exterior_bond.document_object_id not in {
		target.document_object_id for target in render_projection.item_targets.values()
	}:
		raise CaptureError("Attach Cyclohexane Ring dropped the authored exterior C--O bond")
	if (
			_document_revision(tab) <= before_revision
			or _atom_count(tab) != before + 5
			or len(projection.molecules) != 1
			or len(molecule.bonds) != 7
			or len(bond_graphics) != 7
			or render_projection.issues
			or any(plan.plan.issues or plan.member_issues for plan in render_observation.molecule_plans)
			or not any(atom.element == "O" for atom in molecule.atoms)
			):
		raise CaptureError("Attach Cyclohexane Ring did not retain a complete rendered molecule")
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
	"""Show one Rust-owned catalog selection before any template placement."""
	window = _window(application, theme_manager, workspace, _PAIR_CDML)
	return window

#============================================
def _selected_atom_edit_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Select a carbon and complete the visible Change Element workflow."""
	window = _window(application, theme_manager, workspace, _CARBON_CDML)
	tab = _active_tab(window)
	_activate_command(window, application, "Select Structure")
	point = _scene_point(tab, 300.0, 360.0)
	_click(_canvas(tab), point)
	application.processEvents()
	if not _find_action(window, "Change Element").isEnabled():
		raise CaptureError(f"Select Structure did not enable the selected-atom command: {point}")
	PySide6.QtCore.QTimer.singleShot(0, lambda: _accept_item_dialog(application, "N"))
	_activate_command(window, application, "Change Element")
	application.processEvents()
	selected = tab.selected_atom_projection()
	if selected.element != "N":
		raise CaptureError("Change Element did not complete the selected-atom edit")
	return window

#============================================
def _dspc_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Open PubChem-derived DSPC prepared through bounded SDF insertion."""
	window = _window(application, theme_manager, workspace, distearoylphosphatidylcholine_source())
	tab = _active_tab(window)
	if _atom_count(tab) != 54:
		raise CaptureError("SDF ingress did not retain the complete DSPC graph")
	molecules = tab.current_document_observation().projection.molecules
	if len(molecules) != 1:
		raise CaptureError("SDF ingress did not retain one DSPC molecule")
	assert_dspc_geometry(molecules[0])
	return window

#============================================
def _smarts_result_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Find all eight oxygen atoms in imported distearoylphosphatidylcholine."""
	window = _dspc_scene(application, theme_manager, workspace)
	_activate_command(window, application, "SMARTS Query...")
	dock = window.findChild(PySide6.QtWidgets.QDockWidget, "smarts-query-dock")
	if dock is None or not dock.isVisible():
		raise CaptureError("SMARTS Query did not show its dock")
	query = dock.findChild(PySide6.QtWidgets.QLineEdit, "smarts-query-input")
	find = dock.findChild(PySide6.QtWidgets.QPushButton, "smarts-query-find")
	status = dock.findChild(PySide6.QtWidgets.QLabel, "smarts-query-status")
	if query is None or find is None or status is None:
		raise CaptureError("SMARTS Query dock lacks its visible controls")
	query.setText("[O]")
	PySide6.QtTest.QTest.mouseClick(find, PySide6.QtCore.Qt.MouseButton.LeftButton)
	application.processEvents()
	application.processEvents()
	if "Found 8 matches" not in status.text():
		raise CaptureError("SMARTS Query did not produce its completed match status")
	return window

#============================================
def _reaction_arrow_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Draw and commit a completed reaction arrow below sucrose."""
	window = _window(application, theme_manager, workspace, _SUCROSE_CDML)
	tab = _active_tab(window)
	_activate_command(window, application, "Draw Arrow")
	_drag(_canvas(tab), _scene_point(tab, 340.0, 570.0), _scene_point(tab, 460.0, 570.0))
	application.processEvents()
	if "<arrow" not in tab.current_snapshot.cdml:
		raise CaptureError("Draw Arrow did not create a durable reaction arrow")
	return window

#============================================
def _dna_base_pair_scene(application: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		workspace: pathlib.Path) -> PySide6.QtWidgets.QMainWindow:
	"""Show an A-T pair with two noncovalent presentation-line hydrogen bonds."""
	window = _window(application, theme_manager, workspace, dna_base_pair_source())
	tab = _active_tab(window)
	molecules = tab.current_document_observation().projection.molecules
	if (
		tuple(molecule.name for molecule in molecules) != ("Thymine", "Adenine")
		or tab.current_snapshot.cdml.count("<polyline") != 8
		):
		raise CaptureError("A-T base pair lacks its molecules or hydrogen-bond guides")
	return window
#============================================
def _capture_template_catalog_with_qt(window: PySide6.QtWidgets.QMainWindow,
		output: pathlib.Path) -> None:
	"""Capture the current Rust catalog selection and provenance before placement."""
	captured = []

	def capture_catalog() -> None:
		application = PySide6.QtWidgets.QApplication.instance()
		if application is None:
			raise CaptureError("Ferrum Qt application is unavailable for catalog capture")
		widget = next((candidate for candidate in application.topLevelWidgets() if (
			isinstance(candidate, PySide6.QtWidgets.QDialog)
			and candidate.isVisible() and candidate.accessibleName() == "Template Catalog"
		)), None)
		if widget is None:
			raise CaptureError("Ferrum did not show its expected Template Catalog")
		search = next(
			control for control in widget.findChildren(PySide6.QtWidgets.QLineEdit)
			if control.accessibleName() == "Search templates"
		)
		results = next(
			control for control in widget.findChildren(PySide6.QtWidgets.QListWidget)
			if control.accessibleName() == "Template catalog results"
		)
		family = next(
			control for control in widget.findChildren(PySide6.QtWidgets.QComboBox)
			if control.accessibleName() == "Built-in template family"
		)
		category = next(
			control for control in widget.findChildren(PySide6.QtWidgets.QComboBox)
			if control.accessibleName() == "Built-in template category"
		)
		search.setText(_CATALOG_QUERY)
		if results.currentItem() is None or not results.currentItem().data(
				PySide6.QtCore.Qt.ItemDataRole.UserRole
				):
			raise CaptureError("Ferrum catalog search did not select a real Rust entry")
		if _CATALOG_QUERY not in results.currentItem().text().casefold():
			raise CaptureError("Ferrum catalog did not retain the placed template selection")
		details = next(
			label for label in widget.findChildren(PySide6.QtWidgets.QLabel)
			if label.accessibleName() == "Selected template details"
		)
		if family.currentText() == "" or category.currentText() == "" or not details.text():
			raise CaptureError("Ferrum catalog did not expose family, category, and provenance")
		_save_dialog_over_window(window, widget, output)
		captured.append(True)
		widget.reject()

	PySide6.QtCore.QTimer.singleShot(0, capture_catalog)
	_find_action(window, "Template Catalog...").trigger()
	PySide6.QtWidgets.QApplication.processEvents()
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
	"""Require the imported ChemDraw document to remain current and visibly framed."""
	application.processEvents()
	tab = _active_tab(window)
	molecules = tab.current_document_observation().projection.molecules
	if (
		len(molecules) != 1
		or tuple(atom.element for atom in molecules[0].atoms) != ("C", "O", "N", "F")
		or tuple(bond.source_type for bond in molecules[0].bonds) != ("s1", "b1", "d1")
		):
		raise CaptureError("styled CDXML lost its imported structure before capture")
	render = tab._render_observation
	if (
			render is None
			or len(render.molecule_plans) != 1
			or render.molecule_plans[0].plan.issues
			or render.molecule_plans[0].member_issues
			):
		raise CaptureError("styled CDXML lacks one complete issue-free render plan")
	if "Opening drawing" in window.statusBar().currentMessage():
		raise CaptureError("bounded CDXML still reports an in-progress Open before capture")
	# CDXML ingress can schedule its first page frame after the generic scene
	# preparation pass. Use the public view command once import is definitively
	# complete so the screenshot proves the editable projection, not an empty page.
	canvas = _canvas(tab)
	bounds = tab.document_content_bounds()
	if bounds is None:
		raise CaptureError("bounded CDXML has no drawable content bounds after import")
	_documentation_frame(tab)
	application.processEvents()
	center = canvas.mapFromScene(bounds.center())
	mapped_bounds = canvas.mapFromScene(bounds).boundingRect()
	visible = canvas.viewport().rect().adjusted(40, 40, -40, -40)
	if not visible.contains(center) or mapped_bounds.width() < 100:
		raise CaptureError(
			f"bounded CDXML content is not visibly framed after import: "
			f"center={center.x()},{center.y()} visible={visible} bounds={bounds}"
		)

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
	_set_documentation_zoom(window, application)
	tab = _active_tab(window)
	_canvas(tab).centerOn(tab.document_content_bounds().center())
	application.processEvents()
#============================================
def _command_palette_after_prepare(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Open the current live registry palette over an explicitly unselected workspace."""
	_retire_presentation_selection(window, application)
	_PRE_DIALOG_SURFACES[id(window)] = window.grab()
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
	background = _PRE_DIALOG_SURFACES.pop(id(window), None)
	if background is None:
		raise CaptureError("Ferrum Command Palette lacks its settled full-window surface")
	_save_dialog_over_window(window, dialog, output, background)


SCENES = (
	Scene("workspace", "Editable sucrose workspace", _workspace_scene),
	Scene(
		"pentapeptide_import", "Import ANKLE pentapeptide", _pentapeptide_scene,
		ribbon_tab_id="structure",
	),
	Scene(
		"atom_authoring", "Add atom beside sucrose", _atom_authoring_scene,
		post_prepare=_rearm_atom_authoring, ribbon_tab_id="structure",
	),
	Scene("direct_bond", "Draw direct bond", _direct_bond_scene, ribbon_tab_id="structure"),
	Scene(
		"inserted_cyclohexane", "Insert cyclohexane ring", _inserted_cyclohexane_scene,
		ribbon_tab_id="structure",
	),
	Scene(
		"attached_cyclohexane", "Attach cyclohexane ring", _attached_cyclohexane_scene,
		ribbon_tab_id="structure",
	),
	Scene(
		"template_catalog", "Browse Rust-owned template catalog", _template_catalog_scene,
		overlay_capture=_capture_template_catalog_with_qt, ribbon_tab_id="structure",
	),
	Scene(
		"selected_atom_edit", "Change selected carbon to nitrogen", _selected_atom_edit_scene,
		ribbon_tab_id="structure",
	),
	Scene("smarts_result", "Find DSPC oxygen SMARTS matches", _smarts_result_scene),
	Scene(
		"reaction_arrow", "Draw reaction arrow beside sucrose", _reaction_arrow_scene,
		post_prepare=_retire_presentation_selection, ribbon_tab_id="reactions",
	),
	Scene(
		"presentation_vector", "Show Watson-Crick A-T hydrogen bonds", _dna_base_pair_scene,
		post_prepare=_retire_presentation_selection, ribbon_tab_id="annotate",
	),
	Scene(
		"cdxml_open", "Open Wavy, Bold, and Dashed ChemDraw bonds", _cdxml_open_scene,
		post_prepare=_verify_cdxml_after_prepare, ribbon_tab_id="structure",
	),
	Scene(
		"view_controls", "Fit DSPC with status-bar view controls", _dspc_scene,
		post_prepare=_view_controls_after_prepare, ribbon_tab_id="view",
	),
	Scene(
		"command_palette_reaction", "Discover reaction commands from the live palette",
		_reaction_arrow_scene, post_prepare=_command_palette_after_prepare,
		overlay_capture=_capture_command_palette_with_qt, ribbon_tab_id="reactions",
	),
)
SCENE_NAMES = tuple(scene.name for scene in SCENES)

#============================================
def _expose_ribbon_tab(window: PySide6.QtWidgets.QMainWindow, ribbon_tab_id: str) -> None:
	"""Expose and verify one scene-owned task tab after queued Qt state changes."""
	window._authoring_ribbon.select_tab(ribbon_tab_id)
	PySide6.QtWidgets.QApplication.processEvents()
	if window._authoring_ribbon.current_tab_id() != ribbon_tab_id:
		raise CaptureError(f"Ferrum did not retain ribbon tab: {ribbon_tab_id}")

#============================================
def _capture(window: PySide6.QtWidgets.QMainWindow, output: pathlib.Path,
		backend: str, ribbon_tab_id: str,
		overlay_capture: collections.abc.Callable[
			[PySide6.QtWidgets.QMainWindow, pathlib.Path], None
		] | None = None,
		) -> str:
	"""Capture one completed full-window scene and verify its documented surface."""
	if overlay_capture is None:
		PySide6.QtWidgets.QApplication.processEvents()
		tab = _active_tab(window)
		_documentation_frame(tab)
		_canvas(tab).centerOn(tab.document_content_bounds().center())
		PySide6.QtWidgets.QApplication.processEvents()
		_expose_ribbon_tab(window, ribbon_tab_id)
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
	_verify_full_window_capture_surface(window, output, WINDOW_SIZE)
	return used

#============================================
def _close_window(window: PySide6.QtWidgets.QMainWindow,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Discard disposable authored documents before retiring one scene window."""
	tabs = window.centralWidget()
	while tabs.count() > 0:
		if window._close_tab_at_with_decision(
			0, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		) is not ferrum_qt.ferrum.close_decision.CloseResult.CLOSED:
			raise CaptureError("Ferrum capture could not discard its disposable document")
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
	parser.add_argument(
		"--theme", choices=("dark", "light"), default="light",
		help="transient application theme used for this capture; defaults to light",
	)
	parser.add_argument(
		"--ribbon-tab",
		help="stable ribbon tab ID overriding the task tab declared by each capture scene",
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
	# Keep documentation evidence deterministic without mutating the user's saved
	# application preference. Explicit black presentation marks remain legible on
	# the light document page.
	theme_manager.apply_transient_theme(args.theme)
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
			_arrange_documentation_docks(window, application)
			# Authoring can queue a last renderer-owned initial frame.  Let that
			# ordinary visible lifecycle settle before the status-bar client chooses
			# the final documentation zoom.
			PySide6.QtTest.QTest.qWait(75)
			_set_documentation_zoom(window, application)
			ribbon_tab_id = args.ribbon_tab if args.ribbon_tab is not None else scene.ribbon_tab_id
			_expose_ribbon_tab(window, ribbon_tab_id)
			if scene.post_prepare is not None:
				scene.post_prepare(window, application)
			_expose_ribbon_tab(window, ribbon_tab_id)
			output = staged / f"{scene.name}.png"
			print(f"=== Ferrum GUI scene: {scene.name} (capture) ===", flush=True)
			try:
				backends.add(_capture(
					window, output, args.backend, ribbon_tab_id, scene.overlay_capture,
				))
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

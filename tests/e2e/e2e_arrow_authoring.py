"""Offscreen Ferrum workflow: create, move, undo, save, and reopen one Arrow."""

# Standard Library
import json
import pathlib
import subprocess  # nosec B404 - fixed-argv local staged CLI invocation below.
import sys

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
def _curved_equilibrium_geometry(observation: object) -> object | None:
	"""Return one typed curved-equilibrium geometry from a Rust observation."""
	for root in observation.projection.presentation_stack.roots:
		if (
			root.kind == "arrow" and root.arrow is not None and
			root.arrow.geometry.kind == "curved_equilibrium" and
			root.arrow.geometry.curved_equilibrium is not None
		):
			return root.arrow.geometry.curved_equilibrium
	return None


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
def main() -> int:
	"""Run the complete arrow-authoring path and publish a compact receipt."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	tab: object | None = None
	try:
		with e2e_workspace.E2EWorkspaceLease() as workspace_text:
			output_root = pathlib.Path(workspace_text)
			window.show()
			app.processEvents()
			_action(window, "New").trigger()
			app.processEvents()
			tab = _active_canvas_tab(window)
			normal_start, normal_end = _point(tab, 24.0, 30.0), _point(tab, 124.0, 30.0)
			_action(window, "Draw Arrow").trigger()
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, normal_start)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), normal_end)
			PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, normal_end)
			app.processEvents()
			created_cdml = _current_cdml(tab)
			if "<arrow" not in created_cdml:
				raise RuntimeError("Draw Arrow did not create one durable Arrow")
			_action(window, "Move Complete Roots").trigger()
			move_start, move_end = _point(tab, 74.0, 30.0), _point(tab, 94.0, 48.0)
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, move_start)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), move_end)
			PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, move_end)
			app.processEvents()
			moved_cdml = _current_cdml(tab)
			if moved_cdml == created_cdml:
				raise RuntimeError("Move Complete Roots did not translate the created Arrow")
			_action(window, "Undo").trigger()
			app.processEvents()
			if _current_cdml(tab) != created_cdml:
				raise RuntimeError("Undo did not restore the pre-translation Arrow document")
			path = output_root / "arrow.cdml"
			if not window.save_active_to_path(str(path)):
				raise RuntimeError("public Save did not publish the authored Arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if "<arrow" not in reopened.snapshot().cdml:
				raise RuntimeError("Rust reopen did not preserve the authored Arrow")
			start, control, end = (
				_point(tab, 24.0, 100.0), _point(tab, 72.0, 135.0), _point(tab, 124.0, 100.0),
			)
			_action(window, "Draw Curved Equilibrium Arrow").trigger()
			for point in (start, control, end):
				PySide6.QtTest.QTest.mouseClick(
					tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
				)
			app.processEvents()
			if _curved_equilibrium_geometry(tab.current_document_observation()) is None:
				raise RuntimeError("public Curved Equilibrium authoring did not install its typed Rust arrow")
			if not window.save_active_to_path(str(path)):
				raise RuntimeError("public Save did not publish the curved equilibrium arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			curved_equilibrium_geometry = _curved_equilibrium_geometry(reopened.observe(0))
			if curved_equilibrium_geometry is None:
				raise RuntimeError("Rust reopen did not retain a typed curved-equilibrium arrow root")
			ferrum = _REPOSITORY_ROOT / "build" / "bin" / "ferrum"
			curved_equilibrium_svg = _render_reopened_document(ferrum, path, "svg")
			if not _svg_is_parseable(curved_equilibrium_svg):
				raise RuntimeError("native SVG export is not a parseable SVG document")
			_action(window, "Draw Curved Retro Arrow").trigger()
			for point in (start, control, end):
				PySide6.QtTest.QTest.mouseClick(
					tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
				)
			app.processEvents()
			if not window.save_active_to_path(str(path)):
				raise RuntimeError("public Save did not publish the curved retro arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if not _has_typed_arrow(reopened.snapshot().cdml, "retro"):
				raise RuntimeError("Rust reopen did not retain a typed retro-arrow root")
			_render_reopened_document(ferrum, path, "pdf")
			_render_reopened_document(ferrum, path, "png")
			_action(window, "Draw Curved Electron Arrow").trigger()
			for point in (start, control, end):
				PySide6.QtTest.QTest.mouseClick(
					tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
				)
			app.processEvents()
			if not window.save_active_to_path(str(path)):
				raise RuntimeError("public Save did not publish the curved electron arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if not _has_typed_arrow(reopened.snapshot().cdml, "electron"):
				raise RuntimeError("Rust reopen did not retain a typed electron-arrow root")
			_action(window, "Draw Curved Reaction Arrow").trigger()
			for point in (start, control, end):
				PySide6.QtTest.QTest.mouseClick(
					tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
				)
			app.processEvents()
			if not window.save_active_to_path(str(path)):
				raise RuntimeError("public Save did not publish the curved reaction arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if not _has_typed_arrow(reopened.snapshot().cdml, "curved-normal"):
				raise RuntimeError("Rust reopen did not retain a typed curved-normal reaction-arrow root")
			curved_normal_svg = _render_reopened_document(ferrum, path, "svg")
			if not _svg_is_parseable(curved_normal_svg):
				raise RuntimeError("native SVG export is not a parseable SVG document")
			print(json.dumps({"schema": "ferrum-arrow-authoring-e2e-v1", "status": "ok"}))
			return 0
	finally:
		# Retire test-owned UI directly so an earlier E2E failure cannot open a
		# dirty-document refusal dialog and hide its actual exception offscreen.
		if tab is not None:
			tab.dispose()
		window.deleteLater()


if __name__ == "__main__":
	sys.exit(main())

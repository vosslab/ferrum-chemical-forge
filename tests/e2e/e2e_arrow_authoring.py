"""Offscreen Ferrum workflow: create, move, undo, save, and reopen one Arrow."""

# Standard Library
import json
import pathlib
import subprocess
import sys
import tempfile

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import defusedxml.ElementTree

# local repo modules
import ferrum_chem
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


_CDML = """<cdml xmlns='urn:ferrum:cdml'>
  <molecule id='mol-1'><atom id='atom-c' name='C'><point x='10' y='20'/></atom></molecule>
</cdml>"""


#============================================
def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map a finite backend scene point to the live viewport."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


#============================================
def _render_reopened_document(
		ferrum: pathlib.Path, document: pathlib.Path, format_name: str,
		) -> bytes:
	"""Render one saved document through one native artifact profile."""
	artifact = document.with_suffix(f".{format_name}")
	result = subprocess.run(
		(str(ferrum), "render", str(document), "--to", format_name,
		"--output", str(artifact)), capture_output=True, check=False,
	)
	if result.returncode != 0:
		raise RuntimeError(f"native {format_name} render failed: {result.stderr.decode()}")
	return artifact.read_bytes()


#============================================
def _has_electron_arrow(cdml: str) -> bool:
	"""Report whether saved CDML retains one typed electron-arrow root."""
	root = defusedxml.ElementTree.fromstring(cdml)
	return any(
		child.tag == "{urn:ferrum:cdml}arrow" and child.attrib["type"] == "electron"
		for child in root
	)


#============================================
def _svg_has_cubic_path(svg: bytes) -> bool:
	"""Report whether SVG contains an authored cubic curve path."""
	root = defusedxml.ElementTree.fromstring(svg)
	return root.tag == "{http://www.w3.org/2000/svg}svg" and any(
		child.tag == "{http://www.w3.org/2000/svg}path" and "C" in child.attrib["d"]
		for child in root.iter()
	)


#============================================
def main() -> int:
	"""Run the complete arrow-authoring path and publish a compact receipt."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "arrow-e2e.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		app.processEvents()
		start, end = _point(tab, 24.0, 30.0), _point(tab, 124.0, 30.0)
		window._draw_arrow_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		app.processEvents()
		created_cdml = tab.current_snapshot.cdml
		if "<arrow" not in created_cdml or window._render_interaction_selection is None:
			raise RuntimeError("Draw Arrow did not create and select one durable Arrow")
		window._translate_roots_action.trigger()
		move_start, move_end = _point(tab, 74.0, 30.0), _point(tab, 94.0, 48.0)
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, move_start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), move_end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, move_end)
		app.processEvents()
		moved_cdml = tab.current_snapshot.cdml
		if moved_cdml == created_cdml:
			raise RuntimeError("Move Complete Roots did not translate the created Arrow")
		tab.undo()
		app.processEvents()
		if tab.current_snapshot.cdml != created_cdml:
			raise RuntimeError("Undo did not restore the pre-translation Arrow document")
		with tempfile.TemporaryDirectory(
				prefix="ferrum-arrow-e2e-", dir=pathlib.Path.cwd(),
				) as directory:
			path = pathlib.Path(directory) / "arrow.cdml"
			if not window.save_active_to_path(str(path)):
				raise RuntimeError("public Save did not publish the authored Arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if "<arrow" not in reopened.snapshot().cdml:
				raise RuntimeError("Rust reopen did not preserve the authored Arrow")
			control = _point(tab, 72.0, 84.0)
			window._draw_curved_electron_arrow_action.trigger()
			for point in (start, control, end):
				PySide6.QtTest.QTest.mouseClick(
					tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
				)
			app.processEvents()
			if not window.save_active_to_path(str(path)):
				raise RuntimeError("public Save did not publish the curved electron arrow")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if not _has_electron_arrow(reopened.snapshot().cdml):
				raise RuntimeError("Rust reopen did not retain a typed electron-arrow root")
			ferrum = pathlib.Path(__file__).resolve().parents[2] / "build" / "bin" / "ferrum"
			svg = _render_reopened_document(ferrum, path, "svg")
			pdf = _render_reopened_document(ferrum, path, "pdf")
			png = _render_reopened_document(ferrum, path, "png")
			if not _svg_has_cubic_path(svg):
				raise RuntimeError("native SVG export lacks the curved electron-arrow cubic path")
			if not pdf.startswith(b"%PDF-") or not png.startswith(b"\x89PNG\r\n\x1a\n"):
				raise RuntimeError("native PDF or PNG export lacks its required artifact signature")
		print(json.dumps({"schema": "ferrum-arrow-authoring-e2e-v1", "status": "ok"}))
		return 0
	finally:
		# Retire test-owned UI directly so an earlier E2E failure cannot open a
		# dirty-document refusal dialog and hide its actual exception offscreen.
		tab.dispose()
		window.deleteLater()


if __name__ == "__main__":
	sys.exit(main())

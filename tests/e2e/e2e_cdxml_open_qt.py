#!/usr/bin/env python3
"""Open styled CDXML through File/Open and prove its native publication contract."""

# Standard Library
import json
import pathlib
import sys
import tempfile

# local repo modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_STYLED_CDXML = (
	'<CDXML><page><fragment id="presentation-fragment"><n id="a" p="0 0"/>'
	'<n id="b" p="20 0"/><b B="a" E="b" Display="Wavy"/>'
	'<n id="c" p="40 0"/><b B="b" E="c" Display="Bold"/>'
	'<n id="d" p="60 0"/><b B="c" E="d" Display="Dash"/>'
	'</fragment></page></CDXML>'
)


#============================================
class CdxmlOpenQtE2eError(RuntimeError):
	"""Report one broken Qt CDXML File/Open workflow assertion."""


#============================================
def _active_tab(window: PySide6.QtWidgets.QMainWindow) -> object:
	"""Return the currently visible native document tab."""
	tab_widget = window.centralWidget()
	if not isinstance(tab_widget, PySide6.QtWidgets.QTabWidget):
		raise CdxmlOpenQtE2eError("Ferrum did not expose document tabs")
	tab = tab_widget.currentWidget()
	if tab is None:
		raise CdxmlOpenQtE2eError("Ferrum did not select a document tab")
	return tab


#============================================
def _open_action(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtGui.QAction:
	"""Return the visible File/Open action."""
	for action in window.findChildren(PySide6.QtGui.QAction):
		if action.text() == "Open":
			return action
	raise CdxmlOpenQtE2eError("Ferrum did not expose File/Open")


#============================================
def _await_open(window: object, start: object) -> bool:
	"""Wait for one queued File/Open completion without polling or sleeping."""
	if not callable(start):
		raise CdxmlOpenQtE2eError("File/Open start callback is not callable")
	completion_loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer(window)
	timeout.setSingleShot(True)
	outcome: bool | None = None

	def finish(success: bool) -> None:
		"""Record the one public queue result and stop the local event loop."""
		nonlocal outcome
		if outcome is None:
			outcome = success
			completion_loop.quit()

	window.local_document_open_queue_drained.connect(finish)
	timeout.timeout.connect(lambda: finish(False))
	try:
		start()
		if outcome is None:
			timeout.start(10000)
			completion_loop.exec()
	finally:
		timeout.stop()
		window.local_document_open_queue_drained.disconnect(finish)
		timeout.timeout.disconnect()
	return outcome is True


#============================================
def _require_styled_publication(window: object) -> None:
	"""Verify native projection, renderer admission, and durable CDML style tokens."""
	tab = _active_tab(window)
	render = tab._render_observation
	if render is None or len(render.molecule_plans) != 1:
		raise CdxmlOpenQtE2eError("styled CDXML did not publish one render observation")
	plan = render.molecule_plans[0]
	if plan.plan.issues or plan.member_issues:
		raise CdxmlOpenQtE2eError("styled CDXML published a render plan with issues")
	molecules = tab.current_document_observation().projection.molecules
	if len(molecules) != 1:
		raise CdxmlOpenQtE2eError("styled CDXML did not publish one molecule")
	presentations = tuple(bond.presentation for bond in molecules[0].bonds)
	if presentations != (
		ferrum_chem.DocumentBondPresentationV1.wavy,
		ferrum_chem.DocumentBondPresentationV1.bold,
		ferrum_chem.DocumentBondPresentationV1.dashed,
	):
		raise CdxmlOpenQtE2eError("styled CDXML did not preserve its closed presentations")
	cdml = tab.current_snapshot.cdml
	if any(f'type="{token}"' not in cdml for token in ("s1", "b1", "d1")):
		raise CdxmlOpenQtE2eError("styled CDXML did not preserve s1, b1, and d1")


#============================================
def _require_current_tab_refusal(window: object, source: pathlib.Path) -> None:
	"""Verify NewTab-only CDXML refuses replacement without changing the current tab."""
	tab = _active_tab(window)
	before_snapshot = tab.current_snapshot
	warnings: list[object] = []
	window._show_edit_refusal = warnings.append
	if window.open_in_current_tab_path(str(source)):
		raise CdxmlOpenQtE2eError("CDXML incorrectly entered the Current Tab route")
	if _active_tab(window) is not tab or tab.current_snapshot is not before_snapshot:
		raise CdxmlOpenQtE2eError("Current Tab CDXML refusal mutated the active document")
	if not warnings:
		raise CdxmlOpenQtE2eError("Current Tab CDXML refusal was not presented")


#============================================
def main() -> int:
	"""Run the complete styled-CDXML Qt ingress and refusal workflow."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(app),
	)
	try:
		with tempfile.TemporaryDirectory(prefix="ferrum-cdxml-open-qt-") as temporary:
			directory = pathlib.Path(temporary)
			source = directory / "styled.cdxml"
			source.write_text(_STYLED_CDXML, encoding="utf-8")
			original_chooser = PySide6.QtWidgets.QFileDialog.getOpenFileName
			PySide6.QtWidgets.QFileDialog.getOpenFileName = staticmethod(
				lambda *_args, **_kwargs: (str(source), "ChemDraw XML (*.cdxml)"),
			)
			try:
				window.show()
				app.processEvents()
				if not _await_open(window, _open_action(window).trigger):
					raise CdxmlOpenQtE2eError("File/Open did not publish styled CDXML")
			finally:
				PySide6.QtWidgets.QFileDialog.getOpenFileName = original_chooser
			_require_styled_publication(window)
			_require_current_tab_refusal(window, source)
		print(json.dumps({"status": "ok"}))
		return 0
	finally:
		ferrum_qt_e2e.close_e2e_main_window(window, app)


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except CdxmlOpenQtE2eError as exc:
		print(f"e2e_cdxml_open_qt: {exc}", file=sys.stderr)
		raise SystemExit(1)

"""Offscreen P0.3 atom/bond structural deletion workflow."""

import pathlib

# local repo modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()

import ferrum_chem
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager

_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.08'><molecule id='m'><atom id='a' name='C'><point x='0' y='0'/></atom><atom id='b' name='C'><point x='40' y='0'/></atom><atom id='c' name='O'><point x='80' y='0'/></atom><bond id='ab' start='a' end='b' type='n1'/><bond id='bc' start='b' end='c' type='n1'/></molecule></cdml>"""

def point(tab: object, scene_x: float) -> PySide6.QtCore.QPoint:
	"""Return the visible atom target at one Rust-issued scene coordinate."""
	observation = tab.observe_structure_interaction()
	target = next(
		value for value in observation.targets
		if value.kind is ferrum_chem.StructureTargetKindV1.atom
		and abs((value.bounds.left + value.bounds.right) / 2.0 - scene_x) < 0.01
	)
	return tab.view.mapFromScene(PySide6.QtCore.QPointF((target.bounds.left + target.bounds.right) / 2.0, (target.bounds.top + target.bounds.bottom) / 2.0))

def main() -> None:
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_CDML, "structure.cdml", window._require_document_display_palette(),
	)
	window._register_native_tab(tab, activate=True)
	window.show(); app.processEvents()
	try:
		window._select_structure_action.trigger()
		PySide6.QtTest.QTest.mouseClick(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point(tab, 40.0))
		window._commit_structure_deletion()
		if len(tab.current_document_observation().projection.molecules) != 2:
			raise RuntimeError("structural delete did not split molecule roots")
		moved = tab.current_snapshot.revision
		if tab.undo().observation.snapshot.revision <= moved:
			raise RuntimeError("structural delete undo did not create a Rust revision")
		path = pathlib.Path("/private/tmp/ferrum-p0-structure-delete-e2e.cdml")
		tab.save_atomic(path)
		if ferrum_chem.DocumentSession.load(path.read_text()).snapshot().digest != tab.current_snapshot.digest:
			raise RuntimeError("structural delete save/reopen changed Rust state")
		print('{"schema":"ferrum-p0-structure-delete-e2e-v1","status":"ok"}')
	finally:
		tab.dispose(); window.deleteLater()

if __name__ == "__main__":
	raise SystemExit(main())

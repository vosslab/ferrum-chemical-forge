"""Exercise the installed Ferrum public ribbon route for one shipped template."""

import json
import pathlib
import shutil
import sys
import tempfile

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

import ferrum_chem
import ferrum_qt.ferrum.catalog_palette
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Return one authored scene coordinate in the live canvas viewport."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


#============================================
def _ribbon_exposes_action(window: object, action: object) -> bool:
	"""Return whether one visible Authoring Ribbon button owns the QAction."""
	return any(
		button.defaultAction() is action
		for button in window._authoring_ribbon.findChildren(
			PySide6.QtWidgets.QToolButton,
		)
	)


#============================================
def _catalog_modal(app: PySide6.QtWidgets.QApplication) -> None:
	"""Search and accept Benzene through the live modal palette controls."""
	palette = app.activeModalWidget()
	if not isinstance(palette, ferrum_qt.ferrum.catalog_palette.FerrumCatalogPalette):
		raise RuntimeError("The public Insert Template action did not open the Ferrum palette")
	palette.search.setText("benzene")
	app.processEvents()
	if palette.selected_key() != "system/rings/benzene":
		raise RuntimeError("The public Ferrum catalog modal did not select benzene")
	palette.place_button.click()


#============================================
def _cancel_catalog_modal(app: PySide6.QtWidgets.QApplication) -> None:
	"""Reject the live modal through its Qt dialog lifecycle."""
	palette = app.activeModalWidget()
	if not isinstance(palette, ferrum_qt.ferrum.catalog_palette.FerrumCatalogPalette):
		raise RuntimeError("The public Insert Template action did not open the Ferrum palette")
	palette.reject()


#============================================
def _assert_structure_owner_retired(window: object, phase: str) -> None:
	"""Require the former Select Structure controller to be terminally inactive."""
	if window._select_structure_action.isChecked():
		raise RuntimeError("Select Structure action revived after " + phase)
	if window._structure_viewport is not None or window._structure_selection is not None:
		raise RuntimeError("Select Structure canvas owner revived after " + phase)


#============================================
def _arm_structure_owner(window: object) -> None:
	"""Activate the live Select Structure action before catalog handoff."""
	window._select_structure_action.trigger()
	if not window._select_structure_action.isChecked() or window._structure_viewport is None:
		raise RuntimeError("The public Select Structure action did not arm its controller")


#============================================
def main() -> int:
	"""Prove the installed public window places and preserves a shipped template."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	root = pathlib.Path(tempfile.mkdtemp(prefix="ferrum-catalog-public-e2e-", dir=pathlib.Path.cwd()))
	try:
		window.show()
		app.processEvents()
		tab = window._active_native_tab()
		if tab is None:
			raise RuntimeError("The public Ferrum window did not create a Rust document tab")
		if not _ribbon_exposes_action(window, window._insert_catalog_template_action):
			raise RuntimeError("The public Ferrum ribbon does not expose Insert Template")
		before = tab.current_snapshot.revision
		_arm_structure_owner(window)
		PySide6.QtCore.QTimer.singleShot(0, lambda: _catalog_modal(app))
		window._insert_catalog_template_action.trigger()
		intent = window._catalog_placement_intent
		if intent is None:
			raise RuntimeError("The public catalog modal did not begin opaque placement")
		_assert_structure_owner_retired(window, "catalog activation")
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		app.processEvents()
		if tab.current_snapshot.revision != before or window._catalog_placement_intent is not None:
			raise RuntimeError("Cancelling the public catalog placement changed the document")
		_assert_structure_owner_retired(window, "catalog cancellation")
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 80.0, 60.0),
		)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		app.processEvents()
		_assert_structure_owner_retired(window, "post-cancel canvas events")
		_arm_structure_owner(window)
		PySide6.QtCore.QTimer.singleShot(0, lambda: _catalog_modal(app))
		window._insert_catalog_template_action.trigger()
		_assert_structure_owner_retired(window, "second catalog activation")
		intent = window._catalog_placement_intent
		if intent is None:
			raise RuntimeError("The second public catalog modal did not begin opaque placement")
		anchor = _point(tab, 80.0, 60.0)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), anchor)
		app.processEvents()
		intent = window._catalog_placement_intent
		if intent is None or intent.preview is None or intent.item is None:
			raise RuntimeError(
				"The public catalog placement did not paint a Rust preview: "
				+ window.statusBar().currentMessage(),
			)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor,
		)
		app.processEvents()
		created = tab.current_snapshot.cdml
		if "Benzene" not in created or tab.current_snapshot.revision != before + 1:
			raise RuntimeError("The public catalog click did not commit canonical benzene")
		_assert_structure_owner_retired(window, "catalog commit")
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor,
		)
		app.processEvents()
		_assert_structure_owner_retired(window, "post-commit canvas click")
		window._translate_roots_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor,
		)
		moved = _point(tab, 100.0, 80.0)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), moved)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, moved,
		)
		app.processEvents()
		if tab.current_snapshot.cdml == created:
			raise RuntimeError("Move Complete Roots did not move public catalog benzene")
		tab.undo()
		if tab.current_snapshot.cdml != created:
			raise RuntimeError("Undo did not restore public catalog benzene")
		path = root / "ferrum-catalog-public-e2e.cdml"
		if not window.save_active_to_path(str(path)):
			raise RuntimeError("Public Ferrum Save As did not publish catalog benzene")
		reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8")).snapshot()
		if "Benzene" not in reopened.cdml:
			raise RuntimeError("Rust reopen did not preserve public catalog benzene")
		print(json.dumps({"schema": "ferrum-template-catalog-public-window-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()
		shutil.rmtree(root, ignore_errors=True)


if __name__ == "__main__":
	sys.exit(main())

"""Offscreen Ferrum workflow: browse, cancel, place, save, and reopen a template."""

# Standard Library
import json
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import e2e_workspace
import ferrum_chem
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
def _action(window: PySide6.QtWidgets.QMainWindow, text: str) -> PySide6.QtGui.QAction:
	"""Return one public action by its user-facing label."""
	return next(action for action in window.findChildren(PySide6.QtGui.QAction) if action.text() == text)


#============================================
def _canvas(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtWidgets.QGraphicsView:
	"""Return Ferrum's visible, accessibly named document canvas."""
	canvas = next(
		view for view in window.findChildren(PySide6.QtWidgets.QGraphicsView)
		if view.isVisible() and view.accessibleName() == "Ferrum drawing canvas"
	)
	return canvas


#============================================
def _modal_catalog(app: PySide6.QtWidgets.QApplication) -> PySide6.QtWidgets.QDialog:
	"""Return the currently visible public template dialog."""
	dialog = app.activeModalWidget()
	if (
		not isinstance(dialog, PySide6.QtWidgets.QDialog)
		or not dialog.isVisible()
		or dialog.accessibleName() != "Ferrum template palette"
	):
		raise RuntimeError("Insert Template did not open its public modal dialog")
	return dialog


#============================================
def _reject_catalog_modal(app: PySide6.QtWidgets.QApplication) -> None:
	"""Cancel the public modal without starting a placement."""
	_modal_catalog(app).reject()


#============================================
def _select_enabled_result_for_placement(app: PySide6.QtWidgets.QApplication) -> None:
	"""Select and accept one enabled visible result through public dialog controls."""
	dialog = _modal_catalog(app)
	results = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QListWidget)
		if widget.accessibleName() == "Ferrum template results"
	)
	selected = next(
		(
			results.item(row) for row in range(results.count())
			if not results.item(row).isHidden()
			and results.item(row).flags() & PySide6.QtCore.Qt.ItemFlag.ItemIsEnabled
		),
		None,
	)
	if selected is None:
		raise RuntimeError("Insert Template did not expose an enabled visible result")
	results.setCurrentItem(selected)
	results.scrollToItem(selected)
	app.processEvents()
	button_box = next(
		box for box in dialog.findChildren(PySide6.QtWidgets.QDialogButtonBox)
		if box.isVisible()
	)
	place_button = next(
		button for button in button_box.buttons()
		if button_box.buttonRole(button) is PySide6.QtWidgets.QDialogButtonBox.ButtonRole.AcceptRole
		and button.isVisible() and button.isEnabled()
	)
	place_button.click()


#============================================
def _ribbon_exposes(window: PySide6.QtWidgets.QMainWindow, action: PySide6.QtGui.QAction) -> bool:
	"""Report whether the visible authoring ribbon presents one public action."""
	return any(
		button.isVisible() and button.defaultAction() is action
		for button in window.findChildren(PySide6.QtWidgets.QToolButton)
	)


#============================================
def _accept_save_as_modal(app: PySide6.QtWidgets.QApplication, path: pathlib.Path) -> None:
	"""Choose one deterministic destination through the visible native Save As dialog."""
	dialog = app.activeModalWidget()
	if not isinstance(dialog, PySide6.QtWidgets.QFileDialog) or not dialog.isVisible():
		raise RuntimeError("Save As did not open its public file dialog")
	dialog.selectFile(str(path))
	dialog.accept()


#============================================
def _save_as(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, path: pathlib.Path) -> None:
	"""Publish through Ferrum's visible Save As workflow."""
	save_as = _action(window, "Save As")
	PySide6.QtCore.QTimer.singleShot(0, lambda: _accept_save_as_modal(app, path))
	save_as.trigger()
	app.processEvents()
	if not path.is_file():
		raise RuntimeError("Save As did not publish a document artifact")


#============================================
def _load_digest(path: pathlib.Path) -> str:
	"""Observe one saved public artifact through the Rust document loader."""
	session = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
	digest = session.snapshot().digest
	return digest


#============================================
def main() -> int:
	"""Prove the public catalog route preserves one placed template."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		with e2e_workspace.E2EWorkspaceLease() as workspace_text:
			workspace = pathlib.Path(workspace_text)
			window.show()
			app.processEvents()
			insert_template = _action(window, "Insert Template...")
			if not _ribbon_exposes(window, insert_template):
				raise RuntimeError("Ferrum ribbon does not expose Insert Template")
			canvas = _canvas(window)
			baseline_path = workspace / "template-catalog-baseline.cdml"
			cancelled_path = workspace / "template-catalog-cancelled.cdml"
			placed_path = workspace / "template-catalog-placed.cdml"
			_save_as(window, app, baseline_path)
			baseline_digest = _load_digest(baseline_path)
			PySide6.QtCore.QTimer.singleShot(0, lambda: _reject_catalog_modal(app))
			insert_template.trigger()
			app.processEvents()
			_save_as(window, app, cancelled_path)
			cancelled_digest = _load_digest(cancelled_path)
			if cancelled_digest != baseline_digest:
				raise RuntimeError("cancelling Insert Template changed the document")
			PySide6.QtCore.QTimer.singleShot(0, lambda: _select_enabled_result_for_placement(app))
			insert_template.trigger()
			app.processEvents()
			anchor = canvas.viewport().rect().center()
			PySide6.QtTest.QTest.mouseMove(canvas.viewport(), anchor)
			PySide6.QtTest.QTest.mouseClick(
				canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor,
			)
			app.processEvents()
			_save_as(window, app, placed_path)
			placed_digest = _load_digest(placed_path)
			if placed_digest == cancelled_digest:
				raise RuntimeError("catalog placement did not change the saved document artifact")
			reopened = ferrum_chem.DocumentSession.load(placed_path.read_text(encoding="utf-8"))
			if reopened.snapshot().digest != placed_digest:
				raise RuntimeError("Rust reopen did not preserve the placed document")
			print(json.dumps({"schema": "ferrum-template-catalog-authoring-e2e-v2", "status": "ok"}))
			return 0
	finally:
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	raise SystemExit(main())

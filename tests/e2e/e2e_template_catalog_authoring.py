"""Offscreen Ferrum workflow: browse, cancel, place, save, and reopen a template."""

# Standard Library
import collections.abc
import json
import pathlib

# local repo modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()


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
	return next(
		action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == text
	)


#============================================
def _canvas(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtWidgets.QGraphicsView:
	"""Return Ferrum's visible, accessibly named document canvas."""
	canvas = next(
		view for view in window.findChildren(PySide6.QtWidgets.QGraphicsView)
		if view.isVisible() and view.accessibleName() == "Ferrum drawing canvas"
	)
	return canvas


#============================================
def _catalog(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtWidgets.QDialog:
	"""Return the visible, deliberately modeless public template dialog."""
	dialog = next((
		candidate for candidate in window.findChildren(PySide6.QtWidgets.QDialog)
		if candidate.isVisible() and candidate.accessibleName() == "Template Catalog"
	), None)
	if dialog is None or dialog.isModal():
		raise RuntimeError("Template Catalog did not open its public modeless dialog")
	return dialog


#============================================
def _reject_catalog(window: PySide6.QtWidgets.QMainWindow) -> None:
	"""Close the public modeless catalog without starting a placement."""
	_catalog(window).reject()


#============================================
def _select_enabled_result_for_placement(window: PySide6.QtWidgets.QMainWindow) -> None:
	"""Select and accept one enabled visible result through public dialog controls."""
	dialog = _catalog(window)
	results = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QListWidget)
		if widget.accessibleName() == "Template catalog results"
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
		raise RuntimeError("Template Catalog did not expose an enabled visible result")
	results.setCurrentItem(selected)
	results.scrollToItem(selected)
	PySide6.QtWidgets.QApplication.processEvents()
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
def _select_saved_template_for_placement(window: PySide6.QtWidgets.QMainWindow) -> None:
	"""Find one saved template through the public source and search controls."""
	dialog = _catalog(window)
	source = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QComboBox)
		if widget.accessibleName() == "Template source"
	)
	source.setCurrentIndex(source.findData("user_directory"))
	search = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QLineEdit)
		if widget.accessibleName() == "Search templates"
	)
	results = next(
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QListWidget)
		if widget.accessibleName() == "Template catalog results"
	)
	if results.count() == 0:
		state = next(
			widget for widget in dialog.findChildren(PySide6.QtWidgets.QLabel)
			if widget.accessibleName() == "Template catalog status"
		)
		details = next(
			widget for widget in dialog.findChildren(PySide6.QtWidgets.QPlainTextEdit)
			if widget.accessibleName() == "Template refresh details"
		)
		raise RuntimeError(
			"My templates did not expose the saved template: "
			f"{state.text()} {details.toPlainText()}",
		)
	search.setText(results.item(0).text())
	_select_enabled_result_for_placement(window)


#============================================
def _open_template_catalog(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, template_catalog: PySide6.QtGui.QAction,
		on_open: collections.abc.Callable[[], None]) -> None:
	"""Open the catalog through Structure's visible primary ribbon control."""
	tabs = next(
		widget for widget in window.findChildren(PySide6.QtWidgets.QTabBar)
		if widget.isVisible() and widget.accessibleName() == "Authoring tasks"
	)
	structure_index = next(
		index for index in range(tabs.count()) if tabs.tabText(index) == "Structure"
	)
	PySide6.QtTest.QTest.mouseClick(
		tabs, PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		tabs.tabRect(structure_index).center(),
	)
	app.processEvents()
	group = next(
		widget for widget in window.findChildren(PySide6.QtWidgets.QWidget)
		if widget.isVisible()
		and widget.objectName() == "ribbon-group-groups_templates"
		and widget.accessibleName() == "Groups and templates commands"
	)
	direct_clients = [
		widget for widget in group.findChildren(PySide6.QtWidgets.QToolButton)
		if widget.isVisible() and widget.defaultAction() is template_catalog
	]
	if len(direct_clients) != 1:
		raise RuntimeError("Groups and templates did not expose one visible Template Catalog control")
	direct_client = direct_clients[0]
	if not direct_client.isEnabled():
		raise RuntimeError("visible Template Catalog control was disabled")
	if direct_client.text() != "Template Catalog":
		raise RuntimeError("visible Template Catalog control had an unexpected label")
	if direct_client.accessibleName() != "Template Catalog...":
		raise RuntimeError("visible Template Catalog control had an unexpected accessible name")
	PySide6.QtCore.QTimer.singleShot(0, lambda: PySide6.QtCore.QTimer.singleShot(0, on_open))
	PySide6.QtTest.QTest.mouseClick(
		direct_client, PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, direct_client.rect().center(),
	)
	app.processEvents()


#============================================
def _open_template_catalog_from_chemistry_menu(
		window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication,
		on_open: collections.abc.Callable[[], None],
		) -> None:
	"""Invoke Template Catalog through its visible Chemistry menu client."""
	chemistry = next(
		menu for menu in window.findChildren(PySide6.QtWidgets.QMenu)
		if menu.title().replace("&", "") == "Chemistry"
	)
	chemistry.popup(window.menuBar().mapToGlobal(window.menuBar().rect().bottomLeft()))
	app.processEvents()
	menu_action = next(
		action for action in chemistry.actions() if action.text() == "Template Catalog..."
	)
	if not menu_action.isEnabled():
		raise RuntimeError("Chemistry menu exposed a disabled Template Catalog command")
	PySide6.QtCore.QTimer.singleShot(0, lambda: PySide6.QtCore.QTimer.singleShot(0, on_open))
	PySide6.QtTest.QTest.mouseClick(
		chemistry, PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		chemistry.actionGeometry(menu_action).center(),
	)
	app.processEvents()


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
def _save_current_as_template(window: PySide6.QtWidgets.QMainWindow,
		app: PySide6.QtWidgets.QApplication, path: pathlib.Path) -> None:
	"""Publish through the visible Save Current as Template command and file dialog."""
	action = _action(window, "Save Current as Template...")
	PySide6.QtCore.QTimer.singleShot(0, lambda: _accept_save_as_modal(app, path))
	action.trigger()
	app.processEvents()
	if not path.is_file():
		raise RuntimeError("Save Current as Template did not publish a template artifact")


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
	window: ferrum_qt.main_window.MainWindow | None = None
	try:
		with e2e_workspace.E2EWorkspaceLease() as workspace_text:
			workspace = pathlib.Path(workspace_text)
			template_directory = workspace / "templates"
			window = ferrum_qt.main_window.MainWindow(
				theme_manager, user_template_directory=template_directory,
			)
			window.show()
			app.processEvents()
			template_catalog = window._action_registry.get_qt_action("chemistry.template.catalog")
			canvas = _canvas(window)
			baseline_path = workspace / "template-catalog-baseline.cdml"
			cancelled_path = workspace / "template-catalog-cancelled.cdml"
			placed_path = workspace / "template-catalog-placed.cdml"
			reusable_path = template_directory / "reusable.cdml"
			reused_path = workspace / "template-catalog-reused.cdml"
			_save_as(window, app, baseline_path)
			baseline_digest = _load_digest(baseline_path)
			_open_template_catalog(
				window, app, template_catalog, lambda: _reject_catalog(window),
			)
			_open_template_catalog_from_chemistry_menu(
				window, app, lambda: _reject_catalog(window),
			)
			app.processEvents()
			_save_as(window, app, cancelled_path)
			cancelled_digest = _load_digest(cancelled_path)
			if cancelled_digest != baseline_digest:
				raise RuntimeError("cancelling Template Catalog changed the document")
			_open_template_catalog(
				window, app, template_catalog, lambda: _select_enabled_result_for_placement(window),
			)
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
			_save_current_as_template(window, app, reusable_path)
			_open_template_catalog(
				window, app, template_catalog,
				lambda: _select_saved_template_for_placement(window),
			)
			app.processEvents()
			second_anchor = canvas.viewport().rect().topLeft() + PySide6.QtCore.QPoint(24, 24)
			PySide6.QtTest.QTest.mouseClick(
				canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, second_anchor,
			)
			app.processEvents()
			_save_as(window, app, reused_path)
			reused_digest = _load_digest(reused_path)
			if reused_digest == placed_digest:
				raise RuntimeError("saved template placement did not change the saved document artifact")
			reopened = ferrum_chem.DocumentSession.load(reused_path.read_text(encoding="utf-8"))
			if reopened.snapshot().digest != reused_digest:
				raise RuntimeError("Rust reopen did not preserve the saved-template placement")
			print(json.dumps({"schema": "ferrum-template-catalog-authoring-e2e-v2", "status": "ok"}))
			return 0
	finally:
		if window is not None:
			ferrum_qt_e2e.close_e2e_main_window(window, app)


if __name__ == "__main__":
	raise SystemExit(main())

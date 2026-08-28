#!/usr/bin/env python3
"""Run Ferrum's open, keyboard-select, context, save, and Rust-reopen workflow."""

# Standard Library
import argparse
import pathlib
import sys

# local repo modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()


# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.close_decision
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
class KeyboardWorkflowError(RuntimeError):
	"""Report a failed observable keyboard-only product workflow."""


#============================================
def parse_args() -> argparse.Namespace:
	"""Parse explicit fixture and output paths for the installed Ferrum product."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--input", type=pathlib.Path, required=True)
	parser.add_argument("--output", type=pathlib.Path, required=True)
	return parser.parse_args()


#============================================
def wait_for(predicate: object, application: PySide6.QtWidgets.QApplication) -> None:
	"""Wait boundedly for the app's existing asynchronous file-open boundary."""
	for _unused in range(100):
		application.processEvents()
		if predicate():
			return
		PySide6.QtTest.QTest.qWait(25)
	raise KeyboardWorkflowError("Ferrum did not finish opening the requested document")


#============================================
def press(window: PySide6.QtWidgets.QWidget, key: object,
		modifier: object = PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		flush: bool = True) -> None:
	"""Send one author-visible QTest keyboard event, optionally retaining its boundary."""
	PySide6.QtTest.QTest.keyClick(window, key, modifier)
	if flush:
		PySide6.QtWidgets.QApplication.instance().processEvents()


#============================================
def find_atom(projection: object, document_object_id: str) -> object | None:
	"""Return one Rust projection atom by its durable document-object identity."""
	for molecule in projection.molecules:
		for atom in molecule.atoms:
			if atom.document_object_id == document_object_id:
				return atom
	return None


#============================================
def selection_bridge_diagnostic(tab: object, window: object) -> str:
	"""Render bounded bridge facts when keyboard action state diverges."""
	bridge = tab._structure_action_selection_v1
	if bridge is None:
		targets = None
	else:
		targets = tuple((repr(target.kind), target.object_id) for target in bridge.targets)
	atom_action = window._action_registry.get_qt_action("edit.atom.properties")
	return repr({
		"bridge_is_none": bridge is None,
		"bridge_targets": targets,
		"has_one_selected_atom": tab.has_one_selected_atom(),
		"requires_refresh": tab.requires_refresh,
		"atom_properties_enabled": atom_action.isEnabled(),
		"select_structure_checked": window._select_structure_action.isChecked(),
		"select_structure_enabled": window._select_structure_action.isEnabled(),
	})


#============================================
def trigger_atom_properties_and_escape(
		application: PySide6.QtWidgets.QApplication,
		menu: PySide6.QtWidgets.QMenu,
		) -> None:
	"""Activate and dismiss Atom Properties through its public keyboard route."""
	delivery: dict[str, object | None] = {"dialog": None, "error": None, "completed": False}

	def dismiss_from_keyboard() -> None:
		"""Verify the public dialog then use its public Escape input."""
		dialog = application.activeModalWidget()
		if not isinstance(dialog, PySide6.QtWidgets.QDialog) or not dialog.isVisible():
			delivery["error"] = "Atom Properties did not open a visible modal Ferrum dialog"
			if isinstance(dialog, PySide6.QtWidgets.QDialog):
				dialog.reject()
			return
		delivery["dialog"] = dialog
		PySide6.QtTest.QTest.keyClick(dialog, PySide6.QtCore.Qt.Key.Key_Escape)

	def release_liveness_guard() -> None:
		"""Release a synchronous modal loop while preserving a diagnostic failure."""
		if delivery["completed"] or delivery["error"] is not None:
			return
		delivery["error"] = (
			"Atom Properties did not return after keyboard Escape before the E2E "
			"liveness guard"
		)
		dialog = application.activeModalWidget()
		if isinstance(dialog, PySide6.QtWidgets.QDialog):
			dialog.reject()

	PySide6.QtCore.QTimer.singleShot(0, dismiss_from_keyboard)
	PySide6.QtCore.QTimer.singleShot(5000, release_liveness_guard)
	press(menu, PySide6.QtCore.Qt.Key.Key_Return)
	delivery["completed"] = True
	if delivery["error"] is not None:
		raise KeyboardWorkflowError(str(delivery["error"]))
	if delivery["dialog"] is None:
		raise KeyboardWorkflowError(
			"Atom Properties action returned without presenting its modal dialog",
		)


#============================================
#============================================
def main() -> int:
	"""Exercise structural keyboard selection after deterministic dialog setup."""
	args = parse_args()
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(app),
	)
	try:
		# This is a test-only file-dialog boundary. The Open and Save As commands
		# themselves are activated by their product keyboard shortcuts below.
		original_open = PySide6.QtWidgets.QFileDialog.getOpenFileName
		original_save = PySide6.QtWidgets.QFileDialog.getSaveFileName
		PySide6.QtWidgets.QFileDialog.getOpenFileName = staticmethod(
			lambda *_args, **_kwargs: (str(args.input), "CDML (*.cdml)"),
		)
		PySide6.QtWidgets.QFileDialog.getSaveFileName = staticmethod(
			lambda *_args, **_kwargs: (str(args.output), "CDML (*.cdml)"),
		)
		window.show()
		app.processEvents()
		press(window, PySide6.QtCore.Qt.Key.Key_O,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
		wait_for(lambda: window._active_native_tab() is not None and (
			window._active_native_tab().title == args.input.name
		), app)
		tab = window._active_native_tab()
		if tab is None:
			raise KeyboardWorkflowError("Open shortcut did not install a document tab")
		initial_projection = tab.current_document_observation().projection
		initial_atoms = initial_projection.molecules[0].atoms
		if len(initial_atoms) != 1 or initial_atoms[0].element != "C":
			raise KeyboardWorkflowError("keyboard fixture lost its ordinary carbon projection")
		original_atom_id = initial_atoms[0].document_object_id
		tab.view.set_hex_grid_snap_enabled(False)
		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(10.0, 20.0))
		press(window, PySide6.QtCore.Qt.Key.Key_K,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
		palette = window._command_palette_controller.dialog
		if not palette.isVisible() or not palette.search_field.hasFocus():
			raise KeyboardWorkflowError("Command Palette shortcut did not expose keyboard command search")
		PySide6.QtTest.QTest.keyClicks(palette.search_field, "select structure")
		press(palette.search_field, PySide6.QtCore.Qt.Key.Key_Return)
		if not window._select_structure_action.isChecked():
			raise KeyboardWorkflowError("Keyboard command palette did not activate Select Structure")
		if not tab.view.viewport().hasFocus():
			raise KeyboardWorkflowError(
				"Keyboard command activation did not restore Select Structure canvas focus",
			)
		press(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Right)
		press(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Left)
		press(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return)
		selection = window._structure_selection
		if selection is None or tuple(
			target.object_id for target in selection.targets
		) != (original_atom_id,):
			raise KeyboardWorkflowError("keyboard cursor selection did not retain the Rust-opened atom")
		bridge_before_menu = selection_bridge_diagnostic(tab, window)
		press(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Menu)
		context_menu = PySide6.QtWidgets.QApplication.activePopupWidget()
		if not isinstance(context_menu, PySide6.QtWidgets.QMenu):
			raise KeyboardWorkflowError(
				"Menu key did not expose selected-structure actions; "
				f"before Menu: {bridge_before_menu}; after Menu: "
				f"{selection_bridge_diagnostic(tab, window)}",
			)
		atom_properties = window._action_registry.get_qt_action("edit.atom.properties")
		if not atom_properties.isEnabled():
			context_action_ids = tuple(
				view.action_id for view in window._action_registry.live_action_views()
				if view.qt_action in context_menu.actions()
			)
			selected_target = selection.targets[0]
			raise KeyboardWorkflowError(
				"keyboard atom selection did not enable Atom Properties for "
				f"{selected_target.kind!r}/{selected_target.object_id}; "
				f"enabled context actions: {context_action_ids!r}; "
				f"before Menu: {bridge_before_menu}; after Menu: "
				f"{selection_bridge_diagnostic(tab, window)}",
			)
		if atom_properties not in context_menu.actions():
			raise KeyboardWorkflowError("keyboard context menu omitted the enabled Atom Properties action")
		context_menu.setActiveAction(atom_properties)
		context_menu.setFocus()
		trigger_atom_properties_and_escape(app, context_menu)
		app.processEvents()
		if not tab.view.viewport().hasFocus():
			raise KeyboardWorkflowError("Atom Properties did not restore keyboard focus to the canvas")
		press(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_S,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier
			| PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier)
		if not args.output.is_file():
			raise KeyboardWorkflowError("Save As shortcut did not publish the destination")
		reopened = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
			args.output.read_text(encoding="utf-8"), args.output.name,
			window._require_document_display_palette(),
		)
		try:
			reopened_projection = reopened.current_document_observation().projection
			if find_atom(reopened_projection, original_atom_id) is None:
				raise KeyboardWorkflowError("Rust reopen lost the keyboard-selected atom")
		finally:
			reopened.dispose()
		return 0
	finally:
		PySide6.QtWidgets.QFileDialog.getOpenFileName = original_open
		PySide6.QtWidgets.QFileDialog.getSaveFileName = original_save
		while window._tab_widget.count():
			window._close_native_tab_at(
				0, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
			)
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except KeyboardWorkflowError as exc:
		print(f"e2e_keyboard_workflow: {exc}", file=sys.stderr)
		raise SystemExit(1)

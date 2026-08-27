#!/usr/bin/env python3
"""Run Ferrum's open, keyboard-author, undo, save, and Rust-reopen workflow."""

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
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager
import ferrum_qt.canvas.ferrum_render_target


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
def find_bond_between(projection: object, first: str, second: str) -> object | None:
	"""Return the bond with these exact durable endpoint identities, if present."""
	for molecule in projection.molecules:
		for bond in molecule.bonds:
			if frozenset((bond.start.document_object_id, bond.end.document_object_id)) == frozenset((first, second)):
				return bond
	return None


#============================================
def main() -> int:
	"""Exercise only keyboard actions after launch, with deterministic dialog paths."""
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
		original_target = ferrum_qt.canvas.ferrum_render_target.RenderTargetKey(
			"document_object", original_atom_id,
		)
		if original_target not in tab._require_projection().durable_items:
			raise KeyboardWorkflowError(
				"keyboard fixture lacks a durable Rust-to-Qt render target",
			)
		tab.view.set_hex_grid_snap_enabled(False)
		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(90.0, 80.0))
		press(window, PySide6.QtCore.Qt.Key.Key_8,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
		press(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return, flush=False)
		created = tab.selected_atom_projection()
		if (
			created is None
			or created.document_object_id == original_atom_id
			or (created.position.x, created.position.y) != (90.0, 80.0)
		):
			raise KeyboardWorkflowError(
				"Return did not immediately select the newly created Rust atom at (90, 80)",
			)
		created_atom_id = created.document_object_id
		created_target = ferrum_qt.canvas.ferrum_render_target.RenderTargetKey(
			"document_object", created_atom_id,
		)
		if created_target not in tab._require_projection().durable_items:
			raise KeyboardWorkflowError("newly created Rust atom lacks a durable Qt render target")
		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(10.0, 20.0))
		press(window, PySide6.QtCore.Qt.Key.Key_2,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
		press(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return)
		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(
			created.position.x, created.position.y,
		))
		press(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return)
		bond = find_bond_between(
			tab.current_document_observation().projection, original_atom_id, created_atom_id,
		)
		if bond is None:
			raise KeyboardWorkflowError("keyboard bond did not use the newly created Rust atom")
		press(window, PySide6.QtCore.Qt.Key.Key_Z,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
		if find_bond_between(
			tab.current_document_observation().projection, original_atom_id, created_atom_id,
		) is not None:
			raise KeyboardWorkflowError("Undo shortcut did not remove the keyboard bond")
		press(window, PySide6.QtCore.Qt.Key.Key_S,
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
			reopened_atom = find_atom(reopened_projection, created_atom_id)
			if (
				reopened_atom is None
				or (reopened_atom.position.x, reopened_atom.position.y) != (90.0, 80.0)
				or find_bond_between(reopened_projection, original_atom_id, created_atom_id) is not None
			):
				raise KeyboardWorkflowError("Rust reopen lost the saved keyboard workflow state")
			if created_target not in reopened._require_projection().durable_items:
				raise KeyboardWorkflowError("Rust reopen lost the created atom's durable render target")
		finally:
			reopened.dispose()
		return 0
	finally:
		PySide6.QtWidgets.QFileDialog.getOpenFileName = original_open
		PySide6.QtWidgets.QFileDialog.getSaveFileName = original_save
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except KeyboardWorkflowError as exc:
		print(f"e2e_keyboard_workflow: {exc}", file=sys.stderr)
		raise SystemExit(1)

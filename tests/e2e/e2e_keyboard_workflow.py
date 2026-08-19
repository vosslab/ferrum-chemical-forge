#!/usr/bin/env python3
"""Run Ferrum's open, keyboard-author, undo, save, and Rust-reopen workflow."""

# Standard Library
import argparse
import pathlib
import sys

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


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
		modifier: object = PySide6.QtCore.Qt.KeyboardModifier.NoModifier) -> None:
	"""Send one author-visible QTest keyboard event and flush Qt delivery."""
	PySide6.QtTest.QTest.keyClick(window, key, modifier)
	PySide6.QtWidgets.QApplication.instance().processEvents()


#============================================
def main() -> int:
	"""Exercise only keyboard actions after launch, with deterministic dialog paths."""
	args = parse_args()
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.main_window.MainWindow(object())
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
		tab.view.set_hex_grid_snap_enabled(False)
		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(90.0, 80.0))
		press(window, PySide6.QtCore.Qt.Key.Key_8,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
		press(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return)
		created = tab.selected_atom_projection()
		if created is None:
			raise KeyboardWorkflowError("keyboard atom placement did not select Rust atom")
		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(10.0, 20.0))
		press(window, PySide6.QtCore.Qt.Key.Key_2,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
		press(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return)
		tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(
			created.position.x, created.position.y,
		))
		press(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return)
		if len(tab.current_document_observation().projection.molecules[0].bonds) != 1:
			raise KeyboardWorkflowError("keyboard bond placement did not reach Rust")
		press(window, PySide6.QtCore.Qt.Key.Key_Z,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier)
		if len(tab.current_document_observation().projection.molecules[0].bonds) != 0:
			raise KeyboardWorkflowError("Undo shortcut did not remove only the keyboard bond")
		press(window, PySide6.QtCore.Qt.Key.Key_S,
			PySide6.QtCore.Qt.KeyboardModifier.ControlModifier
			| PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier)
		if not args.output.is_file():
			raise KeyboardWorkflowError("Save As shortcut did not publish the destination")
		reopened = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
			args.output.read_text(encoding="utf-8"), args.output.name,
		)
		try:
			molecule = reopened.current_document_observation().projection.molecules[0]
			if len(molecule.atoms) != 2 or len(molecule.bonds) != 0:
				raise KeyboardWorkflowError("reopened Rust facts differ from saved keyboard edit")
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

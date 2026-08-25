#!/usr/bin/env python3
"""Exercise the public native peptide-sequence import action offscreen."""

# Standard Library
import json
import os
import sys


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_installation
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
class PeptideSequenceImportE2eError(RuntimeError):
	"""Report one failed public peptide-import workflow assertion."""


#============================================
def _peptide_import_action(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtGui.QAction:
	"""Return the visible Chemistry action for native peptide import."""
	for action in window.findChildren(PySide6.QtGui.QAction):
		if action.text() == window.tr("Import Supported Peptide Sequence..."):
			return action
	raise PeptideSequenceImportE2eError("Ferrum did not expose peptide import in Chemistry")


#============================================
def _trigger_sequence_import(
		window: PySide6.QtWidgets.QMainWindow, sequence: str,
		) -> None:
	"""Enter one sequence in the real public dialog, then trigger its action."""
	dialog_delivery = {"error": None}

	def enter_sequence() -> None:
		"""Complete the modal text dialog on its ordinary Qt event turn."""
		dialog = PySide6.QtWidgets.QApplication.activeModalWidget()
		if not isinstance(dialog, PySide6.QtWidgets.QInputDialog):
			dialog_delivery["error"] = "Ferrum did not present its peptide text dialog"
			if dialog is not None:
				dialog.reject()
			return
		dialog.setTextValue(sequence)
		dialog.accept()

	PySide6.QtCore.QTimer.singleShot(0, enter_sequence)
	_peptide_import_action(window).trigger()
	if dialog_delivery["error"] is not None:
		raise PeptideSequenceImportE2eError(dialog_delivery["error"])


#============================================
def _await_installation(window: PySide6.QtWidgets.QMainWindow, sequence: str) -> None:
	"""Run one accepted public import through the queued worker lifecycle."""
	receipt: object | None = None
	completion_loop = PySide6.QtCore.QEventLoop()

	def receive_installation(value: object) -> None:
		"""Finish after the product reports the installed document."""
		nonlocal receipt
		receipt = value
		completion_loop.quit()

	window.document_installation_completed.connect(receive_installation)
	try:
		_trigger_sequence_import(window, sequence)
		completion_loop.exec()
	finally:
		window.document_installation_completed.disconnect(receive_installation)
	if receipt is None:
		raise PeptideSequenceImportE2eError("peptide import did not publish an installation")
	if type(receipt) is not ferrum_qt.ferrum.document_installation.FerrumDocumentInstallationV1:
		raise PeptideSequenceImportE2eError("peptide import did not publish its typed installation")
	if receipt.installation_kind != "peptide_sequence_import":
		raise PeptideSequenceImportE2eError("peptide import used the wrong installation route")


#============================================
def main() -> int:
	"""Run accepted peptide import against the staged Ferrum product."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		window.show()
		app.processEvents()
		_await_installation(window, "AC")
		print(json.dumps({"schema": "ferrum-peptide-sequence-import-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.deleteLater()


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except PeptideSequenceImportE2eError as exc:
		print(f"e2e_peptide_sequence_import: {exc}", file=sys.stderr)
		raise SystemExit(1)

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
def _active_canvas_tab(window: PySide6.QtWidgets.QMainWindow) -> object:
	"""Return the current document through the product's visible tab control."""
	tabs = window.centralWidget()
	if not isinstance(tabs, PySide6.QtWidgets.QTabWidget):
		raise PeptideSequenceImportE2eError("Ferrum did not expose its public document tabs")
	tab = tabs.currentWidget()
	if tab is None:
		raise PeptideSequenceImportE2eError("Ferrum did not select its initial document")
	return tab


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
		) -> object:
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
	intent = getattr(window, "_peptide_import_intent")
	if intent is None:
		raise PeptideSequenceImportE2eError("Ferrum did not begin the public peptide import")
	return intent.worker


#============================================
def _await_installation(window: PySide6.QtWidgets.QMainWindow, sequence: str) -> object:
	"""Run one accepted public import through the queued worker lifecycle."""
	receipts: list[object] = []
	completion_loop = PySide6.QtCore.QEventLoop()

	def receive_installation(receipt: object) -> None:
		"""Finish after the product reports the installed document."""
		receipts.append(receipt)
		completion_loop.quit()

	window.document_installation_completed.connect(receive_installation)
	try:
		_trigger_sequence_import(window, sequence)
		completion_loop.exec()
	finally:
		window.document_installation_completed.disconnect(receive_installation)
	if len(receipts) != 1:
		raise PeptideSequenceImportE2eError("peptide import did not publish one installation")
	receipt = receipts[0]
	if type(receipt) is not ferrum_qt.ferrum.document_installation.FerrumDocumentInstallationV1:
		raise PeptideSequenceImportE2eError("peptide import did not publish its typed installation")
	if receipt.installation_kind != "peptide_sequence_import":
		raise PeptideSequenceImportE2eError("peptide import used the wrong installation route")
	return receipt


#============================================
def _await_unsupported_refusal(
		window: PySide6.QtWidgets.QMainWindow, sequence: str,
		) -> object:
	"""Run one unsupported import until its real queued worker reports refusal."""
	completion_loop = PySide6.QtCore.QEventLoop()
	failures: list[object] = []
	worker = _trigger_sequence_import(window, sequence)

	def receive_failure(failure: object) -> None:
		"""Keep the typed worker failure without depending on presentation text."""
		failures.append(failure)
		modal = PySide6.QtWidgets.QApplication.activeModalWidget()
		if isinstance(modal, PySide6.QtWidgets.QMessageBox):
			modal.accept()
		completion_loop.quit()

	worker.failed.connect(receive_failure)
	try:
		completion_loop.exec()
	finally:
		worker.failed.disconnect(receive_failure)
	if len(failures) != 1:
		raise PeptideSequenceImportE2eError("unsupported peptide import did not refuse once")
	return failures[0]


#============================================
def main() -> int:
	"""Run accepted and refused peptide imports against the staged Ferrum product."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		window.show()
		app.processEvents()
		tab = _active_canvas_tab(window)
		initial_snapshot = tab.current_snapshot
		receipt = _await_installation(window, "AC")
		if receipt.source_revision != initial_snapshot.revision:
			raise PeptideSequenceImportE2eError("peptide installation lost its initial document")
		installed_tab = _active_canvas_tab(window)
		installed_snapshot = installed_tab.current_snapshot
		if installed_snapshot.digest != receipt.current_digest_hex:
			raise PeptideSequenceImportE2eError("peptide installation did not reach the active document")
		if not installed_tab.current_document_observation().projection.molecules:
			raise PeptideSequenceImportE2eError("peptide installation did not commit a molecule")
		failure = _await_unsupported_refusal(window, "AH")
		if failure.error_type != "UnsupportedFerrumPeptideProfileError":
			raise PeptideSequenceImportE2eError("unsupported peptide import lost its typed refusal")
		unchanged_tab = _active_canvas_tab(window)
		unchanged_snapshot = unchanged_tab.current_snapshot
		if (
			unchanged_tab is not installed_tab
			or unchanged_snapshot.revision != installed_snapshot.revision
			or unchanged_snapshot.digest != installed_snapshot.digest
		):
			raise PeptideSequenceImportE2eError("unsupported peptide import changed the current document")
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

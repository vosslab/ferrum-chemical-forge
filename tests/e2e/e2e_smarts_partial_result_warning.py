#!/usr/bin/env python3
"""Exercise SMARTS partial-result warning semantics through the staged Qt product."""

# Standard Library
import collections.abc
import json
import os
import pathlib
import sys
import tempfile


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


# This size crosses the current product-wide SMARTS result budget without making
# that tunable budget itself an assertion contract.
GLOBAL_MATCH_BUDGET_CROSSING_MOLECULES = 201
_PARTIAL_WARNING_TERMS = ("Unexamined molecules", "may contain matches")


#============================================
class SmartsPartialResultWarningE2eError(RuntimeError):
	"""Report a failed learner-visible SMARTS partial-result workflow."""


#============================================
def _wait_for_local_document_open(
		window: ferrum_qt.main_window.MainWindow,
		expected_path: str,
		start: collections.abc.Callable[[], object],
		) -> None:
	"""Complete the supplied public document-open request by its path-specific signal."""
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)
	completed = {"success": None}

	def receive_completion(path: str, success: bool) -> None:
		"""Record only the requested document's installed completion outcome."""
		if path != expected_path:
			return
		completed["success"] = success
		loop.quit()

	def receive_timeout() -> None:
		"""End the event loop when local document installation does not complete."""
		loop.quit()

	window.local_document_open_completed.connect(receive_completion)
	timeout.timeout.connect(receive_timeout)
	try:
		PySide6.QtCore.QTimer.singleShot(0, start)
		timeout.start(10000)
		loop.exec()
	finally:
		timeout.stop()
		window.local_document_open_completed.disconnect(receive_completion)
		timeout.timeout.disconnect(receive_timeout)
	if completed["success"] is not True:
		raise SmartsPartialResultWarningE2eError("Ferrum did not finish opening the SMARTS document")


#============================================
def _write_carbon_document(path: pathlib.Path, molecule_count: int) -> None:
	"""Create deterministic local CDML with one carbon match per molecule."""
	molecules = "\n".join(
		"  <molecule id='mol-{0}'><atom id='atom-{0}' name='C'><point x='{0}' y='20'/>"
		"</atom></molecule>".format(index)
		for index in range(molecule_count)
	)
	path.write_text(
		"<cdml xmlns='urn:ferrum:cdml' version='26.08'>\n{0}\n</cdml>\n".format(molecules),
		encoding="utf-8",
	)


#============================================
def _open_smarts_query_from_chemistry_menu(
		window: ferrum_qt.main_window.MainWindow,
		app: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Open the SMARTS dock with the visible Chemistry-menu command."""
	menu_bar = window.menuBar()
	for menu_action in menu_bar.actions():
		menu = menu_action.menu()
		if menu is None:
			continue
		for action in menu.actions():
			if action.text().replace("&", "") != "SMARTS Query...":
				continue
			PySide6.QtTest.QTest.mouseClick(
				menu_bar, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu_bar.actionGeometry(menu_action).center(),
			)
			app.processEvents()
			PySide6.QtTest.QTest.mouseClick(
				menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu.actionGeometry(action).center(),
			)
			app.processEvents()
			return
	raise SmartsPartialResultWarningE2eError("Chemistry menu did not expose SMARTS Query")


#============================================
def _run_raw_carbon_query(
		window: ferrum_qt.main_window.MainWindow,
		app: PySide6.QtWidgets.QApplication,
		) -> PySide6.QtWidgets.QLabel:
	"""Run the dock's supported raw SMARTS query against the active document."""
	_open_smarts_query_from_chemistry_menu(window, app)
	dock = window.findChild(PySide6.QtWidgets.QDockWidget, "smarts-query-dock")
	if dock is None:
		raise SmartsPartialResultWarningE2eError("SMARTS Query did not open its dock")
	query_input = dock.findChild(PySide6.QtWidgets.QLineEdit, "smarts-query-input")
	find_button = dock.findChild(PySide6.QtWidgets.QPushButton, "smarts-query-find")
	status = dock.findChild(PySide6.QtWidgets.QLabel, "smarts-query-status")
	if query_input is None or find_button is None or status is None:
		raise SmartsPartialResultWarningE2eError("SMARTS dock omitted a required public control")
	query_input.setText("[C]")
	PySide6.QtTest.QTest.mouseClick(find_button, PySide6.QtCore.Qt.MouseButton.LeftButton)
	app.processEvents()
	app.processEvents()
	return status


#============================================
def _has_partial_warning(message: str) -> bool:
	"""Recognize the learner-facing global-incompleteness warning semantically."""
	return all(term in message for term in _PARTIAL_WARNING_TERMS)


#============================================
def _has_partial_warning_fragment(message: str) -> bool:
	"""Recognize any semantic fragment of the global-incompleteness warning."""
	return any(term in message for term in _PARTIAL_WARNING_TERMS)


#============================================
def _open_document(
		window: ferrum_qt.main_window.MainWindow,
		app: PySide6.QtWidgets.QApplication,
		path: pathlib.Path,
		) -> None:
	"""Open one local deterministic document through the public product route."""
	path_text = str(path)
	_wait_for_local_document_open(window, path_text, lambda: window.open_file_path(path_text))
	app.processEvents()


#============================================
def main() -> int:
	"""Prove complete and partial SMARTS results stay distinguishable to learners."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		with tempfile.TemporaryDirectory(prefix="ferrum-smarts-warning-") as temporary_directory:
			temporary_path = pathlib.Path(temporary_directory)
			small_path = temporary_path / "small-complete.cdml"
			large_path = temporary_path / "global-budget-crossing.cdml"
			_write_carbon_document(small_path, 1)
			_write_carbon_document(large_path, GLOBAL_MATCH_BUDGET_CROSSING_MOLECULES)
			window.show()
			app.processEvents()
			_open_document(window, app, small_path)
			complete_status = _run_raw_carbon_query(window, app)
			if _has_partial_warning_fragment(complete_status.text()) or _has_partial_warning_fragment(
					complete_status.accessibleDescription(),
					):
				raise SmartsPartialResultWarningE2eError(
					"a complete SMARTS result incorrectly announced partial results",
				)
			_open_document(window, app, large_path)
			partial_status = _run_raw_carbon_query(window, app)
			if not _has_partial_warning(partial_status.text()):
				raise SmartsPartialResultWarningE2eError(
					"partial SMARTS results omitted their visible global warning",
				)
			if not _has_partial_warning(partial_status.accessibleDescription()):
				raise SmartsPartialResultWarningE2eError(
					"partial SMARTS results omitted their accessible global warning",
				)
		print(json.dumps({"schema": "ferrum-smarts-partial-result-warning-e2e-v1", "status": "ok"}))
		return 0
	finally:
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except SmartsPartialResultWarningE2eError as exc:
		print("e2e_smarts_partial_result_warning: {0}".format(exc), file=sys.stderr)
		raise SystemExit(1)

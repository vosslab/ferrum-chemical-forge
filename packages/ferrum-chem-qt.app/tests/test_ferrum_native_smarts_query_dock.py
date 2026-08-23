"""Behavior coverage for the user-visible Ferrum SMARTS query dock."""

# Standard Library
import collections.abc
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window


_CARBON_CDML = """<cdml xmlns='urn:ferrum:cdml' version='26.08'>
  <molecule id='mol-1'>
	<atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  </molecule>
</cdml>"""


#============================================
def _wait_for_open_queue(
		window: ferrum_qt.main_window.MainWindow,
		start: collections.abc.Callable[[], object],
		) -> bool:
	"""Run one public Open request until its declared completion signal fires."""
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)
	outcome: bool | None = None

	def finish(success: bool) -> None:
		"""Capture the first public Open completion result."""
		nonlocal outcome
		if outcome is None:
			outcome = success
			loop.quit()

	def expire() -> None:
		"""Finish the bounded wait when the public completion signal is missing."""
		finish(False)

	window.local_document_open_queue_drained.connect(finish)
	timeout.timeout.connect(expire)
	PySide6.QtCore.QTimer.singleShot(0, start)
	timeout.start(10000)
	loop.exec()
	timeout.stop()
	window.local_document_open_queue_drained.disconnect(finish)
	timeout.timeout.disconnect(expire)
	return outcome is True


#============================================
def _open_saved_document(
		window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""Load a test document through the public Ferrum File/Open operation."""
	path = tmp_path / "smarts-status.cdml"
	path.write_text(_CARBON_CDML, encoding="utf-8")
	window.show()
	qapp.processEvents()
	if not _wait_for_open_queue(window, lambda: window.open_file_path(str(path))):
		raise AssertionError("Ferrum did not finish opening the SMARTS test document")
	qapp.processEvents()


#============================================
def _open_smarts_query_from_chemistry_menu(
		window: PySide6.QtWidgets.QMainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Open the dock through the visible Chemistry menu command."""
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
			qapp.processEvents()
			PySide6.QtTest.QTest.mouseClick(
				menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu.actionGeometry(action).center(),
			)
			qapp.processEvents()
			return
	raise AssertionError("SMARTS Query command is unavailable from the Chemistry menu")


#============================================
def test_smarts_clear_keeps_its_completion_status_after_controls_refresh(
		main_window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""Clearing a completed query leaves an accessible completion message visible."""
	_open_saved_document(main_window, qapp, tmp_path)
	_open_smarts_query_from_chemistry_menu(main_window, qapp)
	dock = main_window.findChild(PySide6.QtWidgets.QDockWidget, "smarts-query-dock")
	assert dock is not None
	query_input = dock.findChild(PySide6.QtWidgets.QLineEdit, "smarts-query-input")
	find_button = dock.findChild(PySide6.QtWidgets.QPushButton, "smarts-query-find")
	status = dock.findChild(PySide6.QtWidgets.QLabel, "smarts-query-status")
	clear_button = next(
		button for button in dock.findChildren(PySide6.QtWidgets.QPushButton)
		if button.accessibleName() == "Clear SMARTS results"
	)
	assert query_input is not None and find_button is not None and status is not None
	query_input.setText("[C]")
	PySide6.QtTest.QTest.mouseClick(find_button, PySide6.QtCore.Qt.MouseButton.LeftButton)
	qapp.processEvents()
	qapp.processEvents()
	assert clear_button.isEnabled()
	PySide6.QtTest.QTest.mouseClick(clear_button, PySide6.QtCore.Qt.MouseButton.LeftButton)
	qapp.processEvents()
	assert (
		status.text() == "SMARTS results cleared."
		and status.accessibleDescription() == "SMARTS results cleared."
		and not clear_button.isEnabled()
	)


#============================================
def test_smarts_native_invalid_query_shows_its_documented_recovery(
		main_window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""A native SMARTS syntax refusal remains a visible query-recovery outcome."""
	_open_saved_document(main_window, qapp, tmp_path)
	_open_smarts_query_from_chemistry_menu(main_window, qapp)
	dock = main_window.findChild(PySide6.QtWidgets.QDockWidget, "smarts-query-dock")
	assert dock is not None
	query_input = dock.findChild(PySide6.QtWidgets.QLineEdit, "smarts-query-input")
	find_button = dock.findChild(PySide6.QtWidgets.QPushButton, "smarts-query-find")
	status = dock.findChild(PySide6.QtWidgets.QLabel, "smarts-query-status")
	assert query_input is not None and find_button is not None and status is not None
	query_input.setText("[")
	PySide6.QtTest.QTest.mouseClick(find_button, PySide6.QtCore.Qt.MouseButton.LeftButton)
	qapp.processEvents()
	qapp.processEvents()
	assert status.text() == "Ferrum could not read that SMARTS query. Check its syntax and try again."


#============================================
def test_smarts_selected_molecule_readiness_keeps_its_token_runnable(
		main_window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""The visible selected-molecule workflow can check readiness then run its one token."""
	_open_saved_document(main_window, qapp, tmp_path)
	_open_smarts_query_from_chemistry_menu(main_window, qapp)
	dock = main_window.findChild(PySide6.QtWidgets.QDockWidget, "smarts-query-dock")
	assert dock is not None
	choose_button = dock.findChild(PySide6.QtWidgets.QPushButton, "smarts-query-choose-molecule")
	find_button = dock.findChild(PySide6.QtWidgets.QPushButton, "smarts-query-find")
	status = dock.findChild(PySide6.QtWidgets.QLabel, "smarts-query-status")
	canvas = next(
		(view for view in main_window.findChildren(PySide6.QtWidgets.QGraphicsView)
			if view.isVisible() and view.accessibleName() == "Ferrum drawing canvas"),
		None,
	)
	assert choose_button is not None and find_button is not None and status is not None and canvas is not None
	PySide6.QtTest.QTest.mouseClick(choose_button, PySide6.QtCore.Qt.MouseButton.LeftButton)
	qapp.processEvents()
	point = canvas.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
	PySide6.QtTest.QTest.mouseClick(
		canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
	)
	qapp.processEvents()
	assert find_button.isEnabled() and status.text() == "Chosen molecule is ready. Choose Find to search this drawing."
	PySide6.QtTest.QTest.mouseClick(find_button, PySide6.QtCore.Qt.MouseButton.LeftButton)
	qapp.processEvents()
	qapp.processEvents()
	assert status.text() == "Found 1 matches in 1 molecules."

"""Behavior coverage for the user-visible Ferrum SMARTS query dock."""

# Standard Library
import collections.abc
import math
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.close_decision
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
def _smarts_controls(
		window: ferrum_qt.main_window.MainWindow,
		) -> tuple[
		PySide6.QtWidgets.QDockWidget,
		PySide6.QtWidgets.QLineEdit,
		PySide6.QtWidgets.QPushButton,
		PySide6.QtWidgets.QLabel,
		PySide6.QtWidgets.QTreeWidget,
		]:
	"""Return the visible controls that define one learner SMARTS interaction."""
	dock = window.findChild(PySide6.QtWidgets.QDockWidget, "smarts-query-dock")
	assert dock is not None
	query_input = dock.findChild(PySide6.QtWidgets.QLineEdit, "smarts-query-input")
	find_button = dock.findChild(PySide6.QtWidgets.QPushButton, "smarts-query-find")
	status = dock.findChild(PySide6.QtWidgets.QLabel, "smarts-query-status")
	results = dock.findChild(PySide6.QtWidgets.QTreeWidget, "smarts-query-results")
	assert query_input is not None and find_button is not None and status is not None and results is not None
	return dock, query_input, find_button, status, results


#============================================
def _run_raw_carbon_query(
		window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		) -> tuple[PySide6.QtWidgets.QLabel, PySide6.QtWidgets.QTreeWidget]:
	"""Run the supported raw-carbon workflow through visible dock controls."""
	_open_smarts_query_from_chemistry_menu(window, qapp)
	_unused_dock, query_input, find_button, status, results = _smarts_controls(window)
	query_input.setText("[C]")
	PySide6.QtTest.QTest.mouseClick(find_button, PySide6.QtCore.Qt.MouseButton.LeftButton)
	qapp.processEvents()
	qapp.processEvents()
	assert status.text() == "Found 1 matches in 1 molecules."
	return status, results


#============================================
def _only_match(results: PySide6.QtWidgets.QTreeWidget) -> PySide6.QtWidgets.QTreeWidgetItem:
	"""Return the one user-selectable result from the deterministic carbon fixture."""
	assert results.topLevelItemCount() == 1
	group = results.topLevelItem(0)
	assert group is not None and group.childCount() == 1
	match = group.child(0)
	assert match is not None
	return match


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


#============================================
def test_smarts_reveal_paints_only_finite_canvas_bounds_and_clear_revokes_the_route(
		main_window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""A learner can reveal one result, then Clear removes its replay route and paint."""
	_open_saved_document(main_window, qapp, tmp_path)
	status, results = _run_raw_carbon_query(main_window, qapp)
	match = _only_match(results)
	before = len(main_window._active_native_tab().view.scene().items())
	results.setCurrentItem(match)
	results.itemActivated.emit(match, 0)
	qapp.processEvents()
	tab = main_window._active_native_tab()
	assert status.text() == "Match shown." and tab._live_smarts_overlay_item_v1 is not None
	overlay = tab._live_smarts_overlay_item_v1
	assert all(
		math.isfinite(value)
		for child in overlay.childItems()
		for value in (
			child.rect().left(), child.rect().top(),
			child.rect().right(), child.rect().bottom(),
		)
	)
	clear_button = next(
		button for button in main_window.findChildren(PySide6.QtWidgets.QPushButton)
		if button.accessibleName() == "Clear SMARTS results"
	)
	PySide6.QtTest.QTest.mouseClick(clear_button, PySide6.QtCore.Qt.MouseButton.LeftButton)
	qapp.processEvents()
	assert (
		status.text() == "SMARTS results cleared."
		and results.topLevelItemCount() == 0
		and tab._live_smarts_overlay_item_v1 is None
		and len(tab.view.scene().items()) == before
	)


#============================================
def test_smarts_tab_switch_and_close_revoke_old_result_before_another_document_can_use_it(
		main_window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""A result belongs to its tab and cannot survive a tab switch or that tab closing."""
	_open_saved_document(main_window, qapp, tmp_path)
	status, results = _run_raw_carbon_query(main_window, qapp)
	first_tab = main_window._active_native_tab()
	assert _only_match(results) is not None
	second_path = tmp_path / "second-smarts-status.cdml"
	second_path.write_text(_CARBON_CDML, encoding="utf-8")
	assert _wait_for_open_queue(main_window, lambda: main_window.open_file_path(str(second_path)))
	qapp.processEvents()
	second_tab = main_window._active_native_tab()
	assert second_tab is not first_tab
	assert (
		status.text() == "The active drawing changed. Run the query again."
		and results.topLevelItemCount() == 0
		and first_tab._live_smarts_receipt_v1 is None
	)
	main_window._tab_widget.setCurrentWidget(first_tab)
	qapp.processEvents()
	assert results.topLevelItemCount() == 0 and status.text() == "The active drawing changed. Run the query again."
	close_index = main_window._tab_widget.indexOf(first_tab)
	assert close_index >= 0
	assert main_window._close_native_tab_at(
		close_index, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
	) is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
	qapp.processEvents()
	assert first_tab.is_disposed and main_window._active_native_tab() is second_tab


#============================================
def test_smarts_reprojection_invalidates_a_visible_result_and_leaves_a_recovery_state(
		main_window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""A renderer reprojection revokes stale display facts before the new plan is used."""
	_open_saved_document(main_window, qapp, tmp_path)
	status, results = _run_raw_carbon_query(main_window, qapp)
	tab = main_window._active_native_tab()
	assert _only_match(results) is not None
	tab._session.observe_render(tab.current_snapshot.revision)
	qapp.processEvents()
	assert (
		status.text() == "The drawing changed. Run the query again."
		and results.topLevelItemCount() == 0
		and tab._live_smarts_receipt_v1 is None
		and tab._live_smarts_overlay_item_v1 is None
	)

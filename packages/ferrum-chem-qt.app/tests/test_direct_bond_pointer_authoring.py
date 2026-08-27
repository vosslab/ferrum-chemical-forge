"""Public canvas coverage for Rust-owned normal direct-bond authoring."""

# Standard Library
import collections.abc
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.line_tool_completion


_EDITABLE_CDML = """<cdml xmlns='urn:ferrum:cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
    <atom id='atom-o' name='O'><point x='70' y='20'/></atom>
  </molecule>
</cdml>"""

_OVERLAPPING_CDML = """<cdml xmlns='urn:ferrum:cdml'>
  <molecule id='mol-1'>
    <atom id='atom-a' name='C'><point x='10' y='20'/></atom>
    <atom id='atom-b' name='O'><point x='10' y='20'/></atom>
  </molecule>
</cdml>"""


#============================================
def _wait_for_open_queue(window: ferrum_qt.main_window.MainWindow,
		start: collections.abc.Callable[[], object]) -> bool:
	"""Run one public Open request until its declared completion signal fires."""
	loop = PySide6.QtCore.QEventLoop()
	outcome: bool | None = None

	def finish(success: bool) -> None:
		"""Capture the first public Open completion result."""
		nonlocal outcome
		if outcome is None:
			outcome = success
			loop.quit()

	window.local_document_open_queue_drained.connect(finish)
	try:
		start()
		if outcome is None:
			loop.exec()
	finally:
		window.local_document_open_queue_drained.disconnect(finish)
	return outcome is True


#============================================
def _current_tab(window: ferrum_qt.main_window.MainWindow) -> PySide6.QtWidgets.QWidget:
	"""Return the selected public canvas page through the central widget tree."""
	tabs = window.centralWidget()
	if not isinstance(tabs, PySide6.QtWidgets.QTabWidget):
		raise AssertionError("Ferrum does not expose a tabbed central canvas")
	tab = tabs.currentWidget()
	if tab is None:
		raise AssertionError("Ferrum has no active canvas document")
	return tab


#============================================
def _action(window: ferrum_qt.main_window.MainWindow,
		action_id: str) -> PySide6.QtGui.QAction:
	"""Return one registered visible authoring action by its stable identifier."""
	action = window._action_registry.get_qt_action(action_id)
	if action is None:
		raise AssertionError(f"Ferrum action is unavailable: {action_id}")
	return action


#============================================
def _viewport_point(tab: object, document_object_id: str) -> PySide6.QtCore.QPoint:
	"""Map one publicly observed atom position to the canvas viewport."""
	atom = next(
		atom for molecule in tab.current_document_observation().projection.molecules
		for atom in molecule.atoms if atom.document_object_id == document_object_id
	)
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(atom.position.x, atom.position.y))


#============================================
def _nearby_non_hit(tab: object, document_object_id: str) -> PySide6.QtCore.QPoint:
	"""Find a close viewport point with no direct atom item for native V3 resolution."""
	center = _viewport_point(tab, document_object_id)
	for offset in range(1, 7):
		candidate = center + PySide6.QtCore.QPoint(offset, 0)
		if tab.durable_atom_at_viewport_point(candidate) is None:
			return candidate
	raise AssertionError("Ferrum atom rendering left no nearby point for V3 native resolution")


#============================================
def _open_window(qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		cdml: str) -> tuple[ferrum_qt.main_window.MainWindow, object]:
	"""Open a test CDML file through the product's public File/Open operation."""
	path = tmp_path / "direct_bond.cdml"
	path.write_text(cdml, encoding="utf-8")
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	window.show()
	qapp.processEvents()
	if not _wait_for_open_queue(window, lambda: window.open_file_path(str(path))):
		raise AssertionError("Ferrum did not finish opening the direct-bond document")
	qapp.processEvents()
	return window, _current_tab(window)


#============================================
def _atom_document_object_ids(tab: object) -> tuple[str, ...]:
	"""Return the current projection's Rust-issued atom addresses in fixture order."""
	return tuple(
		atom.document_object_id
		for atom in tab.current_document_observation().projection.molecules[0].atoms
	)


#============================================
def _close_window(qapp: PySide6.QtWidgets.QApplication,
		window: ferrum_qt.main_window.MainWindow) -> None:
	"""Close the public host after canceling its current interaction normally."""
	window.cancel_active_pointer_authoring()
	window.close()
	window.deleteLater()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.DeferredDelete,
	)
	qapp.processEvents()


#============================================
class _ModalRefusalObserver(PySide6.QtCore.QObject):
	"""Capture and acknowledge one product message box when it becomes visible."""

	def __init__(self, refusals: list[tuple[str, str, str]]) -> None:
		"""Keep the caller-owned receipt list for the observed dialog."""
		super().__init__()
		self._refusals = refusals

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Queue acknowledgement after the actual modal dialog enters its event loop."""
		if event.type() != PySide6.QtCore.QEvent.Type.Show:
			return False
		if not isinstance(watched, PySide6.QtWidgets.QMessageBox):
			return False
		PySide6.QtCore.QTimer.singleShot(0, lambda: self._capture_and_dismiss(watched))
		return False

	def _capture_and_dismiss(self, dialog: PySide6.QtWidgets.QMessageBox) -> None:
		"""Record the configured refusal once its queued presenter update has completed."""
		self._refusals.append((
			dialog.windowTitle(), dialog.text(), dialog.detailedText(),
		))
		button = dialog.button(PySide6.QtWidgets.QMessageBox.StandardButton.Ok)
		if button is None:
			raise AssertionError("Draw Bond refusal has no visible acknowledgement")
		PySide6.QtTest.QTest.mouseClick(
			button, PySide6.QtCore.Qt.MouseButton.LeftButton,
		)


#============================================
def test_normal_pointer_direct_hits_create_the_durable_normal_bond(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Exact atom hits become V3 evidence and commit the public durable bond."""
	window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
	try:
		start_id, end_id = _atom_document_object_ids(tab)
		start = _viewport_point(tab, start_id)
		end = _viewport_point(tab, end_id)
		_action(window, "draw.bond").trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()
		bonds = tab.current_document_observation().projection.molecules[0].bonds
		assert len(bonds) == 1
		assert bonds[0].source_type == "n1"
	finally:
		_close_window(qapp, window)


#============================================
def test_normal_direct_bond_native_no_hit_resolution_reaches_new_endpoints(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Empty Qt hits leave nearest and new-endpoint decisions to native V3."""
	window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
	try:
		start_id, end_id = _atom_document_object_ids(tab)
		start = _nearby_non_hit(tab, start_id)
		end = _viewport_point(tab, end_id)
		_action(window, "draw.bond").trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()
		molecule = tab.current_document_observation().projection.molecules[0]
		assert (len(molecule.atoms), molecule.bonds[0].start.document_object_id) == (2, start_id)
	finally:
		_close_window(qapp, window)

	blank = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	blank.show()
	qapp.processEvents()
	try:
		blank_tab = _current_tab(blank)
		start = blank_tab.view.mapFromScene(PySide6.QtCore.QPointF(100.0, 100.0))
		end = blank_tab.view.mapFromScene(PySide6.QtCore.QPointF(180.0, 100.0))
		_action(blank, "draw.bond").trigger()
		PySide6.QtTest.QTest.mousePress(blank_tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(blank_tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(blank_tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()
		molecule = blank_tab.current_document_observation().projection.molecules[0]
		assert (len(molecule.atoms), len(molecule.bonds)) == (2, 1)
	finally:
		_close_window(qapp, blank)


#============================================
def test_normal_direct_bond_ambiguous_scene_evidence_is_non_modal(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Overlapping atom items remain V3 ambiguity rather than a Qt stacking-order pick."""
	window, tab = _open_window(qapp, tmp_path, _OVERLAPPING_CDML)
	try:
		point = _viewport_point(tab, _atom_document_object_ids(tab)[0])
		action = _action(window, "draw.bond")
		action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		qapp.processEvents()
		assert "Draw Bond refused" in window.statusBar().currentMessage()
		assert not action.isChecked()
	finally:
		_close_window(qapp, window)


#============================================
def test_normal_direct_bond_escape_unchecks_its_visible_action(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Escape cancels normal authoring while preserving the observed document."""
	window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
	try:
		start = _viewport_point(tab, _atom_document_object_ids(tab)[0])
		before = tab.current_document_observation().projection.molecules[0]
		action = _action(window, "draw.bond")
		action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		qapp.processEvents()
		assert not action.isChecked()
		after = tab.current_document_observation().projection.molecules[0]
		assert (len(after.atoms), len(after.bonds)) == (len(before.atoms), len(before.bonds))
	finally:
		_close_window(qapp, window)


#============================================
def test_normal_direct_bond_refusal_is_modal_and_actionable(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A generic self-loop refusal is visible, actionable, and leaves no bond."""
	window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
	try:
		start = _viewport_point(tab, _atom_document_object_ids(tab)[0])
		refusal_dialogs: list[tuple[str, str, str]] = []
		observer = _ModalRefusalObserver(refusal_dialogs)
		qapp.installEventFilter(observer)
		try:
			_action(window, "draw.bond").trigger()
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), start)
			PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		finally:
			qapp.removeEventFilter(observer)
		qapp.processEvents()
		assert refusal_dialogs == [(
			"",
			"What happened: This action is not available for the current drawing.\n\n"
			"Why: The needed selection or document state is not available.\n\n"
			"What to do now: Select the required item or change the drawing, then try again.",
			"document operation rejected",
		)]
		assert not _action(window, "draw.bond").isChecked()
		assert not tab.current_document_observation().projection.molecules[0].bonds
	finally:
		_close_window(qapp, window)

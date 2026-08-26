"""Public canvas coverage for Rust-owned directed stereo-bond authoring."""

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


_EDITABLE_CDML = """<cdml xmlns='urn:ferrum:cdml'>
  <molecule id='mol-1'>
    <atom id='tip' name='C'><point x='10' y='20'/></atom>
    <atom id='base' name='O'><point x='90' y='20'/></atom>
  </molecule>
</cdml>"""


#============================================
def _wait_for_open_queue(window: ferrum_qt.main_window.MainWindow,
		start: collections.abc.Callable[[], object]) -> bool:
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
	action = window.findChild(PySide6.QtGui.QAction, action_id)
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
def _open_window(qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		cdml: str) -> tuple[ferrum_qt.main_window.MainWindow, object]:
	"""Open a test CDML file through the product's public File/Open operation."""
	path = tmp_path / "stereo_bond.cdml"
	path.write_text(cdml, encoding="utf-8")
	window = ferrum_qt.main_window.MainWindow(object())
	window.show()
	qapp.processEvents()
	if not _wait_for_open_queue(window, lambda: window.open_file_path(str(path))):
		raise AssertionError("Ferrum did not finish opening the stereo-bond document")
	qapp.processEvents()
	return window, _current_tab(window)


#============================================
def _atom_document_object_ids(tab: object) -> tuple[str, ...]:
	"""Return current Rust atom addresses in their fixture presentation order."""
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
def test_stereo_actions_create_directed_bonds(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Both wedge actions preserve the public durable tip-to-base bond order."""
	for action_id, source_type in (
		("mode.draw_solid_wedge", "w1"),
		("mode.draw_hashed_wedge", "h1"),
	):
		window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
		try:
			tip_id, base_id = _atom_document_object_ids(tab)
			start = _viewport_point(tab, tip_id)
			end = _viewport_point(tab, base_id)
			_action(window, action_id).trigger()
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
			PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
			qapp.processEvents()
			bond = tab.current_document_observation().projection.molecules[0].bonds[0]
			assert bond.source_type == source_type
			assert (bond.start.document_object_id, bond.end.document_object_id) == (tip_id, base_id)
		finally:
			_close_window(qapp, window)


#============================================
def test_stereo_actions_direct_existing_new_bonds_from_tip_to_blank_base(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Both wedge actions retain the clicked tip when their base is new."""
	for action_id, source_type in (
		("mode.draw_solid_wedge", "w1"),
		("mode.draw_hashed_wedge", "h1"),
	):
		window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
		try:
			tip_id, _base_id = _atom_document_object_ids(tab)
			start = _viewport_point(tab, tip_id)
			end = tab.view.mapFromScene(PySide6.QtCore.QPointF(150.0, 20.0))
			_action(window, action_id).trigger()
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
			PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
			qapp.processEvents()
			molecule = tab.current_document_observation().projection.molecules[0]
			bond = molecule.bonds[0]
			assert (bond.source_type, bond.start.document_object_id) == (source_type, tip_id)
			assert bond.end.document_object_id != tip_id
		finally:
			_close_window(qapp, window)


#============================================
def test_stereo_actions_support_new_existing_and_new_new_endpoints(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Public stereo actions forward both missing endpoint forms to Rust unchanged."""
	for action_id, source_type in (
		("mode.draw_solid_wedge", "w1"),
		("mode.draw_hashed_wedge", "h1"),
	):
		window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
		try:
			tip_id, _base_id = _atom_document_object_ids(tab)
			start = tab.view.mapFromScene(PySide6.QtCore.QPointF(150.0, 20.0))
			end = _viewport_point(tab, tip_id)
			_action(window, action_id).trigger()
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
			PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
			qapp.processEvents()
			molecule = tab.current_document_observation().projection.molecules[0]
			bond = molecule.bonds[0]
			assert (len(molecule.atoms), bond.source_type, bond.end.document_object_id) == (3, source_type, tip_id)
		finally:
			_close_window(qapp, window)

		window = ferrum_qt.main_window.MainWindow(object())
		window.show()
		qapp.processEvents()
		try:
			tab = _current_tab(window)
			start = tab.view.mapFromScene(PySide6.QtCore.QPointF(100.0, 100.0))
			end = tab.view.mapFromScene(PySide6.QtCore.QPointF(180.0, 100.0))
			_action(window, action_id).trigger()
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
			PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
			qapp.processEvents()
			molecule = tab.current_document_observation().projection.molecules[0]
			assert (len(molecule.atoms), len(molecule.bonds), molecule.bonds[0].source_type) == (2, 1, source_type)
		finally:
			_close_window(qapp, window)


#============================================
def test_stereo_escape_unchecks_every_direct_bond_action(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Escape and action handoff retire every visible normal or wedge tool state."""
	window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
	try:
		start = _viewport_point(tab, _atom_document_object_ids(tab)[0])
		actions = tuple(_action(window, action_id) for action_id in (
			"mode.draw", "mode.draw_solid_wedge", "mode.draw_hashed_wedge",
		))
		for action in actions:
			action.trigger()
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
			PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
			qapp.processEvents()
			assert not any(candidate.isChecked() for candidate in actions)
	finally:
		_close_window(qapp, window)


#============================================
def test_stereo_refusal_is_modal_and_actionable(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A typed wedge self-loop has the generic visible recovery dialog and no bond."""
	window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
	try:
		start = _viewport_point(tab, _atom_document_object_ids(tab)[0])
		refusal_dialogs: list[tuple[str, str, str]] = []
		observer = _ModalRefusalObserver(refusal_dialogs)
		qapp.installEventFilter(observer)
		try:
			_action(window, "mode.draw_hashed_wedge").trigger()
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
		assert not _action(window, "mode.draw_hashed_wedge").isChecked()
		assert not tab.current_document_observation().projection.molecules[0].bonds
	finally:
		_close_window(qapp, window)

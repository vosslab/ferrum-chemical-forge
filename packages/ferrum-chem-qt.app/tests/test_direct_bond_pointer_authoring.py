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
def _viewport_point(tab: object, atom_id: str) -> PySide6.QtCore.QPoint:
	"""Map one publicly observed atom position to the canvas viewport."""
	atom = next(
		atom for molecule in tab.current_document_observation().projection.molecules
		for atom in molecule.atoms if atom.source_id == atom_id
	)
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(atom.position.x, atom.position.y))


#============================================
def _nearby_non_hit(tab: object, atom_id: str) -> PySide6.QtCore.QPoint:
	"""Find a close viewport point with no direct atom item for native V3 resolution."""
	center = _viewport_point(tab, atom_id)
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
	window = ferrum_qt.main_window.MainWindow(object())
	window.show()
	qapp.processEvents()
	if not _wait_for_open_queue(window, lambda: window.open_file_path(str(path))):
		raise AssertionError("Ferrum did not finish opening the direct-bond document")
	qapp.processEvents()
	return window, _current_tab(window)


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
def test_normal_pointer_direct_hits_create_the_durable_normal_bond(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Exact atom hits become V3 evidence and commit the public durable bond."""
	window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
	try:
		start = _viewport_point(tab, "atom-c")
		end = _viewport_point(tab, "atom-o")
		_action(window, "mode.draw").trigger()
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
		start = _nearby_non_hit(tab, "atom-c")
		end = _viewport_point(tab, "atom-o")
		_action(window, "mode.draw").trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()
		molecule = tab.current_document_observation().projection.molecules[0]
		assert (len(molecule.atoms), molecule.bonds[0].start.source_id) == (2, "atom-c")
	finally:
		_close_window(qapp, window)

	blank = ferrum_qt.main_window.MainWindow(object())
	blank.show()
	qapp.processEvents()
	try:
		blank_tab = _current_tab(blank)
		start = blank_tab.view.mapFromScene(PySide6.QtCore.QPointF(100.0, 100.0))
		end = blank_tab.view.mapFromScene(PySide6.QtCore.QPointF(180.0, 100.0))
		_action(blank, "mode.draw").trigger()
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
		point = _viewport_point(tab, "atom-a")
		action = _action(window, "mode.draw")
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
	"""Escape retires normal authoring while preserving the observed document."""
	window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
	try:
		start = _viewport_point(tab, "atom-c")
		before = tab.current_document_observation().projection.molecules[0]
		action = _action(window, "mode.draw")
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
def test_normal_direct_bond_refusal_is_non_modal_and_actionable(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A typed self-loop refusal is visible in the status surface and leaves no bond."""
	window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
	try:
		start = _viewport_point(tab, "atom-c")
		_action(window, "mode.draw").trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), start)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		qapp.processEvents()
		assert "Draw Bond refused" in window.statusBar().currentMessage()
		assert "Choose a different atom" in window.statusBar().currentMessage()
		assert not _action(window, "mode.draw").isChecked()
		assert not tab.current_document_observation().projection.molecules[0].bonds
	finally:
		_close_window(qapp, window)


#============================================
def _begin_admission_refusal() -> Exception:
	"""Return one native-shaped V3 refusal from the public binding seam."""
	engine = ferrum_qt.ferrum.engine
	refusal = engine.DirectBondAdmissionRefusalV3("direct bond self loop")
	refusal.category = engine.DirectBondAdmissionCategoryV3.self_loop
	refusal.recovery = engine.DirectBondAdmissionRecoveryV3.adjust_endpoint
	return refusal


#============================================
def test_mouse_begin_admission_refusal_is_non_modal_and_retires_draw_action(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		tmp_path: pathlib.Path,
		) -> None:
	"""A typed V3 begin refusal reaches the visible mouse recovery surface."""
	window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
	try:
		monkeypatch.setattr(
			tab, "begin_direct_bond_gesture",
			lambda *_args: (_ for _ in ()).throw(_begin_admission_refusal()),
		)
		action = _action(window, "mode.draw")
		action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			_viewport_point(tab, "atom-c"),
		)
		qapp.processEvents()
		assert "Choose a different atom" in window.statusBar().currentMessage()
		assert not action.isChecked()
	finally:
		_close_window(qapp, window)


#============================================
def test_keyboard_begin_admission_refusal_is_non_modal_and_retires_draw_action(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		tmp_path: pathlib.Path,
		) -> None:
	"""A typed V3 begin refusal reaches the visible keyboard recovery surface."""
	window, tab = _open_window(qapp, tmp_path, _EDITABLE_CDML)
	try:
		monkeypatch.setattr(
			tab, "begin_direct_bond_gesture",
			lambda *_args: (_ for _ in ()).throw(_begin_admission_refusal()),
		)
		action = _action(window, "mode.draw")
		action.trigger()
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return,
		)
		qapp.processEvents()
		assert "Choose a different atom" in window.statusBar().currentMessage()
		assert not action.isChecked()
	finally:
		_close_window(qapp, window)


#============================================
def test_direct_bond_commit_recovery_contract_matches_native_pairs() -> None:
	"""Every native commit recovery pair has one actionable Qt message."""
	engine = ferrum_qt.ferrum.engine
	pairs = (
		(engine.DirectBondCommitCategoryV1.foreign_session,
			engine.DirectBondCommitRecoveryV1.refresh_and_restart, "Refresh the Rust view"),
		(engine.DirectBondCommitCategoryV1.replayed_receipt,
			engine.DirectBondCommitRecoveryV1.refresh_and_restart, "Refresh the Rust view"),
		(engine.DirectBondCommitCategoryV1.unrenderable_candidate,
			engine.DirectBondCommitRecoveryV1.change_presentation, "supported bond appearance"),
		(engine.DirectBondCommitCategoryV1.stale_revision,
			engine.DirectBondCommitRecoveryV1.refresh_and_restart, "Refresh the Rust view"),
		(engine.DirectBondCommitCategoryV1.stale_digest,
			engine.DirectBondCommitRecoveryV1.refresh_and_restart, "Refresh the Rust view"),
		(engine.DirectBondCommitCategoryV1.identity_allocation_failed,
			engine.DirectBondCommitRecoveryV1.report_conflict, "document conflict"),
		(engine.DirectBondCommitCategoryV1.provisional_token_unavailable,
			engine.DirectBondCommitRecoveryV1.report_conflict, "document conflict"),
		(engine.DirectBondCommitCategoryV1.candidate_application_failed,
			engine.DirectBondCommitRecoveryV1.refresh_and_restart, "could not apply the bond"),
		(engine.DirectBondCommitCategoryV1.revision_exhausted,
			engine.DirectBondCommitRecoveryV1.report_conflict, "document conflict"),
	)
	for category, recovery, action in pairs:
		message = ferrum_qt.ferrum.line_tool_completion.direct_bond_commit_recovery_message(
			category, recovery,
		)
		assert message is not None and action in message
	message = ferrum_qt.ferrum.line_tool_completion.direct_bond_commit_recovery_message(
		engine.DirectBondCommitCategoryV1.identity_allocation_failed,
		engine.DirectBondCommitRecoveryV1.refresh_and_restart,
	)
	assert message is None

#!/usr/bin/env python3
"""Exercise complete attached-cyclohexane admission through the real Qt host."""

# Standard Library
import collections.abc
import json
import pathlib
import sys
import tempfile

# local E2E modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_CARBONYL_CDML = """<cdml xmlns='urn:ferrum:cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='300' y='360'/></atom>
    <atom id='atom-o' name='O'><point x='520' y='360'/></atom>
    <bond id='carbonyl' start='atom-c' end='atom-o' type='n1'/>
  </molecule>
</cdml>"""


#============================================
def _wait_for_open_queue(window: ferrum_qt.main_window.MainWindow,
		start: collections.abc.Callable[[], object]) -> bool:
	"""Run one ordinary Open request until the public completion signal fires."""
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer(window)
	timeout.setSingleShot(True)
	outcome: bool | None = None

	def finish(success: bool) -> None:
		"""Capture the first observable terminal Open result."""
		nonlocal outcome
		if outcome is None:
			outcome = success
			loop.quit()

	window.local_document_open_queue_drained.connect(finish)
	timeout.timeout.connect(lambda: finish(False))
	try:
		start()
		if outcome is None:
			timeout.start(10000)
			loop.exec()
	finally:
		timeout.stop()
		window.local_document_open_queue_drained.disconnect(finish)
		timeout.timeout.disconnect()
	return outcome is True


#============================================
def _open_window(qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path) -> tuple[ferrum_qt.main_window.MainWindow, object]:
	"""Open the exact carbon--oxygen host through the product's public Open route."""
	path = tmp_path / "attached_cyclohexane.cdml"
	path.write_text(_CARBONYL_CDML, encoding="ascii")
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	window.show()
	qapp.processEvents()
	if not _wait_for_open_queue(window, lambda: window.open_file_path(str(path))):
		raise AssertionError("Ferrum did not open the attached-cyclohexane fixture")
	tabs = window.centralWidget()
	if not isinstance(tabs, PySide6.QtWidgets.QTabWidget) or tabs.currentWidget() is None:
		raise AssertionError("Ferrum did not expose the opened document tab")
	return window, tabs.currentWidget()


#============================================
def _close_window(qapp: PySide6.QtWidgets.QApplication,
		window: ferrum_qt.main_window.MainWindow) -> None:
	"""Discard the test document through its ordinary native tab lifecycle."""
	window.cancel_active_pointer_authoring()
	for tab in tuple(window._native_tabs_by_page.values()):
		index = window._tab_widget.indexOf(tab)
		assert window._close_native_tab_at(
			index, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		) is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
	window.close()
	window.deleteLater()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.DeferredDelete,
	)
	qapp.processEvents()


#============================================
def _atom_id(tab: object, element: str) -> str:
	"""Return the one durable atom identity for the explicitly authored element."""
	atoms = tab.current_document_observation().projection.molecules[0].atoms
	matches = tuple(atom.document_object_id for atom in atoms if atom.element == element)
	assert len(matches) == 1
	return matches[0]


#============================================
def _viewport_point(tab: object, document_object_id: str) -> PySide6.QtCore.QPoint:
	"""Map one Rust-issued durable atom coordinate into the visible canvas."""
	atom = next(
		atom for molecule in tab.current_document_observation().projection.molecules
		for atom in molecule.atoms if atom.document_object_id == document_object_id
	)
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(atom.position.x, atom.position.y))


#============================================
def _assert_no_attached_preview(window: ferrum_qt.main_window.MainWindow) -> None:
	"""Require an armed repeat gesture to retain no candidate-owned preview graphics."""
	intent = window._line_gesture_intent
	if intent is None:
		return
	assert intent.attached_cyclohexane_pending is None
	assert intent.preview is None


#============================================
def _require_rightward_refusal_before_document_or_scene_mutation(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The candidate crossing O is a typed admission refusal, never a partial commit."""
	window, tab = _open_window(qapp, tmp_path)
	try:
		carbon_id = _atom_id(tab, "C")
		before_snapshot = tab.current_snapshot
		before_scene = tab.view.scene()
		before_projection = tab._controller.projection
		assert before_projection is not None
		assert before_projection.issues == ()
		_assert_no_attached_preview(window)

		try:
			tab.begin_attached_cyclohexane(
				carbon_id, PySide6.QtCore.QPointF(380.0, 360.0),
			)
		except engine.AttachedCyclohexaneAttachmentError as exc:
			assert exc.category == engine.AttachedCyclohexaneCategoryV1.renderer_admission
		else:
			raise AssertionError("rightward attachment bypassed renderer admission")
		after_snapshot = tab.current_snapshot
		assert (after_snapshot.revision, after_snapshot.digest) == (
			before_snapshot.revision, before_snapshot.digest,
		)
		assert tab.view.scene() is before_scene
		assert tab._controller.projection is before_projection
		assert before_projection.issues == ()
		_assert_no_attached_preview(window)
	finally:
		_close_window(qapp, window)


#============================================
def _require_leftward_attachment_to_commit_a_complete_visible_host_molecule(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A normal leftward drag keeps all seven authored bonds visible and issue-free."""
	window, tab = _open_window(qapp, tmp_path)
	try:
		carbon_id = _atom_id(tab, "C")
		start = _viewport_point(tab, carbon_id)
		end = start - PySide6.QtCore.QPoint(80, 0)
		action = window._action_registry.get_qt_action("draw.ring.cyclohexane.attach")
		assert action is not None
		action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()

		molecule = tab.current_document_observation().projection.molecules[0]
		render_projection = tab._controller.projection
		assert render_projection is not None
		bond_ids = {bond.document_object_id for bond in molecule.bonds}
		rendered_bond_ids = {
			target.document_object_id for target in render_projection.item_targets.values()
			if target.document_object_id in bond_ids
		}
		assert len(molecule.bonds) == 7
		assert rendered_bond_ids == bond_ids
		assert render_projection.issues == ()
		_assert_no_attached_preview(window)
	finally:
		_close_window(qapp, window)


#============================================
def main() -> int:
	"""Run both refusal and successful real-host attachment workflows."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	with tempfile.TemporaryDirectory(prefix="ferrum-attached-cyclohexane-") as temporary:
		directory = pathlib.Path(temporary)
		_require_rightward_refusal_before_document_or_scene_mutation(app, directory)
		_require_leftward_attachment_to_commit_a_complete_visible_host_molecule(
			app, directory,
		)
	print(json.dumps({"status": "ok"}))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except (AssertionError, OSError, RuntimeError) as exc:
		print(f"e2e_attached_cyclohexane_renderer_admission: {exc}", file=sys.stderr)
		raise SystemExit(1)

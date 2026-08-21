"""Behavior coverage for the public Ferrum window seam."""

# Standard Library
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.main_window
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.window_refusals


_EDITABLE_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  </molecule>
</cdml>"""


#============================================
def test_draw_bond_start_uses_the_dedicated_implicit_picker_and_refuses_ambiguity(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The Draw Bond press route delegates only origin selection to its picker."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "draw-bond-start-picker.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	implicit_atom = tab.current_document_observation().projection.molecules[0].atoms[0]
	point = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0)) + PySide6.QtCore.QPoint(6, 0)
	picker_calls: list[PySide6.QtCore.QPoint] = []
	begin_calls: list[str] = []
	try:
		picker = tab.durable_direct_bond_start_atom_at_viewport_point
		monkeypatch.setattr(
			tab, "durable_direct_bond_start_atom_at_viewport_point",
			lambda candidate: picker_calls.append(candidate) or picker(candidate),
		)
		monkeypatch.setattr(
			tab, "begin_direct_bond_gesture",
			lambda atom_id, *_args: begin_calls.append(atom_id) or object(),
		)
		_click_visible_menu_action(window, "Draw Bond", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
		)
		qapp.processEvents()
		assert picker_calls == [point] and begin_calls == [implicit_atom.source_id]
		window._cancel_line_gesture()
		monkeypatch.setattr(
			tab, "durable_direct_bond_start_atom_at_viewport_point",
			lambda _candidate: None,
		)
		_click_visible_menu_action(window, "Draw Bond", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
		)
		qapp.processEvents()
		assert begin_calls == [implicit_atom.source_id]
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_change_element_action_rechecks_selection_and_uses_the_bounded_dialog(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The one owned Edit action submits only an eligible dialog acceptance."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "change-element.cdml",
	)
	window._register_native_tab(tab, activate=True)
	qapp.processEvents()
	prior = tab.current_snapshot
	assert not window._change_element_action.isEnabled()
	window._change_element_action.trigger()
	assert tab.current_snapshot is prior
	tab.select_atom("atom-c")
	window._refresh_actions()
	assert window._change_element_action.isEnabled()
	calls: list[tuple[str, str]] = []
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText",
		lambda _parent, title, label: (calls.append((title, label)) and ("N", True)),
	)
	window._change_element_action.trigger()
	assert calls == [(window.tr("Change Atom Element"), window.tr("Element symbol:"))]
	assert tab.selected_atom_projection().element == "N"
	window.close()
	window.deleteLater()


#============================================
def test_change_element_dialog_cancel_and_refusal_preserve_tab_truth(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Cancel and Rust refusal use the window route without partial document edits."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "change-element-refusal.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	monkeypatch.setattr(PySide6.QtWidgets.QInputDialog, "getText", lambda *_args: ("", False))
	window._change_element_action.trigger()
	assert tab.current_snapshot is prior
	refusals = []
	monkeypatch.setattr(window, "_show_edit_refusal", lambda refusal: refusals.append(refusal))
	monkeypatch.setattr(PySide6.QtWidgets.QInputDialog, "getText", lambda *_args: ("Xx", True))
	window._change_element_action.trigger()
	assert tab.current_snapshot is prior and len(refusals) == 1
	window.close()
	window.deleteLater()


#============================================
def test_change_element_projection_failure_recovers_one_accepted_rust_edit(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""One failed install refreshes once without resubmitting the Rust change."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "change-element-pending.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	messages = []
	replace = tab._controller.replace
	attempts = []
	def fail_once(observation: object, latch: object) -> bool:
		"""Reject only the original install so recovery must reuse its result."""
		attempts.append(observation)
		if len(attempts) == 1:
			return False
		return replace(observation, latch)
	monkeypatch.setattr(tab._controller, "replace", fail_once)
	monkeypatch.setattr(PySide6.QtWidgets.QInputDialog, "getText", lambda *_args: ("N", True))
	monkeypatch.setattr(window, "_unavailable_edit_refusal", lambda message: messages.append(message))
	monkeypatch.setattr(window, "_show_edit_refusal", lambda _request: None)
	window._change_element_action.trigger()
	selected = tab._controller.projection.selected_durable_targets()
	assert tab.current_snapshot.revision == prior.revision + 1 and not tab.requires_refresh
	assert len(attempts) == 2 and selected[0].kind == "atom" and selected[0].identifier == "atom-c"
	assert len(messages) == 1 and "refreshed the authoritative Rust display" in messages[0]
	window.close()
	window.deleteLater()

_BOND_CDML = """<cdml version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  <atom id='atom-o' name='O'><point x='40' y='20'/></atom>
</molecule></cdml>"""

_MIXED_ROOT_CDML = """<cdml version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='13' y='17'/></atom>
</molecule><plus id='plus-1'><point x='73' y='51'/></plus></cdml>"""

_AUTHORED_COORDINATE_TOLERANCE = 0.001 * 72.0 / 2.54


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide an offscreen application without importing legacy fixtures."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _atom_viewport_point(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		atom_id: str,
		) -> PySide6.QtCore.QPoint:
	"""Return a stable interior viewport point for one projected durable atom."""
	item = tab._controller.projection.durable_items[("atom", atom_id)]
	shape = item.shape()
	bounds = shape.boundingRect()
	for x_step in range(1, 10):
		for y_step in range(1, 10):
			point = PySide6.QtCore.QPointF(
				bounds.left() + bounds.width() * x_step / 10.0,
				bounds.top() + bounds.height() * y_step / 10.0,
			)
			if shape.contains(point):
				return tab.view.mapFromScene(item.mapToScene(point))
	raise AssertionError("projected atom has no interior hit-test point")


#============================================
def _empty_viewport_point(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> PySide6.QtCore.QPoint:
	"""Return one visible viewport point that does not hit a durable atom."""
	rect = tab.view.viewport().rect().adjusted(12, 12, -12, -12)
	for x_step in range(1, 10):
		for y_step in range(1, 10):
			point = PySide6.QtCore.QPoint(
				rect.left() + rect.width() * x_step // 10,
				rect.top() + rect.height() * y_step // 10,
			)
			if tab.durable_atom_at_viewport_point(point) is None:
				return point
	raise AssertionError("Ferrum viewport has no empty hit-test point")


#============================================
def _select_mixed_complete_roots(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[tuple[str, str], ...]:
	"""Select one complete molecule and one independent presentation root."""
	plus_key = next(
		key for key in tab._controller.projection.durable_items if key[0] == "plus"
	)
	tab._controller.projection.select_durable((
		("atom", "atom-c"), plus_key,
	))
	return tab.selected_top_level_transform_targets()[1]


#============================================
def _mixed_root_positions(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[tuple[float, float], tuple[float, float]]:
	"""Return authoritative molecule and presentation coordinates after a move."""
	projection = tab.current_document_observation().projection
	atom = projection.molecules[0].atoms[0].position
	plus = projection.presentation_stack.roots[0].plus.anchor
	return (atom.x, atom.y), (plus.x, plus.y)


#============================================
def _click_visible_menu_action(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Activate one labelled command through its visible top-level menu item."""
	menu_bar = window.menuBar()
	for menu_action in menu_bar.actions():
		menu = menu_action.menu()
		if menu is None:
			continue
		for candidate in menu.actions():
			if candidate.text().replace("&", "") != label:
				continue
			PySide6.QtTest.QTest.mouseClick(
				menu_bar, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu_bar.actionGeometry(menu_action).center(),
			)
			qapp.processEvents()
			if not menu.isVisible():
				raise AssertionError(f"Visible menu did not open for {label!r}")
			PySide6.QtTest.QTest.mouseClick(
				menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu.actionGeometry(candidate).center(),
			)
			qapp.processEvents()
			return
	raise AssertionError(f"No visible menu action is labelled {label!r}")


#============================================
def _restore_drawing_parameters(
		window: ferrum_qt.main_window.MainWindow, snapshot: object,
		) -> None:
	"""Restore application-owned choices after a behavior test changes them."""
	window._drawing_parameters.set_element(snapshot.element)
	window._drawing_parameters.set_order_name(snapshot.order_name)
	window._drawing_parameters.set_presentation_name(snapshot.presentation_name)


#============================================
def test_public_native_window_routes_cdml_to_rust_without_a_legacy_session(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The standalone window reaches its Ferrum controller, not a fallback base."""
	del qapp
	source = tmp_path / "source.cdml"
	source.write_text("<cdml/>", encoding="utf-8")
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	loop = PySide6.QtCore.QEventLoop()
	outcomes = []

	def finish(success: bool) -> None:
		"""Capture the complete admission result and stop the local event loop."""
		outcomes.append(success)
		loop.quit()

	window.local_document_open_queue_drained.connect(finish)
	try:
		assert window.open_file_path(str(source))
		loop.exec()
		tab_widget = window.centralWidget()
		assert outcomes == [True] and isinstance(tab_widget, PySide6.QtWidgets.QTabWidget)
		tab = tab_widget.currentWidget()
		assert isinstance(
			tab, ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		)
		assert tab.file_path == source and not tab.current_snapshot.is_dirty
	finally:
		window.close()


#============================================
def test_smarts_action_follows_real_window_tab_readiness(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch) -> None:
	"""The installed SMARTS command follows no-tab, ready, pending, and disposed states."""
	del qapp
	window = ferrum_qt.main_window.MainWindow(object())
	action = window._smarts_query_controller._action
	assert len(window.findChildren(
		PySide6.QtWidgets.QDockWidget, "smarts-query-dock",
	)) == 1
	assert window._smarts_query_action is action
	chemistry_menu = next(
		menu for menu in window.menuBar().findChildren(PySide6.QtWidgets.QMenu)
		if menu.title() == window.tr("Chemistry")
	)
	assert len([
		candidate for candidate in chemistry_menu.actions()
		if candidate is action
	]) == 1
	tab = type("Tab", (), {
		"_disposed": False,
		"requires_refresh": False,
		"_controller": type("Controller", (), {"projection": object()})(),
	})()
	active_tab: list[object | None] = [None]
	monkeypatch.setattr(window, "_active_native_tab", lambda: active_tab[0])
	monkeypatch.setattr(
		ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		"_refresh_actions", lambda *_args: None,
	)
	monkeypatch.setattr(
		ferrum_qt.ferrum.window_shared_seams,
		"refresh_shared_window_seams", lambda _window: None,
	)
	try:
		window._refresh_actions()
		assert not action.isEnabled()
		active_tab[0] = tab
		window._refresh_actions()
		assert action.isEnabled()
		tab.requires_refresh = True
		window._refresh_actions()
		assert not action.isEnabled()
		tab.requires_refresh = False
		tab._disposed = True
		window._refresh_actions()
		assert not action.isEnabled()
	finally:
		window.close()


#============================================
def test_real_add_atom_action_retires_smarts_capture_before_its_handler(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The real MainWindow Add Atom registration retires capture before its handler."""
	class _CaptureHandoffWindow(ferrum_qt.ferrum.main_window.FerrumNativeMainWindow):
		"""Observe the actual Add Atom handler selected while actions are constructed."""

		def __init__(self) -> None:
			"""Build the real window before supplying its test-only active viewport."""
			self._handoff_active_tab: object | None = None
			self.handler_capture_states: list[bool] = []
			super().__init__()

		def _active_native_tab(self) -> object | None:
			"""Return the viewport-only tab used by this signal-ordering regression."""
			return self._handoff_active_tab

		def _on_toggle_add_atom(self, _checked: bool) -> None:
			"""Observe capture exactly where the real QAction dispatch enters its handler."""
			capture = self._smarts_query_controller._selected_capture
			self.handler_capture_states.append(capture.is_armed_v1())

	class _CaptureTab:
		"""Supply only the live viewport contract needed to arm selected-root capture."""

		def __init__(self, parent: PySide6.QtWidgets.QWidget) -> None:
			"""Create the canvas viewport owned temporarily by SMARTS capture."""
			self._disposed = False
			self.requires_refresh = False
			self.view = PySide6.QtWidgets.QGraphicsView(parent)

	window = _CaptureHandoffWindow()
	tab = _CaptureTab(window)
	window._handoff_active_tab = tab
	window._add_atom_action.setEnabled(True)
	window.show()
	qapp.processEvents()
	capture = window._smarts_query_controller._selected_capture
	try:
		capture.begin()
		assert capture.is_armed_v1()
		window._add_atom_action.trigger()
		assert window.handler_capture_states == [False]
		assert not capture.is_armed_v1()
	finally:
		window._handoff_active_tab = None
		window.close()
		window.deleteLater()


#============================================
def test_clean_pending_undo_requires_refresh_before_tab_or_window_close(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A clean Rust baseline with a stale visible scene cannot be discarded."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("atom-c")
	tab.change_selected_atom_element("N")
	replace = tab._controller.replace
	monkeypatch.setattr(tab._controller, "replace", lambda _observation, _latch: False)
	document_tab_module = ferrum_qt.ferrum.document_tab
	error_type = document_tab_module.FerrumNativeDocumentTabMutationPresentationError
	with pytest.raises(error_type):
		tab.undo()
	assert tab.requires_refresh and not tab.is_dirty
	warnings = []
	monkeypatch.setattr(
		window, "_show_edit_refusal",
		lambda request: warnings.append(request),
	)
	window._close_tab_at(window._tab_widget.indexOf(tab))
	close_event = PySide6.QtGui.QCloseEvent()
	window.closeEvent(close_event)
	assert window._tab_widget.indexOf(tab) >= 0 and not close_event.isAccepted()
	assert [request.outcome.value for request in warnings] == ["busy_close", "busy_close"]
	assert all(request.context.value == "close_document" for request in warnings)
	monkeypatch.setattr(tab._controller, "replace", replace)
	assert tab.refresh_authoritative() and not tab.requires_refresh and not tab.is_dirty
	window._close_tab_at(window._tab_widget.indexOf(tab))
	assert window._tab_widget.indexOf(tab) < 0 and tab._disposed
	window.deleteLater()


#============================================
def test_history_actions_mirror_active_tab_rust_availability(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Only the active tab's Rust history facts enable the guarded Edit actions."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	history_tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "history-actions.cdml",
	)
	fresh_tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "fresh-history-actions.cdml",
	)
	window._register_native_tab(history_tab, activate=True)
	try:
		assert not window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		history_tab.select_atoms(("atom-c", "atom-o"))
		window._add_single_bond_action.trigger()
		assert window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		window._register_native_tab(fresh_tab, activate=True)
		assert not window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		window._tab_widget.setCurrentWidget(history_tab)
		qapp.processEvents()
		assert window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		with monkeypatch.context() as patch:
			patch.setattr(type(history_tab), "requires_refresh", property(lambda _tab: True))
			window._refresh_actions()
			assert not window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		assert history_tab.can_undo() and not history_tab.can_redo()
		with monkeypatch.context() as patch:
			patch.setattr(window, "_molecule_import_busy", lambda: True)
			window._refresh_actions()
			assert not window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		assert history_tab.can_undo() and not history_tab.can_redo()
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_add_single_bond_action_connects_exact_selected_atoms(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The public edit action commits, selects, and saves one Rust-owned bond."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atoms(("atom-c", "atom-o"))
	window._add_single_bond_action.trigger()
	selected = tab._controller.projection.selected_durable_targets()
	assert len(selected) == 1 and selected[0].kind == "bond"
	assert 'type="n1" start="atom-c" end="atom-o"' in tab.current_snapshot.cdml
	assert "Added one Ferrum single bond." in window.statusBar().currentMessage()
	tab.save_atomic(tmp_path / "bonded.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_delete_selected_atom_action_removes_incident_bond_through_rust(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The public delete action commits one undoable Rust topology change."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "delete-atom.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atoms(("atom-c", "atom-o"))
	tab.add_single_bond_between_selected_atoms()
	tab.select_atom("atom-o")
	window._delete_atom_action.trigger()
	molecule = tab._document_observation.projection.molecules[0]
	assert tuple(atom.source_id for atom in molecule.atoms) == ("atom-c",)
	assert not molecule.bonds
	assert "Deleted one Ferrum atom" in window.statusBar().currentMessage()
	tab.undo()
	restored = tab._document_observation.projection.molecules[0]
	assert len(restored.atoms) == 2 and len(restored.bonds) == 1
	tab.redo()
	tab.save_atomic(tmp_path / "deleted-atom.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_delete_selected_bond_action_preserves_atoms_through_rust(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The public delete action removes one selected bond and no endpoint atoms."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "delete-bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atoms(("atom-c", "atom-o"))
	created = tab.add_single_bond_between_selected_atoms()
	bond_id = created.observation.projection.molecules[0].bonds[0].source_id
	tab.select_bond(bond_id)
	window._delete_bond_action.trigger()
	molecule = tab._document_observation.projection.molecules[0]
	assert len(molecule.atoms) == 2 and not molecule.bonds
	assert "Deleted one Ferrum bond" in window.statusBar().currentMessage()
	tab.undo()
	assert len(tab._document_observation.projection.molecules[0].bonds) == 1
	tab.redo()
	tab.save_atomic(tmp_path / "deleted-bond.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_change_bond_order_action_uses_the_closed_rust_enum(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The public action changes one selected bond without interpreting CDML in Qt."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond-order.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atoms(("atom-c", "atom-o"))
	created = tab.add_single_bond_between_selected_atoms()
	bond_id = created.observation.projection.molecules[0].bonds[0].source_id
	tab.select_bond(bond_id)
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getItem",
		lambda *_args: ("Double", True),
	)
	window._change_bond_order_action.trigger()
	bond = tab._document_observation.projection.molecules[0].bonds[0]
	assert bond.source_type == "n2"
	assert "double" in window.statusBar().currentMessage()
	tab.save_atomic(tmp_path / "double-bond-action.cdml")
	window.close()
	window.deleteLater()


#============================================

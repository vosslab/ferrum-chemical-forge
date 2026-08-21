"""Visible behavior for the ordinary Ferrum cyclohexane-ring action."""

import dataclasses
import os
import pathlib
import types

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

import ferrum_qt.main_window
import ferrum_qt.ferrum.document_tab


def _click_visible_menu_action(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Activate a labelled command through the visible menu route."""
	for menu_action in window.menuBar().actions():
		menu = menu_action.menu()
		if menu is None:
			continue
		for candidate in menu.actions():
			if candidate.text().replace("&", "") != label:
				continue
			PySide6.QtTest.QTest.mouseClick(
				window.menuBar(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				window.menuBar().actionGeometry(menu_action).center(),
			)
			qapp.processEvents()
			PySide6.QtTest.QTest.mouseClick(
				menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu.actionGeometry(candidate).center(),
			)
			qapp.processEvents()
			return
	raise AssertionError(f"No visible menu action is labelled {label!r}")


def _ring_centre(molecule: object) -> tuple[float, float]:
	"""Return the centre implied by Rust's ordinary authored ring vertices."""
	return (
		sum(atom.position.x for atom in molecule.atoms) / len(molecule.atoms),
		sum(atom.position.y for atom in molecule.atoms) / len(molecule.atoms),
	)


def _arm_attached_cyclohexane(
		window: PySide6.QtWidgets.QMainWindow,
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		qapp: PySide6.QtWidgets.QApplication,
		) -> object:
	"""Start one real C6 gesture and return its opaque native receipt."""
	anchor = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
	_click_visible_menu_action(window, "Attach Cyclohexane Ring", qapp)
	PySide6.QtTest.QTest.mousePress(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor,
	)
	PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), anchor + PySide6.QtCore.QPoint(80, 0))
	qapp.processEvents()
	intent = window._line_gesture_intent
	assert intent is not None and intent.attached_cyclohexane_pending is not None
	assert intent.preview is not None
	return intent.attached_cyclohexane_pending


#============================================
def test_attach_cyclohexane_popup_row_keeps_the_shared_action_armed(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The real menu route arms C6 only after its transient popup has closed."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='m'><atom id='a' name='C'>"
		"<point x='10' y='20'/></atom></molecule></cdml>", "popup-c6.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		_click_visible_menu_action(window, "Attach Cyclohexane Ring", qapp)
		assert window._attach_cyclohexane_ring_action.isChecked()
		assert window._mode_manager.active_mode_id is None
		assert window._line_gesture_intent is not None
		assert window._line_gesture_intent.tool.value == "attach_cyclohexane_ring"
	finally:
		window.close()
		window.deleteLater()
def test_implicit_atom_start_pickers_are_bounded_and_preserve_rendered_hits(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""C6 and Draw Bond share a narrow implicit-start picker, not a canvas picker."""
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='m'><atom id='a' name='C'>"
		"<point x='10' y='20'/></atom></molecule></cdml>", "implicit-anchor.cdml",
	)
	try:
		tab.show()
		qapp.processEvents()
		anchor = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
		implicit_atom = tab.current_document_observation().projection.molecules[0].atoms[0]
		assert implicit_atom.id != implicit_atom.source_id
		assert tab.durable_atom_at_viewport_point(anchor) is None
		assert tab.durable_attachment_atom_at_viewport_point(
			anchor + PySide6.QtCore.QPoint(6, 0),
		) == implicit_atom.id
		assert tab.durable_direct_bond_start_atom_at_viewport_point(
			anchor + PySide6.QtCore.QPoint(6, 0),
		) == implicit_atom.source_id
		# The established rendered-item hit remains authoritative even where the
		# C6-only projection fallback would otherwise find the nearby carbon.
		tab.durable_atom_at_viewport_point = types.MethodType(
			lambda _self, _point: "rendered-hit", tab,
		)
		assert tab.durable_attachment_atom_at_viewport_point(
			anchor + PySide6.QtCore.QPoint(6, 0),
		) == "rendered-hit"
		assert tab.durable_direct_bond_start_atom_at_viewport_point(
			anchor + PySide6.QtCore.QPoint(6, 0),
		) == "rendered-hit"
		assert tab.durable_attachment_atom_at_viewport_point(
			anchor + PySide6.QtCore.QPoint(7, 0),
		) is None
		assert tab.durable_direct_bond_start_atom_at_viewport_point(
			anchor + PySide6.QtCore.QPoint(7, 0),
		) is None
		original_observation = tab._document_observation
		tab.durable_atom_at_viewport_point = types.MethodType(
			lambda _self, _point: None, tab,
		)
		tab._document_observation = types.SimpleNamespace(
			projection=types.SimpleNamespace(molecules=(types.SimpleNamespace(atoms=(
				types.SimpleNamespace(
					id="projection-object", source_id=None,
					position=types.SimpleNamespace(x=10.0, y=20.0),
				),
			),),)),
		)
		assert tab.durable_attachment_atom_at_viewport_point(anchor) is None
		assert tab.durable_direct_bond_start_atom_at_viewport_point(anchor) is None
		tab._document_observation = original_observation
	finally:
		tab.close()
		tab.deleteLater()


def test_attach_cyclohexane_implicit_atom_picker_refuses_a_nearest_tie(
	qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Equal viewport-distance implicit targets refuse rather than guess an ID."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='m'><atom id='a' name='C'><point x='10' y='20'/></atom>"
		"<atom id='b' name='C'><point x='10' y='20'/></atom></molecule></cdml>",
		"ambiguous-anchor.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		anchor = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
		assert tab.durable_atom_at_viewport_point(anchor) is None
		assert tab.durable_attachment_atom_at_viewport_point(anchor) is None
		assert tab.durable_direct_bond_start_atom_at_viewport_point(anchor) is None
		_click_visible_menu_action(window, "Attach Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor,
		)
		qapp.processEvents()
		intent = window._line_gesture_intent
		assert intent is not None
		assert intent.start_atom_id is None
		assert intent.attached_cyclohexane_pending is None
	finally:
		window.close()
		window.deleteLater()
		tab.close()
		tab.deleteLater()


def _count_attached_cyclohexane_cancellations(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		refuse_first: bool = False,
		) -> list[object]:
	"""Record actual tab bridge calls while optionally refusing the first one."""
	calls: list[object] = []
	original_cancel = tab.cancel_attached_cyclohexane

	def cancel(self: object, receipt: object) -> None:
		calls.append(receipt)
		if refuse_first and len(calls) == 1:
			raise RuntimeError("synthetic native retirement refusal")
		original_cancel(receipt)

	tab.cancel_attached_cyclohexane = types.MethodType(cancel, tab)
	return calls


def test_insert_cyclohexane_ring_uses_the_shared_authored_centre_and_selects_rust_atoms(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The ordinary action commits the snapped and raw centres Rust receives."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml/>", "ring.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		snapped_click = PySide6.QtCore.QPoint(143, 91)
		expected_snapped = tab.view.snap_authored_scene_point(
			tab.view.mapToScene(snapped_click),
		)
		_click_visible_menu_action(window, "Insert Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, snapped_click,
		)
		snapped_ring = tab.current_document_observation().projection.molecules[0]
		selected = tab._controller.projection.selected_durable_targets()

		assert _ring_centre(snapped_ring) == pytest.approx(
			(expected_snapped.x(), expected_snapped.y()),
		)
		assert selected and all(
			target.kind == "atom" and target.identifier in {
				atom.source_id for atom in snapped_ring.atoms
			}
			for target in selected
		)

		tab.view.set_hex_grid_snap_enabled(False)
		raw_click = PySide6.QtCore.QPoint(318, 207)
		expected_raw = tab.view.mapToScene(raw_click)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, raw_click,
		)
		raw_ring = tab.current_document_observation().projection.molecules[1]
		assert _ring_centre(raw_ring) == pytest.approx((expected_raw.x(), expected_raw.y()))
	finally:
		window.close()
		window.deleteLater()


def test_insert_cyclohexane_ring_refuses_an_occupied_atom_without_mutation(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An occupied click preserves the authoritative document and prior selection."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='m'><atom id='a' name='C'>"
		"<point x='10' y='20'/></atom></molecule></cdml>",
		"occupied-ring.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		tab._controller.projection.select_durable((("atom", "a"),))
		before_snapshot = tab.current_snapshot
		before_selection = tab._controller.projection.selected_durable_targets()
		occupied = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
		_click_visible_menu_action(window, "Insert Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, occupied,
		)

		assert tab.current_snapshot == before_snapshot
		assert tab._controller.projection.selected_durable_targets() == before_selection
	finally:
		window.close()
		window.deleteLater()


def test_attach_cyclohexane_ring_drag_commits_and_escape_retires_pending_receipt(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The attach command is independent, paint-only until its one release commit."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='m'><atom id='a' name='C'>"
		"<point x='10' y='20'/></atom></molecule></cdml>", "attach-ring.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		anchor = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
		before = tab.current_snapshot
		commit_calls: list[object] = []
		original_commit = tab.commit_attached_cyclohexane

		def commit(self: object, receipt: object) -> object:
			commit_calls.append(receipt)
			return original_commit(receipt)

		tab.commit_attached_cyclohexane = types.MethodType(commit, tab)
		_click_visible_menu_action(window, "Attach Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), anchor + PySide6.QtCore.QPoint(80, 0))
		qapp.processEvents()
		assert window._line_gesture_intent is not None
		assert window._line_gesture_intent.attached_cyclohexane_pending is not None
		assert tab.current_snapshot == before
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		qapp.processEvents()
		assert window._line_gesture_intent is None
		assert tab.current_snapshot == before

		_click_visible_menu_action(window, "Attach Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), anchor + PySide6.QtCore.QPoint(80, 0))
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, anchor + PySide6.QtCore.QPoint(80, 0),
		)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before.revision + 1
		assert len(commit_calls) == 1
		assert len(tab.current_document_observation().projection.molecules[0].atoms) == 6
		undone = tab.undo().observation.snapshot
		assert undone.cdml == before.cdml
		assert not tab.is_dirty
	finally:
		window.close()
		window.deleteLater()


def test_non_attachment_release_still_requires_a_start_scene(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A malformed ordinary-ring release remains a non-mutating refusal."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab("<cdml/>", "guard-ring.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		point = PySide6.QtCore.QPoint(143, 91)
		_click_visible_menu_action(window, "Insert Cyclohexane Ring", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
		)
		qapp.processEvents()
		intent = window._line_gesture_intent
		assert intent is not None and intent.start_scene is not None
		before = tab.current_snapshot
		window._line_gesture_intent = dataclasses.replace(intent, start_scene=None)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
		)
		qapp.processEvents()
		assert tab.current_snapshot == before
	finally:
		window.close()
		window.deleteLater()


def test_attach_cyclohexane_release_without_anchor_retires_pending_without_commit(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A malformed C6 intent loses its receipt rather than committing unanchored."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='m'><atom id='a' name='C'>"
		"<point x='10' y='20'/></atom></molecule></cdml>", "missing-anchor.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		pending = _arm_attached_cyclohexane(window, tab, qapp)
		before = tab.current_snapshot
		commits: list[object] = []
		refusals: list[object] = []
		original_commit = tab.commit_attached_cyclohexane

		def commit(self: object, receipt: object) -> object:
			commits.append(receipt)
			return original_commit(receipt)

		def show_refusal(self: object, refusal: object) -> None:
			refusals.append(refusal)

		tab.commit_attached_cyclohexane = types.MethodType(commit, tab)
		window._show_edit_refusal = types.MethodType(show_refusal, window)
		cancellations = _count_attached_cyclohexane_cancellations(tab)
		intent = window._line_gesture_intent
		assert intent is not None
		window._line_gesture_intent = dataclasses.replace(intent, start_atom_id=None)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			tab.view.mapFromScene(PySide6.QtCore.QPointF(90.0, 20.0)),
		)
		qapp.processEvents()
		assert window._line_gesture_intent is None
		assert cancellations == [pending]
		assert not commits and len(refusals) == 1
		assert tab.current_snapshot == before
	finally:
		window.close()
		window.deleteLater()


def test_attach_cyclohexane_cancellation_blocks_until_the_retained_receipt_retires(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A native retirement refusal fences the receipt until a later retry succeeds."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='m'><atom id='a' name='C'>"
		"<point x='10' y='20'/></atom></molecule></cdml>", "cancel-ring.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		pending = _arm_attached_cyclohexane(window, tab, qapp)
		before = tab.current_snapshot
		calls = _count_attached_cyclohexane_cancellations(tab, refuse_first=True)
		assert not window._cancel_line_gesture()
		blocked = window._line_gesture_intent
		assert blocked is not None
		assert blocked.attached_cyclohexane_pending is pending
		assert blocked.attached_cyclohexane_cancel_blocked
		assert blocked.start_scene is None and blocked.preview is None
		assert not window._line_gesture_is_current(blocked)
		assert tab.current_snapshot == before
		assert len(calls) == 1 and calls[0] is pending

		window._close_tab_at(window._tab_widget.currentIndex())
		assert window._line_gesture_intent is None
		assert len(calls) == 2 and all(receipt is pending for receipt in calls)
		assert tab.current_snapshot == before
		assert tab.is_disposed
		assert window._tab_widget.indexOf(tab) == -1
	finally:
		window.close()
		window.deleteLater()


@pytest.mark.parametrize("transition", ("tab", "stale", "tab_disposal"))
def test_attach_cyclohexane_terminal_routes_retire_the_pending_receipt(
		qapp: PySide6.QtWidgets.QApplication, transition: str,
		) -> None:
	"""Tab, stale-document, and close routes all reach shared C6 retirement."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='m'><atom id='a' name='C'>"
		"<point x='10' y='20'/></atom></molecule></cdml>", "route-ring.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		pending = _arm_attached_cyclohexane(window, tab, qapp)
		before = tab.current_snapshot
		calls = _count_attached_cyclohexane_cancellations(tab)
		if transition == "tab":
			other = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab("<cdml/>", "other.cdml")
			window._register_native_tab(other, activate=True)
		elif transition == "stale":
			tab._pending_result = object()
			window._refresh_actions()
		else:
			window._close_tab_at(window._tab_widget.currentIndex())
		qapp.processEvents()
		assert window._line_gesture_intent is None
		assert len(calls) == 1 and calls[0] is pending
		assert tab.current_snapshot == before
		if transition == "tab_disposal":
			assert tab.is_disposed and window._tab_widget.indexOf(tab) == -1
	finally:
		window.close()
		window.deleteLater()


def test_attach_cyclohexane_bridge_calls_stay_in_the_private_qt_controller() -> None:
	"""Keep the cooperative C6 bridge confined to its one intended Qt client."""
	root = pathlib.Path(__file__).resolve().parents[1]
	controller = root / "ferrum_qt" / "ferrum" / "attached_cyclohexane_tab.py"
	methods = (
		"_begin_attach_cyclohexane_v1", "_preview_attach_cyclohexane_v1",
		"_commit_attach_cyclohexane_v1", "_cancel_attach_cyclohexane_v1",
	)
	controller_text = controller.read_text(encoding="utf-8")
	assert {method: controller_text.count(method) for method in methods} == {
		method: 1 for method in methods
	}
	allowed = {controller.resolve(), pathlib.Path(__file__).resolve()}
	for path in root.rglob("*.py"):
		text = path.read_text(encoding="utf-8")
		if any(method in text for method in methods):
			assert path.resolve() in allowed

"""Qt coverage for the closed Rust-owned regular-ring chooser family."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.engine
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
def _ring_center(tab: object, size: int) -> PySide6.QtCore.QPoint:
	"""Keep each admitted ring separate while using the ordinary canvas mapping."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(float(size * 80), 120.0))


#============================================
def _ring_atom(tab: object) -> PySide6.QtCore.QPoint:
	"""Return one public hit point for the most recently authored ring."""
	atom = tab.current_document_observation().projection.molecules[-1].atoms[0]
	center = tab.view.mapFromScene(PySide6.QtCore.QPointF(atom.position.x, atom.position.y))
	for delta_x in range(-12, 13):
		for delta_y in range(-12, 13):
			point = center + PySide6.QtCore.QPoint(delta_x, delta_y)
			if tab.durable_atom_at_viewport_point(point) is not None:
				return point
	raise AssertionError("Ferrum ring atom was not available to the public pointer")


#============================================
def test_regular_ring_chooser_actions_commit_each_admitted_size(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Every visible C3-C8 action reaches one generic Rust transition at its size."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	try:
		window.show()
		qapp.processEvents()
		tab = window._active_native_tab()
		assert set(window._regular_ring_actions) == {3, 4, 5, 6, 7, 8}
		for size, action in window._regular_ring_actions.items():
			before_revision = tab.current_snapshot.revision
			action.trigger()
			intent = window._line_gesture_intent
			if intent is None:
				raise AssertionError("regular-ring chooser did not arm a canvas gesture")
			assert intent.regular_ring_size == size
			assert intent.regular_ring_action is action
			PySide6.QtTest.QTest.mouseClick(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _ring_center(tab, size),
			)
			qapp.processEvents()
			assert tab.current_snapshot.revision == before_revision + 1
			assert len(tab.current_document_observation().projection.molecules[-1].atoms) == size
	finally:
		window.cancel_active_pointer_authoring()
		window.close()
		window.deleteLater()


#============================================
def test_regular_ring_accepted_commit_with_unavailable_display_cancels_authoring(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch) -> None:
	"""An accepted ring stays pending when both disposable display installations fail."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	try:
		window.show()
		qapp.processEvents()
		tab = window._active_native_tab()
		action = window._regular_ring_actions[5]
		before_revision = tab.current_snapshot.revision
		refusals: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []

		def capture_refusal(
				request: ferrum_qt.dialogs.refusal_presenter.RefusalRequest) -> None:
			refusals.append(request)

		monkeypatch.setattr(tab._controller, "replace", lambda *_args: False)
		monkeypatch.setattr(window, "_show_edit_refusal", capture_refusal)
		action.trigger()
		assert window._line_gesture_intent is not None
		assert window._line_gesture_intent.regular_ring_action is action
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _ring_center(tab, 5),
		)
		qapp.processEvents()

		assert tab.requires_refresh
		assert tab._pending_snapshot.revision == before_revision + 1
		assert tab.current_snapshot.revision == before_revision
		assert window._line_gesture_intent is None
		assert not action.isChecked()
		assert len(refusals) == 1
		assert refusals[0].context.value == "edit_document"
		assert refusals[0].outcome.value == "unavailable_operation"
		assert refusals[0].technical_details == (
			"The regular ring was inserted, but its authoritative display still needs "
			"recovery; refresh before saving or editing."
		)
	finally:
		window.cancel_active_pointer_authoring()
		window.close()
		window.deleteLater()


#============================================
def test_regular_ring_escape_is_mutation_free_and_disarms(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Escape cancels an armed ring chooser without submitting a Rust operation."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	try:
		window.show()
		qapp.processEvents()
		tab = window._active_native_tab()
		action = window._regular_ring_actions[4]
		before_revision = tab.current_snapshot.revision
		before_digest = tab.current_snapshot.digest
		action.trigger()
		qapp.processEvents()
		assert window._line_gesture_intent is not None
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape,
		)
		qapp.processEvents()

		assert tab.current_snapshot.revision == before_revision
		assert tab.current_snapshot.digest == before_digest
		assert window._line_gesture_intent is None
		assert not action.isChecked()
	finally:
		window.cancel_active_pointer_authoring()
		window.close()
		window.deleteLater()


#============================================
def test_regular_ring_occupied_click_keeps_authoring_armed_for_empty_retry(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""An occupied click is a non-mutating recovery that retains the selected tool."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	try:
		window.show()
		qapp.processEvents()
		tab = window._active_native_tab()
		action = window._regular_ring_actions[6]
		action.trigger()
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _ring_center(tab, 6),
		)
		qapp.processEvents()
		before_revision = tab.current_snapshot.revision
		before_digest = tab.current_snapshot.digest
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _ring_atom(tab),
		)
		qapp.processEvents()

		assert tab.current_snapshot.revision == before_revision
		assert tab.current_snapshot.digest == before_digest
		assert window._line_gesture_intent is not None
		assert action.isChecked()
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _ring_center(tab, 8),
		)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before_revision + 1
	finally:
		window.cancel_active_pointer_authoring()
		window.close()
		window.deleteLater()

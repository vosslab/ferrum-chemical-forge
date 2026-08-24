"""Visible Rust-owned straight normal-Arrow pointer authoring."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import ferrum_chem

# local repo modules
import ferrum_qt.canvas.ferrum_presentation_render_plan
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine
import ferrum_qt.main_window


_EDITABLE_CDML = """<cdml xmlns='urn:ferrum:cdml'>
  <molecule id='mol-1'><atom id='atom-c' name='C'><point x='10' y='20'/></atom></molecule>
</cdml>"""


#============================================
def _scene_point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one finite scene coordinate through the public Qt viewport seam."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


#============================================
def test_arrow_drag_uses_rust_preview_commits_and_selects_durable_root(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""A completed drag has one Rust commit and retains only its durable selector."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_EDITABLE_CDML, "arrow.cdml")
	try:
		preview_plans = []
		build_plan = (
			ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_render_plan
		)
		def record_preview_plan(plan: object, telex: object) -> object:
			preview_plans.append(plan)
			return build_plan(plan, telex)
		monkeypatch.setattr(
			ferrum_qt.canvas.ferrum_presentation_render_plan,
			"build_presentation_render_plan",
			record_preview_plan,
		)
		refusals = []
		monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		commits = []
		commit = tab.commit_straight_normal_arrow_gesture
		monkeypatch.setattr(
			tab, "commit_straight_normal_arrow_gesture",
			lambda gesture, preview: commits.append((gesture, preview)) or commit(gesture, preview),
		)
		start = _scene_point(tab, 24.0, 30.0)
		end = _scene_point(tab, 124.0, 30.0)
		before = tab.current_snapshot.revision
		window._draw_arrow_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before
		assert window._line_gesture_intent.presentation_preview is not None
		assert type(preview_plans[-1]) is ferrum_chem.PresentationRenderPlanV1
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()
		assert len(commits) == 1 and not refusals
		assert window._render_interaction_selection is not None
		assert len(window._render_interaction_selection.roots) == 1
		assert window._render_interaction_selection.roots[0].identifier
		assert window._draw_arrow_action.isChecked()
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_arrow_escape_and_collapsed_refusal_leave_rust_unchanged(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""Cancel and backend endpoint refusal retire the overlay without mutation."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_EDITABLE_CDML, "arrow-cancel.cdml")
	try:
		refusals = []
		monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		start = _scene_point(tab, 24.0, 30.0)
		end = _scene_point(tab, 124.0, 30.0)
		before = tab.current_snapshot.revision
		window._draw_arrow_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before
		assert window._line_gesture_intent is None
		window._draw_arrow_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), start)
		qapp.processEvents()
		assert refusals
		assert window._line_gesture_intent is None
		assert tab.current_snapshot.revision == before
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_equilibrium_arrow_action_is_checkable_named_and_cancels(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The public equilibrium tool owns the same cancellable pointer mode as Arrow."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_EDITABLE_CDML, "equilibrium-action.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		action = window._draw_equilibrium_arrow_action
		assert action.text() == "Draw Equilibrium Arrow" and action.isCheckable()
		window._draw_arrow_action.trigger()
		qapp.processEvents()
		action.trigger()
		qapp.processEvents()
		assert action.isChecked() and not window._draw_arrow_action.isChecked()
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		qapp.processEvents()
		assert window._line_gesture_intent is None and not action.isChecked()
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_arrow_commit_remains_truthful_when_selection_recovery_fails(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""A post-commit selection failure cannot turn an accepted Arrow into a refusal."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_EDITABLE_CDML, "arrow-recovery.cdml")
	try:
		refusals = []
		monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
		monkeypatch.setattr(
			tab, "observe_direct_root_interaction",
			lambda: (_ for _ in ()).throw(RuntimeError("selection observation failed")),
		)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		start, end = _scene_point(tab, 24.0, 30.0), _scene_point(tab, 124.0, 30.0)
		window._draw_arrow_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()
		assert "<arrow" in tab.current_snapshot.cdml
		assert window._render_interaction_selection is None and refusals
		assert "reaction arrow was added" in refusals[-1].technical_details.lower()
	finally:
		window.close()
		window.deleteLater()


#============================================

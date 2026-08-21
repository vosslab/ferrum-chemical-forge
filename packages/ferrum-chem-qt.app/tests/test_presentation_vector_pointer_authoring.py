"""Visible renderer-preflighted ordinary vector authoring through Ferrum Qt."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


_CDML = "<cdml><standard line_color='#123456' line_width='3' area_color='#abcdef'/></cdml>"
#============================================
def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one backend scene coordinate through the visible viewport seam."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))

#============================================
def test_all_vector_tools_commit_rust_canonical_roots_and_square_circle_constraints(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""Each visible action carries opaque Rust handles and creates its exact root."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "vectors.cdml")
	try:
		refusals = []
		monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		tools = tuple(window._draw_vector_actions.items())
		for index, (tool, action) in enumerate(tools):
			start = _point(tab, 20.0 + index * 40.0, 20.0)
			end = _point(tab, 38.0 + index * 40.0, 46.0)
			before = tab.current_snapshot.revision
			action.trigger()
			PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
			PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
			qapp.processEvents()
			intent = window._line_gesture_intent
			assert intent is not None and intent.vector_gesture is not None
			assert intent.vector_preview is not None and tab.current_snapshot.revision == before
			overlay = intent.vector_preview.overlay
			if tool.name in ("DRAW_SQUARE", "DRAW_CIRCLE"):
				assert abs((overlay.right - overlay.left) - (overlay.bottom - overlay.top)) < 1e-9
			if tool.name == "DRAW_RECTANGLE":
				assert intent.preview.pen().color().name() == "#123456"
				assert intent.preview.brush().color().name() == "#abcdef"
			PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
			qapp.processEvents()
			assert tab.current_snapshot.revision == before + 1
			assert window._render_interaction_selection is not None
			assert {
				"DRAW_LINE": "<polyline",
				"DRAW_RECTANGLE": "<rect",
				"DRAW_SQUARE": "<square",
				"DRAW_OVAL": "<oval",
				"DRAW_CIRCLE": "<circle",
			}[tool.name] in tab.current_snapshot.cdml
		assert "<square" in tab.current_snapshot.cdml and "<circle" in tab.current_snapshot.cdml
		assert "line_color=\"#123456\"" in tab.current_snapshot.cdml
		assert not refusals
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_vector_renderer_preflight_exclusion_refuses_before_mutation(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""An existing unsupported renderer root vetoes the prepared vector receipt."""
	cdml = (
		"<cdml><arrow id='retro' type='retro'><point x='0' y='0'/>"
		"<point x='30' y='0'/></arrow></cdml>"
	)
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(cdml, "vector-preflight.cdml")
	try:
		refusals = []
		monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		before = tab.current_snapshot.revision
		action = next(iter(window._draw_vector_actions.values()))
		start, end = _point(tab, 40.0, 20.0), _point(tab, 80.0, 40.0)
		action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()
		assert refusals and window._line_gesture_intent is None
		assert tab.current_snapshot.revision == before and "<polyline" not in tab.current_snapshot.cdml
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_vector_commit_recovers_authoritative_display_after_projection_failure(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""A renderer-accepted receipt remains accepted if its first Qt install fails."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "vector-recovery.cdml")
	try:
		refusals = []
		monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		replace = tab._controller.replace
		failed = [True]
		def fail_once(*args: object, **kwargs: object) -> object:
			if failed[0]:
				failed[0] = False
				raise RuntimeError("forced vector projection failure")
			return replace(*args, **kwargs)
		monkeypatch.setattr(tab._controller, "replace", fail_once)
		start, end = _point(tab, 20.0, 20.0), _point(tab, 60.0, 40.0)
		next(iter(window._draw_vector_actions.values())).trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()
		assert "<polyline" in tab.current_snapshot.cdml and not tab.requires_refresh
		assert refusals and "refreshed the authoritative rust display" in refusals[-1].technical_details.lower()
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_vector_commit_truthfully_requires_recovery_when_refresh_fails(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""An accepted receipt remains pending when its authoritative refresh fails."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "vector-pending-recovery.cdml")
	try:
		refusals = []
		monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		replace = tab._controller.replace
		failed = [True]
		def fail_once(*args: object, **kwargs: object) -> object:
			if failed[0]:
				failed[0] = False
				raise RuntimeError("forced vector projection failure")
			return replace(*args, **kwargs)
		monkeypatch.setattr(tab._controller, "replace", fail_once)
		monkeypatch.setattr(tab, "refresh_authoritative", lambda: False)
		start, end = _point(tab, 20.0, 20.0), _point(tab, 60.0, 40.0)
		next(iter(window._draw_vector_actions.values())).trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end)
		qapp.processEvents()
		assert "<polyline" in tab.backend_snapshot_for_recovery_export().cdml and tab.requires_refresh
		assert refusals and "still needs recovery" in refusals[-1].technical_details.lower()
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_vector_escape_and_degenerate_release_preserve_document(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""Cancellation and typed geometry refusal retire the bridge preview pre-commit."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "vector-cancel.cdml")
	try:
		refusals = []
		monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		point = _point(tab, 20.0, 20.0)
		action = next(iter(window._draw_vector_actions.values()))
		before = tab.current_snapshot.revision
		action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), _point(tab, 50.0, 40.0))
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		assert window._line_gesture_intent is None and tab.current_snapshot.revision == before
		action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		qapp.processEvents()
		assert refusals and tab.current_snapshot.revision == before
	finally:
		window.close()
		window.deleteLater()

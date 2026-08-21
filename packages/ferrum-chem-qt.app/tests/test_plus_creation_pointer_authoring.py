"""Visible Rust-owned direct Plus pointer authoring."""

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


_CDML = "<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'><molecule id='m'><atom id='a' name='C'><point x='10' y='20'/></atom></molecule></cdml>"


def test_plus_click_commits_selects_and_remains_movable(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The Qt tool carries only opaque Rust handles and leaves one durable Plus."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "plus.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		point = tab.view.mapFromScene(PySide6.QtCore.QPointF(72.0, 36.0))
		window._draw_plus_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		assert tab.current_snapshot.revision == 0
		assert window._line_gesture_intent.presentation_preview is not None
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		qapp.processEvents()
		assert '<plus' in tab.current_snapshot.cdml
		assert window._render_interaction_selection is not None
		tab.undo()
		assert '<plus' not in tab.current_snapshot.cdml
	finally:
		window.close()
		window.deleteLater()


def test_plus_commit_truthfully_requires_recovery_when_projection_refresh_fails(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""An accepted Plus is never reported as refreshed when refresh failed."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "plus-recovery.cdml")
	try:
		refusals = []
		monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		replace = tab._controller.replace
		failed = [True]
		def fail_install(*args: object, **kwargs: object) -> object:
			if failed[0]:
				failed[0] = False
				raise RuntimeError("forced Plus projection failure")
			return replace(*args, **kwargs)
		monkeypatch.setattr(tab._controller, "replace", fail_install)
		monkeypatch.setattr(tab, "refresh_authoritative", lambda: False)
		point = tab.view.mapFromScene(PySide6.QtCore.QPointF(72.0, 36.0))
		window._draw_plus_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		qapp.processEvents()
		assert '<plus' in tab.backend_snapshot_for_recovery_export().cdml and tab.requires_refresh
		assert refusals and 'still needs recovery' in refusals[-1].technical_details.lower()
		assert 'refreshed' not in refusals[-1].technical_details.lower()
	finally:
		window.close()
		window.deleteLater()


def test_plus_preview_uses_the_renderer_custom_rgb_paint(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Qt converts the renderer's closed RGB wire value before painting it."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml><standard line_color='#123456'/></cdml>", "plus-color.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		point = tab.view.mapFromScene(PySide6.QtCore.QPointF(72.0, 36.0))
		window._draw_plus_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(),
			PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		preview = window._line_gesture_intent.preview
		color = preview.brush().color()
		assert color.isValid()
		assert color.name() == "#123456"
		assert color.name() != "#000000"
	finally:
		window.close()
		window.deleteLater()

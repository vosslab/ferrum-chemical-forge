"""Visible Rust-owned standalone Text pointer authoring."""

import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

import ferrum_qt.themes.theme_loader
import ferrum_qt.dialogs.rich_text_dialog
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.text_placement
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_CDML = "<cdml xmlns='urn:ferrum:cdml'><molecule id='m'><atom id='a' name='C'><point x='10' y='20'/></atom></molecule></cdml>"


def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one finite scene coordinate through the active Qt viewport."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


def _accept_h2o(dialog: object) -> int:
	"""Enter a baseline/subscript/baseline value through actual dialog widgets."""
	dialog.show()
	PySide6.QtWidgets.QApplication.processEvents()
	assert dialog.focusWidget() is dialog._text_edit
	assert not dialog._bold_button.isEnabled() and not dialog._italic_button.isEnabled()
	assert dialog._sub_button.isEnabled() and dialog._sup_button.isEnabled()
	dialog._text_edit.setPlainText("H2O")
	cursor = dialog._text_edit.textCursor()
	cursor.setPosition(1)
	cursor.setPosition(2, PySide6.QtGui.QTextCursor.MoveMode.KeepAnchor)
	dialog._text_edit.setTextCursor(cursor)
	dialog._toggle_style("sub", True)
	dialog.accept()
	return int(dialog.result())


def test_text_adapter_accepts_baseline_subscript_superscript_and_refuses_bold() -> None:
	"""Only the P1 renderer-backed authored style subset crosses into Rust."""
	runs = ferrum_qt.ferrum.text_placement.runs_from_dialog((
		("H", ()), ("2", ("sub",)), ("+", ("sup",)),
	))
	assert len(runs) == 3
	with pytest.raises(ValueError, match="baseline, subscript, and superscript"):
		ferrum_qt.ferrum.text_placement.runs_from_dialog((("bold", ("b",)),))


def test_text_dialog_escape_rejects_without_exporting_draft(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The modal authoring draft is disposable when the author presses Escape."""
	dialog = ferrum_qt.dialogs.rich_text_dialog.RichTextDialog(
		(("Text", ()),), capabilities=(
			ferrum_qt.dialogs.rich_text_dialog.RichTextDialogCapabilities(
				bold=False, italic=False, font_family=False,
			)
		), initial_text_selected=True,
	)
	try:
		dialog.show()
		qapp.processEvents()
		PySide6.QtTest.QTest.keyClick(dialog, PySide6.QtCore.Qt.Key.Key_Escape)
		assert dialog.result() == PySide6.QtWidgets.QDialog.DialogCode.Rejected
	finally:
		dialog.deleteLater()


def test_text_click_commits_exact_runs_selects_and_remains_movable(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object) -> None:
	"""Text uses one opaque Rust preview/commit and later P0.2 translation."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "text.cdml", ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		refusals = []
		monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
		monkeypatch.setattr(ferrum_qt.dialogs.rich_text_dialog.RichTextDialog, "exec", _accept_h2o)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		created_at = _point(tab, 72.0, 36.0)
		window._insert_text_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, created_at)
		qapp.processEvents()
		created = tab.current_snapshot.cdml
		assert "<text" in created and "H&lt;sub&gt;2&lt;/sub&gt;O" in created, refusals
		assert "<font" not in created
		assert window._render_interaction_selection is not None
		window._translate_roots_action.trigger()
		selected_at = _point(tab, 72.0, 36.0)
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, selected_at)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), _point(tab, 92.0, 54.0))
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 92.0, 54.0))
		qapp.processEvents()
		assert tab.current_snapshot.cdml != created
		tab.undo()
		assert tab.current_snapshot.cdml == created
	finally:
		window.close()
		window.deleteLater()


def test_text_cancel_and_preview_failure_leave_cdml_unchanged(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object) -> None:
	"""Dialog cancellation and any backend failure cancel transient state only."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "text-cancel.cdml", ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		monkeypatch.setattr(window, "_show_edit_refusal", lambda _request: None)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		before = tab.current_snapshot.revision
		point = _point(tab, 72.0, 36.0)
		monkeypatch.setattr(ferrum_qt.dialogs.rich_text_dialog.RichTextDialog, "exec",
			lambda _dialog: PySide6.QtWidgets.QDialog.DialogCode.Rejected)
		window._insert_text_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		assert tab.current_snapshot.revision == before
		assert window._line_gesture_intent is None and not window._insert_text_action.isChecked()
		monkeypatch.setattr(ferrum_qt.dialogs.rich_text_dialog.RichTextDialog, "exec",
			lambda _dialog: PySide6.QtWidgets.QDialog.DialogCode.Accepted)
		monkeypatch.setattr(tab, "preview_text_placement_gesture",
			lambda *_args: (_ for _ in ()).throw(RuntimeError("typed preview refusal")))
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before
		assert window._line_gesture_intent is None
	finally:
		window.close()
		window.deleteLater()


def test_text_escape_focus_loss_and_tool_change_cancel_coherently(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Escape, settled focus loss, and tool changes cancel visible Text state."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "text-lifecycle.cdml", ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		before = tab.current_snapshot.revision
		window._insert_text_action.trigger()
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		assert window._line_gesture_intent is None and not window._insert_text_action.isChecked()
		window._insert_text_action.trigger()
		qapp.processEvents()
		qapp.sendEvent(tab.view.viewport(), PySide6.QtGui.QFocusEvent(
			PySide6.QtCore.QEvent.Type.FocusOut,
		))
		tab.view.viewport().clearFocus()
		qapp.processEvents()
		assert window._line_gesture_intent is None and not window._insert_text_action.isChecked()
		window._insert_text_action.trigger()
		window._draw_plus_action.trigger()
		assert not window._insert_text_action.isChecked() and window._draw_plus_action.isChecked()
		window._on_cancel_tool()
		assert window._line_gesture_intent is None and not window._draw_plus_action.isChecked()
		assert tab.current_snapshot.revision == before
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_text_popup_focus_handoff_retains_the_same_armed_intent(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A transient popup FocusOut cannot disarm a viewport that regains focus."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "text-focus-handoff.cdml", ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		window._insert_text_action.trigger()
		qapp.processEvents()
		intent = window._line_gesture_intent
		assert intent is not None
		qapp.sendEvent(tab.view.viewport(), PySide6.QtGui.QFocusEvent(
			PySide6.QtCore.QEvent.Type.FocusOut,
		))
		tab.view.viewport().setFocus()
		qapp.processEvents()
		assert window._line_gesture_intent is intent
		assert window._insert_text_action.isChecked()
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_text_stale_focus_callback_cannot_cancel_a_replacement_intent(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A queued focus-loss callback is fenced to the intent that created it."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "text-stale-focus.cdml", ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		window._insert_text_action.trigger()
		qapp.processEvents()
		old_intent = window._line_gesture_intent
		assert old_intent is not None
		qapp.sendEvent(tab.view.viewport(), PySide6.QtGui.QFocusEvent(
			PySide6.QtCore.QEvent.Type.FocusOut,
		))
		window._draw_plus_action.trigger()
		qapp.processEvents()
		assert window._line_gesture_intent is not old_intent
		assert window._line_gesture_intent is not None
		assert window._draw_plus_action.isChecked()
	finally:
		window.close()
		window.deleteLater()


def test_text_stale_focus_restoration_cannot_touch_a_replacement_intent(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object) -> None:
	"""A stale popup-restoration turn cannot reclaim focus from a new tool."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "text-stale-restore.cdml", ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		callbacks = []
		monkeypatch.setattr(
			PySide6.QtCore.QTimer, "singleShot",
			lambda _delay, callback: callbacks.append(callback),
		)
		window._insert_text_action.trigger()
		old_intent = window._line_gesture_intent
		assert old_intent is not None
		window._draw_plus_action.trigger()
		replacement = window._line_gesture_intent
		assert replacement is not None and replacement is not old_intent
		tab.view.viewport().clearFocus()
		callbacks[0]()
		assert window._line_gesture_intent is replacement and window._draw_plus_action.isChecked()
		assert not tab.view.viewport().hasFocus()
	finally:
		window.close()
		window.deleteLater()


def test_text_commit_selection_failure_reports_recovery_after_rust_acceptance(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object) -> None:
	"""Lost durable selection cannot be presented as ordinary selectable success."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "text-recovery.cdml", ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		refusals = []
		monkeypatch.setattr(window, "_show_edit_refusal", lambda request: refusals.append(request))
		monkeypatch.setattr(ferrum_qt.dialogs.rich_text_dialog.RichTextDialog, "exec",
			lambda dialog: dialog.accept() or int(dialog.result()))
		monkeypatch.setattr(tab, "observe_direct_root_interaction", lambda: (
			_ for _ in ()
		).throw(ferrum_qt.ferrum.engine.RenderInteractionError("selection observation failed")))
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		window._insert_text_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 72.0, 36.0))
		qapp.processEvents()
		assert "<text" in tab.current_snapshot.cdml
		assert window._render_interaction_selection is None and refusals
		assert "text was added" in refusals[-1].technical_details.lower()
		assert "select it again before moving" in refusals[-1].technical_details.lower()
	finally:
		window.close()
		window.deleteLater()


def test_text_dialog_tab_order_and_enter_save_are_accessible(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Live metadata names the Text task controls and permits keyboard Save."""
	dialog = ferrum_qt.dialogs.rich_text_dialog.RichTextDialog(
		(("Text", ()),), capabilities=(
			ferrum_qt.dialogs.rich_text_dialog.RichTextDialogCapabilities(
				bold=False, italic=False, font_family=False,
			)
		), initial_text_selected=True,
	)
	try:
		dialog.show()
		qapp.processEvents()
		assert dialog._font_face.text() == "Telex Regular (bundled)"
		assert dialog._font_face.focusPolicy() == PySide6.QtCore.Qt.FocusPolicy.NoFocus
		assert "font_family" not in dialog.font_values()
		metadata = ferrum_qt.dialogs.accessibility.DIALOG_ACCESSIBILITY_METADATA[
			"RichTextDialog"
		]
		assert metadata.initial_focus == "Text content"
		assert metadata.tab_order[:3] == ("Text content", "Font size", "Subscript")
		assert "Superscript" in metadata.tab_order and "Text color" in metadata.tab_order
		PySide6.QtTest.QTest.keyClick(dialog, PySide6.QtCore.Qt.Key.Key_Tab)
		assert dialog.focusWidget().accessibleName() == "Font size"
		for _unused in range(4):
			PySide6.QtTest.QTest.keyClick(dialog, PySide6.QtCore.Qt.Key.Key_Tab)
		assert dialog.focusWidget().accessibleName() == "Save"
		PySide6.QtTest.QTest.keyClick(dialog, PySide6.QtCore.Qt.Key.Key_Return)
		assert dialog.result() == PySide6.QtWidgets.QDialog.DialogCode.Accepted
	finally:
		dialog.deleteLater()

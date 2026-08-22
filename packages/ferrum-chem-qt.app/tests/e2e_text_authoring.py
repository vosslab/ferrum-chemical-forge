"""Offscreen Ferrum workflow: create, move, undo, save, and reopen Text."""

import json
import os
import pathlib
import sys

import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

import ferrum_chem
import ferrum_qt.dialogs.rich_text_dialog
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


_CDML = "<cdml xmlns='urn:ferrum:cdml'><molecule id='m'><atom id='a' name='C'><point x='10' y='20'/></atom></molecule></cdml>"


def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


def _accept_h2o_with_overrides(dialog: object) -> int:
	"""Author the P1 formatted text subset and explicit root overrides by keyboard."""
	dialog._text_edit.setPlainText("H2O")
	cursor = dialog._text_edit.textCursor()
	cursor.setPosition(1)
	cursor.setPosition(2, PySide6.QtGui.QTextCursor.MoveMode.KeepAnchor)
	dialog._text_edit.setTextCursor(cursor)
	dialog._toggle_style("sub", True)
	dialog._font_spin.setValue(18)
	dialog._color = "#123456"
	dialog._update_color_button()
	dialog.accept()
	return int(dialog.result())


def _reopen(window: object, path: pathlib.Path) -> object:
	completed = []
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)
	def receive(value: str, success: bool) -> None:
		if pathlib.Path(value) == path:
			completed.append(success)
			loop.quit()
	window.local_document_open_completed.connect(receive)
	timeout.timeout.connect(loop.quit)
	try:
		if not window.open_file_path(str(path)):
			raise RuntimeError("Ferrum native Open did not accept the saved Text file")
		timeout.start(10_000)
		loop.exec()
		if completed != [True]:
			raise RuntimeError("Ferrum native Open did not complete the saved Text route")
		return window._active_native_tab()
	finally:
		timeout.stop()
		window.local_document_open_completed.disconnect(receive)


def main() -> int:
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "text-e2e.cdml")
	original_exec = ferrum_qt.dialogs.rich_text_dialog.RichTextDialog.exec
	try:
		ferrum_qt.dialogs.rich_text_dialog.RichTextDialog.exec = _accept_h2o_with_overrides
		window._register_native_tab(tab, activate=True)
		window.show()
		app.processEvents()
		point = _point(tab, 72.0, 36.0)
		window._insert_text_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		app.processEvents()
		created = tab.current_snapshot.cdml
		if (
			"<text" not in created
			or "H&lt;sub&gt;2&lt;/sub&gt;O" not in created
			or '<font size="18" color="#123456"/>' not in created
			or window._render_interaction_selection is None
		):
			raise RuntimeError("Insert Text did not create and select one durable Text")
		window._translate_roots_action.trigger()
		PySide6.QtTest.QTest.mousePress(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), _point(tab, 92.0, 54.0))
		PySide6.QtTest.QTest.mouseRelease(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 92.0, 54.0))
		app.processEvents()
		if tab.current_snapshot.cdml == created:
			raise RuntimeError("Move Complete Roots did not translate the created Text")
		tab.undo()
		app.processEvents()
		if tab.current_snapshot.cdml != created:
			raise RuntimeError("Undo did not restore the created Text document")
		path = pathlib.Path("/private/tmp") / f"ferrum-text-e2e-{os.getpid()}.cdml"
		if not window.save_active_to_path(str(path)):
			raise RuntimeError("Ferrum native Save As did not publish the Text document")
		reopened_session = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
		reopened_cdml = reopened_session.snapshot().cdml
		text = reopened_session.observe(0).projection.presentation_stack.roots[-1].text
		if (
			"H&lt;sub&gt;2&lt;/sub&gt;O" not in reopened_cdml
			or '<font size="18" color="#123456"/>' not in reopened_cdml
			or text.font.size != 18.0 or text.font.color != "#123456"
			or [(run.text, run.styles) for run in text.runs] != [
				("H", ()), ("2", ("subscript",)), ("O", ()),
			]
		):
			raise RuntimeError("Rust reopen did not preserve the authored Text")
		reopened = _reopen(window, path)
		if (
			reopened is None or "H&lt;sub&gt;2&lt;/sub&gt;O" not in reopened.current_snapshot.cdml
			or '<font size="18" color="#123456"/>' not in reopened.current_snapshot.cdml
		):
			raise RuntimeError("Ferrum native Open did not reproject the saved Text")
		print(json.dumps({"schema": "ferrum-text-authoring-e2e-v1", "status": "ok"}))
		return 0
	finally:
		ferrum_qt.dialogs.rich_text_dialog.RichTextDialog.exec = original_exec
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	sys.exit(main())

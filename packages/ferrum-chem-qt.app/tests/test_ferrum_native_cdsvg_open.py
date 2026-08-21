"""Semantic product coverage for decoded CD-SVG in the Ferrum Open lifecycle."""

from __future__ import annotations

import collections.abc
import pathlib

import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

import ferrum_qt.main_window
import ferrum_qt.ferrum.document_tab


_CDSVG = """<svg xmlns="http://www.w3.org/2000/svg">
<cdml xmlns="http://www.freesoftware.fsf.org/bkchem/cdml" version="1.0">
  <plus id="payload-fact"><point x="3" y="4"/></plus>
</cdml><metadata>discarded wrapper content</metadata></svg>"""
_CDML = """<cdml version="1.0"><molecule id="molecule-1">
<atom id="atom-c" name="C"><point x="1" y="2"/></atom>
</molecule></cdml>"""


#============================================
def _make_window(
		qapp: PySide6.QtWidgets.QApplication,
		) -> ferrum_qt.main_window.MainWindow:
	"""Create the ordinary Ferrum product window."""
	del qapp
	return ferrum_qt.main_window.MainWindow(object())


#============================================
def _current_tab(
		window: PySide6.QtWidgets.QMainWindow,
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Return the selected Ferrum document through the public central page."""
	tabs = window.centralWidget()
	assert isinstance(tabs, PySide6.QtWidgets.QTabWidget)
	tab = tabs.currentWidget()
	assert isinstance(tab, ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab)
	return tab


#============================================
def _visible_action(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		) -> PySide6.QtGui.QAction:
	"""Return one user-visible command by its caption."""
	return next(action for action in window.findChildren(PySide6.QtGui.QAction) if action.text() == label)


#============================================
def _wait_for_open(
		window: ferrum_qt.main_window.MainWindow,
		start: collections.abc.Callable[[], object],
		) -> bool:
	"""Wait on the Ferrum controller's completion signal without a timing gate."""
	loop = PySide6.QtCore.QEventLoop()
	outcomes: list[bool] = []

	def complete(success: bool) -> None:
		"""Record the controller outcome and leave its completion loop."""
		outcomes.append(success)
		loop.quit()

	window.local_document_open_queue_drained.connect(complete)
	try:
		start()
		loop.exec()
	finally:
		window.local_document_open_queue_drained.disconnect(complete)
	return outcomes[0]


#============================================
def _choose_open(
		monkeypatch: pytest.MonkeyPatch, source: pathlib.Path,
		) -> None:
	"""Make the public chooser select one local decoded SVG."""
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getOpenFileName",
		lambda *_args: (str(source), ""),
	)


#============================================
def _click_message_button(
		qapp: PySide6.QtWidgets.QApplication, label: str,
		) -> PySide6.QtCore.QTimer:
	"""Choose one visible recovery action after its message box appears."""
	timer = PySide6.QtCore.QTimer(qapp)

	def click() -> None:
		"""Click the named visible message-box button once available."""
		for dialog in qapp.topLevelWidgets():
			if not isinstance(dialog, PySide6.QtWidgets.QMessageBox) or not dialog.isVisible():
				continue
			button = next((item for item in dialog.buttons() if item.text().replace("&", "") == label), None)
			if button is not None:
				PySide6.QtTest.QTest.mouseClick(button, PySide6.QtCore.Qt.MouseButton.LeftButton)
				return

	timer.timeout.connect(click)
	timer.start(1)
	return timer


#============================================
def _acknowledge_message(
		qapp: PySide6.QtWidgets.QApplication,
		) -> tuple[PySide6.QtCore.QTimer, list[str]]:
	"""Acknowledge one visible recovery message and retain its safe recovery text."""
	messages: list[str] = []
	timer = PySide6.QtCore.QTimer(qapp)

	def acknowledge() -> None:
		"""Capture the visible recovery text and choose the acknowledgement."""
		for dialog in qapp.topLevelWidgets():
			if not isinstance(dialog, PySide6.QtWidgets.QMessageBox) or not dialog.isVisible():
				continue
			messages.append(dialog.text())
			button = next((item for item in dialog.buttons() if item.text().replace("&", "") == "OK"), None)
			if button is not None:
				PySide6.QtTest.QTest.mouseClick(button, PySide6.QtCore.Qt.MouseButton.LeftButton)
				return

	timer.timeout.connect(acknowledge)
	timer.start(1)
	return timer, messages


#============================================
def test_visible_open_decodes_svg_then_save_as_publishes_cdml_without_rewriting_wrapper(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A visible SVG Open is an extraction source, while Save As becomes CDML."""
	source = tmp_path / "drawing.svg"
	destination = tmp_path / "published.cdml"
	source.write_text(_CDSVG, encoding="utf-8")
	wrapper_before = source.read_text(encoding="utf-8")
	_choose_open(monkeypatch, source)
	window = _make_window(qapp)
	try:
		assert _wait_for_open(window, _visible_action(window, "Open").trigger)
		tab = _current_tab(window)
		assert tab.file_path is None and "embedded CDML document" in tab.toolTip()
		assert window.save_active_to_path(str(destination))
		assert destination.exists() and source.read_text(encoding="utf-8") == wrapper_before
		assert _wait_for_open(window, lambda: window.open_file_path(str(source)))
		assert _current_tab(window) is tab and "<svg" not in tab.current_snapshot.cdml
	finally:
		window.close()


#============================================
def test_svg_hardlink_activates_the_original_tab_after_cdml_publication(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""The retained SVG descriptor identity survives a later CDML Save As."""
	source = tmp_path / "drawing.svg"
	alias = tmp_path / "drawing-alias.svg"
	destination = tmp_path / "published.cdml"
	source.write_text(_CDSVG, encoding="utf-8")
	alias.hardlink_to(source)
	window = _make_window(qapp)
	try:
		assert _wait_for_open(window, lambda: window.open_file_path(str(source)))
		opened = _current_tab(window)
		assert window.save_active_to_path(str(destination))
		assert _wait_for_open(window, lambda: window.open_file_path(str(alias)))
		assert _current_tab(window) is opened and opened.file_path == destination
	finally:
		window.close()


#============================================
def test_explicit_current_svg_replaces_clean_tab_but_cancel_and_refusal_preserve_dirty_tab(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Explicit replacement admits first, preserving a dirty target on recovery."""
	target_path = tmp_path / "target.cdml"
	source = tmp_path / "replacement.svg"
	cancel_source = tmp_path / "cancel-replacement.svg"
	malformed = tmp_path / "rejected.svg"
	target_path.write_text(_CDML, encoding="utf-8")
	source.write_text(_CDSVG, encoding="utf-8")
	cancel_source.write_text(_CDSVG.replace("payload-fact", "cancel-fact"), encoding="utf-8")
	malformed.write_text('<svg xmlns="http://www.w3.org/2000/svg"/>', encoding="utf-8")
	window = _make_window(qapp)
	try:
		assert _wait_for_open(window, lambda: window.open_file_path(str(target_path)))
		target = _current_tab(window)
		_choose_open(monkeypatch, source)
		assert _wait_for_open(window, _visible_action(window, "Open in Current Tab...").trigger)
		assert _current_tab(window) is not target and _current_tab(window).file_path is None
		assert _wait_for_open(window, lambda: window.open_file_path(str(target_path)))
		current = _current_tab(window)
		current.select_atom("atom-c")
		current.change_selected_atom_element("N")
		baseline = current.current_snapshot
		cancel = _click_message_button(qapp, "Cancel")
		try:
			_choose_open(monkeypatch, cancel_source)
			assert not _wait_for_open(window, _visible_action(window, "Open in Current Tab...").trigger)
		finally:
			cancel.stop()
		assert _current_tab(window) is current and current.current_snapshot == baseline
		acknowledge = _click_message_button(qapp, "OK")
		try:
			assert not _wait_for_open(window, lambda: window.open_in_current_tab_path(str(malformed)))
		finally:
			acknowledge.stop()
		assert _current_tab(window) is current and current.current_snapshot == baseline
	finally:
		window.close()


#============================================
def test_recent_svg_request_forces_new_tab_and_keeps_source_out_of_cdml(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""Recent CD-SVG composition leaves the existing document and CDML personal-free."""
	first = tmp_path / "first.cdml"
	source = tmp_path / "recent.svg"
	first.write_text(_CDML, encoding="utf-8")
	source.write_text(_CDSVG, encoding="utf-8")
	window = _make_window(qapp)
	try:
		assert _wait_for_open(window, lambda: window.open_file_path(str(first)))
		existing = _current_tab(window)
		baseline = existing.current_snapshot
		assert _wait_for_open(window, lambda: window.open_recent_native_document_path(str(source)))
		opened = _current_tab(window)
		assert opened is not existing and existing.current_snapshot == baseline
		assert str(source) not in opened.current_snapshot.cdml
	finally:
		window.close()


#============================================
def test_svg_installation_failure_preserves_the_current_document(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A construction failure after admission never replaces the current Rust page."""
	target_path = tmp_path / "target.cdml"
	source = tmp_path / "install-failure.svg"
	target_path.write_text(_CDML, encoding="utf-8")
	source.write_text(_CDSVG, encoding="utf-8")
	window = _make_window(qapp)
	try:
		assert _wait_for_open(window, lambda: window.open_file_path(str(target_path)))
		target = _current_tab(window)
		baseline = target.current_snapshot

		def fail_construction(*_args: object) -> object:
			"""Model the bounded post-admission construction failure seam."""
			raise RuntimeError("test construction failure")

		monkeypatch.setattr(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			"from_admitted_local_open",
			fail_construction,
		)
		acknowledge, messages = _acknowledge_message(qapp)
		try:
			assert not _wait_for_open(window, lambda: window.open_file_path(str(source)))
		finally:
			acknowledge.stop()
		assert _current_tab(window) is target and target.current_snapshot == baseline
		assert (
			"current tab is unchanged" in messages[-1].lower()
			and "test construction failure" not in messages[-1]
		)
	finally:
		window.close()

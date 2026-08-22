"""Behavior coverage for explicit asynchronous CDML replacement in the active tab."""

# Standard Library
import collections.abc
import pathlib

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


_EMPTY_CDML = '<cdml xmlns="urn:ferrum:cdml" version="1.0"/>'
_EDITABLE_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
</molecule></cdml>"""


#============================================
def _make_window(
		qapp: PySide6.QtWidgets.QApplication,
		) -> ferrum_qt.main_window.MainWindow:
	"""Create the ordinary Ferrum product window."""
	del qapp
	return ferrum_qt.main_window.MainWindow(object())


#============================================
def _open_saved_native_tab(
		qapp: PySide6.QtWidgets.QApplication,
		window: ferrum_qt.main_window.MainWindow,
		path: pathlib.Path,
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Load one real saved document through the ordinary Ferrum ingress."""
	assert _wait_for_open_queue(window, lambda: window.open_file_path(str(path)))
	tab = _current_native_tab(window)
	assert tab.file_path == path and not tab.current_snapshot.is_dirty
	return tab


#============================================
def _visible_action(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		) -> PySide6.QtGui.QAction:
	"""Return one user-visible Ferrum command by its stable label."""
	return next(
		candidate
		for candidate in window.findChildren(PySide6.QtGui.QAction)
		if candidate.text() == label
	)


#============================================
def _click_visible_message_button(
		qapp: PySide6.QtWidgets.QApplication, label: str,
		) -> PySide6.QtCore.QTimer:
	"""Click one explicitly named button on the visible Ferrum recovery dialog."""
	timer = PySide6.QtCore.QTimer(qapp)

	def click() -> None:
		"""Click the requested visible action once its message box is shown."""
		for dialog in qapp.topLevelWidgets():
			if not isinstance(dialog, PySide6.QtWidgets.QMessageBox) or not dialog.isVisible():
				continue
			button = next(
				(
					candidate
					for candidate in dialog.buttons()
					if candidate.text().replace("&", "") == label
				),
				None,
			)
			if button is not None:
				PySide6.QtTest.QTest.mouseClick(
					button, PySide6.QtCore.Qt.MouseButton.LeftButton,
				)
				return

	timer.timeout.connect(click)
	timer.start(1)
	return timer


#============================================
def _current_native_tab(
		window: PySide6.QtWidgets.QMainWindow,
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Return the selected Ferrum page through the public central widget tree."""
	tab_widget = window.centralWidget()
	assert isinstance(tab_widget, PySide6.QtWidgets.QTabWidget)
	tab = tab_widget.currentWidget()
	assert isinstance(
		tab, ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
	)
	return tab


#============================================
def _wait_for_open_queue(
		window: ferrum_qt.main_window.MainWindow,
		start: collections.abc.Callable[[], object],
		) -> bool:
	"""Run one queued Open batch with one completion outcome or bounded timeout."""
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)
	outcome: bool | None = None

	def finish(success: bool) -> None:
		"""Capture the public batch outcome and stop the local event loop."""
		nonlocal outcome
		if outcome is not None:
			return
		outcome = success
		loop.quit()

	def expire() -> None:
		"""Finish the bounded wait when the public completion signal is missing."""
		finish(False)

	window.local_document_open_queue_drained.connect(finish)
	timeout.timeout.connect(expire)
	PySide6.QtCore.QTimer.singleShot(0, start)
	timeout.start(10000)
	loop.exec()
	timeout.stop()
	window.local_document_open_queue_drained.disconnect(finish)
	timeout.timeout.disconnect(expire)
	return outcome is True


#============================================
def _author_dirty_atom(
		window: ferrum_qt.main_window.MainWindow,
		element: str = "N",
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Make one authoritative atom change on an otherwise idle Ferrum tab."""
	tab = _current_native_tab(window)
	tab.select_atom("atom-c")
	tab.change_selected_atom_element(element)
	assert tab.current_snapshot.is_dirty
	return tab


#============================================
def _trigger_current_tab_open(
		window: ferrum_qt.main_window.MainWindow,
		path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> bool:
	"""Choose one explicit replacement source through the public File action."""
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getOpenFileName",
		lambda *_args: (str(path), "Ferrum CDML (*.cdml)"),
	)
	action = _visible_action(window, "Open in Current Tab...")
	assert action.isEnabled()
	return _wait_for_open_queue(window, action.trigger)


#============================================
def test_open_in_current_tab_replaces_a_clean_saved_document_in_place(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The explicit File command swaps one admitted clean document at its tab position."""
	target_path = tmp_path / "target.cdml"
	incoming_path = tmp_path / "incoming.cdml"
	target_path.write_text(_EDITABLE_CDML, encoding="utf-8")
	incoming_path.write_text(
		'<cdml xmlns="urn:ferrum:cdml" version="1.0"><plus id="incoming"><point x="3" y="4"/></plus></cdml>',
		encoding="utf-8",
	)
	window = _make_window(qapp)
	try:
		target = _open_saved_native_tab(qapp, window, target_path)
		index = window.centralWidget().currentIndex()
		assert _trigger_current_tab_open(window, incoming_path, monkeypatch)
		installed = _current_native_tab(window)
		assert (
			installed is not target
			and installed.file_path == incoming_path
			and not installed.current_snapshot.is_dirty
			and window.centralWidget().currentIndex() == index
			and target.is_disposed
		)
	finally:
		window.close()


#============================================
def test_open_in_current_tab_replace_discards_only_the_dirty_target(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Replace intentionally discards unsaved authored work without publishing it."""
	target_path = tmp_path / "target.cdml"
	incoming_path = tmp_path / "incoming.cdml"
	target_path.write_text(_EDITABLE_CDML, encoding="utf-8")
	incoming_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	try:
		_open_saved_native_tab(qapp, window, target_path)
		target = _author_dirty_atom(window)
		assert window.save_active_to_path(str(target_path))
		target = _author_dirty_atom(window, "O")
		discarded_digest = target.current_snapshot.digest
		click = _click_visible_message_button(qapp, "Replace")
		try:
			assert _trigger_current_tab_open(window, incoming_path, monkeypatch)
		finally:
			click.stop()
		installed = _current_native_tab(window)
		prepared = ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(str(target_path))
		reopened, _observation, _origin, _source_kind = prepared.take_admission_v1()
		assert (
			installed.file_path == incoming_path
			and installed is not target
			and reopened.snapshot().digest != discarded_digest
		)
	finally:
		window.close()


#============================================
def test_open_in_current_tab_save_publishes_a_dirty_named_target_before_swap(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Save establishes a durable baseline before the admitted tab replaces it."""
	target_path = tmp_path / "named-target.cdml"
	incoming_path = tmp_path / "incoming.cdml"
	target_path.write_text(_EDITABLE_CDML, encoding="utf-8")
	incoming_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	try:
		_open_saved_native_tab(qapp, window, target_path)
		target = _author_dirty_atom(window)
		dirty_digest = target.current_snapshot.digest
		click = _click_visible_message_button(qapp, "Save")
		try:
			assert _trigger_current_tab_open(window, incoming_path, monkeypatch)
		finally:
			click.stop()
		reopened, _observation, _origin, _source_kind = (
			ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(str(target_path)).take_admission_v1()
		)
		assert (
			_current_native_tab(window).file_path == incoming_path
			and reopened.snapshot().digest == dirty_digest
		)
	finally:
		window.close()


#============================================
def test_open_in_current_tab_save_as_publishes_dirty_unnamed_target_before_swap(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Save As durably publishes the old authored molecule before explicit replacement."""
	incoming_path = tmp_path / "incoming.cdml"
	saved_path = tmp_path / "saved-before-replace.cdml"
	incoming_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	try:
		target = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
			_EDITABLE_CDML, "Untitled",
		)
		window._register_native_tab(target, activate=True)
		target = _author_dirty_atom(window)
		dirty_digest = target.current_snapshot.digest
		monkeypatch.setattr(
			PySide6.QtWidgets.QFileDialog,
			"getSaveFileName",
			lambda *_args: (str(saved_path), "Ferrum CDML (*.cdml)"),
		)
		click = _click_visible_message_button(qapp, "Save")
		try:
			assert _trigger_current_tab_open(window, incoming_path, monkeypatch)
		finally:
			click.stop()
		reopened, _observation, _origin, _source_kind = (
			ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(str(saved_path)).take_admission_v1()
		)
		assert (
			_current_native_tab(window).file_path == incoming_path
			and reopened.snapshot().digest == dirty_digest
		)
	finally:
		window.close()


#============================================
def test_open_in_current_tab_cancel_and_admission_failure_preserve_dirty_target(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Cancel and rejected admission retain the active authored snapshot and focus."""
	incoming_path = tmp_path / "incoming.cdml"
	source = tmp_path / "source.cdml"
	link = tmp_path / "source-link.cdml"
	incoming_path.write_text(_EMPTY_CDML, encoding="utf-8")
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	link.symlink_to(source)
	window = _make_window(qapp)
	try:
		target = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
			_EDITABLE_CDML, "Untitled",
		)
		window._register_native_tab(target, activate=True)
		target = _author_dirty_atom(window)
		baseline = target.current_snapshot
		cancel = _click_visible_message_button(qapp, "Cancel")
		try:
			assert not _trigger_current_tab_open(window, incoming_path, monkeypatch)
		finally:
			cancel.stop()
		assert (
			_current_native_tab(window) is target
			and target.current_snapshot == baseline
		)
		acknowledge = _click_visible_message_button(qapp, "OK")
		try:
			assert not _wait_for_open_queue(
				window, lambda: window.open_in_current_tab_path(str(link)),
			)
		finally:
			acknowledge.stop()
		assert _current_native_tab(window) is target and target.current_snapshot == baseline
	finally:
		window.close()


#============================================
def test_open_in_current_tab_duplicate_hard_link_activates_existing_tab_and_keeps_target(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A duplicate descriptor activates its tab and leaves the explicit target untouched."""
	source = tmp_path / "single-origin.cdml"
	alias = tmp_path / "single-origin-alias.cdml"
	target_path = tmp_path / "target.cdml"
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	alias.hardlink_to(source)
	target_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	try:
		assert _wait_for_open_queue(window, lambda: window.open_file_path(str(source)))
		existing = _current_native_tab(window)
		assert _wait_for_open_queue(window, lambda: window.open_file_path(str(target_path)))
		target = _current_native_tab(window)
		baseline = target.current_snapshot
		assert _trigger_current_tab_open(window, alias, monkeypatch)
		assert (
			_current_native_tab(window) is existing
			and not target.is_disposed
			and target.current_snapshot == baseline
		)
	finally:
		window.close()

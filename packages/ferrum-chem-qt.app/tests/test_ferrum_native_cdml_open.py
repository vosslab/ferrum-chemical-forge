"""Behavior coverage for ordinary asynchronous local-CDML Open."""

# Standard Library
import collections.abc
import pathlib

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.native.ferrum_native_document_tab


_EMPTY_CDML = '<cdml version="1.0"/>'


#============================================
def _make_window(
		qapp: PySide6.QtWidgets.QApplication,
		) -> ferrum_qt.main_window.MainWindow:
	"""Create the ordinary OASA-free product window."""
	del qapp
	return ferrum_qt.main_window.MainWindow(object())


#============================================
def _open_action(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtGui.QAction:
	"""Find the public File Open action by its visible caption."""
	action = next(
		candidate
		for candidate in window.findChildren(PySide6.QtGui.QAction)
		if candidate.text() == "Open"
	)
	return action


#============================================
def _cancel_open_action(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtGui.QAction:
	"""Find the public cancellation action by its visible caption."""
	return next(
		candidate
		for candidate in window.findChildren(PySide6.QtGui.QAction)
		if candidate.text() == "Cancel Open"
	)


#============================================
def _current_native_tab(
		window: PySide6.QtWidgets.QMainWindow,
		) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
	"""Return the selected native page through the public central widget tree."""
	tab_widget = window.centralWidget()
	assert isinstance(tab_widget, PySide6.QtWidgets.QTabWidget)
	tab = tab_widget.currentWidget()
	assert isinstance(
		tab, ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
	)
	return tab


#============================================
def _wait_for_open_queue(
		window: ferrum_qt.main_window.MainWindow,
		start: collections.abc.Callable[[], object],
		) -> bool:
	"""Run the Qt event loop until one accepted Open batch drains."""
	loop = PySide6.QtCore.QEventLoop()
	outcomes: list[bool] = []

	def finish(success: bool) -> None:
		"""Capture the public batch outcome and stop the local event loop."""
		outcomes.append(success)
		loop.quit()

	window.local_cdml_open_queue_drained.connect(finish)
	start()
	loop.exec()
	window.local_cdml_open_queue_drained.disconnect(finish)
	return outcomes[0]


#============================================
def test_public_open_action_loads_saves_and_reopens_through_rust(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The visible Open action installs a clean native tab with a durable origin."""
	source = tmp_path / "ordinary-open.cdml"
	destination = tmp_path / "ordinary-open-copy.cdml"
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	initial_tab = _current_native_tab(window)
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getOpenFileName",
		lambda *_args: (str(source), "Ferrum CDML (*.cdml)"),
	)
	try:
		completed = _wait_for_open_queue(window, _open_action(window).trigger)
		tab = _current_native_tab(window)
		assert completed and tab is not initial_tab and tab.file_path == source
		assert not tab.current_snapshot.is_dirty and window.save_active_to_path(str(destination))
		prepared = ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(str(destination))
		reopened, observation = prepared.take_admission_v1()
		assert observation.document.snapshot.digest == reopened.snapshot().digest
	finally:
		window.close()


#============================================
def test_programmatic_open_queues_multiple_launch_documents(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""Multiple launch paths drain sequentially without blocking the Qt thread."""
	first = tmp_path / "first.cdml"
	second = tmp_path / "second.cdml"
	first.write_text(_EMPTY_CDML, encoding="utf-8")
	second.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)

	def start() -> None:
		"""Submit both launch paths before queued worker delivery can run."""
		assert window.open_file_path(str(first))
		assert window.open_file_path(str(second))

	try:
		completed = _wait_for_open_queue(window, start)
		origins = {
			tab.file_path
			for tab in window.findChildren(
				ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			)
			if tab.file_path is not None
		}
		assert completed
		assert origins == {first, second}
	finally:
		window.close()


#============================================
def test_duplicate_exact_origin_activates_the_existing_native_tab(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""A second exact path does not create another session authority."""
	source = tmp_path / "single-origin.cdml"
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	try:
		assert _wait_for_open_queue(window, lambda: window.open_file_path(str(source)))
		tab_widget = window.centralWidget()
		assert isinstance(tab_widget, PySide6.QtWidgets.QTabWidget)
		opened = _current_native_tab(window)
		pages = tuple(tab_widget.widget(index) for index in range(tab_widget.count()))
		assert window.open_file_path(str(source))
		assert tuple(tab_widget.widget(index) for index in range(tab_widget.count())) == pages
		assert _current_native_tab(window) is opened
		assert not window.has_pending_local_cdml_open()
	finally:
		window.close()


#============================================
def test_cancel_open_action_invalidates_delivery_without_replacing_the_tab(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""Cancellation is truthful delivery invalidation, not native-read preemption."""
	source = tmp_path / "cancelled.cdml"
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	initial_tab = _current_native_tab(window)
	initial_snapshot = initial_tab.current_snapshot

	def start_and_cancel() -> None:
		"""Cancel synchronously before queued worker delivery can reach the UI."""
		assert window.open_file_path(str(source))
		cancel = _cancel_open_action(window)
		assert cancel.isEnabled()
		cancel.trigger()

	try:
		completed = _wait_for_open_queue(window, start_and_cancel)
		assert not completed and _current_native_tab(window) is initial_tab
		assert initial_tab.current_snapshot == initial_snapshot
		assert _open_action(window).isEnabled() and not _cancel_open_action(window).isEnabled()
	finally:
		window.close()


#============================================
def test_symlink_rejection_leaves_the_current_document_unchanged(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Rust source policy rejects a symlink without replacing the active tab."""
	source = tmp_path / "source.cdml"
	link = tmp_path / "source-link.cdml"
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	link.symlink_to(source)
	warnings: list[tuple[str, str]] = []
	monkeypatch.setattr(
		PySide6.QtWidgets.QMessageBox,
		"warning",
		lambda _parent, title, message: warnings.append((title, message)),
	)
	window = _make_window(qapp)
	initial_tab = _current_native_tab(window)
	initial_snapshot = initial_tab.current_snapshot
	try:
		completed = _wait_for_open_queue(
			window, lambda: window.open_file_path(str(link)),
		)
		assert not completed and _current_native_tab(window) is initial_tab
		assert initial_tab.current_snapshot == initial_snapshot
		assert warnings and warnings[-1][0] == "CDML Source Rejected"
	finally:
		window.close()


#============================================
def test_admitted_tab_rejects_an_observation_from_a_different_session(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""The UI constructor authenticates the one-use Rust pair before projection."""
	del qapp
	first = tmp_path / "first-pair.cdml"
	second = tmp_path / "second-pair.cdml"
	first.write_text(_EMPTY_CDML, encoding="utf-8")
	second.write_text(
		'<cdml version="1.0"><plus id="p"><point x="3" y="4"/></plus></cdml>',
		encoding="utf-8",
	)
	first_session, _first_observation = (
		ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(str(first)).take_admission_v1()
	)
	_second_session, second_observation = (
		ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(str(second)).take_admission_v1()
	)
	with pytest.raises(
		ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTabError,
		match="does not match its admitted session",
	):
		(
			ferrum_qt.native.ferrum_native_document_tab.
			FerrumNativeDocumentTab.from_admitted_local_open(
				first_session, "mismatched.cdml", second_observation,
			)
		)

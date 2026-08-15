"""Behavior coverage for ordinary asynchronous local-CDML Open."""

# Standard Library
import collections.abc
import os
import pathlib

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.app
import ferrum_qt.config.preferences
import ferrum_qt.main_window
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_recent_files


_EMPTY_CDML = '<cdml version="1.0"/>'
_EDITABLE_CDML = """<cdml version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
</molecule></cdml>"""
_COORDINATE_CDML = """<cdml version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  <atom id='atom-o' name='O'><point x='50' y='20'/></atom>
  <bond id='bond-co' start='atom-c' end='atom-o' type='n1'/>
</molecule></cdml>"""


#============================================
def _make_window(
		qapp: PySide6.QtWidgets.QApplication,
		) -> ferrum_qt.main_window.MainWindow:
	"""Create the ordinary Rust-native product window."""
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
def _visible_action(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		) -> PySide6.QtGui.QAction:
	"""Return one user-visible native command by its stable label."""
	return next(
		candidate
		for candidate in window.findChildren(PySide6.QtGui.QAction)
		if candidate.text() == label
	)


#============================================
def _recent_menu(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtWidgets.QMenu:
	"""Return the visible File-menu Recent Files cascade."""
	menu = getattr(window, "_recent_files_menu", None)
	if not isinstance(menu, PySide6.QtWidgets.QMenu):
		raise RuntimeError("ordinary File menu did not expose Recent Files")
	return menu


#============================================
def _recent_paths() -> tuple[str, ...]:
	"""Read the current personal recent-display paths through QSettings."""
	prefs = ferrum_qt.config.preferences.Preferences.instance()
	value = prefs.value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES)
	if type(value) is not dict:
		return ()
	paths = value.get("paths")
	return tuple(paths) if type(paths) is list else ()


#============================================
def _restore_recent_paths(value: object) -> None:
	"""Restore the one application preference owned by these tests."""
	prefs = ferrum_qt.config.preferences.Preferences.instance()
	prefs.set_value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES, value)


#============================================
def _click_visible_message_button(
		qapp: PySide6.QtWidgets.QApplication, label: str,
		) -> PySide6.QtCore.QTimer:
	"""Click one explicitly named button on the visible native recovery dialog."""
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
def _wait_for_action_enabled(action: PySide6.QtGui.QAction) -> None:
	"""Let one real asynchronous product operation restore its public command."""
	if action.isEnabled():
		return
	loop = PySide6.QtCore.QEventLoop()

	def finish() -> None:
		"""Leave once the visible command becomes usable again."""
		if action.isEnabled():
			loop.quit()

	action.changed.connect(finish)
	try:
		loop.exec()
	finally:
		action.changed.disconnect(finish)


#============================================
def test_cdxml_open_refusal_preserves_the_active_native_document(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A dropped ChemDraw XML request gives recovery guidance without mutation."""
	window = _make_window(qapp)
	tab = _current_native_tab(window)
	before = tab.current_snapshot
	warnings: list[tuple[str, str]] = []
	monkeypatch.setattr(
		window, "_show_native_file_warning",
		lambda title, message: warnings.append((title, message)),
	)
	try:
		assert not window.open_file_path(str(tmp_path / "drawing.cdxml"))
		assert (
			_current_native_tab(window) is tab
			and tab.current_snapshot is before
			and warnings
			and "converter" in warnings[-1][1].lower()
			and ".cdml" in warnings[-1][1].lower()
		)
	finally:
		window.close()


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
		assert (
			completed
			and tab.file_path == source
			and initial_tab.is_disposed
		)
		assert not tab.current_snapshot.is_dirty and window.save_active_to_path(str(destination))
		prepared = ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(str(destination))
		reopened, observation, _origin, _source_kind = prepared.take_admission_v1()
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
	initial_tab = _current_native_tab(window)

	def start() -> None:
		"""Submit both launch paths before queued worker delivery can run."""
		assert ferrum_qt.app._open_launch_files(window, [str(first), str(second)]) == 2

	try:
		completed = _wait_for_open_queue(window, start)
		origins = {
			tab.file_path
			for tab in window.findChildren(
				ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			)
			if tab.file_path is not None
		}
		assert completed and origins == {first, second}
		assert not initial_tab.is_disposed and initial_tab.file_path is None
	finally:
		window.close()


#============================================
def test_hard_link_alias_activates_the_existing_native_tab(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""A descriptor-identical alias activates the existing native document."""
	source = tmp_path / "single-origin.cdml"
	alias = tmp_path / "single-origin-alias.cdml"
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	alias.hardlink_to(source)
	window = _make_window(qapp)
	try:
		assert _wait_for_open_queue(window, lambda: window.open_file_path(str(source)))
		opened = _current_native_tab(window)
		assert _wait_for_open_queue(window, lambda: window.open_file_path(str(alias)))
		assert _current_native_tab(window) is opened and opened.file_path == source
	finally:
		window.close()


#============================================
def test_interactive_open_preserves_a_loaded_document_in_a_new_tab(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Only the bootstrap page is disposable to the visible Open command."""
	first = tmp_path / "loaded.cdml"
	second = tmp_path / "later.cdml"
	first.write_text(
		'<cdml version="1.0"><plus id="p"><point x="3" y="4"/></plus></cdml>',
		encoding="utf-8",
	)
	second.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	paths = iter((str(first), str(second)))
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getOpenFileName",
		lambda *_args: (next(paths), "Ferrum CDML (*.cdml)"),
	)
	try:
		assert _wait_for_open_queue(window, _open_action(window).trigger)
		loaded = _current_native_tab(window)
		assert loaded.file_path == first and not loaded.is_disposed
		assert _wait_for_open_queue(window, _open_action(window).trigger)
		assert not loaded.is_disposed
		assert _current_native_tab(window).file_path == second
	finally:
		window.close()


#============================================
def test_interactive_open_preserves_an_armed_bootstrap_canvas_gesture(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""An armed canvas preview fences first Open into a separate document."""
	source = tmp_path / "opened-while-armed.cdml"
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	bootstrap = _current_native_tab(window)
	baseline = bootstrap.current_snapshot
	try:
		window.show()
		qapp.processEvents()
		_visible_action(window, "Insert Cyclohexane Ring").trigger()
		PySide6.QtTest.QTest.mousePress(
			bootstrap.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			PySide6.QtCore.QPoint(80, 80),
		)
		qapp.processEvents()
		assert _wait_for_open_queue(
			window, lambda: window.open_file_path(str(source), interactive=True),
		)
		admitted = next(
			tab
			for tab in window.findChildren(
				ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			)
			if tab.file_path == source
		)
		assert (
			_current_native_tab(window) is bootstrap
			and not bootstrap.is_disposed
			and bootstrap.current_snapshot == baseline
			and not admitted.is_disposed
		)
		PySide6.QtTest.QTest.keyClick(
			bootstrap.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape,
		)
		assert _current_native_tab(window) is bootstrap and bootstrap.current_snapshot == baseline
	finally:
		window.close()


#============================================
def test_open_in_current_tab_recovers_after_an_armed_ring_preview(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The explicit command waits for a visible active tool, then works after Escape."""
	source = tmp_path / "opened-after-escape.cdml"
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	bootstrap = _current_native_tab(window)
	baseline = bootstrap.current_snapshot
	chosen: list[str] = []

	def choose_source(*_args: object) -> tuple[str, str]:
		"""Record one real chooser request and return the selected local source."""
		chosen.append(str(source))
		return str(source), "Ferrum CDML (*.cdml)"

	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getOpenFileName",
		choose_source,
	)
	try:
		window.show()
		qapp.processEvents()
		_visible_action(window, "Insert Cyclohexane Ring").trigger()
		PySide6.QtTest.QTest.mousePress(
			bootstrap.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			PySide6.QtCore.QPoint(80, 80),
		)
		qapp.processEvents()
		current_tab_open = _visible_action(window, "Open in Current Tab...")
		assert not current_tab_open.isEnabled()
		current_tab_open.trigger()
		assert not chosen and _current_native_tab(window) is bootstrap
		assert bootstrap.current_snapshot == baseline
		PySide6.QtTest.QTest.keyClick(
			bootstrap.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape,
		)
		qapp.processEvents()
		assert current_tab_open.isEnabled()
		assert _wait_for_open_queue(window, current_tab_open.trigger)
		assert _current_native_tab(window).file_path == source
	finally:
		window.close()


#============================================
def test_open_in_current_tab_waits_for_real_coordinate_generation(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A target-owned native worker fences replacement until its terminal refresh."""
	target_path = tmp_path / "coordinates.cdml"
	incoming_path = tmp_path / "replacement.cdml"
	target_path.write_text(_COORDINATE_CDML, encoding="utf-8")
	incoming_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	chosen: list[str] = []

	def choose_source(*_args: object) -> tuple[str, str]:
		"""Record whether the unavailable command ever reaches its chooser."""
		chosen.append(str(incoming_path))
		return str(incoming_path), "Ferrum CDML (*.cdml)"

	monkeypatch.setattr(PySide6.QtWidgets.QFileDialog, "getOpenFileName", choose_source)
	try:
		target = _open_saved_native_tab(qapp, window, target_path)
		baseline = target.current_snapshot
		_visible_action(window, "Generate Molecule Coordinates").trigger()
		current_tab_open = _visible_action(window, "Open in Current Tab...")
		assert not current_tab_open.isEnabled()
		current_tab_open.trigger()
		assert not chosen and _current_native_tab(window) is target
		assert target.current_snapshot == baseline
		_wait_for_action_enabled(current_tab_open)
		assert current_tab_open.isEnabled() and _current_native_tab(window) is target
		click = _click_visible_message_button(qapp, "Replace")
		try:
			assert _wait_for_open_queue(window, current_tab_open.trigger)
		finally:
			click.stop()
		assert _current_native_tab(window).file_path == incoming_path
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
	first_session, _first_observation, _first_origin, _first_source_kind = (
		ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(str(first)).take_admission_v1()
	)
	_second_session, second_observation, _second_origin, _second_source_kind = (
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


#============================================
def test_confirmed_native_open_and_save_promote_personal_recent_paths(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""Confirmed native ingress and publication update only personal recency."""
	source = tmp_path / "opened.cdml"
	destination = tmp_path / "saved.cdml"
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	prefs = ferrum_qt.config.preferences.Preferences.instance()
	previous = prefs.value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES)
	prefs.set_value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES, {})
	window = _make_window(qapp)
	try:
		assert _wait_for_open_queue(window, lambda: window.open_file_path(str(source)))
		tab = _current_native_tab(window)
		baseline_cdml = tab.current_snapshot.cdml
		assert window.save_active_to_path(str(destination))
		assert (
			_recent_paths() == (str(destination), str(source))
			and tab.current_snapshot.cdml == baseline_cdml
			and not tab.current_snapshot.is_dirty
		)
	finally:
		window.close()
		_restore_recent_paths(previous)


#============================================
def test_recent_model_promotes_normalized_paths_with_injected_capacity(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""The personal MRU promotes duplicates and sheds old entries at any capacity."""
	first = tmp_path / "first.cdml"
	second = tmp_path / "second.cdml"
	third = tmp_path / "third.cdml"
	prefs = ferrum_qt.config.preferences.Preferences.instance()
	previous = prefs.value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES)
	prefs.set_value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES, {})
	window = _make_window(qapp)
	model = ferrum_qt.native.ferrum_native_recent_files.FerrumNativeRecentFiles(
		window, prefs, capacity=2,
	)
	try:
		model.record_confirmed_path(os.path.relpath(first))
		model.record_confirmed_path(second)
		model.record_confirmed_path(first)
		model.record_confirmed_path(third)
		assert _recent_paths() == (str(third), str(first))
	finally:
		window.close()
		_restore_recent_paths(previous)


#============================================
def test_recent_file_action_uses_native_new_tab_route_and_origin_identity(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""A visible recent selection uses ordinary native admission and token reuse."""
	source = tmp_path / "recent.cdml"
	alias = tmp_path / "recent-alias.cdml"
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	alias.hardlink_to(source)
	prefs = ferrum_qt.config.preferences.Preferences.instance()
	previous = prefs.value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES)
	prefs.set_value(
		ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES,
		ferrum_qt.native.ferrum_native_recent_files.FerrumNativeRecentFilesV1(
			(str(alias),),
		).to_settings_value(),
	)
	window = _make_window(qapp)
	try:
		bootstrap = _current_native_tab(window)
		recent_action = next(
			action for action in _recent_menu(window).actions()
			if action.text() == alias.name
		)
		assert _wait_for_open_queue(window, recent_action.trigger)
		opened = _current_native_tab(window)
		repeat_action = next(
			action for action in _recent_menu(window).actions()
			if action.text() == alias.name
		)
		assert _wait_for_open_queue(window, repeat_action.trigger)
		assert (
			_current_native_tab(window) is opened
			and opened.file_path == alias
			and not bootstrap.is_disposed
		)
	finally:
		window.close()
		_restore_recent_paths(previous)


#============================================
def test_recent_menu_disambiguates_names_and_clear_keeps_document_state(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""Recent labels distinguish collisions, while Clear leaves the Rust page alone."""
	first = tmp_path / "first" / "shared.cdml"
	second = tmp_path / "second" / "shared.cdml"
	first.parent.mkdir()
	second.parent.mkdir()
	prefs = ferrum_qt.config.preferences.Preferences.instance()
	previous = prefs.value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES)
	prefs.set_value(
		ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES,
		ferrum_qt.native.ferrum_native_recent_files.FerrumNativeRecentFilesV1(
			(str(second), str(first)),
		).to_settings_value(),
	)
	window = _make_window(qapp)
	try:
		baseline = _current_native_tab(window).current_snapshot.cdml
		labels = {
			action.text() for action in _recent_menu(window).actions()
			if not action.isSeparator()
		}
		assert labels.issuperset({"shared.cdml \N{EM DASH} first", "shared.cdml \N{EM DASH} second"})
		clear = next(
			action for action in _recent_menu(window).actions()
			if action.text() == "Clear Recent Files"
		)
		clear.trigger()
		assert _recent_paths() == () and _current_native_tab(window).current_snapshot.cdml == baseline
	finally:
		window.close()
		_restore_recent_paths(previous)


#============================================
def test_recent_missing_file_visible_keep_and_remove_recovery(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""Visible Keep and Remove recover one stale path without changing the Rust page."""
	valid = tmp_path / "available.cdml"
	missing = tmp_path / "missing.cdml"
	valid.write_text(_EMPTY_CDML, encoding="utf-8")
	prefs = ferrum_qt.config.preferences.Preferences.instance()
	previous = prefs.value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES)
	prefs.set_value(
		ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES,
		ferrum_qt.native.ferrum_native_recent_files.FerrumNativeRecentFilesV1(
			(str(missing), str(valid)),
		).to_settings_value(),
	)
	window = _make_window(qapp)
	try:
		window.show()
		qapp.processEvents()
		bootstrap = _current_native_tab(window)
		baseline = bootstrap.current_snapshot
		missing_action = next(
			action for action in _recent_menu(window).actions()
			if action.text() == missing.name
		)
		keep = _click_visible_message_button(qapp, "Keep")
		assert not _wait_for_open_queue(window, missing_action.trigger)
		keep.stop()
		assert _recent_paths() == (str(missing), str(valid)) and _current_native_tab(window) is bootstrap
		remove = _click_visible_message_button(qapp, "Remove from Recent Files")
		assert not _wait_for_open_queue(window, missing_action.trigger)
		remove.stop()
		assert _recent_paths() == (str(valid),) and bootstrap.current_snapshot == baseline
		valid_action = next(
			action for action in _recent_menu(window).actions()
			if action.text() == valid.name
		)
		assert _wait_for_open_queue(window, valid_action.trigger)
		assert _recent_paths() == (str(valid),) and _current_native_tab(window).file_path == valid
	finally:
		window.close()
		_restore_recent_paths(previous)


#============================================
def _open_saved_native_tab(
		qapp: PySide6.QtWidgets.QApplication, window: ferrum_qt.main_window.MainWindow,
		path: pathlib.Path,
		) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
	"""Load one real saved document through the ordinary native ingress."""
	assert _wait_for_open_queue(window, lambda: window.open_file_path(str(path)))
	tab = _current_native_tab(window)
	assert tab.file_path == path and not tab.current_snapshot.is_dirty
	return tab


#============================================
def _author_dirty_atom(
		window: ferrum_qt.main_window.MainWindow,
		element: str = "N",
		) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
	"""Make one authoritative atom change on an otherwise idle native tab."""
	tab = _current_native_tab(window)
	tab.select_atom("atom-c")
	tab.change_selected_atom_element(element)
	assert tab.current_snapshot.is_dirty
	return tab


#============================================
def _trigger_current_tab_open(
		window: ferrum_qt.main_window.MainWindow, path: pathlib.Path,
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
		'<cdml version="1.0"><plus id="incoming"><point x="3" y="4"/></plus></cdml>',
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
		target = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
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
		target = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
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

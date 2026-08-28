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
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.operation_leases
import ferrum_qt.ferrum.recent_files
import ferrum_qt.ferrum.window_refusals


_EMPTY_CDML = '<cdml xmlns="urn:ferrum:cdml" version="1.0"/>'
_EDITABLE_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
</molecule></cdml>"""
_SIMPLE_CML = (
	'<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule id="cml-molecule">'
	'<atomArray><atom id="cml-carbon" elementType="C" x2="0" y2="0"/>'
	'</atomArray></molecule></cml>'
)
_SIMPLE_CDXML = (
	'<CDXML><page><fragment id="fragment-1"><n id="carbon" p="0 0"/>'
	'<n id="oxygen" p="20 0" Element="8"/><b B="carbon" E="oxygen"/>'
	'</fragment></page></CDXML>'
)
_UNSUPPORTED_CDXML = (
	'<CDXML><page><fragment id="fragment-1"><n id="carbon" p="0 0"/>'
	'<n id="oxygen" p="20 0" Element="8"/><b B="carbon" E="oxygen" Order="4"/>'
	'</fragment></page></CDXML>'
)
_TWO_RECORD_SDF = """ethanol
  Ferrum

  3  2  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    1.5000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    3.0000    0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
  1  2  1  0
  2  3  1  0
M  END
$$$$
water
  Ferrum

  1  0  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
M  END
$$$$
"""


#============================================
def _make_window(
		qapp: PySide6.QtWidgets.QApplication,
		) -> ferrum_qt.main_window.MainWindow:
	"""Create the ordinary Ferrum product window."""
	return ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)


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
	"""Return one user-visible Ferrum command by its stable label."""
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
def _local_open_handle(suffix: str) -> object:
	"""Return the extension-issued File/Open handle accepting ``suffix``."""
	for descriptor in ferrum_chem.DocumentSession.local_document_open_descriptors_v2():
		if suffix in descriptor.suffixes:
			return descriptor.route_handle
	raise AssertionError(f"Ferrum did not publish a File/Open route for {suffix}")


#============================================
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
def _close_test_window(
		qapp: PySide6.QtWidgets.QApplication,
		window: ferrum_qt.main_window.MainWindow,
		) -> None:
	"""Discard every exact test tab before ordinary window shutdown can prompt."""
	for tab in tuple(window._native_tabs_by_page.values()):
		index = window._tab_widget.indexOf(tab)
		result = window._close_native_tab_at(
			index, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		)
		assert result is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
	assert not window._native_tabs_by_page
	window.close()
	window.deleteLater()
	qapp.processEvents()


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
#============================================
def test_visible_open_actions_pass_distinct_interchange_and_current_tab_filters(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Visible File actions distinguish new-document and replacement inputs."""
	window = _make_window(qapp)
	captured_filters: list[str] = []

	def capture_filter(*args: object) -> tuple[str, str]:
		"""Record one visible dialog's exact filter and cancel locally."""
		captured_filters.append(str(args[-1]))
		return "", ""

	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog, "getOpenFileName", capture_filter,
	)
	try:
		_open_action(window).trigger()
		_visible_action(window, "Open in Current Tab...").trigger()
		assert len(captured_filters) == 2
		new_document_filter, current_tab_filter = captured_filters
		assert all(suffix in new_document_filter for suffix in ("*.cdxml", "*.sdf", "*.sd"))
		assert all(suffix not in current_tab_filter for suffix in ("*.cdxml", "*.sdf", "*.sd"))
	finally:
		_close_test_window(qapp, window)


#============================================
def test_cdxml_open_installs_a_new_tab_with_durable_molecule_facts(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Visible File/Open routes producer-style CDXML through the Rust descriptor."""
	source = tmp_path / "producer.cdxml"
	source.write_text(_SIMPLE_CDXML, encoding="utf-8")
	window = _make_window(qapp)
	bootstrap = _current_native_tab(window)
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog, "getOpenFileName",
		lambda *_args: (str(source), "ChemDraw XML (*.cdxml)"),
	)
	try:
		assert _wait_for_open_queue(window, _open_action(window).trigger)
		tab = _current_native_tab(window)
		molecule = tab.current_document_observation().projection.molecules[0]
		projection = tab._require_projection()
		assert (
			tab is not bootstrap
			and not bootstrap.is_disposed
			and tab.file_path is None
			and tuple(atom.element for atom in molecule.atoms) == ("C", "O")
			and len(molecule.bonds) == 1
		)
		content_bounds = tab.document_content_bounds()
		assert content_bounds is not None
		root = projection.molecule_roots[0]
		assert (
			root.document_object_id == molecule.document_object_id
			and content_bounds == root.sceneBoundingRect()
			and all(item.parentItem() is root for item in projection.items)
		)
	finally:
		_close_test_window(qapp, window)


#============================================
def test_cdxml_open_refusal_preserves_the_active_native_document(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Unsupported CDXML returns the typed refusal without installing a new tab."""
	source = tmp_path / "unsupported.cdxml"
	source.write_text(_UNSUPPORTED_CDXML, encoding="utf-8")
	window = _make_window(qapp)
	tab = _current_native_tab(window)
	before = tab.current_snapshot
	warnings: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []
	monkeypatch.setattr(window, "_show_edit_refusal", warnings.append)
	try:
		assert not _wait_for_open_queue(window, lambda: window.open_file_path(str(source)))
		assert _current_native_tab(window) is tab
		assert tab.current_snapshot is before
		assert len(warnings) == 1
		refusal = warnings[-1]
		assert refusal.outcome.value == "invalid_document"
		assert refusal.context.value == "open_document"
		assert str(source) not in ferrum_qt.dialogs.refusal_presenter.present_refusal(
			refusal,
		).ordinary_text()
	finally:
		_close_test_window(qapp, window)


#============================================
def test_cml_open_keeps_import_provenance_and_saves_authoritative_cdml(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Visible File/Open keeps its initial tab while CML imports into a new tab."""
	source = tmp_path / "imported.cml"
	destination = tmp_path / "imported.cdml"
	source.write_text(_SIMPLE_CML, encoding="utf-8")
	window = _make_window(qapp)
	bootstrap = _current_native_tab(window)
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getOpenFileName",
		lambda *_args: (str(source), "CML (*.cml)"),
	)
	try:
		assert _wait_for_open_queue(window, _open_action(window).trigger)
		tab = _current_native_tab(window)
		assert (
			not bootstrap.is_disposed
			and tab is not bootstrap
			and tab.file_path is None
		)
		assert window.save_active_to_path(str(destination))
		assert tab.file_path == destination
		reopened, _observation, _origin, source_kind, _summary = (
			ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
				str(destination), _local_open_handle(".cdml"),
			).take_admission_v2()
		)
		assert source_kind == "cdml" and reopened.snapshot().digest == tab.current_snapshot.digest
	finally:
		_close_test_window(qapp, window)


#============================================
def test_sdf_file_open_creates_one_new_tab_with_source_order_and_durable_cdml(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""File/Open admits ordered SDF records as one new document, never insertion."""
	source = tmp_path / "two-records.sd"
	destination = tmp_path / "two-records.cdml"
	source.write_text(_TWO_RECORD_SDF, encoding="utf-8")
	window = _make_window(qapp)
	bootstrap = _current_native_tab(window)
	warnings: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []
	monkeypatch.setattr(window, "_show_edit_refusal", warnings.append)
	try:
		assert _wait_for_open_queue(window, lambda: window.open_file_path(str(source))), warnings
		tab = _current_native_tab(window)
		assert (
			tab is not bootstrap
			and not bootstrap.is_disposed
			and not warnings
			and tuple(molecule.name for molecule in tab.current_document_observation().projection.molecules)
			== ("ethanol", "water")
		)
		assert window.save_active_to_path(str(destination))
		reopened, _observation, _origin, source_kind, _summary = (
			ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
				str(destination), _local_open_handle(".cdml"),
			).take_admission_v2()
		)
		assert source_kind == "cdml" and reopened.snapshot().digest == tab.current_snapshot.digest
	finally:
		_close_test_window(qapp, window)


#============================================
def test_sdf_file_open_refusal_preserves_the_active_document(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A failed SDF Open cannot publish a tab or mutate the active drawing."""
	source = tmp_path / "invalid.sdf"
	source.write_text("not an SD file", encoding="utf-8")
	window = _make_window(qapp)
	tab = _current_native_tab(window)
	before = tab.current_snapshot
	warnings: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []
	monkeypatch.setattr(window, "_show_edit_refusal", warnings.append)
	try:
		assert not _wait_for_open_queue(window, lambda: window.open_file_path(str(source)))
		assert _current_native_tab(window) is tab and tab.current_snapshot is before and warnings
	finally:
		_close_test_window(qapp, window)


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
				ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			)
			if tab.file_path is not None
		}
		assert completed and origins == {first, second}
		assert not initial_tab.is_disposed and initial_tab.file_path is None
	finally:
		_close_test_window(qapp, window)


#============================================
def test_hard_link_alias_activates_the_existing_native_tab(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""A descriptor-identical alias activates the existing Ferrum document."""
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
		_close_test_window(qapp, window)


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
		'<cdml xmlns="urn:ferrum:cdml" version="1.0"><plus id="p"><point x="3" y="4"/></plus></cdml>',
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
		_close_test_window(qapp, window)


#============================================
def test_interactive_open_cancels_an_armed_bootstrap_canvas_gesture(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""Open cancels its tool and admits its source into a separate document."""
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
		assert _visible_action(window, "Cancel Tool").isEnabled()
		assert _wait_for_open_queue(
			window, lambda: window.open_file_path(str(source), interactive=True),
		)
		admitted = next(
			tab
			for tab in window.findChildren(
				ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			)
			if tab.file_path == source
		)
		assert (
			_current_native_tab(window) is bootstrap
			and not bootstrap.is_disposed
			and bootstrap.current_snapshot == baseline
			and not admitted.is_disposed
		)
		assert not _visible_action(window, "Cancel Tool").isEnabled()
	finally:
		_close_test_window(qapp, window)


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
		_close_test_window(qapp, window)


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
		_close_test_window(qapp, window)


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
	refusals: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []
	monkeypatch.setattr(
		ferrum_qt.ferrum.window_refusals,
		"show_refusal",
		lambda _window, request: refusals.append(request),
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
		assert refusals
		presentation = ferrum_qt.dialogs.refusal_presenter.present_refusal(refusals[-1])
		assert presentation.title == "Cannot Open This File"
		assert presentation.technical_details is not None
		assert "non-symlink" in presentation.technical_details.lower()
	finally:
		_close_test_window(qapp, window)


#============================================
def test_confirmed_native_open_and_save_promote_personal_recent_paths(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""Confirmed Ferrum ingress and publication update only personal recency."""
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
		_close_test_window(qapp, window)
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
	model = ferrum_qt.ferrum.recent_files.FerrumNativeRecentFiles(
		window, prefs, capacity=2,
	)
	try:
		model.record_confirmed_path(os.path.relpath(first))
		model.record_confirmed_path(second)
		model.record_confirmed_path(first)
		model.record_confirmed_path(third)
		assert _recent_paths() == (str(third), str(first))
	finally:
		_close_test_window(qapp, window)
		_restore_recent_paths(previous)


#============================================
def test_recent_file_action_uses_native_new_tab_route_and_origin_identity(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""A visible recent selection uses ordinary Ferrum admission and token reuse."""
	source = tmp_path / "recent.cdml"
	alias = tmp_path / "recent-alias.cdml"
	source.write_text(_EMPTY_CDML, encoding="utf-8")
	alias.hardlink_to(source)
	prefs = ferrum_qt.config.preferences.Preferences.instance()
	previous = prefs.value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES)
	prefs.set_value(
		ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES,
		ferrum_qt.ferrum.recent_files.FerrumNativeRecentFilesV1(
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
		_close_test_window(qapp, window)
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
		ferrum_qt.ferrum.recent_files.FerrumNativeRecentFilesV1(
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
		_close_test_window(qapp, window)
		_restore_recent_paths(previous)


#============================================

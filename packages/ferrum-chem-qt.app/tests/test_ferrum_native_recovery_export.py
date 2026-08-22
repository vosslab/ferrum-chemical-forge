"""Behavior coverage for Ferrum recovery export without Save presentation effects."""

# Standard Library
import os
import pathlib
import types


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import ferrum_chem
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.coordinate_generation
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.window_refusals


_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
</molecule></cdml>"""


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide the existing offscreen Qt application seam."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _window_with_tab() -> tuple[
		ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		]:
	"""Create one public Ferrum window with an active dirty document tab."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "Untitled")
	window._register_native_tab(tab, activate=True)
	return window, tab


#============================================
def _tab_facts(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[object, ...]:
	"""Capture externally observable facts that Recovery Export must preserve."""
	snapshot = tab.current_snapshot
	facts = (
		tab.file_path,
		tab.title,
		snapshot.revision,
		snapshot.digest,
		tab.is_dirty,
		tab.requires_refresh,
		tab.view.scene(),
	)
	return facts


#============================================
def _dispose_window(
		window: ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		) -> None:
	"""Retire test tabs without invoking the production dirty-close confirmation."""
	for tab in tuple(window._native_tabs_by_page.values()):
		window._tab_widget.removeTab(window._tab_widget.indexOf(tab))
		tab.dispose()
	window.deleteLater()


#============================================
def _suppress_refusals(monkeypatch: pytest.MonkeyPatch) -> None:
	"""Keep expected refusal paths nonmodal in offscreen tests."""
	monkeypatch.setattr(
		ferrum_qt.ferrum.window_refusals, "show_refusal",
		lambda _window, _request: None,
	)


#============================================
def test_recovery_export_writes_current_backend_without_save_effects(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A confirmed recovery copy preserves the live tab while writing exact CDML."""
	window, tab = _window_with_tab()
	tab.select_atom("atom-c")
	tab.change_selected_atom_element("N")
	before = _tab_facts(tab)
	destination = tmp_path / "recovery"
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog, "getSaveFileName", lambda *_args: (str(destination), ""),
	)
	try:
		window._recovery_export_action.trigger()
		assert destination.with_suffix(".cdml").read_text(encoding="utf-8") == (
			tab.backend_snapshot_for_recovery_export().cdml
		)
		assert _tab_facts(tab) == before
	finally:
		_dispose_window(window)


#============================================
def test_recovery_export_remains_reachable_for_pending_or_busy_native_state(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The public action stays available when display or unrelated work is pending."""
	window, tab = _window_with_tab()
	snapshot = tab.backend_snapshot_for_recovery_export()
	worker = types.SimpleNamespace(delivery_cancelled=False)
	intent_type = (
		ferrum_qt.ferrum.coordinate_generation.FerrumNativeCoordinateGenerationIntent
	)
	intent = intent_type(
		tab, snapshot.revision, snapshot.digest, worker,
	)
	window._coordinate_generation_intent = intent
	window._refresh_actions()
	destination = tmp_path / "busy.cdml"
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog, "getSaveFileName", lambda *_args: (str(destination), ""),
	)
	try:
		window._recovery_export_action.trigger()
		assert window._recovery_export_action.isEnabled() and destination.exists()
		assert window._coordinate_generation_intent is intent and not worker.delivery_cancelled
	finally:
		_dispose_window(window)


#============================================
def test_recovery_export_refuses_a_tab_switch_after_destination_dialog(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The post-dialog identity fence refuses to publish a different active tab."""
	window, first = _window_with_tab()
	second = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "Second")
	window._register_native_tab(second, activate=False)
	destination = tmp_path / "should-not-exist.cdml"

	def choose_path(*_args: object) -> tuple[str, str]:
		"""Change the selected tab at the deterministic dialog seam."""
		window._tab_widget.setCurrentWidget(second)
		return str(destination), ""

	monkeypatch.setattr(PySide6.QtWidgets.QFileDialog, "getSaveFileName", choose_path)
	_suppress_refusals(monkeypatch)
	try:
		assert not window._on_native_recovery_export() and not destination.exists()
		assert first is not second and first.file_path is None and second.file_path is None
	finally:
		_dispose_window(window)


#============================================
def test_recovery_export_refuses_closed_or_changed_backend_after_destination_dialog(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Close/removal and a changed Rust revision each fail the post-dialog fence."""
	for mutate in ("close", "revision"):
		window, tab = _window_with_tab()
		destination = tmp_path / f"{mutate}.cdml"

		def choose_path(*_args: object) -> tuple[str, str]:
			"""Apply one deterministic mutation while the file dialog is open."""
			if mutate == "close":
				window._tab_widget.removeTab(window._tab_widget.indexOf(tab))
				window._native_tabs_by_page.pop(tab)
				tab.dispose()
			else:
				tab.select_atom("atom-c")
				tab.change_selected_atom_element("N")
			return str(destination), ""

		monkeypatch.setattr(PySide6.QtWidgets.QFileDialog, "getSaveFileName", choose_path)
		_suppress_refusals(monkeypatch)
		try:
			assert not window._on_native_recovery_export() and not destination.exists()
		finally:
			_dispose_window(window)


#============================================
def test_recovery_export_rejects_non_cdml_destination_without_adopting_it(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A different suffix is rejected before publication and leaves the tab unchanged."""
	window, tab = _window_with_tab()
	before = _tab_facts(tab)
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getSaveFileName",
		lambda *_args: (str(tmp_path / "recovery.svg"), ""),
	)
	_suppress_refusals(monkeypatch)
	try:
		assert not window._on_native_recovery_export()
		assert _tab_facts(tab) == before
	finally:
		_dispose_window(window)


#============================================
def test_recovery_export_uses_newer_pending_backend_not_old_installed_projection(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A failed display replacement leaves its newer Rust snapshot recoverable."""
	window, tab = _window_with_tab()
	tab.select_atom("atom-c")
	monkeypatch.setattr(tab._controller, "replace", lambda *_args: False)
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab.change_selected_atom_element("N")
	destination = tmp_path / "pending.cdml"
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog, "getSaveFileName", lambda *_args: (str(destination), ""),
	)
	try:
		assert window._on_native_recovery_export()
		assert (
			tab.requires_refresh
			and destination.read_text() == tab.backend_snapshot_for_recovery_export().cdml
		)
	finally:
		_dispose_window(window)


#============================================
def test_recovery_export_refuses_mismatched_normal_receipt_provenance(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A normal receipt must corroborate both snapshots before any success message."""
	window, tab = _window_with_tab()
	capture = tab.backend_snapshot_for_recovery_export()
	for field_name in ("published_snapshot", "snapshot"):
		mismatch = types.SimpleNamespace(revision=capture.revision, digest="0" * 64)
		receipt = types.SimpleNamespace(
			published_snapshot=mismatch if field_name == "published_snapshot" else capture,
			snapshot=mismatch if field_name == "snapshot" else capture,
			outcome=types.SimpleNamespace(is_confirmed=True),
		)
		warnings = []
		monkeypatch.setattr(tab, "recovery_export", lambda *_args: receipt)
		monkeypatch.setattr(
			PySide6.QtWidgets.QFileDialog,
			"getSaveFileName",
			lambda *_args: (str(tmp_path / field_name), ""),
		)
		monkeypatch.setattr(
			window, "_show_edit_refusal", lambda request: warnings.append(request),
		)
		assert not window._on_native_recovery_export()
		assert warnings[-1].outcome.value == "unavailable_operation" and "may already" in warnings[-1].technical_details or ""
	_dispose_window(window)


#============================================
def test_recovery_action_requires_a_live_registered_native_page(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Zero pages are disabled while either clean or dirty live Ferrum page is enabled."""
	window, tab = _window_with_tab()
	window._tab_widget.removeTab(window._tab_widget.indexOf(tab))
	window._native_tabs_by_page.pop(tab)
	window._refresh_actions()
	no_page = not window._recovery_export_action.isEnabled()
	window._register_native_tab(tab, activate=True)
	window._refresh_actions()
	try:
		assert no_page and window._recovery_export_action.isEnabled()
	finally:
		_dispose_window(window)


#============================================
def test_recovery_action_rejects_non_native_and_disposed_current_pages(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Only a live registered Ferrum widget may enable the recovery action."""
	window, tab = _window_with_tab()
	non_native = PySide6.QtWidgets.QWidget()
	window._tab_widget.addTab(non_native, "Other")
	window._tab_widget.setCurrentWidget(non_native)
	window._refresh_actions()
	non_native_disabled = not window._recovery_export_action.isEnabled()
	window._tab_widget.setCurrentWidget(tab)
	tab.dispose()
	window._refresh_actions()
	try:
		assert non_native_disabled and not window._recovery_export_action.isEnabled()
	finally:
		window._tab_widget.removeTab(window._tab_widget.indexOf(non_native))
		window._native_tabs_by_page.pop(tab)
		window._tab_widget.removeTab(window._tab_widget.indexOf(tab))
		window.deleteLater()


#============================================
def test_recovery_export_unconfirmed_matching_receipt_preserves_tab(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A fully corroborated unconfirmed receipt warns without Save side effects."""
	window, tab = _window_with_tab()
	capture = tab.backend_snapshot_for_recovery_export()
	before = _tab_facts(tab)
	receipt = types.SimpleNamespace(
		published_snapshot=capture,
		snapshot=capture,
		outcome=types.SimpleNamespace(is_confirmed=False),
	)
	warnings = []
	monkeypatch.setattr(tab, "recovery_export", lambda *_args: receipt)
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog, "getSaveFileName", lambda *_args: (str(tmp_path / "x"), ""),
	)
	monkeypatch.setattr(
		window, "_show_edit_refusal", lambda request: warnings.append(request),
	)
	try:
		assert not window._on_native_recovery_export()
		assert _tab_facts(tab) == before and warnings[-1].outcome.value == "unavailable_operation"
	finally:
		_dispose_window(window)


#============================================
def test_recovery_export_error_messages_preserve_tab_facts(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Typed Rust publication failures retain distinct actionable recovery wording."""
	for error_type, expected_title, expected_text in (
		(ferrum_chem.PublicationPossiblyCompletedError, "Recovery Export Possibly Completed", "Verify"),
		(ferrum_chem.PublicationNotStartedError, "Recovery Export Not Started", "did not start"),
		(ferrum_chem.InvalidDestinationError, "Recovery Export Destination Rejected", "Choose"),
		(RuntimeError, "Recovery Export Error", "Could not export"),
	):
		window, tab = _window_with_tab()
		before = _tab_facts(tab)
		warnings = []
		error = error_type(str(tmp_path / "x"), "test failure")
		monkeypatch.setattr(tab, "recovery_export", lambda *_args: (_ for _ in ()).throw(error))
		monkeypatch.setattr(
			PySide6.QtWidgets.QFileDialog, "getSaveFileName", lambda *_args: (str(tmp_path / "x"), ""),
		)
		monkeypatch.setattr(
			window, "_show_edit_refusal", lambda request: warnings.append(request),
		)
		try:
			assert not window._on_native_recovery_export()
			assert (
				_tab_facts(tab) == before
				and warnings[-1].outcome.value == "unavailable_operation"
				and "test failure" in (warnings[-1].technical_details or "")
			)
		finally:
			_dispose_window(window)

"""Qt behavior for the read-only Check Structure workflow."""

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.themes.theme_loader
import ferrum_qt.themes.theme_manager
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.molecule_diagnostics
import ferrum_qt.main_window
import ferrum_qt.modes.base_mode


_CDML = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07"><standard line_width="9"/><paper id="paper"/>
<molecule id="root"><atom id="carbon" name="C"><point x="0" y="0"/></atom></molecule></cdml>
"""


#============================================
def _window_with_selected_root(
		qapp: PySide6.QtWidgets.QApplication,
		) -> tuple[object, object, str]:
	"""Return one real window whose current selection belongs to one direct root."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "diagnostics.cdml", ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	window._register_native_tab(tab, activate=True)
	molecule = tab.current_document_observation().projection.molecules[0]
	tab.select_atom(molecule.atoms[0].document_object_id)
	window._refresh_actions()
	return window, tab, molecule.document_object_id


#============================================
def _receipt(tab: object, molecule_id: str) -> object:
	"""Return one closed display receipt with no diagnostic findings."""
	snapshot = tab.current_snapshot
	return ferrum_qt.ferrum.molecule_diagnostics._Receipt(
		snapshot.revision, snapshot.digest, molecule_id, (),
	)


#============================================
def _dispose_window_and_tab(window: object, tab: object) -> None:
	"""Dispose one clean tab and its owned window after a widget behavior test."""
	window._close_tab_at(window.centralWidget().indexOf(tab))
	window.deleteLater()


#============================================
def test_check_structure_action_and_no_issues_dialog_use_real_selected_root(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A real selected direct root enables the action and exposes an accessible clean result."""
	window, tab, molecule_id = _window_with_selected_root(qapp)
	try:
		window._show_molecule_diagnostics_dialog(_receipt(tab, molecule_id), tab)
		dialog = window._molecule_diagnostics_dialog
		assert dialog is not None
		assert (
			window._check_structure_action.isEnabled(), dialog.isVisible(),
			dialog._no_issues.isVisible(), dialog._no_issues.accessibleName(),
		) == (True, True, True, "No structure issues found")
	finally:
		_dispose_window_and_tab(window, tab)
		del qapp


#============================================
def test_check_structure_dialog_recovers_when_the_original_selection_returns(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Returning to the same source fence and root clears stale state and enables rerun."""
	window, tab, molecule_id = _window_with_selected_root(qapp)
	try:
		window._show_molecule_diagnostics_dialog(_receipt(tab, molecule_id), tab)
		dialog = window._molecule_diagnostics_dialog
		assert dialog is not None
		tab.view.scene().clearSelection()
		qapp.processEvents()
		window._refresh_actions()
		was_stale = dialog._stale.isVisible() and not dialog._rerun.isEnabled()
		tab.select_atom(
			tab.current_document_observation().projection.molecules[0].atoms[0].document_object_id,
		)
		qapp.processEvents()
		window._refresh_actions()
		assert (was_stale, dialog._stale.isVisible(), dialog._rerun.isEnabled()) == (
			True, False, True,
		)
	finally:
		_dispose_window_and_tab(window, tab)
		del qapp


#============================================
def test_check_structure_worker_calls_controlled_owned_snapshot_executor(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch) -> None:
	"""The detached worker reaches its executor with owned snapshot facts only."""
	module = ferrum_qt.ferrum.molecule_diagnostics
	receipt = module._Receipt(7, "digest", "root", ())
	calls: list[tuple[str, int, str, tuple[str, ...]]] = []
	received: list[object] = []

	def execute(cdml: str, revision: int, digest: str,
			molecule_ids: tuple[str, ...]) -> object:
		"""Record the owned worker boundary and return its immutable receipt."""
		calls.append((cdml, revision, digest, molecule_ids))
		return receipt

	monkeypatch.setattr(module, "_execute_diagnostics_from_snapshot", execute)
	worker = module.FerrumNativeMoleculeDiagnosticsWorker("<cdml/>", 7, "digest", ("root",))
	worker.diagnosed.connect(received.append)
	worker.run()

	assert calls == [("<cdml/>", 7, "digest", ("root",))]
	assert received == [receipt]
	del qapp


#============================================
def test_admitted_check_structure_receipt_survives_selection_change_after_busy_refresh(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch) -> None:
	"""Read-only work preserves selection, and later selection change cannot lose its receipt."""
	module = ferrum_qt.ferrum.molecule_diagnostics
	window, tab, molecule_id = _window_with_selected_root(qapp)
	monkeypatch.setattr(module.FerrumNativeMoleculeDiagnosticsWorker, "start", lambda worker: None)
	try:
		assert window._start_molecule_diagnostics()
		intent = window._molecule_diagnostics_intent
		assert intent is not None
		assert window._selected_molecule_diagnostics_address(tab) is not None
		tab.view.scene().clearSelection()
		window._refresh_actions()
		window._on_document_molecule_diagnosed(intent.worker, _receipt(tab, molecule_id))
		dialog = window._molecule_diagnostics_dialog
		assert dialog is not None
		window._refresh_actions()
		assert dialog._stale.isVisible() and not dialog._rerun.isEnabled()
	finally:
		if window._molecule_diagnostics_intent is not None:
			window._on_document_molecule_diagnostics_finished(
				window._molecule_diagnostics_intent.worker,
			)
		_dispose_window_and_tab(window, tab)
		del qapp


#============================================
def test_diagnostics_keeps_structure_navigation_but_disables_selection_deletion(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch) -> None:
	"""Read-only diagnostics keeps the Rust selection while withholding its mutation action."""
	module = ferrum_qt.ferrum.molecule_diagnostics
	window, tab, _molecule_id = _window_with_selected_root(qapp)
	monkeypatch.setattr(module.FerrumNativeMoleculeDiagnosticsWorker, "start", lambda worker: None)
	try:
		assert window._window_mode_sync.select_action(window._select_structure_action)
		window._select_structure_at(
			ferrum_qt.modes.base_mode.ScenePoint(0.0, 0.0),
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		)
		selection = window._structure_selection
		assert selection is not None and selection.targets
		assert window._delete_structure_selection_action.isEnabled()
		assert window._start_molecule_diagnostics()
		intent = window._molecule_diagnostics_intent
		assert intent is not None
		assert window._select_structure_action.isEnabled()
		assert window._structure_selection is selection
		assert not window._delete_structure_selection_action.isEnabled()
		window._on_document_molecule_diagnostics_finished(intent.worker)
		assert window._delete_structure_selection_action.isEnabled()
	finally:
		window._window_mode_sync.cancel()
		if window._molecule_diagnostics_intent is not None:
			window._on_document_molecule_diagnostics_finished(
				window._molecule_diagnostics_intent.worker,
			)
		_dispose_window_and_tab(window, tab)
		del qapp


#============================================
def test_queued_selection_deletion_refuses_when_diagnostics_starts_first(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch) -> None:
	"""A queued deletion rechecks its mutation capability immediately before commit."""
	module = ferrum_qt.ferrum.molecule_diagnostics
	window, tab, _molecule_id = _window_with_selected_root(qapp)
	monkeypatch.setattr(module.FerrumNativeMoleculeDiagnosticsWorker, "start", lambda worker: None)
	try:
		assert window._window_mode_sync.select_action(window._select_structure_action)
		window._select_structure_at(
			ferrum_qt.modes.base_mode.ScenePoint(0.0, 0.0),
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		)
		assert window._delete_structure_selection_action.isEnabled()
		before_revision = tab.current_snapshot.revision
		before_undo = tab.can_undo()
		window._request_structure_deletion()
		assert window._start_molecule_diagnostics()
		intent = window._molecule_diagnostics_intent
		assert intent is not None
		qapp.processEvents()
		assert (tab.current_snapshot.revision, tab.can_undo()) == (before_revision, before_undo)
		assert not window._delete_structure_selection_action.isEnabled()
		window._on_document_molecule_diagnostics_finished(intent.worker)
		assert window._delete_structure_selection_action.isEnabled()
	finally:
		window._window_mode_sync.cancel()
		if window._molecule_diagnostics_intent is not None:
			window._on_document_molecule_diagnostics_finished(
				window._molecule_diagnostics_intent.worker,
			)
		_dispose_window_and_tab(window, tab)
		del qapp


#============================================
def test_check_structure_drops_receipt_with_a_changed_document_fence(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch) -> None:
	"""A receipt with a mismatched source revision cannot publish to the current tab."""
	module = ferrum_qt.ferrum.molecule_diagnostics
	window, tab, molecule_id = _window_with_selected_root(qapp)
	monkeypatch.setattr(module.FerrumNativeMoleculeDiagnosticsWorker, "start", lambda worker: None)
	try:
		receipt = _receipt(tab, molecule_id)
		assert window._start_molecule_diagnostics()
		intent = window._molecule_diagnostics_intent
		assert intent is not None
		window._molecule_diagnostics_intent = module._Intent(
			intent.tab, intent.revision + 1, intent.digest, intent.molecule_id, intent.worker,
		)
		window._on_document_molecule_diagnosed(intent.worker, receipt)
		assert window._molecule_diagnostics_dialog is None
	finally:
		if window._molecule_diagnostics_intent is not None:
			window._on_document_molecule_diagnostics_finished(
				window._molecule_diagnostics_intent.worker,
			)
		_dispose_window_and_tab(window, tab)
		del qapp

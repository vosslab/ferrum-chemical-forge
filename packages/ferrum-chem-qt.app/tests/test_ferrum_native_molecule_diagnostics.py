"""Qt behavior for the read-only Check Structure workflow."""

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.molecule_diagnostics


_CDML = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07"><standard line_width="9"/><paper id="paper"/>
<molecule id="root"><atom id="carbon" name="C"><point x="0" y="0"/></atom></molecule></cdml>
"""


#============================================
def _window_with_selected_root() -> tuple[object, object, str]:
	"""Return one real window whose current selection belongs to one direct root."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "diagnostics.cdml")
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
	window, tab, molecule_id = _window_with_selected_root()
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
	window, tab, molecule_id = _window_with_selected_root()
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

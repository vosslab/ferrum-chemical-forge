"""Qt behavior for fenced asynchronous canonical SMILES exports."""

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.themes.theme_loader
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.molecule_exports
import ferrum_qt.main_window
import ferrum_qt.modes.base_mode


_CDML = """
<cdml xmlns="urn:ferrum:cdml" version="26.07"><standard line_width="9"/><paper id="paper"/>
<molecule id="root"><atom id="carbon" name="C"><point x="0" y="0"/></atom></molecule></cdml>
"""


#============================================
def _window_with_selected_root(
		qapp: PySide6.QtWidgets.QApplication,
		) -> tuple[object, object, str]:
	"""Return one real native window with an admitted durable molecule selection."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_CDML, "exports.cdml",
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	window._register_native_tab(tab, activate=True)
	molecule = tab.current_document_observation().projection.molecules[0]
	tab.select_atom(molecule.atoms[0].document_object_id)
	window._refresh_actions()
	return window, tab, molecule.document_object_id


#============================================
def _dispose_window_and_tab(window: object, tab: object) -> None:
	"""Dispose the one window-owned tab after a Qt behavior test."""
	window._close_tab_at(window.centralWidget().indexOf(tab))
	window.deleteLater()


#============================================
def _smiles_receipt(tab: object, molecule_id: str) -> object:
	"""Issue one real Rust receipt for the tab's current immutable snapshot."""
	observation = tab.current_document_observation()
	snapshot = observation.snapshot
	return engine.export_document_molecule_smiles_v1(
		observation, snapshot.revision, snapshot.digest, molecule_id,
	)


#============================================
def test_admitted_smiles_clipboard_receipt_survives_selection_change_after_busy_refresh(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch) -> None:
	"""Busy refresh preserves selection, and later selection change cannot lose its receipt."""
	module = ferrum_qt.ferrum.molecule_exports
	window, tab, molecule_id = _window_with_selected_root(qapp)
	shown: list[str] = []
	monkeypatch.setattr(module.FerrumNativeMoleculeSmilesExportWorker, "start", lambda worker: None)
	monkeypatch.setattr(window, "_show_document_molecule_smiles", shown.append)
	try:
		assert window._window_mode_sync.select_action(window._select_structure_action)
		window._select_structure_at(
			ferrum_qt.modes.base_mode.ScenePoint(0.0, 0.0),
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		)
		selection = window._structure_selection
		assert selection is not None and selection.targets
		assert window._delete_structure_selection_action.isEnabled()
		before_revision = tab.current_snapshot.revision
		before_undo = tab.can_undo()
		window._request_structure_deletion()
		window._choose_document_molecule_smiles_export()
		intent = window._molecule_export_intent
		assert intent is not None
		assert window._selected_molecule_diagnostics_address(tab) is not None
		assert window._select_structure_action.isEnabled()
		assert window._structure_selection is selection
		assert not window._delete_structure_selection_action.isEnabled()
		qapp.processEvents()
		assert (tab.current_snapshot.revision, tab.can_undo()) == (before_revision, before_undo)
		tab.view.scene().clearSelection()
		window._refresh_actions()
		receipt = _smiles_receipt(tab, molecule_id)
		window._on_document_molecule_smiles_exported(intent.worker, receipt)
		assert (PySide6.QtWidgets.QApplication.clipboard().text(), shown) == (
			receipt.smiles, [receipt.smiles],
		)
		window._on_document_molecule_export_finished(intent.worker)
		assert window._delete_structure_selection_action.isEnabled()
	finally:
		window._window_mode_sync.cancel()
		if window._molecule_export_intent is not None:
			window._on_document_molecule_export_finished(window._molecule_export_intent.worker)
		_dispose_window_and_tab(window, tab)
		del qapp


#============================================
def test_admitted_smiles_file_receipt_survives_selection_change_after_busy_refresh(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: object) -> None:
	"""The file route rechecks before admission, then publishes despite selection change."""
	module = ferrum_qt.ferrum.molecule_exports
	window, tab, molecule_id = _window_with_selected_root(qapp)
	published: list[tuple[object, str]] = []
	monkeypatch.setattr(module.FerrumNativeMoleculeSmilesExportWorker, "start", lambda worker: None)
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog, "getSaveFileName",
		lambda *unused: (str(tmp_path / "receipt.smi"), module._SMILES_FILE_FILTER),
	)
	monkeypatch.setattr(window, "_publish_document_molecule_smiles_file",
		lambda receipt, destination: published.append((receipt, destination)))
	try:
		window._choose_document_molecule_smiles_file_export()
		intent = window._molecule_export_intent
		assert intent is not None
		assert window._selected_molecule_diagnostics_address(tab) is not None
		tab.view.scene().clearSelection()
		window._refresh_actions()
		receipt = _smiles_receipt(tab, molecule_id)
		window._on_document_molecule_smiles_exported(intent.worker, receipt)
		assert published == [(receipt, str(tmp_path / "receipt.smi"))]
	finally:
		if window._molecule_export_intent is not None:
			window._on_document_molecule_export_finished(window._molecule_export_intent.worker)
		_dispose_window_and_tab(window, tab)
		del qapp


#============================================
def test_smiles_file_export_refuses_selection_change_during_destination_choice(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: object) -> None:
	"""The file chooser reauthenticates selection before it admits any worker."""
	module = ferrum_qt.ferrum.molecule_exports
	window, tab, _molecule_id = _window_with_selected_root(qapp)
	refusals: list[object] = []

	def choose_destination(*unused: object) -> tuple[str, str]:
		"""Withdraw the original selection before the chooser returns its path."""
		del unused
		tab.view.scene().clearSelection()
		return str(tmp_path / "receipt.smi"), module._SMILES_FILE_FILTER

	monkeypatch.setattr(PySide6.QtWidgets.QFileDialog, "getSaveFileName", choose_destination)
	monkeypatch.setattr(window, "_show_edit_refusal", refusals.append)
	try:
		window._choose_document_molecule_smiles_file_export()
		assert window._molecule_export_intent is None
		refusal = refusals[-1]
		assert refusal.technical_details == (
			"The selected molecule changed while choosing a destination. "
			"Choose Export SMILES File again for the current selection."
		)
	finally:
		_dispose_window_and_tab(window, tab)
		del qapp

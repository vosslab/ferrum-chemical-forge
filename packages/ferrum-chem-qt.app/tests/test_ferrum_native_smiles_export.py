"""Behavior coverage for selected Ferrum document-to-SMILES export."""

# Standard Library
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.window_refusals


_SOURCE = """<cdml version='26.08'><molecule id='m1'>
<atom id='a1' name='N' charge='1' isotope='15' explicit_hydrogens='3'>
<point x='0' y='0'/></atom>
<atom id='a2' name='C'><point x='20' y='0'/></atom>
<bond id='b1' start='a1' end='a2' type='n1'/>
</molecule></cdml>"""

_MULTI_SOURCE = """<cdml version='26.08'>
<molecule id='first'>
<atom id='carbon' name='C'><point x='0' y='0'/></atom>
<atom id='carbon-2' name='C'><point x='20' y='0'/></atom>
<bond id='first-bond' start='carbon' end='carbon-2' type='n1'/>
</molecule>
<molecule id='second'><atom id='oxygen' name='O'><point x='40' y='0'/></atom></molecule>
</cdml>"""


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide the offscreen application used by the ordinary Ferrum window."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _action(window: object, text: str) -> PySide6.QtGui.QAction:
	"""Find one user-visible action without depending on its storage field."""
	actions = window.findChildren(PySide6.QtGui.QAction)
	return next(action for action in actions if action.text() == text)


#============================================
def _register(source: str) -> tuple[object, object]:
	"""Create one ordinary window with one active Rust-owned tab."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		source, "molecule.cdml",
	)
	window._register_native_tab(tab, activate=True)
	return window, tab


#============================================
def _wait_for_export(window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Join the one worker, then deliver its already-queued terminal signal."""
	intent = window._molecule_export_intent
	assert intent is not None and intent.worker.wait(10000)
	qapp.processEvents()


#============================================
def _dispose(window: object, tab: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Finish any live worker and retire the controlled tab/window pair."""
	intent = window._molecule_export_intent
	if intent is not None:
		intent.worker.cancel_delivery()
		intent.worker.wait(10000)
		qapp.processEvents()
	index = window._tab_widget.indexOf(tab)
	if index >= 0:
		window._close_tab_at(index)
	window.deleteLater()


#============================================
def test_visible_action_exports_canonical_smiles_without_mutation(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The public QAction reaches Rust and copies one exact current receipt."""
	window, tab = _register(_SOURCE)
	shown = []

	def capture_dialog(dialog: PySide6.QtWidgets.QMessageBox) -> int:
		"""Capture the real selectable result surface without blocking the test."""
		shown.append((dialog.text(), dialog.textInteractionFlags()))
		return int(PySide6.QtWidgets.QMessageBox.StandardButton.Ok)

	monkeypatch.setattr(PySide6.QtWidgets.QMessageBox, "exec", capture_dialog)
	try:
		action = _action(window, "Export SMILES")
		assert not action.isEnabled()
		tab.select_atom("a1")
		window._refresh_actions()
		assert action.isEnabled()
		before = tab.current_snapshot
		observation = tab.current_document_observation()
		selected = tab.selected_molecule_information_targets()
		clipboard = PySide6.QtWidgets.QApplication.clipboard()
		clipboard.setText("unchanged")
		action.trigger()
		_wait_for_export(window, qapp)
		flags = shown[0][1]
		selectable = (
			PySide6.QtCore.Qt.TextInteractionFlag.TextSelectableByMouse
			| PySide6.QtCore.Qt.TextInteractionFlag.TextSelectableByKeyboard
		)
		assert PySide6.QtWidgets.QApplication.clipboard().text() == "C[15NH3+]"
		assert "C[15NH3+]" in shown[0][0]
		assert flags & selectable == selectable
		assert tab.current_snapshot == before
		assert tab.current_document_observation() is observation
		assert tab.selected_molecule_information_targets() == selected
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_visible_file_action_publishes_exact_smiles_without_clipboard_or_document_effects(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""The public file action reaches Rust's publisher with one frozen receipt."""
	window, tab = _register(_SOURCE)
	destination = tmp_path / "selected-molecule"
	warnings = []
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getSaveFileName",
		lambda *_args: (str(destination), ""),
	)
	monkeypatch.setattr(
		window,
		"_show_edit_refusal",
		lambda request: warnings.append(request),
	)
	try:
		tab.select_atom("a1")
		window._refresh_actions()
		action = _action(window, "Export SMILES File...")
		assert action.isEnabled()
		before = tab.current_snapshot
		observation = tab.current_document_observation()
		selected = tab.selected_molecule_information_targets()
		clipboard = PySide6.QtWidgets.QApplication.clipboard()
		clipboard.setText("unchanged")

		action.trigger()
		_wait_for_export(window, qapp)

		assert destination.with_suffix(".smi").read_bytes() == b"C[15NH3+]\n"
		assert clipboard.text() == "unchanged"
		assert tab.current_snapshot == before
		assert tab.current_document_observation() is observation
		assert tab.selected_molecule_information_targets() == selected
		assert not warnings or warnings[-1].outcome.value == "unavailable_operation"
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_file_action_refuses_changed_selection_after_destination_dialog(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""A dialog-time root change cannot start chemistry or publish a stale receipt."""
	window, tab = _register(_MULTI_SOURCE)
	destination = tmp_path / "stale.smi"
	warnings = []

	def choose_path(*_args: object) -> tuple[str, str]:
		"""Change to a different durable root at the deterministic dialog seam."""
		tab.select_atom("oxygen")
		return str(destination), ""

	monkeypatch.setattr(PySide6.QtWidgets.QFileDialog, "getSaveFileName", choose_path)
	monkeypatch.setattr(
		window,
		"_show_edit_refusal",
		lambda request: warnings.append(request),
	)
	try:
		tab.select_atom("carbon")
		window._refresh_actions()
		_action(window, "Export SMILES File...").trigger()
		assert window._molecule_export_intent is None
		assert not destination.exists()
		assert warnings[-1].outcome.value == "unavailable_operation"
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_visible_inchi_file_actions_publish_each_explicit_mode_without_mutation(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""Standard and Fixed-H actions publish exact receipts and preserve the tab."""
	window, tab = _register(_SOURCE)
	destinations = iter((tmp_path / "standard", tmp_path / "fixed"))
	warnings = []
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getSaveFileName",
		lambda *_args: (str(next(destinations)), ""),
	)
	monkeypatch.setattr(
		window,
		"_show_edit_refusal",
		lambda request: warnings.append(request),
	)
	clipboard = PySide6.QtWidgets.QApplication.clipboard()
	clipboard.setText("unchanged")
	before = tab.current_snapshot
	selected = tab.selected_molecule_information_targets()
	try:
		_action(window, "Export Standard InChI File...").trigger()
		_wait_for_export(window, qapp)
		_action(window, "Export Fixed-H InChI File...").trigger()
		_wait_for_export(window, qapp)

		standard = (tmp_path / "standard.inchi").read_text(encoding="ascii")
		fixed = (tmp_path / "fixed.inchi").read_text(encoding="ascii")
		assert standard.startswith("InChI=1S/") and standard.endswith("\n")
		assert fixed.startswith("InChI=1/") and "/f" in fixed and fixed.endswith("\n")
		assert clipboard.text() == "unchanged"
		assert tab.current_snapshot == before
		assert tab.selected_molecule_information_targets() == selected
		assert all(title == "InChI File Durability Unconfirmed" for title, _text in warnings)
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_inchi_file_action_refuses_revision_change_after_destination_dialog(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""A dialog-time document change cannot publish an obsolete chosen root."""
	window, tab = _register(_SOURCE)
	destination = tmp_path / "stale.inchi"
	warnings = []

	def choose_path(*_args: object) -> tuple[str, str]:
		"""Commit one real revision at the deterministic destination seam."""
		tab.select_atom("a2")
		tab.change_selected_atom_element("O")
		return str(destination), ""

	monkeypatch.setattr(PySide6.QtWidgets.QFileDialog, "getSaveFileName", choose_path)
	monkeypatch.setattr(
		window,
		"_show_edit_refusal",
		lambda request: warnings.append(request),
	)
	try:
		_action(window, "Export Standard InChI File...").trigger()
		assert window._molecule_export_intent is None and not destination.exists()
		assert warnings[-1].outcome.value == "unavailable_operation"
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_action_requires_one_selected_durable_molecule(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Atoms and bonds qualify, while a selection spanning roots does not."""
	window, tab = _register(_MULTI_SOURCE)
	try:
		action = _action(window, "Export SMILES")
		tab.select_atom("carbon")
		window._refresh_actions()
		assert action.isEnabled()
		tab.select_bond("first-bond")
		window._refresh_actions()
		assert action.isEnabled()
		tab.select_atoms(("carbon", "oxygen"))
		window._refresh_actions()
		assert not action.isEnabled()
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_changed_selection_discards_completed_worker_result(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Queued success cannot escape after its selected direct root changes."""
	window, tab = _register(_MULTI_SOURCE)
	shown = []
	monkeypatch.setattr(window, "_show_document_molecule_smiles", shown.append)
	clipboard = PySide6.QtWidgets.QApplication.clipboard()
	clipboard.setText("unchanged")
	try:
		tab.select_atom("carbon")
		window._refresh_actions()
		action = _action(window, "Export SMILES")
		action.trigger()
		tab.select_atom("oxygen")
		window._refresh_actions()
		_wait_for_export(window, qapp)
		assert clipboard.text() == "unchanged" and shown == []
		assert "Discarded stale SMILES export" in window.statusBar().currentMessage()
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_tab_switch_discards_completed_worker_result(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A result from a background tab cannot replace the current clipboard."""
	window, source_tab = _register(_SOURCE)
	other_tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_MULTI_SOURCE, "other.cdml",
	)
	shown = []
	monkeypatch.setattr(window, "_show_document_molecule_smiles", shown.append)
	clipboard = PySide6.QtWidgets.QApplication.clipboard()
	clipboard.setText("unchanged")
	try:
		window._register_native_tab(other_tab, activate=False)
		source_tab.select_atom("a1")
		window._refresh_actions()
		_action(window, "Export SMILES").trigger()
		tabs = window.findChild(PySide6.QtWidgets.QTabWidget)
		tabs.setCurrentWidget(other_tab)
		_wait_for_export(window, qapp)
		assert clipboard.text() == "unchanged" and shown == []
		assert "Discarded stale SMILES export" in window.statusBar().currentMessage()
	finally:
		for tab in (other_tab, source_tab):
			index = window._tab_widget.indexOf(tab)
			if index >= 0:
				window._close_tab_at(index)
		window.deleteLater()


#============================================
def test_cancel_action_withholds_completed_worker_result(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Cancellation invalidates delivery without claiming Ferrum interruption."""
	window, tab = _register(_SOURCE)
	shown = []
	monkeypatch.setattr(window, "_show_document_molecule_smiles", shown.append)
	clipboard = PySide6.QtWidgets.QApplication.clipboard()
	clipboard.setText("unchanged")
	try:
		tab.select_atom("a1")
		window._refresh_actions()
		_action(window, "Export SMILES").trigger()
		cancel = _action(window, "Cancel Molecule Export")
		assert cancel.isEnabled()
		cancel.trigger()
		_wait_for_export(window, qapp)
		assert clipboard.text() == "unchanged" and shown == []
		assert not cancel.isEnabled()
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_unsupported_drawing_bond_reports_failure_without_fallback(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A presentation-only bond style remains a visible typed Rust refusal."""
	window, tab = _register(_SOURCE.replace("type='n1'", "type='w1'"))
	warnings: list[ferrum_qt.dialogs.refusal_presenter.RefusalPresentation] = []

	def capture_warning(
			_window: object, request: object,
			) -> None:
		"""Capture the public warning surface without opening a modal dialog."""
		warnings.append(ferrum_qt.dialogs.refusal_presenter.present_refusal(request))

	monkeypatch.setattr(ferrum_qt.ferrum.window_refusals, "show_refusal", capture_warning)
	clipboard = PySide6.QtWidgets.QApplication.clipboard()
	clipboard.setText("unchanged")
	try:
		tab.select_atom("a1")
		window._refresh_actions()
		before = tab.current_snapshot
		action = _action(window, "Export SMILES")
		action.trigger()
		_wait_for_export(window, qapp)
		assert warnings and warnings[-1].title == "Action Not Available"
		assert warnings[-1].technical_details is not None
		assert "drawing style" in warnings[-1].technical_details
		assert clipboard.text() == "unchanged" and tab.current_snapshot == before
	finally:
		_dispose(window, tab, qapp)

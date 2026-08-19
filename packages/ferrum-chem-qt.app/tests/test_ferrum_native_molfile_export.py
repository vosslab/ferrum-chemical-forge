"""Behavior coverage for selected Ferrum document-to-Molfile export."""

# Standard Library
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_chem
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window


_SOURCE = """<cdml version='26.08'><molecule id='m1' name='selected molecule'>
<atom id='a1' name='N' charge='1' isotope='15' explicit_hydrogens='3'>
<point x='2.5' y='7.5'/></atom>
<atom id='a2' name='C'><point x='12.5' y='-4'/></atom>
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
def _wait_for_molfile(window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Join the worker, then deliver its already-queued terminal signal."""
	intent = window._molfile_export_intent
	assert intent is not None and intent.worker.wait(10000)
	qapp.processEvents()


#============================================
def _dispose(window: object, tab: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Retire the controlled tab/window pair after any worker finishes."""
	intent = window._molfile_export_intent
	if intent is not None:
		intent.worker.cancel_delivery()
		intent.worker.wait(10000)
		qapp.processEvents()
	index = window._tab_widget.indexOf(tab)
	if index >= 0:
		window._close_tab_at(index)
	window.deleteLater()


#============================================
def test_visible_actions_publish_both_explicit_molfile_syntaxes_without_mutation(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""Both public actions reach Rust and preserve source and selected facts."""
	window, tab = _register(_SOURCE)
	destinations = iter((tmp_path / "selected-v2000", tmp_path / "selected-v3000"))
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
	try:
		v2000 = _action(window, "Export Molfile V2000...")
		v3000 = _action(window, "Export Molfile V3000...")
		assert not v2000.isEnabled() and not v3000.isEnabled()
		tab.select_atom("a1")
		window._refresh_actions()
		assert v2000.isEnabled() and v3000.isEnabled()
		before = tab.current_snapshot
		observation = tab.current_document_observation()
		selected = tab.selected_molecule_information_targets()

		v2000.trigger()
		_wait_for_molfile(window, qapp)
		v3000.trigger()
		_wait_for_molfile(window, qapp)

		for path, marker in (
			(tmp_path / "selected-v2000.mol", "V2000"),
			(tmp_path / "selected-v3000.mol", "V3000"),
		):
			text = path.read_text(encoding="utf-8")
			assert text.splitlines()[0] == "selected molecule"
			assert marker in text
			molecule = ferrum_chem.molblock_to_molecule(text)
			assert tuple((point.x, point.y) for point in molecule.coordinates) == (
				(2.5, -7.5), (12.5, 4.0),
			)
		assert tab.current_snapshot == before
		assert tab.current_document_observation() is observation
		assert tab.selected_molecule_information_targets() == selected
		assert not warnings or all(
			title == "Molfile Durability Unconfirmed" for title, _text in warnings
		)
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_action_requires_one_root_and_reauthenticates_after_the_dialog(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""A cross-root or dialog-time selection change cannot publish a file."""
	window, tab = _register(_MULTI_SOURCE)
	destination = tmp_path / "stale.mol"
	warnings = []

	def choose_path(*_args: object) -> tuple[str, str]:
		"""Change to another durable root at the deterministic dialog seam."""
		tab.select_atom("oxygen")
		return str(destination), ""

	monkeypatch.setattr(PySide6.QtWidgets.QFileDialog, "getSaveFileName", choose_path)
	monkeypatch.setattr(
		window,
		"_show_edit_refusal",
		lambda request: warnings.append(request),
	)
	try:
		action = _action(window, "Export Molfile V2000...")
		tab.select_atoms(("carbon", "oxygen"))
		window._refresh_actions()
		assert not action.isEnabled()
		tab.select_atom("carbon")
		window._refresh_actions()
		assert action.isEnabled()
		action.trigger()
		assert not destination.exists()
		assert warnings[-1].outcome.value == "unavailable_operation"
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_shared_cancel_action_withholds_molfile_publication(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""Cancellation invalidates delivery without claiming Ferrum interruption."""
	window, tab = _register(_SOURCE)
	destination = tmp_path / "cancelled.mol"
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getSaveFileName",
		lambda *_args: (str(destination), ""),
	)
	try:
		tab.select_atom("a1")
		window._refresh_actions()
		_action(window, "Export Molfile V3000...").trigger()
		cancel = _action(window, "Cancel Molecule Export")
		assert cancel.isEnabled()
		cancel.trigger()
		_wait_for_molfile(window, qapp)
		assert not destination.exists()
		assert not cancel.isEnabled()
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_unsupported_drawing_bond_reports_failure_without_file_fallback(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""A drawing-only bond remains a visible typed Rust refusal."""
	window, tab = _register(_SOURCE.replace("type='n1'", "type='w1'"))
	destination = tmp_path / "unsupported.mol"
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
		before = tab.current_snapshot
		_action(window, "Export Molfile V2000...").trigger()
		_wait_for_molfile(window, qapp)
		assert warnings[-1].outcome.value == "unavailable_operation"
		assert "drawing style" in warnings[-1].technical_details or ""
		assert not destination.exists() and tab.current_snapshot == before
	finally:
		_dispose(window, tab, qapp)

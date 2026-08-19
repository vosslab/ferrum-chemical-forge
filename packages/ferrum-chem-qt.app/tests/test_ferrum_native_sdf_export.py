"""Behavior coverage for exact selected Ferrum SDF record export."""

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


_SOURCE = (
	'<cdml><molecule id="m" name="display name">'
	'<atom id="a" name="C"><point x="2" y="7"/></atom>'
	'<atom id="b" name="O"><point x="12" y="-4"/></atom>'
	'<bond id="ab" start="a" end="b" type="n1"/>'
	'<f:sdf-record xmlns:f="urn:ferrum-chemical-forge:sdf-import:v1" '
	'encoding="utf8-hex-v1" title="496d706f72746564207469746c65">'
	'<f:property name="4e4f5445" value="6669727374"/>'
	'<f:property name="4e4f5445" value="7365636f6e64"/>'
	'</f:sdf-record></molecule></cdml>'
)

_MULTI_SOURCE = (
	'<cdml><molecule id="first"><atom id="carbon" name="C">'
	'<point x="0" y="0"/></atom></molecule>'
	'<molecule id="second"><atom id="oxygen" name="O">'
	'<point x="40" y="0"/></atom></molecule></cdml>'
)


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
def _wait_for_sdf(window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Join the worker, then deliver its already-queued terminal signal."""
	intent = window._sdf_export_intent
	assert intent is not None and intent.worker.wait(10000)
	qapp.processEvents()


#============================================
def _dispose(window: object, tab: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Retire the controlled tab/window pair after any worker finishes."""
	intent = window._sdf_export_intent
	if intent is not None:
		intent.worker.cancel_delivery()
		intent.worker.wait(10000)
		qapp.processEvents()
	index = window._tab_widget.indexOf(tab)
	if index >= 0:
		window._close_tab_at(index)
	window.deleteLater()


#============================================
def test_visible_actions_publish_both_syntaxes_with_exact_import_metadata(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""Both public actions preserve duplicate fields and leave the tab unchanged."""
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
		v2000 = _action(window, "Export SDF Record V2000...")
		v3000 = _action(window, "Export SDF Record V3000...")
		assert not v2000.isEnabled() and not v3000.isEnabled()
		tab.select_atom("a")
		window._refresh_actions()
		assert v2000.isEnabled() and v3000.isEnabled()
		before = tab.current_snapshot
		selection = tab.selected_molecule_information_targets()

		v2000.trigger()
		_wait_for_sdf(window, qapp)
		v3000.trigger()
		_wait_for_sdf(window, qapp)

		for path, marker in (
			(tmp_path / "selected-v2000.sdf", "V2000"),
			(tmp_path / "selected-v3000.sdf", "V3000"),
		):
			text = path.read_text(encoding="utf-8")
			assert text.startswith("Imported title\n") and marker in text
			assert ">  <NOTE>\nfirst\n\n>  <NOTE>\nsecond\n\n$$$$\n" in text
			record = ferrum_chem.sdf_to_records(text)[0]
			assert record.title == "Imported title"
			assert tuple((item.name, item.value) for item in record.properties) == (
				("NOTE", "first"), ("NOTE", "second"),
			)
		assert tab.current_snapshot == before
		assert tab.selected_molecule_information_targets() == selection
		assert not warnings or all(
			title == "SDF Durability Unconfirmed" for title, _text in warnings
		)
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_action_reauthenticates_selection_after_the_destination_dialog(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""A dialog-time root change cannot publish the captured SDF record."""
	window, tab = _register(_MULTI_SOURCE)
	destination = tmp_path / "stale.sdf"
	warnings = []

	def choose_path(*_args: object) -> tuple[str, str]:
		"""Change to another root at the deterministic destination seam."""
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
		_action(window, "Export SDF Record V2000...").trigger()
		assert not destination.exists()
		assert warnings[-1].outcome.value == "unavailable_operation"
	finally:
		_dispose(window, tab, qapp)


#============================================
def test_shared_cancel_action_withholds_sdf_publication(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""Cancellation invalidates delivery without claiming Ferrum interruption."""
	window, tab = _register(_SOURCE)
	destination = tmp_path / "cancelled.sdf"
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getSaveFileName",
		lambda *_args: (str(destination), ""),
	)
	try:
		tab.select_atom("a")
		window._refresh_actions()
		_action(window, "Export SDF Record V3000...").trigger()
		cancel = _action(window, "Cancel Molecule Export")
		assert cancel.isEnabled()
		cancel.trigger()
		_wait_for_sdf(window, qapp)
		assert not destination.exists() and not cancel.isEnabled()
	finally:
		_dispose(window, tab, qapp)

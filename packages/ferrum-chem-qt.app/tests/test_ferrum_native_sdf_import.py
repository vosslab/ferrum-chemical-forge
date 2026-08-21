"""Behavior coverage for revision-bound Ferrum SDF record insertion."""

# Standard Library
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import ferrum_chem
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window


_EMPTY_CDML = "<cdml/>"


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one offscreen application for the ordinary Ferrum window."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _two_record_sdf() -> str:
	"""Return two coordinate-bearing SDF records through the native adapter."""
	first = ferrum_chem.prepare_sdf_record(ferrum_chem.parse_smiles("CCO"), "ethanol", ())
	second = ferrum_chem.prepare_sdf_record(ferrum_chem.parse_smiles("O"), "water", ())
	return ferrum_chem.records_to_sdf(
		(first, second), ferrum_chem.MolblockVersionV1.v2000,
	)


#============================================
def _finish_worker(qapp: PySide6.QtWidgets.QApplication, window: object) -> None:
	"""Join preparation and deliver the queued terminal event exactly once."""
	intent = window._sdf_import_intent
	assert intent is not None and intent.worker.wait(10000)
	for _iteration in range(3):
		qapp.processEvents()
	assert window._sdf_import_intent is None


#============================================
def test_sdf_action_reads_descriptor_bound_text_and_commits_one_batch(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Current-tab SDF insertion preserves records in one generic session commit."""
	path = tmp_path / "records.sdf"
	path.write_text(_two_record_sdf(), encoding="utf-8")
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_EMPTY_CDML, "native.cdml")
	window._register_native_tab(tab, activate=True)
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getOpenFileName",
		lambda *_args: (str(path), ""),
	)
	try:
		window._import_sdf_action.trigger()
		_finish_worker(qapp, window)

		projection = tab._document_observation.projection
		assert tab.current_snapshot.revision == 1 and tab.is_dirty
		assert tuple(molecule.name for molecule in projection.molecules) == ("ethanol", "water")
	finally:
		tab.dispose()
		window.deleteLater()


#============================================
def test_current_document_sdf_route_has_no_retired_file_preparation_call() -> None:
	"""Qt supplies bounded text to the generic record insertion boundary only."""
	source = pathlib.Path(__file__).parents[1] / "ferrum_qt/ferrum/sdf_import.py"
	assert "prepare_sdf_file_v1" not in source.read_text(encoding="utf-8")

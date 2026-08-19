"""Behavior coverage for bounded Ferrum molfile insertion."""

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
import ferrum_qt.ferrum.molblock_import


_EMPTY_CDML = "<cdml/>"


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one offscreen application without legacy document ownership."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _molblock() -> str:
	"""Return one valid coordinate-bearing V2000 molecule."""
	molecule = ferrum_chem.parse_smiles("CCO")
	molblock = ferrum_chem.molecule_to_molblock(
		molecule,
		ferrum_chem.MolblockVersionV1.v2000,
	)
	return molblock


#============================================
def _finish_worker(qapp: PySide6.QtWidgets.QApplication,
		window: object) -> None:
	"""Wait for bounded parsing and deliver queued Qt results."""
	intent = window._molblock_import_intent
	assert intent is not None and intent.worker.wait(10000)
	for _iteration in range(3):
		qapp.processEvents()
	assert window._molblock_import_intent is None


#============================================
def test_worker_reads_one_bounded_file_into_frozen_ferrum_facts(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path) -> None:
	"""Python supplies a path while Rust owns reading, validation, and parsing."""
	path = tmp_path / "ethanol.mol"
	path.write_text(_molblock(), encoding="utf-8")
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)
	worker = (
		ferrum_qt.ferrum.molblock_import.
		FerrumNativeMolblockPreparationWorker(str(path), placement)
	)
	prepared = []
	failures = []
	worker.prepared.connect(prepared.append)
	worker.failed.connect(failures.append)
	worker.start()
	assert worker.wait(10000)
	qapp.processEvents()

	assert failures == [] and type(prepared[0]) is ferrum_chem.MoleculeInsertionV1
	assert (prepared[0].atom_count, prepared[0].bond_count) == (3, 2)
	worker.deleteLater()


#============================================
def test_invalid_utf8_file_fails_without_document_mutation(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path) -> None:
	"""Rust admission rejects bytes before an adapter or session can own them."""
	path = tmp_path / "invalid.mol"
	path.write_bytes(b"\xff")
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EMPTY_CDML, "native.cdml",
	)
	window._register_native_tab(tab, activate=True)
	warnings = []
	window._show_edit_refusal = (
		lambda request: warnings.append(request)
	)
	assert window.start_molblock_import(str(path))
	_finish_worker(qapp, window)

	assert tab.current_snapshot.revision == 0 and not tab.is_dirty
	assert warnings[-1].outcome.value == "unavailable_operation"
	tab.dispose()
	window.deleteLater()


#============================================
def test_public_molfile_action_commits_and_saves_rust_owned_chemistry(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The menu route performs bounded parse, revision commit, render, and save."""
	path = tmp_path / "ethanol.mol"
	path.write_text(_molblock(), encoding="utf-8")
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EMPTY_CDML, "native.cdml",
	)
	window._register_native_tab(tab, activate=True)
	monkeypatch.setattr(
		PySide6.QtWidgets.QFileDialog,
		"getOpenFileName",
		lambda *_args: (str(path), ""),
	)
	window._import_molblock_action.trigger()
	_finish_worker(qapp, window)

	molecule = tab._document_observation.projection.molecules[0]
	assert tuple(atom.element for atom in molecule.atoms) == ("C", "C", "O")
	assert tab.current_snapshot.revision == 1 and tab.is_dirty

	destination = tmp_path / "molfile-import.cdml"
	tab.save_atomic(destination)
	reopened = ferrum_chem.DocumentSession.load(destination.read_text(encoding="utf-8"))
	reopened_molecule = reopened.observe_render(0).document.projection.molecules[0]
	assert tuple(atom.element for atom in reopened_molecule.atoms) == ("C", "C", "O")
	window.close()
	window.deleteLater()

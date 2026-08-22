"""Behavior coverage for Ferrum SMILES preparation and insertion."""

# Standard Library
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
import ferrum_chem
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.smiles_import


_EMPTY_CDML = "<cdml xmlns='urn:ferrum:cdml'/>"


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one offscreen application without importing the legacy host."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _finish_window_worker(
		qapp: PySide6.QtWidgets.QApplication,
		window: ferrum_qt.ferrum.main_window.FerrumNativeMainWindow,
		) -> None:
	"""Wait for worker cleanup, then deliver its already-queued Qt outcome."""
	intent = window._smiles_import_intent
	assert intent is not None and intent.worker.wait(10000)
	for _iteration in range(3):
		qapp.processEvents()
	assert window._smiles_import_intent is None


#============================================
def test_worker_prepares_one_frozen_native_cco_value_off_the_qt_thread(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The real worker delivers owned Rust facts rather than a Python graph."""
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)
	worker = (
		ferrum_qt.ferrum.smiles_import.
		FerrumNativeSmilesPreparationWorker("CCO", placement)
	)
	prepared = []
	failures = []
	worker.prepared.connect(prepared.append)
	worker.failed.connect(failures.append)
	worker.start()
	assert worker.wait(10000)
	qapp.processEvents()

	assert failures == []
	assert len(prepared) == 1
	assert type(prepared[0]) is ferrum_chem.MoleculeInsertionV1
	assert (prepared[0].atom_count, prepared[0].bond_count) == (3, 2)
	worker.deleteLater()


#============================================
def test_worker_cancellation_invalidates_delivery_without_claiming_preemption() -> None:
	"""Cancellation drops a completed value while worker cleanup remains ordinary."""
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)
	worker = (
		ferrum_qt.ferrum.smiles_import.
		FerrumNativeSmilesPreparationWorker._from_fixture(
			"CCO", placement, lambda _smiles, _placement: object(),
		)
	)
	prepared = []
	worker.prepared.connect(prepared.append)
	worker.cancel_delivery()
	worker.run()

	assert worker.delivery_cancelled and prepared == []
	worker.deleteLater()


#============================================
def test_cancel_after_native_completion_still_drops_queued_document_delivery(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A close-time cancel wins even when the Ferrum result is already queued."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EMPTY_CDML, "native.cdml",
	)
	window._register_native_tab(tab, activate=True)
	assert window.start_smiles_import("CCO")
	intent = window._smiles_import_intent
	assert intent is not None and intent.worker.wait(10000)
	intent.worker.cancel_delivery()
	for _iteration in range(3):
		qapp.processEvents()

	assert tab.current_snapshot.revision == 0 and not tab.is_dirty
	assert window._smiles_import_intent is None
	tab.dispose()
	window.deleteLater()


#============================================
def test_public_native_action_imports_renders_and_round_trips_cco(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""One user action performs the Rust worker, transaction, render, and save loop."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EMPTY_CDML, "native.cdml",
	)
	window._register_native_tab(tab, activate=True)
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText", lambda *_args: ("CCO", True),
	)
	window._import_smiles_action.trigger()
	_finish_window_worker(qapp, window)

	projection = tab._document_observation.projection
	assert tab.current_snapshot.revision == 1 and tab.is_dirty
	assert len(projection.molecules) == 1
	assert tuple(atom.element for atom in projection.molecules[0].atoms) == ("C", "C", "O")
	assert len(projection.molecules[0].bonds) == 2
	assert len(tab._controller.projection.items) > 0

	destination = tmp_path / "smiles-cco.cdml"
	tab.save_atomic(destination)
	reopened = ferrum_chem.DocumentSession.load(destination.read_text(encoding="utf-8"))
	reopened_projection = reopened.observe_render(0).document.projection
	assert tuple(
		atom.element for atom in reopened_projection.molecules[0].atoms
	) == ("C", "C", "O")
	window.close()
	window.deleteLater()


#============================================
def test_post_commit_render_failure_retains_pending_rust_authority(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An accepted molecule cannot be hidden by a failed disposable projection."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EMPTY_CDML, "native.cdml",
	)
	window._register_native_tab(tab, activate=True)
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText", lambda *_args: ("CCO", True),
	)
	monkeypatch.setattr(tab._controller, "replace", lambda _observation, _latch: False)
	warnings = []
	monkeypatch.setattr(
		window, "_show_edit_refusal",
		lambda request: warnings.append(request),
	)
	window._import_smiles_action.trigger()
	_finish_window_worker(qapp, window)

	assert tab.requires_refresh and tab.is_dirty
	assert tab._pending_snapshot.revision == 1 and tab.current_snapshot.revision == 0
	assert warnings[-1].outcome.value == "unavailable_operation"
	assert not window._save_action.isEnabled()
	assert not window._import_smiles_action.isEnabled()
	assert window._refresh_action.isEnabled()
	window._cancel_smiles_import()
	tab.dispose()
	window.deleteLater()

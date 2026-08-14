"""Behavior coverage for Rust-native existing-molecule coordinate generation."""

# Standard Library
import math
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import ferrum_chem
import pytest

# local repo modules
import ferrum_qt.native.ferrum_native_coordinate_generation
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_main_window


_SOURCE = """<cdml version='26.08'><molecule id='mol-1' name='Ethanol'>
  <atom id='atom-1' name='C'><point x='10' y='20'/></atom>
  <atom id='atom-2' name='C'><point x='55' y='25'/></atom>
  <atom id='atom-3' name='O'><point x='45' y='70'/></atom>
  <bond id='bond-1' start='atom-1' end='atom-2' type='n1'/>
  <bond id='bond-2' start='atom-2' end='atom-3' type='n1'/>
</molecule></cdml>"""


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one offscreen application without importing the legacy host."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _positions(tab: object) -> tuple[tuple[float, float], ...]:
	"""Return current source-ordered atom positions from the Rust projection."""
	molecule = tab._document_observation.projection.molecules[0]
	return tuple((atom.position.x, atom.position.y) for atom in molecule.atoms)


#============================================
def _centroid(points: tuple[tuple[float, float], ...]) -> tuple[float, float]:
	"""Return the arithmetic centroid used by the explicit placement contract."""
	return (
		sum(point[0] for point in points) / len(points),
		sum(point[1] for point in points) / len(points),
	)


#============================================
def _mean_bond_length(points: tuple[tuple[float, float], ...]) -> float:
	"""Return the source graph's mean length for its two ordered bonds."""
	return (
		math.dist(points[0], points[1]) + math.dist(points[1], points[2])
	) / 2.0


#============================================
def _finish_coordinate_worker(
		qapp: PySide6.QtWidgets.QApplication,
		window: ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow,
		) -> None:
	"""Wait for native teardown, then deliver its queued UI-thread result."""
	intent = window._coordinate_generation_intent
	assert intent is not None and intent.worker.wait(10000)
	for _iteration in range(3):
		qapp.processEvents()
	assert window._coordinate_generation_intent is None


#============================================
def test_worker_returns_frozen_revision_bound_coordinates(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The worker returns owned Ferrum facts rather than an OASA molecule graph."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation = session.observe(0)
	molecule_id = observation.projection.molecules[0].id
	worker = (
		ferrum_qt.native.ferrum_native_coordinate_generation.
		FerrumNativeCoordinatePreparationWorker(observation, molecule_id)
	)
	prepared = []
	failures = []
	worker.prepared.connect(prepared.append)
	worker.failed.connect(failures.append)
	worker.start()
	assert worker.wait(10000)
	qapp.processEvents()

	assert failures == [] and len(prepared) == 1
	assert type(prepared[0]) is ferrum_chem.PreparedMoleculeCoordinatesV1
	assert prepared[0].source_revision == 0
	assert prepared[0].source_digest == observation.snapshot.digest
	assert prepared[0].molecule_id == molecule_id and prepared[0].atom_count == 3
	worker.deleteLater()


#============================================
def test_public_action_preserves_current_placement_and_round_trips_history(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""One action performs worker, transaction, render, undo, redo, and save."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_SOURCE, "ethanol.cdml",
	)
	window._register_native_tab(tab, activate=True)
	before = _positions(tab)
	window._generate_coordinates_action.trigger()
	_finish_coordinate_worker(qapp, window)
	after = _positions(tab)

	assert tab.current_snapshot.revision == 1 and tab.is_dirty and after != before
	assert math.isclose(_centroid(after)[0], _centroid(before)[0], abs_tol=1e-10)
	assert math.isclose(_centroid(after)[1], _centroid(before)[1], abs_tol=1e-10)
	assert math.isclose(_mean_bond_length(after), _mean_bond_length(before), rel_tol=1e-12)
	assert "Generated Rust-native coordinates." in window.statusBar().currentMessage()

	tab.undo()
	assert _positions(tab) == before
	tab.redo()
	assert _positions(tab) == after
	destination = tmp_path / "regenerated.cdml"
	tab.save_atomic(destination)
	reopened = ferrum_chem.DocumentSession.load(destination.read_text(encoding="utf-8")).observe(0)
	reopened_positions = tuple(
		(atom.position.x, atom.position.y)
		for atom in reopened.projection.molecules[0].atoms
	)
	assert reopened_positions == after
	window.close()
	window.deleteLater()


#============================================
def test_worker_result_is_discarded_when_the_source_revision_changes(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A queued coordinate result cannot overwrite a newer authoritative edit."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_SOURCE, "ethanol.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window._generate_coordinates_action.trigger()
	intent = window._coordinate_generation_intent
	assert intent is not None and intent.worker.wait(10000)
	tab.select_atom("atom-1")
	tab.change_selected_atom_element("N")
	changed = tab.current_snapshot
	for _iteration in range(3):
		qapp.processEvents()

	assert tab.current_snapshot.digest == changed.digest
	assert tab._document_observation.projection.molecules[0].atoms[0].element == "N"
	assert "Discarded stale coordinates" in window.statusBar().currentMessage()
	window.deleteLater()

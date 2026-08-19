"""Behavior tests for the public Rust-owned Ferrum Repair menu."""

# Standard Library
import math
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window


_TERMINAL_CDML = """<cdml><molecule id='m'>
  <atom id='z' name='O'><point x='0.9659258262890683'
    y='-0.25881904510252074' z='3'/></atom>
  <atom id='a' name='C'><point x='0' y='0'/></atom>
  <bond id='az' start='a' end='z' type='n1'/>
</molecule></cdml>"""

_SNAP_CDML = """<cdml>
  <molecule id='first'><atom id='a' name='C'><point x='0.2' y='0.2'/></atom></molecule>
  <molecule id='second'><atom id='b' name='O'><point x='2.2' y='0.2'/></atom></molecule>
</cdml>"""

_RING_CDML = """<cdml><molecule id='ring'>
  <atom id='a' name='C'><point x='0' y='0'/></atom>
  <atom id='b' name='C'><point x='20' y='0'/></atom>
  <atom id='c' name='C'><point x='15' y='10'/></atom>
  <atom id='d' name='C'><point x='0' y='10'/></atom>
  <atom id='side' name='O'><point x='-10' y='10'/></atom>
  <bond id='ab' start='a' end='b' type='n1'/>
  <bond id='bc' start='b' end='c' type='n1'/>
  <bond id='cd' start='c' end='d' type='n1'/>
  <bond id='da' start='d' end='a' type='n1'/>
  <bond id='ds' start='d' end='side' type='n1'/>
</molecule></cdml>"""

_ANGLE_CDML = """<cdml><molecule id='m'>
  <atom id='root' name='C'><point x='0' y='0'/></atom>
  <atom id='z_first' name='N'><point x='10' y='1' z='3'/></atom>
  <atom id='a_second' name='O'><point x='10' y='2'/></atom>
  <bond id='z_first_bond' start='root' end='z_first' type='n1'/>
  <bond id='a_second_bond' start='root' end='a_second' type='n1'/>
</molecule></cdml>"""

_HALF_AUTHORED_UNIT_POINTS = (0.001 * 72.0 / 2.54) / 2.0


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide the offscreen application used by the real Ferrum widgets."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def test_straighten_action_uses_selected_molecule_and_restores_selection(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The public action submits durable IDs and never applies Qt geometry."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_TERMINAL_CDML, "terminal.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("z")
	window._refresh_actions()
	assert window._straighten_bonds_action.isEnabled()
	window._on_straighten_bonds()
	moved, fixed = tab._document_observation.projection.molecules[0].atoms
	assert moved.position.x == pytest.approx(
		math.sqrt(3.0) / 2.0, abs=_HALF_AUTHORED_UNIT_POINTS,
	)
	assert moved.position.y == pytest.approx(-0.5, abs=_HALF_AUTHORED_UNIT_POINTS)
	assert moved.position.z == 3.0
	assert (fixed.position.x, fixed.position.y) == (0.0, 0.0)
	assert tuple(
		(target.kind, target.identifier)
		for target in tab._controller.projection.selected_durable_targets()
	) == (("atom", "z"),)
	assert tab.current_snapshot.revision == 1
	tab.undo()
	window.close()


#============================================
def test_snap_action_requires_explicit_positive_spacing_and_repairs_all(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch) -> None:
	"""No selection means all durable molecules; invalid user input is atomic."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_SNAP_CDML, "snap.cdml",
	)
	window._register_native_tab(tab, activate=True)
	warnings = []
	monkeypatch.setattr(
		window, "_show_edit_refusal",
		lambda request: warnings.append(request),
	)
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText",
		lambda *_args: ("0", True),
	)
	window._on_snap_to_hex_grid()
	assert tab.current_snapshot.revision == 0
	assert warnings[-1].outcome.value == "unavailable_operation"
	assert warnings[-1].technical_details

	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText",
		lambda *_args: ("1.0", True),
	)
	window._on_snap_to_hex_grid()
	assert tab.current_snapshot.revision == 1
	first, second = tab._document_observation.projection.molecules
	assert (first.atoms[0].position.x, first.atoms[0].position.y) == (0.0, 0.0)
	assert second.atoms[0].position.x == pytest.approx(
		3.0 * math.sqrt(3.0) / 2.0, abs=_HALF_AUTHORED_UNIT_POINTS,
	)
	assert second.atoms[0].position.y == pytest.approx(
		0.5, abs=_HALF_AUTHORED_UNIT_POINTS,
	)
	assert not tab._controller.projection.selected_durable_targets()
	tab.undo()
	window.close()


#============================================
def test_normalize_lengths_action_uses_explicit_spacing_and_rust_geometry(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch) -> None:
	"""The UI supplies intent while Rust owns directions, anchors, and persistence."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_TERMINAL_CDML, "length.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("z")
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText",
		lambda *_args: ("2.0", True),
	)
	window._on_normalize_bond_lengths()
	moved, fixed = tab._document_observation.projection.molecules[0].atoms
	length = math.hypot(
		moved.position.x - fixed.position.x,
		moved.position.y - fixed.position.y,
	)
	assert length == pytest.approx(2.0, abs=2.0 * _HALF_AUTHORED_UNIT_POINTS)
	assert tuple(
		(target.kind, target.identifier)
		for target in tab._controller.projection.selected_durable_targets()
	) == (("atom", "z"),)
	assert tab.current_snapshot.revision == 1
	tab.undo()
	window.close()


#============================================
def test_normalize_angles_action_uses_rust_slots_and_restores_selection(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch) -> None:
	"""The Ferrum action supplies intent while Rust assigns authored-order slots."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_ANGLE_CDML, "angles.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("z_first")
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText",
		lambda *_args: ("20.0", True),
	)
	window._on_normalize_bond_angles()
	atoms = {
		atom.source_id: atom.position
		for atom in tab._document_observation.projection.molecules[0].atoms
	}
	first_distance = math.hypot(10.0, 1.0)
	second_distance = math.hypot(10.0, 2.0)
	assert (atoms["z_first"].x, atoms["z_first"].y) == pytest.approx(
		(first_distance, 0.0), abs=_HALF_AUTHORED_UNIT_POINTS,
	)
	assert atoms["z_first"].z == 3.0
	assert (atoms["a_second"].x, atoms["a_second"].y) == pytest.approx(
		(second_distance / 2.0, second_distance * math.sqrt(3.0) / 2.0),
		abs=_HALF_AUTHORED_UNIT_POINTS,
	)
	assert tuple(
		(target.kind, target.identifier)
		for target in tab._controller.projection.selected_durable_targets()
	) == (("atom", "z_first"),)
	assert tab.current_snapshot.revision == 1
	tab.undo()
	window.close()


#============================================
def test_normalize_ring_action_preserves_centroid_and_durable_selection(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch) -> None:
	"""The public action supplies spacing while Rust owns ring topology and layout."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_RING_CDML, "ring.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("a")
	before_atoms = tab._document_observation.projection.molecules[0].atoms
	before = {atom.source_id: atom.position for atom in before_atoms}
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText",
		lambda *_args: ("20.0", True),
	)
	window._on_normalize_rings()
	after_atoms = tab._document_observation.projection.molecules[0].atoms
	after = {atom.source_id: atom.position for atom in after_atoms}
	ring = ("a", "b", "c", "d")
	before_center = tuple(
		sum(getattr(before[key], axis) for key in ring) / 4 for axis in ("x", "y")
	)
	after_center = tuple(
		sum(getattr(after[key], axis) for key in ring) / 4 for axis in ("x", "y")
	)
	assert after_center == pytest.approx(before_center, abs=_HALF_AUTHORED_UNIT_POINTS)
	assert tuple(
		(target.kind, target.identifier)
		for target in tab._controller.projection.selected_durable_targets()
	) == (("atom", "a"),)
	assert tab.current_snapshot.revision == 1
	tab.undo()
	window.close()


#============================================
def test_clean_geometry_requires_explicit_spacing_before_native_work(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch) -> None:
	"""Invalid user intent leaves the session unchanged and starts no worker."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_TERMINAL_CDML, "clean.cdml",
	)
	window._register_native_tab(tab, activate=True)
	warnings = []
	monkeypatch.setattr(
		window, "_show_edit_refusal",
		lambda request: warnings.append(request),
	)
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText",
		lambda *_args: ("0", True),
	)

	assert window._clean_geometry_action.isEnabled()
	window._clean_geometry_action.trigger()
	assert tab.current_snapshot.revision == 0
	assert window._coordinate_generation_intent is None
	assert warnings[-1].outcome.value == "unavailable_operation"
	assert warnings[-1].technical_details
	window.close()

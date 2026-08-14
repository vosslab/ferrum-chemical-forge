"""Behavior coverage for the public OASA-free Ferrum-native window seam."""

# Standard Library
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_main_window


_EDITABLE_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  </molecule>
</cdml>"""

_MULTI_MOLECULE_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1' name='First'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  </molecule>
  <molecule id='mol-2' name='Second'>
    <atom id='atom-n' name='N'><point x='30' y='40'/></atom>
  </molecule>
</cdml>"""

_BOND_CDML = """<cdml version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  <atom id='atom-o' name='O'><point x='40' y='20'/></atom>
</molecule></cdml>"""

_DUPLICATE_MARK_CDML = """<cdml version='26.07'><molecule id='mol-1'>
  <atom id='atom-c' name='C' charge='2'><point x='10' y='20'/>
    <mark type='plus' x='18' y='28' size='10' data-origin='first'/>
    <mark type='plus' x='20' y='30' size='10' data-origin='second'/>
  </atom>
</molecule></cdml>"""


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide an offscreen application without importing legacy fixtures."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _atom_viewport_point(
		tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
		atom_id: str,
		) -> PySide6.QtCore.QPoint:
	"""Return a stable interior viewport point for one projected durable atom."""
	item = tab._controller.projection.durable_items[("atom", atom_id)]
	shape = item.shape()
	bounds = shape.boundingRect()
	for x_step in range(1, 10):
		for y_step in range(1, 10):
			point = PySide6.QtCore.QPointF(
				bounds.left() + bounds.width() * x_step / 10.0,
				bounds.top() + bounds.height() * y_step / 10.0,
			)
			if shape.contains(point):
				return tab.view.mapFromScene(item.mapToScene(point))
	raise AssertionError("projected atom has no interior hit-test point")


#============================================
def _empty_viewport_point(
		tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
		) -> PySide6.QtCore.QPoint:
	"""Return one visible viewport point that does not hit a durable atom."""
	rect = tab.view.viewport().rect().adjusted(12, 12, -12, -12)
	for x_step in range(1, 10):
		for y_step in range(1, 10):
			point = PySide6.QtCore.QPoint(
				rect.left() + rect.width() * x_step // 10,
				rect.top() + rect.height() * y_step // 10,
			)
			if tab.durable_atom_at_viewport_point(point) is None:
				return point
	raise AssertionError("native viewport has no empty hit-test point")


#============================================
def test_public_native_window_routes_cdml_to_rust_without_a_legacy_session(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The standalone window reaches its native controller, not a fallback base."""
	del qapp
	source = tmp_path / "source.cdml"
	source.write_text("<cdml/>", encoding="utf-8")
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	loop = PySide6.QtCore.QEventLoop()
	outcomes = []

	def finish(success: bool) -> None:
		"""Capture the complete admission result and stop the local event loop."""
		outcomes.append(success)
		loop.quit()

	window.local_cdml_open_queue_drained.connect(finish)
	try:
		assert window.open_file_path(str(source))
		loop.exec()
		tab_widget = window.centralWidget()
		assert outcomes == [True] and isinstance(tab_widget, PySide6.QtWidgets.QTabWidget)
		tab = tab_widget.currentWidget()
		assert isinstance(
			tab, ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
		)
		assert tab.file_path == source and not tab.current_snapshot.is_dirty
	finally:
		window.close()


#============================================
def test_clean_pending_undo_requires_refresh_before_tab_or_window_close(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A clean Rust baseline with a stale visible scene cannot be discarded."""
	del qapp
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("atom-c")
	tab.change_selected_atom_element("N")
	replace = tab._controller.replace
	monkeypatch.setattr(tab._controller, "replace", lambda _observation, _latch: False)
	document_tab_module = ferrum_qt.native.ferrum_native_document_tab
	error_type = document_tab_module.FerrumNativeDocumentTabMutationPresentationError
	with pytest.raises(error_type):
		tab.undo()
	assert tab.requires_refresh and not tab.is_dirty
	warnings = []
	monkeypatch.setattr(
		window, "_show_native_file_warning",
		lambda title, message: warnings.append((title, message)),
	)
	window._close_tab_at(window._tab_widget.indexOf(tab))
	close_event = PySide6.QtGui.QCloseEvent()
	window.closeEvent(close_event)
	assert window._tab_widget.indexOf(tab) >= 0 and not close_event.isAccepted()
	assert warnings == [
		(
			"Authoritative Refresh Required",
			"Refresh the authoritative Rust view before closing this tab.",
		),
		(
			"Authoritative Refresh Required",
			"Refresh every pending authoritative Rust view before closing Ferrum-Qt.",
		),
	]
	monkeypatch.setattr(tab._controller, "replace", replace)
	assert tab.refresh_authoritative() and not tab.requires_refresh and not tab.is_dirty
	window._close_tab_at(window._tab_widget.indexOf(tab))
	assert window._tab_widget.indexOf(tab) < 0 and tab._disposed
	window.deleteLater()


#============================================
def test_add_atom_action_maps_one_view_click_to_the_rust_scene_point(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The public action captures one point and selects only the Rust-created atom."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText",
		lambda *_args: ("O", True),
	)
	click = PySide6.QtCore.QPoint(40, 55)
	expected = tab.view.mapToScene(click)
	window._add_atom_action.trigger()
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, click,
	)
	selected = tab._controller.projection.selected_durable_targets()
	created = next(
		atom for atom in tab._document_observation.projection.molecules[0].atoms
		if atom.source_id == selected[0].identifier
	)
	assert created.position.x == expected.x() and created.position.y == expected.y()
	assert len(selected) == 1 and selected[0].identifier == created.source_id
	assert not window._add_atom_action.isChecked()
	tab.save_atomic(tmp_path / "added.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_add_single_bond_action_connects_exact_selected_atoms(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The public edit action commits, selects, and saves one Rust-owned bond."""
	del qapp
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atoms(("atom-c", "atom-o"))
	window._add_single_bond_action.trigger()
	selected = tab._controller.projection.selected_durable_targets()
	assert len(selected) == 1 and selected[0].kind == "bond"
	assert 'type="n1" start="atom-c" end="atom-o"' in tab.current_snapshot.cdml
	assert "Added one Rust-native single bond." in window.statusBar().currentMessage()
	tab.save_atomic(tmp_path / "bonded.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_delete_selected_atom_action_removes_incident_bond_through_rust(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The public delete action commits one undoable Rust topology change."""
	del qapp
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "delete-atom.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atoms(("atom-c", "atom-o"))
	tab.add_single_bond_between_selected_atoms()
	tab.select_atom("atom-o")
	window._delete_atom_action.trigger()
	molecule = tab._document_observation.projection.molecules[0]
	assert tuple(atom.source_id for atom in molecule.atoms) == ("atom-c",)
	assert not molecule.bonds
	assert "Deleted one Rust-native atom" in window.statusBar().currentMessage()
	tab.undo()
	restored = tab._document_observation.projection.molecules[0]
	assert len(restored.atoms) == 2 and len(restored.bonds) == 1
	tab.redo()
	tab.save_atomic(tmp_path / "deleted-atom.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_delete_selected_bond_action_preserves_atoms_through_rust(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The public delete action removes one selected bond and no endpoint atoms."""
	del qapp
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "delete-bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atoms(("atom-c", "atom-o"))
	created = tab.add_single_bond_between_selected_atoms()
	bond_id = created.observation.projection.molecules[0].bonds[0].source_id
	tab.select_bond(bond_id)
	window._delete_bond_action.trigger()
	molecule = tab._document_observation.projection.molecules[0]
	assert len(molecule.atoms) == 2 and not molecule.bonds
	assert "Deleted one Rust-native bond" in window.statusBar().currentMessage()
	tab.undo()
	assert len(tab._document_observation.projection.molecules[0].bonds) == 1
	tab.redo()
	tab.save_atomic(tmp_path / "deleted-bond.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_change_bond_order_action_uses_the_closed_rust_enum(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The public action changes one selected bond without interpreting CDML in Qt."""
	del qapp
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond-order.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atoms(("atom-c", "atom-o"))
	created = tab.add_single_bond_between_selected_atoms()
	bond_id = created.observation.projection.molecules[0].bonds[0].source_id
	tab.select_bond(bond_id)
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getItem",
		lambda *_args: ("Double", True),
	)
	window._change_bond_order_action.trigger()
	bond = tab._document_observation.projection.molecules[0].bonds[0]
	assert bond.source_type == "n2"
	assert "double" in window.statusBar().currentMessage()
	tab.save_atomic(tmp_path / "double-bond-action.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_draw_single_bond_drag_commits_rust_and_retires_preview(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""One visible atom-to-atom drag creates no Qt model and commits one Rust bond."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	start = _atom_viewport_point(tab, "atom-c")
	end = _atom_viewport_point(tab, "atom-o")
	window._draw_single_bond_action.trigger()
	PySide6.QtTest.QTest.mousePress(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
	)
	assert window._line_gesture_intent.preview is not None
	PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
	PySide6.QtTest.QTest.mouseRelease(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
	)
	selected = tab._controller.projection.selected_durable_targets()
	assert len(selected) == 1 and selected[0].kind == "bond"
	assert 'type="n1" start="atom-c" end="atom-o"' in tab.current_snapshot.cdml
	assert window._line_gesture_intent.preview is None
	assert window._draw_single_bond_action.isChecked()
	PySide6.QtTest.QTest.keyClick(
		tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape,
	)
	assert window._line_gesture_intent is None
	assert not window._draw_single_bond_action.isChecked()
	tab.save_atomic(tmp_path / "drag-bond.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_draw_single_bond_rejects_revision_changed_during_drag(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A gesture captured before another edit cannot commit against the new revision."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	start = _atom_viewport_point(tab, "atom-c")
	end = _atom_viewport_point(tab, "atom-o")
	warnings = []
	monkeypatch.setattr(
		window, "_show_native_file_warning",
		lambda title, message: warnings.append((title, message)),
	)
	window._draw_single_bond_action.trigger()
	PySide6.QtTest.QTest.mousePress(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
	)
	tab.select_atom("atom-c")
	tab.change_selected_atom_element("N")
	PySide6.QtTest.QTest.mouseRelease(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
	)
	assert not tab._document_observation.projection.molecules[0].bonds
	assert warnings[-1][0] == "Native Draw Bond Stale"
	assert window._line_gesture_intent is None
	tab.save_atomic(tmp_path / "stale-drag.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_draw_single_bond_to_empty_space_creates_carbon_and_bond_atomically(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Empty-space release creates one default carbon and bond in one Rust edit."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "extend-bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	start = _atom_viewport_point(tab, "atom-c")
	end = _empty_viewport_point(tab)
	end_scene = tab.view.mapToScene(end)
	window._draw_single_bond_action.trigger()
	PySide6.QtTest.QTest.mousePress(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
	)
	PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
	PySide6.QtTest.QTest.mouseRelease(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
	)
	molecule = tab._document_observation.projection.molecules[0]
	selected = tab._controller.projection.selected_durable_targets()
	assert len(molecule.atoms) == 3 and len(molecule.bonds) == 1
	assert molecule.atoms[-1].element == "C"
	assert (molecule.atoms[-1].position.x, molecule.atoms[-1].position.y) == (
		end_scene.x(), end_scene.y(),
	)
	assert len(selected) == 1 and selected[0].kind == "atom"
	assert "carbon and single bond" in window.statusBar().currentMessage()
	tab.undo()
	assert len(tab._document_observation.projection.molecules[0].atoms) == 2
	assert not tab._document_observation.projection.molecules[0].bonds
	tab.save_atomic(tmp_path / "empty-space-bond.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_move_atom_drag_preserves_pointer_offset_and_commits_one_rust_move(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The move tool translates the Rust anchor by the visible pointer delta."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "move-atom.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	start = _atom_viewport_point(tab, "atom-c")
	end = _empty_viewport_point(tab)
	start_pointer = tab.view.mapToScene(start)
	end_pointer = tab.view.mapToScene(end)
	anchor = tab.durable_atom_scene_position("atom-c")
	expected = anchor + (end_pointer - start_pointer)
	window._move_atom_action.trigger()
	PySide6.QtTest.QTest.mousePress(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
	)
	assert window._line_gesture_intent.preview is not None
	PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
	PySide6.QtTest.QTest.mouseRelease(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
	)
	atom = tab._document_observation.projection.molecules[0].atoms[0]
	selected = tab._controller.projection.selected_durable_targets()
	assert (atom.position.x, atom.position.y) == (expected.x(), expected.y())
	assert len(selected) == 1 and selected[0].identifier == "atom-c"
	assert window._line_gesture_intent.preview is None
	assert window._move_atom_action.isChecked()
	assert "Moved one Rust-native atom" in window.statusBar().currentMessage()
	tab.undo()
	restored = tab._document_observation.projection.molecules[0].atoms[0].position
	assert (restored.x, restored.y) == (10.0, 20.0)
	window._move_atom_action.trigger()
	tab.save_atomic(tmp_path / "moved-atom.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_add_atom_click_rejects_a_locally_stale_captured_revision(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A document change between activation and click cannot insert another atom."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText",
		lambda *_args: ("O", True),
	)
	warnings = []
	monkeypatch.setattr(
		window, "_show_native_file_warning",
		lambda title, message: warnings.append((title, message)),
	)
	window._add_atom_action.trigger()
	tab.select_atom("atom-c")
	tab.change_selected_atom_element("N")
	before_ids = tuple(
		atom.source_id for atom in tab._document_observation.projection.molecules[0].atoms
	)
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, PySide6.QtCore.QPoint(40, 55),
	)
	after_ids = tuple(
		atom.source_id for atom in tab._document_observation.projection.molecules[0].atoms
	)
	assert after_ids == before_ids and warnings[-1][0] == "Native Add Atom Stale"
	tab.save_atomic(tmp_path / "stale-click.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_add_atom_chooser_targets_the_selected_durable_molecule(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Multiple molecules are named for the user but submitted by opaque Rust ID."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_MULTI_MOLECULE_CDML, "multi.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText",
		lambda *_args: ("O", True),
	)

	def choose_second(_parent: object, _title: str, _label: str,
			items: tuple[str, ...], _current: int, _editable: bool) -> tuple[str, bool]:
		"""Select the second explicit source-ordered molecule choice."""
		return items[1], True

	monkeypatch.setattr(PySide6.QtWidgets.QInputDialog, "getItem", choose_second)
	window._add_atom_action.trigger()
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, PySide6.QtCore.QPoint(40, 55),
	)
	selected = tab._controller.projection.selected_durable_targets()[0].identifier
	first_ids = tuple(
		atom.source_id for atom in tab._document_observation.projection.molecules[0].atoms
	)
	second_ids = tuple(
		atom.source_id for atom in tab._document_observation.projection.molecules[1].atoms
	)
	assert selected not in first_ids and selected in second_ids
	tab.save_atomic(tmp_path / "multi-added.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_atom_mark_actions_toggle_every_closed_kind_through_rust(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Every visible native mark action changes Rust state and retains atom selection."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "marks.cdml",
	)
	window._register_native_tab(tab, activate=True)
	warnings = []
	monkeypatch.setattr(
		window, "_show_native_file_warning",
		lambda title, message: warnings.append((title, message)),
	)
	tab.select_atom("atom-c")
	qapp.processEvents()

	for kind_name, action in window._atom_mark_actions.items():
		assert action.isEnabled()
		action.trigger()
		assert warnings == []
		atom = tab.selected_atom_projection()
		assert len(atom.marks) == 1
		assert atom.marks[0].kind == getattr(ferrum_chem.AtomMarkKindV1, kind_name)
		assert tab._controller.projection.selected_durable_targets()[0].identifier == "atom-c"
		action.trigger()
		assert tab.selected_atom_projection().marks == []

	assert "Toggled one Rust-native atom mark." in window.statusBar().currentMessage()
	assert warnings == []
	tab.save_atomic(tmp_path / "marks.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_remove_atom_mark_chooser_uses_exact_duplicate_ordinal(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The chooser removes the selected duplicate without string-derived mutation."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_DUPLICATE_MARK_CDML, "duplicate-marks.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("atom-c")
	qapp.processEvents()
	assert window._remove_atom_mark_action.isEnabled()

	def choose_second(_parent: object, _title: str, _label: str,
			items: tuple[str, ...], _current: int, _editable: bool) -> tuple[str, bool]:
		"""Select the second source-ordered plus mark from the explicit chooser."""
		return items[1], True

	monkeypatch.setattr(PySide6.QtWidgets.QInputDialog, "getItem", choose_second)
	window._remove_atom_mark_action.trigger()
	atom = tab.selected_atom_projection()
	assert atom.formal_charge == 1 and len(atom.marks) == 1
	assert atom.marks[0].same_type_ordinal == 0
	assert "data-origin=\"first\"" in tab.current_snapshot.cdml
	assert "data-origin=\"second\"" not in tab.current_snapshot.cdml
	assert "Removed one Rust-native atom mark." in window.statusBar().currentMessage()
	tab.undo()
	assert len(tab.selected_atom_projection().marks) == 2
	tab.save_atomic(tmp_path / "duplicate-marks.cdml")
	window.close()
	window.deleteLater()

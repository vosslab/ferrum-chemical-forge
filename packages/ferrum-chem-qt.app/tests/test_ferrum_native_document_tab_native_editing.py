"""Native document-tab lifecycle and editing behavior."""

# Standard Library
import hashlib
import os
import pathlib

# Qt reads the platform selection before this isolated test creates an application.
os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
import pytest

# local test modules
from test_ferrum_native_document_tab import (
	_BOND_CDML,
	_Controller,
	_EDITABLE_CDML,
	_LIVE_SMARTS_MULTIROW_CDML,
	_ObserveFailureSession,
	_Snapshot,
	_UNRENDERABLE_LABEL_CDML,
	_install_transient_overlay,
	_tab,
	_unattached_transient_overlay,
)

# local repo modules
import ferrum_qt.ferrum.document_tab


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide an isolated offscreen QApplication without the legacy app host."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


def _sealed_live_smarts_qt_session() -> object:
	"""Load one real installed ABI-5 bridge, never a source or fixture substitute."""
	configured = os.environ.get("FERRUM_SMARTS_QT_SEALED_WHEEL_ROOT")
	if not configured:
		pytest.skip("requires an isolated installed ABI-5 native and Qt wheel")
	root = pathlib.Path(configured).resolve()
	wheel = pathlib.Path(os.environ["FERRUM_SMARTS_QT_NATIVE_WHEEL"]).resolve()
	qt_wheel = pathlib.Path(os.environ["FERRUM_SMARTS_QT_WHEEL"]).resolve()
	for path, digest_name in (
		(wheel, "FERRUM_SMARTS_QT_NATIVE_WHEEL_SHA256"),
		(qt_wheel, "FERRUM_SMARTS_QT_WHEEL_SHA256"),
	):
		expected = os.environ.get(digest_name)
		if not path.is_file() or len(expected or "") != 64:
			raise RuntimeError("sealed SMARTS Qt artifact provenance is incomplete")
		if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
			raise RuntimeError("sealed SMARTS Qt artifact digest mismatch")
	import ferrum_chem
	import ferrum_qt
	if root not in pathlib.Path(ferrum_chem.__file__).resolve().parents:
		raise RuntimeError("sealed SMARTS Qt test imported a non-installed native bridge")
	if root not in pathlib.Path(ferrum_qt.__file__).resolve().parents:
		raise RuntimeError("sealed SMARTS Qt test imported a non-installed Qt wheel")
	session = ferrum_chem.DocumentSession.load(_LIVE_SMARTS_MULTIROW_CDML)
	session._publish_live_render_plan_v1(session.snapshot().revision)
	return session


#============================================
def _renderer_issued_smarts_overlay_item(paint: object) -> PySide6.QtWidgets.QGraphicsItemGroup:
	"""Project only real private-bridge bounds into a noninteractive Qt overlay."""
	root = PySide6.QtWidgets.QGraphicsItemGroup()
	root.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	for left, top, right, bottom in paint.atom_bounds:
		item = PySide6.QtWidgets.QGraphicsRectItem(root)
		item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		item.setRect(float(left), float(top), float(right - left), float(bottom - top))
	return root


#============================================
def test_sealed_live_bridge_multiple_rows_replay_and_restore_failure_retire(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Use the installed PyO3 bridge through two redemptions and fail closed."""
	del qapp
	import ferrum_chem
	native_session = _sealed_live_smarts_qt_session()
	class _RetirementSpy:
		"""Record only the actual private retirement call while delegating the wheel bridge."""
		def __init__(self, session: object) -> None:
			self._session = session
			self.calls = 0
		def __getattr__(self, name: str) -> object:
			return getattr(self._session, name)
		def _retire_live_document_smarts_query_v1(self) -> object:
			self.calls += 1
			return self._session._retire_live_document_smarts_query_v1()
	session = _RetirementSpy(native_session)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", session, _Controller(),
	)
	calls_before_query = session.calls
	assert calls_before_query == 1
	scene = PySide6.QtWidgets.QGraphicsScene(tab)
	tab._view.setScene(scene)
	run = session._run_live_document_smarts_query_v1("[#6]", 3, 3)
	assert [(item.source_order, item.match_count) for item in run.molecules] == [
		(0, 1), (1, 1), (2, 1),
	]
	receipt = run.receipt
	first_item = _renderer_issued_smarts_overlay_item(
		session._show_live_document_smarts_match_v1(receipt, 0),
	)
	scene.addItem(first_item)
	token = tab._install_live_smarts_query_overlay_v1(first_item, receipt)
	assert session.calls == calls_before_query
	second_item = _renderer_issued_smarts_overlay_item(
		session._show_live_document_smarts_match_v1(receipt, 1),
	)
	tab._replace_live_smarts_query_overlay_v1(second_item)
	assert (
		first_item.scene() is None
		and second_item.scene() is scene
		and tab._live_smarts_receipt_v1 is receipt
		and tab._live_smarts_active_run_token_v1 == token
		and session.calls == calls_before_query
	)
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError,
			match="^SMARTS query cannot continue$"):
		session._show_live_document_smarts_match_v1(receipt, 0)
	assert tab._live_smarts_overlay_item_v1 is second_item
	candidate = _renderer_issued_smarts_overlay_item(
		session._show_live_document_smarts_match_v1(receipt, 2),
	)
	monkeypatch.setattr(tab, "_attach_live_smarts_overlay_item_v1",
		lambda _scene, _item: (_ for _ in ()).throw(RuntimeError("injected attach failure")))
	monkeypatch.setattr(tab, "_restore_replaced_live_smarts_overlay_item_v1",
		lambda _scene, _item: False)
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._replace_live_smarts_query_overlay_v1(candidate)
	assert (
		second_item.scene() is None
		and candidate.scene() is None
		and tab._live_smarts_overlay_item_v1 is None
		and tab._live_smarts_receipt_v1 is None
		and tab._live_smarts_active_run_token_v1 is None
		and session.calls == calls_before_query + 1
	)
	for row_index in (0, 1, 2):
		with pytest.raises(ferrum_chem.LiveDocumentSmartsError,
				match="^SMARTS query cannot continue$"):
			session._show_live_document_smarts_match_v1(receipt, row_index)
	tab.dispose()


#============================================
def test_live_overlay_replacement_failure_restores_prior_visual_and_run(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A canvas attach failure cannot consume the prior visual or opaque run state."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	tab, _controller = _tab(current, current, True)
	prior_item, token = _install_transient_overlay(tab)
	receipt = tab._live_smarts_receipt_v1
	candidate = _unattached_transient_overlay()
	def fail_attach(_scene: object, _item: object) -> None:
		"""Model a renderer scene ownership failure after prior paint is removed."""
		raise RuntimeError("injected scene attachment failure")
	monkeypatch.setattr(tab, "_attach_live_smarts_overlay_item_v1", fail_attach)
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._replace_live_smarts_query_overlay_v1(candidate)
	assert (
		prior_item.scene() is not None
		and candidate.scene() is None
		and tab._live_smarts_overlay_item_v1 is prior_item
		and tab._live_smarts_receipt_v1 is receipt
		and tab._live_smarts_active_run_token_v1 == token
	)
	tab.dispose()


#============================================
def test_disposal_retires_live_overlay_before_controller_disposal(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Close/shutdown disposal cannot leave transient overlay paint in a scene."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	tab, controller = _tab(current, current, True)
	item, _token = _install_transient_overlay(tab)
	tab.dispose()
	assert item.scene() is None and controller.disposed


#============================================
def test_valid_native_observation_installs_snapshot_and_dirty_state(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A valid Rust observation becomes the tab's user-visible backend state."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	tab, controller = _tab(current, current, True)
	assert tab.current_snapshot is current and tab.is_dirty
	assert controller.installed is not None
	tab.dispose()


#============================================
def test_confirmed_save_adopts_saved_snapshot_and_destination_title(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Confirmed publication advances only with its matching render observation."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	saved = _Snapshot(4, "b" * 64, False)
	tab, controller = _tab(current, saved, True)
	tab.save_atomic(str(tmp_path / "ethanol.cdml"))
	assert (
		tab.title == "ethanol.cdml"
		and tab.current_snapshot is saved
		and controller.installed.document.snapshot is saved
	)
	tab.dispose()


#============================================
def test_unconfirmed_save_preserves_dirty_snapshot_and_title(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""An uncertain publication cannot make the tab claim the document was saved."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	saved = _Snapshot(4, "b" * 64, False)
	tab, _controller = _tab(current, saved, False)
	tab.save_atomic(str(tmp_path / "ethanol.cdml"))
	assert tab.title == "Untitled" and tab.current_snapshot is current and tab.is_dirty
	tab.dispose()


#============================================
def test_confirmed_save_with_rejected_refresh_preserves_prior_presentation(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A post-save projection failure cannot split tab and controller provenance."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	saved = _Snapshot(4, "b" * 64, False)
	tab, controller = _tab(current, saved, True, (True, False))
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab.save_atomic(str(tmp_path / "ethanol.cdml"))
	assert (
		tab.current_snapshot is current
		and controller.installed.document.snapshot is current
		and tab.title == "Untitled"
		and tab.file_path is None
	)
	tab.dispose()


#============================================
def test_disposal_rejects_late_native_operations(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A closed tab cannot revive projection delivery or save through a stale view."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	tab, controller = _tab(current, current, True)
	tab.dispose()
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab.save_atomic(str(tmp_path / "ethanol.cdml"))
	assert controller.disposed and controller.installed is not None


#============================================
def test_native_selection_edit_undo_redo_restores_durable_atom(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The Rust element operation retains identity through history and persistence."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	changed = tab.change_selected_atom_element("N").observation.snapshot
	selected_after_change = tab._controller.projection.selected_durable_targets()
	undone = tab.undo().observation.snapshot
	redone = tab.redo().observation.snapshot
	selected = tab._controller.projection.selected_durable_targets()
	assert changed.revision < undone.revision < redone.revision and tab.is_dirty
	assert (
		len(selected_after_change) == 1
		and selected_after_change[0].kind == "atom"
		and selected_after_change[0].identifier == "atom-c"
		and len(selected) == 1
		and selected[0].identifier == "atom-c"
	)
	tab.save_atomic(tmp_path / "changed-element.cdml")
	reopened = ferrum_chem.DocumentSession.load(
		(tmp_path / "changed-element.cdml").read_text(encoding="utf-8"),
	)
	atom = reopened.observe_render(0).document.projection.molecules[0].atoms[0]
	assert atom.source_id == "atom-c" and atom.element == "N"
	tab.dispose()


#============================================
def test_change_element_refuses_ineligible_selection_and_invalid_rust_input(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Selection preconditions and Rust validation leave installed truth unchanged."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "change-element-preconditions.cdml",
	)
	prior = tab.current_snapshot
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab.change_selected_atom_element("N")
	tab.select_atoms(("atom-c", "atom-o"))
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab.change_selected_atom_element("N")
	created = tab.add_single_bond_between_selected_atoms()
	bond_id = created.observation.projection.molecules[0].bonds[0].source_id
	tab.select_bond(bond_id)
	bond_selected = tab.current_snapshot
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab.change_selected_atom_element("N")
	tab.select_atom("atom-c")
	with pytest.raises(ferrum_chem.OperationValidationError):
		tab.change_selected_atom_element("Xx")
	assert tab.current_snapshot is bond_selected and prior.revision == 0
	tab.dispose()


#============================================
def test_native_add_atom_uses_rust_identity_point_history_and_save(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""One bounded insertion stays Rust-owned through selection, history, and reopen."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	choice = tab.durable_molecule_choices()[0]
	result = tab.add_atom_at(choice.object_id, "O", 30.5, 40.25)
	identifier = tab._controller.projection.selected_durable_targets()[0].identifier
	assert result.observation.snapshot.revision == 1 and identifier.startswith("ferrum-atom-v1-")
	assert 'x="30.5" y="40.25" z="0"' in result.observation.snapshot.cdml
	tab.undo()
	tab.redo()
	output = tmp_path / "inserted.cdml"
	tab.save_atomic(output)
	reopened = ferrum_chem.DocumentSession.load(output.read_text(encoding="utf-8")).observe(0)
	reopened_ids = tuple(atom.source_id for atom in reopened.projection.molecules[0].atoms)
	assert identifier in reopened_ids and not tab.is_dirty
	tab.dispose()


#============================================
def test_native_add_atom_refuses_molecule_without_installed_rust_render_plan(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A rich-label molecule cannot allocate an invisible canvas-authored atom."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_UNRENDERABLE_LABEL_CDML, "unrenderable-label.cdml",
	)
	try:
		molecule_id = tab.durable_molecule_choices()[0].object_id
		assert not any(
			plan.molecule.id == molecule_id for plan in tab._render_observation.molecule_plans
		)
		before = tab.current_snapshot
		before_atom_ids = tuple(
			atom.source_id for atom in tab.current_document_observation().projection.molecules[0].atoms
		)
		before_selection = tab._controller.projection.selected_durable_targets()
		assert tab.canvas_authorable_molecule_choices() == ()
		with pytest.raises(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabUnrenderableMoleculeError,
		) as raised:
			tab.add_atom_at(molecule_id, "O", 30.5, 40.25)
		assert raised.value.molecule_object_id == molecule_id
		after = tab.current_snapshot
		after_atom_ids = tuple(
			atom.source_id for atom in tab.current_document_observation().projection.molecules[0].atoms
		)
		assert after.revision == before.revision and after.digest == before.digest
		assert after.cdml == before.cdml and after_atom_ids == before_atom_ids
		assert tab._controller.projection.selected_durable_targets() == before_selection
		assert not tab.requires_refresh and not any(
			atom_id.startswith("ferrum-atom-v1-") for atom_id in after_atom_ids
		)
	finally:
		tab.dispose()


#============================================
def test_native_add_double_bond_uses_rust_identity_history_and_save(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Two selected atoms become one durable Rust bond through save and reopen."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	result = tab.add_bond_between_atoms(
		"atom-c", "atom-o", ferrum_chem.DocumentBondPresentationV1.normal_double,
	)
	selected_bond = tab.selected_bond_projection()
	bond_id = selected_bond.source_id
	assert result.observation.snapshot.revision == 1
	assert selected_bond.source_type == "n2"
	assert bond_id.startswith("ferrum-bond-v1-")
	tab.undo()
	assert "<bond" not in tab.current_snapshot.cdml
	tab.redo()
	output = tmp_path / "bonded.cdml"
	tab.save_atomic(output)
	reopened = ferrum_chem.DocumentSession.load(output.read_text(encoding="utf-8")).observe(0)
	bond = reopened.projection.molecules[0].bonds[0]
	assert bond.source_id == bond_id and bond.source_type == "n2" and not tab.is_dirty
	tab.dispose()


#============================================
def test_native_add_single_bond_requires_exactly_two_selected_atoms(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Selection mistakes are rejected before a Rust candidate or revision exists."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	tab.select_atom("atom-c")
	before = tab.current_snapshot
	with pytest.raises(
		ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError,
	):
		tab.add_single_bond_between_selected_atoms()
	assert tab.current_snapshot is before and not tab.is_dirty and not tab.requires_refresh
	tab.dispose()


#============================================
def test_native_add_bonded_atom_is_one_rust_history_entry(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A new atom and its bond appear and disappear through one Rust revision."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bonded-atom.cdml",
	)
	import ferrum_chem
	result = tab.add_bonded_atom_at(
		"atom-c", "N", 60.0, 40.0,
		ferrum_chem.DocumentBondPresentationV1.normal_triple,
	)
	molecule = result.observation.projection.molecules[0]
	selected = tab._controller.projection.selected_durable_targets()
	assert result.observation.snapshot.revision == 1
	assert len(molecule.atoms) == 3 and len(molecule.bonds) == 1
	assert molecule.atoms[-1].source_id.startswith("ferrum-atom-v1-")
	assert (molecule.atoms[-1].position.x, molecule.atoms[-1].position.y) == (60.0, 40.0)
	assert molecule.bonds[0].source_id.startswith("ferrum-bond-v1-")
	assert molecule.bonds[0].source_type == "n3"
	assert len(selected) == 1 and selected[0].kind == "atom"
	undone = tab.undo().observation.projection.molecules[0]
	assert len(undone.atoms) == 2 and not undone.bonds
	redone = tab.redo().observation.projection.molecules[0]
	assert len(redone.atoms) == 3 and len(redone.bonds) == 1
	tab.dispose()


#============================================
def test_native_move_atom_uses_rust_position_history_and_selection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""One exact scene point becomes the authoritative Rust atom coordinate."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "move-atom.cdml",
	)
	result = tab.move_atom_to("atom-c", 55.5, 66.25)
	atom = result.observation.projection.molecules[0].atoms[0]
	selected = tab._controller.projection.selected_durable_targets()
	assert result.observation.snapshot.revision == 1
	assert (atom.position.x, atom.position.y, atom.position.z) == (55.5, 66.25, 0.0)
	assert len(selected) == 1 and selected[0].identifier == "atom-c"
	undone = tab.undo().observation.projection.molecules[0].atoms[0]
	assert (undone.position.x, undone.position.y) == (0.0, 0.0)
	redone = tab.redo().observation.projection.molecules[0].atoms[0]
	assert (redone.position.x, redone.position.y) == (55.5, 66.25)
	tab.dispose()


#============================================
def test_native_delete_atom_removes_incident_bonds_and_round_trips(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""One selected atom deletion stays Rust-owned through undo and save."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "delete-atom.cdml",
	)
	tab.select_atoms(("atom-c", "atom-o"))
	tab.add_single_bond_between_selected_atoms()
	tab.select_atom("atom-o")
	deleted = tab.delete_selected_atom().observation
	molecule = deleted.projection.molecules[0]
	assert tuple(atom.source_id for atom in molecule.atoms) == ("atom-c",)
	assert not molecule.bonds
	restored = tab.undo().observation.projection.molecules[0]
	assert len(restored.atoms) == 2 and len(restored.bonds) == 1
	tab.redo()
	output = tmp_path / "deleted.cdml"
	tab.save_atomic(output)
	reopened = ferrum_chem.DocumentSession.load(output.read_text(encoding="utf-8")).observe(0)
	assert tuple(atom.source_id for atom in reopened.projection.molecules[0].atoms) == ("atom-c",)
	assert not reopened.projection.molecules[0].bonds
	tab.dispose()


#============================================
def test_native_delete_bond_preserves_atoms_and_round_trips(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""One selected bond deletion stays Rust-owned through undo and save."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "delete-bond.cdml",
	)
	tab.select_atoms(("atom-c", "atom-o"))
	created = tab.add_single_bond_between_selected_atoms()
	bond_id = created.observation.projection.molecules[0].bonds[0].source_id
	tab.select_bond(bond_id)
	deleted = tab.delete_selected_bond().observation.projection.molecules[0]
	assert len(deleted.atoms) == 2 and not deleted.bonds
	restored = tab.undo().observation.projection.molecules[0]
	assert len(restored.atoms) == 2 and len(restored.bonds) == 1
	tab.redo()
	output = tmp_path / "deleted-bond.cdml"
	tab.save_atomic(output)
	reopened = ferrum_chem.DocumentSession.load(output.read_text(encoding="utf-8")).observe(0)
	assert len(reopened.projection.molecules[0].atoms) == 2
	assert not reopened.projection.molecules[0].bonds
	tab.dispose()


#============================================
def test_native_bond_order_change_round_trips_and_selects_the_same_bond(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""One selected Rust bond becomes a visible double bond and remains undoable."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond-order.cdml",
	)
	tab.select_atoms(("atom-c", "atom-o"))
	created = tab.add_single_bond_between_selected_atoms()
	bond_id = created.observation.projection.molecules[0].bonds[0].source_id
	tab.select_bond(bond_id)
	changed = tab.set_selected_bond_order(
		ferrum_chem.DocumentBondOrderV1.double,
	).observation
	assert changed.projection.molecules[0].bonds[0].source_type == "n2"
	selected = tab._controller.projection.selected_durable_targets()
	assert len(selected) == 1 and selected[0].identifier == bond_id
	plan = tab._session.observe_render(changed.snapshot.revision).molecule_plans[0].plan
	bond_batch = next(batch for batch in plan.batches if batch.target.record_id.kind == "Bond")
	assert len(bond_batch.operations) == 2
	assert tab.undo().observation.projection.molecules[0].bonds[0].source_type == "n1"
	assert tab.redo().observation.projection.molecules[0].bonds[0].source_type == "n2"
	output = tmp_path / "double-bond.cdml"
	tab.save_atomic(output)
	reopened = ferrum_chem.DocumentSession.load(output.read_text(encoding="utf-8")).observe(0)
	assert reopened.projection.molecules[0].bonds[0].source_type == "n2"
	tab.dispose()


#============================================
def test_native_add_atom_refresh_selects_only_after_replacement_succeeds(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An accepted insertion retains its Rust ID until Refresh installs its scene."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	choice = tab.durable_molecule_choices()[0]
	prior = tab.current_snapshot
	replace = tab._controller.replace
	monkeypatch.setattr(tab._controller, "replace", lambda _observation, _latch: False)
	with pytest.raises(
		ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError,
	):
		tab.add_atom_at(choice.object_id, "O", 30.5, 40.25)
	assert tab.current_snapshot is prior and tab.requires_refresh and tab.is_dirty
	assert tab._controller.projection.selected_durable_targets() == ()
	monkeypatch.setattr(tab._controller, "replace", replace)
	assert tab.refresh_authoritative() and not tab.requires_refresh
	selected = tab._controller.projection.selected_durable_targets()
	assert len(selected) == 1 and selected[0].identifier.startswith("ferrum-atom-v1-")
	tab.dispose()


#============================================
def test_accepted_mutation_projection_failure_blocks_save_until_exact_refresh(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""A scene failure never hides an accepted Rust edit behind the prior display."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	replace = tab._controller.replace
	monkeypatch.setattr(tab._controller, "replace", lambda _observation, _latch: False)
	with pytest.raises(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError,
	):
		tab.change_selected_atom_element("N")
	assert tab.current_snapshot is prior and tab.requires_refresh and tab.is_dirty
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab.save_atomic(str(tmp_path / "blocked-native-edit.cdml"))
	monkeypatch.setattr(tab._controller, "replace", replace)
	assert tab.refresh_authoritative() and not tab.requires_refresh
	tab.dispose()


#============================================
def test_stale_native_edit_keeps_the_installed_scene_truth(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A stale revision rejection cannot replace the current Ferrum projection."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	tab._session.submit(
		prior.revision, ferrum_chem.DocumentOperationV1.set_atom_element("atom-c", "N"),
	)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		tab.change_selected_atom_element("O")
	assert tab.current_snapshot is prior and not tab.requires_refresh
	tab.dispose()


#============================================
def test_post_accept_observe_exception_retains_authoritative_pending_state(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An observation exception after submit cannot make a clean tab closable."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	session = tab._session
	tab._session = _ObserveFailureSession(session)
	with pytest.raises(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError,
	):
		tab.change_selected_atom_element("N")
	assert tab.current_snapshot is prior and tab.requires_refresh and tab.is_dirty
	assert (
		tab._pending_snapshot.revision > prior.revision
		and tab._pending_snapshot.digest != prior.digest
	)
	tab._session = session
	assert tab.refresh_authoritative() and not tab.requires_refresh
	tab.dispose()


#============================================
def test_post_accept_replace_exception_retains_authority_until_refresh(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A controller exception after Rust acceptance preserves recovery ownership."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	replace = tab._controller.replace
	def fail_replace(_observation: object, _latch: object) -> bool:
		"""Raise after the mutation result has become Rust authority."""
		raise RuntimeError("injected replacement failure")
	monkeypatch.setattr(tab._controller, "replace", fail_replace)
	with pytest.raises(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError,
	):
		tab.change_selected_atom_element("N")
	assert tab.current_snapshot is prior and tab.requires_refresh and tab.is_dirty
	monkeypatch.setattr(tab._controller, "replace", replace)
	assert tab.refresh_authoritative() and not tab.requires_refresh
	tab.dispose()

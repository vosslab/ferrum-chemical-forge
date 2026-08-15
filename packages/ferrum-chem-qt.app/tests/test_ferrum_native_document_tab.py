"""Behavioral coverage for the isolated Rust-owned native document tab."""

# Standard Library
import dataclasses
import os
import pathlib


# Qt reads the platform selection before this isolated test creates an application.
os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab


_EDITABLE_CDML = (
	'<cdml version="26.08"><molecule id="molecule-1"><atom id="atom-c" name="C">'
	'<point x="0cm" y="0cm"/></atom></molecule></cdml>'
)

_BOND_CDML = (
	'<cdml version="26.08"><molecule id="molecule-1">'
	'<atom id="atom-c" name="C"><point x="0" y="0"/></atom>'
	'<atom id="atom-o" name="O"><point x="30" y="0"/></atom>'
	'</molecule></cdml>'
)


#============================================
class _ObserveFailureSession:
	"""Real-session wrapper that fails only the post-accept render observation."""

	#============================================
	def __init__(self, session: object) -> None:
		"""Retain the exact real Rust session for every non-render call."""
		self._session = session

	#============================================
	def __getattr__(self, name: str) -> object:
		"""Delegate the closed PyO3 API unchanged except for rendering."""
		return getattr(self._session, name)

	#============================================
	def observe_render(self, _revision: int) -> object:
		"""Make the presentation follow-up fail after Rust already accepted."""
		raise RuntimeError("injected render observation failure")


@dataclasses.dataclass(frozen=True, slots=True)
class _Snapshot:
	"""Compact immutable Rust snapshot fixture."""

	revision: int
	digest: str
	is_dirty: bool


@dataclasses.dataclass(frozen=True, slots=True)
class _DocumentObservation:
	"""Compact document envelope used by the render fixture."""

	snapshot: _Snapshot


@dataclasses.dataclass(frozen=True, slots=True)
class _RenderObservation:
	"""Render fixture that carries the same durable snapshot provenance."""

	document: _DocumentObservation


@dataclasses.dataclass(frozen=True, slots=True)
class _Outcome:
	"""Immutable publication confirmation fact."""

	is_confirmed: bool


@dataclasses.dataclass(frozen=True, slots=True)
class _Publication:
	"""Rust publication result fixture."""

	snapshot: _Snapshot
	outcome: _Outcome


class _Session:
	"""Owned-value native session fake with explicit current snapshots."""

	#============================================
	def __init__(self, current: _Snapshot, saved: _Snapshot,
			confirmed: bool) -> None:
		"""Retain explicit backend facts for one deterministic interaction."""
		self._current = current
		self._saved = saved
		self._confirmed = confirmed
		self._published = False

	#============================================
	def snapshot(self) -> _Snapshot:
		"""Return the current backend snapshot."""
		return self._current

	#============================================
	def observe_render(self, revision: int) -> _RenderObservation:
		"""Return the observation associated with the requested current revision."""
		snapshot = self._saved if self._published else self._current
		if revision != snapshot.revision:
			raise ValueError("unexpected revision")
		return _RenderObservation(_DocumentObservation(snapshot))

	#============================================
	def save_atomic(self, _path: object, revision: int) -> _Publication:
		"""Publish only the current requested revision."""
		if revision != self._current.revision:
			raise ValueError("unexpected revision")
		self._published = self._confirmed
		return _Publication(self._saved, _Outcome(self._confirmed))


class _Controller:
	"""Projection controller fake that retains accepted latches and terminal state."""

	#============================================
	def __init__(self, acceptances: tuple[bool, ...] = (True,)) -> None:
		"""Create one current generation with deterministic render decisions."""
		self.generation = 0
		self._acceptances = iter(acceptances)
		self.disposed = False
		self.installed: _RenderObservation | None = None

	#============================================
	def replace(self, observation: _RenderObservation, latch: object) -> bool:
		"""Accept only current non-terminal delivery with matching provenance."""
		if self.disposed or latch.generation != self.generation:
			return False
		if observation.document.snapshot.revision != latch.revision:
			return False
		if observation.document.snapshot.digest != latch.digest:
			return False
		accepted = next(self._acceptances)
		if accepted:
			self.installed = observation
		return accepted

	#============================================
	def dispose(self) -> None:
		"""Make all later render deliveries terminally stale."""
		self.disposed = True
		self.generation += 1


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide an isolated offscreen QApplication without the legacy app host."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _tab(current: _Snapshot, saved: _Snapshot, confirmed: bool,
		acceptances: tuple[bool, ...] = (True, True)) -> tuple[
			ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			_Controller,
		]:
	"""Build a tab only through its explicitly private owned-value fixture seam."""
	controller = _Controller(acceptances)
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", _Session(current, saved, confirmed), controller,
	)
	return tab, controller


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
	with pytest.raises(ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTabError):
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
	with pytest.raises(ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTabError):
		tab.save_atomic(str(tmp_path / "ethanol.cdml"))
	assert controller.disposed and controller.installed is not None


#============================================
def test_native_selection_edit_undo_redo_restores_durable_atom(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The public native loop changes one selected atom through Rust history."""
	del qapp
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	changed = tab.change_selected_atom_element("N").observation.snapshot
	undone = tab.undo().observation.snapshot
	redone = tab.redo().observation.snapshot
	selected = tab._controller.projection.selected_durable_targets()
	assert changed.revision < undone.revision < redone.revision and tab.is_dirty
	assert len(selected) == 1 and selected[0].kind == "atom" and selected[0].identifier == "atom-c"
	tab.dispose()


#============================================
def test_native_add_atom_uses_rust_identity_point_history_and_save(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""One bounded insertion stays Rust-owned through selection, history, and reopen."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
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
def test_native_add_double_bond_uses_rust_identity_history_and_save(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Two selected atoms become one durable Rust bond through save and reopen."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
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
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	tab.select_atom("atom-c")
	before = tab.current_snapshot
	with pytest.raises(
		ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTabError,
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
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
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
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
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
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
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
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
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
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
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
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	choice = tab.durable_molecule_choices()[0]
	prior = tab.current_snapshot
	replace = tab._controller.replace
	monkeypatch.setattr(tab._controller, "replace", lambda _observation, _latch: False)
	with pytest.raises(
		ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTabMutationPresentationError,
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
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	replace = tab._controller.replace
	monkeypatch.setattr(tab._controller, "replace", lambda _observation, _latch: False)
	with pytest.raises(
			ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTabMutationPresentationError,
	):
		tab.change_selected_atom_element("N")
	assert tab.current_snapshot is prior and tab.requires_refresh and tab.is_dirty
	with pytest.raises(ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTabError):
		tab.save_atomic(str(tmp_path / "blocked-native-edit.cdml"))
	monkeypatch.setattr(tab._controller, "replace", replace)
	assert tab.refresh_authoritative() and not tab.requires_refresh
	tab.dispose()


#============================================
def test_stale_native_edit_keeps_the_installed_scene_truth(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A stale revision rejection cannot replace the current native projection."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
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
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	session = tab._session
	tab._session = _ObserveFailureSession(session)
	with pytest.raises(
			ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTabMutationPresentationError,
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
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
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
			ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTabMutationPresentationError,
	):
		tab.change_selected_atom_element("N")
	assert tab.current_snapshot is prior and tab.requires_refresh and tab.is_dirty
	monkeypatch.setattr(tab._controller, "replace", replace)
	assert tab.refresh_authoritative() and not tab.requires_refresh
	tab.dispose()

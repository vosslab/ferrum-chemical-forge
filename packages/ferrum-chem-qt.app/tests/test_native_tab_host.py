"""Behavioral coverage for native-page hosting beside legacy document sessions."""

# Standard Library
import dataclasses
import pathlib

# PIP3 modules
import pytest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.legacy.compatibility_main_window
import ferrum_qt.dialogs.bond_dialog
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.window_files


_EDITABLE_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1'>
	<atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  </molecule>
</cdml>"""


_EDITABLE_BOND_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1'>
    <atom id='atom-a' name='C'><point x='10' y='20'/></atom>
    <atom id='atom-b' name='C'><point x='30' y='20'/></atom>
    <bond id='bond-ab' start='atom-a' end='atom-b' type='n2'/>
  </molecule>
</cdml>"""


@dataclasses.dataclass(frozen=True, slots=True)
class _Snapshot:
	"""Small immutable native snapshot fixture."""

	revision: int
	digest: str
	is_dirty: bool


@dataclasses.dataclass(frozen=True, slots=True)
class _DocumentObservation:
	"""Minimal observation envelope for the native-tab fixture."""

	snapshot: _Snapshot


@dataclasses.dataclass(frozen=True, slots=True)
class _RenderObservation:
	"""Fixture render observation with durable snapshot provenance."""

	document: _DocumentObservation


class _Session:
	"""Small owned-value session fixture consumed only by the native tab."""

	#============================================
	def __init__(self, snapshot: _Snapshot) -> None:
		"""Retain one current snapshot for native observation requests."""
		self._snapshot = snapshot

	#============================================
	def snapshot(self) -> _Snapshot:
		"""Return the one current native snapshot."""
		return self._snapshot

	#============================================
	def observe_render(self, revision: int) -> _RenderObservation:
		"""Return the observation only for its exact current revision."""
		if revision != self._snapshot.revision:
			raise ValueError("native fixture observed an unexpected revision")
		return _RenderObservation(_DocumentObservation(self._snapshot))


class _Controller:
	"""Projection owner fixture with explicit terminal disposal evidence."""

	#============================================
	def __init__(self) -> None:
		"""Create one accepting projection generation."""
		self.generation = 0
		self.disposed = False

	#============================================
	def replace(self, observation: _RenderObservation, latch: object) -> bool:
		"""Accept one live observation only when its provenance agrees."""
		return (
			not self.disposed
			and latch.generation == self.generation
			and latch.revision == observation.document.snapshot.revision
			and latch.digest == observation.document.snapshot.digest
		)

	#============================================
	def dispose(self) -> None:
		"""Record native controller retirement."""
		self.disposed = True
		self.generation += 1


#============================================
def _native_tab(dirty: bool) -> tuple[
		ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
		_Controller,
		]:
	"""Build an exact native page through its private fixture constructor."""
	snapshot = _Snapshot(7, "d" * 64, dirty)
	controller = _Controller()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab._from_fixture(
		"Native ethanol", _Session(snapshot), controller,
	)
	return tab, controller


#============================================
def test_native_and_legacy_pages_activate_without_shared_session_aliases(
		main_window: object,
		) -> None:
	"""A native page clears legacy consumers and legacy activation restores them."""
	legacy = main_window._active_session
	tab, _controller = _native_tab(False)
	main_window._register_native_tab(tab)

	assert main_window._tab_widget.currentWidget() is tab
	assert main_window._active_session is None
	assert main_window._document is None and main_window._mode_manager is None
	assert legacy in main_window.sessions and tab not in main_window.sessions
	assert main_window._action_open.isEnabled() and main_window._action_save.isEnabled()

	main_window._tab_widget.setCurrentIndex(main_window._tab_widget.indexOf(legacy.view))
	assert main_window._active_session is legacy
	assert main_window._document is legacy.document
	assert main_window._mode_toolbar.isEnabled()
	assert main_window._action_save.isEnabled() == legacy.can_write_authoritative_snapshot

	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_ordinary_cdml_open_keeps_legacy_file_controller_by_default(
		main_window: object, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The ordinary Open route does not silently turn a CDML path into a Rust tab."""
	paths = []

	def record_legacy_open(
			self: object, file_path: str, replace_current: bool = False,
			) -> bool:
		"""Record the legacy-controller handoff without loading a document."""
		paths.append((self, file_path, replace_current))
		return True

	monkeypatch.setattr(
		ferrum_qt.window_files.WindowFileMixin, "open_file_path", record_legacy_open,
	)
	assert main_window.open_file_path("ordinary.cdml")
	assert paths == [(main_window, "ordinary.cdml", False)]


#============================================
def test_explicit_native_open_registers_a_rust_tab_without_replacing_legacy(
		main_window: object, monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""The explicit action's route owns a separate Rust page beside the legacy tab."""
	source = tmp_path / "native.cdml"
	source.write_text("<svg/>", encoding="utf-8")
	tab, _controller = _native_tab(False)
	monkeypatch.setattr(main_window, "_create_native_tab", lambda _cdml, _title: tab)

	assert main_window.open_native_cdml_path(str(source))
	assert main_window._active_native_tab() is tab
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_legacy_tabs_cannot_invoke_the_explicit_native_element_action(
		main_window: object,
		) -> None:
	"""Element changes remain disabled until one durable atom is selected in a native tab."""
	assert not main_window._action_undo_native.isEnabled()
	assert not main_window._action_redo_native.isEnabled()
	assert not main_window._action_change_element_native.isEnabled()
	assert not main_window._action_atom_properties_native.isEnabled()
	assert not main_window._action_atom_number_native.isEnabled()
	assert not main_window._action_clear_atom_number_native.isEnabled()
	assert not main_window._action_delete_atom_native.isEnabled()
	assert not main_window._action_bond_properties_native.isEnabled()
	assert not main_window._action_delete_bond_native.isEnabled()


#============================================
def test_explicit_native_history_restores_and_reapplies_one_rust_atom_edit(
		main_window: object, tmp_path: pathlib.Path,
		) -> None:
	"""Ferrum history moves a native atom edit between authoritative revisions."""
	source = tmp_path / "history.cdml"
	source.write_text(_EDITABLE_CDML, encoding="utf-8")
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_atom("atom-c")
	tab.change_selected_atom_element("N")
	edited_revision = tab.current_snapshot.revision
	main_window._action_undo_native.trigger()
	undone = tab.selected_atom_projection()
	undone_revision = tab.current_snapshot.revision
	main_window._action_redo_native.trigger()
	redone = tab.selected_atom_projection()

	assert undone.element == "C" and undone_revision > edited_revision
	assert redone.element == "N" and tab.current_snapshot.revision > undone_revision
	main_window._native_tab_close_guard = lambda _operation, _tab: True
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))
	main_window._native_tab_close_guard = None


#============================================
def test_switching_back_to_legacy_restores_legacy_history_action_policy(
		main_window: object, tmp_path: pathlib.Path,
		) -> None:
	"""Leaving an edited Ferrum page returns Undo and Redo to the legacy session."""
	source = tmp_path / "history-owner.cdml"
	source.write_text(_EDITABLE_CDML, encoding="utf-8")
	legacy = main_window._active_session
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_atom("atom-c")
	tab.change_selected_atom_element("N")
	main_window._action_undo_native.trigger()

	main_window._tab_widget.setCurrentIndex(main_window._tab_widget.indexOf(legacy.view))

	assert not main_window._action_undo_native.isEnabled()
	assert not main_window._action_redo_native.isEnabled()
	assert main_window._action_undo.isEnabled() == main_window.can_undo()
	assert main_window._action_redo.isEnabled() == main_window.can_redo()
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_empty_native_history_warns_without_changing_the_rust_tab(
		main_window: object, monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""A fresh native tab exposes Rust's typed empty-history outcome visibly."""
	source = tmp_path / "empty-history.cdml"
	source.write_text(_EDITABLE_CDML, encoding="utf-8")
	warnings = []
	monkeypatch.setattr(
		main_window, "_show_native_file_warning",
		lambda title, message: warnings.append((title, message)),
	)
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	before = tab.current_snapshot
	main_window._action_undo_native.trigger()

	assert tab.current_snapshot == before
	assert warnings and warnings[-1][0] == "Native History Unavailable"
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
#============================================
def test_native_atom_selection_updates_the_explicit_element_action(
		main_window: object, tmp_path: pathlib.Path,
		) -> None:
	"""Selecting then clearing one Rust atom keeps the visible edit action truthful."""
	source = tmp_path / "selectable.cdml"
	source.write_text(_EDITABLE_CDML, encoding="utf-8")
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_atom("atom-c")
	assert (
		main_window._action_change_element_native.isEnabled()
		and main_window._action_atom_properties_native.isEnabled()
		and main_window._action_atom_number_native.isEnabled()
		and main_window._action_delete_atom_native.isEnabled()
	)
	tab.view.scene().clearSelection()
	assert (
		not main_window._action_change_element_native.isEnabled()
		and not main_window._action_atom_properties_native.isEnabled()
		and not main_window._action_atom_number_native.isEnabled()
		and not main_window._action_delete_atom_native.isEnabled()
	)
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_native_delete_atom_action_removes_incident_bond_and_undo_restores_it(
		main_window: object, tmp_path: pathlib.Path,
		) -> None:
	"""The ordinary host deletes one selected Rust atom as an undoable topology edit."""
	source = tmp_path / "delete-native-atom.cdml"
	source.write_text(_EDITABLE_BOND_CDML, encoding="utf-8")
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_atom("atom-a")
	main_window._action_delete_atom_native.trigger()
	deleted_cdml = tab.current_snapshot.cdml

	assert (
		'id="atom-a"' not in deleted_cdml and 'id="bond-ab"' not in deleted_cdml
		and not main_window._action_delete_atom_native.isEnabled()
	)
	main_window._action_undo_native.trigger()
	restored_cdml = tab.current_snapshot.cdml
	assert 'id="atom-a"' in restored_cdml and 'id="bond-ab"' in restored_cdml
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_native_bond_selection_owns_the_explicit_bond_properties_action(
		main_window: object, tmp_path: pathlib.Path,
		) -> None:
	"""Only one selected native bond enables the ordinary-window bond action."""
	source = tmp_path / "selectable-bond.cdml"
	source.write_text(_EDITABLE_BOND_CDML, encoding="utf-8")
	legacy = main_window._active_session
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_bond("bond-ab")
	assert (
		main_window._action_bond_properties_native.isEnabled()
		and main_window._action_delete_bond_native.isEnabled()
	)
	tab.select_atom("atom-a")
	assert (
		not main_window._action_bond_properties_native.isEnabled()
		and not main_window._action_delete_bond_native.isEnabled()
	)
	tab.view.scene().clearSelection()
	assert (
		not main_window._action_bond_properties_native.isEnabled()
		and not main_window._action_delete_bond_native.isEnabled()
	)
	main_window._tab_widget.setCurrentIndex(main_window._tab_widget.indexOf(legacy.view))
	assert (
		not main_window._action_bond_properties_native.isEnabled()
		and not main_window._action_delete_bond_native.isEnabled()
	)
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_native_delete_bond_action_preserves_atoms_and_undo_restores_the_bond(
		main_window: object, tmp_path: pathlib.Path,
		) -> None:
	"""The ordinary host deletes one Rust bond and native Undo restores it."""
	source = tmp_path / "delete-native-bond.cdml"
	source.write_text(_EDITABLE_BOND_CDML, encoding="utf-8")
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_bond("bond-ab")
	before = tab.current_snapshot
	main_window._action_delete_bond_native.trigger()
	deleted_cdml = tab.current_snapshot.cdml

	assert tab.current_snapshot.revision == before.revision + 1
	assert (
		'id="atom-a"' in deleted_cdml and 'id="atom-b"' in deleted_cdml
		and 'id="bond-ab"' not in deleted_cdml
		and not main_window._action_delete_bond_native.isEnabled()
	)
	main_window._action_undo_native.trigger()
	assert 'id="bond-ab"' in tab.current_snapshot.cdml
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_cancelled_native_bond_properties_leaves_the_rust_tab_unchanged(
		main_window: object, monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""Declining the native bond form submits no document operation."""
	source = tmp_path / "bond-properties-cancel.cdml"
	source.write_text(_EDITABLE_BOND_CDML, encoding="utf-8")
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_bond("bond-ab")
	before = tab.current_snapshot
	monkeypatch.setattr(
		ferrum_qt.dialogs.bond_dialog.BondDialog, "exec",
		lambda _dialog: PySide6.QtWidgets.QDialog.DialogCode.Rejected,
	)
	main_window._action_bond_properties_native.trigger()
	assert tab.current_snapshot == before and tab.has_one_selected_bond()
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_native_bond_properties_refuses_an_unrepresentable_rust_fact(
		main_window: object, monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""A form that cannot display a source width leaves the native tab unchanged."""
	source = tmp_path / "bond-properties-unrepresentable.cdml"
	source.write_text(
		_EDITABLE_BOND_CDML.replace("type='n2'/>", "type='n2' line_width='1.05'/>"),
		encoding="utf-8",
	)
	warnings = []
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_bond("bond-ab")
	before = tab.current_snapshot
	monkeypatch.setattr(
		main_window, "_show_native_file_warning",
		lambda title, message: warnings.append((title, message)),
	)

	main_window._action_bond_properties_native.trigger()

	assert (
		tab.current_snapshot == before and tab.has_one_selected_bond()
		and warnings[-1] == (
			"Native Bond Properties Unavailable",
			"selected Rust bond line width is not representable by BondDialog",
		)
	)
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_explicit_native_atom_number_commits_one_selected_rust_atom(
		main_window: object, monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""The accepted number form preserves selection through one Rust operation."""
	source = tmp_path / "atom-number.cdml"
	source.write_text(_EDITABLE_CDML, encoding="utf-8")
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_atom("atom-c")
	before = tab.current_snapshot

	def accept_visible_number(dialog: object) -> int:
		"""Enter one visible number through the public dialog controls."""
		dialog.number_edit.setText("12")
		dialog.show_number.setChecked(True)
		return PySide6.QtWidgets.QDialog.DialogCode.Accepted

	monkeypatch.setattr(
		ferrum_qt.native.ferrum_native_atom_number.FerrumNativeAtomNumberDialog,
		"exec", accept_visible_number,
	)
	main_window._action_atom_number_native.trigger()
	atom = tab.selected_atom_projection()
	assert tab.current_snapshot.revision == before.revision + 1
	assert atom.number == 12 and atom.show_number and tab.has_one_selected_atom()
	assert main_window._action_clear_atom_number_native.isEnabled()
	main_window._action_clear_atom_number_native.trigger()
	atom = tab.selected_atom_projection()
	assert tab.current_snapshot.revision == before.revision + 2
	assert atom.number is None and atom.show_number is None and tab.has_one_selected_atom()
	assert not main_window._action_clear_atom_number_native.isEnabled()
	main_window._native_tab_close_guard = lambda _operation, _tab: True
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))
	main_window._native_tab_close_guard = None


#============================================
def test_cancelled_native_atom_number_leaves_the_rust_tab_unchanged(
		main_window: object, monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""Declining the native number form submits no document operation."""
	source = tmp_path / "atom-number-cancel.cdml"
	source.write_text(_EDITABLE_CDML, encoding="utf-8")
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_atom("atom-c")
	before = tab.current_snapshot
	monkeypatch.setattr(
		ferrum_qt.native.ferrum_native_atom_number.FerrumNativeAtomNumberDialog,
		"exec", lambda _dialog: PySide6.QtWidgets.QDialog.DialogCode.Rejected,
	)
	main_window._action_atom_number_native.trigger()
	assert tab.current_snapshot == before and tab.has_one_selected_atom()
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


def test_explicit_native_atom_properties_commit_one_selected_rust_atom(
		main_window: object, monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""The accepted visible action preserves selection through one Rust patch."""
	source = tmp_path / "properties.cdml"
	source.write_text(_EDITABLE_CDML, encoding="utf-8")
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_atom("atom-c")
	before = tab.current_snapshot

	def accept_with_charge(dialog: object) -> int:
		"""Change one supported form field before accepting its native patch."""
		dialog._charge_spin.setValue(1)
		return PySide6.QtWidgets.QDialog.DialogCode.Accepted

	monkeypatch.setattr(
		ferrum_qt.dialogs.atom_dialog.AtomDialog, "exec", accept_with_charge,
	)
	main_window._action_atom_properties_native.trigger()
	atom = tab.selected_atom_projection()
	assert (
		tab.current_snapshot.revision == before.revision + 1
		and atom.formal_charge == 1 and tab.has_one_selected_atom()
	)
	main_window._native_tab_close_guard = lambda _operation, _tab: True
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))
	main_window._native_tab_close_guard = None


#============================================
def test_cancelled_native_atom_properties_leaves_the_rust_tab_unchanged(
		main_window: object, monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""Declining the explicit form does not submit a document operation."""
	source = tmp_path / "properties-cancel.cdml"
	source.write_text(_EDITABLE_CDML, encoding="utf-8")
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_atom("atom-c")
	before = tab.current_snapshot
	monkeypatch.setattr(
		ferrum_qt.dialogs.atom_dialog.AtomDialog, "exec",
		lambda _dialog: PySide6.QtWidgets.QDialog.DialogCode.Rejected,
	)
	main_window._action_atom_properties_native.trigger()
	assert tab.current_snapshot == before and tab.has_one_selected_atom()
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_native_atom_properties_reports_unrepresentable_rust_facts_without_mutation(
		main_window: object, monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""The public Rust action leaves an exact tab intact when its form cannot show a fact."""
	source = tmp_path / "fractional-font.cdml"
	source.write_text(
		'<cdml><molecule id="m"><atom id="a" name="C"><point x="10" y="20"/>'
		'<font size="14.5"/></atom></molecule></cdml>', encoding="utf-8",
	)
	warnings = []
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_atom("a")
	before = tab.current_snapshot
	monkeypatch.setattr(
		main_window, "_show_native_file_warning",
		lambda title, message: warnings.append((title, message)),
	)

	main_window._action_atom_properties_native.trigger()

	assert (
		tab.current_snapshot == before and tab.has_one_selected_atom()
		and warnings[-1] == (
			"Native Atom Properties Unavailable",
			"selected Rust atom font size is not representable by AtomDialog",
		)
	)
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_explicit_native_open_cancel_keeps_the_ordinary_session_active(
		main_window: object, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Cancelling the opt-in chooser leaves the visible ordinary document untouched."""
	legacy = main_window._active_session
	monkeypatch.setattr(
		ferrum_qt.main_window.PySide6.QtWidgets.QFileDialog,
		"getOpenFileName", lambda *_args: ("", ""),
	)

	assert not main_window._on_open_native_cdml()
	assert main_window._active_session is legacy and not main_window._native_tabs_by_page


#============================================
def test_explicit_native_open_rejects_non_cdml_without_changing_the_session(
		main_window: object, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The explicit route rejects a foreign extension before it can create a tab."""
	legacy = main_window._active_session
	warnings = []
	monkeypatch.setattr(
		ferrum_qt.main_window.PySide6.QtWidgets.QFileDialog,
		"getOpenFileName", lambda *_args: ("not-a-ferrum-document.sdf", ""),
	)
	monkeypatch.setattr(
		main_window, "_show_native_file_warning",
		lambda title, message: warnings.append((title, message)),
	)

	assert not main_window._on_open_native_cdml()
	assert main_window._active_session is legacy and not main_window._native_tabs_by_page
	assert warnings[-1][0] == "Unsupported File Format"


#============================================
def test_mainwindow_save_publishes_the_active_rust_tab(
		main_window: object, tmp_path: pathlib.Path,
		) -> None:
	"""MainWindow Save publishes an edited native tab through its Rust owner."""
	source = tmp_path / "save-native.cdml"
	source.write_text(_EDITABLE_CDML, encoding="utf-8")
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_atom("atom-c")
	tab.change_selected_atom_element("N")

	assert main_window._on_save() and not tab.is_dirty
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_mainwindow_save_as_publishes_the_active_rust_tab_to_the_chosen_path(
		main_window: object, monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""MainWindow Save As keeps a native tab on its Rust publication route."""
	source = tmp_path / "save-as-native.cdml"
	destination = tmp_path / "published-native.cdml"
	source.write_text(_EDITABLE_CDML, encoding="utf-8")
	assert main_window.open_native_cdml_path(str(source))
	tab = main_window._active_native_tab()
	assert tab is not None
	tab.select_atom("atom-c")
	tab.change_selected_atom_element("N")
	monkeypatch.setattr(
		ferrum_qt.window_native_files.PySide6.QtWidgets.QFileDialog,
		"getSaveFileName", lambda *_args: (str(destination), ""),
	)

	assert main_window._on_save_as() and tab.file_path == destination and not tab.is_dirty
	assert main_window.close_session_at(main_window._tab_widget.indexOf(tab))


#============================================
def test_native_registration_failure_restores_the_prior_legacy_activation(
		main_window: object, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A failed native activation returns every legacy consumer to its prior owner."""
	legacy = main_window._active_session
	mode = legacy.mode_manager.current_mode
	tab, controller = _native_tab(False)
	original_activate = main_window._activate_native_tab
	selection_refreshes = []
	original_selection_change = main_window._on_native_tab_selection_changed

	def record_selection_refresh() -> None:
		"""Record a stale signal delivery while retaining the normal slot behavior."""
		selection_refreshes.append(True)
		original_selection_change()

	def fail_after_native_activation(
			target: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Inject a failure after native activation changed every legacy alias."""
		original_activate(target)
		raise RuntimeError("injected native activation failure")

	monkeypatch.setattr(
		main_window, "_on_native_tab_selection_changed", record_selection_refresh,
	)
	monkeypatch.setattr(main_window, "_activate_native_tab", fail_after_native_activation)
	with pytest.raises(RuntimeError, match="injected native activation failure"):
		main_window._register_native_tab(tab)
	selection_refreshes.clear()
	tab.selection_changed.emit()

	assert main_window._tab_widget.currentWidget() is legacy.view
	assert main_window._active_session is legacy
	assert main_window._document is legacy.document
	assert main_window._document_signal_source is legacy.document
	assert legacy.mode_manager.current_mode is mode and main_window._mode_toolbar.isEnabled()
	assert tab not in main_window._native_tabs_by_page
	assert main_window._tab_widget.indexOf(tab) < 0
	assert not selection_refreshes
	tab.dispose()


#============================================
def test_native_close_guard_preserves_legacy_page_and_disposes_once(
		main_window: object,
		) -> None:
	"""A dirty native close respects its guard and never retires the legacy tab."""
	legacy = main_window._active_session
	tab, controller = _native_tab(True)
	main_window._register_native_tab(tab)
	main_window._native_tab_close_guard = lambda _operation, _tab: False

	assert not main_window.close_session_at(main_window._tab_widget.currentIndex())
	assert not controller.disposed

	main_window._native_tab_close_guard = lambda _operation, _tab: True
	assert main_window.close_session_at(main_window._tab_widget.currentIndex())
	assert controller.disposed
	assert main_window._tab_widget.currentWidget() is legacy.view
	assert main_window._active_session is legacy
	main_window._native_tab_close_guard = None


#============================================
def test_shutdown_guards_then_disposes_native_pages_without_legacy_cross_disposal(
		qapp: object, theme_manager: object,
		) -> None:
	"""Terminal shutdown retires native pages only after every save guard approves."""
	window = ferrum_qt.legacy.compatibility_main_window.LegacyCompatibilityMainWindow(theme_manager)
	legacy = window._active_session
	tab, controller = _native_tab(False)
	window._register_native_tab(tab)

	assert window.prepare_application_shutdown()
	assert controller.disposed
	assert legacy.is_disposed
	window.deleteLater()
	qapp.processEvents()

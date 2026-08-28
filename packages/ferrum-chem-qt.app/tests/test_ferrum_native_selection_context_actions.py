"""Durable selection-context action contracts for the native Ferrum canvas."""

# Standard Library
import json
import types

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.themes.theme_loader
import ferrum_qt.actions.context_menu
import ferrum_qt.declarative_resources
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.document_tab_errors
import ferrum_qt.ferrum.keyboard_canvas


#============================================
#============================================
def _selected_atom_tab(
		qapp: PySide6.QtWidgets.QApplication, window: object, name: str,
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Return one active tab with its visible atom selected through canvas input."""
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='mol-1'><atom id='a1' "
		"name='C'><point x='10' y='10'/></atom></molecule></cdml>", name,
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	assert window._window_mode_sync.select_action(window._select_structure_action)
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 10.0)),
	)
	qapp.processEvents()
	return tab


#============================================
def _selected_bond_tab(
		qapp: PySide6.QtWidgets.QApplication, window: object, name: str,
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Return one active tab with its bond selected through the Rust canvas route."""
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='mol-1'><atom id='a1' "
		"name='C'><point x='10' y='10'/></atom><atom id='a2' name='O'>"
		"<point x='50' y='10'/></atom><bond id='b1' start='a1' end='a2' "
		"type='n1'/></molecule></cdml>", name,
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	assert window._window_mode_sync.select_action(window._select_structure_action)
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		tab.view.mapFromScene(PySide6.QtCore.QPointF(30.0, 10.0)),
	)
	qapp.processEvents()
	return tab


#============================================
def _atoms_remain(tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab) -> bool:
	"""Return whether the test document still has a Rust-projected atom."""
	projection = tab.current_document_observation().projection
	return any(molecule.atoms for molecule in projection.molecules)


#============================================
def _selection_object_ids(window: object) -> tuple[str, ...]:
	"""Return the opaque Rust selection's durable object identities."""
	selection = window._structure_selection
	assert selection is not None
	return tuple(target.object_id for target in selection.targets)


#============================================
def _select_at_keyboard_cursor(
		qapp: PySide6.QtWidgets.QApplication, tab: object,
		x: float, y: float,
		modifier: PySide6.QtCore.Qt.KeyboardModifiers = (
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier),
		) -> None:
	"""Position the view cursor, then use the public Enter selection route."""
	tab.view.set_hex_grid_snap_enabled(False)
	tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(x, y))
	PySide6.QtTest.QTest.keyClick(
		tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return, modifier,
	)
	qapp.processEvents()


#============================================
def _open_context_menu(
		qapp: PySide6.QtWidgets.QApplication,
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		invocation: str,
		) -> PySide6.QtWidgets.QMenu:
	"""Open the selected canvas context client through one public input route."""
	viewport = tab.view.viewport()
	if invocation == "Menu":
		PySide6.QtTest.QTest.keyClick(viewport, PySide6.QtCore.Qt.Key.Key_Menu)
	elif invocation == "Shift+F10":
		PySide6.QtTest.QTest.keyClick(
			viewport,
			PySide6.QtCore.Qt.Key.Key_F10,
			PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier,
		)
	else:
		PySide6.QtTest.QTest.mouseClick(
			viewport,
			PySide6.QtCore.Qt.MouseButton.RightButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 10.0)),
		)
	qapp.processEvents()
	menu = PySide6.QtWidgets.QApplication.activePopupWidget()
	assert isinstance(menu, PySide6.QtWidgets.QMenu)
	return menu


#============================================
def test_context_menu_reuses_enabled_registry_actions_in_yaml_group_order(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Selected-structure clients retain action identity and YAML ordering."""
	tab = _selected_atom_tab(qapp, main_window, "context-order.cdml")
	registry = main_window._action_registry
	accessible_name, action_groups = ferrum_qt.declarative_resources.load_context_menu_placement(
		registry,
	)
	filtered = tuple(
		registry.get_qt_action(action_id)
		for group in action_groups
		for action_id in group
		if registry.get_qt_action(action_id).isEnabled()
	)
	menu = ferrum_qt.actions.context_menu.build_context_menu(
		tab.view.viewport(), registry, action_groups, accessible_name,
	)
	assert menu is not None
	menu_actions = tuple(action for action in menu.actions() if not action.isSeparator())
	assert tuple(id(action) for action in menu_actions) == tuple(id(action) for action in filtered)
	menu.deleteLater()


#============================================
@pytest.mark.parametrize("invocation", ("Menu", "Shift+F10", "right button", "Delete", "Backspace"))
def test_context_action_and_normalized_delete_keys_remove_the_same_selection(
		qapp: PySide6.QtWidgets.QApplication, main_window: object, invocation: str,
		) -> None:
	"""Context activation and both delete keys converge on Rust selection deletion."""
	tab = _selected_atom_tab(qapp, main_window, f"selection-{invocation}.cdml")
	if invocation in {"Menu", "Shift+F10", "right button"}:
		registry = main_window._action_registry
		menu = _open_context_menu(qapp, tab, invocation)
		delete_action = registry.get_qt_action("edit.delete_selection")
		assert delete_action in menu.actions()
		delete_action.trigger()
	else:
		key = getattr(PySide6.QtCore.Qt.Key, f"Key_{invocation}")
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), key)
	qapp.processEvents()
	assert not _atoms_remain(tab)


#============================================
@pytest.mark.parametrize("invocation", ("Menu", "Shift+F10"))
def test_keyboard_context_menu_close_restores_native_viewport_focus(
		qapp: PySide6.QtWidgets.QApplication, main_window: object, invocation: str,
		) -> None:
	"""Closing each keyboard context route deterministically returns focus to the canvas."""
	tab = _selected_atom_tab(qapp, main_window, f"context-focus-{invocation}.cdml")
	viewport = tab.view.viewport()
	menu = _open_context_menu(qapp, tab, invocation)
	PySide6.QtTest.QTest.keyClick(menu, PySide6.QtCore.Qt.Key.Key_Escape)
	qapp.processEvents()
	assert viewport.hasFocus()


#============================================
def test_rust_atom_selection_bridge_enables_context_properties_without_qt_atom_identity(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Rust selection drives atom actions even though molecule roots own Qt items."""
	tab = _selected_atom_tab(qapp, main_window, "selection-action-bridge.cdml")
	bridge = tab._structure_action_selection_v1
	assert bridge is not None and len(bridge.targets) == 1
	assert main_window._action_registry.get_qt_action("edit.atom.properties").isEnabled()
	menu = _open_context_menu(qapp, tab, "Menu")
	assert main_window._action_registry.get_qt_action("edit.atom.properties") in menu.actions()
	menu.close()


#============================================
def test_rust_bond_selection_bridge_enables_bond_properties(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""One Rust-selected bond enables only its registered bond property action."""
	tab = _selected_bond_tab(qapp, main_window, "selection-bond-bridge.cdml")
	assert tab.has_one_selected_bond()
	assert main_window._action_registry.get_qt_action("edit.bond.properties").isEnabled()


#============================================
def test_multi_target_selection_disables_single_target_properties(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Two Rust-selected targets cannot enable one-target atom or bond actions."""
	tab = _selected_bond_tab(qapp, main_window, "selection-multi-target.cdml")
	for point, modifier in (
			(PySide6.QtCore.QPointF(10.0, 10.0), PySide6.QtCore.Qt.KeyboardModifier.NoModifier),
			(PySide6.QtCore.QPointF(50.0, 10.0), PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier),
		):
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, modifier,
			tab.view.mapFromScene(point),
		)
	qapp.processEvents()
	assert len(tab.selected_structure_targets()) == 2
	assert not main_window._action_registry.get_qt_action("edit.atom.properties").isEnabled()
	assert not main_window._action_registry.get_qt_action("edit.bond.properties").isEnabled()


#============================================
def test_tab_switch_clears_the_outgoing_structural_action_bridge(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Changing native tabs cannot leave the old Rust selection actionable."""
	first = _selected_atom_tab(qapp, main_window, "selection-tab-switch-first.cdml")
	second = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "selection-tab-switch-second.cdml",
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	main_window._register_native_tab(second, activate=False)
	main_window._tab_widget.setCurrentWidget(second)
	qapp.processEvents()
	assert first._structure_action_selection_v1 is None
	assert not main_window._action_registry.get_qt_action("edit.atom.properties").isEnabled()


#============================================
def test_mode_switch_and_tab_close_clear_the_structural_action_bridge(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Another authoring mode and normal close both retire bridge-backed actions."""
	tab = _selected_atom_tab(qapp, main_window, "selection-mode-and-close.cdml")
	main_window._draw_bond_action.trigger()
	qapp.processEvents()
	assert tab._structure_action_selection_v1 is None
	assert not main_window._action_registry.get_qt_action("edit.atom.properties").isEnabled()
	main_window._close_native_tab_at(
		main_window._tab_widget.indexOf(tab),
		ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
	)
	assert tab.is_disposed


#============================================
def test_successful_atom_mutation_clears_the_structural_action_bridge(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""A committed Rust mutation cannot retain a stale opaque selection fence."""
	tab = _selected_atom_tab(qapp, main_window, "selection-successful-mutation.cdml")
	tab.set_selected_atom_number(1, True)
	qapp.processEvents()
	assert tab._structure_action_selection_v1 is None


#============================================
def test_failed_projection_install_clears_bridge_before_refresh_required(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A pending replacement cannot retain or rebase stale Rust selection targets."""
	tab = _selected_atom_tab(qapp, main_window, "selection-refresh-required.cdml")
	monkeypatch.setattr(tab._controller, "replace", lambda *_args: False)
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError):
		tab.set_selected_atom_number(1, True)
	qapp.processEvents()
	assert tab.requires_refresh and tab._structure_action_selection_v1 is None
	assert tab._structure_action_targets_v1 == () and tab.selected_structure_targets() == ()
	assert not main_window._action_registry.get_qt_action("edit.atom.properties").isEnabled()
	monkeypatch.undo()
	assert tab.refresh_authoritative()


#============================================
def test_refused_atom_mutation_retains_the_structural_action_bridge(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""A rejected atom update leaves the current Rust-issued action target live."""
	tab = _selected_atom_tab(qapp, main_window, "selection-refused-mutation.cdml")
	selection = tab._structure_action_selection_v1
	with pytest.raises(TypeError):
		tab.set_selected_atom_number(1, 1)
	qapp.processEvents()
	assert tab._structure_action_selection_v1 is selection
	assert main_window._action_registry.get_qt_action("edit.atom.properties").isEnabled()


#============================================
def test_structural_bridge_excludes_generic_canvas_selection_until_cleared(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Rust bridge targets win over generic selection rather than merging sources."""
	tab = _selected_bond_tab(qapp, main_window, "selection-source-exclusivity.cdml")
	projection = tab.current_document_observation().projection.molecules[0]
	first_atom_id, second_atom_id = (
		projection.atoms[0].document_object_id,
		projection.atoms[1].document_object_id,
	)
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 10.0)),
	)
	tab._require_projection().select_durable((("document_object", second_atom_id),))
	qapp.processEvents()
	assert tuple(target.object_id for target in tab.selected_structure_targets()) == (first_atom_id,)
	tab.clear_structure_action_selection_v1()
	assert tuple(target.object_id for target in tab.selected_structure_targets()) == (second_atom_id,)


#============================================
def test_keyboard_selection_matches_pointer_identity_and_shift_enter_toggles(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Cursor Enter uses Rust's pointer-equivalent identity and toggle algebra."""
	tab = _selected_atom_tab(qapp, main_window, "keyboard-identity.cdml")
	pointer_ids = _selection_object_ids(main_window)
	main_window._window_mode_sync.cancel()
	assert main_window._window_mode_sync.select_action(main_window._select_structure_action)
	_select_at_keyboard_cursor(qapp, tab, 10.0, 10.0)
	assert _selection_object_ids(main_window) == pointer_ids
	_select_at_keyboard_cursor(
		qapp, tab, 10.0, 10.0,
		PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier,
	)
	assert main_window._structure_selection is not None
	assert not main_window._structure_selection.targets


#============================================
def test_keyboard_cursor_move_preserves_document_and_selection(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Normal and fine cursor movement are view-only while selection remains active."""
	tab = _selected_atom_tab(qapp, main_window, "keyboard-move.cdml")
	selected_ids = _selection_object_ids(main_window)
	revision = tab.current_snapshot.revision
	tab.view.set_hex_grid_snap_enabled(False)
	tab.view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(10.0, 10.0))
	viewport = tab.view.viewport()
	PySide6.QtTest.QTest.keyClick(viewport, PySide6.QtCore.Qt.Key.Key_Right)
	PySide6.QtTest.QTest.keyClick(
		viewport, PySide6.QtCore.Qt.Key.Key_Right,
		PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier,
	)
	qapp.processEvents()
	cursor = tab.view.keyboard_cursor_scene()
	assert cursor is not None
	assert cursor.x() == 10.0 + ferrum_qt.ferrum.keyboard_canvas.KEYBOARD_CURSOR_GRID_INCREMENT_PT + ferrum_qt.ferrum.keyboard_canvas.KEYBOARD_CURSOR_FINE_INCREMENT_PT
	assert (tab.current_snapshot.revision, _selection_object_ids(main_window)) == (revision, selected_ids)


#============================================
def test_keyboard_empty_hit_preserves_selection_without_marquee(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""An empty cursor Enter retains the current Rust selection and action state."""
	tab = _selected_atom_tab(qapp, main_window, "keyboard-empty.cdml")
	selected_ids = _selection_object_ids(main_window)
	selection_item = main_window._structure_selection_item
	revision = tab.current_snapshot.revision
	delete_enabled = main_window._delete_structure_selection_action.isEnabled()
	_select_at_keyboard_cursor(qapp, tab, 300.0, 300.0)
	assert (tab.current_snapshot.revision, _selection_object_ids(main_window)) == (revision, selected_ids)
	assert (main_window._structure_selection_item, main_window._structure_marquee) == (selection_item, None)
	assert main_window._delete_structure_selection_action.isEnabled() is delete_enabled
	assert main_window.statusBar().currentMessage() == "No selectable structure at document cursor."


#============================================
def test_select_structure_accessibility_and_escape_are_transient_only(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Mode entry describes keyboard operation and Escape restores inactive canvas text."""
	tab = _selected_atom_tab(qapp, main_window, "keyboard-accessibility.cdml")
	revision = tab.current_snapshot.revision
	accessible = tab.view.accessibleDescription()
	assert tab.view._keyboard_cursor_item is not None and tab.view._keyboard_cursor_item.isVisible()
	assert "Select Structure mode." in accessible and "Shift+Enter toggles" in accessible
	PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
	qapp.processEvents()
	assert tab.current_snapshot.revision == revision
	assert tab._structure_action_selection_v1 is None
	assert not main_window._action_registry.get_qt_action("edit.atom.properties").isEnabled()
	assert tab.view.accessibleDescription() == (
		"Document-space cursor. Arrow keys move by one grid increment; "
		"Shift+Arrow moves by a fine increment."
	)
	assert not tab.view._keyboard_cursor_item.isVisible() and tab.view.viewport().hasFocus()


#============================================
def test_hydrogen_action_captures_fenced_target_before_selection_cleanup(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Transient cleanup may retire selection after one valid hydrogen target is captured."""
	tab = _selected_atom_tab(qapp, main_window, "hydrogen-captured-target.cdml")
	address = tab.selected_molecule_atom_address()
	refusals: list[str] = []
	calls: list[tuple[object, ...]] = []
	monkeypatch.setattr(main_window, "_show_edit_refusal", refusals.append)
	monkeypatch.setattr(
		tab._session, "materialize_live_molecule_hydrogens_v1",
		lambda *args: calls.append(args) or types.SimpleNamespace(
			outcome=types.SimpleNamespace(
				molecule_hydrogens_materialized=types.SimpleNamespace(changed=False),
			),
		),
	)
	assert main_window._make_hydrogens_explicit()
	assert tab._structure_action_selection_v1 is None
	assert calls == [(address.revision, address.digest, address.molecule_id, address.atom_id)]
	assert refusals == []


#============================================
def test_oxidation_action_captures_fenced_target_before_selection_cleanup(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An oxidation observation runs from captured Rust facts after transient cleanup."""
	tab = _selected_atom_tab(qapp, main_window, "oxidation-captured-target.cdml")
	address = tab.selected_molecule_atom_address()
	refusals: list[str] = []
	calls: list[tuple[object, ...]] = []
	monkeypatch.setattr(main_window, "_show_edit_refusal", refusals.append)
	monkeypatch.setattr(main_window, "_show_atom_oxidation_dialog", lambda *_args: None)
	monkeypatch.setattr(main_window, "_queue_operation_presentation_v1", lambda *_args: None)
	monkeypatch.setattr(
		tab._session, "observe_live_atom_oxidation_v1",
		lambda *args: calls.append(args) or types.SimpleNamespace(
			source_revision=address.revision,
			source_digest_hex=address.digest,
			molecule_object_id=address.molecule_id,
			atom_object_id=address.atom_id,
			status="accepted",
			oxidation_number=0,
		),
	)
	assert main_window._start_atom_oxidation()
	assert tab._structure_action_selection_v1 is None
	assert calls == [(address.revision, address.digest, address.molecule_id, address.atom_id)]
	assert refusals == []


#============================================
def test_compact_group_action_captures_fenced_target_before_selection_cleanup(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A Rust-eligible compact target survives only as the captured fenced intent."""
	tab = _selected_atom_tab(qapp, main_window, "compact-captured-target.cdml")
	snapshot = tab.current_snapshot
	address = types.SimpleNamespace(
		revision=snapshot.revision,
		digest=snapshot.digest,
		molecule_id="mol-1",
		compact_group_id="group-1",
	)
	refusals: list[str] = []
	captured_intents: list[object] = []
	monkeypatch.setattr(main_window, "_show_edit_refusal", refusals.append)
	def selected_compact_group_address() -> object:
		if tab._structure_action_selection_v1 is None:
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"select one current compact group first",
			)
		return address
	monkeypatch.setattr(
		tab, "selected_molecule_compact_group_address", selected_compact_group_address,
	)
	monkeypatch.setattr(
		tab._session, "compact_group_materialization_availability_v1",
		lambda request_json: types.SimpleNamespace(response_json=json.dumps({
			"schema": "ferrum-live-document-compact-group-materialization-availability-v1",
			"document_fence": {
				"expected_revision": address.revision,
				"expected_digest_hex": address.digest,
			},
			"molecule_object_id": address.molecule_id,
			"compact_group_object_id": address.compact_group_id,
			"availability": "eligible",
		})),
	)
	monkeypatch.setattr(
		main_window, "_run_compact_group_materialization",
		lambda: captured_intents.append(main_window._compact_group_materialization_intent),
	)
	assert main_window._materialize_selected_compact_group()
	assert tab._structure_action_selection_v1 is None
	assert len(captured_intents) == 1
	assert captured_intents[0].address is address
	assert refusals == []
	main_window._compact_group_materialization_intent = None

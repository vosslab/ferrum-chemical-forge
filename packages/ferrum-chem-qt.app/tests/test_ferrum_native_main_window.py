"""Behavior coverage for the public Rust-native Ferrum window seam."""

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
import ferrum_qt.main_window
import ferrum_qt.native.ferrum_native_drawing_parameters
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

_MIXED_ROOT_CDML = """<cdml version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='13' y='17'/></atom>
</molecule><plus id='plus-1'><point x='73' y='51'/></plus></cdml>"""

_DUPLICATE_MARK_CDML = """<cdml version='26.07'><molecule id='mol-1'>
  <atom id='atom-c' name='C' charge='2'><point x='10' y='20'/>
    <mark type='plus' x='18' y='28' size='10' data-origin='first'/>
    <mark type='plus' x='20' y='30' size='10' data-origin='second'/>
  </atom>
</molecule></cdml>"""

_AUTHORED_COORDINATE_TOLERANCE = 0.001 * 72.0 / 2.54


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
def _select_mixed_complete_roots(
		tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
		) -> tuple[tuple[str, str], ...]:
	"""Select one complete molecule and one independent presentation root."""
	plus_key = next(
		key for key in tab._controller.projection.durable_items if key[0] == "plus"
	)
	tab._controller.projection.select_durable((
		("atom", "atom-c"), plus_key,
	))
	return tab.selected_top_level_transform_targets()[1]


#============================================
def _mixed_root_positions(
		tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
		) -> tuple[tuple[float, float], tuple[float, float]]:
	"""Return authoritative molecule and presentation coordinates after a move."""
	projection = tab.current_document_observation().projection
	atom = projection.molecules[0].atoms[0].position
	plus = projection.presentation_stack.roots[0].plus.anchor
	return (atom.x, atom.y), (plus.x, plus.y)


#============================================
def _click_visible_menu_action(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Activate one labelled command through its visible top-level menu item."""
	menu_bar = window.menuBar()
	for menu_action in menu_bar.actions():
		menu = menu_action.menu()
		if menu is None:
			continue
		for candidate in menu.actions():
			if candidate.text().replace("&", "") != label:
				continue
			PySide6.QtTest.QTest.mouseClick(
				menu_bar, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu_bar.actionGeometry(menu_action).center(),
			)
			qapp.processEvents()
			if not menu.isVisible():
				raise AssertionError(f"Visible menu did not open for {label!r}")
			PySide6.QtTest.QTest.mouseClick(
				menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu.actionGeometry(candidate).center(),
			)
			qapp.processEvents()
			return
	raise AssertionError(f"No visible menu action is labelled {label!r}")


#============================================
def _editing_tool_button(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		) -> PySide6.QtWidgets.QToolButton:
	"""Find one visible Editing Tools command by its accessible user-facing name."""
	for toolbar in window.findChildren(PySide6.QtWidgets.QToolBar):
		if toolbar.accessibleName() != "Editing tools toolbar":
			continue
		for button in toolbar.findChildren(PySide6.QtWidgets.QToolButton):
			if button.accessibleName() == label and button.isVisible():
				return button
	raise AssertionError(f"Editing Tools has no visible {label!r} command.")


#============================================
def _drawing_parameter_combo(
		window: PySide6.QtWidgets.QMainWindow, accessible_name: str,
		) -> PySide6.QtWidgets.QComboBox:
	"""Find one visible Next Drawing control through its accessible name."""
	for combo in window.findChildren(PySide6.QtWidgets.QComboBox):
		if combo.accessibleName() == accessible_name and combo.isVisible():
			return combo
	raise AssertionError(f"Editing Tools has no visible {accessible_name!r} control.")


#============================================
def _choose_next_element(
		window: PySide6.QtWidgets.QMainWindow, element: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Complete one editable visible Next atom choice through its Qt client."""
	combo = _drawing_parameter_combo(window, "Next atom")
	line_edit = combo.lineEdit()
	PySide6.QtTest.QTest.mouseClick(
		line_edit, PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
	)
	PySide6.QtTest.QTest.keyClick(
		line_edit, PySide6.QtCore.Qt.Key.Key_A,
		PySide6.QtCore.Qt.KeyboardModifier.ControlModifier,
	)
	PySide6.QtTest.QTest.keyClicks(line_edit, element)
	PySide6.QtTest.QTest.keyClick(line_edit, PySide6.QtCore.Qt.Key.Key_Return)
	qapp.processEvents()


#============================================
def _choose_next_bond(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Choose one visible closed Next bond value through its Qt client."""
	combo = _drawing_parameter_combo(window, "Next bond")
	combo.setFocus()
	combo.setCurrentText(label)
	qapp.processEvents()


#============================================
def _choose_next_presentation(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Choose one visible directed presentation for the next Draw Bond gesture."""
	combo = _drawing_parameter_combo(window, "Next presentation")
	combo.setFocus()
	combo.setCurrentText(label)
	qapp.processEvents()


#============================================
def _next_drawing_choices(
		window: PySide6.QtWidgets.QMainWindow,
		) -> tuple[str, str, str]:
	"""Return the current visible Next Drawing values for later restoration."""
	return (
		_drawing_parameter_combo(window, "Next atom").currentText(),
		_drawing_parameter_combo(window, "Next bond").currentText(),
		_drawing_parameter_combo(window, "Next presentation").currentText(),
	)


#============================================
def _restore_next_drawing_choices(
		window: PySide6.QtWidgets.QMainWindow, choices: tuple[str, str, str],
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Return the real application preference owner to its prior visible values."""
	_choose_next_element(window, choices[0], qapp)
	_choose_next_bond(window, choices[1], qapp)
	_choose_next_presentation(window, choices[2], qapp)


#============================================
def _restore_drawing_parameters(
		window: ferrum_qt.main_window.MainWindow, snapshot: object,
		) -> None:
	"""Restore application-owned choices after a behavior test changes them."""
	window._drawing_parameters.set_element(snapshot.element)
	window._drawing_parameters.set_order_name(snapshot.order_name)
	window._drawing_parameters.set_presentation_name(snapshot.presentation_name)


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
def test_editing_tools_draw_bond_commits_rust_and_escape_preserves_result(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Editing Tools reaches the Rust-native bond gesture and safe Escape recovery."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	prior_choices = _next_drawing_choices(window)
	try:
		start = _atom_viewport_point(tab, "atom-c")
		end = _atom_viewport_point(tab, "atom-o")
		_choose_next_presentation(window, "Solid wedge from start atom", qapp)
		_editing_tool_button(window, "Draw Bond").click()
		assert "Solid wedge (Single)" in window.statusBar().currentMessage()
		assert "narrow tip to the wide end" in window._draw_bond_action.toolTip()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		selected_bond_id = tab.selected_bond_projection().source_id
		bond = tab.selected_bond_projection()
		assert bond.source_type == "w1"
		assert (bond.start.source_id, bond.end.source_id) == ("atom-c", "atom-o")
		accepted_snapshot = tab.current_snapshot
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape,
		)
		assert tab.current_snapshot == accepted_snapshot
		assert tab.selected_bond_projection().source_id == selected_bond_id
		tab.save_atomic(tmp_path / "drag-bond.cdml")
	finally:
		_restore_next_drawing_choices(window, prior_choices, qapp)
		window.close()
		window.deleteLater()


#============================================
def test_next_drawing_menu_route_updates_the_shared_application_choice(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The compact menu client changes the same next-drawing model as the toolbar."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	window.show()
	qapp.processEvents()
	prior_choices = window._drawing_parameters.snapshot()
	try:
		assert window._next_drawing_action in window._edit_menu.actions()

		def choose_from_dialog() -> None:
			dialog = qapp.activeModalWidget()
			if not isinstance(dialog, PySide6.QtWidgets.QDialog):
				raise AssertionError("Next Drawing action did not open its dialog")
			for combo in dialog.findChildren(PySide6.QtWidgets.QComboBox):
				if combo.accessibleName() == "Next presentation":
					combo.setCurrentText("Hashed wedge from start atom")
					dialog.reject()
					return
			raise AssertionError("Next Drawing dialog has no presentation chooser")

		PySide6.QtCore.QTimer.singleShot(0, choose_from_dialog)
		window._next_drawing_action.trigger()
		qapp.processEvents()
		assert window._drawing_parameters.snapshot().presentation_name == "hashed_wedge"
		assert _drawing_parameter_combo(
			window, "Next presentation",
		).currentText() == "Hashed wedge from start atom"
	finally:
		window._drawing_parameters.set_element(prior_choices.element)
		window._drawing_parameters.set_order_name(prior_choices.order_name)
		window._drawing_parameters.set_presentation_name(prior_choices.presentation_name)
		window.close()
		window.deleteLater()


#============================================
def test_editing_tools_cancel_preserves_document_and_selected_atom(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The visible Cancel Tool client preserves current native work and selection."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "cancel-tool.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	prior_choices = None
	try:
		prior_choices = window._drawing_parameters.snapshot()
		tab.select_atom("atom-c")
		before_snapshot = tab.current_snapshot
		before_atom_id = tab.selected_atom_projection().source_id
		window._drawing_parameters.set_element("N")
		window._drawing_parameters.set_order_name("triple")
		_editing_tool_button(window, "Draw Bond").click()
		_click_visible_menu_action(window, "Cancel Tool", qapp)
		assert tab.current_snapshot == before_snapshot
		assert tab.selected_atom_projection().source_id == before_atom_id
		assert window._drawing_parameters.snapshot() == (
			ferrum_qt.native.ferrum_native_drawing_parameters.
			FerrumNativeDrawingParametersSnapshot("N", "triple", "normal")
		)
	finally:
		if prior_choices is not None:
			_restore_drawing_parameters(window, prior_choices)
		window.close()
		window.deleteLater()


#============================================
def test_draw_bond_stale_gesture_preserves_intervening_snapshot_and_selection(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A gesture captured before another edit cannot commit against the new revision."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	monkeypatch.setattr(
		PySide6.QtWidgets.QMessageBox, "warning",
		lambda *_args: PySide6.QtWidgets.QMessageBox.StandardButton.Ok,
	)
	start = _atom_viewport_point(tab, "atom-c")
	end = _atom_viewport_point(tab, "atom-o")
	_editing_tool_button(window, "Draw Bond").click()
	PySide6.QtTest.QTest.mousePress(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
	)
	tab.select_atom("atom-c")
	tab.change_selected_atom_element("N")
	intervening_snapshot = tab.current_snapshot
	intervening_selection = tab.selected_atom_projection().source_id
	PySide6.QtTest.QTest.mouseRelease(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
	)
	assert (
		tab.current_snapshot == intervening_snapshot
		and tab.selected_atom_projection().source_id == intervening_selection
	)
	tab.save_atomic(tmp_path / "stale-drag.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_draw_bond_to_empty_space_uses_chosen_atom_order_and_shared_snap_point(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Empty-space release commits one chosen atom/order at the shared snap point."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "extend-bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	prior_choices = window._drawing_parameters.snapshot()
	try:
		start = _atom_viewport_point(tab, "atom-c")
		end = _empty_viewport_point(tab)
		end_scene = tab.view.snap_authored_scene_point(tab.view.mapToScene(end))
		window._drawing_parameters.set_element("O")
		window._drawing_parameters.set_order_name("triple")
		_editing_tool_button(window, "Draw Bond").click()
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
		created = next(
			atom for atom in molecule.atoms if atom.source_id not in {"atom-c", "atom-o"}
		)
		assert (
			created.element, created.position.x, created.position.y,
		) == ("O", end_scene.x(), end_scene.y())
		assert molecule.bonds[0].source_type == "n3"
		tab.save_atomic(tmp_path / "empty-space-bond.cdml")
	finally:
		_restore_drawing_parameters(window, prior_choices)
		window.close()
		window.deleteLater()


#============================================
def test_draw_bond_gesture_uses_parameters_captured_at_mouse_press(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""One active drag retains the element and order visible when it began."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "captured-drawing.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	prior_choices = _next_drawing_choices(window)
	try:
		start = _atom_viewport_point(tab, "atom-c")
		end = _empty_viewport_point(tab)
		_choose_next_element(window, "N", qapp)
		_choose_next_presentation(window, "Hashed wedge from start atom", qapp)
		_editing_tool_button(window, "Draw Bond").click()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		_choose_next_element(window, "O", qapp)
		_choose_next_presentation(window, "Normal", qapp)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		molecule = tab.current_document_observation().projection.molecules[0]
		created = next(
			atom for atom in molecule.atoms if atom.source_id not in {"atom-c", "atom-o"}
		)
		assert created.element == "N"
		assert molecule.bonds[0].source_type == "h1"
	finally:
		_restore_next_drawing_choices(window, prior_choices, qapp)
		window.close()
		window.deleteLater()


#============================================
def test_move_atom_drag_snaps_the_translated_atom_target(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The move tool applies the shared snap policy after pointer translation."""
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
	expected = tab.view.snap_authored_scene_point(anchor + (end_pointer - start_pointer))
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
	assert (atom.position.x, atom.position.y) == (expected.x(), expected.y())
	tab.undo()
	restored = tab._document_observation.projection.molecules[0].atoms[0].position
	assert (restored.x, restored.y) == (10.0, 20.0)
	window._move_atom_action.trigger()
	tab.save_atomic(tmp_path / "moved-atom.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_move_complete_roots_drag_resolves_one_snapped_rust_anchor_delta(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A visible mixed-root move applies one snapped rigid translation."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_MIXED_ROOT_CDML, "snapped-roots.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	try:
		durable_selection = _select_mixed_complete_roots(tab)
		window._refresh_actions()
		start = _atom_viewport_point(tab, "atom-c")
		end = _empty_viewport_point(tab)
		start_scene = tab.view.mapToScene(start)
		end_scene = tab.view.mapToScene(end)
		receipt = tab.selected_top_level_translation()
		resolved_anchor = tab.view.resolve_authored_scene_point(
			PySide6.QtCore.QPointF(
				receipt.anchor_x + end_scene.x() - start_scene.x(),
				receipt.anchor_y + end_scene.y() - start_scene.y(),
			), True,
		)
		expected_delta = (
			resolved_anchor.x() - receipt.anchor_x,
			resolved_anchor.y() - receipt.anchor_y,
		)
		_click_visible_menu_action(window, "Move Complete Roots", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		atom, plus = _mixed_root_positions(tab)
		assert (
			atom == pytest.approx(
				(13.0 + expected_delta[0], 17.0 + expected_delta[1]),
				abs=_AUTHORED_COORDINATE_TOLERANCE,
			)
			and plus == pytest.approx(
				(73.0 + expected_delta[0], 51.0 + expected_delta[1]),
				abs=_AUTHORED_COORDINATE_TOLERANCE,
			)
			and tab.selected_top_level_transform_targets()[1] == durable_selection
		)
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_move_complete_roots_drag_keeps_the_unsnapped_raw_delta(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Disabling the shared preference retains raw pointer displacement."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_MIXED_ROOT_CDML, "raw-roots.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	try:
		_select_mixed_complete_roots(tab)
		window._refresh_actions()
		tab.view.set_hex_grid_snap_enabled(False)
		start = _atom_viewport_point(tab, "atom-c")
		end = _empty_viewport_point(tab)
		start_scene = tab.view.mapToScene(start)
		end_scene = tab.view.mapToScene(end)
		expected_delta = (end_scene.x() - start_scene.x(), end_scene.y() - start_scene.y())
		_click_visible_menu_action(window, "Move Complete Roots", qapp)
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		atom, plus = _mixed_root_positions(tab)
		assert (
			atom == pytest.approx(
				(13.0 + expected_delta[0], 17.0 + expected_delta[1]),
				abs=_AUTHORED_COORDINATE_TOLERANCE,
			)
			and plus == pytest.approx(
				(73.0 + expected_delta[0], 51.0 + expected_delta[1]),
				abs=_AUTHORED_COORDINATE_TOLERANCE,
			)
		)
	finally:
		window.close()
		window.deleteLater()


#============================================
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

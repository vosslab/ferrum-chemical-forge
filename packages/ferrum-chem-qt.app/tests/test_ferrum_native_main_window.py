"""Behavior coverage for the public Ferrum window seam."""

# Standard Library
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import tests.ferrum_native_menu_actions
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.main_window
import ferrum_qt.ferrum.direct_bond_gesture_tab
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.window_refusals


_EDITABLE_CDML = """<cdml xmlns='urn:ferrum:cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  </molecule>
</cdml>"""


#============================================
def test_change_element_action_rechecks_selection_and_uses_the_bounded_dialog(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""The one owned Edit action submits only an eligible dialog acceptance."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "change-element.cdml",
	)
	window._register_native_tab(tab, activate=True)
	qapp.processEvents()
	assert not window._change_element_action.isEnabled()
	tab.select_atom("atom-c")
	window._refresh_actions()
	assert window._change_element_action.isEnabled()
	calls: list[tuple[str, str]] = []
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText",
		lambda _parent, title, label: (calls.append((title, label)) or ("N", True)),
	)
	window._change_element_action.trigger()
	assert calls == [(window.tr("Change Atom Element"), window.tr("Element symbol:"))]
	assert tab.selected_atom_projection().element == "N"
	tab.save_atomic(tmp_path / "change-element.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_change_element_dialog_cancel_and_refusal_preserve_tab_truth(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Cancel and Rust refusal use the window route without partial document edits."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "change-element-refusal.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	monkeypatch.setattr(PySide6.QtWidgets.QInputDialog, "getText", lambda *_args: ("", False))
	window._change_element_action.trigger()
	assert tab.current_snapshot is prior
	refusals = []
	monkeypatch.setattr(window, "_show_edit_refusal", lambda refusal: refusals.append(refusal))
	monkeypatch.setattr(PySide6.QtWidgets.QInputDialog, "getText", lambda *_args: ("Xx", True))
	window._change_element_action.trigger()
	assert tab.current_snapshot is prior and len(refusals) == 1
	window.close()
	window.deleteLater()


_BOND_CDML = """<cdml xmlns='urn:ferrum:cdml' version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
  <atom id='atom-o' name='O'><point x='40' y='20'/></atom>
</molecule></cdml>"""

_MIXED_ROOT_CDML = """<cdml xmlns='urn:ferrum:cdml' version='26.08'><molecule id='mol-1'>
  <atom id='atom-c' name='C'><point x='13' y='17'/></atom>
</molecule><plus id='plus-1'><point x='73' y='51'/></plus></cdml>"""

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
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
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
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
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
	raise AssertionError("Ferrum viewport has no empty hit-test point")


#============================================
def _select_mixed_complete_roots(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
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
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[tuple[float, float], tuple[float, float]]:
	"""Return authoritative molecule and presentation coordinates after a move."""
	projection = tab.current_document_observation().projection
	atom = projection.molecules[0].atoms[0].position
	plus = projection.presentation_stack.roots[0].plus.anchor
	return (atom.x, atom.y), (plus.x, plus.y)


#============================================
#============================================
def _visible_form_field(
		dialog: PySide6.QtWidgets.QDialog, label: str,
		) -> PySide6.QtWidgets.QWidget:
	"""Return the visible form field paired with one visible dialog label."""
	for form in dialog.findChildren(PySide6.QtWidgets.QFormLayout):
		for row in range(form.rowCount()):
			label_item = form.itemAt(row, PySide6.QtWidgets.QFormLayout.ItemRole.LabelRole)
			if label_item is None:
				continue
			label_widget = label_item.widget()
			if not isinstance(label_widget, PySide6.QtWidgets.QLabel):
				continue
			if label_widget.text() != label:
				continue
			field_item = form.itemAt(row, PySide6.QtWidgets.QFormLayout.ItemRole.FieldRole)
			if field_item is None or field_item.widget() is None:
				raise AssertionError(f"Visible form label {label!r} has no field")
			field = field_item.widget()
			if not field.isVisible():
				raise AssertionError(f"Form field for {label!r} is not visible")
			return field
	raise AssertionError(f"Visible property form has no {label!r} field")


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
	"""The standalone window reaches its Ferrum controller, not a fallback base."""
	del qapp
	source = tmp_path / "source.cdml"
	source.write_text("<cdml xmlns='urn:ferrum:cdml'/>", encoding="utf-8")
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	loop = PySide6.QtCore.QEventLoop()
	outcomes = []

	def finish(success: bool) -> None:
		"""Capture the complete admission result and stop the local event loop."""
		outcomes.append(success)
		loop.quit()

	window.local_document_open_queue_drained.connect(finish)
	try:
		assert window.open_file_path(str(source))
		loop.exec()
		tab_widget = window.centralWidget()
		assert outcomes == [True] and isinstance(tab_widget, PySide6.QtWidgets.QTabWidget)
		tab = tab_widget.currentWidget()
		assert isinstance(
			tab, ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		)
		assert tab.file_path == source and not tab.current_snapshot.is_dirty
	finally:
		window.close()


#============================================
def test_history_actions_mirror_active_tab_rust_availability(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""Only the active tab's Rust history facts enable the guarded Edit actions."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	history_tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "history-actions.cdml",
	)
	fresh_tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "fresh-history-actions.cdml",
	)
	window._register_native_tab(history_tab, activate=True)
	try:
		assert not window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		history_tab.select_atoms(("atom-c", "atom-o"))
		window._add_single_bond_action.trigger()
		assert window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		window._register_native_tab(fresh_tab, activate=True)
		assert not window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		window._tab_widget.setCurrentWidget(history_tab)
		qapp.processEvents()
		assert window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		with monkeypatch.context() as patch:
			patch.setattr(type(history_tab), "requires_refresh", property(lambda _tab: True))
			window._refresh_actions()
			assert not window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		assert history_tab.can_undo() and not history_tab.can_redo()
		with monkeypatch.context() as patch:
			patch.setattr(window, "_molecule_import_busy", lambda: True)
			window._refresh_actions()
			assert not window._undo_action.isEnabled() and not window._redo_action.isEnabled()
		assert history_tab.can_undo() and not history_tab.can_redo()
	finally:
		history_tab.save_atomic(tmp_path / "history-actions.cdml")
		window.close()
		window.deleteLater()


#============================================
def test_add_single_bond_action_connects_exact_selected_atoms(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The public edit action commits, selects, and saves one Rust-owned bond."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atoms(("atom-c", "atom-o"))
	window._add_single_bond_action.trigger()
	selected = tab._controller.projection.selected_durable_targets()
	assert len(selected) == 1 and selected[0].kind == "bond"
	assert 'type="n1" start="atom-c" end="atom-o"' in tab.current_snapshot.cdml
	assert "Added one Ferrum single bond." in window.statusBar().currentMessage()
	tab.save_atomic(tmp_path / "bonded.cdml")
	window.close()
	window.deleteLater()


#============================================
def test_delete_selected_atom_action_removes_incident_bond_through_rust(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The public delete action commits one undoable Rust topology change."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
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
	assert "Deleted one Ferrum atom" in window.statusBar().currentMessage()
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
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
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
	assert "Deleted one Ferrum bond" in window.statusBar().currentMessage()
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
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
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
def test_visible_property_actions_persist_one_atom_and_bond_change(
		tmp_path: pathlib.Path,
		) -> None:
	"""The visible property forms save accepted atom and bond edits through Rust."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "property-actions.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	app.processEvents()

	def accept_atom_charge() -> None:
		"""Set one visible atom field and accept its real modal dialog."""
		dialog = app.activeModalWidget()
		if not isinstance(dialog, PySide6.QtWidgets.QDialog):
			raise AssertionError("Atom Properties dialog did not open")
		charge_spin = _visible_form_field(dialog, "Charge:")
		if not isinstance(charge_spin, PySide6.QtWidgets.QSpinBox):
			raise AssertionError("Charge field is not a visible spin box")
		charge_spin.setValue(1)
		dialog.accept()

	def accept_double_bond() -> None:
		"""Set one visible bond field and accept its real modal dialog."""
		dialog = app.activeModalWidget()
		if not isinstance(dialog, PySide6.QtWidgets.QDialog):
			raise AssertionError("Bond Properties dialog did not open")
		order_combo = _visible_form_field(dialog, "Order:")
		if not isinstance(order_combo, PySide6.QtWidgets.QComboBox):
			raise AssertionError("Order field is not a visible combo box")
		order_combo.setCurrentText("Double")
		dialog.accept()

	try:
		tab.select_atom("atom-c")
		window._refresh_actions()
		tests.ferrum_native_menu_actions.click_visible_menu_action(
			window, "Edit Atom Properties", app,
			lambda: PySide6.QtCore.QTimer.singleShot(0, accept_atom_charge),
		)
		assert tab.selected_atom_projection().formal_charge == 1
		assert "Updated one Ferrum atom." in window.statusBar().currentMessage()

		tab.select_atoms(("atom-c", "atom-o"))
		window._add_single_bond_action.trigger()
		tests.ferrum_native_menu_actions.click_visible_menu_action(
			window, "Edit Bond Properties", app,
			lambda: PySide6.QtCore.QTimer.singleShot(0, accept_double_bond),
		)
		assert tab.selected_bond_projection().source_type == "n2"
		assert "Updated one bond." in window.statusBar().currentMessage()

		saved = tmp_path / "property-actions.cdml"
		tab.save_atomic(saved)
		persisted = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
			saved.read_text(encoding="utf-8"), "reopened-property-actions.cdml",
		)
		try:
			molecule = persisted.current_document_observation().projection.molecules[0]
			assert molecule.atoms[0].formal_charge == 1
			assert molecule.bonds[0].source_type == "n2"
		finally:
			persisted.dispose()
	finally:
		window.close()
		window.deleteLater()


#============================================

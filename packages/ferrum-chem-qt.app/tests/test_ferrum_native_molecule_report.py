"""Visible native Molecule Report behavior."""

# Standard Library
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.molecule_report
import ferrum_qt.ferrum.molecule_report_stereo_contract


_DIAGNOSTIC_CDML = """<cdml xmlns="urn:ferrum:cdml" version="26.08"><molecule id="m">
<atom id="c" name="C"><point x="0" y="0"/></atom>
<text id="text"><point x="2" y="0"/></text>
</molecule></cdml>"""


#============================================
def _open_selected_document() -> tuple[object, object]:
	"""Open one native document with a selected molecule through the normal tab seam."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_DIAGNOSTIC_CDML, "diagnostic.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("c")
	window._refresh_actions()
	return window, tab


#============================================
def _report_dialog(window: object) -> PySide6.QtWidgets.QDialog | None:
	"""Find the visible modeless report surface by its public widget identity."""
	return window.findChild(PySide6.QtWidgets.QDialog, "molecule-report-dialog")


#============================================
def _visible_child_index(
		model: PySide6.QtCore.QAbstractItemModel,
		parent: PySide6.QtCore.QModelIndex,
		text: str,
		) -> PySide6.QtCore.QModelIndex:
	"""Find a displayed report-tree child by its user-visible text."""
	for row in range(model.rowCount(parent)):
		index = model.index(row, 0, parent)
		if text in str(model.data(index, PySide6.QtCore.Qt.DisplayRole)):
			return index
	raise AssertionError(f"Missing visible report item: {text}")


#============================================
def test_selected_molecule_report_shows_rust_diagnostic(
		qapp: PySide6.QtWidgets.QApplication, qtbot: object,
		) -> None:
	"""The visible action presents one real Rust-issued diagnostic without local chemistry."""
	window, tab = _open_selected_document()
	try:
		window.show()
		qapp.processEvents()
		tab.set_selected_atom_number(1, True)
		window._refresh_actions()
		assert tab.current_snapshot.revision > 0
		action = window.findChild(PySide6.QtGui.QAction, "molecule-report-action")
		assert action is not None and action.isEnabled() and action.text() == "Molecule Report..."
		action.trigger()
		qtbot.waitUntil(lambda: _report_dialog(window) is not None, timeout=5000)
		dialog = _report_dialog(window)
		assert dialog is not None and dialog.isVisible() and not dialog.isModal()
		tree = dialog.findChild(PySide6.QtWidgets.QTreeView, "molecule-report-tree")
		details = dialog.findChild(PySide6.QtWidgets.QPlainTextEdit, "molecule-report-details")
		assert tree is not None and details is not None
		model = tree.model()
		molecule = _visible_child_index(model, PySide6.QtCore.QModelIndex(), "m")
		diagnostics = _visible_child_index(model, molecule, "Diagnostics")
		finding = _visible_child_index(model, diagnostics, "text_atom_present")
		tree.setCurrentIndex(finding)
		assert (
			"Code: text_atom_present" in details.toPlainText()
			and "Location: vertex: text" in details.toPlainText()
			and "Recovery: choose_supported_representation" in details.toPlainText()
		)
	finally:
		action = window.findChild(PySide6.QtGui.QAction, "molecule-report-action")
		if action is not None:
			qtbot.waitUntil(action.isEnabled, timeout=5000)
		dialog = _report_dialog(window)
		if dialog is not None:
			dialog.close()
			qtbot.waitUntil(lambda: not dialog.isVisible(), timeout=5000)
		while tab.current_snapshot.is_dirty:
			tab.undo()
		window._close_tab_at(window._tab_widget.indexOf(tab))
		window.deleteLater()


#============================================
def test_delivered_molecule_report_becomes_visibly_stale_after_source_mutation(
		qapp: PySide6.QtWidgets.QApplication, qtbot: object,
		) -> None:
	"""A displayed snapshot report declares stale provenance after a real session mutation."""
	window, tab = _open_selected_document()
	try:
		window.show()
		qapp.processEvents()
		action = window.findChild(PySide6.QtGui.QAction, "molecule-report-action")
		assert action is not None and action.isEnabled()
		action.trigger()
		qtbot.waitUntil(lambda: _report_dialog(window) is not None, timeout=5000)
		dialog = _report_dialog(window)
		assert dialog is not None
		tab.change_selected_atom_element("N")
		window._refresh_actions()
		warning = dialog.findChild(PySide6.QtWidgets.QLabel, "molecule-report-stale-warning")
		assert warning is not None and warning.isVisible()
	finally:
		dialog = _report_dialog(window)
		if dialog is not None:
			dialog.close()
			qtbot.waitUntil(lambda: not dialog.isVisible(), timeout=5000)
		while tab.current_snapshot.is_dirty:
			tab.undo()
		window._close_tab_at(window._tab_widget.indexOf(tab))
		window.deleteLater()


#============================================
def test_source_tab_closes_while_detached_molecule_report_retires(
		qapp: PySide6.QtWidgets.QApplication, qtbot: object,
		) -> None:
	"""Closing a report source revokes presentation delivery without blocking tab disposal."""
	window, tab = _open_selected_document()
	try:
		window.show()
		qapp.processEvents()
		action = window.findChild(PySide6.QtGui.QAction, "molecule-report-action")
		assert action is not None and action.isEnabled()
		action.trigger()
		assert window._molecule_report_intent is not None
		window._close_tab_at(window._tab_widget.indexOf(tab))
		assert window._tab_widget.indexOf(tab) == -1
		qtbot.waitUntil(lambda: window._molecule_report_intent is None, timeout=5000)
		assert _report_dialog(window) is None
	finally:
		window.deleteLater()


#============================================
def test_molecule_report_ingress_rejects_malformed_refusals_and_classifies_resource_refusal() -> None:
	"""The decoder rejects malformed input and uses resource recovery before rendering."""
	resource_refusal = {
		"schema": "ferrum-operation-error-v1",
		"request_id": "qt-molecule-report",
		"error": {
			"category": "resource_limit",
			"operation": "document.molecule.report.v1",
			"message": "ignored presentation detail",
			"resource_limit": {
				"reason": "response_size_exceeded",
				"recovery": "reduce_requested_result",
			},
		},
	}
	malformed_refusal = {**resource_refusal, "error": {
		**resource_refusal["error"],
		"resource_limit": {"reason": "response_size_exceeded", "recovery": "retry"},
	}}
	assert ferrum_qt.ferrum.molecule_report.decode_molecule_report_refusal(malformed_refusal) is not None
	refusal = ferrum_qt.ferrum.molecule_report.decode_molecule_report_refusal(resource_refusal)
	assert refusal is not None and refusal.recovery == "reduce_requested_result"
	assert "Reduce the selected molecules" in refusal.message


#============================================
def test_molecule_report_stereo_receipt_requires_typed_rust_facts() -> None:
	"""The Qt boundary displays admitted descriptors and refuses absent or malformed facts."""
	semantics = {
		"tetrahedral": [{
			"center": 4,
			"ligands": [
				{"kind": "atom", "index": 2},
				{"kind": "explicit_hydrogen"},
				{"kind": "atom", "index": 8},
				{"kind": "atom", "index": 9},
			],
			"parity": "clockwise",
		}],
		"double_bonds": [
			{"bond_index": 7, "start_ligand": 1, "end_ligand": 6, "configuration": "e"},
			{"bond_index": 10, "start_ligand": 3, "end_ligand": 11, "configuration": "z"},
		],
	}
	depiction = {
		"directed_bonds": [{
			"bond_index": 5, "start": 4, "end": 2, "presentation": "solid_wedge",
		}],
		"double_bond_carrier_marks": [{
			"double_bond_index": 7, "carrier_bond_index": 6, "mark": "up",
		}],
	}
	contract = ferrum_qt.ferrum.molecule_report_stereo_contract
	assert contract.valid_stereo_semantics(semantics)
	assert contract.valid_stereo_depiction(depiction)
	assert contract.display_lines(semantics, depiction) == [
		"Stereo semantics:",
		"  tetrahedral atom 4: [2, explicit hydrogen, 8, 9], clockwise",
		"  double bond 7: ligands 1/6, e",
		"  double bond 10: ligands 3/11, z",
		"Stereo depiction:",
		"  directed bond 5: 4 -> 2, solid_wedge",
		"  double bond carrier: double bond 7, carrier bond 6, up",
	]
	assert contract.display_lines(None, None) == ["Stereo semantics: none", "Stereo depiction: none"]
	assert not contract.valid_stereo_semantics({
		"tetrahedral": [{"center": 4, "ligands": [], "parity": "clockwise"}],
		"double_bonds": [],
	})
	assert not contract.valid_stereo_depiction({
		"directed_bonds": [],
		"double_bond_carrier_marks": [{"double_bond_index": 7, "carrier_bond_index": 6, "mark": "sideways"}],
	})
	assert not contract.valid_stereo_semantics({
		"tetrahedral": [{
			"center": 4, "ligands": [{"kind": "explicit_hydrogen"}] * 4,
			"parity": ["clockwise"],
		}],
		"double_bonds": [{
			"bond_index": 7, "start_ligand": 1, "end_ligand": 6,
			"configuration": ["e"],
		}],
	})
	assert not contract.valid_stereo_depiction({
		"directed_bonds": [{
			"bond_index": 5, "start": 4, "end": 2, "presentation": ["solid_wedge"],
		}],
		"double_bond_carrier_marks": [{
			"double_bond_index": 7, "carrier_bond_index": 6, "mark": ["up"],
		}],
	})
	assert not ferrum_qt.ferrum.molecule_report._valid_record({}, object())

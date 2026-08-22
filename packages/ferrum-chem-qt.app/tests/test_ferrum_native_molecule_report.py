"""Public JSON protocol coverage for the modeless Molecule Report client."""

# Standard Library
import json
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.molecule_report


_SOURCE = """<cdml xmlns="urn:ferrum:cdml" version='26.08'><molecule id='mixed' name='Mixed'>
<atom id='carbon' name='C' explicit_hydrogens='4'><point x='0' y='0'/></atom>
<atom id='oxygen' name='O' charge='0'><point x='10' y='0'/></atom>
<bond id='bond' start='carbon' end='oxygen' type='n1'/>
</molecule></cdml>"""

_SECOND_SOURCE = """<cdml xmlns="urn:ferrum:cdml" version='26.08'><molecule id='second' name='Second'>
<atom id='carbon' name='C' explicit_hydrogens='4'><point x='0' y='0'/></atom>
</molecule></cdml>"""


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide the offscreen application used by this Qt worker-state test."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
class _PublicProtocolFake:
	"""One narrow public JSON boundary fake, with no chemistry or private DTOs."""

	def __init__(self, extension: object) -> None:
		"""Record the one request sent through the frozen public operation route."""
		self._extension = extension
		self.requests: list[dict] = []

	def __getattr__(self, name: str) -> object:
		"""Leave every non-report application capability on the real extension."""
		value = getattr(self._extension, name)
		return value

	def execute_operation_v1(self, request_json: str) -> str:
		"""Return report facts derived only from request identity fields for testing."""
		request = json.loads(request_json)
		self.requests.append(request)
		operation = request["operation"]
		records = []
		for order, molecule_id in enumerate(operation["molecule_ids"]):
			records.append({
				"molecule_id": molecule_id,
				"source_id": "mixed",
				"document_root_order": order,
				"atom_count": 2,
				"bond_count": 1,
				"authored_elements": [{"symbol": "C", "atom_count": 1}, {
					"symbol": "O", "atom_count": 1,
				}],
				"authored_name": "Mixed",
				"authored_charge": 0,
				"composition": {
					"formula": "CH4O",
					"net_formal_charge": 0,
					"average_molecular_weight_da": 32.04186,
					"monoisotopic_mass_da": 32.02621475,
					"elements": [{
						"symbol": "C",
						"isotope": 13,
						"atom_count": 1,
						"average_mass_contribution_da": 13.00335484,
						"mass_percentage": 40.5823,
					}, {
						"symbol": "H",
						"isotope": None,
						"atom_count": 4,
						"average_mass_contribution_da": 4.03176,
						"mass_percentage": 12.5835,
					}, {
						"symbol": "O",
						"isotope": None,
						"atom_count": 1,
						"average_mass_contribution_da": 15.9994,
						"mass_percentage": 46.8342,
					}],
				},
				"neutral_bond_capacity": "exceeds_capacity",
				"finding_codes": ["neutral_capacity_exceeded"],
			})
		response = {
			"schema": "ferrum-operation-response-v1",
			"request_id": request["request_id"],
			"outcome": {
				"kind": "document.molecule.report.v1",
				"report": {
					"schema": "ferrum-document-molecule-report-v1",
					"source_revision": operation["expected_revision"],
					"source_digest_hex": operation["expected_digest_hex"],
					"records": records,
					"aggregate": {
						"kind": "omitted",
						"reason": "fewer_than_two_selected",
					},
				},
			},
		}
		response_json = json.dumps(response)
		return response_json


#============================================
class _DeferredReportWorker(PySide6.QtCore.QObject):
	"""Delay one public-protocol delivery so cancellation has an exact test turn."""

	reported = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)
	finished = PySide6.QtCore.Signal()
	current: "_DeferredReportWorker | None" = None

	def __init__(self, request_json: str) -> None:
		"""Keep the public request text but do not inspect its document content."""
		super().__init__()
		self.request_json = request_json
		self._delivery_cancelled = False

	@property
	def delivery_cancelled(self) -> bool:
		"""Match the production worker delivery contract."""
		return self._delivery_cancelled

	def cancel_delivery(self) -> None:
		"""Suppress the later controlled delivery."""
		self._delivery_cancelled = True

	def start(self) -> None:
		"""Retain the worker until this test explicitly issues delivery."""
		type(self).current = self

	def deliver(self, response: dict) -> None:
		"""Emit one stale-or-current envelope through the production relay shape."""
		self.reported.emit(response)
		self.finished.emit()

	def deleteLater(self) -> None:
		"""Provide the normal Qt retirement surface to the production controller."""


#============================================
def _open_window() -> tuple[object, object]:
	"""Create one selected native tab through the ordinary public window seam."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_SOURCE, "mixed.cdml")
	window._register_native_tab(tab, activate=True)
	tab.select_atom("carbon")
	window._refresh_actions()
	return window, tab


#============================================
def test_report_action_uses_only_public_json_and_opens_modeless_view(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""One selected root creates a report without a typed report API or document edit."""
	protocol = _PublicProtocolFake(ferrum_qt.ferrum.engine.extension_module())
	monkeypatch.setattr(ferrum_qt.ferrum.engine, "extension_module", lambda: protocol)
	window, tab = _open_window()
	try:
		before = tab.current_snapshot
		selection = tab.selected_molecule_information_targets()
		assert window._molecule_report_action.isEnabled()
		assert window._molecule_report_action.text() == "Molecule Report..."
		window._molecule_report_action.trigger()
		intent = window._molecule_report_intent
		assert intent is not None and intent.worker.wait(10000)
		qapp.processEvents()
		dialog = window._molecule_report_dialog
		assert dialog is not None and not dialog.isModal()
		assert protocol.requests[0]["operation"]["kind"] == "document.molecule.report.v1"
		assert protocol.requests[0]["operation"]["molecule_ids"] == [
			address.molecule_id for address in intent.addresses
		]
		assert dialog.findChild(PySide6.QtWidgets.QTreeView, "molecule-report-tree") is not None
		details = dialog.findChild(PySide6.QtWidgets.QPlainTextEdit, "molecule-report-details")
		assert details is not None and "Formula: CH4O" in details.toPlainText()
		assert "Average molecular weight: 32.041860 Da" in details.toPlainText()
		assert "13C: 1 atoms; 13.003355 Da (40.5823%)" in details.toPlainText()
		aggregate = dialog._model.item(dialog._model.rowCount() - 1)
		assert aggregate.data(PySide6.QtCore.Qt.ItemDataRole.UserRole) == (
			"Aggregate composition: omitted (fewer_than_two_selected)"
		)
		dialog._copy_report()
		copied = PySide6.QtWidgets.QApplication.clipboard().text()
		assert copied.count("Aggregate composition: omitted (fewer_than_two_selected)") == 1
		assert not dialog.findChild(PySide6.QtWidgets.QPushButton, "molecule-report-show-canvas").isEnabled()
		assert tab.current_snapshot == before
		assert tab.selected_molecule_information_targets() == selection
	finally:
		if window._molecule_report_dialog is not None:
			window._molecule_report_dialog.close()
		window._close_tab_at(window._tab_widget.indexOf(tab))
		window.deleteLater()


#============================================
def test_cancelled_or_stale_public_delivery_never_opens_a_report(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Cancellation and stale data suppress presentation rather than inventing recovery."""
	protocol = _PublicProtocolFake(ferrum_qt.ferrum.engine.extension_module())
	monkeypatch.setattr(ferrum_qt.ferrum.engine, "extension_module", lambda: protocol)
	monkeypatch.setattr(
		ferrum_qt.ferrum.molecule_report,
		"FerrumNativeMoleculeReportWorker", _DeferredReportWorker,
	)
	window, tab = _open_window()
	try:
		before = tab.current_snapshot
		window._molecule_report_action.trigger()
		worker = _DeferredReportWorker.current
		assert worker is not None and window._cancel_molecule_report_action.isEnabled()
		window._cancel_molecule_report_action.trigger()
		assert worker.delivery_cancelled
		response = json.loads(protocol.execute_operation_v1(worker.request_json))
		worker.deliver(response)
		qapp.processEvents()
		assert window._molecule_report_dialog is None
		assert tab.current_snapshot == before
	finally:
		window._close_tab_at(window._tab_widget.indexOf(tab))
		window.deleteLater()


#============================================
def test_open_report_marks_stale_without_reinterpreting_old_receipt(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A later snapshot disables the unavailable reveal route and retains history text."""
	protocol = _PublicProtocolFake(ferrum_qt.ferrum.engine.extension_module())
	monkeypatch.setattr(ferrum_qt.ferrum.engine, "extension_module", lambda: protocol)
	window, tab = _open_window()
	try:
		window._molecule_report_action.trigger()
		intent = window._molecule_report_intent
		assert intent is not None and intent.worker.wait(10000)
		qapp.processEvents()
		dialog = window._molecule_report_dialog
		assert dialog is not None
		dialog._report["source_revision"] = -1
		window._refresh_actions()
		warning = dialog.findChild(PySide6.QtWidgets.QLabel, "molecule-report-stale-warning")
		assert warning is not None and warning.isVisible()
		assert "Formula: CH4O" in dialog.findChild(
			PySide6.QtWidgets.QPlainTextEdit, "molecule-report-details",
		).toPlainText()
	finally:
		if window._molecule_report_dialog is not None:
			window._molecule_report_dialog.close()
		window._close_tab_at(window._tab_widget.indexOf(tab))
		window.deleteLater()


#============================================
def test_complete_aggregate_uses_the_nested_rust_composition_dto(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A complete aggregate presents its exact Rust composition rather than local chemistry."""
	record = {
		"molecule_id": "mixed",
		"source_id": "mixed",
		"document_root_order": 0,
		"atom_count": 2,
		"bond_count": 1,
		"authored_elements": [{"symbol": "C", "atom_count": 1}],
		"authored_name": "Mixed",
		"authored_charge": 0,
		"composition": {
			"formula": "13CH4O",
			"net_formal_charge": 0,
			"average_molecular_weight_da": 33.038505,
			"monoisotopic_mass_da": 33.02957,
			"elements": [{
				"symbol": "C",
				"isotope": 13,
				"atom_count": 1,
				"average_mass_contribution_da": 13.003355,
				"mass_percentage": 39.3585,
			}],
		},
		"neutral_bond_capacity": "within_capacity",
		"finding_codes": [],
	}
	report = {
		"schema": "ferrum-document-molecule-report-v1",
		"source_revision": 0,
		"source_digest_hex": "a" * 64,
		"records": [record],
		"aggregate": {"kind": "complete", "composition": record["composition"]},
	}
	parent = PySide6.QtWidgets.QWidget()
	dialog = ferrum_qt.ferrum.molecule_report.FerrumNativeMoleculeReportDialog(
		report, object(), parent,
	)
	try:
		aggregate = dialog._model.item(dialog._model.rowCount() - 1)
		text = aggregate.data(PySide6.QtCore.Qt.ItemDataRole.UserRole)
		assert "Aggregate composition: complete" in text
		assert "Formula: 13CH4O" in text and "13C: 1 atoms; 13.003355 Da (39.3585%)" in text
	finally:
		dialog.close()
		parent.deleteLater()


#============================================
def test_report_rerun_never_redirects_to_another_active_tab(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A modeless report remains historical while a different document is active."""
	protocol = _PublicProtocolFake(ferrum_qt.ferrum.engine.extension_module())
	monkeypatch.setattr(ferrum_qt.ferrum.engine, "extension_module", lambda: protocol)
	window, source_tab = _open_window()
	second_tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_SECOND_SOURCE, "second.cdml")
	try:
		window._molecule_report_action.trigger()
		intent = window._molecule_report_intent
		assert intent is not None and intent.worker.wait(10000)
		qapp.processEvents()
		dialog = window._molecule_report_dialog
		assert dialog is not None
		window._register_native_tab(second_tab, activate=True)
		second_tab.select_atom("carbon")
		window._refresh_actions()
		dialog.rerun_button.click()
		assert not dialog.rerun_button.isEnabled() and window._molecule_report_intent is None
		assert source_tab.current_snapshot.revision == 0 and second_tab.current_snapshot.revision == 0
	finally:
		if window._molecule_report_dialog is not None:
			window._molecule_report_dialog.close()
		window._close_tab_at(window._tab_widget.indexOf(second_tab))
		window._close_tab_at(window._tab_widget.indexOf(source_tab))
		window.deleteLater()


#============================================
def test_closing_a_report_source_retires_its_modeless_dialog(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Closing a delivered report source removes its action surface before disposal."""
	protocol = _PublicProtocolFake(ferrum_qt.ferrum.engine.extension_module())
	monkeypatch.setattr(ferrum_qt.ferrum.engine, "extension_module", lambda: protocol)
	window, source_tab = _open_window()
	second_tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_SECOND_SOURCE, "second.cdml")
	try:
		window._molecule_report_action.trigger()
		intent = window._molecule_report_intent
		assert intent is not None and intent.worker.wait(10000)
		qapp.processEvents()
		dialog = window._molecule_report_dialog
		assert dialog is not None
		window._register_native_tab(second_tab, activate=True)
		window._close_tab_at(window._tab_widget.indexOf(source_tab))
		qapp.processEvents()
		dialog.rerun_button.click()
		assert window._molecule_report_dialog is None and not dialog.rerun_button.isEnabled()
		assert second_tab.current_snapshot.revision == 0
	finally:
		window._close_tab_at(window._tab_widget.indexOf(second_tab))
		window.deleteLater()

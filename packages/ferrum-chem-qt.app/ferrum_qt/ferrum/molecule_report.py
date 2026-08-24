"""Public-protocol Molecule Report client for one frozen Ferrum snapshot."""

# Standard Library
import dataclasses
import json
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
from ferrum_qt.dialogs.accessibility import FerrumAccessibleDialog
from ferrum_qt.ferrum.background_job import FerrumDetachedJobThread

# local repo modules
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.ferrum.molecule_inspection


_FINDING_SEVERITIES = {"info", "warning", "error"}
_FINDING_RECOVERIES = {
	"none",
	"inspect_structure",
	"correct_chemical_facts",
	"choose_supported_representation",
	"reduce_selection",
	"retry_with_chemistry_runtime",
}
_FINDING_SUBJECTS = {"atom", "vertex", "bond"}
_NEUTRAL_BOND_CAPACITY = {"within_capacity", "exceeds_capacity", "not_checked"}
_AGGREGATE_OMISSION_REASONS = {
	"fewer_than_two_selected", "incomplete_record_composition",
}


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _MoleculeReportIntent:
	"""One immutable public request and its Qt delivery fence."""

	tab: object
	revision: int
	digest: str
	addresses: tuple[object, ...]
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeMoleculeReportFailure:
	"""Safe typed terminal refusal delivered back to the Qt event loop."""

	message: str
	category: str | None = None
	recovery: str | None = None


#============================================
def _report_request(snapshot: object, addresses: tuple[object, ...]) -> str:
	"""Build the closed JSON request without inspecting CDML or chemistry facts."""
	request = {
		"schema": "ferrum-operation-request-v1",
		"request_id": "qt-molecule-report",
		"operation": {
			"kind": "document.molecule.report.v1",
			"snapshot": {
				"cdml": snapshot.cdml,
				"revision": snapshot.revision,
				"digest_hex": snapshot.digest,
			},
			"molecule_ids": [address.molecule_id for address in addresses],
		},
	}
	request_json = json.dumps(request, separators=(",", ":"), ensure_ascii=True)
	return request_json


#============================================
def _execute_report_request(execute_operation: object, request_json: str) -> dict:
	"""Run exactly the public JSON operation and decode its response envelope."""
	if not callable(execute_operation):
		raise TypeError("Ferrum molecule report requires a public operation callable")
	response_json = execute_operation(request_json)
	response = json.loads(response_json)
	if type(response) is not dict:
		raise TypeError("Ferrum molecule report returned a non-object protocol envelope")
	return response


#============================================
class FerrumNativeMoleculeReportWorker(FerrumDetachedJobThread):
	"""Run one public JSON request outside the Qt event thread."""

	reported = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, execute_operation: object, request_json: str) -> None:
		"""Retain a public callable and immutable protocol text, never a live session."""
		if type(request_json) is not str:
			raise TypeError("Ferrum molecule report requires a JSON request string")
		super().__init__(
			lambda: _execute_report_request(execute_operation, request_json),
			lambda error: FerrumNativeMoleculeReportFailure(str(error)),
		)

	#============================================
	def _emit_success(self, result: object) -> None:
		"""Publish the JSON envelope through the feature-specific signal."""
		self.reported.emit(result)


#============================================
class _MoleculeReportDeliveryRelay(PySide6.QtCore.QObject):
	"""Return exact worker identity with every queued terminal event."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Keep the owning window alive through its queued worker callbacks."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_reported(self, result: object) -> None:
		"""Forward a public JSON envelope to the owning window."""
		self._owner._on_document_molecule_reported(self.sender(), result)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward one terminal public-protocol failure."""
		self._owner._on_document_molecule_report_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release only the stopped worker that owns this signal."""
		self._owner._on_document_molecule_report_finished(self.sender())


#============================================
def _record_text(record: dict) -> str:
	"""Format record facts supplied by Rust without deriving new chemistry."""
	name = record["authored_name"]
	label = "(unnamed)" if name is None else name
	elements = ", ".join(
		"{0}: {1}".format(entry["symbol"], entry["atom_count"])
		for entry in record["authored_elements"]
	)
	charge = record["authored_charge"]
	charge_text = "not completely authored" if charge is None else "{0:+d}".format(charge)
	lines = [
		"Name: {0}".format(label),
		"Source ID: {0}".format(record["source_id"]),
		"Authored graph: {0} atoms, {1} bonds".format(
			record["atom_count"], record["bond_count"],
		),
		"Authored elements: {0}".format(elements),
		"Complete authored formal charge: {0}".format(charge_text),
	]
	composition = record["composition"]
	if composition is None:
		lines.append("Composition: unavailable (see diagnostics)")
	else:
		lines.extend(_composition_lines(composition))
	lines.append("Neutral bond-capacity result: {0}".format(record["neutral_bond_capacity"]))
	text = "\n".join(lines)
	return text


#============================================
def _composition_lines(composition: dict) -> list[str]:
	"""Render one complete Rust composition DTO without recalculating its facts."""
	lines = [
		"Formula: {0}".format(composition["formula"]),
		"Net formal charge: {0:+d}".format(composition["net_formal_charge"]),
		"Average molecular weight: {0:.6f} Da".format(
			composition["average_molecular_weight_da"],
		),
		"Monoisotopic mass: {0:.6f} Da".format(
			composition["monoisotopic_mass_da"],
		),
		"Isotope-aware element contributions:",
	]
	for element in composition["elements"]:
		isotope = element["isotope"]
		isotope_label = element["symbol"] if isotope is None else "{0}{1}".format(
			isotope, element["symbol"],
		)
		lines.append("  {0}: {1} atoms; {2:.6f} Da ({3:.4f}%)".format(
			isotope_label,
			element["atom_count"],
			element["average_mass_contribution_da"],
			element["mass_percentage"],
		))
	return lines


#============================================
def _aggregate_text(aggregate: dict) -> str:
	"""Render the tagged Rust aggregate outcome without interpreting its chemistry."""
	kind = aggregate["kind"]
	if kind == "complete":
		lines = ["Aggregate composition: complete"]
		lines.extend(_composition_lines(aggregate["composition"]))
		text = "\n".join(lines)
		return text
	if kind == "omitted":
		lines = [
			"Aggregate composition: omitted",
			"Reason: {0}".format(aggregate["reason"]),
			"Recovery: {0}".format(aggregate["recovery"]),
		]
		text = "\n".join(lines)
		return text
	raise ValueError("unknown Rust molecule-report aggregate outcome: {0}".format(kind))


#============================================
def _finding_location_text(location: dict) -> str:
	"""Render one authenticated diagnostic location without locating scene items."""
	kind = location["kind"]
	if kind == "root":
		text = "root"
	elif kind == "unaddressable":
		text = "unaddressable {0}".format(location["subject"])
	else:
		text = "{0}: {1}".format(kind, location["identifier"])
	return text


#============================================
def _finding_text(finding: dict) -> str:
	"""Present one complete ordered Rust finding without deriving chemistry in Qt."""
	lines = [
		"Severity: {0}".format(finding["severity"]),
		"Code: {0}".format(finding["code"]),
		"Location: {0}".format(_finding_location_text(finding["location"])),
		"Recovery: {0}".format(finding["recovery"]),
	]
	if finding["detail"] is not None:
		lines.append("Detail: {0}".format(finding["detail"]))
	text = "\n".join(lines)
	return text


#============================================
def _finding_summary_text(finding: dict) -> str:
	"""Provide a compact tree label for one already-validated Rust finding."""
	text = "{0}: {1} - {2}; {3}".format(
		finding["severity"],
		finding["code"],
		_finding_location_text(finding["location"]),
		finding["recovery"],
	)
	return text


#============================================
def _finite_number(value: object) -> bool:
	"""Require JSON numeric facts that the Qt formatter can present safely."""
	return type(value) in {int, float} and type(value) is not bool and math.isfinite(value)


#============================================
def _valid_element_count(element: object) -> bool:
	"""Validate one authored element count before it reaches a report formatter."""
	return (
		type(element) is dict and set(element) == {"symbol", "atom_count"}
		and type(element.get("symbol")) is str and bool(element["symbol"])
		and type(element.get("atom_count")) is int and element["atom_count"] >= 0
	)


#============================================
def _valid_composition_element(element: object) -> bool:
	"""Validate one isotope-aware complete-composition contribution."""
	return (
		type(element) is dict
		and {"symbol", "atom_count", "average_mass_contribution_da", "mass_percentage"} <= set(element)
		and set(element) <= {
			"symbol", "isotope", "atom_count", "average_mass_contribution_da", "mass_percentage",
		}
		and type(element.get("symbol")) is str and bool(element["symbol"])
		and (element.get("isotope") is None or (
			type(element["isotope"]) is int and 0 < element["isotope"] <= 65535
		))
		and type(element.get("atom_count")) is int and element["atom_count"] >= 0
		and _finite_number(element.get("average_mass_contribution_da"))
		and element["average_mass_contribution_da"] >= 0.0
		and _finite_number(element.get("mass_percentage"))
		and 0.0 <= element["mass_percentage"] <= 100.0
	)


#============================================
def _valid_composition(composition: object) -> bool:
	"""Validate one complete composition and its directly consumed numeric facts."""
	if type(composition) is not dict or set(composition) != {
		"formula", "net_formal_charge", "average_molecular_weight_da",
		"monoisotopic_mass_da", "elements",
	}:
		return False
	return (
		type(composition["formula"]) is str and bool(composition["formula"])
		and type(composition["net_formal_charge"]) is int
		and _finite_number(composition["average_molecular_weight_da"])
		and composition["average_molecular_weight_da"] >= 0.0
		and _finite_number(composition["monoisotopic_mass_da"])
		and composition["monoisotopic_mass_da"] >= 0.0
		and type(composition["elements"]) is list
		and all(_valid_composition_element(element) for element in composition["elements"])
	)


#============================================
def _valid_finding_location(location: object) -> bool:
	"""Recognize only the closed location grammar supplied by the public receipt."""
	if type(location) is not dict:
		return False
	kind = location.get("kind")
	if kind == "root":
		return set(location) == {"kind"}
	if kind in _FINDING_SUBJECTS:
		return set(location) == {"kind", "identifier"} and type(location.get("identifier")) is str
	if kind == "unaddressable":
		return set(location) == {"kind", "subject"} and location.get("subject") in _FINDING_SUBJECTS
	return False


#============================================
def _valid_finding(finding: object) -> bool:
	"""Recognize one complete canonical diagnostic finding at the protocol boundary."""
	if (
		type(finding) is not dict
		or not {"severity", "code", "recovery", "location"} <= set(finding)
		or not set(finding) <= {"severity", "code", "recovery", "location", "detail"}
	):
		return False
	return (
		finding.get("severity") in _FINDING_SEVERITIES
		and type(finding.get("code")) is str
		and finding.get("recovery") in _FINDING_RECOVERIES
		and _valid_finding_location(finding.get("location"))
		and (finding.get("detail") is None or (
			type(finding.get("detail")) is str and len(finding["detail"]) <= 4096
		))
	)


#============================================
def _valid_aggregate(aggregate: object) -> bool:
	"""Recognize the tagged aggregate outcome without replacing a Rust decision."""
	if type(aggregate) is not dict:
		return False
	if aggregate.get("kind") == "complete":
		return set(aggregate) == {"kind", "composition"} and _valid_composition(
			aggregate.get("composition"),
		)
	if aggregate.get("kind") == "omitted":
		return (
			set(aggregate) == {"kind", "reason", "recovery"}
			and aggregate.get("reason") in _AGGREGATE_OMISSION_REASONS
			and aggregate.get("recovery") in _FINDING_RECOVERIES
		)
	return False


#============================================
def _valid_record(record: object, address: object) -> bool:
	"""Validate one selected direct-root record against its captured public address."""
	if (
		type(record) is not dict
		or not {
			"molecule_id", "source_id", "document_root_order", "atom_count", "bond_count",
			"authored_elements", "neutral_bond_capacity", "findings",
		} <= set(record)
		or not set(record) <= {
			"molecule_id", "source_id", "document_root_order", "authored_name", "atom_count",
			"bond_count", "authored_charge", "authored_elements", "composition",
			"neutral_bond_capacity", "findings",
		}
	):
		return False
	return (
		record["molecule_id"] == address.molecule_id
		and record["source_id"] == address.source_id
		and record["document_root_order"] == address.document_root_order
		and type(record["molecule_id"]) is str
		and type(record["source_id"]) is str
		and type(record["document_root_order"]) is int and record["document_root_order"] >= 0
		and (record["authored_name"] is None or type(record["authored_name"]) is str)
		and type(record["atom_count"]) is int and record["atom_count"] >= 0
		and type(record["bond_count"]) is int and record["bond_count"] >= 0
		and (record["authored_charge"] is None or type(record["authored_charge"]) is int)
		and type(record["authored_elements"]) is list
		and all(_valid_element_count(element) for element in record["authored_elements"])
		and (record["composition"] is None or _valid_composition(record["composition"]))
		and record["neutral_bond_capacity"] in _NEUTRAL_BOND_CAPACITY
		and type(record["findings"]) is list
		and all(_valid_finding(finding) for finding in record["findings"])
	)


#============================================
def _presentation_composition(composition: dict) -> dict:
	"""Supply explicit Qt defaults for schema-optional composition fields."""
	return {
		**composition,
		"elements": [{"isotope": None, **element} for element in composition["elements"]],
	}


#============================================
def _presentation_record(record: dict) -> dict:
	"""Supply explicit Qt defaults for schema-optional record fields."""
	composition = record.get("composition")
	return {
		"authored_name": None,
		"authored_charge": None,
		**record,
		"composition": None if composition is None else _presentation_composition(composition),
		"findings": [{"detail": None, **finding} for finding in record["findings"]],
	}


#============================================
def _presentation_report(report: dict) -> dict:
	"""Supply only schema-defined presentation defaults to an authenticated receipt."""
	aggregate = report["aggregate"]
	if aggregate["kind"] == "complete":
		aggregate = {**aggregate, "composition": _presentation_composition(aggregate["composition"])}
	return {
		**report,
		"records": [_presentation_record(record) for record in report["records"]],
		"aggregate": aggregate,
	}


#============================================
def decode_molecule_report_refusal(response: object) -> FerrumNativeMoleculeReportFailure | None:
	"""Classify the report operation's typed error envelope by stable facts only."""
	if type(response) is not dict or response.get("schema") != "ferrum-operation-error-v1":
		return None
	error = response.get("error")
	if type(error) is not dict or type(error.get("category")) is not str:
		return None
	if error.get("operation") not in {None, "document.molecule.report.v1"}:
		return None
	category = error["category"]
	resource = error.get("resource_limit")
	if category == "resource_limit" and type(resource) is dict and resource == {
		"reason": "response_size_exceeded", "recovery": "reduce_requested_result",
	}:
		return FerrumNativeMoleculeReportFailure(
			"Molecule Report is too large. Reduce the selected molecules and run it again.",
			category, "reduce_requested_result",
		)
	if category == "chemistry_unavailable":
		return FerrumNativeMoleculeReportFailure(
			"Chemistry is temporarily unavailable. Retry Molecule Report when Ferrum chemistry is ready.",
			category, "retry_with_chemistry_runtime",
		)
	if category in {"stale_document", "document_admission_failed", "document_invalid"}:
		return FerrumNativeMoleculeReportFailure(
			"Ferrum could not use this document snapshot. Reload the document, then run Molecule Report again.",
			category, "reload_and_rerun",
		)
	return FerrumNativeMoleculeReportFailure(
		"Ferrum could not prepare Molecule Report. Run it again from the current document.",
		category, "rerun",
	)


#============================================
class FerrumNativeMoleculeReportDialog(FerrumAccessibleDialog):
	"""Modeless structured presentation of one immutable JSON report envelope."""

	rerun_requested = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, report: dict, tab: object, parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build the resizable report view entirely from public receipt fields."""
		super().__init__(parent)
		self._report = report
		self._tab = tab
		self._stale = False
		self._retired = False
		self.setWindowTitle(self.tr("Molecule Report"))
		self.setObjectName("molecule-report-dialog")
		self.setAccessibleName(self.tr("Molecule Report"))
		self.setAccessibleDescription(self.tr(
			"Read-only Ferrum Rust facts and supported diagnostics for selected molecules.",
		))
		self.setWindowFlag(PySide6.QtCore.Qt.WindowType.Tool, True)
		self.setModal(False)
		self.setMinimumSize(680, 480)
		self.resize(860, 560)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		self._heading = PySide6.QtWidgets.QLabel(self.tr(
			"Molecule Report - {0} selected molecules".format(len(report["records"])),
		), self)
		self._heading.setObjectName("molecule-report-heading")
		self._heading.setAccessibleName(self.tr("Molecule Report summary"))
		self._heading.setStyleSheet("font-weight: bold;")
		layout.addWidget(self._heading)
		self._stamp = PySide6.QtWidgets.QLabel(self.tr(
			"Current document revision {0}".format(report["source_revision"]),
		), self)
		self._stamp.setObjectName("molecule-report-snapshot")
		self._stamp.setAccessibleName(self.tr("Report snapshot"))
		layout.addWidget(self._stamp)
		self._warning = PySide6.QtWidgets.QLabel(self.tr(
			"This report is from an earlier document revision. Run again for current facts.",
		), self)
		self._warning.setObjectName("molecule-report-stale-warning")
		self._warning.setAccessibleName(self.tr("Stale report warning"))
		self._warning.setWordWrap(True)
		self._warning.hide()
		layout.addWidget(self._warning)
		splitter = PySide6.QtWidgets.QSplitter(PySide6.QtCore.Qt.Orientation.Horizontal, self)
		self._tree = PySide6.QtWidgets.QTreeView(splitter)
		self._tree.setObjectName("molecule-report-tree")
		self._tree.setAccessibleName(self.tr("Molecule report facts and findings"))
		self._tree.setAccessibleDescription(self.tr(
			"Ordered Rust-issued molecule facts and diagnostic findings.",
		))
		self._model = PySide6.QtGui.QStandardItemModel(self)
		self._model.setHorizontalHeaderLabels((self.tr("Report facts"),))
		self._tree.setModel(self._model)
		self._details = PySide6.QtWidgets.QPlainTextEdit(splitter)
		self._details.setObjectName("molecule-report-details")
		self._details.setReadOnly(True)
		self._details.setFont(PySide6.QtGui.QFontDatabase.systemFont(
			PySide6.QtGui.QFontDatabase.SystemFont.FixedFont,
		))
		self._details.setAccessibleName(self.tr("Selected molecule report details"))
		self._details.setAccessibleDescription(self.tr(
			"Selectable report details. Ferrum evaluates only supported neutral bond capacity; "
			"this is not a general validity or oxidation-state check.",
		))
		splitter.setStretchFactor(0, 2)
		splitter.setStretchFactor(1, 3)
		layout.addWidget(splitter, 1)
		self._reveal = PySide6.QtWidgets.QPushButton(self.tr("Show on canvas"), self)
		self._reveal.setObjectName("molecule-report-show-canvas")
		self._reveal.setAccessibleName(self.tr("Show affected atom on canvas"))
		self._reveal.setAccessibleDescription(self.tr(
			"Unavailable: the current public molecule-report receipt has no atom address or bounds.",
		))
		self._reveal.setEnabled(False)
		self._copy = PySide6.QtWidgets.QPushButton(self.tr("Copy report"), self)
		self._copy.setObjectName("molecule-report-copy")
		self._copy.setAccessibleName(self.tr("Copy report"))
		self._rerun = PySide6.QtWidgets.QPushButton(self.tr("Run again"), self)
		self._rerun.setObjectName("molecule-report-run-again")
		self._rerun.setAccessibleName(self.tr("Run Molecule Report again"))
		self._close = PySide6.QtWidgets.QPushButton(self.tr("Close"), self)
		self._close.setObjectName("molecule-report-close")
		self._close.setAccessibleName(self.tr("Close Molecule Report"))
		buttons = PySide6.QtWidgets.QHBoxLayout()
		buttons.addWidget(self._reveal)
		buttons.addStretch(1)
		buttons.addWidget(self._copy)
		buttons.addWidget(self._rerun)
		buttons.addWidget(self._close)
		layout.addLayout(buttons)
		self._copy.clicked.connect(self._copy_report)
		self._rerun.clicked.connect(self._request_rerun)
		self._close.clicked.connect(self.close)
		self._tree.selectionModel().currentChanged.connect(self._show_current_details)
		self._populate(report)
		PySide6.QtWidgets.QWidget.setTabOrder(self._tree, self._details)
		PySide6.QtWidgets.QWidget.setTabOrder(self._details, self._reveal)
		PySide6.QtWidgets.QWidget.setTabOrder(self._reveal, self._copy)
		PySide6.QtWidgets.QWidget.setTabOrder(self._copy, self._rerun)
		PySide6.QtWidgets.QWidget.setTabOrder(self._rerun, self._close)

	#============================================
	def _populate(self, report: dict) -> None:
		"""Build the tree in preserved Rust direct-root order."""
		for record in report["records"]:
			name = record["authored_name"]
			label = record["source_id"] if name is None else name
			root = PySide6.QtGui.QStandardItem(self.tr("Molecule: {0}".format(label)))
			root.setData(_record_text(record), PySide6.QtCore.Qt.ItemDataRole.UserRole)
			self._model.appendRow(root)
			facts = PySide6.QtGui.QStandardItem(self.tr("Facts"))
			facts.setData(_record_text(record), PySide6.QtCore.Qt.ItemDataRole.UserRole)
			root.appendRow(facts)
			diagnostics = PySide6.QtGui.QStandardItem(self.tr("Diagnostics"))
			root.appendRow(diagnostics)
			# Validated findings retain the canonical Rust report order.
			for finding_summary in record["findings"]:
				finding_text = _finding_text(finding_summary)
				finding = PySide6.QtGui.QStandardItem(finding_text)
				finding.setData(finding_text, PySide6.QtCore.Qt.ItemDataRole.UserRole)
				diagnostics.appendRow(finding)
			if diagnostics.rowCount() == 0:
				clear = PySide6.QtGui.QStandardItem(self.tr("OK - no supported diagnostics"))
				clear.setData(clear.text(), PySide6.QtCore.Qt.ItemDataRole.UserRole)
				diagnostics.appendRow(clear)
			self._tree.expand(root.index())
		aggregate = PySide6.QtGui.QStandardItem(self.tr("Aggregate composition"))
		aggregate.setData(_aggregate_text(report["aggregate"]), PySide6.QtCore.Qt.ItemDataRole.UserRole)
		self._model.appendRow(aggregate)
		if self._model.rowCount() > 0:
			first = self._model.index(0, 0)
			self._tree.setCurrentIndex(first)
			self._show_current_details(first)

	#============================================
	def _show_current_details(self, index: PySide6.QtCore.QModelIndex, *unused: object) -> None:
		"""Show only the selected row's receipt-provided complete display text."""
		del unused
		text = index.data(PySide6.QtCore.Qt.ItemDataRole.UserRole)
		if type(text) is str:
			self._details.setPlainText(text)

	#============================================
	def _copy_report(self) -> None:
		"""Copy explicitly rendered report text without changing document state."""
		lines = []
		for row in range(self._model.rowCount()):
			root = self._model.item(row)
			lines.append(root.text())
			lines.append(root.data(PySide6.QtCore.Qt.ItemDataRole.UserRole))
			for child in range(root.rowCount()):
				item = root.child(child)
				if item.text() == "Diagnostics":
					for finding_index in range(item.rowCount()):
						lines.append("Diagnostic: {0}".format(item.child(finding_index).text()))
		clipboard = PySide6.QtWidgets.QApplication.clipboard()
		clipboard.setText("\n".join(lines))

	#============================================
	def mark_stale(self) -> None:
		"""Retain history as display while preventing current-snapshot interpretation."""
		if self._stale:
			return
		self._stale = True
		self._warning.show()
		self._reveal.setEnabled(False)

	#============================================
	def set_rerun_availability(self, available: bool, explanation: str) -> None:
		"""Expose whether the captured source remains the active live rerun target."""
		self._rerun.setEnabled(available and not self._retired)
		self._rerun.setToolTip(explanation)
		self._rerun.setAccessibleDescription(explanation)

	#============================================
	def retire_for_closed_source(self) -> None:
		"""Terminally retire a receipt before its captured source tab is disposed."""
		if self._retired:
			return
		self._retired = True
		self.set_rerun_availability(False, self.tr(
			"Unavailable: this report's source document was closed.",
		))
		self.close()

	#============================================
	def _request_rerun(self) -> None:
		"""Emit only this dialog as the explicit source-bound recapture request."""
		if not self._retired:
			self.rerun_requested.emit(self)

	#============================================
	@property
	def rerun_button(self) -> PySide6.QtWidgets.QPushButton:
		"""Expose the explicit recapture action to its owning report controller."""
		return self._rerun


#============================================
class FerrumNativeMoleculeReportMixin:
	"""Own one cancellable public report operation and modeless receipt dialog."""

	#============================================
	def _initialize_molecule_inspection(self) -> None:
		"""Retain existing window initialization seam under its replacement controller."""
		self._molecule_report_intent: _MoleculeReportIntent | None = None
		self._molecule_report_dialog: FerrumNativeMoleculeReportDialog | None = None
		self._molecule_report_relay = _MoleculeReportDeliveryRelay(self)

	#============================================
	def _build_molecule_inspection_actions(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Install the one task-oriented report route after chemistry authoring actions."""
		menu.addSeparator()
		self._molecule_report_action = PySide6.QtGui.QAction(self.tr("Molecule Report..."), self)
		self._molecule_report_action.setObjectName("molecule-report-action")
		self._molecule_report_action.setStatusTip(self.tr(
			"Show Ferrum Rust facts and supported diagnostics for the selected molecules. "
			"Does not change the document.",
		))
		self._molecule_report_action.setToolTip(self.tr(
			"Select an atom or bond belonging to a complete molecule.",
		))
		self._molecule_report_action.setWhatsThis(self.tr(
			"Select an atom or bond belonging to a complete molecule.",
		))
		self._molecule_report_action.triggered.connect(self._start_molecule_report)
		menu.addAction(self._molecule_report_action)
		self._cancel_molecule_report_action = PySide6.QtGui.QAction(
			self.tr("Cancel Molecule Report"), self,
		)
		self._cancel_molecule_report_action.setObjectName("cancel-molecule-report-action")
		self._cancel_molecule_report_action.triggered.connect(self._cancel_molecule_report)
		menu.addAction(self._cancel_molecule_report_action)

	#============================================
	def _molecule_inspection_busy(self) -> bool:
		"""Provide the existing window-wide busy seam for the replacement report."""
		return self._molecule_report_intent is not None

	#============================================
	def _selected_molecule_information_addresses(self) -> tuple[object, ...] | None:
		"""Reuse only the complete selected direct-root address resolver."""
		tab = self._active_native_tab()
		addresses = None if tab is None else (
			ferrum_qt.ferrum.molecule_inspection.selected_durable_molecule_addresses(tab)
		)
		return addresses

	#============================================
	def _start_molecule_report(self) -> bool:
		"""Start one report from the ordinary active-document action route."""
		tab = self._active_native_tab()
		return self._start_molecule_report_for_tab(tab)

	#============================================
	def _start_molecule_report_for_tab(self, tab: object | None) -> bool:
		"""Freeze one source-bound public request without reading XML or mutating state."""
		if (
			self._molecule_inspection_busy()
			or self._molecule_import_busy()
			or self._molecule_export_busy()
			or self._coordinate_generation_intent is not None
		):
			return False
		if tab is None or self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed:
			return False
		addresses = ferrum_qt.ferrum.molecule_inspection.selected_durable_molecule_addresses(tab)
		if addresses is None:
			return False
		try:
			snapshot = tab.current_snapshot
			request_json = _report_request(snapshot, addresses)
			execute_operation = engine.extension_module().execute_operation_v1
			worker = FerrumNativeMoleculeReportWorker(execute_operation, request_json)
		except Exception as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return False
		self._molecule_report_intent = _MoleculeReportIntent(
			tab, snapshot.revision, snapshot.digest, addresses, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.reported.connect(self._molecule_report_relay.on_reported, connection)
		worker.failed.connect(self._molecule_report_relay.on_failed, connection)
		worker.finished.connect(self._molecule_report_relay.on_finished, connection)
		self.statusBar().showMessage(self.tr("Preparing Molecule Report with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _current_molecule_report_intent(self, worker: object) -> _MoleculeReportIntent | None:
		"""Return only the exact live worker whose captured source remains current."""
		intent = self._molecule_report_intent
		if intent is None or worker is not intent.worker or worker.delivery_cancelled:
			return None
		if intent.tab not in self._native_tabs_by_page or intent.tab.requires_refresh:
			return None
		snapshot = intent.tab.current_snapshot
		if snapshot.revision != intent.revision or snapshot.digest != intent.digest:
			return None
		return intent

	#============================================
	def _report_from_current_intent(self, intent: _MoleculeReportIntent, response: object) -> dict | None:
		"""Authenticate the public envelope to every captured direct-root address."""
		if (
			type(response) is not dict
			or set(response) != {"schema", "request_id", "outcome"}
			or response.get("schema") != "ferrum-operation-response-v1"
			or response.get("request_id") != "qt-molecule-report"
		):
			return None
		outcome = response.get("outcome")
		if type(outcome) is not dict or set(outcome) != {"kind", "report"} or (
			outcome.get("kind") != "document.molecule.report.v1"
		):
			return None
		report = outcome.get("report")
		if (
			type(report) is not dict
			or set(report) != {"schema", "source_revision", "source_digest_hex", "records", "aggregate"}
			or report.get("schema") != "ferrum-document-molecule-report-v1"
			or type(report.get("source_revision")) is not int
			or report["source_revision"] != intent.revision
			or type(report.get("source_digest_hex")) is not str
			or report["source_digest_hex"] != intent.digest
			or type(report.get("records")) is not list
			or len(report["records"]) != len(intent.addresses)
			or not _valid_aggregate(report.get("aggregate"))
		):
			return None
		for record, address in zip(report["records"], intent.addresses, strict=True):
			if not _valid_record(record, address):
				return None
		return _presentation_report(report)

	#============================================
	def _on_document_molecule_reported(self, worker: object, response: object) -> None:
		"""Open a modeless dialog only for a current authenticated report receipt."""
		intent = self._current_molecule_report_intent(worker)
		if intent is None:
			return
		refusal = decode_molecule_report_refusal(response)
		if refusal is not None:
			self._show_edit_refusal(self._unavailable_edit_refusal(refusal.message))
			return
		report = self._report_from_current_intent(intent, response)
		if report is None:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum returned an invalid Molecule Report receipt. Run it again from the current document.",
			))
			return
		self._show_molecule_report_dialog(report, intent.tab)

	#============================================
	def _show_molecule_report_dialog(self, report: dict, tab: object) -> None:
		"""Replace the previous report with one parent-owned snapshot dialog."""
		if self._molecule_report_dialog is not None:
			self._molecule_report_dialog.close()
		dialog = FerrumNativeMoleculeReportDialog(report, tab, self)
		dialog.rerun_requested.connect(self._rerun_molecule_report_from_dialog)
		dialog.finished.connect(self._on_molecule_report_dialog_finished)
		self._molecule_report_dialog = dialog
		dialog.show()
		dialog._tree.setFocus()

	#============================================
	def _rerun_molecule_report_from_dialog(self, dialog: object) -> bool:
		"""Recapture only the report dialog's active live source, never another tab."""
		if dialog is not self._molecule_report_dialog:
			return False
		tab = dialog._tab
		if not self._molecule_report_dialog_source_is_active(dialog):
			dialog.set_rerun_availability(False, self.tr(
				"Select this report's source document before running it again.",
			))
			return False
		return self._start_molecule_report_for_tab(tab)

	#============================================
	def _on_molecule_report_dialog_finished(self, *unused: object) -> None:
		"""Release only the report dialog after its ordinary close lifecycle."""
		del unused
		dialog = self.sender()
		if dialog is self._molecule_report_dialog:
			self._molecule_report_dialog = None

	#============================================
	def _on_document_molecule_report_failed(self, worker: object, failure: object) -> None:
		"""Show a terminal protocol failure without fabricating a report fallback."""
		if self._current_molecule_report_intent(worker) is not None:
			self._show_edit_refusal(self._unavailable_edit_refusal(failure.message))

	#============================================
	def _on_document_molecule_report_finished(self, worker: object) -> None:
		"""Release a retired report worker and restore ordinary action reachability."""
		intent = self._molecule_report_intent
		if intent is None or worker is not intent.worker:
			return
		self._molecule_report_intent = None
		worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _cancel_molecule_report(self) -> None:
		"""Suppress late delivery while a detached Rust call retires normally."""
		intent = self._molecule_report_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(self.tr("Cancelling Molecule Report delivery..."), 0)
		self._refresh_actions()

	#============================================
	def _refresh_molecule_inspection_actions(
			self, active: bool, pending: bool, busy_elsewhere: bool) -> None:
		"""Expose report prerequisites and mark an open dialog stale when required."""
		addresses = self._selected_molecule_information_addresses()
		self._molecule_report_action.setEnabled(
			active and not pending and not busy_elsewhere and not self._molecule_inspection_busy()
			and addresses is not None,
		)
		self._cancel_molecule_report_action.setEnabled(
			self._molecule_report_intent is not None
			and not self._molecule_report_intent.worker.delivery_cancelled,
		)
		dialog = self._molecule_report_dialog
		if dialog is not None:
			if dialog._tab not in self._native_tabs_by_page:
				dialog.retire_for_closed_source()
				return
			snapshot = dialog._tab.current_snapshot
			if (
				snapshot.revision != dialog._report["source_revision"]
				or snapshot.digest != dialog._report["source_digest_hex"]
			):
				dialog.mark_stale()
			if self._molecule_report_dialog_source_is_active(dialog):
				addresses = ferrum_qt.ferrum.molecule_inspection.selected_durable_molecule_addresses(
					dialog._tab,
				)
				if addresses is not None:
					dialog.set_rerun_availability(True, self.tr(
						"Run Molecule Report again for this document's current selection.",
					))
				else:
					dialog.set_rerun_availability(False, self.tr(
						"Select a complete molecule in this document before running again.",
					))
			else:
				dialog.set_rerun_availability(False, self.tr(
					"Select this report's source document before running it again.",
				))

	#============================================
	def _molecule_report_dialog_source_is_active(
			self, dialog: FerrumNativeMoleculeReportDialog) -> bool:
		"""Return whether a report dialog retains its exact active, non-disposed source."""
		tab = dialog._tab
		return (
			self._native_tabs_by_page.get(tab) is tab
			and not tab.is_disposed
			and self._active_native_tab() is tab
		)

	#============================================
	def _retire_molecule_report_dialog_for_tab(self, tab: object) -> None:
		"""Remove a modeless report before its source tab loses its live identity."""
		dialog = self._molecule_report_dialog
		if dialog is not None and dialog._tab is tab:
			dialog.retire_for_closed_source()

	#============================================
	def _molecule_inspection_blocks_tab_close(self, tab: object) -> bool:
		"""Withdraw delivery before a detached report source tab is disposed."""
		intent = self._molecule_report_intent
		if intent is None or intent.tab is not tab:
			return False
		intent.worker.cancel_delivery()
		return False

	#============================================
	def _cancel_molecule_inspection_for_close(self) -> bool:
		"""Cancel delivery before a later user-authorized close attempt."""
		if self._molecule_report_intent is None:
			return False
		self._cancel_molecule_report()
		self._show_edit_refusal(self._unavailable_edit_refusal(
			"Ferrum cancelled delivery; close again after the current operation finishes.",
		))
		return True

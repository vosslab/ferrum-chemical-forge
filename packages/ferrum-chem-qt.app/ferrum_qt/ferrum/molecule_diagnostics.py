"""Read-only Qt presentation for one fenced Ferrum structure check."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
from ferrum_qt.dialogs.accessibility import FerrumAccessibleDialog
from ferrum_qt.ferrum.background_job import FerrumDetachedJobThread

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.ferrum.molecule_inspection


_SCHEMA = "ferrum-document-molecule-diagnostics-v1"
_SEVERITIES = {"info", "warning", "error"}
_RECOVERIES = {
	"none", "inspect_structure", "correct_chemical_facts",
	"choose_supported_representation", "materialize_compact_group", "reduce_selection",
	"retry_with_chemistry_runtime",
}


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _Finding:
	"""One Rust finding copied into Qt-safe immutable display facts."""

	severity: str
	code: str
	recovery: str
	location: str
	detail: str | None


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _Receipt:
	"""One immutable source-fenced Rust receipt for the selected root."""

	revision: int
	digest: str
	molecule_id: str
	findings: tuple[_Finding, ...]


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _Intent:
	"""Exact tab, fence, direct-root selection, and detached delivery owner."""

	tab: object
	revision: int
	digest: str
	molecule_id: str
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeMoleculeDiagnosticsFailure:
	"""One typed terminal detached failure for standard refusal presentation."""

	message: str


#============================================
def _text(value: object, field: str) -> str:
	"""Require one nonempty Rust-provided display field."""
	if type(value) is not str or not value:
		raise TypeError(f"Ferrum Check Structure returned invalid {field}")
	return value


#============================================
def _location_text(location: object) -> str:
	"""Present Rust's closed location category without leaking source identifiers."""
	kind = _text(getattr(location, "kind", None), "finding location")
	subject = getattr(location, "subject", None)
	if kind == "root" and subject is None:
		return "Molecule root"
	if kind in {"atom", "vertex", "bond"} and subject is None:
		return f"{kind.title()} finding"
	if kind == "unaddressable" and subject in {"atom", "vertex", "bond"}:
		return f"Unaddressable {subject}"
	raise TypeError("Ferrum Check Structure returned invalid finding location")


#============================================
def _receipt_from_native(value: object) -> _Receipt:
	"""Copy only closed private-binding receipt facts for queued Qt delivery."""
	if getattr(value, "schema", None) != _SCHEMA:
		raise TypeError("Ferrum Check Structure returned an unknown receipt schema")
	revision = getattr(value, "source_revision", None)
	digest = _text(getattr(value, "source_digest", None), "source digest")
	records = getattr(value, "records", None)
	if type(revision) is not int or revision < 0 or type(records) is not tuple or len(records) != 1:
		raise TypeError("Ferrum Check Structure returned an invalid selected-root receipt")
	record = records[0]
	molecule_id = _text(getattr(record, "molecule_id", None), "molecule ID")
	if type(getattr(record, "document_paint_order", None)) is not int:
		raise TypeError("Ferrum Check Structure returned an invalid document paint order")
	findings = getattr(record, "findings", None)
	if type(findings) is not tuple:
		raise TypeError("Ferrum Check Structure returned invalid findings")
	result = []
	for finding in findings:
		severity = _text(getattr(finding, "severity", None), "finding severity")
		recovery = _text(getattr(finding, "recovery", None), "finding recovery")
		detail = getattr(finding, "detail", None)
		if severity not in _SEVERITIES or recovery not in _RECOVERIES:
			raise TypeError("Ferrum Check Structure returned an unknown finding vocabulary")
		if detail is not None and type(detail) is not str:
			raise TypeError("Ferrum Check Structure returned invalid finding detail")
		result.append(_Finding(
			severity, _text(getattr(finding, "code", None), "finding code"),
			recovery, _location_text(getattr(finding, "location", None)), detail,
		))
	return _Receipt(revision, digest, molecule_id, tuple(result))


#============================================
def _execute_diagnostics_from_snapshot(cdml: str, source_revision: int,
		source_digest: str, molecule_ids: tuple[str, ...]) -> _Receipt:
	"""Run Rust's thread-safe owned-snapshot executor without a document session."""
	return _receipt_from_native(engine._document_molecule_diagnostics_from_snapshot_v1(
		cdml, source_revision, source_digest, molecule_ids,
	))


#============================================
class FerrumNativeMoleculeDiagnosticsWorker(FerrumDetachedJobThread):
	"""Run one immutable read-only diagnostics request off the Qt event thread."""

	diagnosed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, cdml: str, revision: int, digest: str,
			molecule_ids: tuple[str, ...]) -> None:
		"""Capture only owned snapshot text, source fence, and direct-root IDs."""
		if type(cdml) is not str or not cdml:
			raise TypeError("Ferrum Check Structure needs an owned CDML snapshot")
		if type(revision) is not int or revision < 0 or type(digest) is not str or not digest:
			raise TypeError("Ferrum Check Structure needs an exact source fence")
		if type(molecule_ids) is not tuple or len(molecule_ids) != 1:
			raise TypeError("Ferrum Check Structure needs exactly one molecule root")
		super().__init__(
			lambda: _execute_diagnostics_from_snapshot(cdml, revision, digest, molecule_ids),
			lambda error: FerrumNativeMoleculeDiagnosticsFailure(str(error)),
		)

	#============================================
	def _emit_success(self, result: object) -> None:
		"""Deliver the copied receipt through the feature-specific signal."""
		self.diagnosed.emit(result)


#============================================
class _DeliveryRelay(PySide6.QtCore.QObject):
	"""Preserve exact worker identity for each queued terminal delivery."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Keep the receiver alive through its queued worker callbacks."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_diagnosed(self, receipt: object) -> None:
		"""Forward one receipt to the owning Qt window."""
		self._owner._on_document_molecule_diagnosed(self.sender(), receipt)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward one typed terminal failure to the owning Qt window."""
		self._owner._on_document_molecule_diagnostics_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release only the worker that emitted this terminal lifecycle signal."""
		self._owner._on_document_molecule_diagnostics_finished(self.sender())


#============================================
class FerrumNativeMoleculeDiagnosticsDialog(FerrumAccessibleDialog):
	"""Modeless accessible display of one read-only Rust structure receipt."""

	rerun_requested = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, receipt: _Receipt, tab: object,
			parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build a focused finding list and details surface from Rust facts only."""
		super().__init__(parent)
		self._receipt = receipt
		self._tab = tab
		self._source_closed = False
		self.setWindowTitle(self.tr("Check Structure"))
		self.setObjectName("check-structure-dialog")
		self.setAccessibleName(self.tr("Check Structure"))
		self.setAccessibleDescription(self.tr(
			"Read-only Ferrum Rust structure findings. This dialog does not change the document.",
		))
		self.setWindowFlag(PySide6.QtCore.Qt.WindowType.Tool, True)
		self.setModal(False)
		self.setMinimumSize(640, 420)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		heading = PySide6.QtWidgets.QLabel(self.tr("Check Structure"), self)
		heading.setAccessibleName(self.tr("Check Structure heading"))
		heading.setStyleSheet("font-weight: bold;")
		layout.addWidget(heading)
		statement = PySide6.QtWidgets.QLabel(self.tr(
			"Read-only: this check does not change the document.",
		), self)
		statement.setAccessibleName(self.tr("Read-only check statement"))
		statement.setWordWrap(True)
		layout.addWidget(statement)
		self._stale_message = self.tr(
			"This result is from an earlier document state. Run the check again for current findings.",
		)
		self._stale = PySide6.QtWidgets.QLabel("", self)
		self._stale.setAccessibleName(self.tr("Stale Check Structure warning"))
		self._stale.setWordWrap(True)
		self._stale.hide()
		layout.addWidget(self._stale)
		self._no_issues = PySide6.QtWidgets.QLabel(self.tr(
			"No issues found for the selected molecule.",
		), self)
		self._no_issues.setAccessibleName(self.tr("No structure issues found"))
		self._no_issues.setVisible(not receipt.findings)
		layout.addWidget(self._no_issues)
		splitter = PySide6.QtWidgets.QSplitter(PySide6.QtCore.Qt.Orientation.Horizontal, self)
		self._findings = PySide6.QtWidgets.QTreeWidget(splitter)
		self._findings.setAccessibleName(self.tr("Structure findings"))
		self._findings.setHeaderLabels((self.tr("Severity"), self.tr("Finding"), self.tr("Location")))
		self._details = PySide6.QtWidgets.QPlainTextEdit(splitter)
		self._details.setAccessibleName(self.tr("Selected finding details"))
		self._details.setReadOnly(True)
		for index, finding in enumerate(receipt.findings):
			item = PySide6.QtWidgets.QTreeWidgetItem((
				finding.severity.title(), finding.code, finding.location,
			))
			item.setData(0, PySide6.QtCore.Qt.ItemDataRole.UserRole, index)
			self._findings.addTopLevelItem(item)
		self._findings.currentItemChanged.connect(self._show_finding)
		if receipt.findings:
			self._findings.setCurrentItem(self._findings.topLevelItem(0))
		else:
			self._details.setPlainText(self.tr(
				"Ferrum found no supported structure issues for this selected molecule.",
			))
		layout.addWidget(splitter, 1)
		buttons = PySide6.QtWidgets.QDialogButtonBox(self)
		self._rerun = buttons.addButton(self.tr("Run Check Again"),
			PySide6.QtWidgets.QDialogButtonBox.ButtonRole.ActionRole)
		self._rerun.setAccessibleName(self.tr("Run Check Structure again"))
		self._rerun.clicked.connect(lambda: self.rerun_requested.emit(self))
		close = buttons.addButton(PySide6.QtWidgets.QDialogButtonBox.StandardButton.Close)
		close.setAccessibleName(self.tr("Close Check Structure"))
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)
		self.setProperty("ferrum_initial_focus_widget", self._findings)

	#============================================
	def _show_finding(self, item: object, _previous: object) -> None:
		"""Present selected closed finding facts in a dedicated focusable detail control."""
		if not isinstance(item, PySide6.QtWidgets.QTreeWidgetItem):
			return
		index = item.data(0, PySide6.QtCore.Qt.ItemDataRole.UserRole)
		if type(index) is int and 0 <= index < len(self._receipt.findings):
			finding = self._receipt.findings[index]
			lines = [
				f"Severity: {finding.severity}", f"Code: {finding.code}",
				f"Location: {finding.location}", f"Recovery: {finding.recovery}",
			]
			if finding.detail is not None:
				lines.append(f"Detail: {finding.detail}")
			self._details.setPlainText("\n".join(lines))

	#============================================
	def mark_stale(self) -> None:
		"""Retain historical facts while making their source state explicit."""
		self._stale.setText(self._stale_message)
		self._stale.show()

	#============================================
	def mark_current(self) -> None:
		"""Remove stale presentation when this receipt's exact intent is current again."""
		self._stale.setText("")
		self._stale.hide()

	#============================================
	def set_rerun_availability(self, available: bool, explanation: str) -> None:
		"""Expose whether this exact receipt intent can be recaptured now."""
		self._rerun.setEnabled(available and not self._source_closed)
		self._rerun.setToolTip(explanation)
		self._rerun.setAccessibleDescription(explanation)

	#============================================
	def close_for_closed_source(self) -> None:
		"""Withdraw a modeless receipt after its source tab is closed."""
		self._source_closed = True
		self._rerun.setEnabled(False)
		self.close()


#============================================
class FerrumNativeMoleculeDiagnosticsMixin:
	"""Own Check Structure action reachability, source fences, and delivery."""

	#============================================
	def _initialize_molecule_diagnostics(self) -> None:
		"""Create the dedicated read-only feature state owned by this window."""
		self._molecule_diagnostics_intent: _Intent | None = None
		self._molecule_diagnostics_dialog: FerrumNativeMoleculeDiagnosticsDialog | None = None
		self._molecule_diagnostics_relay = _DeliveryRelay(self)

	#============================================
	def _build_molecule_diagnostics_action(self) -> None:
		"""Create and register the explicit Chemistry action."""
		self._check_structure_action = PySide6.QtGui.QAction(self.tr("Check Structure..."), self)
		self._check_structure_action.setObjectName("check-structure-action")
		self._check_structure_action.setStatusTip(self.tr(
			"Check the selected molecule with Ferrum Rust. Does not change the document.",
		))
		self._check_structure_action.setToolTip(self.tr(
			"Select atoms or bonds belonging to exactly one complete molecule.",
		))
		self._check_structure_action.triggered.connect(self._start_molecule_diagnostics)
		self._action_registry.register_existing(
			"chemistry.diagnostics.structure", self._check_structure_action,
			shortcut_exemption_reason="Available by its labelled Chemistry menu client.",
		)

	#============================================
	def _molecule_diagnostics_busy(self) -> bool:
		"""Expose the one active detached read-only diagnostic delivery."""
		return self._molecule_diagnostics_intent is not None

	#============================================
	def _selected_molecule_diagnostics_address(self, tab: object) -> object | None:
		"""Resolve exactly one complete current direct-root molecule through the shared selector."""
		return ferrum_qt.ferrum.molecule_inspection.selected_durable_molecule_address(tab)

	#============================================
	def _start_molecule_diagnostics(self) -> bool:
		"""Capture the active stable tab and its exact selected root."""
		return self._start_molecule_diagnostics_for_tab(self._active_native_tab())

	#============================================
	def _start_molecule_diagnostics_for_tab(self, tab: object | None) -> bool:
		"""Start one detached private Rust call with immutable current intent facts."""
		if (
			self._molecule_diagnostics_busy() or self._molecule_inspection_busy()
			or self._molecule_import_busy() or self._molecule_export_busy()
			or self._coordinate_generation_intent is not None
		):
			return False
		if (
			tab is None or self._native_tabs_by_page.get(tab) is not tab
			or tab.is_disposed or tab.requires_refresh or self._active_native_tab() is not tab
		):
			return False
		try:
			address = self._selected_molecule_diagnostics_address(tab)
			if address is None:
				return False
			snapshot = tab.current_snapshot
			worker = FerrumNativeMoleculeDiagnosticsWorker(
				snapshot.cdml, snapshot.revision, snapshot.digest, (address.molecule_id,),
			)
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError as error:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(error)))
			return False
		self._molecule_diagnostics_intent = _Intent(
			tab, snapshot.revision, snapshot.digest, address.molecule_id, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.diagnosed.connect(self._molecule_diagnostics_relay.on_diagnosed, connection)
		worker.failed.connect(self._molecule_diagnostics_relay.on_failed, connection)
		worker.finished.connect(self._molecule_diagnostics_relay.on_finished, connection)
		self.statusBar().showMessage(self.tr("Checking selected structure with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _current_molecule_diagnostics_intent(self, worker: object) -> _Intent | None:
		"""Accept an admitted receipt while its worker, tab, and source fence remain current."""
		intent = self._molecule_diagnostics_intent
		if intent is None or worker is not intent.worker or worker.delivery_cancelled:
			return None
		tab = intent.tab
		if (
			self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed
			or tab.requires_refresh or self._active_native_tab() is not tab
		):
			return None
		snapshot = tab.current_snapshot
		if (
			snapshot.revision != intent.revision or snapshot.digest != intent.digest
		):
			return None
		return intent

	#============================================
	def _on_document_molecule_diagnosed(self, worker: object, receipt: object) -> None:
		"""Open a modeless result only for its current authenticated receipt."""
		intent = self._current_molecule_diagnostics_intent(worker)
		if intent is None:
			return
		if (
			type(receipt) is not _Receipt or receipt.revision != intent.revision
			or receipt.digest != intent.digest or receipt.molecule_id != intent.molecule_id
		):
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum returned an invalid Check Structure receipt. Run the check again.",
			))
			return
		self._show_molecule_diagnostics_dialog(receipt, intent.tab)

	#============================================
	def _on_document_molecule_diagnostics_failed(self, worker: object, failure: object) -> None:
		"""Present a current typed worker failure through the standard refusal route."""
		if self._current_molecule_diagnostics_intent(worker) is not None:
			message = getattr(failure, "message", None)
			self._show_edit_refusal(self._unavailable_edit_refusal(
				message if type(message) is str and message else "Ferrum could not complete Check Structure.",
			))

	#============================================
	def _show_molecule_diagnostics_dialog(self, receipt: _Receipt, tab: object) -> None:
		"""Replace any older receipt with the new modeless result window."""
		if self._molecule_diagnostics_dialog is not None:
			self._molecule_diagnostics_dialog.close()
		dialog = FerrumNativeMoleculeDiagnosticsDialog(receipt, tab, self)
		dialog.rerun_requested.connect(self._rerun_molecule_diagnostics_from_dialog)
		dialog.finished.connect(self._on_molecule_diagnostics_dialog_finished)
		self._molecule_diagnostics_dialog = dialog
		dialog.show()
		dialog._findings.setFocus()

	#============================================
	def _rerun_molecule_diagnostics_from_dialog(self, dialog: object) -> bool:
		"""Rerun only when this historical receipt still identifies the current intent."""
		if dialog is not self._molecule_diagnostics_dialog:
			return False
		if not self._molecule_diagnostics_dialog_source_is_current(dialog):
			dialog.set_rerun_availability(False, self.tr(
				"Select the original molecule again before running Check Structure.",
			))
			return False
		return self._start_molecule_diagnostics_for_tab(dialog._tab)

	#============================================
	def _on_molecule_diagnostics_dialog_finished(self, *unused: object) -> None:
		"""Release only the dialog that completed its ordinary close lifecycle."""
		del unused
		if self.sender() is self._molecule_diagnostics_dialog:
			self._molecule_diagnostics_dialog = None

	#============================================
	def _on_document_molecule_diagnostics_finished(self, worker: object) -> None:
		"""Release the stopped worker and restore ordinary action reachability."""
		if self._molecule_diagnostics_intent is not None and worker is self._molecule_diagnostics_intent.worker:
			self._molecule_diagnostics_intent = None
			worker.deleteLater()
			self._refresh_actions()

	#============================================
	def _refresh_molecule_diagnostics_action(self, active: bool, pending: bool,
			busy_elsewhere: bool) -> None:
		"""Refresh exact one-root action eligibility and historical result recovery."""
		tab = self._active_native_tab()
		address = None if tab is None else self._selected_molecule_diagnostics_address(tab)
		self._check_structure_action.setEnabled(
			active and not pending and not busy_elsewhere and not self._molecule_diagnostics_busy()
			and not self._molecule_inspection_busy() and address is not None,
		)
		dialog = self._molecule_diagnostics_dialog
		if dialog is None:
			return
		if dialog._tab not in self._native_tabs_by_page:
			dialog.close_for_closed_source()
			return
		if not self._molecule_diagnostics_dialog_source_is_current(dialog):
			dialog.mark_stale()
			dialog.set_rerun_availability(False, self.tr(
				"Select the original molecule again before running Check Structure.",
			))
			return
		dialog.mark_current()
		dialog.set_rerun_availability(True, self.tr(
			"Run Check Structure again for this current selected molecule.",
		))

	#============================================
	def _molecule_diagnostics_dialog_source_is_current(self,
			dialog: FerrumNativeMoleculeDiagnosticsDialog) -> bool:
		"""Require the historical dialog's active tab, fence, and direct-root selection."""
		tab = dialog._tab
		if (
			self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed
			or tab.requires_refresh or self._active_native_tab() is not tab
		):
			return False
		snapshot = tab.current_snapshot
		address = self._selected_molecule_diagnostics_address(tab)
		return (
			snapshot.revision == dialog._receipt.revision and snapshot.digest == dialog._receipt.digest
			and address is not None and address.molecule_id == dialog._receipt.molecule_id
		)

	#============================================
	def _close_molecule_diagnostics_dialog_for_tab(self, tab: object) -> None:
		"""Remove a modeless receipt before its source tab is disposed."""
		dialog = self._molecule_diagnostics_dialog
		if dialog is not None and dialog._tab is tab:
			dialog.close_for_closed_source()

	#============================================
	def _molecule_diagnostics_blocks_tab_close(self, tab: object) -> bool:
		"""Suppress late detached delivery before its source tab is disposed."""
		intent = self._molecule_diagnostics_intent
		if intent is not None and intent.tab is tab:
			intent.worker.cancel_delivery()
		return False

	#============================================
	def _cancel_molecule_diagnostics_for_close(self) -> bool:
		"""Withdraw any read-only diagnostics delivery during window shutdown."""
		intent = self._molecule_diagnostics_intent
		if intent is not None:
			intent.worker.cancel_delivery()
		return False

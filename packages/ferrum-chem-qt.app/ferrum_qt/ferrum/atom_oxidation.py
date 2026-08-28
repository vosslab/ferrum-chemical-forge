"""Read-only public-protocol Atom Oxidation State interaction."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
from ferrum_qt.dialogs.accessibility import FerrumAccessibleDialog
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors

#============================================
_OPERATION_KIND = "document.atom.oxidation.observe.v1"
_CONVENTION = "formal-electron-assignment-hcno-v1"
_UNAVAILABLE_RECOVERY = {
	"element_outside_profile": "Use a fully materialized H/C/N/O molecule.",
	"formal_charge_unavailable": "Author explicit formal charges and hydrogens, then run again.",
	"hydrogen_topology_unsupported": "Author explicit hydrogen vertices, then run again.",
	"aromaticity_unsupported": "Use a non-aromatic materialized molecule.",
	"radical_unsupported": "Use an authored non-radical molecule.",
	"bond_order_unavailable": "Author supported bond orders, then run again.",
	"bond_order_unsupported": "Use single, double, or triple bonds.",
	"non_atom_vertex_unsupported": "Use an atom-only direct molecule root.",
	"coordination_or_delocalization_unsupported": "Use a non-coordination materialized molecule.",
	"component_invariant_failed": "Repair the authored molecular graph, then run again.",
	"arithmetic_overflow": "Use a smaller supported molecular graph.",
	"resource_limit": "Use a smaller supported molecular graph.",
}


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _AtomOxidationIntent:
	"""One immutable source fence and worker delivery identity."""

	tab: object
	revision: int
	digest: str
	molecule_id: str
	atom_id: str


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _AtomOxidationPresentation:
	"""One terminal receipt awaiting final Qt action-state presentation."""

	intent: _AtomOxidationIntent
	terminal_kind: str


#============================================
#============================================
def _number_text(number: int) -> str:
	"""Render a protocol-provided signed integer without deriving chemistry."""
	return str(number) if number <= 0 else "+{0}".format(number)


#============================================
def _accepted_text(observation: dict) -> str:
	"""Render one recognized accepted observation receipt."""
	return "\n".join((
		"Oxidation state: {0}".format(_number_text(observation["oxidation_number"])),
		"Convention: {0}".format(observation["convention"]),
	))


#============================================
def _unavailable_text(observation: dict) -> str:
	"""Render one closed unavailable outcome without making chemistry claims."""
	reason = observation["unavailable_reason"]
	return "\n".join((
		"Oxidation state: unavailable",
		"Reason: {0}".format(reason),
		"Recovery: {0}".format(_UNAVAILABLE_RECOVERY[reason]),
	))


#============================================
class FerrumNativeAtomOxidationDialog(FerrumAccessibleDialog):
	"""Modeless historical display of one immutable oxidation observation."""

	rerun_requested = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, tab: object, atom_id: str, revision: int, digest: str,
			details: str, source_status: str, parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build an accessible, resizable read-only receipt dialog."""
		super().__init__(parent)
		self._tab = tab
		self._source_revision = revision
		self._source_digest = digest
		self._source_closed = False
		self.setWindowTitle(self.tr("Atom Oxidation State"))
		self.setObjectName("atom-oxidation-dialog")
		self.setAccessibleName(self.tr("Atom Oxidation State"))
		self.setAccessibleDescription(self.tr(
			"Read-only Ferrum Rust oxidation-state observation for one selected atom.",
		))
		self.setWindowFlag(PySide6.QtCore.Qt.WindowType.Tool, True)
		self.setModal(False)
		self.setMinimumSize(560, 340)
		self.resize(680, 440)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		source = PySide6.QtWidgets.QLabel(self.tr(
			"Selected atom {0}; Ferrum document revision {1}".format(atom_id, revision),
		), self)
		source.setObjectName("atom-oxidation-source")
		source.setWordWrap(True)
		source.setAccessibleName(self.tr("Atom oxidation state source"))
		layout.addWidget(source)
		self._source_status = PySide6.QtWidgets.QLabel(source_status, self)
		self._source_status.setObjectName("atom-oxidation-source-status")
		self._source_status.setWordWrap(True)
		self._source_status.setAccessibleName(self.tr("Atom oxidation state source status"))
		layout.addWidget(self._source_status)
		self._details = PySide6.QtWidgets.QPlainTextEdit(self)
		self._details.setObjectName("atom-oxidation-details")
		self._details.setReadOnly(True)
		self._details.setPlainText(details)
		self._details.setFont(PySide6.QtGui.QFontDatabase.systemFont(
			PySide6.QtGui.QFontDatabase.SystemFont.FixedFont,
		))
		self._details.setAccessibleName(self.tr("Atom oxidation state details"))
		layout.addWidget(self._details, 1)
		self._copy = PySide6.QtWidgets.QPushButton(self.tr("Copy result"), self)
		self._copy.setObjectName("atom-oxidation-copy")
		self._copy.setAccessibleName(self.tr("Copy atom oxidation state result"))
		self._rerun = PySide6.QtWidgets.QPushButton(self.tr("Run Again"), self)
		self._rerun.setObjectName("atom-oxidation-run-again")
		self._rerun.setAccessibleName(self.tr("Run Atom Oxidation State again"))
		self._close = PySide6.QtWidgets.QPushButton(self.tr("Close"), self)
		self._close.setObjectName("atom-oxidation-close")
		self._close.setAccessibleName(self.tr("Close Atom Oxidation State"))
		buttons = PySide6.QtWidgets.QHBoxLayout()
		buttons.addWidget(self._copy)
		buttons.addStretch(1)
		buttons.addWidget(self._rerun)
		buttons.addWidget(self._close)
		layout.addLayout(buttons)
		self._copy.clicked.connect(self._copy_result)
		self._rerun.clicked.connect(self._request_rerun)
		self._close.clicked.connect(self.close)
		PySide6.QtWidgets.QWidget.setTabOrder(self._details, self._copy)
		PySide6.QtWidgets.QWidget.setTabOrder(self._copy, self._rerun)
		PySide6.QtWidgets.QWidget.setTabOrder(self._rerun, self._close)

	#============================================
	def _copy_result(self) -> None:
		"""Copy the visible receipt without changing the document."""
		PySide6.QtWidgets.QApplication.clipboard().setText(self._details.toPlainText())

	#============================================
	def mark_stale(self, explanation: str) -> None:
		"""Retain historical content while marking its source fence obsolete."""
		self._source_status.setText(explanation)

	#============================================
	def set_rerun_availability(self, available: bool, explanation: str) -> None:
		"""Expose only source-bound current-selection recapture."""
		self._rerun.setEnabled(available and not self._source_closed)
		self._rerun.setToolTip(explanation)
		self._rerun.setAccessibleDescription(explanation)

	#============================================
	def close_for_closed_source(self) -> None:
		"""Close before the captured source tab loses its live identity."""
		if self._source_closed:
			return
		self._source_closed = True
		self.close()

	#============================================
	def _request_rerun(self) -> None:
		"""Ask the owner to recapture only this dialog's source tab."""
		if not self._source_closed:
			self.rerun_requested.emit(self)


#============================================
class FerrumNativeAtomOxidationMixin:
	"""Own the source-fenced public oxidation observation lifecycle."""

	#============================================
	def _initialize_atom_oxidation(self) -> None:
		self._atom_oxidation_intent: _AtomOxidationIntent | None = None
		self._atom_oxidation_presentation: _AtomOxidationPresentation | None = None
		self._atom_oxidation_dialog: FerrumNativeAtomOxidationDialog | None = None

	#============================================
	def _build_atom_oxidation_action(self) -> None:
		self._atom_oxidation_action = PySide6.QtGui.QAction(
			self.tr("Atom Oxidation State..."), self,
		)
		self._atom_oxidation_action.setObjectName("atom-oxidation-action")
		self._atom_oxidation_action.setIconText(self.tr("Atom Oxidation State"))
		self._atom_oxidation_action.setStatusTip(self.tr(
			"Observe one selected atom's oxidation state with Ferrum Rust. "
			"Does not change the document.",
		))
		self._atom_oxidation_action.setToolTip(self.tr(
			"Observe the oxidation state of one selected atom with Ferrum Rust.",
		))
		self._atom_oxidation_action.setWhatsThis(self.tr(
			"Observe the oxidation state of one selected atom with Ferrum Rust. "
			"This action does not change the document.",
		))
		self._atom_oxidation_action.triggered.connect(self._start_atom_oxidation)
		self._action_registry.register_existing(
			"chemistry.atom.oxidation_state", self._atom_oxidation_action,
			shortcut_exemption_reason="Available by its labelled Chemistry menu client.",
		)

	#============================================
	def _atom_oxidation_busy(self) -> bool:
		return self._atom_oxidation_intent is not None

	#============================================
	def _start_atom_oxidation(self) -> bool:
		"""Capture one active selected atom and observe it through the live session."""
		return self._start_atom_oxidation_for_tab(self._active_native_tab())

	#============================================
	def _start_atom_oxidation_for_tab(self, tab: object | None) -> bool:
		"""Observe one fresh durable selected atom only on its original live tab."""
		if (
			self._atom_oxidation_busy()
			or self._molecule_import_busy()
			or self._molecule_export_busy()
			or self._molecule_inspection_busy()
			or self._coordinate_generation_intent is not None
		):
			return False
		if tab is None or self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed:
			return False
		try:
			address = tab.selected_molecule_atom_address()
		except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return False
		if self._atom_insertion_intent is not None:
			self._cancel_atom_insertion()
		if self._line_gesture_intent is not None:
			self._cancel_line_gesture()
		if self._structure_tab is tab:
			self._cancel_structure_selection()
		intent = _AtomOxidationIntent(
			tab, address.revision, address.digest, address.molecule_id, address.atom_id,
		)
		self._atom_oxidation_intent = intent
		self.statusBar().showMessage(self.tr("Observing atom oxidation state with Ferrum Rust..."), 0)
		self._refresh_actions()
		try:
			receipt = tab._session.observe_live_atom_oxidation_v1(
				address.revision, address.digest, address.molecule_id, address.atom_id,
			)
			observation = self._observation_from_live_receipt(intent, receipt)
		except (TypeError, ValueError, RuntimeError) as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._record_atom_oxidation_presentation(intent, "failed")
		else:
			if observation is None:
				self._show_edit_refusal(self._unavailable_edit_refusal(
					"Ferrum returned an invalid live atom oxidation observation.",
				))
				self._record_atom_oxidation_presentation(intent, "failed")
			else:
				details = _accepted_text(observation) if observation["status"] == "accepted" else (
					_unavailable_text(observation)
				)
				self._show_atom_oxidation_dialog(
					intent, details, "Current source document revision {0}.".format(intent.revision),
					"succeeded" if observation["status"] == "accepted" else "unavailable",
				)
		self._finish_atom_oxidation_intent(intent)
		return True

	#============================================
	def _observation_from_live_receipt(self, intent: _AtomOxidationIntent,
			receipt: object) -> dict | None:
		"""Copy one frozen live Rust receipt into the dialog's closed display vocabulary."""
		if (
			type(getattr(receipt, "source_revision", None)) is not int
			or receipt.source_revision != intent.revision
			or getattr(receipt, "source_digest_hex", None) != intent.digest
			or getattr(receipt, "molecule_object_id", None) != intent.molecule_id
			or getattr(receipt, "atom_object_id", None) != intent.atom_id
			or getattr(receipt, "status", None) not in ("accepted", "unavailable")
		):
			return None
		status = receipt.status
		if status == "accepted":
			if type(receipt.oxidation_number) is not int:
				return None
			return {
				"status": "accepted",
				"oxidation_number": receipt.oxidation_number,
				"convention": _CONVENTION,
			}
		reason = getattr(receipt, "unavailable_reason", None)
		if reason not in _UNAVAILABLE_RECOVERY:
			return None
		return {"status": "unavailable", "unavailable_reason": reason}

	#============================================
	def _show_atom_oxidation_dialog(self, intent: _AtomOxidationIntent,
			details: str, source_status: str, terminal_kind: str) -> None:
		if self._atom_oxidation_dialog is not None:
			self._atom_oxidation_dialog.close()
		dialog = FerrumNativeAtomOxidationDialog(
			intent.tab, intent.atom_id, intent.revision, intent.digest, details, source_status, self,
		)
		dialog.rerun_requested.connect(self._rerun_atom_oxidation_from_dialog)
		dialog.finished.connect(self._on_atom_oxidation_dialog_finished)
		self._atom_oxidation_dialog = dialog
		dialog.show()
		dialog._details.setFocus()
		self._record_atom_oxidation_presentation(intent, terminal_kind)

	#============================================
	def _record_atom_oxidation_presentation(self, intent: _AtomOxidationIntent,
			terminal_kind: str) -> None:
		"""Retain one visible terminal outcome until live-session presentation completes."""
		if self._atom_oxidation_intent is intent:
			self._atom_oxidation_presentation = _AtomOxidationPresentation(
				intent, terminal_kind,
			)

	#============================================
	def _rerun_atom_oxidation_from_dialog(self, dialog: object) -> bool:
		if dialog is not self._atom_oxidation_dialog:
			return False
		if not self._atom_oxidation_dialog_source_is_active(dialog):
			dialog.set_rerun_availability(False, self.tr(
				"Return to this result's source document, refresh it if needed, and select one atom.",
			))
			return False
		return self._start_atom_oxidation_for_tab(dialog._tab)

	#============================================
	def _on_atom_oxidation_dialog_finished(self, *unused: object) -> None:
		del unused
		if self.sender() is self._atom_oxidation_dialog:
			self._atom_oxidation_dialog = None

	#============================================
	def _finish_atom_oxidation_intent(self, intent: _AtomOxidationIntent) -> None:
		"""Release one completed owner-thread live observation and publish its receipt."""
		if self._atom_oxidation_intent is not intent:
			return
		presentation = self._atom_oxidation_presentation
		self._atom_oxidation_presentation = None
		self._atom_oxidation_intent = None
		self._refresh_actions()
		if presentation is not None and presentation.intent is intent:
			self._queue_operation_presentation_v1(
				intent.tab, _OPERATION_KIND, presentation.terminal_kind, "unchanged",
				intent.revision, intent.digest,
			)

	#============================================
	def _refresh_atom_oxidation_action(self, active: bool, pending: bool,
			busy_elsewhere: bool) -> None:
		tab = self._active_native_tab()
		available = False
		if active and not pending and not busy_elsewhere and not self._atom_oxidation_busy():
			try:
				tab.selected_molecule_atom_address()
			except native_document_tab_errors.FerrumNativeDocumentTabError:
				available = False
			else:
				available = True
		self._atom_oxidation_action.setEnabled(available)
		dialog = self._atom_oxidation_dialog
		if dialog is None:
			return
		if dialog._tab not in self._native_tabs_by_page:
			dialog.close_for_closed_source()
			return
		snapshot = dialog._tab.current_snapshot
		if snapshot.revision != dialog._source_revision or snapshot.digest != dialog._source_digest:
			dialog.mark_stale(self.tr("This result is from an earlier document revision."))
		if self._atom_oxidation_dialog_source_is_active(dialog):
			try:
				dialog._tab.selected_molecule_atom_address()
			except native_document_tab_errors.FerrumNativeDocumentTabError:
				dialog.set_rerun_availability(False, self.tr(
					"Select one atom in this source document before running again.",
				))
			else:
				dialog.set_rerun_availability(True, self.tr(
					"Run Atom Oxidation State again for this document's current selection.",
				))
		else:
			dialog.set_rerun_availability(False, self.tr(
				"Return to this result's source document, refresh it if needed, and select one atom.",
			))

	#============================================
	def _atom_oxidation_dialog_source_is_active(self, dialog: FerrumNativeAtomOxidationDialog) -> bool:
		"""Return whether this receipt's source remains a live active rerun target."""
		if (
			self._native_tabs_by_page.get(dialog._tab) is not dialog._tab
			or dialog._tab.is_disposed
			or dialog._tab.requires_refresh
			or self._active_native_tab() is not dialog._tab
		):
			return False
		# Receipt provenance controls historical display, while rerun recaptures
		# the current selected atom and its current revision/digest fence.
		return True

	#============================================
	def _close_atom_oxidation_dialog_for_tab(self, tab: object) -> None:
		dialog = self._atom_oxidation_dialog
		if dialog is not None and dialog._tab is tab:
			dialog.close_for_closed_source()

	#============================================
	def _atom_oxidation_blocks_tab_close(self, tab: object) -> bool:
		intent = self._atom_oxidation_intent
		if intent is None or intent.tab is not tab:
			return False
		self._show_edit_refusal(self._unavailable_edit_refusal(
			"Wait for Atom Oxidation State to finish before closing this tab.",
		))
		return True

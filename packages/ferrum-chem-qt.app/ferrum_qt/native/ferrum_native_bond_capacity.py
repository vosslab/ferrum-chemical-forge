"""Read-only native selected-molecule bond-capacity report."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_chem

# local repo modules
import ferrum_qt.native.ferrum_native_molecule_inspection


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _BondCapacityIntent:
	"""One immutable source address set and its native worker."""

	tab: object
	revision: int
	digest: str
	addresses: tuple
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeBondCapacityFailure:
	"""Terminal program failure facts safe for the Qt event thread."""

	message: str


#============================================
class FerrumNativeBondCapacityWorker(PySide6.QtCore.QThread):
	"""Calculate an immutable Rust receipt away from the event thread."""

	completed = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, observation: object, revision: int, digest: str,
			addresses: tuple) -> None:
		"""Freeze only Rust input and exact eventual-delivery facts."""
		if type(observation) is not ferrum_chem.SessionDocumentObservationV1:
			raise TypeError("bond capacity requires an exact Ferrum observation")
		super().__init__()
		self._arguments = (
			observation, revision, digest,
			tuple(address.molecule_id for address in addresses),
		)
		self._delivery_cancelled = False

	#============================================
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether future receipt delivery was invalidated."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Invalidate delivery while bounded Rust work reaches its normal end."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def run(self) -> None:
		"""Run the private read-only operation once."""
		try:
			result = ferrum_chem.inspect_document_bond_capacity_v1(*self._arguments)
		except Exception as exc:
			if not self._delivery_cancelled and not self.isInterruptionRequested():
				self.failed.emit(FerrumNativeBondCapacityFailure(str(exc)))
			return
		if not self._delivery_cancelled and not self.isInterruptionRequested():
			self.completed.emit(result)


#============================================
class _BondCapacityDeliveryRelay(PySide6.QtCore.QObject):
	"""Deliver exact worker identity to its owning native window."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the window that owns the active diagnostic intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_completed(self, result: object) -> None:
		"""Forward one read-only receipt."""
		self._owner._on_document_bond_capacity_completed(self.sender(), result)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward one program-state failure."""
		self._owner._on_document_bond_capacity_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the stopped worker after queued delivery."""
		self._owner._on_document_bond_capacity_finished(self.sender())


#============================================
def _not_checked_detail(reason: str) -> str:
	"""Turn one Rust-owned category into bounded author-facing guidance."""
	messages = {
		"non_atom_vertex": "This structure contains a non-atom vertex.",
		"non_neutral_charge": "This structure contains a nonzero authored charge.",
		"authored_atom_capacity_fact": "This structure contains authored atom capacity facts.",
		"unsupported_element": "This structure contains an element outside the supported neutral table.",
		"unsupported_bond_endpoint": "This structure contains a bond without two ordinary atom endpoints.",
		"unsupported_bond_order": "This structure contains a bond order outside single, double, or triple.",
		"aromatic_fact": "This structure contains an aromatic representation fact.",
	}
	return messages[reason]


#============================================
def _authored_capacity_facts(atom: object) -> str:
	"""Present retained source presence without extending the chemistry claim."""
	charge = atom.formal_charge
	hydrogens = atom.explicit_hydrogens
	charge_text = (
		f"authored charge {charge.value_or_zero:+d}"
		if charge.was_authored else "charge absent (used as 0)"
	)
	hydrogen_text = (
		f"authored explicit H {hydrogens.value_or_zero}"
		if hydrogens.was_authored else "explicit H absent (used as 0)"
	)
	return f"{charge_text}; {hydrogen_text}"


#============================================
def format_bond_capacity(result: object) -> str:
	"""Format only Rust receipt facts; Qt performs no capacity arithmetic."""
	lines = []
	for index, record in enumerate(result.records):
		name = record.authored_name if record.authored_name else "Molecule"
		lines.extend((name, f"Source ID: {record.source_id}"))
		if record.category == "not_checked":
			lines.extend((
				"Not checked",
				_not_checked_detail(record.not_checked_reason),
				"Use Molecule Information for authored facts, or simplify or replace this representation before checking it.",
			))
		elif record.category == "within_capacity":
			lines.append("No atom exceeds Ferrum's supported neutral bond-capacity table.")
			for atom in record.atoms:
				where = atom.source_id if atom.source_id else atom.element
				lines.append(f"{where}: {_authored_capacity_facts(atom)}.")
		else:
			lines.append("Bond-capacity finding")
			for atom in record.atoms:
				if atom.category == "exceeds_capacity":
					where = atom.source_id if atom.source_id else atom.element
					lines.append(
						f"{where}: demand {atom.demand}; supported capacity {atom.capacity}; "
						f"{_authored_capacity_facts(atom)}.",
					)
			for atom in record.atoms:
				if atom.category == "within_capacity":
					where = atom.source_id if atom.source_id else atom.element
					lines.append(
						f"{where}: demand {atom.demand}; supported capacity {atom.capacity}; "
						f"{_authored_capacity_facts(atom)}.",
					)
		if index + 1 < len(result.records):
			lines.extend(("", "--------------------", ""))
	return "\n".join(lines)


#============================================
class FerrumNativeBondCapacityDialog(PySide6.QtWidgets.QDialog):
	"""Selectable, read-only report for a current authenticated receipt."""

	#============================================
	def __init__(self, result: object, parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build one focused report without changing author-owned state."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Bond Capacity Check"))
		self.setModal(True)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		intro = PySide6.QtWidgets.QLabel(self.tr(
			"Ferrum checked the selected molecule structures against its supported neutral "
			"bond-capacity rules. This report does not change the document.",
		))
		intro.setWordWrap(True)
		layout.addWidget(intro)
		self._details = PySide6.QtWidgets.QPlainTextEdit(format_bond_capacity(result))
		self._details.setReadOnly(True)
		self._details.setAccessibleName(self.tr("Bond capacity check details"))
		self._details.setAccessibleDescription(self.tr(
			"Selected bond-capacity findings. This report does not edit the document.",
		))
		layout.addWidget(self._details, 1)
		buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Close,
		)
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)
		self._details.setFocus()

	#============================================
	@property
	def details_text(self) -> str:
		"""Return the selectable report for focused behavior checks."""
		return self._details.toPlainText()


#============================================
class FerrumNativeBondCapacityMixin:
	"""Own the cancellable read-only bond-capacity action and delivery fence."""

	#============================================
	def _initialize_bond_capacity(self) -> None:
		"""Create one intent slot and its queued delivery relay."""
		self._bond_capacity_intent: _BondCapacityIntent | None = None
		self._bond_capacity_relay = _BondCapacityDeliveryRelay(self)

	#============================================
	def _build_bond_capacity_actions(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the ordinary Chemistry action and explicit delivery cancellation."""
		self._check_bond_capacity_action = PySide6.QtGui.QAction(
			self.tr("Check Bond Capacity..."), self,
		)
		self._check_bond_capacity_action.setStatusTip(self.tr(
			"Check selected molecule structures using Ferrum's supported diagnostic rules. This does not change the document.",
		))
		self._check_bond_capacity_action.triggered.connect(self._start_bond_capacity_check)
		menu.addAction(self._check_bond_capacity_action)
		self._cancel_bond_capacity_action = PySide6.QtGui.QAction(
			self.tr("Cancel Bond Capacity Check"), self,
		)
		self._cancel_bond_capacity_action.triggered.connect(self._cancel_document_bond_capacity)
		menu.addAction(self._cancel_bond_capacity_action)

	#============================================
	def _bond_capacity_busy(self) -> bool:
		"""Return whether one worker still owns a source tab."""
		return self._bond_capacity_intent is not None

	#============================================
	def _start_bond_capacity_check(self) -> bool:
		"""Start only from current unambiguous durable selected roots."""
		if self._bond_capacity_busy() or self._molecule_inspection_busy():
			return False
		tab = self._active_native_tab()
		addresses = None if tab is None else (
			ferrum_qt.native.ferrum_native_molecule_inspection.
			selected_durable_molecule_addresses(tab)
		)
		if tab is None or addresses is None:
			return False
		try:
			snapshot = tab.current_snapshot
			worker = FerrumNativeBondCapacityWorker(
				tab.current_document_observation(), snapshot.revision, snapshot.digest, addresses,
			)
		except Exception as exc:
			self._show_native_file_warning("Bond Capacity Check Unavailable", str(exc))
			return False
		self._bond_capacity_intent = _BondCapacityIntent(
			tab, snapshot.revision, snapshot.digest, addresses, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.completed.connect(self._bond_capacity_relay.on_completed, connection)
		worker.failed.connect(self._bond_capacity_relay.on_failed, connection)
		worker.finished.connect(self._bond_capacity_relay.on_finished, connection)
		self.statusBar().showMessage(self.tr("Checking selected molecule with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _current_bond_capacity_intent(self, worker: object) -> _BondCapacityIntent | None:
		"""Return only a current active-tab exact-worker intent."""
		intent = self._bond_capacity_intent
		if intent is None or worker is not intent.worker or worker.delivery_cancelled:
			return None
		if intent.tab not in self._native_tabs_by_page or self._active_native_tab() is not intent.tab:
			return None
		snapshot = intent.tab.current_snapshot
		if intent.tab.requires_refresh or snapshot.revision != intent.revision or snapshot.digest != intent.digest:
			return None
		return intent

	#============================================
	def _on_document_bond_capacity_completed(self, worker: object, result: object) -> None:
		"""Show only an exact receipt corroborated against every selected root."""
		intent = self._current_bond_capacity_intent(worker)
		if intent is None:
			return
		if (
			type(result) is not ferrum_chem.DocumentBondCapacityV1
			or result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or len(result.records) != len(intent.addresses)
		):
			return
		for record, address in zip(result.records, intent.addresses, strict=True):
			if (
				record.molecule_id != address.molecule_id
				or record.projection_key != address.projection_key
				or record.source_id != address.source_id
				or record.document_root_order != address.document_root_order
			):
				return
		FerrumNativeBondCapacityDialog(result, self).exec()

	#============================================
	def _on_document_bond_capacity_failed(self, worker: object, failure: object) -> None:
		"""Show a current program failure separately from molecule findings."""
		if self._current_bond_capacity_intent(worker) is not None:
			self._show_native_file_warning("Bond Capacity Check Error", failure.message)

	#============================================
	def _on_document_bond_capacity_finished(self, worker: object) -> None:
		"""Release a stopped worker and restore ordinary reachability."""
		intent = self._bond_capacity_intent
		if intent is None or worker is not intent.worker:
			return
		self._bond_capacity_intent = None
		worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _cancel_document_bond_capacity(self) -> None:
		"""Suppress late delivery while the bounded native worker finishes."""
		intent = self._bond_capacity_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(self.tr("Cancelling bond-capacity check delivery..."), 0)
		self._refresh_actions()

	#============================================
	def _refresh_bond_capacity_actions(self, active: bool, pending: bool,
			busy_elsewhere: bool) -> None:
		"""Make selection and lifecycle reachability visible without inference."""
		tab = self._active_native_tab()
		addresses = None if tab is None else (
			ferrum_qt.native.ferrum_native_molecule_inspection.
			selected_durable_molecule_addresses(tab)
		)
		self._check_bond_capacity_action.setEnabled(
			active and not pending and not busy_elsewhere and not self._bond_capacity_busy()
			and addresses is not None,
		)
		self._cancel_bond_capacity_action.setEnabled(
			self._bond_capacity_intent is not None
			and not self._bond_capacity_intent.worker.delivery_cancelled,
		)

	#============================================
	def _bond_capacity_blocks_tab_close(self, tab: object) -> bool:
		"""Keep the exact source tab live until native work ends."""
		intent = self._bond_capacity_intent
		if intent is None or intent.tab is not tab:
			return False
		self._show_native_file_warning(
			"Bond Capacity Check Still Running",
			"Cancel the check and wait for native work before closing.",
		)
		return True

	#============================================
	def _cancel_bond_capacity_for_close(self) -> bool:
		"""Invalidate delivery before a later close attempt."""
		if self._bond_capacity_intent is None:
			return False
		self._cancel_document_bond_capacity()
		self._show_native_file_warning(
			"Bond Capacity Check Still Running",
			"Ferrum cancelled delivery; close again after native work finishes.",
		)
		return True

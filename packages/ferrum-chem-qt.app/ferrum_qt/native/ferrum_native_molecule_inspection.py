"""Read-only source facts and composition for selected durable molecules."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_chem


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeMoleculeInspectionAddress:
	"""One selected child mapped to one direct durable molecule root."""

	molecule_id: str
	projection_key: str
	source_id: str
	document_root_order: int


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _MoleculeInspectionIntent:
	"""One source tab and immutable receipt corroborators."""

	tab: object
	revision: int
	digest: str
	addresses: tuple[FerrumNativeMoleculeInspectionAddress, ...]
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeMoleculeInspectionFailure:
	"""Plain terminal worker failure facts safe for the Qt event thread."""

	message: str


#============================================
def selected_durable_molecule_addresses(
		tab: object) -> tuple[FerrumNativeMoleculeInspectionAddress, ...] | None:
	"""Resolve selected durable atoms/bonds to unique direct projected roots."""
	if getattr(tab, "requires_refresh", True):
		return None
	targets = tab.selected_molecule_information_targets()
	if type(targets) is not tuple or not targets:
		return None
	projection = tab.current_document_observation().projection
	addresses = {}
	for target in targets:
		if (
			target.kind not in ("atom", "bond")
			or type(target.identifier) is not str
			or not target.identifier
			or type(target.source_order) is not int
		):
			return None
		matches = []
		for molecule in projection.molecules:
			children = molecule.atoms if target.kind == "atom" else molecule.bonds
			matches.extend(
				molecule for child in children
				if child.source_id == target.identifier
				and child.source_order == target.source_order
			)
		if len(matches) != 1:
			return None
		molecule = matches[0]
		address = _molecule_information_address(molecule)
		if address is None:
			return None
		previous = addresses.setdefault(address.molecule_id, address)
		if previous != address:
			return None
	return tuple(sorted(addresses.values(), key=lambda address: address.document_root_order))


#============================================
def _molecule_information_address(
		molecule: object) -> FerrumNativeMoleculeInspectionAddress | None:
	"""Freeze one complete direct-root corroboration address."""
	if (
		type(molecule.id) is not str
		or not molecule.id
		or type(molecule.projection_key) is not str
		or not molecule.projection_key
		or type(molecule.source_id) is not str
		or not molecule.source_id
		or type(molecule.source_order) is not int
	):
		return None
	return FerrumNativeMoleculeInspectionAddress(
		molecule.id, molecule.projection_key, molecule.source_id, molecule.source_order,
	)


#============================================
def selected_durable_molecule_address(
		tab: object) -> FerrumNativeMoleculeInspectionAddress | None:
	"""Return the sole root only when the complete selection maps to one root."""
	addresses = selected_durable_molecule_addresses(tab)
	return None if addresses is None or len(addresses) != 1 else addresses[0]


#============================================
class FerrumNativeMoleculeInspectionWorker(PySide6.QtCore.QThread):
	"""Calculate frozen source and chemistry facts off the Qt event thread."""

	inspected = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, observation: object, revision: int, digest: str,
			addresses: tuple[FerrumNativeMoleculeInspectionAddress, ...]) -> None:
		"""Capture only immutable Rust input and stable delivery facts."""
		if type(observation) is not ferrum_chem.SessionDocumentObservationV1:
			raise TypeError("native inspection requires an exact Ferrum observation")
		super().__init__()
		self._arguments = (
			observation, revision, digest,
			tuple(address.molecule_id for address in addresses),
		)
		self._delivery_cancelled = False

	#============================================
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether result delivery has been invalidated."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Invalidate future delivery without claiming to interrupt Rust parsing."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def run(self) -> None:
		"""Run the private native composition operation at most once."""
		try:
			result = ferrum_chem.inspect_document_molecule_information_v1(*self._arguments)
		except Exception as exc:
			if not self._delivery_cancelled and not self.isInterruptionRequested():
				self.failed.emit(FerrumNativeMoleculeInspectionFailure(str(exc)))
			return
		if not self._delivery_cancelled and not self.isInterruptionRequested():
			self.inspected.emit(result)


#============================================
class _MoleculeInspectionDeliveryRelay(PySide6.QtCore.QObject):
	"""Deliver worker signals back to the owning ordinary native window."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the window responsible for the one inspection intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_inspected(self, result: object) -> None:
		"""Forward a receipt with the exact emitting worker identity."""
		self._owner._on_document_molecule_inspected(self.sender(), result)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward a failure with the exact emitting worker identity."""
		self._owner._on_document_molecule_inspection_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the stopped worker owned by this window."""
		self._owner._on_document_molecule_inspection_finished(self.sender())


#============================================
def _isotope_symbol(entry: object) -> str:
	"""Format one Rust-validated element key without changing its chemistry."""
	return entry.symbol if entry.isotope is None else f"[{entry.isotope}{entry.symbol}]"


#============================================
def _composition_lines(composition: object) -> tuple[str, ...]:
	"""Format one frozen Rust composition without recalculating it."""
	counts = ", ".join(
		f"{_isotope_symbol(entry)}: {entry.atom_count}"
		for entry in composition.element_counts
	)
	percentages = ", ".join(
		f"{_isotope_symbol(entry)}: {entry.percentage:.3f}%"
		for entry in composition.mass_percentages
	)
	return (
		f"Formula: {composition.formula}",
		f"Net formal charge: {composition.net_formal_charge:+d}",
		f"Average molecular weight: {composition.average_molecular_weight:.4f} Da",
		f"Monoisotopic mass: {composition.monoisotopic_mass:.8f} Da",
		f"Perceived element counts: {counts}",
		f"Composition by average mass: {percentages}",
	)


#============================================
def format_molecule_information(result: object) -> str:
	"""Format retained source facts followed by native perceived composition."""
	lines = []
	if len(result.records) > 1:
		lines.extend(("Individual molecules", "====================", ""))
	for index, record in enumerate(result.records):
		source = record.source_facts
		name = source.authored_name if source.authored_name else "(unnamed)"
		elements = ", ".join(
			f"{entry.symbol}: {entry.atom_count}" for entry in source.element_inventory
		)
		charge = (
			"not completely authored"
			if source.total_formal_charge is None
			else f"{source.total_formal_charge:+d}"
		)
		bounds = source.bounds
		bounds_text = "empty" if bounds is None else (
			f"x: {bounds.min_x:g} to {bounds.max_x:g}; "
			f"y: {bounds.min_y:g} to {bounds.max_y:g}"
		)
		atom_label = "atom" if source.atom_count == 1 else "atoms"
		bond_label = "bond" if source.bond_count == 1 else "bonds"
		lines.extend((
			f"Name: {name}",
			f"Source ID: {source.source_id}",
			f"Authored graph: {source.atom_count} {atom_label}, "
			f"{source.bond_count} {bond_label}",
			f"Authored elements: {elements}",
			f"Complete authored formal charge: {charge}",
			f"Authored bounds (points): {bounds_text}",
			*_composition_lines(record.composition),
		))
		if index + 1 < len(result.records):
			lines.extend(("", "--------------------", ""))
	if result.combined_selection is not None:
		lines.extend((
			"", "Combined selection", "==================", "",
			*_composition_lines(result.combined_selection),
		))
	return "\n".join(lines)


#============================================
class FerrumNativeMoleculeInformationDialog(PySide6.QtWidgets.QDialog):
	"""Resizable, selectable, Close-only native molecule information."""

	#============================================
	def __init__(self, result: object, parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build one read-only surface from a frozen Rust receipt."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Molecule Information"))
		self.setModal(True)
		self.setMinimumSize(600, 420)
		self.resize(640, 520)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		intro = PySide6.QtWidgets.QLabel(self.tr(
			f"Calculated by Ferrum Rust from document revision {result.source_revision}; "
			"implicit hydrogens are included.",
		))
		intro.setWordWrap(True)
		layout.addWidget(intro)
		self._details = PySide6.QtWidgets.QPlainTextEdit(format_molecule_information(result))
		self._details.setReadOnly(True)
		self._details.setFont(PySide6.QtGui.QFontDatabase.systemFont(
			PySide6.QtGui.QFontDatabase.SystemFont.FixedFont,
		))
		self._details.setAccessibleName(self.tr("Molecule chemistry details"))
		self._details.setAccessibleDescription(self.tr(
			"Selectable Ferrum Rust source, formula, charge, mass, and composition results.",
		))
		layout.addWidget(self._details, 1)
		buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Close,
		)
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)

	#============================================
	@property
	def details_text(self) -> str:
		"""Return visible selectable details for behavior verification."""
		return self._details.toPlainText()


#============================================
class FerrumNativeMoleculeInspectionMixin:
	"""Own the cancellable selected-molecule inspection action and delivery fence."""

	#============================================
	def _initialize_molecule_inspection(self) -> None:
		"""Initialize the one inspection intent and Qt-thread relay."""
		self._molecule_inspection_intent: _MoleculeInspectionIntent | None = None
		self._molecule_inspection_relay = _MoleculeInspectionDeliveryRelay(self)

	#============================================
	def _build_molecule_inspection_actions(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the one native molecule-information action and its cancellation."""
		menu.addSeparator()
		self._inspect_selected_molecule_action = PySide6.QtGui.QAction(
			self.tr("Molecule Information..."), self,
		)
		self._inspect_selected_molecule_action.triggered.connect(
			self._start_selected_molecule_inspection,
		)
		menu.addAction(self._inspect_selected_molecule_action)
		self._cancel_molecule_inspection_action = PySide6.QtGui.QAction(
			self.tr("Cancel Molecule Inspection"), self,
		)
		self._cancel_molecule_inspection_action.triggered.connect(
			self._cancel_document_molecule_inspection,
		)
		menu.addAction(self._cancel_molecule_inspection_action)

	#============================================
	def _molecule_inspection_busy(self) -> bool:
		"""Return whether an inspection worker remains live."""
		return self._molecule_inspection_intent is not None

	#============================================
	def _selected_molecule_inspection_address(self) -> FerrumNativeMoleculeInspectionAddress | None:
		"""Retain the old singular helper for one-root internal callers."""
		tab = self._active_native_tab()
		addresses = None if tab is None else selected_durable_molecule_addresses(tab)
		return None if addresses is None or len(addresses) != 1 else addresses[0]

	#============================================
	def _selected_molecule_information_addresses(
			self) -> tuple[FerrumNativeMoleculeInspectionAddress, ...] | None:
		"""Resolve the complete current selection for refresh and start."""
		tab = self._active_native_tab()
		return None if tab is None else selected_durable_molecule_addresses(tab)

	#============================================
	def _start_selected_molecule_inspection(self) -> bool:
		"""Begin information only for current direct durable molecule roots."""
		if (
			self._molecule_inspection_busy()
			or self._molecule_import_busy()
			or self._molecule_export_busy()
		):
			return False
		if self._coordinate_generation_intent is not None:
			return False
		tab = self._active_native_tab()
		addresses = self._selected_molecule_information_addresses()
		if tab is None or addresses is None:
			return False
		try:
			observation = tab.current_document_observation()
			snapshot = tab.current_snapshot
			worker = FerrumNativeMoleculeInspectionWorker(
				observation, snapshot.revision, snapshot.digest, addresses,
			)
		except Exception as exc:
			self._show_native_file_warning("Molecule Inspection Unavailable", str(exc))
			return False
		self._molecule_inspection_intent = _MoleculeInspectionIntent(
			tab, snapshot.revision, snapshot.digest, addresses, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.inspected.connect(self._molecule_inspection_relay.on_inspected, connection)
		worker.failed.connect(self._molecule_inspection_relay.on_failed, connection)
		worker.finished.connect(self._molecule_inspection_relay.on_finished, connection)
		self.statusBar().showMessage(
			self.tr("Calculating molecule information with Ferrum Rust..."), 0,
		)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _on_document_molecule_inspected(self, worker: object, result: object) -> None:
		"""Show only a receipt authenticated to every selected direct root."""
		intent = self._current_molecule_inspection_intent(worker)
		if intent is None:
			return
		if (
			type(result) is not ferrum_chem.DocumentMoleculeInformationV1
			or result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or len(result.records) != len(intent.addresses)
		):
			self.statusBar().showMessage(self.tr("Document changed; inspect again."), 5000)
			return
		for record, address in zip(result.records, intent.addresses, strict=True):
			source = record.source_facts
			if (
				source.source_revision != intent.revision
				or source.source_digest != intent.digest
				or source.molecule_id != address.molecule_id
				or source.projection_key != address.projection_key
				or source.source_id != address.source_id
				or source.document_root_order != address.document_root_order
			):
				self.statusBar().showMessage(
					self.tr("Document changed; inspect again."), 5000,
				)
				return
		if (result.combined_selection is None) != (len(intent.addresses) == 1):
			self.statusBar().showMessage(self.tr("Document changed; inspect again."), 5000)
			return
		self._show_molecule_information_dialog(result)

	#============================================
	def _show_molecule_information_dialog(self, result: object) -> None:
		"""Run one modal read-only dialog owned by the ordinary native window."""
		FerrumNativeMoleculeInformationDialog(result, self).exec()

	#============================================
	def _on_document_molecule_inspection_failed(self, worker: object, failure: object) -> None:
		"""Show a current noncancelled native receipt failure without fallback."""
		if self._current_molecule_inspection_intent(worker) is None:
			return
		self._show_native_file_warning("Molecule Information Error", failure.message)

	#============================================
	def _current_molecule_inspection_intent(self, worker: object) -> _MoleculeInspectionIntent | None:
		"""Return only an exact worker intent whose source remains displayable."""
		intent = self._molecule_inspection_intent
		if intent is None or worker is not intent.worker or worker.delivery_cancelled:
			return None
		tab = intent.tab
		if (
			tab not in self._native_tabs_by_page
			or self._active_native_tab() is not tab
			or tab.requires_refresh
		):
			return None
		snapshot = tab.current_snapshot
		if snapshot.revision != intent.revision or snapshot.digest != intent.digest:
			return None
		return intent

	#============================================
	def _on_document_molecule_inspection_finished(self, worker: object) -> None:
		"""Release one exact stopped worker and restore action reachability."""
		intent = self._molecule_inspection_intent
		if intent is None or worker is not intent.worker:
			return
		self._molecule_inspection_intent = None
		worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _cancel_document_molecule_inspection(self) -> None:
		"""Suppress delivery while Rust work finishes normally."""
		intent = self._molecule_inspection_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(self.tr("Cancelling molecule inspection delivery..."), 0)
		self._refresh_actions()

	#============================================
	def _refresh_molecule_inspection_actions(
			self, active: bool, pending: bool, busy_elsewhere: bool) -> None:
		"""Apply selection and lifecycle reachability to inspection actions."""
		self._inspect_selected_molecule_action.setEnabled(
			active
			and not pending
			and not busy_elsewhere
			and not self._molecule_inspection_busy()
			and self._selected_molecule_information_addresses() is not None
		)
		self._cancel_molecule_inspection_action.setEnabled(
			self._molecule_inspection_intent is not None
			and not self._molecule_inspection_intent.worker.delivery_cancelled,
		)

	#============================================
	def _molecule_inspection_blocks_tab_close(self, tab: object) -> bool:
		"""Keep an inspection source tab alive through worker teardown."""
		intent = self._molecule_inspection_intent
		if intent is None or intent.tab is not tab:
			return False
		self._show_native_file_warning(
			"Molecule Inspection Still Running",
			"Cancel the inspection and wait for native work before closing.",
		)
		return True

	#============================================
	def _cancel_molecule_inspection_for_close(self) -> bool:
		"""Cancel delivery and retain the source tab until a later close attempt."""
		if self._molecule_inspection_intent is None:
			return False
		self._cancel_document_molecule_inspection()
		self._show_native_file_warning(
			"Molecule Inspection Still Running",
			"Ferrum cancelled delivery; close again after native work finishes.",
		)
		return True

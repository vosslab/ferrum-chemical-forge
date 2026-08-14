"""Revision-bound molecule exports for the standalone Rust-native window."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_chem


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeMoleculeExportFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _MoleculeExportIntent:
	"""One exact source tab and its handle-free export worker."""

	tab: object
	revision: int
	digest: str
	molecule_id: str
	mode: object
	worker: object


#============================================
class FerrumNativeMoleculeInchiExportWorker(PySide6.QtCore.QThread):
	"""Export one validated document molecule without blocking the Qt thread."""

	exported = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, observation: object, molecule_id: str, mode: object) -> None:
		"""Capture immutable Rust values before native work begins."""
		if type(observation) is not ferrum_chem.SessionDocumentObservationV1:
			raise TypeError("native InChI export requires exact Ferrum observation")
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("native InChI export requires a durable molecule selector")
		if type(mode) is not ferrum_chem.InchiModeV1:
			raise TypeError("native InChI export requires an exact Ferrum mode")
		super().__init__()
		self._arguments = (observation, molecule_id, mode)
		self._export_operation = ferrum_chem.export_document_molecule_inchi_v1
		self._delivery_cancelled = False

	#============================================
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether future delivery has been invalidated."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Invalidate delivery without claiming to interrupt native chemistry."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def run(self) -> None:
		"""Export and emit at most one still-current terminal outcome."""
		try:
			result = self._export_operation(*self._arguments)
		except Exception as exc:
			if not self._delivery_cancelled and not self.isInterruptionRequested():
				self.failed.emit(
					FerrumNativeMoleculeExportFailure(type(exc).__name__, str(exc)),
				)
			return
		if not self._delivery_cancelled and not self.isInterruptionRequested():
			self.exported.emit(result)


#============================================
class _MoleculeExportDeliveryRelay(PySide6.QtCore.QObject):
	"""Route worker signals to their owning window on the Qt thread."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the window that owns the export intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_exported(self, result: object) -> None:
		"""Forward one result with its exact emitting worker."""
		self._owner._on_document_molecule_inchi_exported(self.sender(), result)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward one failure with its exact emitting worker."""
		self._owner._on_document_molecule_inchi_export_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the exact stopped worker."""
		self._owner._on_document_molecule_inchi_export_finished(self.sender())


#============================================
class FerrumNativeMoleculeExportsMixin:
	"""Own asynchronous molecule export intent and presentation."""

	#============================================
	def _initialize_molecule_exports(self) -> None:
		"""Initialize the one native molecule export intent."""
		self._molecule_export_intent: _MoleculeExportIntent | None = None
		self._molecule_export_relay = _MoleculeExportDeliveryRelay(self)

	#============================================
	def _build_molecule_export_actions(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add Standard and Fixed-H InChI export actions."""
		menu.addSeparator()
		self._export_standard_inchi_action = PySide6.QtGui.QAction(
			self.tr("Export Standard InChI"), self,
		)
		self._export_standard_inchi_action.triggered.connect(
			lambda: self._choose_document_molecule_inchi_export(
				ferrum_chem.InchiModeV1.standard,
			),
		)
		menu.addAction(self._export_standard_inchi_action)
		self._export_fixed_h_inchi_action = PySide6.QtGui.QAction(
			self.tr("Export Fixed-H InChI"), self,
		)
		self._export_fixed_h_inchi_action.triggered.connect(
			lambda: self._choose_document_molecule_inchi_export(
				ferrum_chem.InchiModeV1.fixed_hydrogen,
			),
		)
		menu.addAction(self._export_fixed_h_inchi_action)
		self._cancel_inchi_export_action = PySide6.QtGui.QAction(
			self.tr("Cancel InChI Export"), self,
		)
		self._cancel_inchi_export_action.triggered.connect(
			self._cancel_document_molecule_inchi_export,
		)
		menu.addAction(self._cancel_inchi_export_action)

	#============================================
	def _molecule_export_busy(self) -> bool:
		"""Return whether one native export worker remains live."""
		return self._molecule_export_intent is not None

	#============================================
	def _choose_document_molecule_inchi_export(self, mode: object) -> None:
		"""Choose one durable molecule and begin its native export."""
		tab = self._active_native_tab()
		if tab is None:
			return
		choices = tab.durable_molecule_choices()
		if not choices:
			self._show_native_file_warning(
				"Native InChI Export Unavailable",
				"This document has no durable molecule that Rust can export.",
			)
			return
		choice = choices[0]
		if len(choices) > 1:
			labels = tuple(item.label for item in choices)
			selected, accepted = PySide6.QtWidgets.QInputDialog.getItem(
				self, self.tr("Export InChI"), self.tr("Molecule:"), labels, 0, False,
			)
			if not accepted:
				return
			choice = choices[labels.index(selected)]
		self.start_document_molecule_inchi_export(choice.object_id, mode)

	#============================================
	def start_document_molecule_inchi_export(
			self, molecule_id: str, mode: object) -> bool:
		"""Start one exact-revision Rust-native InChI export."""
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("native InChI export requires a durable molecule selector")
		if type(mode) is not ferrum_chem.InchiModeV1:
			raise TypeError("native InChI export requires an exact Ferrum mode")
		if (
			self._molecule_export_busy()
			or self._molecule_import_busy()
			or self._coordinate_generation_intent is not None
		):
			return False
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return False
		try:
			observation = tab.current_document_observation()
			worker = FerrumNativeMoleculeInchiExportWorker(
				observation, molecule_id, mode,
			)
		except Exception as exc:
			self._show_native_file_warning("Native InChI Export Error", str(exc))
			return False
		snapshot = tab.current_snapshot
		self._molecule_export_intent = _MoleculeExportIntent(
			tab, snapshot.revision, snapshot.digest, molecule_id, mode, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.exported.connect(self._molecule_export_relay.on_exported, connection)
		worker.failed.connect(self._molecule_export_relay.on_failed, connection)
		worker.finished.connect(self._molecule_export_relay.on_finished, connection)
		self.statusBar().showMessage(self.tr("Exporting InChI with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _on_document_molecule_inchi_exported(self, worker: object, result: object) -> None:
		"""Publish one result only while its source observation remains current."""
		intent = self._molecule_export_intent
		if intent is None or worker is not intent.worker or intent.worker.delivery_cancelled:
			return
		if type(result) is not ferrum_chem.DocumentMoleculeInchiV1:
			self._show_native_file_warning(
				"Native InChI Export Error", "Ferrum returned an unexpected export value.",
			)
			return
		tab = intent.tab
		snapshot = tab.current_snapshot
		if (
			tab not in self._native_tabs_by_page
			or tab.requires_refresh
			or snapshot.revision != intent.revision
			or snapshot.digest != intent.digest
			or result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or result.molecule_id != intent.molecule_id
			or result.mode is not intent.mode
		):
			self.statusBar().showMessage(
				self.tr("Discarded stale InChI export; the source document changed."), 5000,
			)
			return
		PySide6.QtWidgets.QApplication.clipboard().setText(result.inchi)
		PySide6.QtWidgets.QMessageBox.information(
			self, self.tr("Ferrum InChI Export"),
			self.tr("InChI copied to the clipboard:\n\n{0}").format(result.inchi),
		)

	#============================================
	def _on_document_molecule_inchi_export_failed(
			self, worker: object, failure: object) -> None:
		"""Show one current noncancelled export failure without fallback."""
		intent = self._molecule_export_intent
		if intent is None or worker is not intent.worker or intent.worker.delivery_cancelled:
			return
		self._show_native_file_warning(
			"Native InChI Export Error", getattr(failure, "message", str(failure)),
		)

	#============================================
	def _on_document_molecule_inchi_export_finished(self, worker: object) -> None:
		"""Release one stopped export worker and restore action reachability."""
		intent = self._molecule_export_intent
		if intent is None or worker is not intent.worker:
			return
		self._molecule_export_intent = None
		intent.worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _cancel_document_molecule_inchi_export(self) -> None:
		"""Invalidate export delivery while native teardown finishes normally."""
		intent = self._molecule_export_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(
			self.tr("Cancelling InChI delivery; waiting for native work to finish..."), 0,
		)
		self._refresh_actions()

	#============================================
	def _refresh_molecule_export_actions(
			self, active: bool, pending: bool, busy_elsewhere: bool) -> None:
		"""Apply host reachability to native molecule exports."""
		can_start = active and not pending and not busy_elsewhere and not self._molecule_export_busy()
		self._export_standard_inchi_action.setEnabled(can_start)
		self._export_fixed_h_inchi_action.setEnabled(can_start)
		self._cancel_inchi_export_action.setEnabled(
			self._molecule_export_intent is not None
			and not self._molecule_export_intent.worker.delivery_cancelled,
		)

	#============================================
	def _molecule_export_blocks_tab_close(self, tab: object) -> bool:
		"""Keep the export's source tab alive through worker teardown."""
		intent = self._molecule_export_intent
		if intent is None or intent.tab is not tab:
			return False
		self._show_native_file_warning(
			"Native InChI Export Still Running",
			"Cancel the InChI export and wait for native work before closing.",
		)
		return True

	#============================================
	def _cancel_molecule_export_for_close(self) -> bool:
		"""Cancel live delivery and tell the host to ignore this close attempt."""
		if self._molecule_export_intent is None:
			return False
		self._cancel_document_molecule_inchi_export()
		self._show_native_file_warning(
			"Native InChI Export Still Running",
			"Ferrum cancelled delivery; close again after native work finishes.",
		)
		return True

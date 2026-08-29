"""Revision-bound molecule exports for the standalone Ferrum window."""

# Standard Library
import dataclasses
import os
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
from ferrum_qt.ferrum.background_job import FerrumDetachedJobThread
import ferrum_qt.ferrum.engine as engine

# local repo modules
import ferrum_qt.ferrum.molecule_inspection


_INCHI_EXPORT = "inchi"
_SMILES_EXPORT = "smiles"
_SMILES_PROFILE = "canonical-isomeric-v1"
_INCHI_FILE_FILTER = "InChI files (*.inchi);;All Files (*)"
_SMILES_FILE_FILTER = "SMILES files (*.smi);;All Files (*)"


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
	kind: str
	mode: object | None
	destination: str | None
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _MoleculeSmilesFileCapture:
	"""One exact selection retained across the destination dialog."""

	tab: object
	revision: int
	digest: str
	molecule_id: str


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _MoleculeInchiFileCapture:
	"""One chosen durable root retained across the destination dialog."""

	tab: object
	revision: int
	digest: str
	molecule_id: str


#============================================
class _FerrumNativeMoleculeExportWorker(FerrumDetachedJobThread):
	"""Run one handle-free Rust molecule export away from the Qt event thread."""

	exported = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(
			self, operation: object, arguments: tuple[object, ...],
			failure_mapper: object | None = None,
			) -> None:
		"""Capture one operation and its immutable handle-free arguments."""
		self._arguments = arguments
		self._export_operation = operation
		if failure_mapper is None:
			failure_mapper = lambda error: FerrumNativeMoleculeExportFailure(
				type(error).__name__, str(error),
			)
		super().__init__(lambda: self._export_operation(*self._arguments), failure_mapper)

	#============================================
	def _emit_success(self, result: object) -> None:
		"""Retain the export route's established result signal."""
		self.exported.emit(result)


#============================================
class FerrumNativeMoleculeInchiExportWorker(_FerrumNativeMoleculeExportWorker):
	"""Export one validated document molecule as InChI."""

	#============================================
	def __init__(self, observation: object, molecule_id: str, mode: object) -> None:
		"""Validate and freeze one exact InChI request."""
		if type(observation) is not engine.SessionDocumentObservationV1:
			raise TypeError("Ferrum InChI export requires exact Ferrum observation")
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("Ferrum InChI export requires a durable molecule selector")
		if type(mode) is not engine.InchiModeV1:
			raise TypeError("Ferrum InChI export requires an exact Ferrum mode")
		snapshot = observation.snapshot
		format = (
			engine.DocumentMoleculeExportFormat.inchi_standard
			if mode is engine.InchiModeV1.standard else
			engine.DocumentMoleculeExportFormat.inchi_fixed_hydrogen
		)
		super().__init__(engine.export_document_molecule,
			(observation, snapshot.revision, snapshot.digest, molecule_id, format))


#============================================
class FerrumNativeMoleculeSmilesExportWorker(_FerrumNativeMoleculeExportWorker):
	"""Export one validated document molecule as canonical isomeric SMILES."""

	#============================================
	def __init__(self, observation: object, molecule_id: str) -> None:
		"""Validate and freeze one exact SMILES request."""
		if type(observation) is not engine.SessionDocumentObservationV1:
			raise TypeError("Ferrum SMILES export requires exact Ferrum observation")
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("Ferrum SMILES export requires a durable molecule selector")
		snapshot = observation.snapshot
		super().__init__(engine.export_document_molecule,
			(observation, snapshot.revision, snapshot.digest, molecule_id,
				engine.DocumentMoleculeExportFormat.canonical_smiles))


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
		self._owner._on_document_molecule_exported(self.sender(), result)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward one failure with its exact emitting worker."""
		self._owner._on_document_molecule_export_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the exact stopped worker."""
		self._owner._on_document_molecule_export_finished(self.sender())


#============================================
class FerrumNativeMoleculeExportsMixin:
	"""Own asynchronous molecule export intent and presentation."""

	#============================================
	def _initialize_molecule_exports(self) -> None:
		"""Initialize the one Ferrum molecule export intent."""
		self._molecule_export_intent: _MoleculeExportIntent | None = None
		self._molecule_export_relay = _MoleculeExportDeliveryRelay(self)

	#============================================
	def _build_molecule_export_actions(self) -> None:
		"""Create and register exact Ferrum SMILES and InChI export actions."""
		self._export_smiles_action = PySide6.QtGui.QAction(
			self.tr("Export SMILES"), self,
		)
		self._export_smiles_action.triggered.connect(
			self._choose_document_molecule_smiles_export,
		)
		self._export_smiles_file_action = PySide6.QtGui.QAction(
			self.tr("Export SMILES File..."), self,
		)
		self._export_smiles_file_action.triggered.connect(
			self._choose_document_molecule_smiles_file_export,
		)
		self._export_standard_inchi_action = PySide6.QtGui.QAction(
			self.tr("Export Standard InChI"), self,
		)
		self._export_standard_inchi_action.triggered.connect(
			lambda: self._choose_document_molecule_inchi_export(
				engine.InchiModeV1.standard,
			),
		)
		self._export_standard_inchi_file_action = PySide6.QtGui.QAction(
			self.tr("Export Standard InChI File..."), self,
		)
		self._export_standard_inchi_file_action.triggered.connect(
			lambda: self._choose_document_molecule_inchi_file_export(
				engine.InchiModeV1.standard,
			),
		)
		self._export_fixed_h_inchi_action = PySide6.QtGui.QAction(
			self.tr("Export Fixed-H InChI"), self,
		)
		self._export_fixed_h_inchi_action.triggered.connect(
			lambda: self._choose_document_molecule_inchi_export(
				engine.InchiModeV1.fixed_hydrogen,
			),
		)
		self._export_fixed_h_inchi_file_action = PySide6.QtGui.QAction(
			self.tr("Export Fixed-H InChI File..."), self,
		)
		self._export_fixed_h_inchi_file_action.triggered.connect(
			lambda: self._choose_document_molecule_inchi_file_export(
				engine.InchiModeV1.fixed_hydrogen,
			),
		)
		self._cancel_molecule_export_action = PySide6.QtGui.QAction(
			self.tr("Cancel Molecule Export"), self,
		)
		self._cancel_molecule_export_action.triggered.connect(
			self._cancel_document_molecule_export,
		)
		for action_id, action in (
			("file.export.smiles", self._export_smiles_action),
			("file.export.smiles_file", self._export_smiles_file_action),
			("file.export.inchi.standard", self._export_standard_inchi_action),
			("file.export.inchi.standard_file", self._export_standard_inchi_file_action),
			("file.export.inchi.fixed_h", self._export_fixed_h_inchi_action),
			("file.export.inchi.fixed_h_file", self._export_fixed_h_inchi_file_action),
			("file.export.cancel", self._cancel_molecule_export_action),
		):
			action.setStatusTip(action.text())
			self._action_registry.register_existing(
				action_id, action,
				lifecycle="stateful-cancel" if action_id == "file.export.cancel" else "static",
				shortcut_exemption_reason="Available by its labelled File menu client.",
			)

	#============================================
	def _molecule_export_busy(self) -> bool:
		"""Return whether one Ferrum export worker remains live."""
		return self._molecule_export_intent is not None

	#============================================
	def _choose_document_molecule_inchi_export(self, mode: object) -> None:
		"""Choose one durable molecule and begin its Ferrum export."""
		selected = self._choose_document_molecule_inchi_target("Export InChI")
		if selected is None:
			return
		_tab, molecule_id = selected
		self.start_document_molecule_inchi_export(molecule_id, mode)

	#============================================
	def _choose_document_molecule_inchi_target(
			self, title: str) -> tuple[object, str] | None:
		"""Choose one exact durable molecule without starting the current operation."""
		tab = self._active_native_tab()
		if tab is None:
			return None
		choices = tab.durable_molecule_choices()
		if not choices:
			self._show_edit_refusal(self._unavailable_edit_refusal("This document has no durable molecule that Rust can export."))
			return None
		choice = choices[0]
		if len(choices) > 1:
			labels = tuple(item.label for item in choices)
			selected, accepted = PySide6.QtWidgets.QInputDialog.getItem(
				self, self.tr(title), self.tr("Molecule:"), labels, 0, False,
			)
			if not accepted:
				return None
			choice = choices[labels.index(selected)]
		return tab, choice.object_id

	#============================================
	def _choose_document_molecule_inchi_file_export(self, mode: object) -> None:
		"""Choose one exact root and a destination for its InChI receipt."""
		selected = self._choose_document_molecule_inchi_target("Export InChI File")
		if selected is None:
			return
		tab, molecule_id = selected
		snapshot = tab.current_snapshot
		capture = _MoleculeInchiFileCapture(
			tab, snapshot.revision, snapshot.digest, molecule_id,
		)
		selected_path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Export InChI File"), "", self.tr(_INCHI_FILE_FILTER),
		)[0]
		if not selected_path:
			return
		destination = self._normalize_document_molecule_inchi_path(selected_path)
		if destination is None:
			return
		if not self._inchi_file_capture_is_current(capture):
			self._show_edit_refusal(self._unavailable_edit_refusal("The chosen molecule changed while choosing a destination. "
				"Choose Export InChI File again for the current document."))
			return
		if not self.start_document_molecule_inchi_export(
				capture.molecule_id, mode, destination):
			self._show_edit_refusal(self._unavailable_edit_refusal("Another operation started while choosing the destination. "
				"Choose Export InChI File again after it finishes."))

	#============================================
	def _normalize_document_molecule_inchi_path(self, selected_path: str) -> str | None:
		"""Apply the closed .inchi file policy without writing or adopting the path."""
		path = pathlib.Path(selected_path)
		if not path.suffix:
			path = path.with_suffix(".inchi")
		elif path.suffix.lower() != ".inchi":
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum InChI files must use the .inchi extension."))
			return None
		return os.path.abspath(str(path))

	#============================================
	def _inchi_file_capture_is_current(self, capture: _MoleculeInchiFileCapture) -> bool:
		"""Reauthenticate the exact root and tab after the file dialog."""
		tab = capture.tab
		if (
			self._active_native_tab() is not tab
			or self._native_tabs_by_page.get(tab) is not tab
			or tab.is_disposed
			or tab.requires_refresh
		):
			return False
		snapshot = tab.current_snapshot
		choices = tab.durable_molecule_choices()
		return (
			snapshot.revision == capture.revision
			and snapshot.digest == capture.digest
			and any(choice.object_id == capture.molecule_id for choice in choices)
		)

	#============================================
	def _choose_document_molecule_smiles_export(self) -> None:
		"""Export only a selection that maps to one exact durable molecule."""
		tab = self._active_native_tab()
		if tab is None:
			return
		address = (
			ferrum_qt.ferrum.molecule_inspection.
			selected_durable_molecule_address(tab)
		)
		if address is None:
			self._show_edit_refusal(self._unavailable_edit_refusal("Select atoms or bonds from exactly one durable molecule."))
			return
		self.start_document_molecule_smiles_export(address.molecule_id)

	#============================================
	def _choose_document_molecule_smiles_file_export(self) -> None:
		"""Choose a destination for one exact selected canonical SMILES receipt."""
		tab = self._active_native_tab()
		if tab is None:
			return
		address = (
			ferrum_qt.ferrum.molecule_inspection.
			selected_durable_molecule_address(tab)
		)
		if address is None:
			self._show_edit_refusal(self._unavailable_edit_refusal("Select atoms or bonds from exactly one durable molecule."))
			return
		snapshot = tab.current_snapshot
		capture = _MoleculeSmilesFileCapture(
			tab, snapshot.revision, snapshot.digest, address.molecule_id,
		)
		selected_path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Export SMILES File"), "", self.tr(_SMILES_FILE_FILTER),
		)[0]
		if not selected_path:
			return
		destination = self._normalize_document_molecule_smiles_path(selected_path)
		if destination is None:
			return
		if not self._smiles_file_capture_is_current(capture):
			self._show_edit_refusal(self._unavailable_edit_refusal("The selected molecule changed while choosing a destination. "
				"Choose Export SMILES File again for the current selection."))
			return
		if not self.start_document_molecule_smiles_export(
				capture.molecule_id, destination):
			self._show_edit_refusal(self._unavailable_edit_refusal("Another operation started while choosing the destination. "
				"Choose Export SMILES File again after it finishes."))

	#============================================
	def _normalize_document_molecule_smiles_path(self, selected_path: str) -> str | None:
		"""Apply the closed .smi file policy without writing or adopting the path."""
		path = pathlib.Path(selected_path)
		if not path.suffix:
			path = path.with_suffix(".smi")
		elif path.suffix.lower() != ".smi":
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum SMILES files must use the .smi extension."))
			return None
		return os.path.abspath(str(path))

	#============================================
	def _smiles_file_capture_is_current(self, capture: _MoleculeSmilesFileCapture) -> bool:
		"""Reauthenticate the exact active selection after the file dialog."""
		tab = capture.tab
		if (
			self._active_native_tab() is not tab
			or self._native_tabs_by_page.get(tab) is not tab
			or tab.is_disposed
			or tab.requires_refresh
		):
			return False
		address = (
			ferrum_qt.ferrum.molecule_inspection.
			selected_durable_molecule_address(tab)
		)
		snapshot = tab.current_snapshot
		return (
			address is not None
			and address.molecule_id == capture.molecule_id
			and snapshot.revision == capture.revision
			and snapshot.digest == capture.digest
		)

	#============================================
	def start_document_molecule_inchi_export(
			self, molecule_id: str, mode: object,
			destination: str | None = None) -> bool:
		"""Start one exact-revision Ferrum InChI export."""
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("Ferrum InChI export requires a durable molecule selector")
		if type(mode) is not engine.InchiModeV1:
			raise TypeError("Ferrum InChI export requires an exact Ferrum mode")
		if destination is not None and (
			type(destination) is not str or not os.path.isabs(destination)
		):
			raise ValueError("Ferrum InChI file export requires an absolute destination")
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
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return False
		snapshot = tab.current_snapshot
		self._molecule_export_intent = _MoleculeExportIntent(
			tab, snapshot.revision, snapshot.digest, molecule_id,
			_INCHI_EXPORT, mode, destination, worker,
		)
		self._connect_molecule_export_worker(worker)
		message = (
			self.tr("Preparing InChI file with Ferrum Rust...")
			if destination is not None else
			self.tr("Exporting InChI with Ferrum Rust...")
		)
		self.statusBar().showMessage(message, 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def start_document_molecule_smiles_export(
			self, molecule_id: str, destination: str | None = None) -> bool:
		"""Start one exact-revision Ferrum canonical SMILES export."""
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("Ferrum SMILES export requires a durable molecule selector")
		if destination is not None and (
			type(destination) is not str or not os.path.isabs(destination)
		):
			raise ValueError("Ferrum SMILES file export requires an absolute destination")
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
			worker = FerrumNativeMoleculeSmilesExportWorker(observation, molecule_id)
		except Exception as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return False
		snapshot = tab.current_snapshot
		self._molecule_export_intent = _MoleculeExportIntent(
			tab, snapshot.revision, snapshot.digest, molecule_id,
			_SMILES_EXPORT, None, destination, worker,
		)
		self._connect_molecule_export_worker(worker)
		message = (
			self.tr("Preparing SMILES file with Ferrum Rust...")
			if destination is not None else
			self.tr("Exporting SMILES with Ferrum Rust...")
		)
		self.statusBar().showMessage(message, 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _connect_molecule_export_worker(self, worker: object) -> None:
		"""Connect one worker to the sole export delivery relay."""
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.exported.connect(self._molecule_export_relay.on_exported, connection)
		worker.failed.connect(self._molecule_export_relay.on_failed, connection)
		worker.finished.connect(self._molecule_export_relay.on_finished, connection)

	#============================================
	def _current_molecule_export_intent(
			self, worker: object, kind: str) -> _MoleculeExportIntent | None:
		"""Return one admitted export while its exact source tab and fence remain current."""
		intent = self._molecule_export_intent
		if (
			intent is None
			or worker is not intent.worker
			or intent.kind != kind
			or intent.worker.delivery_cancelled
		):
			return None
		tab = intent.tab
		if (
			tab not in self._native_tabs_by_page
			or tab is not self._active_native_tab()
			or tab.is_disposed
			or tab.requires_refresh
		):
			return None
		snapshot = tab.current_snapshot
		if snapshot.revision != intent.revision or snapshot.digest != intent.digest:
			return None
		return intent

	#============================================
	def _on_document_molecule_exported(self, worker: object, result: object) -> None:
		"""Route one exact result through its operation-specific verifier."""
		intent = self._molecule_export_intent
		if intent is None or worker is not intent.worker:
			return
		if intent.kind == _INCHI_EXPORT:
			self._on_document_molecule_inchi_exported(worker, result)
		elif intent.kind == _SMILES_EXPORT:
			self._on_document_molecule_smiles_exported(worker, result)

	#============================================
	def _on_document_molecule_inchi_exported(self, worker: object, result: object) -> None:
		"""Publish one InChI only while its source observation remains current."""
		intent = self._current_molecule_export_intent(worker, _INCHI_EXPORT)
		if intent is None:
			self._show_stale_molecule_export(_INCHI_EXPORT)
			return
		if type(result) is not engine.DocumentMoleculeExport:
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum returned an unexpected export value."))
			return
		if (
			result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or result.molecule_id != intent.molecule_id
			or result.format is not (
				engine.DocumentMoleculeExportFormat.inchi_standard
				if intent.mode is engine.InchiModeV1.standard else
				engine.DocumentMoleculeExportFormat.inchi_fixed_hydrogen
			)
		):
			self._show_stale_molecule_export(_INCHI_EXPORT)
			return
		if intent.destination is not None:
			self._publish_document_molecule_export_file(result, intent.destination, "InChI")
			return
		PySide6.QtWidgets.QApplication.clipboard().setText(result.text)
		PySide6.QtWidgets.QMessageBox.information(
			self, self.tr("Ferrum InChI Export"),
			self.tr("InChI copied to the clipboard:\n\n{0}").format(result.text),
		)

	#============================================
	def _publish_document_molecule_export_file(
			self, receipt: object, destination: str, label: str) -> None:
		"""Publish one verified frozen receipt through Rust's artifact writer."""
		try:
			publication = engine.publish_document_molecule_export(
				receipt, destination,
			)
		except Exception as exc:
			self._report_molecule_file_publication_error(label, destination, exc)
			return
		if type(publication) is not engine.DocumentMoleculeExportPublication:
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum returned an unexpected publication value. Inspect the destination "
				"because Rust may already have written it."))
			return
		if publication.directory_entry_confirmed:
			self.statusBar().showMessage(
				self.tr("%s file exported: %s") % (label, destination), 5000,
			)
			return
		self._show_edit_refusal(self._unavailable_edit_refusal("The exact %s file is present at %s, but directory-entry durability "
			"needs verification. Inspect the destination before relying on it."
			% (label, destination)))

	#============================================
	def _on_document_molecule_smiles_exported(self, worker: object, result: object) -> None:
		"""Copy one canonical SMILES only while every source fact remains exact."""
		intent = self._current_molecule_export_intent(worker, _SMILES_EXPORT)
		if intent is None:
			self._show_stale_molecule_export(_SMILES_EXPORT)
			return
		if type(result) is not engine.DocumentMoleculeExport:
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum returned an unexpected export value."))
			return
		if (
			result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or result.molecule_id != intent.molecule_id
			or result.format is not engine.DocumentMoleculeExportFormat.canonical_smiles
		):
			self._show_stale_molecule_export(_SMILES_EXPORT)
			return
		if intent.destination is not None:
			self._publish_document_molecule_export_file(result, intent.destination, "SMILES")
			return
		PySide6.QtWidgets.QApplication.clipboard().setText(result.text)
		self._show_document_molecule_smiles(result.text)

	#============================================

	#============================================
	def _report_molecule_file_publication_error(
			self, label: str, destination: str, exc: Exception) -> None:
		"""Describe each publisher outcome without claiming a completed export."""
		if type(exc) is engine.PublicationPossiblyCompletedError:
			message = (
				"Rust could not confirm whether publication completed at %s. Verify the "
				"destination before relying on it.\n\n%s" % (destination, exc)
			)
		elif type(exc) is engine.PublicationNotStartedError:
			message = "Rust did not publish a %s file to %s.\n\n%s" % (
				label, destination, exc,
			)
		elif type(exc) is engine.InvalidDestinationError:
			message = "Rust rejected %s. Choose a different destination.\n\n%s" % (
				destination, exc,
			)
		else:
			message = "Could not export %s to %s:\n%s" % (label, destination, exc)
		self._show_edit_refusal(self._unavailable_edit_refusal(message))

	#============================================
	def _show_document_molecule_smiles(self, smiles: str) -> None:
		"""Show one copied SMILES line in a selectable result dialog."""
		dialog = PySide6.QtWidgets.QMessageBox(self)
		dialog.setIcon(PySide6.QtWidgets.QMessageBox.Icon.Information)
		dialog.setWindowTitle(self.tr("Ferrum SMILES Export"))
		dialog.setText(self.tr("SMILES copied to the clipboard:\n\n{0}").format(smiles))
		dialog.setTextInteractionFlags(
			PySide6.QtCore.Qt.TextInteractionFlag.TextSelectableByMouse
			| PySide6.QtCore.Qt.TextInteractionFlag.TextSelectableByKeyboard,
		)
		dialog.exec()

	#============================================
	def _show_stale_molecule_export(self, kind: str) -> None:
		"""Report that an old worker result was deliberately withheld."""
		label = "InChI" if kind == _INCHI_EXPORT else "SMILES"
		self.statusBar().showMessage(
			self.tr("Discarded stale {0} export; the source selection changed.").format(label),
			5000,
		)

	#============================================
	def _on_document_molecule_export_failed(
			self, worker: object, failure: object) -> None:
		"""Show one current noncancelled export failure without fallback."""
		intent = self._molecule_export_intent
		if intent is None or worker is not intent.worker:
			return
		if self._current_molecule_export_intent(worker, intent.kind) is None:
			self._show_stale_molecule_export(intent.kind)
			return
		self._show_edit_refusal(self._unavailable_edit_refusal(getattr(failure, "message", str(failure))))

	#============================================
	def _on_document_molecule_export_finished(self, worker: object) -> None:
		"""Release one stopped export worker and restore action reachability."""
		intent = self._molecule_export_intent
		if intent is None or worker is not intent.worker:
			return
		self._molecule_export_intent = None
		intent.worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _cancel_document_molecule_export(self) -> None:
		"""Invalidate export delivery while worker cleanup finishes normally."""
		intent = self._molecule_export_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(
			self.tr("Cancelling molecule export; waiting for the current operation to finish..."), 0,
		)
		self._refresh_actions()

	#============================================
	def _refresh_molecule_export_actions(
			self, active: bool, pending: bool, busy_elsewhere: bool) -> None:
		"""Apply host reachability to Ferrum molecule exports."""
		can_start = active and not pending and not busy_elsewhere and not self._molecule_export_busy()
		tab = self._active_native_tab() if can_start else None
		address = None if tab is None else (
			ferrum_qt.ferrum.molecule_inspection.
			selected_durable_molecule_address(tab)
		)
		self._export_smiles_action.setEnabled(can_start and address is not None)
		self._export_smiles_file_action.setEnabled(can_start and address is not None)
		self._export_standard_inchi_action.setEnabled(can_start)
		self._export_standard_inchi_file_action.setEnabled(can_start)
		self._export_fixed_h_inchi_action.setEnabled(can_start)
		self._export_fixed_h_inchi_file_action.setEnabled(can_start)
		self._cancel_molecule_export_action.setEnabled(
			self._molecule_export_intent is not None
			and not self._molecule_export_intent.worker.delivery_cancelled,
		)

	#============================================
	def _molecule_export_blocks_tab_close(self, tab: object) -> bool:
		"""Keep the export's source tab alive through worker teardown."""
		intent = self._molecule_export_intent
		if intent is None or intent.tab is not tab:
			return False
		self._show_edit_refusal(self._unavailable_edit_refusal("Cancel the molecule export and wait for the current operation before closing."))
		return True

	#============================================
	def _cancel_molecule_export_for_close(self) -> bool:
		"""Cancel live delivery and tell the host to ignore this close attempt."""
		if self._molecule_export_intent is None:
			return False
		self._cancel_document_molecule_export()
		self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum cancelled delivery; close again after the current operation finishes."))
		return True

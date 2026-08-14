"""Revision-bound molecule exports for the standalone Rust-native window."""

# Standard Library
import dataclasses
import os
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_chem

# local repo modules
import ferrum_qt.native.ferrum_native_molecule_inspection


_INCHI_EXPORT = "inchi"
_SMILES_EXPORT = "smiles"
_SMILES_PROFILE = "canonical-isomeric-v1"
_SMILES_SCHEMA = "ferrum-document-molecule-smiles-v1"
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
class _FerrumNativeMoleculeExportWorker(PySide6.QtCore.QThread):
	"""Run one handle-free Rust molecule export away from the Qt event thread."""

	exported = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, operation: object, arguments: tuple[object, ...]) -> None:
		"""Capture one operation and its immutable handle-free arguments."""
		super().__init__()
		self._arguments = arguments
		self._export_operation = operation
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
class FerrumNativeMoleculeInchiExportWorker(_FerrumNativeMoleculeExportWorker):
	"""Export one validated document molecule as InChI."""

	#============================================
	def __init__(self, observation: object, molecule_id: str, mode: object) -> None:
		"""Validate and freeze one exact InChI request."""
		if type(observation) is not ferrum_chem.SessionDocumentObservationV1:
			raise TypeError("native InChI export requires exact Ferrum observation")
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("native InChI export requires a durable molecule selector")
		if type(mode) is not ferrum_chem.InchiModeV1:
			raise TypeError("native InChI export requires an exact Ferrum mode")
		super().__init__(
			ferrum_chem.export_document_molecule_inchi_v1,
			(observation, molecule_id, mode),
		)


#============================================
class FerrumNativeMoleculeSmilesExportWorker(_FerrumNativeMoleculeExportWorker):
	"""Export one validated document molecule as canonical isomeric SMILES."""

	#============================================
	def __init__(self, observation: object, molecule_id: str) -> None:
		"""Validate and freeze one exact SMILES request."""
		if type(observation) is not ferrum_chem.SessionDocumentObservationV1:
			raise TypeError("native SMILES export requires exact Ferrum observation")
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("native SMILES export requires a durable molecule selector")
		snapshot = observation.snapshot
		super().__init__(
			ferrum_chem.export_document_molecule_smiles_v1,
			(observation, snapshot.revision, snapshot.digest, molecule_id),
		)


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
		"""Initialize the one native molecule export intent."""
		self._molecule_export_intent: _MoleculeExportIntent | None = None
		self._molecule_export_relay = _MoleculeExportDeliveryRelay(self)

	#============================================
	def _build_molecule_export_actions(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add exact native SMILES and InChI export actions."""
		menu.addSeparator()
		self._export_smiles_action = PySide6.QtGui.QAction(
			self.tr("Export SMILES"), self,
		)
		self._export_smiles_action.triggered.connect(
			self._choose_document_molecule_smiles_export,
		)
		menu.addAction(self._export_smiles_action)
		self._export_smiles_file_action = PySide6.QtGui.QAction(
			self.tr("Export SMILES File..."), self,
		)
		self._export_smiles_file_action.triggered.connect(
			self._choose_document_molecule_smiles_file_export,
		)
		menu.addAction(self._export_smiles_file_action)
		self._export_standard_inchi_action = PySide6.QtGui.QAction(
			self.tr("Export Standard InChI"), self,
		)
		self._export_standard_inchi_action.triggered.connect(
			lambda: self._choose_document_molecule_inchi_export(
				ferrum_chem.InchiModeV1.standard,
			),
		)
		menu.addAction(self._export_standard_inchi_action)
		self._export_standard_inchi_file_action = PySide6.QtGui.QAction(
			self.tr("Export Standard InChI File..."), self,
		)
		self._export_standard_inchi_file_action.triggered.connect(
			lambda: self._choose_document_molecule_inchi_file_export(
				ferrum_chem.InchiModeV1.standard,
			),
		)
		menu.addAction(self._export_standard_inchi_file_action)
		self._export_fixed_h_inchi_action = PySide6.QtGui.QAction(
			self.tr("Export Fixed-H InChI"), self,
		)
		self._export_fixed_h_inchi_action.triggered.connect(
			lambda: self._choose_document_molecule_inchi_export(
				ferrum_chem.InchiModeV1.fixed_hydrogen,
			),
		)
		menu.addAction(self._export_fixed_h_inchi_action)
		self._export_fixed_h_inchi_file_action = PySide6.QtGui.QAction(
			self.tr("Export Fixed-H InChI File..."), self,
		)
		self._export_fixed_h_inchi_file_action.triggered.connect(
			lambda: self._choose_document_molecule_inchi_file_export(
				ferrum_chem.InchiModeV1.fixed_hydrogen,
			),
		)
		menu.addAction(self._export_fixed_h_inchi_file_action)
		self._cancel_molecule_export_action = PySide6.QtGui.QAction(
			self.tr("Cancel Molecule Export"), self,
		)
		self._cancel_molecule_export_action.triggered.connect(
			self._cancel_document_molecule_export,
		)
		menu.addAction(self._cancel_molecule_export_action)

	#============================================
	def _molecule_export_busy(self) -> bool:
		"""Return whether one native export worker remains live."""
		return self._molecule_export_intent is not None

	#============================================
	def _choose_document_molecule_inchi_export(self, mode: object) -> None:
		"""Choose one durable molecule and begin its native export."""
		selected = self._choose_document_molecule_inchi_target("Export InChI")
		if selected is None:
			return
		_tab, molecule_id = selected
		self.start_document_molecule_inchi_export(molecule_id, mode)

	#============================================
	def _choose_document_molecule_inchi_target(
			self, title: str) -> tuple[object, str] | None:
		"""Choose one exact durable molecule without starting native work."""
		tab = self._active_native_tab()
		if tab is None:
			return None
		choices = tab.durable_molecule_choices()
		if not choices:
			self._show_native_file_warning(
				"Native InChI Export Unavailable",
				"This document has no durable molecule that Rust can export.",
			)
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
			self._show_native_file_warning(
				"Native InChI Export Unavailable",
				"The chosen molecule changed while choosing a destination. "
				"Choose Export InChI File again for the current document.",
			)
			return
		if not self.start_document_molecule_inchi_export(
				capture.molecule_id, mode, destination):
			self._show_native_file_warning(
				"Native InChI Export Unavailable",
				"Another native operation started while choosing the destination. "
				"Choose Export InChI File again after it finishes.",
			)

	#============================================
	def _normalize_document_molecule_inchi_path(self, selected_path: str) -> str | None:
		"""Apply the closed .inchi file policy without writing or adopting the path."""
		path = pathlib.Path(selected_path)
		if not path.suffix:
			path = path.with_suffix(".inchi")
		elif path.suffix.lower() != ".inchi":
			self._show_native_file_warning(
				"Unsupported InChI Export Format",
				"Ferrum InChI files must use the .inchi extension.",
			)
			return None
		return os.path.abspath(str(path))

	#============================================
	def _inchi_file_capture_is_current(self, capture: _MoleculeInchiFileCapture) -> bool:
		"""Reauthenticate the exact root and tab after the file dialog."""
		tab = capture.tab
		if (
			self._active_native_tab() is not tab
			or self._native_tabs_by_page.get(tab) is not tab
			or tab._disposed
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
			ferrum_qt.native.ferrum_native_molecule_inspection.
			selected_durable_molecule_address(tab)
		)
		if address is None:
			self._show_native_file_warning(
				"Native SMILES Export Unavailable",
				"Select atoms or bonds from exactly one durable molecule.",
			)
			return
		self.start_document_molecule_smiles_export(address.molecule_id)

	#============================================
	def _choose_document_molecule_smiles_file_export(self) -> None:
		"""Choose a destination for one exact selected canonical SMILES receipt."""
		tab = self._active_native_tab()
		if tab is None:
			return
		address = (
			ferrum_qt.native.ferrum_native_molecule_inspection.
			selected_durable_molecule_address(tab)
		)
		if address is None:
			self._show_native_file_warning(
				"Native SMILES Export Unavailable",
				"Select atoms or bonds from exactly one durable molecule.",
			)
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
			self._show_native_file_warning(
				"Native SMILES Export Unavailable",
				"The selected molecule changed while choosing a destination. "
				"Choose Export SMILES File again for the current selection.",
			)
			return
		if not self.start_document_molecule_smiles_export(
				capture.molecule_id, destination):
			self._show_native_file_warning(
				"Native SMILES Export Unavailable",
				"Another native operation started while choosing the destination. "
				"Choose Export SMILES File again after it finishes.",
			)

	#============================================
	def _normalize_document_molecule_smiles_path(self, selected_path: str) -> str | None:
		"""Apply the closed .smi file policy without writing or adopting the path."""
		path = pathlib.Path(selected_path)
		if not path.suffix:
			path = path.with_suffix(".smi")
		elif path.suffix.lower() != ".smi":
			self._show_native_file_warning(
				"Unsupported SMILES Export Format",
				"Ferrum SMILES files must use the .smi extension.",
			)
			return None
		return os.path.abspath(str(path))

	#============================================
	def _smiles_file_capture_is_current(self, capture: _MoleculeSmilesFileCapture) -> bool:
		"""Reauthenticate the exact active selection after the file dialog."""
		tab = capture.tab
		if (
			self._active_native_tab() is not tab
			or self._native_tabs_by_page.get(tab) is not tab
			or tab._disposed
			or tab.requires_refresh
		):
			return False
		address = (
			ferrum_qt.native.ferrum_native_molecule_inspection.
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
		"""Start one exact-revision Rust-native InChI export."""
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("native InChI export requires a durable molecule selector")
		if type(mode) is not ferrum_chem.InchiModeV1:
			raise TypeError("native InChI export requires an exact Ferrum mode")
		if destination is not None and (
			type(destination) is not str or not os.path.isabs(destination)
		):
			raise ValueError("native InChI file export requires an absolute destination")
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
		"""Start one exact-revision Rust-native canonical SMILES export."""
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("native SMILES export requires a durable molecule selector")
		if destination is not None and (
			type(destination) is not str or not os.path.isabs(destination)
		):
			raise ValueError("native SMILES file export requires an absolute destination")
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
			self._show_native_file_warning("Native SMILES Export Error", str(exc))
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
		"""Return one export only while its exact source tab remains current."""
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
			or tab._disposed
			or tab.requires_refresh
		):
			return None
		if kind == _SMILES_EXPORT:
			address = (
				ferrum_qt.native.ferrum_native_molecule_inspection.
				selected_durable_molecule_address(tab)
			)
			if address is None or address.molecule_id != intent.molecule_id:
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
		if type(result) is not ferrum_chem.DocumentMoleculeInchiV1:
			self._show_native_file_warning(
				"Native InChI Export Error", "Ferrum returned an unexpected export value.",
			)
			return
		if (
			result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or result.molecule_id != intent.molecule_id
			or result.mode is not intent.mode
		):
			self._show_stale_molecule_export(_INCHI_EXPORT)
			return
		if intent.destination is not None:
			self._publish_document_molecule_inchi_file(result, intent.destination)
			return
		PySide6.QtWidgets.QApplication.clipboard().setText(result.inchi)
		PySide6.QtWidgets.QMessageBox.information(
			self, self.tr("Ferrum InChI Export"),
			self.tr("InChI copied to the clipboard:\n\n{0}").format(result.inchi),
		)

	#============================================
	def _publish_document_molecule_inchi_file(
			self, receipt: object, destination: str) -> None:
		"""Publish one verified InChI receipt through Rust's artifact writer."""
		try:
			publication = ferrum_chem.publish_document_molecule_inchi_v1(
				receipt, destination,
			)
		except Exception as exc:
			self._report_molecule_file_publication_error("InChI", destination, exc)
			return
		if type(publication) is not ferrum_chem.DocumentMoleculeInchiPublicationV1:
			self._show_native_file_warning(
				"Native InChI File Export Error",
				"Ferrum returned an unexpected publication value. Inspect the destination "
				"because Rust may already have written it.",
			)
			return
		if publication.directory_entry_confirmed:
			self.statusBar().showMessage(
				self.tr("InChI file exported: %s") % destination, 5000,
			)
			return
		self._show_native_file_warning(
			"InChI File Durability Unconfirmed",
			"The exact InChI file is present at %s, but directory-entry durability "
			"needs verification. Inspect the destination before relying on it."
			% destination,
		)

	#============================================
	def _on_document_molecule_smiles_exported(self, worker: object, result: object) -> None:
		"""Copy one canonical SMILES only while every source fact remains exact."""
		intent = self._current_molecule_export_intent(worker, _SMILES_EXPORT)
		if intent is None:
			self._show_stale_molecule_export(_SMILES_EXPORT)
			return
		if type(result) is not ferrum_chem.DocumentMoleculeSmilesV1:
			self._show_native_file_warning(
				"Native SMILES Export Error", "Ferrum returned an unexpected export value.",
			)
			return
		if (
			result.schema != _SMILES_SCHEMA
			or result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or result.molecule_id != intent.molecule_id
			or result.profile != _SMILES_PROFILE
		):
			self._show_stale_molecule_export(_SMILES_EXPORT)
			return
		if intent.destination is not None:
			self._publish_document_molecule_smiles_file(result, intent.destination)
			return
		PySide6.QtWidgets.QApplication.clipboard().setText(result.smiles)
		self._show_document_molecule_smiles(result.smiles)

	#============================================
	def _publish_document_molecule_smiles_file(
			self, receipt: object, destination: str) -> None:
		"""Publish one verified receipt through Rust's secure artifact writer."""
		try:
			publication = ferrum_chem.publish_document_molecule_smiles_v1(
				receipt, destination,
			)
		except Exception as exc:
			self._report_molecule_file_publication_error("SMILES", destination, exc)
			return
		if type(publication) is not ferrum_chem.DocumentMoleculeSmilesPublicationV1:
			self._show_native_file_warning(
				"Native SMILES File Export Error",
				"Ferrum returned an unexpected publication value. Inspect the destination "
				"because Rust may already have written it.",
			)
			return
		if publication.directory_entry_confirmed:
			self.statusBar().showMessage(
				self.tr("SMILES file exported: %s") % destination, 5000,
			)
			return
		self._show_native_file_warning(
			"SMILES File Durability Unconfirmed",
			"The exact SMILES file is present at %s, but directory-entry durability "
			"needs verification. Inspect the destination before relying on it."
			% destination,
		)

	#============================================
	def _report_molecule_file_publication_error(
			self, label: str, destination: str, exc: Exception) -> None:
		"""Describe each publisher outcome without claiming a completed export."""
		if type(exc) is ferrum_chem.PublicationPossiblyCompletedError:
			title = f"{label} File Export Possibly Completed"
			message = (
				"Rust could not confirm whether publication completed at %s. Verify the "
				"destination before relying on it.\n\n%s" % (destination, exc)
			)
		elif type(exc) is ferrum_chem.PublicationNotStartedError:
			title = f"{label} File Export Not Started"
			message = "Rust did not publish a %s file to %s.\n\n%s" % (
				label, destination, exc,
			)
		elif type(exc) is ferrum_chem.InvalidDestinationError:
			title = f"{label} File Destination Rejected"
			message = "Rust rejected %s. Choose a different destination.\n\n%s" % (
				destination, exc,
			)
		else:
			title = f"Native {label} File Export Error"
			message = "Could not export %s to %s:\n%s" % (label, destination, exc)
		self._show_native_file_warning(title, message)

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
		label = "InChI" if intent.kind == _INCHI_EXPORT else "SMILES"
		self._show_native_file_warning(
			f"Native {label} Export Error", getattr(failure, "message", str(failure)),
		)

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
		"""Invalidate export delivery while native teardown finishes normally."""
		intent = self._molecule_export_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(
			self.tr("Cancelling molecule export; waiting for native work to finish..."), 0,
		)
		self._refresh_actions()

	#============================================
	def _refresh_molecule_export_actions(
			self, active: bool, pending: bool, busy_elsewhere: bool) -> None:
		"""Apply host reachability to native molecule exports."""
		can_start = active and not pending and not busy_elsewhere and not self._molecule_export_busy()
		tab = self._active_native_tab() if can_start else None
		address = None if tab is None else (
			ferrum_qt.native.ferrum_native_molecule_inspection.
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
		self._show_native_file_warning(
			"Native Molecule Export Still Running",
			"Cancel the molecule export and wait for native work before closing.",
		)
		return True

	#============================================
	def _cancel_molecule_export_for_close(self) -> bool:
		"""Cancel live delivery and tell the host to ignore this close attempt."""
		if self._molecule_export_intent is None:
			return False
		self._cancel_document_molecule_export()
		self._show_native_file_warning(
			"Native Molecule Export Still Running",
			"Ferrum cancelled delivery; close again after native work finishes.",
		)
		return True

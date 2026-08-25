"""Revision-bound molecule imports for the standalone Ferrum window."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.bridge.insertion_placement
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.inchi_import
import ferrum_qt.ferrum.molblock_import
import ferrum_qt.ferrum.peptide_import
import ferrum_qt.ferrum.sdf_import
import ferrum_qt.ferrum.smiles_import


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _MoleculeImportIntent:
	"""One source-tab generation plus one handle-free preparation worker."""

	tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
	revision: int
	digest: str
	worker: PySide6.QtCore.QThread


#============================================
#============================================
class _MoleculeImportDeliveryRelay(PySide6.QtCore.QObject):
	"""Deliver worker outcomes to the owning window on the Qt thread."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the one window that owns all molecule-import intents."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_smiles_prepared(self, molecule: object) -> None:
		"""Forward one SMILES result with its exact emitting worker."""
		self._owner._on_smiles_prepared(self.sender(), molecule)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_inchi_prepared(self, molecule: object) -> None:
		"""Forward one InChI result with its exact emitting worker."""
		self._owner._on_inchi_prepared(self.sender(), molecule)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_molblock_prepared(self, molecule: object) -> None:
		"""Forward one molfile result with its exact emitting worker."""
		self._owner._on_molblock_prepared(self.sender(), molecule)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_sdf_prepared(self, batch: object) -> None:
		"""Forward one SDF batch with its exact emitting worker."""
		self._owner._on_sdf_prepared(self.sender(), batch)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_peptide_prepared(self, molecule: object) -> None:
		"""Forward one supported peptide sequence result with its emitting worker."""
		self._owner._on_peptide_prepared(self.sender(), molecule)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_smiles_failed(self, failure: object) -> None:
		"""Forward one SMILES failure with its exact emitting worker."""
		self._owner._on_smiles_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_inchi_failed(self, failure: object) -> None:
		"""Forward one InChI failure with its exact emitting worker."""
		self._owner._on_inchi_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_molblock_failed(self, failure: object) -> None:
		"""Forward one molfile failure with its exact emitting worker."""
		self._owner._on_molblock_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_sdf_failed(self, failure: object) -> None:
		"""Forward one SDF failure with its exact emitting worker."""
		self._owner._on_sdf_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_peptide_failed(self, failure: object) -> None:
		"""Forward one supported peptide sequence failure with its emitting worker."""
		self._owner._on_peptide_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_smiles_finished(self) -> None:
		"""Release the exact SMILES worker that has stopped."""
		self._owner._finish_import("smiles", self.sender())

	#============================================
	@PySide6.QtCore.Slot()
	def on_inchi_finished(self) -> None:
		"""Release the exact InChI worker that has stopped."""
		self._owner._finish_import("inchi", self.sender())

	#============================================
	@PySide6.QtCore.Slot()
	def on_molblock_finished(self) -> None:
		"""Release the exact molfile worker that has stopped."""
		self._owner._finish_import("molblock", self.sender())

	#============================================
	@PySide6.QtCore.Slot()
	def on_sdf_finished(self) -> None:
		"""Release the exact SDF worker that has stopped."""
		self._owner._finish_import("sdf", self.sender())

	#============================================
	@PySide6.QtCore.Slot()
	def on_peptide_finished(self) -> None:
		"""Release the exact supported peptide sequence worker that has stopped."""
		self._owner._finish_import("peptide", self.sender())

#============================================
class FerrumNativeMoleculeImportsMixin:
	"""Own asynchronous text and bounded local-file molecule imports.

	The host owns tabs, status, warnings, and action refresh. This mixin owns only
	immutable import intent and worker delivery; the selected tab's Rust session
	remains the sole mutation authority.
	"""

	#============================================
	def _initialize_molecule_imports(self) -> None:
		"""Initialize the mutually exclusive Ferrum import intents."""
		self._smiles_import_intent: _MoleculeImportIntent | None = None
		self._inchi_import_intent: _MoleculeImportIntent | None = None
		self._molblock_import_intent: _MoleculeImportIntent | None = None
		self._sdf_import_intent: _MoleculeImportIntent | None = None
		self._peptide_import_intent: _MoleculeImportIntent | None = None
		self._molecule_import_relay = _MoleculeImportDeliveryRelay(self)

	#============================================
	def _build_molecule_import_actions(self,
			menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add Ferrum molecule import and cancellation actions."""
		self._import_smiles_action = PySide6.QtGui.QAction(self.tr("Import SMILES"), self)
		self._import_smiles_action.triggered.connect(self._on_import_smiles)
		menu.addAction(self._import_smiles_action)
		self._import_inchi_action = PySide6.QtGui.QAction(self.tr("Import InChI"), self)
		self._import_inchi_action.triggered.connect(self._on_import_inchi)
		menu.addAction(self._import_inchi_action)
		self._import_molblock_action = PySide6.QtGui.QAction(
			self.tr("Import V2000/V3000 Molfile"), self,
		)
		self._import_molblock_action.triggered.connect(self._on_import_molblock)
		menu.addAction(self._import_molblock_action)
		self._import_sdf_action = PySide6.QtGui.QAction(
			self.tr("Import SDF Records"), self,
		)
		self._import_sdf_action.triggered.connect(self._on_import_sdf)
		menu.addAction(self._import_sdf_action)
		self._import_peptide_action = PySide6.QtGui.QAction(
			self.tr("Import Supported Peptide Sequence..."), self,
		)
		self._import_peptide_action.triggered.connect(self._on_import_peptide)
		menu.addAction(self._import_peptide_action)
		self._cancel_smiles_action = PySide6.QtGui.QAction(
			self.tr("Cancel SMILES Import"), self,
		)
		self._cancel_smiles_action.triggered.connect(self._cancel_smiles_import)
		menu.addAction(self._cancel_smiles_action)
		self._cancel_inchi_action = PySide6.QtGui.QAction(
			self.tr("Cancel InChI Import"), self,
		)
		self._cancel_inchi_action.triggered.connect(self._cancel_inchi_import)
		menu.addAction(self._cancel_inchi_action)
		self._cancel_molblock_action = PySide6.QtGui.QAction(
			self.tr("Cancel Molfile Import"), self,
		)
		self._cancel_molblock_action.triggered.connect(self._cancel_molblock_import)
		menu.addAction(self._cancel_molblock_action)
		self._cancel_sdf_action = PySide6.QtGui.QAction(
			self.tr("Cancel SDF Import"), self,
		)
		self._cancel_sdf_action.triggered.connect(self._cancel_sdf_import)
		menu.addAction(self._cancel_sdf_action)
		self._cancel_peptide_action = PySide6.QtGui.QAction(
			self.tr("Cancel Supported Peptide Sequence Import"), self,
		)
		self._cancel_peptide_action.triggered.connect(self._cancel_peptide_import)
		menu.addAction(self._cancel_peptide_action)

	#============================================
	def _molecule_import_busy(self) -> bool:
		"""Return whether any Ferrum import has a live worker intent."""
		return any(intent is not None for intent in (
			self._smiles_import_intent,
			self._inchi_import_intent,
			self._molblock_import_intent,
			self._sdf_import_intent,
			self._peptide_import_intent,
		))

	#============================================
	def _on_import_smiles(self) -> None:
		"""Collect SMILES text and start its Ferrum import route."""
		smiles, accepted = PySide6.QtWidgets.QInputDialog.getText(
			self, self.tr("Import SMILES"), self.tr("SMILES:"),
		)
		if accepted and smiles.strip():
			self.start_smiles_import(smiles)

	#============================================
	def start_smiles_import(self, smiles: str) -> bool:
		"""Start one Ferrum SMILES preparation for the active document."""
		if type(smiles) is not str or not smiles.strip():
			raise ValueError("Ferrum SMILES import requires nonblank text")
		smiles = smiles.strip()
		if (
			self._molecule_import_busy()
			or getattr(self, "_molecule_export_intent", None) is not None
			or self._coordinate_generation_intent is not None
		):
			return False
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return False
		try:
			placement = ferrum_qt.bridge.insertion_placement.capture_insertion_placement_v1(tab)
			worker = (
				ferrum_qt.ferrum.smiles_import.
				FerrumNativeSmilesPreparationWorker(smiles, placement)
			)
		except Exception as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return False
		self._smiles_import_intent = self._start_import_intent(tab, worker)
		worker.prepared.connect(
			self._molecule_import_relay.on_smiles_prepared,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.failed.connect(
			self._molecule_import_relay.on_smiles_failed,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.finished.connect(
			self._molecule_import_relay.on_smiles_finished,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		self.statusBar().showMessage(self.tr("Preparing SMILES with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _on_import_inchi(self) -> None:
		"""Collect InChI text and start its Ferrum import route."""
		inchi, accepted = PySide6.QtWidgets.QInputDialog.getText(
			self, self.tr("Import InChI"), self.tr("InChI:"),
		)
		if accepted and inchi.strip():
			self.start_inchi_import(inchi)

	#============================================
	def start_inchi_import(self, inchi: str) -> bool:
		"""Start one Ferrum InChI preparation for the active document."""
		if type(inchi) is not str or not inchi.strip():
			raise ValueError("Ferrum InChI import requires nonblank text")
		inchi = inchi.strip()
		if (
			self._molecule_import_busy()
			or getattr(self, "_molecule_export_intent", None) is not None
			or self._coordinate_generation_intent is not None
		):
			return False
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return False
		try:
			placement = ferrum_qt.bridge.insertion_placement.capture_insertion_placement_v1(tab)
			worker = (
				ferrum_qt.ferrum.inchi_import.
				FerrumNativeInchiPreparationWorker(inchi, placement)
			)
		except Exception as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return False
		self._inchi_import_intent = self._start_import_intent(tab, worker)
		worker.prepared.connect(
			self._molecule_import_relay.on_inchi_prepared,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.failed.connect(
			self._molecule_import_relay.on_inchi_failed,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.finished.connect(
			self._molecule_import_relay.on_inchi_finished,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		self.statusBar().showMessage(self.tr("Preparing InChI with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _on_import_molblock(self) -> None:
		"""Choose one local molfile without reading it in Python."""
		path = PySide6.QtWidgets.QFileDialog.getOpenFileName(
			self,
			self.tr("Import Rust Molfile"),
			"",
			self.tr("MDL Molfile (*.mol *.molfile)"),
		)[0]
		if path:
			self.start_molblock_import(path)

	#============================================
	def start_molblock_import(self, path: str) -> bool:
		"""Start one Rust-bounded local V2000/V3000 preparation."""
		if type(path) is not str or not path:
			raise ValueError("Ferrum molfile import requires a nonempty path")
		if (
			self._molecule_import_busy()
			or getattr(self, "_molecule_export_intent", None) is not None
			or self._coordinate_generation_intent is not None
		):
			return False
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return False
		try:
			placement = ferrum_qt.bridge.insertion_placement.capture_insertion_placement_v1(tab)
			worker = (
				ferrum_qt.ferrum.molblock_import.
				FerrumNativeMolblockPreparationWorker(path, placement)
			)
		except Exception as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return False
		self._molblock_import_intent = self._start_import_intent(tab, worker)
		worker.prepared.connect(
			self._molecule_import_relay.on_molblock_prepared,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.failed.connect(
			self._molecule_import_relay.on_molblock_failed,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.finished.connect(
			self._molecule_import_relay.on_molblock_finished,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		self.statusBar().showMessage(self.tr("Reading bounded molfile with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _on_import_sdf(self) -> None:
		"""Choose one local SDF without reading it in Python."""
		path = PySide6.QtWidgets.QFileDialog.getOpenFileName(
			self,
			self.tr("Import Rust SDF Records"),
			"",
			self.tr("Structure Data File (*.sdf *.sd)"),
		)[0]
		if path:
			self.start_sdf_import(path)

	#============================================
	def start_sdf_import(self, path: str) -> bool:
		"""Start one Rust-bounded local multi-record SDF preparation."""
		if type(path) is not str or not path:
			raise ValueError("Ferrum SDF import requires a nonempty path")
		if (
			self._molecule_import_busy()
			or getattr(self, "_molecule_export_intent", None) is not None
			or self._coordinate_generation_intent is not None
		):
			return False
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return False
		try:
			placement = ferrum_qt.bridge.insertion_placement.capture_insertion_placement_v1(tab)
			route_handle = self._sdf_import_route_handle()
			worker = (
				ferrum_qt.ferrum.sdf_import.
				FerrumNativeSdfPreparationWorker(path, placement, route_handle)
			)
		except Exception as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return False
		self._sdf_import_intent = self._start_import_intent(tab, worker)
		worker.prepared.connect(
			self._molecule_import_relay.on_sdf_prepared,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.failed.connect(
			self._molecule_import_relay.on_sdf_failed,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.finished.connect(
			self._molecule_import_relay.on_sdf_finished,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		self.statusBar().showMessage(self.tr("Reading bounded SDF with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _sdf_import_route_handle(self) -> object:
		"""Return the registry-issued route handle for the selected SDF source."""
		return self._local_ingress_registry.interchange_route_handle_for_suffix(".sdf")

	#============================================
	def _on_import_peptide(self) -> None:
		"""Collect strict template text without trimming or normalizing it."""
		sequence, accepted = PySide6.QtWidgets.QInputDialog.getText(
			self,
			self.tr("Import Supported Peptide Sequence"),
			self.tr(
				"Uppercase, no spaces; supported: ACDEFGIKLMNQRSTVY; H/P/W unsupported:",
			),
		)
		if accepted:
			self.start_supported_peptide_import(sequence)

	#============================================
	def start_supported_peptide_import(self, sequence: str) -> bool:
		"""Start strict native peptide preparation for the active document."""
		if type(sequence) is not str:
			raise TypeError("Ferrum peptide import requires exact text")
		if (
			self._molecule_import_busy()
			or getattr(self, "_molecule_export_intent", None) is not None
			or self._coordinate_generation_intent is not None
		):
			return False
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return False
		try:
			placement = ferrum_qt.bridge.insertion_placement.capture_insertion_placement_v1(tab)
			worker = self._create_peptide_preparation_worker(sequence, placement)
		except Exception as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return False
		self._peptide_import_intent = self._start_import_intent(tab, worker)
		worker.prepared.connect(
			self._molecule_import_relay.on_peptide_prepared,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.failed.connect(
			self._molecule_import_relay.on_peptide_failed,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.finished.connect(
			self._molecule_import_relay.on_peptide_finished,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		self.statusBar().showMessage(
			self.tr("Preparing supported peptide sequence with native Ferrum..."), 0,
		)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _create_peptide_preparation_worker(self, sequence: str,
			placement: object) -> PySide6.QtCore.QThread:
		"""Create the one strict-template worker without interpreting its inputs."""
		worker = (
			ferrum_qt.ferrum.peptide_import.
			FerrumNativePeptidePreparationWorker(sequence, placement)
		)
		return worker

	#============================================
	def _start_import_intent(self, tab: object,
			worker: PySide6.QtCore.QThread) -> _MoleculeImportIntent:
		"""Capture one exact tab generation before a worker starts."""
		snapshot = tab.current_snapshot
		return _MoleculeImportIntent(tab, snapshot.revision, snapshot.digest, worker)

	#============================================
	def _on_smiles_prepared(self, worker: object, molecule: object) -> None:
		"""Commit one still-current SMILES result."""
		self._commit_prepared_import(
			self._smiles_import_intent, worker, molecule, "SMILES", "smiles_import",
		)

	#============================================
	def _on_inchi_prepared(self, worker: object, molecule: object) -> None:
		"""Commit one still-current InChI result."""
		self._commit_prepared_import(
			self._inchi_import_intent, worker, molecule, "InChI", "inchi_import",
		)

	#============================================
	def _on_molblock_prepared(self, worker: object, molecule: object) -> None:
		"""Commit one still-current bounded molfile result."""
		self._commit_prepared_import(
			self._molblock_import_intent, worker, molecule, "Molfile", "molfile_import",
		)

	#============================================
	def _on_sdf_prepared(self, worker: object, batch: object) -> None:
		"""Commit one still-current complete SDF batch."""
		self._commit_prepared_import(
			self._sdf_import_intent, worker, batch, "SDF", "sdf_import",
		)

	#============================================
	def _on_peptide_prepared(self, worker: object, molecule: object) -> None:
		"""Commit one still-current supported peptide sequence result."""
		self._commit_prepared_import(
			self._peptide_import_intent, worker, molecule, "Supported Peptide Sequence",
			"peptide_sequence_import",
		)

	#============================================
	def _commit_prepared_import(self, intent: _MoleculeImportIntent | None,
			worker: object, molecule: object, label: str,
			installation_kind: str) -> None:
		"""Commit only a worker result whose tab revision and digest remain current."""
		if intent is None or worker is not intent.worker:
			return
		if intent.worker.delivery_cancelled:
			return
		tab = intent.tab
		snapshot = tab.current_snapshot
		if (
			tab not in self._native_tabs_by_page
			or tab.requires_refresh
			or snapshot.revision != intent.revision
			or snapshot.digest != intent.digest
		):
			self.statusBar().showMessage(
				self.tr(f"Discarded stale {label} result; the source document changed."),
				5000,
			)
			return
		try:
			if label == "SDF":
				result = tab.insert_prepared_sdf_records(molecule)
			else:
				result = tab.insert_prepared_molecule(molecule)
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return
		message = (
			f"Imported {molecule.record_count} Ferrum SDF records."
			if label == "SDF" else
			"Imported one Ferrum molecule."
		)
		self.statusBar().showMessage(self.tr(message), 5000)
		self._refresh_actions()
		target = result.observation.snapshot
		record_count = molecule.record_count if label == "SDF" else 1
		self._publish_document_installation_v1(
			tab, installation_kind, intent.revision, intent.digest,
			target.revision, target.digest, record_count,
		)

	#============================================
	def _on_smiles_failed(self, worker: object, failure: object) -> None:
		"""Present one current SMILES preparation failure."""
		self._show_import_failure(
			self._smiles_import_intent, worker, failure, "SMILES",
		)

	#============================================
	def _on_inchi_failed(self, worker: object, failure: object) -> None:
		"""Present one current InChI preparation failure."""
		self._show_import_failure(
			self._inchi_import_intent, worker, failure, "InChI",
		)

	#============================================
	def _on_molblock_failed(self, worker: object, failure: object) -> None:
		"""Present one current bounded molfile preparation failure."""
		self._show_import_failure(
			self._molblock_import_intent, worker, failure, "Molfile",
		)

	#============================================
	def _on_sdf_failed(self, worker: object, failure: object) -> None:
		"""Present one current bounded SDF preparation failure."""
		self._show_import_failure(self._sdf_import_intent, worker, failure, "SDF")

	#============================================
	def _on_peptide_failed(self, worker: object, failure: object) -> None:
		"""Present one current supported peptide sequence preparation failure."""
		self._show_import_failure(
			self._peptide_import_intent, worker, failure, "Supported Peptide Sequence",
		)

	#============================================
	def _show_import_failure(self, intent: _MoleculeImportIntent | None,
			worker: object, failure: object, label: str) -> None:
		"""Show one failure only for its current noncancelled worker."""
		if intent is None or worker is not intent.worker:
			return
		if not intent.worker.delivery_cancelled:
			message = getattr(failure, "message", str(failure))
			self._show_edit_refusal(self._unavailable_edit_refusal(message))

	#============================================
	def _finish_import(self, kind: str, worker: object) -> None:
		"""Release one exact stopped worker after its delivery is complete."""
		attribute = f"_{kind}_import_intent"
		intent = getattr(self, attribute)
		if intent is None or worker is not intent.worker:
			return
		setattr(self, attribute, None)
		if intent.worker.delivery_cancelled:
			self.statusBar().showMessage(self.tr(f"{kind.capitalize()} import cancelled."), 5000)
		intent.worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _cancel_smiles_import(self) -> None:
		"""Invalidate pending SMILES delivery."""
		self._cancel_import(self._smiles_import_intent, "SMILES")

	#============================================
	def _cancel_inchi_import(self) -> None:
		"""Invalidate pending InChI delivery."""
		self._cancel_import(self._inchi_import_intent, "InChI")

	#============================================
	def _cancel_molblock_import(self) -> None:
		"""Invalidate pending molfile delivery."""
		self._cancel_import(self._molblock_import_intent, "Molfile")

	#============================================
	def _cancel_sdf_import(self) -> None:
		"""Invalidate pending SDF delivery."""
		self._cancel_import(self._sdf_import_intent, "SDF")

	#============================================
	def _cancel_peptide_import(self) -> None:
		"""Invalidate pending supported peptide sequence delivery."""
		self._cancel_import(self._peptide_import_intent, "Supported Peptide Sequence")

	#============================================
	def _cancel_import(self, intent: _MoleculeImportIntent | None, label: str) -> None:
		"""Invalidate one delivery while worker cleanup finishes normally."""
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(
			self.tr(f"Cancelling {label} delivery; waiting for the current operation to finish..."),
			0,
		)
		self._refresh_actions()

	#============================================
	def _refresh_molecule_import_actions(self, active: bool, pending: bool,
			busy_coordinates: bool) -> None:
		"""Apply the host action policy to both import routes."""
		busy_import = self._molecule_import_busy()
		can_start = active and not pending and not busy_coordinates and not busy_import
		self._import_smiles_action.setEnabled(can_start)
		self._import_inchi_action.setEnabled(can_start)
		self._import_molblock_action.setEnabled(can_start)
		self._import_sdf_action.setEnabled(can_start)
		self._import_peptide_action.setEnabled(can_start)
		self._cancel_smiles_action.setEnabled(
			self._smiles_import_intent is not None
			and not self._smiles_import_intent.worker.delivery_cancelled,
		)
		self._cancel_inchi_action.setEnabled(
			self._inchi_import_intent is not None
			and not self._inchi_import_intent.worker.delivery_cancelled,
		)
		self._cancel_molblock_action.setEnabled(
			self._molblock_import_intent is not None
			and not self._molblock_import_intent.worker.delivery_cancelled,
		)
		self._cancel_sdf_action.setEnabled(
			self._sdf_import_intent is not None
			and not self._sdf_import_intent.worker.delivery_cancelled,
		)
		self._cancel_peptide_action.setEnabled(
			self._peptide_import_intent is not None
			and not self._peptide_import_intent.worker.delivery_cancelled,
		)

	#============================================
	def _molecule_import_blocks_tab_close(self, tab: object) -> bool:
		"""Keep a tab alive while any Ferrum import still owns delivery."""
		for label, intent in (
			("SMILES", self._smiles_import_intent),
			("InChI", self._inchi_import_intent),
			("Molfile", self._molblock_import_intent),
			("SDF", self._sdf_import_intent),
			("Supported Peptide Sequence", self._peptide_import_intent),
		):
			if intent is not None and intent.tab is tab:
				self._show_edit_refusal(self._unavailable_edit_refusal(f"Cancel the {label} import and wait for the current operation before closing."))
				return True
		return False

	#============================================
	def _cancel_molecule_imports_for_close(self) -> bool:
		"""Cancel each live import intent and await its worker's normal release."""
		for label, intent, cancel in (
			("SMILES", self._smiles_import_intent, self._cancel_smiles_import),
			("InChI", self._inchi_import_intent, self._cancel_inchi_import),
			("Molfile", self._molblock_import_intent, self._cancel_molblock_import),
			("SDF", self._sdf_import_intent, self._cancel_sdf_import),
			("Supported Peptide Sequence", self._peptide_import_intent, self._cancel_peptide_import),
		):
			if intent is not None:
				cancel()
				self.statusBar().showMessage(
					self.tr(
						f"Ferrum cancelled {label} delivery; close again after it finishes."
					),
					0,
				)
				return True
		return False

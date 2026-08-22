"""Revision-bound selected-molecule SDF export for Ferrum tabs."""

# Standard Library
import dataclasses
import os
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_qt.ferrum.engine as engine

# local repo modules
import ferrum_qt.ferrum.molecule_inspection
from ferrum_qt.ferrum.molecule_exports import (
	_FerrumNativeMoleculeExportWorker,
	FerrumNativeMoleculeExportFailure,
)


_SDF_SCHEMA = "ferrum-document-molecule-sdf-v1"
_SDF_PROFILE = "document-xy-to-chemistry-x-minus-y-rust-sdf-envelope-v1"
_SDF_FILTER = "SDF files (*.sdf);;All Files (*)"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _SdfExportIntent:
	"""One exact selected root and its handle-free SDF worker."""

	tab: object
	revision: int
	digest: str
	molecule_id: str
	version: object
	destination: str
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _SdfFileCapture:
	"""One exact selection retained across the destination dialog."""

	tab: object
	revision: int
	digest: str
	molecule_id: str
	version: object


#============================================
class FerrumNativeSdfExportWorker(_FerrumNativeMoleculeExportWorker):
	"""Export one validated document molecule as one complete SDF record."""

	#============================================
	def __init__(
			self, observation: object, molecule_id: str, version: object,
			) -> None:
		"""Validate and freeze one exact document-SDF request."""
		if type(observation) is not engine.SessionDocumentObservationV1:
			raise TypeError("Ferrum SDF export requires exact Ferrum observation")
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("Ferrum SDF export requires a durable molecule selector")
		if type(version) is not engine.MolblockVersionV1:
			raise TypeError("Ferrum SDF export requires an exact Ferrum version")
		snapshot = observation.snapshot
		super().__init__(
			engine.export_document_molecule_sdf_v1,
			(observation, snapshot.revision, snapshot.digest, molecule_id, version),
		)


#============================================
class _SdfExportDeliveryRelay(PySide6.QtCore.QObject):
	"""Route SDF worker signals to their owning window on the Qt thread."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the window that owns the export intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_exported(self, result: object) -> None:
		"""Forward one result with its exact emitting worker."""
		self._owner._on_document_sdf_exported(self.sender(), result)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward one failure with its exact emitting worker."""
		self._owner._on_document_sdf_export_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the exact stopped worker."""
		self._owner._on_document_sdf_export_finished(self.sender())


#============================================
class FerrumNativeSdfExportMixin:
	"""Own asynchronous selected-molecule SDF publication."""

	#============================================
	def _initialize_sdf_exports(self) -> None:
		"""Initialize the one Ferrum SDF export intent."""
		self._sdf_export_intent: _SdfExportIntent | None = None
		self._sdf_export_relay = _SdfExportDeliveryRelay(self)

	#============================================
	def _build_sdf_export_actions(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the two explicit SDF Molfile-syntax actions."""
		self._export_sdf_v2000_action = PySide6.QtGui.QAction(
			self.tr("Export SDF Record V2000..."), self,
		)
		self._export_sdf_v2000_action.triggered.connect(
			self._choose_document_sdf_v2000_export,
		)
		menu.addAction(self._export_sdf_v2000_action)
		self._export_sdf_v3000_action = PySide6.QtGui.QAction(
			self.tr("Export SDF Record V3000..."), self,
		)
		self._export_sdf_v3000_action.triggered.connect(
			self._choose_document_sdf_v3000_export,
		)
		menu.addAction(self._export_sdf_v3000_action)

	#============================================
	def _choose_document_sdf_v2000_export(self) -> None:
		"""Choose one exact selected root for V2000 SDF publication."""
		self._choose_document_sdf_export(engine.MolblockVersionV1.v2000)

	#============================================
	def _choose_document_sdf_v3000_export(self) -> None:
		"""Choose one exact selected root for V3000 SDF publication."""
		self._choose_document_sdf_export(engine.MolblockVersionV1.v3000)

	#============================================
	def _choose_document_sdf_export(self, version: object) -> None:
		"""Choose one destination for the exact current selected root."""
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
		capture = _SdfFileCapture(
			tab, snapshot.revision, snapshot.digest, address.molecule_id, version,
		)
		label = self._sdf_version_label(version)
		selected_path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Export SDF Record %s" % label), "", self.tr(_SDF_FILTER),
		)[0]
		if not selected_path:
			return
		destination = self._normalize_document_sdf_path(selected_path)
		if destination is None:
			return
		if not self._sdf_file_capture_is_current(capture):
			self._show_edit_refusal(self._unavailable_edit_refusal("The selected molecule changed while choosing a destination. "
				"Choose the SDF export again for the current selection."))
			return
		if not self.start_document_sdf_export(
				capture.molecule_id, capture.version, destination,
				):
			self._show_edit_refusal(self._unavailable_edit_refusal("Another operation started while choosing the destination. "
				"Choose the SDF export again after it finishes."))

	#============================================
	def _normalize_document_sdf_path(self, selected_path: str) -> str | None:
		"""Apply the closed .sdf policy without writing or adopting the path."""
		path = pathlib.Path(selected_path)
		if not path.suffix:
			path = path.with_suffix(".sdf")
		elif path.suffix.lower() != ".sdf":
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum SDF exports must use the .sdf extension."))
			return None
		return os.path.abspath(str(path))

	#============================================
	def _sdf_file_capture_is_current(self, capture: _SdfFileCapture) -> bool:
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
	def start_document_sdf_export(
			self, molecule_id: str, version: object, destination: str,
			) -> bool:
		"""Start one exact-revision Ferrum SDF export."""
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("Ferrum SDF export requires a durable molecule selector")
		if type(version) is not engine.MolblockVersionV1:
			raise TypeError("Ferrum SDF export requires an exact Ferrum version")
		if type(destination) is not str or not os.path.isabs(destination):
			raise ValueError("Ferrum SDF export requires an absolute destination")
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
			projection_matches = tuple(
				molecule for molecule in observation.projection.molecules
				if molecule.id == molecule_id
			)
			if len(projection_matches) != 1:
				raise ValueError("Ferrum SDF export requires one exact projection root")
			worker = FerrumNativeSdfExportWorker(observation, molecule_id, version)
		except Exception as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return False
		snapshot = tab.current_snapshot
		self._sdf_export_intent = _SdfExportIntent(
			tab, snapshot.revision, snapshot.digest, molecule_id,
			version, destination, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.exported.connect(self._sdf_export_relay.on_exported, connection)
		worker.failed.connect(self._sdf_export_relay.on_failed, connection)
		worker.finished.connect(self._sdf_export_relay.on_finished, connection)
		self.statusBar().showMessage(
			self.tr("Preparing SDF record %s with Ferrum Rust...")
			% self._sdf_version_label(version),
			0,
		)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _current_sdf_export_intent(self, worker: object) -> _SdfExportIntent | None:
		"""Return one export only while every source fact remains exact."""
		intent = self._sdf_export_intent
		if (
			intent is None
			or worker is not intent.worker
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
		address = (
			ferrum_qt.ferrum.molecule_inspection.
			selected_durable_molecule_address(tab)
		)
		snapshot = tab.current_snapshot
		if (
			address is None
			or address.molecule_id != intent.molecule_id
			or snapshot.revision != intent.revision
			or snapshot.digest != intent.digest
		):
			return None
		return intent

	#============================================
	def _on_document_sdf_exported(self, worker: object, result: object) -> None:
		"""Publish one receipt only while every source fact remains exact."""
		intent = self._current_sdf_export_intent(worker)
		if intent is None:
			self._show_stale_sdf_export()
			return
		if type(result) is not engine.DocumentMoleculeSdfV1:
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum returned an unexpected export value."))
			return
		if (
			result.schema != _SDF_SCHEMA
			or result.profile != _SDF_PROFILE
			or result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or result.molecule_id != intent.molecule_id
			or result.version is not intent.version
		):
			self._show_stale_sdf_export()
			return
		self._publish_document_sdf(result, intent)

	#============================================
	def _publish_document_sdf(
			self, receipt: object, intent: _SdfExportIntent,
			) -> None:
		"""Publish one verified receipt through Rust's artifact writer."""
		try:
			publication = engine.publish_document_molecule_sdf_v1(
				receipt, intent.destination,
			)
		except Exception as exc:
			self._report_molecule_file_publication_error(
				"SDF", intent.destination, exc,
			)
			return
		if type(publication) is not engine.DocumentMoleculeSdfPublicationV1:
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum returned an unexpected publication value. Inspect the destination "
				"because Rust may already have written it."))
			return
		label = self._sdf_version_label(intent.version)
		if publication.directory_entry_confirmed:
			self.statusBar().showMessage(
				self.tr("SDF record %s exported: %s") % (label, intent.destination),
				5000,
			)
			return
		self._show_edit_refusal(self._unavailable_edit_refusal("The exact SDF record is present at %s, but directory-entry durability "
			"needs verification. Inspect the destination before relying on it."
			% intent.destination))

	#============================================
	def _on_document_sdf_export_failed(self, worker: object, failure: object) -> None:
		"""Show one current closed worker failure or refuse a protocol violation."""
		if self._current_sdf_export_intent(worker) is None:
			self._show_stale_sdf_export()
			return
		if type(failure) is not FerrumNativeMoleculeExportFailure:
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum returned an unexpected export failure."))
			return
		self._show_edit_refusal(self._unavailable_edit_refusal(failure.message))

	#============================================
	def _on_document_sdf_export_finished(self, worker: object) -> None:
		"""Release one stopped worker and restore action reachability."""
		intent = self._sdf_export_intent
		if intent is None or worker is not intent.worker:
			return
		self._sdf_export_intent = None
		intent.worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _show_stale_sdf_export(self) -> None:
		"""Report that an old result was deliberately withheld."""
		self.statusBar().showMessage(
			self.tr("Discarded stale SDF export; the source selection changed."),
			5000,
		)

	#============================================
	def _sdf_version_label(self, version: object) -> str:
		"""Return the user-visible closed Molfile syntax label."""
		if version is engine.MolblockVersionV1.v2000:
			return "V2000"
		if version is engine.MolblockVersionV1.v3000:
			return "V3000"
		raise TypeError("Ferrum SDF export requires an exact Ferrum version")

	#============================================
	def _molecule_export_busy(self) -> bool:
		"""Include SDF work in the shared molecule-export exclusion."""
		return self._sdf_export_intent is not None or super()._molecule_export_busy()

	#============================================
	def _cancel_document_molecule_export(self) -> None:
		"""Cancel the active SDF delivery or delegate to other exporters."""
		intent = self._sdf_export_intent
		if intent is None:
			super()._cancel_document_molecule_export()
			return
		if intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(
			self.tr("Cancelling SDF export; waiting for the current operation to finish..."), 0,
		)
		self._refresh_actions()

	#============================================
	def _refresh_sdf_export_actions(
			self, active: bool, pending: bool, busy_elsewhere: bool,
			) -> None:
		"""Apply host and exact-selection reachability to both syntaxes."""
		can_start = (
			active and not pending and not busy_elsewhere and not self._molecule_export_busy()
		)
		tab = self._active_native_tab() if can_start else None
		address = None if tab is None else (
			ferrum_qt.ferrum.molecule_inspection.
			selected_durable_molecule_address(tab)
		)
		self._export_sdf_v2000_action.setEnabled(address is not None)
		self._export_sdf_v3000_action.setEnabled(address is not None)

	#============================================
	def _refresh_molecule_export_actions(
			self, active: bool, pending: bool, busy_elsewhere: bool,
			) -> None:
		"""Keep the shared cancel action accurate for an SDF worker."""
		super()._refresh_molecule_export_actions(active, pending, busy_elsewhere)
		intent = self._sdf_export_intent
		if intent is not None:
			self._cancel_molecule_export_action.setEnabled(
				not intent.worker.delivery_cancelled,
			)

	#============================================
	def _molecule_export_blocks_tab_close(self, tab: object) -> bool:
		"""Keep the SDF source tab alive through worker teardown."""
		intent = self._sdf_export_intent
		if intent is None or intent.tab is not tab:
			return super()._molecule_export_blocks_tab_close(tab)
		self._show_edit_refusal(self._unavailable_edit_refusal("Cancel the SDF export and wait for the current operation before closing."))
		return True

	#============================================
	def _cancel_molecule_export_for_close(self) -> bool:
		"""Invalidate SDF delivery and preserve the window through teardown."""
		if self._sdf_export_intent is None:
			return super()._cancel_molecule_export_for_close()
		self._cancel_document_molecule_export()
		self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum cancelled delivery; close again after the current operation finishes."))
		return True

"""Rust-owned selected-root multi-record SDF publication for Ferrum windows."""

# Standard Library
import dataclasses
import os
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.ferrum.molecule_inspection
from ferrum_qt.ferrum.molecule_exports import (
	FerrumNativeMoleculeExportFailure,
	_FerrumNativeMoleculeExportWorker,
)


_SDF_FILTER = "SDF files (*.sdf);;All Files (*)"


#============================================
def _multi_sdf_failure(error: Exception) -> object:
	"""Preserve typed Rust V2 refusals across the detached worker boundary."""
	if type(error) is engine.DocumentMoleculesSdfError:
		return error
	return FerrumNativeMoleculeExportFailure(type(error).__name__, str(error))


#============================================
def _has_same_multi_sdf_membership(
		expected: tuple[str, ...], observed: tuple[str, ...],
		) -> bool:
	"""Match selected roots as a distinct membership set, never an output order."""
	return (
		len(observed) == len(expected)
		and len(set(observed)) == len(observed)
		and set(observed) == set(expected)
	)


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _MultiSdfExportIntent:
	"""One exact selected-root export and its detached Rust worker."""

	tab: object
	revision: int
	digest: str
	molecule_ids: tuple[str, ...]
	version: object
	destination: str
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _MultiSdfFileCapture:
	"""One exact selected-root membership retained across a file dialog."""

	tab: object
	revision: int
	digest: str
	molecule_ids: tuple[str, ...]
	version: object


#============================================
class FerrumNativeMultiSdfExportWorker(_FerrumNativeMoleculeExportWorker):
	"""Ask Rust for one complete canonical multi-record SDF receipt."""

	#============================================
	def __init__(
			self, observation: object, molecule_ids: tuple[str, ...], version: object,
			) -> None:
		"""Freeze only the observation fence and selected-root membership."""
		if type(observation) is not engine.SessionDocumentObservationV1:
			raise TypeError("Ferrum multi-record SDF export requires exact Ferrum observation")
		if len(molecule_ids) < 2 or any(
			type(molecule_id) is not str or not molecule_id
			for molecule_id in molecule_ids
		):
			raise ValueError("Ferrum multi-record SDF export requires two durable molecule selectors")
		if len(set(molecule_ids)) != len(molecule_ids):
			raise ValueError("Ferrum multi-record SDF export requires distinct molecule selectors")
		if type(version) is not engine.MolblockVersionV1:
			raise TypeError("Ferrum multi-record SDF export requires an exact Ferrum version")
		snapshot = observation.snapshot
		super().__init__(
			engine.export_document_molecules_sdf_v2,
			(observation, snapshot.revision, snapshot.digest, molecule_ids, version),
			_multi_sdf_failure,
		)


#============================================
class _MultiSdfExportDeliveryRelay(PySide6.QtCore.QObject):
	"""Route selected-root worker signals to the owning Qt window."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the sole window that owns this delivery route."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_exported(self, result: object) -> None:
		"""Deliver one complete Rust receipt with its emitting worker."""
		self._owner._on_document_multi_sdf_exported(self.sender(), result)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Deliver one typed export refusal with its emitting worker."""
		self._owner._on_document_multi_sdf_export_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release one stopped worker from its owning window."""
		self._owner._on_document_multi_sdf_export_finished(self.sender())


#============================================
class FerrumNativeMultiSdfExportMixin:
	"""Own selected-root multi-record SDF export without widening V1."""

	#============================================
	def _initialize_multi_sdf_exports(self) -> None:
		"""Initialize the one selected-root SDF export intent."""
		self._multi_sdf_export_intent: _MultiSdfExportIntent | None = None
		self._multi_sdf_export_relay = _MultiSdfExportDeliveryRelay(self)

	#============================================
	def _build_multi_sdf_export_actions(
			self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add explicit selected-molecule V2000 and V3000 actions."""
		self._export_selected_sdf_v2000_action = PySide6.QtGui.QAction(
			self.tr("Export Selected Molecules as SDF V2000..."), self,
		)
		self._export_selected_sdf_v2000_action.triggered.connect(
			self._choose_document_multi_sdf_v2000_export,
		)
		menu.addAction(self._export_selected_sdf_v2000_action)
		self._export_selected_sdf_v3000_action = PySide6.QtGui.QAction(
			self.tr("Export Selected Molecules as SDF V3000..."), self,
		)
		self._export_selected_sdf_v3000_action.triggered.connect(
			self._choose_document_multi_sdf_v3000_export,
		)
		menu.addAction(self._export_selected_sdf_v3000_action)

	#============================================
	def _choose_document_multi_sdf_v2000_export(self) -> None:
		"""Choose one destination for the current V2000 selected roots."""
		self._choose_document_multi_sdf_export(engine.MolblockVersionV1.v2000)

	#============================================
	def _choose_document_multi_sdf_v3000_export(self) -> None:
		"""Choose one destination for the current V3000 selected roots."""
		self._choose_document_multi_sdf_export(engine.MolblockVersionV1.v3000)

	#============================================
	def _choose_document_multi_sdf_export(self, version: object) -> None:
		"""Capture selected membership, then choose its output destination."""
		tab = self._active_native_tab()
		if tab is None:
			return
		molecule_ids = self._selected_multi_sdf_molecule_ids(tab)
		if molecule_ids is None:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Select atoms or bonds from at least two durable molecules.",
			))
			return
		snapshot = tab.current_snapshot
		capture = _MultiSdfFileCapture(
			tab, snapshot.revision, snapshot.digest, molecule_ids, version,
		)
		label = self._multi_sdf_version_label(version)
		selected_path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Export Selected Molecules as SDF %s" % label), "",
			self.tr(_SDF_FILTER),
		)[0]
		if not selected_path:
			return
		destination = self._normalize_multi_sdf_path(selected_path)
		if destination is None:
			return
		if not self._multi_sdf_file_capture_is_current(capture):
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The selected molecules changed while choosing a destination. "
				"Choose the SDF export again for the current selection.",
			))
			return
		if not self.start_document_multi_sdf_export(
				capture.molecule_ids, capture.version, destination,
			):
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Another operation started while choosing the destination. "
				"Choose the SDF export again after it finishes.",
			))

	#============================================
	def _selected_multi_sdf_molecule_ids(self, tab: object) -> tuple[str, ...] | None:
		"""Return selected durable membership without assigning record order."""
		addresses = ferrum_qt.ferrum.molecule_inspection.selected_durable_molecule_addresses(
			tab,
		)
		if addresses is None or len(addresses) < 2:
			return None
		molecule_ids = tuple(address.molecule_id for address in addresses)
		return molecule_ids if len(set(molecule_ids)) == len(molecule_ids) else None

	#============================================
	def _normalize_multi_sdf_path(self, selected_path: str) -> str | None:
		"""Apply the closed .sdf destination policy without touching the file."""
		path = pathlib.Path(selected_path)
		if not path.suffix:
			path = path.with_suffix(".sdf")
		elif path.suffix.lower() != ".sdf":
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum SDF exports must use the .sdf extension.",
			))
			return None
		return os.path.abspath(str(path))

	#============================================
	def _multi_sdf_file_capture_is_current(self, capture: _MultiSdfFileCapture) -> bool:
		"""Reauthenticate the active tab, observation fence, and membership."""
		tab = capture.tab
		if (
			self._active_native_tab() is not tab
			or self._native_tabs_by_page.get(tab) is not tab
			or tab.is_disposed
			or tab.requires_refresh
		):
			return False
		snapshot = tab.current_snapshot
		return (
			_has_same_multi_sdf_membership(
				capture.molecule_ids, self._selected_multi_sdf_molecule_ids(tab) or (),
			)
			and snapshot.revision == capture.revision
			and snapshot.digest == capture.digest
		)

	#============================================
	def start_document_multi_sdf_export(
			self, molecule_ids: tuple[str, ...], version: object, destination: str,
			) -> bool:
		"""Start one frozen selected-root export through the V2 Rust boundary."""
		if len(molecule_ids) < 2 or len(set(molecule_ids)) != len(molecule_ids):
			raise ValueError("Ferrum multi-record SDF export requires distinct molecule selectors")
		if type(version) is not engine.MolblockVersionV1:
			raise TypeError("Ferrum multi-record SDF export requires an exact Ferrum version")
		if type(destination) is not str or not os.path.isabs(destination):
			raise ValueError("Ferrum multi-record SDF export requires an absolute destination")
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
			if not _has_same_multi_sdf_membership(
				molecule_ids, self._selected_multi_sdf_molecule_ids(tab) or (),
			):
				raise ValueError("Ferrum selected molecules changed before export started")
			worker = FerrumNativeMultiSdfExportWorker(observation, molecule_ids, version)
		except (AttributeError, TypeError, ValueError) as error:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(error)))
			return False
		snapshot = tab.current_snapshot
		self._multi_sdf_export_intent = _MultiSdfExportIntent(
			tab, snapshot.revision, snapshot.digest, molecule_ids,
			version, destination, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.exported.connect(self._multi_sdf_export_relay.on_exported, connection)
		worker.failed.connect(self._multi_sdf_export_relay.on_failed, connection)
		worker.finished.connect(self._multi_sdf_export_relay.on_finished, connection)
		self.statusBar().showMessage(
			self.tr("Preparing selected SDF records %s with Ferrum Rust...")
			% self._multi_sdf_version_label(version),
			0,
		)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _current_multi_sdf_export_intent(
			self, worker: object) -> _MultiSdfExportIntent | None:
		"""Return an export only while its selected roots remain current."""
		intent = self._multi_sdf_export_intent
		if (
			intent is None
			or worker is not intent.worker
			or intent.worker.delivery_cancelled
			or not self._multi_sdf_file_capture_is_current(_MultiSdfFileCapture(
				intent.tab, intent.revision, intent.digest, intent.molecule_ids, intent.version,
			))
		):
			return None
		return intent

	#============================================
	def _on_document_multi_sdf_exported(self, worker: object, result: object) -> None:
		"""Publish only one complete, fresh, canonically ordered Rust receipt."""
		intent = self._current_multi_sdf_export_intent(worker)
		if intent is None:
			self._show_stale_multi_sdf_export()
			return
		if type(result) is not engine.DocumentMoleculesSdfV2:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum returned an unexpected selected-molecule export value.",
			))
			return
		if (
			result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or result.record_count != len(intent.molecule_ids)
			or not _has_same_multi_sdf_membership(
				intent.molecule_ids, tuple(result.molecule_ids),
			)
			or result.version is not intent.version
			or type(result.sdf) is not str
		):
			self._show_stale_multi_sdf_export()
			return
		try:
			publication = engine.publish_document_molecules_sdf_v2(result, intent.destination)
		except engine.FerrumError as error:
			self._report_molecule_file_publication_error("SDF", intent.destination, error)
			return
		if type(publication) is not engine.DocumentMoleculesSdfPublicationV2:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum returned an unexpected selected-molecule publication result.",
			))
			return
		message = self.tr("Selected SDF records %s exported: %s") % (
				self._multi_sdf_version_label(intent.version), intent.destination,
		)
		if not publication.directory_entry_confirmed:
			message += self.tr(" (file written; directory confirmation unavailable)")
		self.statusBar().showMessage(message, 5000)

	#============================================
	def _on_document_multi_sdf_export_failed(self, worker: object, failure: object) -> None:
		"""Present the V2 typed refusal only while the selected roots are current."""
		if self._current_multi_sdf_export_intent(worker) is None:
			self._show_stale_multi_sdf_export()
			return
		if type(failure) is not engine.DocumentMoleculesSdfError:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum returned an unexpected selected-molecule export failure.",
			))
			return
		self._show_edit_refusal(self._unavailable_edit_refusal(str(failure)))

	#============================================
	def _on_document_multi_sdf_export_finished(self, worker: object) -> None:
		"""Release the selected-root worker and restore command reachability."""
		intent = self._multi_sdf_export_intent
		if intent is None or worker is not intent.worker:
			return
		self._multi_sdf_export_intent = None
		intent.worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _show_stale_multi_sdf_export(self) -> None:
		"""Explain that stale selected-root output was deliberately withheld."""
		self.statusBar().showMessage(
			self.tr("Discarded stale selected SDF export; the source selection changed."),
			5000,
		)

	#============================================
	def _multi_sdf_version_label(self, version: object) -> str:
		"""Return the closed visible Molfile syntax label."""
		if version is engine.MolblockVersionV1.v2000:
			return "V2000"
		if version is engine.MolblockVersionV1.v3000:
			return "V3000"
		raise TypeError("Ferrum multi-record SDF export requires an exact Ferrum version")

	#============================================
	def _molecule_export_busy(self) -> bool:
		"""Include selected-root SDF work in the shared export exclusion."""
		return self._multi_sdf_export_intent is not None or super()._molecule_export_busy()

	#============================================
	def _cancel_document_molecule_export(self) -> None:
		"""Cancel selected-root delivery or continue through the export stack."""
		intent = self._multi_sdf_export_intent
		if intent is None:
			super()._cancel_document_molecule_export()
			return
		if intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(
			self.tr("Cancelling selected SDF export; waiting for the current operation to finish..."),
			0,
		)
		self._refresh_actions()

	#============================================
	def _refresh_multi_sdf_export_actions(
			self, active: bool, pending: bool, busy_elsewhere: bool,
			) -> None:
		"""Enable selected-root actions only for two current eligible roots."""
		can_start = (
			active and not pending and not busy_elsewhere and not self._molecule_export_busy()
		)
		tab = self._active_native_tab() if can_start else None
		enabled = tab is not None and self._selected_multi_sdf_molecule_ids(tab) is not None
		self._export_selected_sdf_v2000_action.setEnabled(enabled)
		self._export_selected_sdf_v3000_action.setEnabled(enabled)

	#============================================
	def _refresh_molecule_export_actions(
			self, active: bool, pending: bool, busy_elsewhere: bool,
			) -> None:
		"""Keep the shared cancel action accurate for a selected-root worker."""
		super()._refresh_molecule_export_actions(active, pending, busy_elsewhere)
		intent = self._multi_sdf_export_intent
		if intent is not None:
			self._cancel_molecule_export_action.setEnabled(
				not intent.worker.delivery_cancelled,
			)

	#============================================
	def _molecule_export_blocks_tab_close(self, tab: object) -> bool:
		"""Keep a selected-root source tab live until worker teardown completes."""
		intent = self._multi_sdf_export_intent
		if intent is None or intent.tab is not tab:
			return super()._molecule_export_blocks_tab_close(tab)
		self._show_edit_refusal(self._unavailable_edit_refusal(
			"Cancel the selected SDF export and wait for the current operation before closing.",
		))
		return True

	#============================================
	def _cancel_molecule_export_for_close(self) -> bool:
		"""Invalidate selected-root delivery before close continues."""
		if self._multi_sdf_export_intent is None:
			return super()._cancel_molecule_export_for_close()
		self._cancel_document_molecule_export()
		self._show_edit_refusal(self._unavailable_edit_refusal(
			"Ferrum cancelled selected SDF delivery; close again after it finishes.",
		))
		return True

"""Current-document SVG, PDF, and PNG export through Ferrum Rust."""

# Standard Library
import dataclasses
import enum
import os
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_chem

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab


#============================================
class FerrumNativeSnapshotFormat(enum.Enum):
	"""Closed complete-document artifact formats for the native window."""

	SVG = "svg"
	PDF = "pdf"
	PNG = "png"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _SnapshotExportCapture:
	"""One exact document observation retained while choosing a destination."""

	tab: object
	revision: int
	digest: str
	observation: object
	origin: object | None


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _SnapshotExportIntent:
	"""One immutable artifact preparation that has not yet been published."""

	capture: _SnapshotExportCapture
	destination: str
	export_format: FerrumNativeSnapshotFormat
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _SnapshotExportFailure:
	"""One user-safe native preparation failure delivered to the Qt thread."""

	category: str


#============================================
class _SnapshotExportWorker(PySide6.QtCore.QThread):
	"""Prepare one immutable Rust artifact away from the Qt event thread."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, capture: _SnapshotExportCapture,
			export_format: FerrumNativeSnapshotFormat) -> None:
		"""Freeze one authenticated observation and one closed export profile."""
		super().__init__()
		if type(capture.observation) is not ferrum_chem.SessionDocumentObservationV1:
			raise TypeError("native artifact export requires an exact Ferrum observation")
		self._capture = capture
		self._profile = _rust_profile(export_format)
		self._delivery_cancelled = False

	#============================================
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether this worker must withhold future receipt delivery."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Discard future delivery without claiming to interrupt Rust rendering."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def run(self) -> None:
		"""Prepare and emit at most one receipt from the frozen observation."""
		try:
			receipt = ferrum_chem.prepare_document_native_artifact_v1(
				self._capture.observation, self._capture.revision,
				self._capture.digest, self._profile,
			)
		except ferrum_chem.DocumentNativeArtifactError as error:
			if not self._delivery_cancelled and not self.isInterruptionRequested():
				category = getattr(error, "category", "preparation_failed")
				self.failed.emit(_SnapshotExportFailure(
					category if type(category) is str else "preparation_failed",
				))
			return
		if not self._delivery_cancelled and not self.isInterruptionRequested():
			self.prepared.emit(receipt)


#============================================
class _SnapshotExportDeliveryRelay(PySide6.QtCore.QObject):
	"""Return native artifact worker events to their owning window."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the exact window that owns the current export intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_prepared(self, receipt: object) -> None:
		"""Forward one prepared receipt with its exact emitting worker."""
		self._owner._on_snapshot_export_prepared(self.sender(), receipt)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward one preparation failure with its exact emitting worker."""
		self._owner._on_snapshot_export_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release one stopped worker owned by the window."""
		self._owner._on_snapshot_export_finished(self.sender())


#============================================
class FerrumNativeSnapshotExportWindowMixin:
	"""Own lifecycle-fenced publication of current whole-document artifacts."""

	#============================================
	def _initialize_snapshot_exports(self) -> None:
		"""Initialize the single native artifact export intent."""
		self._snapshot_export_intent: _SnapshotExportIntent | None = None
		self._snapshot_export_relay = _SnapshotExportDeliveryRelay(self)

	#============================================
	def _build_snapshot_export_actions(self, file_menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the ordinary whole-document artifact commands."""
		menu = file_menu.addMenu(self.tr("Export..."))
		self._snapshot_export_actions = {}
		for export_format, label in (
			(FerrumNativeSnapshotFormat.SVG, "Export SVG..."),
			(FerrumNativeSnapshotFormat.PDF, "Export PDF..."),
			(FerrumNativeSnapshotFormat.PNG, "Export PNG (1 pixel per point)..."),
		):
			action = PySide6.QtGui.QAction(self.tr(label), self)
			action.triggered.connect(
				lambda _checked=False, selected=export_format:
				self._choose_snapshot_export(selected),
			)
			menu.addAction(action)
			self._snapshot_export_actions[export_format] = action

	#============================================
	def _snapshot_export_busy(self) -> bool:
		"""Return whether one native artifact receipt remains in flight."""
		return self._snapshot_export_intent is not None

	#============================================
	def _refresh_snapshot_export_actions(
			self, active: bool, pending: bool, busy: bool) -> None:
		"""Keep artifact export available only for an idle current document."""
		available = (
			active and not pending and not busy and not self.has_pending_local_cdml_open()
		)
		for action in self._snapshot_export_actions.values():
			action.setEnabled(available)

	#============================================
	def _choose_snapshot_export(self, export_format: FerrumNativeSnapshotFormat) -> None:
		"""Capture before a destination dialog and return quietly on cancel."""
		capture = self._capture_snapshot_export()
		if capture is None:
			return
		selected_path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Export"), "", self.tr(_file_filter(export_format)),
		)[0]
		if not selected_path:
			return
		destination = self._normalize_snapshot_export_path(selected_path, export_format)
		if destination is None:
			return
		if not self._snapshot_capture_is_current(capture):
			self._show_stale_snapshot_export()
			return
		if not self.start_document_snapshot_export(capture, destination, export_format):
			self._show_native_file_warning(
				"Export Unavailable",
				"Another native operation started while choosing the destination. "
				"Choose Export again after it finishes.",
			)

	#============================================
	def export_active_snapshot(
			self, path: str, export_format: FerrumNativeSnapshotFormat) -> bool:
		"""Start a programmatic export with the same capture and current-tab fences."""
		if type(path) is not str or type(export_format) is not FerrumNativeSnapshotFormat:
			raise TypeError("native artifact export requires an exact path and format")
		capture = self._capture_snapshot_export()
		if capture is None:
			return False
		destination = self._normalize_snapshot_export_path(path, export_format)
		if destination is None:
			return False
		if not self._snapshot_capture_is_current(capture):
			self._show_stale_snapshot_export()
			return False
		return self.start_document_snapshot_export(capture, destination, export_format)

	#============================================
	def _capture_snapshot_export(self) -> _SnapshotExportCapture | None:
		"""Take one immutable observation from the exact current live tab."""
		if self._snapshot_export_conflicts_with_native_operation():
			return None
		tab = self._active_native_tab()
		if (
			tab is None
			or self._native_tabs_by_page.get(tab) is not tab
			or tab._disposed
			or tab.requires_refresh
		):
			return None
		try:
			observation = tab.current_document_observation()
		except ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTabError:
			self._show_native_file_warning(
				"Export Unavailable",
				"The current document is not ready for export. Refresh it and try again.",
			)
			return None
		snapshot = tab.current_snapshot
		observed = observation.snapshot
		if observed.revision != snapshot.revision or observed.digest != snapshot.digest:
			self._show_native_file_warning(
				"Export Unavailable",
				"The current document changed before export could begin. Choose Export again.",
			)
			return None
		return _SnapshotExportCapture(
			tab, snapshot.revision, snapshot.digest, observation,
			tab.local_cdml_origin_token,
		)

	#============================================
	def _normalize_snapshot_export_path(
			self, selected_path: str, export_format: FerrumNativeSnapshotFormat,
			) -> str | None:
		"""Apply the closed suffix policy without reserving a destination."""
		path = pathlib.Path(selected_path)
		if not path.suffix:
			path = path.with_suffix(f".{export_format.value}")
		elif path.suffix.lower() != f".{export_format.value}":
			self._show_native_file_warning(
				"Unsupported Export Format",
				"Ferrum %s exports must use the .%s extension."
				% (_format_label(export_format), export_format.value),
			)
			return None
		return os.path.abspath(str(path))

	#============================================
	def _snapshot_capture_is_current(
			self, capture: _SnapshotExportCapture, *, allow_snapshot_export: bool = False,
			) -> bool:
		"""Reauthenticate tab identity, liveness, provenance, and idle ownership."""
		tab = capture.tab
		if self._snapshot_export_conflicts_with_native_operation(allow_snapshot_export):
			return False
		if (
			self._active_native_tab() is not tab
			or self._native_tabs_by_page.get(tab) is not tab
			or tab._disposed
			or tab.requires_refresh
		):
			return False
		snapshot = tab.current_snapshot
		return snapshot.revision == capture.revision and snapshot.digest == capture.digest

	#============================================
	def _snapshot_export_conflicts_with_native_operation(
			self, allow_snapshot_export: bool = False) -> bool:
		"""Require the same idle native-operation boundary as the rest of the window."""
		return (
			(self._snapshot_export_busy() and not allow_snapshot_export)
			or self.has_pending_local_cdml_open()
			or self._molecule_import_busy()
			or self._molecule_export_busy()
			or self._molecule_inspection_busy()
			or self._clipboard_busy()
			or self._coordinate_generation_intent is not None
			or self._user_template_placement_intent is not None
		)

	#============================================
	def start_document_snapshot_export(
			self, capture: _SnapshotExportCapture, destination: str,
			export_format: FerrumNativeSnapshotFormat) -> bool:
		"""Start one exact-observation preparation after the post-dialog fence."""
		if type(capture) is not _SnapshotExportCapture:
			raise TypeError("native artifact export requires an exact document capture")
		if type(destination) is not str or not os.path.isabs(destination):
			raise ValueError("native artifact export requires an absolute destination")
		if type(export_format) is not FerrumNativeSnapshotFormat:
			raise TypeError("native artifact export requires an exact closed format")
		if not self._snapshot_capture_is_current(capture):
			return False
		try:
			worker = _SnapshotExportWorker(capture, export_format)
		except (TypeError, ValueError) as error:
			self._show_native_file_warning("Export Unavailable", str(error))
			return False
		self._snapshot_export_intent = _SnapshotExportIntent(
			capture, destination, export_format, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.prepared.connect(self._snapshot_export_relay.on_prepared, connection)
		worker.failed.connect(self._snapshot_export_relay.on_failed, connection)
		worker.finished.connect(self._snapshot_export_relay.on_finished, connection)
		self.statusBar().showMessage(
			self.tr("Preparing %s with Ferrum Rust...") % _format_label(export_format), 0,
		)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _current_snapshot_export_intent(self, worker: object) -> _SnapshotExportIntent | None:
		"""Return a receipt intent only while the original tab remains current."""
		intent = self._snapshot_export_intent
		if intent is None or worker is not intent.worker or worker.delivery_cancelled:
			return None
		if not self._snapshot_capture_is_current(
				intent.capture, allow_snapshot_export=True,
			):
			return None
		return intent

	#============================================
	def _on_snapshot_export_prepared(self, worker: object, receipt: object) -> None:
		"""Publish a current authenticated receipt through the Rust publisher."""
		intent = self._current_snapshot_export_intent(worker)
		if intent is None:
			self._show_stale_snapshot_export()
			return
		if type(receipt) is not ferrum_chem.PreparedDocumentNativeArtifactV1:
			self._show_native_file_warning(
				"Export Unavailable",
				"Ferrum returned an unexpected prepared artifact. No successful export "
				"is being reported.",
			)
			return
		if (
			receipt.profile != intent.export_format.value
			or receipt.source_revision != intent.capture.revision
			or receipt.source_digest != intent.capture.digest
		):
			self._show_stale_snapshot_export()
			return
		try:
			publication = ferrum_chem.publish_prepared_document_native_artifact_v1(
				receipt, pathlib.Path(intent.destination), intent.capture.origin,
			)
		except (
				ferrum_chem.InvalidDestinationError,
				ferrum_chem.PublicationNotStartedError,
				ferrum_chem.PublicationPossiblyCompletedError,
			) as error:
			self._report_snapshot_publication_error(intent, error)
			return
		if type(publication) is not ferrum_chem.DocumentNativeArtifactPublicationV1:
			self._show_native_file_warning(
				"Export Uncertain",
				"Ferrum returned an unexpected publication value. Inspect %s because Rust "
				"may already have written it." % intent.destination,
			)
			return
		label = _format_label(intent.export_format)
		if publication.directory_entry_confirmed:
			self.statusBar().showMessage(
				self.tr("%s exported: %s") % (label, intent.destination), 5000,
			)
			return
		self._show_native_file_warning(
			"Export Durability Unconfirmed",
			"The exact %s artifact may be present at %s, but directory-entry durability "
			"needs verification. Inspect the destination before relying on it."
			% (label, intent.destination),
		)

	#============================================
	def _on_snapshot_export_failed(self, worker: object, failure: object) -> None:
		"""Show one current preparation refusal without a fallback renderer."""
		if self._current_snapshot_export_intent(worker) is None:
			self._show_stale_snapshot_export()
			return
		category = failure.category if type(failure) is _SnapshotExportFailure else (
			"preparation_failed"
		)
		if category == "provenance_mismatch":
			message = (
				"The current document changed before export could begin. Choose Export again."
			)
		elif category == "unsupported_complete_document":
			message = (
				"This complete-document export cannot represent all current content. "
				"Keep the document unchanged, adjust the content, then choose Export again."
			)
		else:
			message = (
				"Ferrum could not prepare this artifact. Keep the document open and try "
				"Export again."
			)
		self._show_native_file_warning("Export Unavailable", message)

	#============================================
	def _on_snapshot_export_finished(self, worker: object) -> None:
		"""Release one stopped artifact worker and restore ordinary actions."""
		intent = self._snapshot_export_intent
		if intent is None or worker is not intent.worker:
			return
		self._snapshot_export_intent = None
		intent.worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _cancel_snapshot_export(self) -> None:
		"""Invalidate prepared-receipt delivery while Rust preparation winds down."""
		intent = self._snapshot_export_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(
			self.tr("Cancelling export; waiting for native work to finish..."), 0,
		)
		self._refresh_actions()

	#============================================
	def _snapshot_export_blocks_tab_close(self, tab: object) -> bool:
		"""Keep an export source tab live until its immutable worker tears down."""
		intent = self._snapshot_export_intent
		if intent is None or intent.capture.tab is not tab:
			return False
		self._cancel_snapshot_export()
		self._show_native_file_warning(
			"Export Still Running",
			"Ferrum cancelled delivery; close this tab again after native work finishes.",
		)
		return True

	#============================================
	def _cancel_snapshot_export_for_close(self) -> bool:
		"""Invalidate delivery and preserve the window until the worker stops."""
		if self._snapshot_export_intent is None:
			return False
		self._cancel_snapshot_export()
		self._show_native_file_warning(
			"Export Still Running",
			"Ferrum cancelled delivery; close again after native work finishes.",
		)
		return True

	#============================================
	def _show_stale_snapshot_export(self) -> None:
		"""Explain that a changed document deliberately withheld publication."""
		self.statusBar().showMessage(
			self.tr("Discarded export; the current document changed. Choose Export again."),
			5000,
		)

	#============================================
	def _report_snapshot_publication_error(
			self, intent: _SnapshotExportIntent, error: Exception) -> None:
		"""Describe Rust publication facts without leaking raw backend errors."""
		label = _format_label(intent.export_format)
		if type(error) is ferrum_chem.PublicationPossiblyCompletedError:
			title = "Export Possibly Completed"
			message = (
				"Ferrum could not confirm whether the %s artifact was published at %s. "
				"Inspect the destination before relying on it."
				% (label, intent.destination)
			)
		elif type(error) is ferrum_chem.PublicationNotStartedError:
			title = "Export Not Started"
			message = "Ferrum did not start %s publication to %s. Choose another destination." % (
				label, intent.destination,
			)
		else:
			title = "Export Destination Rejected"
			message = "Ferrum rejected %s. Choose a different destination." % intent.destination
		self._show_native_file_warning(title, message)


#============================================
def _file_filter(export_format: FerrumNativeSnapshotFormat) -> str:
	"""Return one user-facing chooser filter for a closed artifact format."""
	filters = {
		FerrumNativeSnapshotFormat.SVG: "Scalable Vector Graphics (*.svg)",
		FerrumNativeSnapshotFormat.PDF: "Portable Document Format (*.pdf)",
		FerrumNativeSnapshotFormat.PNG: "Portable Network Graphics (*.png)",
	}
	return filters[export_format]


#============================================
def _rust_profile(export_format: FerrumNativeSnapshotFormat) -> str:
	"""Map one user-facing format to its closed private Rust profile."""
	profiles = {
		FerrumNativeSnapshotFormat.SVG: "svg",
		FerrumNativeSnapshotFormat.PDF: "pdf",
		FerrumNativeSnapshotFormat.PNG: "png_one_pixel_per_point_transparent",
	}
	return profiles[export_format]


#============================================
def _format_label(export_format: FerrumNativeSnapshotFormat) -> str:
	"""Return the short user-facing format name for messages."""
	return export_format.value.upper()

"""OASA-free selected-root SVG publication for Rust-native documents."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_chem

# local repo modules
import ferrum_qt.io.clipboard_mime
from ferrum_qt.native.ferrum_native_clipboard import (
	selected_durable_clipboard_object_ids,
)


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _SelectionSvgIntent:
	"""One source tab and exact result corroborators for native SVG work."""

	tab: object
	revision: int
	digest: str
	object_ids: tuple[str, ...]
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeSelectionSvgFailure:
	"""Plain terminal worker failure safe for the Qt event thread."""

	message: str


#============================================
class FerrumNativeSelectionSvgWorker(PySide6.QtCore.QThread):
	"""Render one immutable selected-root SVG off the Qt event thread."""

	rendered = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, observation: object, object_ids: tuple[str, ...],
			parent: PySide6.QtCore.QObject) -> None:
		"""Capture only immutable Rust input and exact durable selectors."""
		if type(observation) is not ferrum_chem.SessionDocumentObservationV1:
			raise TypeError("native selected SVG requires an exact Ferrum observation")
		if type(object_ids) is not tuple or not object_ids:
			raise TypeError("native selected SVG requires a nonempty selector tuple")
		super().__init__(parent)
		self._arguments = (observation, object_ids)
		self._delivery_cancelled = False

	#============================================
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether result delivery has been invalidated."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Suppress delivery while native rendering finishes normally."""
		self._delivery_cancelled = True

	#============================================
	def run(self) -> None:
		"""Render one selected SVG and emit only detached terminal values."""
		try:
			result = ferrum_chem.render_document_selection_svg_v1(*self._arguments)
		except Exception as exc:
			if not self._delivery_cancelled:
				self.failed.emit(FerrumNativeSelectionSvgFailure(str(exc)))
			return
		if not self._delivery_cancelled:
			self.rendered.emit(result)


#============================================
class _SelectionSvgDeliveryRelay(PySide6.QtCore.QObject):
	"""Deliver selected SVG worker signals to the owning native window."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the window responsible for the current SVG intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_rendered(self, result: object) -> None:
		"""Forward a receipt with the exact emitting worker identity."""
		self._owner._on_native_selection_svg_rendered(self.sender(), result)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward a terminal failure with exact worker identity."""
		self._owner._on_native_selection_svg_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the exact stopped worker owned by this window."""
		self._owner._on_native_selection_svg_finished(self.sender())


#============================================
def publish_native_selection_svg(svg: str) -> None:
	"""Publish one authenticated native SVG in vector and text MIME forms."""
	if type(svg) is not str or not svg.startswith("<svg "):
		raise TypeError("native selected SVG must be one nonempty SVG document")
	mime_data = PySide6.QtCore.QMimeData()
	mime_data.setData("image/svg+xml", PySide6.QtCore.QByteArray(svg.encode("utf-8")))
	mime_data.setText(svg)
	mime_data.setProperty(
		ferrum_qt.io.clipboard_mime.FERRUM_OWNED_MIME_PROPERTY, True,
	)
	PySide6.QtWidgets.QApplication.clipboard().setMimeData(mime_data)


#============================================
class FerrumNativeSelectionSvgWindowMixin:
	"""Own asynchronous native Copy as SVG publication and delivery fences."""

	#============================================
	def _initialize_native_clipboard(self) -> None:
		"""Initialize ordinary clipboard operations and selected SVG delivery."""
		super()._initialize_native_clipboard()
		self._selection_svg_intent: _SelectionSvgIntent | None = None
		self._selection_svg_relay = _SelectionSvgDeliveryRelay(self)

	#============================================
	def _build_native_clipboard_actions(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add selected SVG beside the ordinary native clipboard actions."""
		super()._build_native_clipboard_actions(menu)
		self._copy_selection_svg_action = PySide6.QtGui.QAction(
			self.tr("Copy as SVG"), self,
		)
		self._copy_selection_svg_action.setToolTip(self.tr(
			"Copy complete native render roots for the exact current selection",
		))
		self._copy_selection_svg_action.triggered.connect(
			self._start_native_selection_svg,
		)
		menu.insertAction(self._paste_action, self._copy_selection_svg_action)
		self._cancel_selection_svg_action = PySide6.QtGui.QAction(
			self.tr("Cancel Copy as SVG"), self,
		)
		self._cancel_selection_svg_action.triggered.connect(
			self._cancel_native_selection_svg,
		)
		menu.insertAction(self._cancel_paste_action, self._cancel_selection_svg_action)

	#============================================
	def _clipboard_busy(self) -> bool:
		"""Include selected SVG work in the mutually exclusive clipboard family."""
		return self._selection_svg_intent is not None or super()._clipboard_busy()

	#============================================
	def _start_native_selection_svg(self) -> bool:
		"""Begin native SVG rendering for one exact current durable selection."""
		if (
			self._clipboard_busy()
			or self._molecule_import_busy()
			or self._molecule_export_busy()
			or self._molecule_inspection_busy()
			or self._coordinate_generation_intent is not None
		):
			return False
		tab = self._active_native_tab()
		object_ids = None if tab is None else selected_durable_clipboard_object_ids(tab)
		if tab is None or object_ids is None:
			return False
		try:
			observation = tab.current_document_observation()
			snapshot = tab.current_snapshot
			worker = FerrumNativeSelectionSvgWorker(observation, object_ids, self)
		except Exception as exc:
			self._show_native_file_warning("Native Copy as SVG Unavailable", str(exc))
			return False
		self._selection_svg_intent = _SelectionSvgIntent(
			tab, snapshot.revision, snapshot.digest, object_ids, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.rendered.connect(self._selection_svg_relay.on_rendered, connection)
		worker.failed.connect(self._selection_svg_relay.on_failed, connection)
		worker.finished.connect(self._selection_svg_relay.on_finished, connection)
		self.statusBar().showMessage(self.tr("Rendering selected roots as native SVG..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _on_native_selection_svg_rendered(self, worker: object, result: object) -> None:
		"""Publish only a receipt authenticated to the active source selection."""
		intent = self._current_selection_svg_intent(worker)
		if intent is None:
			return
		if (
			type(result) is not ferrum_chem.DocumentSelectionSvgV1
			or result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or result.selected_objects != intent.object_ids
			or not result.selected_roots
		):
			self.statusBar().showMessage(self.tr("Document changed; copy SVG again."), 5000)
			return
		try:
			publish_native_selection_svg(result.svg)
		except Exception as exc:
			self._show_native_file_warning("Native Copy as SVG Error", str(exc))
			return
		self.statusBar().showMessage(self.tr(
			"Copied {0} native render root(s) as SVG."
		).format(len(result.selected_roots)), 5000)

	#============================================
	def _on_native_selection_svg_failed(
			self, worker: object, failure: FerrumNativeSelectionSvgFailure) -> None:
		"""Show one current rendering failure while preserving the clipboard."""
		if self._current_selection_svg_intent(worker) is None:
			return
		self._show_native_file_warning("Native Copy as SVG Error", failure.message)

	#============================================
	def _current_selection_svg_intent(self, worker: object) -> _SelectionSvgIntent | None:
		"""Return only the live intent whose source and selection remain current."""
		intent = self._selection_svg_intent
		if intent is None or worker is not intent.worker or worker.delivery_cancelled:
			return None
		tab = intent.tab
		if (
			tab not in self._native_tabs_by_page
			or self._active_native_tab() is not tab
			or tab.requires_refresh
			or selected_durable_clipboard_object_ids(tab) != intent.object_ids
		):
			return None
		snapshot = tab.current_snapshot
		if snapshot.revision != intent.revision or snapshot.digest != intent.digest:
			return None
		return intent

	#============================================
	def _on_native_selection_svg_finished(self, worker: object) -> None:
		"""Release one exact stopped worker and restore action reachability."""
		intent = self._selection_svg_intent
		if intent is None or worker is not intent.worker:
			return
		self._selection_svg_intent = None
		worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _cancel_native_selection_svg(self) -> None:
		"""Suppress selected SVG delivery while native rendering completes."""
		intent = self._selection_svg_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(self.tr("Cancelling Copy as SVG delivery..."), 0)
		self._refresh_actions()

	#============================================
	def _refresh_native_clipboard_actions(self, active: bool, pending: bool,
			busy_elsewhere: bool) -> None:
		"""Refresh ordinary clipboard and selected SVG action reachability."""
		super()._refresh_native_clipboard_actions(active, pending, busy_elsewhere)
		tab = self._active_native_tab() if active and not pending else None
		object_ids = None if tab is None else selected_durable_clipboard_object_ids(tab)
		self._copy_selection_svg_action.setEnabled(
			active
			and not pending
			and not busy_elsewhere
			and not self._clipboard_busy()
			and object_ids is not None
		)
		self._cancel_selection_svg_action.setEnabled(
			self._selection_svg_intent is not None
			and not self._selection_svg_intent.worker.delivery_cancelled
		)

	#============================================
	def _clipboard_operation_blocks_tab_close(self, tab: object) -> bool:
		"""Keep one selected SVG source tab alive until its worker stops."""
		intent = self._selection_svg_intent
		if intent is None or intent.tab is not tab:
			return super()._clipboard_operation_blocks_tab_close(tab)
		self._show_native_file_warning(
			"Native Copy as SVG Still Running",
			"Cancel Copy as SVG and wait for native work before closing this tab.",
		)
		return True

	#============================================
	def _cancel_clipboard_operations_for_close(self) -> bool:
		"""Cancel selected SVG delivery before ordinary clipboard shutdown."""
		if self._selection_svg_intent is None:
			return super()._cancel_clipboard_operations_for_close()
		self._cancel_native_selection_svg()
		self._show_native_file_warning(
			"Native Copy as SVG Still Running",
			"Ferrum cancelled delivery; close again after native rendering finishes.",
		)
		return True

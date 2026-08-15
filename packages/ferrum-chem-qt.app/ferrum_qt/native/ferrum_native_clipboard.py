"""Revision-bound Copy and Paste for Rust-native documents."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_chem
import shiboken6

# local repo modules
import ferrum_qt.io.clipboard_mime
import ferrum_qt.native.ferrum_native_document_tab_errors as native_document_tab_errors


CDML_MIME_TYPE = "application/x-ferrum-cdml"
_STRUCTURE_KINDS = frozenset({"atom", "bond"})
_PRESENTATION_KINDS = frozenset({
	"arrow", "plus", "text", "polyline", "rectangle", "square", "oval", "circle",
	"polygon",
})


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _ClipboardSelectionFact:
	"""One selected scene target authenticated to a durable projected object."""

	object_id: str
	document_root_order: int
	child_order: int


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _ClipboardCopyIntent:
	"""One source tab and immutable receipt corroborators for a running Copy."""

	tab: object
	revision: int
	digest: str
	object_ids: tuple[str, ...]
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _ClipboardCutIntent:
	"""One source tab and immutable corroborators for a running Cut preparation."""

	tab: object
	revision: int
	digest: str
	object_ids: tuple[str, ...]
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _ClipboardPasteIntent:
	"""One destination tab and immutable corroborators for a running Paste."""

	tab: object
	revision: int
	digest: str
	worker: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeClipboardCopyFailure:
	"""Plain terminal worker failure facts safe for the Qt event thread."""

	message: str


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeClipboardCutFailure:
	"""Plain terminal Cut preparation failure safe for the Qt event thread."""

	message: str


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeClipboardPasteFailure:
	"""Plain terminal preparation failure facts safe for the Qt event thread."""

	message: str


#============================================
def selected_durable_clipboard_object_ids(tab: object) -> tuple[str, ...] | None:
	"""Resolve every selected scene target to one exact durable projection object."""
	if getattr(tab, "requires_refresh", True):
		return None
	targets = tab.selected_molecule_information_targets()
	if type(targets) is not tuple or not targets:
		return None
	observation = tab.current_document_observation()
	projection = observation.projection
	facts = []
	for target in targets:
		matches = _structure_matches(projection, target)
		if matches is None:
			matches = _presentation_matches(projection, target)
		if matches is None or len(matches) != 1:
			return None
		facts.append(matches[0])
	object_ids = [fact.object_id for fact in facts]
	if len(set(object_ids)) != len(object_ids):
		return None
	facts.sort(key=lambda fact: (fact.document_root_order, fact.child_order))
	return tuple(fact.object_id for fact in facts)


#============================================
def _structure_matches(projection: object,
		target: object) -> list[_ClipboardSelectionFact] | None:
	"""Map one selected atom/bond source address to its opaque Rust object ID."""
	if target.kind not in _STRUCTURE_KINDS:
		return None
	if (
		type(target.identifier) is not str
		or not target.identifier
		or type(target.source_order) is not int
	):
		return []
	matches = []
	for molecule in projection.molecules:
		children = molecule.atoms if target.kind == "atom" else molecule.bonds
		for child in children:
			if (
				child.source_id == target.identifier
				and child.source_order == target.source_order
				and type(child.id) is str
				and child.id
			):
				matches.append(_ClipboardSelectionFact(
					child.id, molecule.source_order, child.source_order,
				))
	return matches


#============================================
def _presentation_matches(projection: object,
		target: object) -> list[_ClipboardSelectionFact] | None:
	"""Map one selected presentation identity to its exact opaque Rust object ID."""
	if target.kind not in _PRESENTATION_KINDS:
		return None
	if (
		type(target.identifier) is not str
		or not target.identifier
		or type(target.source_order) is not int
	):
		return []
	matches = []
	for root in projection.presentation_stack.roots:
		projected = _presentation_root_target(root)
		if (
			projected.id == target.identifier
			and projected.record_kind == target.kind
			and projected.source_order == target.source_order
		):
			matches.append(_ClipboardSelectionFact(
				projected.id, projected.source_order, 0,
			))
	return matches


#============================================
def _presentation_root_target(root: object) -> object:
	"""Return the exact target carried by one closed projection root variant."""
	if root.kind == "arrow":
		return root.arrow.target
	if root.kind == "plus":
		return root.plus.target
	if root.kind == "text":
		return root.text.target
	if root.kind in ("polyline", "wavy", "round_bracket"):
		return root.polyline.target
	if root.kind in ("rectangle", "square", "oval", "circle"):
		return root.shape.target
	if root.kind == "polygon":
		return root.polygon.target
	raise ValueError("Rust projection contains an unsupported presentation root kind")


#============================================
class FerrumNativeClipboardCopyWorker(PySide6.QtCore.QThread):
	"""Extract one immutable selected-only CDML fragment off the Qt event thread."""

	copied = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, observation: object, object_ids: tuple[str, ...],
			parent: PySide6.QtCore.QObject) -> None:
		"""Capture only immutable Rust input and exact durable selectors."""
		if type(observation) is not ferrum_chem.SessionDocumentObservationV1:
			raise TypeError("native Copy requires an exact Ferrum observation")
		if type(object_ids) is not tuple or not object_ids:
			raise TypeError("native Copy requires a nonempty exact selector tuple")
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
		"""Suppress delivery without unsafely terminating native work."""
		self._delivery_cancelled = True

	#============================================
	def run(self) -> None:
		"""Extract one fragment and emit only detached terminal values."""
		try:
			result = ferrum_chem.extract_document_clipboard_fragment_v1(*self._arguments)
		except Exception as exc:
			if not self._delivery_cancelled:
				self.failed.emit(FerrumNativeClipboardCopyFailure(str(exc)))
			return
		if not self._delivery_cancelled:
			self.copied.emit(result)


#============================================
class FerrumNativeClipboardCutWorker(PySide6.QtCore.QThread):
	"""Prepare one immutable fragment and deletion plan off the Qt event thread."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, observation: object, object_ids: tuple[str, ...],
			parent: PySide6.QtCore.QObject) -> None:
		"""Capture only immutable Rust input and exact durable selectors."""
		if type(observation) is not ferrum_chem.SessionDocumentObservationV1:
			raise TypeError("native Cut requires an exact Ferrum observation")
		if type(object_ids) is not tuple or not object_ids:
			raise TypeError("native Cut requires a nonempty exact selector tuple")
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
		"""Suppress delivery while native preparation finishes normally."""
		self._delivery_cancelled = True

	#============================================
	def run(self) -> None:
		"""Prepare one Cut plan and emit only a detached terminal value."""
		try:
			result = ferrum_chem.prepare_document_clipboard_cut_v1(*self._arguments)
		except Exception as exc:
			if not self._delivery_cancelled:
				self.failed.emit(FerrumNativeClipboardCutFailure(str(exc)))
			return
		if not self._delivery_cancelled:
			self.prepared.emit(result)


#============================================
class FerrumNativeClipboardPasteWorker(PySide6.QtCore.QThread):
	"""Prepare one captured clipboard string without borrowing a document session."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, source: str, parent: PySide6.QtCore.QObject) -> None:
		"""Capture one exact owned source value for worker-safe Rust admission."""
		if type(source) is not str:
			raise TypeError("native Paste requires an exact clipboard string")
		super().__init__(parent)
		self._source = source
		self._delivery_cancelled = False

	#============================================
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether result delivery has been invalidated."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Suppress delivery without unsafely terminating native work."""
		self._delivery_cancelled = True

	#============================================
	def run(self) -> None:
		"""Prepare one fragment and emit only a detached immutable plan."""
		try:
			result = ferrum_chem.prepare_clipboard_paste_v1(self._source)
		except Exception as exc:
			if not self._delivery_cancelled:
				self.failed.emit(FerrumNativeClipboardPasteFailure(str(exc)))
			return
		if not self._delivery_cancelled:
			self.prepared.emit(result)


#============================================
class _ClipboardCopyDeliveryRelay(PySide6.QtCore.QObject):
	"""Deliver worker signals back to the owning ordinary native window."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the window responsible for the one Copy intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_copied(self, result: object) -> None:
		"""Forward a receipt with the exact emitting worker identity."""
		self._owner._on_native_clipboard_copied(self.sender(), result)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward a failure with the exact emitting worker identity."""
		self._owner._on_native_clipboard_copy_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the stopped worker owned by this window."""
		self._owner._on_native_clipboard_copy_finished(self.sender())


#============================================
class _ClipboardCutDeliveryRelay(PySide6.QtCore.QObject):
	"""Deliver Cut preparation back to the owning ordinary native window."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the window responsible for the one Cut intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_prepared(self, result: object) -> None:
		"""Forward a plan with the exact emitting worker identity."""
		self._owner._on_native_clipboard_cut_prepared(self.sender(), result)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward a failure with the exact emitting worker identity."""
		self._owner._on_native_clipboard_cut_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the stopped Cut worker owned by this window."""
		self._owner._on_native_clipboard_cut_finished(self.sender())


#============================================
class _ClipboardPasteDeliveryRelay(PySide6.QtCore.QObject):
	"""Deliver Paste preparation back to the owning ordinary native window."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the window responsible for the one Paste intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_prepared(self, prepared: object) -> None:
		"""Forward a plan with the exact emitting worker identity."""
		self._owner._on_native_clipboard_paste_prepared(self.sender(), prepared)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward a preparation failure with exact worker identity."""
		self._owner._on_native_clipboard_paste_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the stopped Paste worker owned by this window."""
		self._owner._on_native_clipboard_paste_finished(self.sender())


#============================================
def publish_native_clipboard_fragment(fragment_cdml: str) -> None:
	"""Publish one already-authenticated CDML fragment in Ferrum MIME formats."""
	if type(fragment_cdml) is not str or not fragment_cdml:
		raise TypeError("native clipboard fragment must be nonempty text")
	encoded = fragment_cdml.encode("utf-8")
	mime_data = PySide6.QtCore.QMimeData()
	mime_data.setData(CDML_MIME_TYPE, PySide6.QtCore.QByteArray(encoded))
	mime_data.setText(fragment_cdml)
	mime_data.setProperty(
		ferrum_qt.io.clipboard_mime.FERRUM_OWNED_MIME_PROPERTY, True,
	)
	PySide6.QtWidgets.QApplication.clipboard().setMimeData(mime_data)


#============================================
def native_clipboard_has_paste_candidate() -> bool:
	"""Return whether the UI clipboard advertises plausible Ferrum CDML."""
	mime_data = PySide6.QtWidgets.QApplication.clipboard().mimeData()
	if mime_data is None:
		return False
	if mime_data.hasFormat(CDML_MIME_TYPE):
		return True
	if not mime_data.hasText():
		return False
	return mime_data.text().lstrip().startswith("<cdml")


#============================================
def read_native_clipboard_source() -> str:
	"""Capture one preferred custom-MIME or plausible plain-text CDML value."""
	mime_data = PySide6.QtWidgets.QApplication.clipboard().mimeData()
	if mime_data is None:
		raise ValueError("Clipboard does not contain Ferrum CDML")
	if mime_data.hasFormat(CDML_MIME_TYPE):
		encoded = bytes(mime_data.data(CDML_MIME_TYPE))
		try:
			return encoded.decode("utf-8")
		except UnicodeDecodeError as exc:
			raise ValueError("Ferrum clipboard CDML is not valid UTF-8") from exc
	if mime_data.hasText():
		source = mime_data.text()
		if source.lstrip().startswith("<cdml"):
			return source
		raise ValueError("Clipboard text is not a complete CDML document")
	raise ValueError("Clipboard does not contain Ferrum CDML")


#============================================
class FerrumNativeClipboardWindowMixin:
	"""Own cancellable native Copy/Paste actions and their delivery fences."""

	#============================================
	def _initialize_native_clipboard(self) -> None:
		"""Initialize mutually exclusive clipboard intents and Qt relays."""
		self._clipboard_copy_intent: _ClipboardCopyIntent | None = None
		self._clipboard_cut_intent: _ClipboardCutIntent | None = None
		self._clipboard_paste_intent: _ClipboardPasteIntent | None = None
		self._clipboard_copy_relay = _ClipboardCopyDeliveryRelay(self)
		self._clipboard_cut_relay = _ClipboardCutDeliveryRelay(self)
		self._clipboard_paste_relay = _ClipboardPasteDeliveryRelay(self)
		self._native_clipboard = PySide6.QtWidgets.QApplication.clipboard()
		self._native_clipboard_data_changed_connected = True
		self._native_clipboard.dataChanged.connect(
			self._on_native_clipboard_data_changed,
		)
		self.destroyed.connect(self._dispose_native_clipboard)

	#============================================
	def _dispose_native_clipboard(self, *_unused: object) -> None:
		"""Disconnect QApplication clipboard delivery before native UI disappears."""
		if not self._native_clipboard_data_changed_connected:
			return
		self._native_clipboard.dataChanged.disconnect(
			self._on_native_clipboard_data_changed,
		)
		self._native_clipboard_data_changed_connected = False

	#============================================
	def _build_native_clipboard_actions(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add Cut, Copy, Paste, and explicit cancellation to the native Edit menu."""
		self._cut_action = PySide6.QtGui.QAction(self.tr("Cut"), self)
		self._cut_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Cut)
		self._cut_action.setToolTip(self.tr(
			"Copy the exact selection, then remove it through one Rust transaction",
		))
		self._cut_action.triggered.connect(self._start_native_clipboard_cut)
		menu.addAction(self._cut_action)
		self._copy_action = PySide6.QtGui.QAction(self.tr("Copy"), self)
		self._copy_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Copy)
		self._copy_action.setToolTip(self.tr(
			"Copy the exact selected Rust document objects as Ferrum CDML",
		))
		self._copy_action.triggered.connect(self._start_native_clipboard_copy)
		menu.addAction(self._copy_action)
		self._paste_action = PySide6.QtGui.QAction(self.tr("Paste"), self)
		self._paste_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Paste)
		self._paste_action.setToolTip(self.tr(
			"Paste Ferrum CDML through the authenticated Rust document session",
		))
		self._paste_action.triggered.connect(self._start_native_clipboard_paste)
		menu.addAction(self._paste_action)
		self._cancel_copy_action = PySide6.QtGui.QAction(self.tr("Cancel Copy"), self)
		self._cancel_copy_action.triggered.connect(self._cancel_native_clipboard_copy)
		menu.addAction(self._cancel_copy_action)
		self._cancel_cut_action = PySide6.QtGui.QAction(self.tr("Cancel Cut"), self)
		self._cancel_cut_action.triggered.connect(self._cancel_native_clipboard_cut)
		menu.addAction(self._cancel_cut_action)
		self._cancel_paste_action = PySide6.QtGui.QAction(self.tr("Cancel Paste"), self)
		self._cancel_paste_action.triggered.connect(self._cancel_native_clipboard_paste)
		menu.addAction(self._cancel_paste_action)

	#============================================
	def _clipboard_busy(self) -> bool:
		"""Return whether any native clipboard worker remains live."""
		return (
			self._clipboard_copy_intent is not None
			or self._clipboard_cut_intent is not None
			or self._clipboard_paste_intent is not None
		)

	#============================================
	def _start_native_clipboard_copy(self) -> bool:
		"""Begin Copy only for one exact current durable selection."""
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
			worker = FerrumNativeClipboardCopyWorker(observation, object_ids, self)
		except Exception as exc:
			self._show_native_file_warning("Native Copy Unavailable", str(exc))
			return False
		self._clipboard_copy_intent = _ClipboardCopyIntent(
			tab, snapshot.revision, snapshot.digest, object_ids, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.copied.connect(self._clipboard_copy_relay.on_copied, connection)
		worker.failed.connect(self._clipboard_copy_relay.on_failed, connection)
		worker.finished.connect(self._clipboard_copy_relay.on_finished, connection)
		self.statusBar().showMessage(self.tr("Copying selected objects with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _start_native_clipboard_cut(self) -> bool:
		"""Prepare Cut only for one exact current durable selection."""
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
			worker = FerrumNativeClipboardCutWorker(observation, object_ids, self)
		except Exception as exc:
			self._show_native_file_warning("Native Cut Unavailable", str(exc))
			return False
		self._clipboard_cut_intent = _ClipboardCutIntent(
			tab, snapshot.revision, snapshot.digest, object_ids, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.prepared.connect(self._clipboard_cut_relay.on_prepared, connection)
		worker.failed.connect(self._clipboard_cut_relay.on_failed, connection)
		worker.finished.connect(self._clipboard_cut_relay.on_finished, connection)
		self.statusBar().showMessage(self.tr("Preparing selected objects for Cut..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _start_native_clipboard_paste(self) -> bool:
		"""Capture and prepare one clipboard fragment for the current document."""
		if (
			self._clipboard_busy()
			or self._molecule_import_busy()
			or self._molecule_export_busy()
			or self._molecule_inspection_busy()
			or self._coordinate_generation_intent is not None
		):
			return False
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return False
		try:
			source = read_native_clipboard_source()
			snapshot = tab.current_snapshot
			worker = FerrumNativeClipboardPasteWorker(source, self)
		except Exception as exc:
			self._show_native_file_warning("Native Paste Unavailable", str(exc))
			return False
		self._clipboard_paste_intent = _ClipboardPasteIntent(
			tab, snapshot.revision, snapshot.digest, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.prepared.connect(self._clipboard_paste_relay.on_prepared, connection)
		worker.failed.connect(self._clipboard_paste_relay.on_failed, connection)
		worker.finished.connect(self._clipboard_paste_relay.on_finished, connection)
		self.statusBar().showMessage(self.tr("Preparing clipboard CDML with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()
		return True

	#============================================
	def _on_native_clipboard_copied(self, worker: object, result: object) -> None:
		"""Publish only a receipt authenticated to the current source tab."""
		intent = self._current_clipboard_copy_intent(worker)
		if intent is None:
			return
		if (
			type(result) is not ferrum_chem.DocumentClipboardFragmentV1
			or result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or result.selected_objects != intent.object_ids
			or result.kind not in ("structure", "top_level")
		):
			self.statusBar().showMessage(self.tr("Document changed; copy again."), 5000)
			return
		try:
			publish_native_clipboard_fragment(result.fragment_cdml)
		except Exception as exc:
			self._show_native_file_warning("Native Copy Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Copied selected Ferrum objects."), 5000)

	#============================================
	def _on_native_clipboard_copy_failed(self, worker: object,
			failure: FerrumNativeClipboardCopyFailure) -> None:
		"""Show one current extraction failure without changing the clipboard."""
		if self._current_clipboard_copy_intent(worker) is None:
			return
		self._show_native_file_warning("Native Copy Error", failure.message)

	#============================================
	def _current_clipboard_copy_intent(self, worker: object) -> _ClipboardCopyIntent | None:
		"""Return only the exact worker intent whose source remains current."""
		intent = self._clipboard_copy_intent
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
	def _on_native_clipboard_copy_finished(self, worker: object) -> None:
		"""Release one exact stopped worker and restore action reachability."""
		intent = self._clipboard_copy_intent
		if intent is None or worker is not intent.worker:
			return
		self._clipboard_copy_intent = None
		worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _on_native_clipboard_cut_prepared(self, worker: object, result: object) -> None:
		"""Publish the admitted fragment, then commit its exact deletion plan."""
		intent = self._current_clipboard_cut_intent(worker)
		if intent is None:
			return
		if (
			type(result) is not ferrum_chem.DocumentClipboardCutPlanV1
			or result.source_revision != intent.revision
			or result.source_digest != intent.digest
			or result.selected_objects != intent.object_ids
		):
			self.statusBar().showMessage(self.tr("Document changed; cut again."), 5000)
			return
		try:
			publish_native_clipboard_fragment(result.fragment_cdml)
		except Exception as exc:
			self._show_native_file_warning("Native Cut Error", str(exc))
			return
		try:
			intent.tab.apply_prepared_clipboard_cut(
				result, intent.revision, intent.digest,
			)
		except native_document_tab_errors.FerrumNativeDocumentTabMutationPresentationError as exc:
			self._show_native_file_warning("Native Cut Needs Refresh", str(exc))
			return
		except Exception as exc:
			self._show_native_file_warning(
				"Native Cut Copied Selection",
				f"The selection is on the clipboard and remains in the document: {exc}",
			)
			return
		self.statusBar().showMessage(self.tr("Cut selected Ferrum objects."), 5000)

	#============================================
	def _on_native_clipboard_cut_failed(self, worker: object,
			failure: FerrumNativeClipboardCutFailure) -> None:
		"""Show one current planning failure while retaining document and clipboard."""
		if self._current_clipboard_cut_intent(worker) is None:
			return
		self._show_native_file_warning("Native Cut Error", failure.message)

	#============================================
	def _current_clipboard_cut_intent(self, worker: object) -> _ClipboardCutIntent | None:
		"""Return the exact Cut intent while its source and selection remain current."""
		intent = self._clipboard_cut_intent
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
	def _on_native_clipboard_cut_finished(self, worker: object) -> None:
		"""Release one exact stopped Cut worker and restore action reachability."""
		intent = self._clipboard_cut_intent
		if intent is None or worker is not intent.worker:
			return
		self._clipboard_cut_intent = None
		worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _on_native_clipboard_paste_prepared(self, worker: object,
			prepared: object) -> None:
		"""Commit only an exact plan authenticated to the current destination."""
		intent = self._current_clipboard_paste_intent(worker)
		if intent is None:
			return
		if type(prepared) is not ferrum_chem.DocumentClipboardPastePlanV1:
			self._show_native_file_warning(
				"Native Paste Error", "Ferrum returned an invalid clipboard Paste plan.",
			)
			return
		try:
			intent.tab.apply_prepared_clipboard_paste(
				prepared, intent.revision, intent.digest,
			)
		except Exception as exc:
			self._show_native_file_warning("Native Paste Error", str(exc))
			return
		self.statusBar().showMessage(
			self.tr("Pasted clipboard CDML with Ferrum Rust."), 5000,
		)

	#============================================
	def _on_native_clipboard_paste_failed(self, worker: object,
			failure: FerrumNativeClipboardPasteFailure) -> None:
		"""Show one current preparation failure without mutating the document."""
		if self._current_clipboard_paste_intent(worker) is None:
			return
		self._show_native_file_warning("Native Paste Error", failure.message)

	#============================================
	def _current_clipboard_paste_intent(self, worker: object) -> _ClipboardPasteIntent | None:
		"""Return only the exact worker intent whose destination remains current."""
		intent = self._clipboard_paste_intent
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
	def _on_native_clipboard_paste_finished(self, worker: object) -> None:
		"""Release one exact stopped worker and restore action reachability."""
		intent = self._clipboard_paste_intent
		if intent is None or worker is not intent.worker:
			return
		self._clipboard_paste_intent = None
		worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _cancel_native_clipboard_copy(self) -> None:
		"""Suppress delivery while Rust extraction finishes normally."""
		intent = self._clipboard_copy_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(self.tr("Cancelling Copy delivery..."), 0)
		self._refresh_actions()

	#============================================
	def _cancel_native_clipboard_cut(self) -> None:
		"""Suppress Cut delivery while Rust preparation finishes normally."""
		intent = self._clipboard_cut_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(self.tr("Cancelling Cut delivery..."), 0)
		self._refresh_actions()

	#============================================
	def _cancel_native_clipboard_paste(self) -> None:
		"""Suppress prepared-plan delivery while Rust preparation finishes."""
		intent = self._clipboard_paste_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(self.tr("Cancelling Paste delivery..."), 0)
		self._refresh_actions()

	#============================================
	@PySide6.QtCore.Slot()
	def _on_native_clipboard_data_changed(self) -> None:
		"""Refresh Paste reachability when the desktop clipboard changes."""
		if not self._native_clipboard_data_changed_connected:
			return
		tab_widget = getattr(self, "_tab_widget", None)
		if tab_widget is None or not shiboken6.isValid(tab_widget):
			self._dispose_native_clipboard()
			return
		if hasattr(self, "_paste_action"):
			self._refresh_actions()

	#============================================
	def _refresh_native_clipboard_actions(self, active: bool, pending: bool,
			busy_elsewhere: bool) -> None:
		"""Apply clipboard, selection, and lifecycle reachability to actions."""
		tab = self._active_native_tab() if active and not pending else None
		object_ids = None if tab is None else selected_durable_clipboard_object_ids(tab)
		self._copy_action.setEnabled(
			active
			and not pending
			and not busy_elsewhere
			and not self._clipboard_busy()
			and object_ids is not None
		)
		self._cut_action.setEnabled(
			active
			and not pending
			and not busy_elsewhere
			and not self._clipboard_busy()
			and object_ids is not None
		)
		self._paste_action.setEnabled(
			active
			and not pending
			and not busy_elsewhere
			and not self._clipboard_busy()
			and native_clipboard_has_paste_candidate()
		)
		self._cancel_copy_action.setEnabled(
			self._clipboard_copy_intent is not None
			and not self._clipboard_copy_intent.worker.delivery_cancelled,
		)
		self._cancel_cut_action.setEnabled(
			self._clipboard_cut_intent is not None
			and not self._clipboard_cut_intent.worker.delivery_cancelled,
		)
		self._cancel_paste_action.setEnabled(
			self._clipboard_paste_intent is not None
			and not self._clipboard_paste_intent.worker.delivery_cancelled,
		)

	#============================================
	def _clipboard_operation_blocks_tab_close(self, tab: object) -> bool:
		"""Keep any clipboard worker's source/destination tab alive through teardown."""
		copy_intent = self._clipboard_copy_intent
		cut_intent = self._clipboard_cut_intent
		paste_intent = self._clipboard_paste_intent
		if (
			(copy_intent is None or copy_intent.tab is not tab)
			and (cut_intent is None or cut_intent.tab is not tab)
			and (paste_intent is None or paste_intent.tab is not tab)
		):
			return False
		operation = "Copy"
		if cut_intent is not None:
			operation = "Cut"
		elif copy_intent is None:
			operation = "Paste"
		self._show_native_file_warning(
			f"Native {operation} Still Running",
			f"Cancel {operation} and wait for native work before closing this tab.",
		)
		return True

	#============================================
	def _cancel_clipboard_operations_for_close(self) -> bool:
		"""Cancel clipboard deliveries and require a later close attempt."""
		if not self._clipboard_busy():
			return False
		if self._clipboard_copy_intent is not None:
			self._cancel_native_clipboard_copy()
		if self._clipboard_cut_intent is not None:
			self._cancel_native_clipboard_cut()
		if self._clipboard_paste_intent is not None:
			self._cancel_native_clipboard_paste()
		self._show_native_file_warning(
			"Native Clipboard Work Still Running",
			"Ferrum cancelled delivery; close again after native work finishes.",
		)
		return True

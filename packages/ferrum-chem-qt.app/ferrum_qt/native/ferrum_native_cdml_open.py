"""Asynchronous Rust-owned local CDML admission for ordinary Ferrum windows."""

# Standard Library
import collections
import dataclasses
import os
import pathlib

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab


_NATIVE_CDML_FILTER = "Ferrum CDML (*.cdml)"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeCdmlOpenFailure:
	"""Plain typed failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str
	stage: str | None
	limit: int | None
	actual: int | None
	observed_at_least: int | None


#============================================
class FerrumNativeCdmlOpenWorker(PySide6.QtCore.QThread):
	"""Admit one bounded local CDML file outside the Qt event thread."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, path: str) -> None:
		"""Capture one exact absolute local path for Rust admission."""
		if type(path) is not str or not path or not os.path.isabs(path):
			raise ValueError("native CDML Open requires a nonempty absolute path")
		super().__init__()
		self._path = path
		self._prepare_operation = ferrum_chem.DocumentSession.prepare_local_cdml_file_v1
		self._delivery_cancelled = False

	#============================================
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether future delivery has been invalidated."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Invalidate delivery without pretending to preempt Rust parsing."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def run(self) -> None:
		"""Admit the file and emit at most one still-current terminal outcome."""
		try:
			prepared = self._prepare_operation(self._path)
		except Exception as exc:
			if not self._delivery_cancelled and not self.isInterruptionRequested():
				self.failed.emit(_cdml_open_failure(exc))
			return
		if not self._delivery_cancelled and not self.isInterruptionRequested():
			self.prepared.emit(prepared)


#============================================
def _cdml_open_failure(exc: Exception) -> FerrumNativeCdmlOpenFailure:
	"""Copy stable ingress facts without retaining a worker-thread exception."""
	if type(exc) is ferrum_chem.DocumentInputError:
		return FerrumNativeCdmlOpenFailure(
			type(exc).__name__, str(exc), exc.stage, exc.limit, exc.actual,
			exc.observed_at_least,
		)
	return FerrumNativeCdmlOpenFailure(
		type(exc).__name__, str(exc), None, None, None, None,
	)


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _LocalCdmlOpenIntent:
	"""One exact local path and its sole admission worker."""

	path: str
	worker: FerrumNativeCdmlOpenWorker


#============================================
class _LocalCdmlOpenRelay(PySide6.QtCore.QObject):
	"""Deliver admission outcomes to the owning window on the Qt thread."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the window that owns the current Open intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_prepared(self, prepared: object) -> None:
		"""Forward one admitted session receipt with its exact worker."""
		self._owner._on_local_cdml_open_prepared(self.sender(), prepared)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward one copied admission failure with its exact worker."""
		self._owner._on_local_cdml_open_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the exact worker after its native call has stopped."""
		self._owner._on_local_cdml_open_finished(self.sender())


#============================================
class FerrumNativeCdmlOpenMixin:
	"""Own ordinary asynchronous CDML Open without frontend document parsing."""

	#============================================
	def _initialize_local_cdml_open(self) -> None:
		"""Create the sole local-CDML Open intent and Qt-thread relay."""
		self._local_cdml_open_intent: _LocalCdmlOpenIntent | None = None
		self._local_cdml_open_queue: collections.deque[str] = collections.deque()
		self._local_cdml_open_outcome: bool | None = None
		self._local_cdml_open_batch_success = True
		self._local_cdml_open_relay = _LocalCdmlOpenRelay(self)

	#============================================
	def _build_local_cdml_open_action(
			self, menu: PySide6.QtWidgets.QMenu,
			) -> PySide6.QtGui.QAction:
		"""Add explicit cancellation next to the host-owned Open action."""
		self._open_action.setToolTip(self.tr(
			"Open one uncompressed CDML file through Ferrum's local V1 policy",
		))
		action = PySide6.QtGui.QAction(self.tr("Cancel Open"), self)
		action.triggered.connect(self._cancel_local_cdml_open)
		menu.addAction(action)
		self._cancel_open_action = action
		return action

	#============================================
	def _on_open(self) -> bool:
		"""Choose one uncompressed CDML path for bounded Rust admission."""
		path = PySide6.QtWidgets.QFileDialog.getOpenFileName(
			self, self.tr("Open Ferrum CDML"), "", self.tr(_NATIVE_CDML_FILTER),
		)[0]
		if not path:
			return False
		return self.open_file_path(path)

	#============================================
	def open_file_path(self, file_path: str, replace_current: bool = False) -> bool:
		"""Begin one profile-owned Rust admission into a new native tab."""
		if type(file_path) is not str:
			raise TypeError("native CDML Open requires an exact path string")
		if replace_current:
			self._show_native_file_warning(
				"Open in Current Tab Unavailable",
				"Ferrum CDML opens in a new Rust-native tab.",
			)
			return False
		absolute_path = os.path.abspath(file_path)
		if pathlib.Path(absolute_path).suffix.lower() != ".cdml":
			self._show_native_file_warning(
				"Unsupported File Format",
				"Ferrum opens uncompressed local CDML files with the .cdml extension.",
			)
			return False
		existing = self._native_tab_for_exact_origin(absolute_path)
		if existing is not None:
			self._tab_widget.setCurrentIndex(self._tab_widget.indexOf(existing))
			return True
		if self._local_cdml_open_intent is not None:
			if self._local_cdml_open_intent.path == absolute_path:
				return True
			if absolute_path not in self._local_cdml_open_queue:
				self._local_cdml_open_queue.append(absolute_path)
			self.statusBar().showMessage(self.tr("Queued Ferrum CDML Open request."), 3000)
			self._refresh_actions()
			return True
		self._local_cdml_open_batch_success = True
		self._start_local_cdml_open(absolute_path)
		return True

	#============================================
	def _start_local_cdml_open(self, absolute_path: str) -> None:
		"""Start one already-validated path as the current queue head."""
		worker = self._create_local_cdml_open_worker(absolute_path)
		self._local_cdml_open_intent = _LocalCdmlOpenIntent(absolute_path, worker)
		self._local_cdml_open_outcome = None
		worker.prepared.connect(
			self._local_cdml_open_relay.on_prepared,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.failed.connect(
			self._local_cdml_open_relay.on_failed,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.finished.connect(
			self._local_cdml_open_relay.on_finished,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		self.statusBar().showMessage(self.tr("Opening CDML with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()

	#============================================
	def open_native_cdml_path(self, file_path: str) -> bool:
		"""Begin the same ordinary bounded Open route for an explicit CDML path."""
		return self.open_file_path(file_path)

	#============================================
	def _create_local_cdml_open_worker(
			self, path: str,
			) -> FerrumNativeCdmlOpenWorker:
		"""Construct the one worker responsible for this admission."""
		return FerrumNativeCdmlOpenWorker(path)

	#============================================
	def _native_tab_for_exact_origin(
			self, absolute_path: str,
			) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab | None:
		"""Return a tab loaded from the exact normalized path spelling."""
		candidate = os.path.normcase(os.path.abspath(absolute_path))
		for tab in self._native_tabs_by_page.values():
			if tab.file_path is None:
				continue
			loaded = os.path.normcase(os.path.abspath(str(tab.file_path)))
			if loaded == candidate:
				return tab
		return None

	#============================================
	def _on_local_cdml_open_prepared(self, worker: object, prepared: object) -> None:
		"""Install one exact still-current admitted session on the Qt thread."""
		intent = self._local_cdml_open_intent
		if intent is None or worker is not intent.worker or intent.worker.delivery_cancelled:
			return
		if type(prepared) is not ferrum_chem.PreparedLocalCdmlOpenV1:
			self._local_cdml_open_outcome = False
			self._show_native_file_warning(
				"CDML Open Error", "Ferrum returned an invalid local-CDML admission receipt.",
			)
			return
		existing = self._native_tab_for_exact_origin(intent.path)
		if existing is not None:
			self._local_cdml_open_outcome = True
			self._tab_widget.setCurrentIndex(self._tab_widget.indexOf(existing))
			return
		tab = None
		try:
			session, observation = prepared.take_admission_v1()
			title = pathlib.Path(intent.path).name
			tab = (
				ferrum_qt.native.ferrum_native_document_tab.
				FerrumNativeDocumentTab.from_admitted_local_open(
					session, title, observation,
				)
			)
			tab._adopt_loaded_origin_path(intent.path)
			self._register_native_tab(tab, activate=True)
		except Exception as exc:
			self._local_cdml_open_outcome = False
			if tab is not None:
				tab.dispose()
			self._show_native_file_warning(
				"CDML Open Error", f"Ferrum could not install {intent.path}:\n{exc}",
			)
			return
		self._local_cdml_open_outcome = True
		self.statusBar().showMessage(self.tr(f"Loaded Rust CDML: {intent.path}"), 3000)

	#============================================
	def _on_local_cdml_open_failed(self, worker: object, failure: object) -> None:
		"""Present one current typed Rust admission failure."""
		intent = self._local_cdml_open_intent
		if intent is None or worker is not intent.worker or intent.worker.delivery_cancelled:
			return
		if type(failure) is not FerrumNativeCdmlOpenFailure:
			self._local_cdml_open_outcome = False
			self._show_native_file_warning(
				"CDML Open Error", "Ferrum returned an invalid CDML Open failure.",
			)
			return
		self._local_cdml_open_outcome = False
		title, guidance = _cdml_open_guidance(failure)
		message = f"Could not open {intent.path}.\n\n{guidance}\n\n{failure.message}"
		self._show_native_file_warning(title, message)

	#============================================
	def _on_local_cdml_open_finished(self, worker: object) -> None:
		"""Retire one exact stopped worker and restore Open reachability."""
		intent = self._local_cdml_open_intent
		if intent is None or worker is not intent.worker:
			return
		outcome = self._local_cdml_open_outcome is True
		self._local_cdml_open_batch_success &= outcome
		self._local_cdml_open_intent = None
		self._local_cdml_open_outcome = None
		worker.deleteLater()
		self.local_cdml_open_completed.emit(intent.path, outcome)
		if self._local_cdml_open_queue and not getattr(self, "_shutdown_prepared", False):
			next_path = self._local_cdml_open_queue.popleft()
			self._start_local_cdml_open(next_path)
			return
		batch_success = self._local_cdml_open_batch_success
		self._local_cdml_open_batch_success = True
		self._refresh_actions()
		self.local_cdml_open_queue_drained.emit(batch_success)

	#============================================
	def _cancel_local_cdml_open(self) -> None:
		"""Invalidate delivery while bounded Rust admission finishes normally."""
		intent = self._local_cdml_open_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		self._local_cdml_open_queue.clear()
		self._local_cdml_open_outcome = False
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(
			self.tr("Cancelling CDML Open delivery; waiting for Rust to finish..."), 0,
		)
		self._refresh_actions()

	#============================================
	def _cancel_local_cdml_open_for_close(self) -> bool:
		"""Cancel a live Open delivery and require a later close attempt."""
		if self._local_cdml_open_intent is None:
			return False
		self._cancel_local_cdml_open()
		return True

	#============================================
	def has_pending_local_cdml_open(self) -> bool:
		"""Return whether Rust admission or a queued launch path remains pending."""
		return self._local_cdml_open_intent is not None or bool(self._local_cdml_open_queue)

	#============================================
	def _refresh_local_cdml_open_action(self) -> None:
		"""Mirror the one-worker lifecycle onto Open and Cancel Open."""
		intent = self._local_cdml_open_intent
		shutdown = getattr(self, "_shutdown_prepared", False)
		self._open_action.setEnabled(intent is None and not shutdown)
		self._cancel_open_action.setEnabled(
			intent is not None and not intent.worker.delivery_cancelled,
		)


#============================================
def _cdml_open_guidance(failure: FerrumNativeCdmlOpenFailure) -> tuple[str, str]:
	"""Return actionable text for one stable Rust failure category."""
	if failure.error_type == "DocumentInputError":
		if failure.stage == "source_policy":
			return (
				"CDML Source Rejected",
				"Choose a regular, non-symlink, uncompressed .cdml file.",
			)
		if failure.stage == "bytes":
			return (
				"CDML Resource Limit",
				"The file exceeds Ferrum's versioned local-CDML V1 source envelope.",
			)
		if failure.stage == "utf8":
			return "CDML Text Rejected", "The file must contain valid UTF-8 CDML text."
		if failure.stage == "resource":
			return "CDML Resource Error", "Ferrum could not reserve the path safely."
		if failure.stage == "path":
			return "CDML Path Rejected", "Choose a path representable by the local platform."
		return "CDML Document Rejected", "The file does not satisfy the local-CDML V1 profile."
	if failure.error_type == "DocumentLoadError":
		return "CDML Document Rejected", "The admitted file is not supported typed CDML."
	return "CDML Open Error", "Ferrum could not complete the local CDML admission."

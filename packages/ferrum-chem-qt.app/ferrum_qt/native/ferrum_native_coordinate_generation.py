"""Rust-native background preparation for existing-molecule coordinates."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_chem


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeCoordinatePreparationFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeCoordinateGenerationIntent:
	"""One source tab plus its revision-bound handle-free worker."""

	tab: object
	revision: int
	digest: str
	worker: object


#============================================
class FerrumNativeCoordinatePreparationWorker(PySide6.QtCore.QThread):
	"""Prepare one complete coordinate update without borrowing a live session."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, observation: object, molecule_id: str) -> None:
		"""Capture one exact immutable observation and durable molecule selector."""
		if type(observation) is not ferrum_chem.SessionDocumentObservationV1:
			raise TypeError("coordinate preparation requires exact Ferrum observation")
		if type(molecule_id) is not str or not molecule_id:
			raise ValueError("coordinate preparation requires a durable molecule selector")
		super().__init__()
		self._observation = observation
		self._molecule_id = molecule_id
		self._prepare_operation = ferrum_chem.prepare_molecule_coordinates_v1
		self._prepare_arguments = (observation, molecule_id)
		self._delivery_cancelled = False
		self.success_message = "Generated Rust-native coordinates."

	#============================================
	def apply_prepared(self, tab: object, prepared: object) -> object:
		"""Apply this worker's exact prepared payload on the Qt thread."""
		return tab.apply_prepared_molecule_coordinates(prepared)

	#============================================
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether future delivery has been invalidated."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Invalidate delivery without claiming to preempt native chemistry."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def run(self) -> None:
		"""Run native preparation and emit at most one still-current outcome."""
		try:
			prepared = self._prepare_operation(*self._prepare_arguments)
		except Exception as exc:
			if not self._delivery_cancelled and not self.isInterruptionRequested():
				self.failed.emit(
					FerrumNativeCoordinatePreparationFailure(type(exc).__name__, str(exc)),
				)
			return
		if not self._delivery_cancelled and not self.isInterruptionRequested():
			self.prepared.emit(prepared)


#============================================
class FerrumNativeCleanGeometryPreparationWorker(
		FerrumNativeCoordinatePreparationWorker):
	"""Prepare one explicit-spacing multi-molecule clean-geometry batch."""

	#============================================
	def __init__(
			self, observation: object, molecule_ids: tuple[str, ...],
			target_spacing_points: float,
			restore: tuple[tuple[str, str], ...]) -> None:
		"""Capture exact immutable targets before native generation begins."""
		if type(observation) is not ferrum_chem.SessionDocumentObservationV1:
			raise TypeError("clean geometry requires exact Ferrum observation")
		if (
			type(molecule_ids) is not tuple
			or not molecule_ids
			or any(type(value) is not str or not value for value in molecule_ids)
		):
			raise ValueError("clean geometry requires durable molecule selectors")
		if type(target_spacing_points) is not float:
			raise TypeError("clean geometry spacing must be an exact float")
		if type(restore) is not tuple or any(
			type(item) is not tuple
			or len(item) != 2
			or type(item[0]) is not str
			or item[0] not in ("atom", "bond")
			or type(item[1]) is not str
			or not item[1]
			for item in restore
		):
			raise TypeError("clean geometry requires an exact selection tuple")
		PySide6.QtCore.QThread.__init__(self)
		self._observation = observation
		self._molecule_ids = molecule_ids
		self._target_spacing_points = target_spacing_points
		self._restore = restore
		self._prepare_operation = ferrum_chem.prepare_clean_geometry_v1
		self._prepare_arguments = (
			observation, molecule_ids, target_spacing_points,
		)
		self._delivery_cancelled = False
		self.success_message = "Cleaned Rust-native molecule geometry."

	#============================================
	def apply_prepared(self, tab: object, prepared: object) -> object:
		"""Apply the whole prepared batch and restore durable selection once."""
		return tab.apply_prepared_clean_geometry(prepared, self._restore)


#============================================
class _CoordinateGenerationDeliveryRelay(PySide6.QtCore.QObject):
	"""Route worker signals to their owning window on the Qt thread."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the window that owns the coordinate-generation intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_prepared(self, prepared: object) -> None:
		"""Forward one prepared result with its exact emitting worker."""
		self._owner._on_coordinates_prepared(self.sender(), prepared)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward one failure with its exact emitting worker."""
		self._owner._on_coordinates_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the exact stopped worker."""
		self._owner._on_coordinate_worker_finished(self.sender())


#============================================
class FerrumNativeCoordinateGenerationWindowMixin:
	"""Own coordinate-generation actions, intent, and worker delivery."""

	#============================================
	def _initialize_coordinate_generation(self) -> None:
		"""Initialize the one mutually exclusive coordinate worker intent."""
		self._coordinate_generation_intent: FerrumNativeCoordinateGenerationIntent | None = None
		self._coordinate_generation_relay = _CoordinateGenerationDeliveryRelay(self)

	#============================================
	def _build_coordinate_generation_actions(
			self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add coordinate generation and cancellation to one Chemistry menu."""
		self._generate_coordinates_action = PySide6.QtGui.QAction(
			self.tr("Generate Molecule Coordinates"), self,
		)
		self._generate_coordinates_action.setToolTip(
			self.tr("Regenerate one durable molecule while retaining its centroid and scale"),
		)
		self._generate_coordinates_action.triggered.connect(self._on_generate_coordinates)
		menu.addAction(self._generate_coordinates_action)
		self._cancel_coordinates_action = PySide6.QtGui.QAction(
			self.tr("Cancel Coordinate Generation"), self,
		)
		self._cancel_coordinates_action.triggered.connect(self._cancel_coordinate_generation)
		menu.addAction(self._cancel_coordinates_action)

	#============================================
	def _on_generate_coordinates(self) -> None:
		"""Choose one durable molecule and start Rust-native coordinate generation."""
		tab = self._active_native_tab()
		if (
			tab is None
			or self._coordinate_generation_intent is not None
			or self._molecule_import_busy()
			or getattr(self, "_molecule_export_intent", None) is not None
		):
			return
		choices = tab.durable_molecule_choices()
		if not choices:
			self._show_native_file_warning(
				"Native Coordinate Generation Unavailable",
				"This document has no durable molecule that Rust can regenerate.",
			)
			return
		choice = choices[0]
		if len(choices) > 1:
			labels = tuple(item.label for item in choices)
			selected, accepted = PySide6.QtWidgets.QInputDialog.getItem(
				self, self.tr("Generate Coordinates"), self.tr("Molecule:"),
				labels, 0, False,
			)
			if not accepted:
				return
			choice = choices[labels.index(selected)]
		snapshot = tab.current_snapshot
		worker = FerrumNativeCoordinatePreparationWorker(
			tab.current_document_observation(), choice.object_id,
		)
		self._coordinate_generation_intent = FerrumNativeCoordinateGenerationIntent(
			tab, snapshot.revision, snapshot.digest, worker,
		)
		connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
		worker.prepared.connect(self._coordinate_generation_relay.on_prepared, connection)
		worker.failed.connect(self._coordinate_generation_relay.on_failed, connection)
		worker.finished.connect(self._coordinate_generation_relay.on_finished, connection)
		self.statusBar().showMessage(self.tr("Generating Rust-native coordinates..."), 0)
		self._refresh_actions()
		worker.start()

	#============================================
	def _on_coordinates_prepared(self, worker: object, prepared: object) -> None:
		"""Commit one still-current coordinate result through the UI-thread session."""
		intent = self._coordinate_generation_intent
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
				self.tr("Discarded stale coordinates; the source document changed."), 5000,
			)
			return
		try:
			intent.worker.apply_prepared(tab, prepared)
		except Exception as exc:
			self._refresh_actions()
			self._show_native_file_warning("Native Coordinate Generation Error", str(exc))
			return
		self.statusBar().showMessage(self.tr(intent.worker.success_message), 5000)
		self._refresh_actions()

	#============================================
	def _on_coordinates_failed(self, worker: object, failure: object) -> None:
		"""Present one current coordinate preparation failure without fallback."""
		intent = self._coordinate_generation_intent
		if intent is None or worker is not intent.worker:
			return
		message = getattr(failure, "message", str(failure))
		self._show_native_file_warning("Native Coordinate Preparation Error", message)

	#============================================
	def _on_coordinate_worker_finished(self, worker: object) -> None:
		"""Release one stopped coordinate worker and restore action reachability."""
		intent = self._coordinate_generation_intent
		if intent is None or worker is not intent.worker:
			return
		self._coordinate_generation_intent = None
		intent.worker.deleteLater()
		self._refresh_actions()

	#============================================
	def _cancel_coordinate_generation(self) -> None:
		"""Invalidate coordinate delivery while native teardown finishes normally."""
		intent = self._coordinate_generation_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(
			self.tr("Cancelling coordinate delivery; waiting for native work to finish..."), 0,
		)
		self._refresh_actions()

	#============================================
	def _coordinate_generation_blocks_tab_close(self, tab: object) -> bool:
		"""Keep the worker's source tab alive until native teardown finishes."""
		intent = self._coordinate_generation_intent
		if intent is None or intent.tab is not tab:
			return False
		self._show_native_file_warning(
			"Native Coordinates Still Running",
			"Cancel coordinate generation and wait for native work before closing.",
		)
		return True

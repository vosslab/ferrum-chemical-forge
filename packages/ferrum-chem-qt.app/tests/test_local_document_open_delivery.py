"""Focused regression coverage for the Local Open Qt delivery boundary.

Coverage mapping: worker-to-relay staging, QObject teardown, typed candidate
failure, escaped invariants, and post-commit presentation recovery live here.
Full File/Open/save/reopen workflows remain in ``tests/e2e``.
"""

# Standard Library
import dataclasses
import pathlib

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtWidgets
import pytest
import shiboken6

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.local_document_open_contract as local_open_contract
import ferrum_qt.ferrum.local_document_open_delivery
import ferrum_qt.ferrum.operation_leases
import ferrum_qt.main_window


_EMPTY_CDML = '<cdml xmlns="urn:ferrum:cdml" version="1.0"/>'


#============================================
class _PreparedThenHeldWorker(PySide6.QtCore.QThread):
	"""Emit a supplied prepared receipt from the worker thread, then wait."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, prepared: object | None = None) -> None:
		"""Create a deterministic worker that only finishes when released."""
		super().__init__()
		self._prepared = prepared
		self._gate = PySide6.QtCore.QWaitCondition()
		self._lock = PySide6.QtCore.QMutex()
		self._prepared_emitted = PySide6.QtCore.QSemaphore()
		self._released = False
		self.delivery_cancelled = False

	#============================================
	def run(self) -> None:
		"""Emit one fact from this QThread and wait for the terminal boundary."""
		if self._prepared is not None:
			self.prepared.emit(self._prepared)
		self._prepared_emitted.release()
		self._lock.lock()
		while not self._released:
			self._gate.wait(self._lock)
		self._lock.unlock()

	#============================================
	def cancel_delivery(self) -> None:
		"""Match the production delivery-only cancellation protocol."""
		self.delivery_cancelled = True

	#============================================
	def wait_until_prepared(self) -> None:
		"""Synchronize without sleeps or event-loop timing assumptions."""
		self._prepared_emitted.acquire()

	#============================================
	def finish_safely(self) -> None:
		"""Release and join the worker exactly once."""
		if not shiboken6.isValid(self) or not self.isRunning():
			return
		self._lock.lock()
		self._released = True
		self._gate.wakeAll()
		self._lock.unlock()
		self.wait()


#============================================
class _StartFailingWorker(PySide6.QtCore.QThread):
	"""Expose one startup invariant failure without entering a worker thread."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self) -> None:
		"""Create the minimum worker-shaped object for startup rollback."""
		super().__init__()
		self.delivery_cancelled = False

	#============================================
	def cancel_delivery(self) -> None:
		"""Match the production delivery-only cancellation protocol."""
		self.delivery_cancelled = True

	#============================================
	def start(
			self,
			_priority: PySide6.QtCore.QThread.Priority = (
				PySide6.QtCore.QThread.Priority.InheritPriority
			),
			) -> None:
		"""Raise the injected startup invariant before a thread begins."""
		raise RuntimeError("worker start invariant")


#============================================
class _RelaySender(PySide6.QtCore.QObject):
	"""Emit worker-shaped signals for the QObject lifetime harness."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)
	finished = PySide6.QtCore.Signal()


#============================================
class _RelayDelivery:
	"""Record whether a destroyed relay could still invoke a host callback."""

	#============================================
	def __init__(self) -> None:
		"""Start with no observed callback."""
		self.callbacks: list[tuple[str, object]] = []

	#============================================
	def stage_prepared(self, sender: object, prepared: object) -> None:
		"""Record a queued prepared callback."""
		self.callbacks.append(("prepared", sender, prepared))

	#============================================
	def stage_failed(self, sender: object, failure: object) -> None:
		"""Record a queued failure callback."""
		self.callbacks.append(("failed", sender, failure))

	#============================================
	def finish(self, sender: object) -> None:
		"""Record a queued finish callback."""
		self.callbacks.append(("finished", sender))


#============================================
def _prepared_cdml(path: pathlib.Path) -> object:
	"""Issue a real one-use Rust receipt for a minimal native CDML document."""
	descriptor = next(
		descriptor
		for descriptor in ferrum_chem.DocumentSession.local_document_open_descriptors_v2()
		if ".cdml" in descriptor.suffixes
	)
	return ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
		str(path), descriptor.route_handle,
	)


#============================================
def _active_tab(
		window: ferrum_qt.main_window.MainWindow,
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Return the current registered document tab."""
	tab = window._tab_widget.currentWidget()
	assert isinstance(tab, ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab)
	return tab


#============================================
def _flush_queued_calls(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Deliver queued Qt slots without a clock-based wait."""
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.MetaCall,
	)
	qapp.processEvents()


#============================================
def _capture_settlements(
		monkeypatch: pytest.MonkeyPatch,
		window: ferrum_qt.main_window.MainWindow,
		) -> list[ferrum_qt.ferrum.operation_leases.OperationLease]:
	"""Observe exact terminal receipts without altering registry behavior."""
	receipts: list[ferrum_qt.ferrum.operation_leases.OperationLease] = []
	settle = window._operation_leases.settle

	def capture(
			capability: ferrum_qt.ferrum.operation_leases.LeaseOwnerCapability,
			lease: ferrum_qt.ferrum.operation_leases.OperationLease,
			terminal: ferrum_qt.ferrum.operation_leases.LeaseState,
			) -> ferrum_qt.ferrum.operation_leases.OperationLease:
		receipt = settle(capability, lease, terminal)
		receipts.append(receipt)
		return receipt

	monkeypatch.setattr(window._operation_leases, "settle", capture)
	return receipts


#============================================
def _start_held_open(
		monkeypatch: pytest.MonkeyPatch,
		window: ferrum_qt.main_window.MainWindow,
		path: pathlib.Path,
		worker: _PreparedThenHeldWorker,
		) -> object:
	"""Start one controlled request and return its exact delivery owner."""
	controller = window._local_document_open_controller
	monkeypatch.setattr(
		controller, "_create_local_document_open_worker",
		lambda _path, _route_handle: worker,
	)
	assert window.open_file_path(str(path), interactive=True)
	worker.wait_until_prepared()
	delivery = controller._local_document_open_delivery
	assert delivery is not None
	return delivery


#============================================
def _finish_worker(
		qapp: PySide6.QtWidgets.QApplication,
		worker: _PreparedThenHeldWorker,
		) -> None:
	"""Release a controlled worker and flush its queued terminal signal."""
	worker.finish_safely()
	_flush_queued_calls(qapp)


#============================================
def _flush_deferred_deletes(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Complete controller-requested QObject retirement deterministically."""
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.DeferredDelete,
	)
	qapp.processEvents()


#============================================
def _settle_remaining_local_open_lease(
		window: ferrum_qt.main_window.MainWindow,
		source: object,
		) -> None:
	"""Release a deliberately faulted registry record after controller assertions."""
	controller = window._local_document_open_controller
	for lease in window._operation_leases.active_for_tab(source):
		if lease.family is ferrum_qt.ferrum.operation_leases.OperationFamily.LOCAL_DOCUMENT_OPEN:
			window._operation_leases.settle(
				controller._local_document_open_capability, lease,
				ferrum_qt.ferrum.operation_leases.LeaseState.FAILED,
			)


#============================================
def test_worker_prepared_signal_only_stages_until_queued_finished(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The named relay stages a worker-thread result before it may install."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	source = _active_tab(main_window)
	worker = _PreparedThenHeldWorker(prepared)
	receipts = _capture_settlements(monkeypatch, main_window)
	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		monkeypatch.setattr(delivery, "_can_replace_pristine", lambda: False)
		_flush_queued_calls(qapp)
		assert (
			delivery.prepared is prepared
			and _active_tab(main_window) is source
			and len(receipts) == 0
			and main_window._operation_leases.active_for_tab(source)
		)
		_finish_worker(qapp, worker)
		assert (
			_active_tab(main_window) is not source
			and len(receipts) == 1
			and receipts[0].state
			is ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED
		)
	finally:
		_finish_worker(qapp, worker)


#============================================
def test_destroyed_relay_drops_later_queued_worker_callbacks(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A controller-owned relay cannot call into delivery after destruction."""
	owner = PySide6.QtCore.QObject()
	delivery = _RelayDelivery()
	relay = ferrum_qt.ferrum.local_document_open_delivery._LocalDocumentOpenWorkerRelay(
		delivery, owner,
	)
	sender = _RelaySender()
	sender.prepared.connect(
		relay.on_prepared, PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
	)
	sender.failed.connect(
		relay.on_failed, PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
	)
	sender.finished.connect(
		relay.on_finished, PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
	)
	owner.deleteLater()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.DeferredDelete,
	)
	sender.prepared.emit("receipt")
	sender.failed.emit("failure")
	sender.finished.emit()
	_flush_queued_calls(qapp)
	assert not shiboken6.isValid(relay)
	assert delivery.callbacks == []


#============================================
def test_worker_start_invariant_rolls_back_lease_delivery_and_qobjects(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A failed start leaves no pending Open transaction or live source lease."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	source = _active_tab(main_window)
	worker = _StartFailingWorker()
	receipts = _capture_settlements(monkeypatch, main_window)
	monkeypatch.setattr(
		main_window._local_document_open_controller,
		"_create_local_document_open_worker",
		lambda _path, _route_handle: worker,
	)
	with pytest.raises(RuntimeError, match="worker start invariant"):
		main_window.open_file_path(str(path), interactive=True)
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.DeferredDelete,
	)
	qapp.processEvents()
	assert (
		len(receipts) == 1
		and receipts[0].state is ferrum_qt.ferrum.operation_leases.LeaseState.FAILED
		and main_window._operation_leases.active_for_tab(source) == ()
		and main_window._local_document_open_controller.
		has_pending_local_document_open() is False
		and not shiboken6.isValid(worker)
	)


#============================================
def test_forced_new_tab_commits_candidate_through_terminal_retirement(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A forced NewTab keeps its published candidate after its worker retires."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	source = _active_tab(main_window)
	worker = _PreparedThenHeldWorker(prepared)
	receipts = _capture_settlements(monkeypatch, main_window)
	completed: list[tuple[str, bool]] = []

	def capture_completed(opened_path: str, success: bool) -> None:
		completed.append((opened_path, success))

	main_window.local_document_open_completed.connect(capture_completed)
	controller = main_window._local_document_open_controller
	monkeypatch.setattr(
		controller, "_create_local_document_open_worker",
		lambda _path, _route_handle: worker,
	)
	try:
		assert main_window.open_file_path(
			str(path), interactive=True, force_new_tab=True,
		)
		worker.wait_until_prepared()
		_flush_queued_calls(qapp)
		_finish_worker(qapp, worker)
		candidate = _active_tab(main_window)
		assert (
			candidate is not source
			and candidate in main_window._native_tabs_by_page
			and main_window._tab_widget.currentWidget() is candidate
			and not candidate.is_disposed
			and shiboken6.isValid(candidate)
			and source in main_window._native_tabs_by_page
			and not source.is_disposed
			and shiboken6.isValid(source)
		)
		assert (
			len(receipts) == 1
			and receipts[0].state
			is ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED
			and completed == [(str(path), True)]
		)
		_flush_deferred_deletes(qapp)
		assert (
			candidate in main_window._native_tabs_by_page
			and not candidate.is_disposed
			and shiboken6.isValid(candidate)
		)
	finally:
		main_window.local_document_open_completed.disconnect(capture_completed)
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)


#============================================
def test_typed_candidate_display_failure_disposes_candidate_and_fails_once(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A typed installation refusal preserves the source and retires its candidate."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	source = _active_tab(main_window)
	worker = _PreparedThenHeldWorker()
	receipts = _capture_settlements(monkeypatch, main_window)
	presented: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []
	candidates: list[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab] = []
	completed: list[tuple[str, bool]] = []
	original_factory = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab.from_admitted_local_open

	def capture_candidate(*args: object, **kwargs: object) -> object:
		candidate = original_factory(*args, **kwargs)
		candidates.append(candidate)
		return candidate

	def refuse_publication(_tab: object, resolution: object) -> None:
		resolution.refuse_publication()
		raise ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError("display refused")

	def capture_completed(opened_path: str, success: bool) -> None:
		completed.append((opened_path, success))

	main_window.local_document_open_completed.connect(capture_completed)
	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		monkeypatch.setattr(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			"from_admitted_local_open", staticmethod(capture_candidate),
		)
		monkeypatch.setattr(delivery, "_can_replace_pristine", lambda: False)
		delivery._host = dataclasses.replace(
			delivery._host,
			publish_open_tab=refuse_publication,
		)
		delivery._present_refusal = presented.append
		delivery.stage_prepared(worker, prepared)
		_finish_worker(qapp, worker)
		assert (
			_active_tab(main_window) is source
			and source in main_window._native_tabs_by_page
			and candidates[0].is_disposed
			and len(receipts) == 1
			and receipts[0].state
			is ferrum_qt.ferrum.operation_leases.LeaseState.FAILED
			and len(presented) == 1
			and presented[0].outcome is ferrum_qt.dialogs.refusal_presenter.
			RefusalOutcome.DOCUMENT_DISPLAY_FAILED
			and completed == [(str(path), False)]
		)
	finally:
		main_window.local_document_open_completed.disconnect(capture_completed)
		_finish_worker(qapp, worker)


#============================================
def test_invariant_error_escapes_after_failed_settlement_and_retirement(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Unexpected defects remain visible after the controller makes cleanup safe."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	worker = _PreparedThenHeldWorker()
	receipts = _capture_settlements(monkeypatch, main_window)
	presented: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []
	completed: list[tuple[str, bool]] = []

	def capture_completed(path: str, outcome: bool) -> None:
		completed.append((path, outcome))

	main_window.local_document_open_completed.connect(capture_completed)
	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		monkeypatch.setattr(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			"from_admitted_local_open",
			staticmethod(lambda *_args, **_kwargs: (_ for _ in ()).throw(RuntimeError("invariant"))),
		)
		delivery._present_refusal = presented.append
		delivery.stage_prepared(worker, prepared)
		with pytest.raises(RuntimeError, match="invariant"):
			main_window._local_document_open_controller._finish_local_document_open_delivery(
				delivery,
			)
		assert (
			delivery.retired
			and len(receipts) == 1
			and receipts[0].state
			is ferrum_qt.ferrum.operation_leases.LeaseState.FAILED
			and completed == [(str(path), False)]
			and presented == []
			and main_window._local_document_open_controller.
			has_pending_local_document_open() is False
		)
	finally:
		main_window.local_document_open_completed.disconnect(capture_completed)
		_finish_worker(qapp, worker)
		qapp.processEvents()


#============================================
def test_postcommit_presentation_error_preserves_completed_candidate(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A typed post-commit refresh error reports recovery without lying about commit."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	source = _active_tab(main_window)
	worker = _PreparedThenHeldWorker()
	receipts = _capture_settlements(monkeypatch, main_window)
	presented: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []

	def fail_refresh(_receipt: object) -> None:
		raise local_open_contract.LocalOpenPostCommitPresentationError("refresh failed")

	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		delivery._host = dataclasses.replace(
			delivery._host,
			finish_open_replacement=fail_refresh,
		)
		delivery._present_refusal = presented.append
		delivery.stage_prepared(worker, prepared)
		_finish_worker(qapp, worker)
		candidate = _active_tab(main_window)
		assert (
			candidate is not source
			and not candidate.is_disposed
			and source.is_disposed
			and len(receipts) == 0
			and len(presented) == 1
			and presented[0].outcome is ferrum_qt.dialogs.refusal_presenter.
			RefusalOutcome.DOCUMENT_DISPLAY_FAILED
		)
	finally:
		_finish_worker(qapp, worker)


#============================================
@pytest.mark.parametrize("faulted_callback", ["record_recent_success", "show_status"])
def test_late_new_tab_presentation_failure_keeps_completed_installation_truth(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		faulted_callback: str,
		) -> None:
	"""A post-install host defect cannot rewrite a committed NewTab receipt as failed."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	source = _active_tab(main_window)
	worker = _PreparedThenHeldWorker()
	receipts = _capture_settlements(monkeypatch, main_window)
	completed: list[tuple[str, bool]] = []

	def fail_late(*_args: object) -> None:
		raise RuntimeError(f"{faulted_callback} invariant")

	def capture_completed(opened_path: str, success: bool) -> None:
		completed.append((opened_path, success))

	main_window.local_document_open_completed.connect(capture_completed)
	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		monkeypatch.setattr(delivery, "_can_replace_pristine", lambda: False)
		delivery._host = dataclasses.replace(
			delivery._host, **{faulted_callback: fail_late},
		)
		delivery.stage_prepared(worker, prepared)
		with pytest.raises(RuntimeError, match=f"{faulted_callback} invariant"):
			main_window._local_document_open_controller._finish_local_document_open_delivery(
				delivery,
			)
		candidate = _active_tab(main_window)
		assert (
			candidate is not source
			and main_window._tab_widget.currentWidget() is candidate
			and candidate in main_window._native_tabs_by_page
			and not candidate.is_disposed
			and shiboken6.isValid(candidate)
			and not source.is_disposed
		)
		assert receipts[0].state is ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED
		assert completed == [(str(path), True)]
		assert delivery.retired and (
			main_window._local_document_open_controller.
			has_pending_local_document_open() is False
		)
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)
		assert (
			main_window._tab_widget.currentWidget() is candidate
			and candidate in main_window._native_tabs_by_page
			and not candidate.is_disposed
			and shiboken6.isValid(candidate)
			and shiboken6.isValid(source)
		)
	finally:
		main_window.local_document_open_completed.disconnect(capture_completed)
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)


#============================================
def test_late_explicit_postcommit_error_keeps_completed_replacement_receipt(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A callback after replacement cannot erase its exact completed source receipt."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	source = _active_tab(main_window)
	worker = _PreparedThenHeldWorker()
	receipts = _capture_settlements(monkeypatch, main_window)
	completed: list[tuple[str, bool]] = []

	controller = main_window._local_document_open_controller
	monkeypatch.setattr(
		controller, "_create_local_document_open_worker",
		lambda _path, _route_handle: worker,
	)
	def capture_completed(opened_path: str, success: bool) -> None:
		completed.append((opened_path, success))

	main_window.local_document_open_completed.connect(capture_completed)
	try:
		assert main_window.open_in_current_tab_path(str(path))
		worker.wait_until_prepared()
		delivery = controller._local_document_open_delivery
		assert delivery is not None
		finish_open_replacement = delivery._host.finish_open_replacement

		def fail_postcommit(receipt: object) -> None:
			finish_open_replacement(receipt)
			raise RuntimeError("postcommit invariant")

		delivery._host = dataclasses.replace(
			delivery._host, finish_open_replacement=fail_postcommit,
		)
		delivery.stage_prepared(worker, prepared)
		with pytest.raises(RuntimeError, match="postcommit invariant"):
			controller._finish_local_document_open_delivery(delivery)
		candidate = _active_tab(main_window)
		assert (
			candidate is not source
			and main_window._tab_widget.currentWidget() is candidate
			and candidate in main_window._native_tabs_by_page
			and not candidate.is_disposed
			and shiboken6.isValid(candidate)
			and source.is_disposed
		)
		assert receipts == []
		assert completed == [(str(path), True)]
		assert delivery.retired and controller.has_pending_local_document_open() is False
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)
		assert (
			main_window._tab_widget.currentWidget() is candidate
			and candidate in main_window._native_tabs_by_page
			and not candidate.is_disposed
			and shiboken6.isValid(candidate)
			and not shiboken6.isValid(source)
		)
	finally:
		main_window.local_document_open_completed.disconnect(capture_completed)
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)


#============================================
def test_startup_settlement_error_still_retires_the_unstarted_delivery(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A startup rollback clears its owned Qt objects even when settlement faults."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	source = _active_tab(main_window)
	worker = _StartFailingWorker()
	controller = main_window._local_document_open_controller
	settle = main_window._operation_leases.settle
	settlement_attempted: list[bool] = []

	def fail_settlement(*_args: object) -> object:
		settlement_attempted.append(True)
		raise ferrum_qt.ferrum.operation_leases.OperationLeaseError("settlement invariant")

	monkeypatch.setattr(controller, "_create_local_document_open_worker", lambda *_args: worker)
	monkeypatch.setattr(main_window._operation_leases, "settle", fail_settlement)
	try:
		with pytest.raises(RuntimeError, match="worker start invariant"):
			main_window.open_file_path(str(path), interactive=True)
	finally:
		monkeypatch.setattr(main_window._operation_leases, "settle", settle)
		if controller.has_pending_local_document_open() is False:
			_settle_remaining_local_open_lease(main_window, source)
	_flush_deferred_deletes(qapp)
	assert controller.has_pending_local_document_open() is False
	assert settlement_attempted == [True]
	assert not shiboken6.isValid(worker)


#============================================
def test_ordinary_finish_settlement_error_still_retires_delivery(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An ordinary terminal settlement defect cannot retain a finished Open owner."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	source = _active_tab(main_window)
	worker = _PreparedThenHeldWorker()
	controller = main_window._local_document_open_controller
	settle = main_window._operation_leases.settle
	settlement_attempted: list[bool] = []

	def fail_settlement(*_args: object) -> object:
		settlement_attempted.append(True)
		raise ferrum_qt.ferrum.operation_leases.OperationLeaseError("settlement invariant")

	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		monkeypatch.setattr(delivery, "_can_replace_pristine", lambda: False)
		delivery.stage_prepared(worker, prepared)
		monkeypatch.setattr(main_window._operation_leases, "settle", fail_settlement)
		with pytest.raises(
			ferrum_qt.ferrum.operation_leases.OperationLeaseError,
			match="settlement invariant",
			):
			controller._finish_local_document_open_delivery(delivery)
		assert delivery.retired and controller.has_pending_local_document_open() is False
		assert settlement_attempted
	finally:
		monkeypatch.setattr(main_window._operation_leases, "settle", settle)
		if controller.has_pending_local_document_open() is False:
			_settle_remaining_local_open_lease(main_window, source)
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)
	assert not shiboken6.isValid(worker) and not shiboken6.isValid(delivery.relay)

#============================================
def test_escaped_finish_settlement_error_still_retires_delivery(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An escaped invariant plus settlement fault leaves no controller-owned delivery."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	source = _active_tab(main_window)
	worker = _PreparedThenHeldWorker()
	controller = main_window._local_document_open_controller
	settle = main_window._operation_leases.settle
	settlement_attempted: list[bool] = []

	def fail_settlement(*_args: object) -> object:
		settlement_attempted.append(True)
		raise ferrum_qt.ferrum.operation_leases.OperationLeaseError("settlement invariant")

	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		monkeypatch.setattr(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			"from_admitted_local_open",
			staticmethod(
				lambda *_args, **_kwargs: (_ for _ in ()).throw(RuntimeError("delivery invariant")),
			),
		)
		delivery.stage_prepared(worker, prepared)
		monkeypatch.setattr(main_window._operation_leases, "settle", fail_settlement)
		with pytest.raises(RuntimeError, match="delivery invariant"):
			controller._finish_local_document_open_delivery(delivery)
		assert delivery.retired and controller.has_pending_local_document_open() is False
		assert settlement_attempted == [True]
	finally:
		monkeypatch.setattr(main_window._operation_leases, "settle", settle)
		if controller.has_pending_local_document_open() is False:
			_settle_remaining_local_open_lease(main_window, source)
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)
	assert not shiboken6.isValid(worker) and not shiboken6.isValid(delivery.relay)

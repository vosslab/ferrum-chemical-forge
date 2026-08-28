"""Terminal local-Open replacement lifecycle receipts."""

# Standard Library
import pathlib

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.operation_leases
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_EMPTY_CDML = '<cdml xmlns="urn:ferrum:cdml" version="1.0"/>'


#============================================
class _HeldWorker(PySide6.QtCore.QThread):
	"""Hold one Open worker until the test chooses its finished boundary."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self) -> None:
		"""Create a deterministic, non-preempted worker stand-in."""
		super().__init__()
		self._gate = PySide6.QtCore.QWaitCondition()
		self._lock = PySide6.QtCore.QMutex()
		self._entered = PySide6.QtCore.QSemaphore()
		self._released = False
		self._joined = False
		self.delivery_cancelled = False

	#============================================
	def run(self) -> None:
		"""Wait until the test permits the normal Qt finished signal."""
		self._entered.release()
		self._lock.lock()
		while not self._released:
			self._gate.wait(self._lock)
		self._lock.unlock()

	#============================================
	def cancel_delivery(self) -> None:
		"""Match the real worker's delivery-only cancellation contract."""
		self.delivery_cancelled = True

	#============================================
	def wait_until_started(self) -> None:
		"""Synchronize with worker start without clock-based polling."""
		self._entered.acquire()

	#============================================
	def finish_safely(self) -> None:
		"""End work, allowing Qt to deliver the worker's finished signal."""
		if self._joined:
			return
		self._lock.lock()
		self._released = True
		self._gate.wakeAll()
		self._lock.unlock()
		self.wait()
		self._joined = True


#============================================
def _make_window(
		qapp: PySide6.QtWidgets.QApplication,
		) -> ferrum_qt.main_window.MainWindow:
	"""Create the product window and its first registered document."""
	return ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)


#============================================
def _current_tab(
		window: ferrum_qt.main_window.MainWindow,
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Return the current registered Ferrum page."""
	tab = window._tab_widget.currentWidget()
	assert isinstance(tab, ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab)
	return tab


#============================================
def _close_window(
		qapp: PySide6.QtWidgets.QApplication,
		window: ferrum_qt.main_window.MainWindow,
		) -> None:
	"""Explicitly discard test-owned documents before Qt object teardown."""
	for tab in tuple(window._native_tabs_by_page.values()):
		result = window._close_native_tab_at(
			window._tab_widget.indexOf(tab),
			ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		)
		assert result is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
	window.close()
	window.deleteLater()
	qapp.processEvents()


#============================================
def _hold_open(
		monkeypatch: pytest.MonkeyPatch,
		window: ferrum_qt.main_window.MainWindow,
		) -> _HeldWorker:
	"""Make the next controller worker stop at its finished boundary."""
	worker = _HeldWorker()
	monkeypatch.setattr(
		window._local_document_open_controller, "_create_local_document_open_worker",
		lambda _path, _route: worker,
	)
	return worker


#============================================
def _prepared_cdml(path: pathlib.Path) -> object:
	"""Issue a real one-use Rust admission receipt for the supplied CDML."""
	descriptor = next(
		descriptor
		for descriptor in ferrum_chem.DocumentSession.local_document_open_descriptors_v2()
		if ".cdml" in descriptor.suffixes
	)
	return ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
		str(path), descriptor.route_handle,
	)


#============================================
def _deliver_finished(
		qapp: PySide6.QtWidgets.QApplication, worker: _HeldWorker, *,
		process_events: bool = True,
		) -> None:
	"""Flush the exact queued Qt finished relay without timing assumptions."""
	worker.finish_safely()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.MetaCall,
	)
	if process_events:
		qapp.processEvents()


#============================================
def _stage_prepared(
		window: ferrum_qt.main_window.MainWindow, worker: _HeldWorker, prepared: object,
		) -> None:
	"""Stage one receipt through the current per-intent delivery boundary."""
	delivery = window._local_document_open_controller._local_document_open_delivery
	assert delivery is not None
	delivery.stage_prepared(worker, prepared)


#============================================
def _capture_terminal_receipts(
		monkeypatch: pytest.MonkeyPatch,
		window: ferrum_qt.main_window.MainWindow,
		) -> list[ferrum_qt.ferrum.operation_leases.OperationLease]:
	"""Observe controller terminal settlement without changing registry behavior."""
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
def test_pristine_replacement_keeps_the_source_lease_until_worker_finish(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A pristine replacement commits before the original-source completion receipt."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _current_tab(window)
	worker = _hold_open(monkeypatch, window)
	receipts = _capture_terminal_receipts(monkeypatch, window)
	try:
		assert window.open_file_path(str(path), interactive=True)
		worker.wait_until_started()
		lease = next(iter(window._operation_leases.active_for_tab(source)))
		_stage_prepared(window, worker, _prepared_cdml(path))
		assert (
			_current_tab(window) is source
			and window._operation_leases.active_for_tab(source)[0].lease_id == lease.lease_id
			and window._operation_leases.active_for_tab(source)[0].tab_identity == lease.tab_identity
		)
		with pytest.raises(ferrum_qt.ferrum.operation_leases.OperationLeaseError):
			window._operation_leases.unregister_tab(source)
		_deliver_finished(qapp, worker)
		assert (
			_current_tab(window) is not source
			and source.is_disposed
			and receipts[-1].lease_id == lease.lease_id
			and receipts[-1].tab_identity == lease.tab_identity
			and receipts[-1].tab() is source
			and receipts[-1].state is ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED
		)
	finally:
		_deliver_finished(qapp, worker)
		_close_window(qapp, window)


#============================================
def test_explicit_replacement_rolls_back_the_exact_source_on_disposal_refusal(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A typed source-disposal refusal retires its candidate and fails its same lease."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _current_tab(window)
	worker = _hold_open(monkeypatch, window)
	receipts = _capture_terminal_receipts(monkeypatch, window)
	presented: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []

	def refuse_disposal() -> None:
		raise ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError("source refuses disposal")

	monkeypatch.setattr(source, "dispose", refuse_disposal)
	monkeypatch.setattr(
		window._local_document_open_controller, "_present_refusal",
		lambda request: presented.append(request),
	)
	try:
		assert window.open_in_current_tab_path(str(path))
		worker.wait_until_started()
		lease = next(iter(window._operation_leases.active_for_tab(source)))
		_stage_prepared(window, worker, _prepared_cdml(path))
		_deliver_finished(qapp, worker)
		assert (
			_current_tab(window) is source
			and source in window._native_tabs_by_page
			and window._operation_leases.bind_tab(source) == lease.tab_identity
			and receipts[-1].lease_id == lease.lease_id
			and receipts[-1].state is ferrum_qt.ferrum.operation_leases.LeaseState.FAILED
			and presented[-1].outcome is ferrum_qt.dialogs.refusal_presenter.
			RefusalOutcome.DOCUMENT_DISPLAY_FAILED
			and presented[-1].technical_details is not None
		)
	finally:
		monkeypatch.undo()
		_deliver_finished(qapp, worker)
		_close_window(qapp, window)


#============================================
def test_stale_explicit_fence_refuses_without_publishing_a_candidate(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A switched source stays intact when explicit replacement loses its fence."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _current_tab(window)
	other = window._create_empty_native_tab()
	window._register_native_tab(other, activate=False)
	worker = _hold_open(monkeypatch, window)
	receipts = _capture_terminal_receipts(monkeypatch, window)
	presented: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []
	try:
		assert window.open_in_current_tab_path(str(path))
		worker.wait_until_started()
		lease = next(iter(window._operation_leases.active_for_tab(source)))
		delivery = window._local_document_open_controller._local_document_open_delivery
		assert delivery is not None
		monkeypatch.setattr(delivery, "_present_refusal", presented.append)
		window._tab_widget.setCurrentIndex(window._tab_widget.indexOf(other))
		_stage_prepared(window, worker, _prepared_cdml(path))
		_deliver_finished(qapp, worker)
		assert (
			_current_tab(window) is other
			and source in window._native_tabs_by_page
			and not source.is_disposed
			and receipts[-1].lease_id == lease.lease_id
			and receipts[-1].state is ferrum_qt.ferrum.operation_leases.LeaseState.REFUSED
			and presented[-1].outcome is ferrum_qt.dialogs.refusal_presenter.
			RefusalOutcome.UNAVAILABLE_OPERATION
			and presented[-1].technical_details is not None
			and "did not replace the changed document" in presented[-1].technical_details
		)
	finally:
		_deliver_finished(qapp, worker)
		_close_window(qapp, window)


#============================================
def test_cancelled_explicit_replacement_settles_only_after_finished_without_candidate(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Cancellation leaves its original source published until safe worker finish."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _current_tab(window)
	worker = _hold_open(monkeypatch, window)
	receipts = _capture_terminal_receipts(monkeypatch, window)
	try:
		assert window.open_in_current_tab_path(str(path))
		worker.wait_until_started()
		lease = next(iter(window._operation_leases.active_for_tab(source)))
		window._local_document_open_controller._cancel_local_document_open()
		assert (
			_current_tab(window) is source
			and source in window._native_tabs_by_page
			and worker.delivery_cancelled
			and window._operation_leases.active_for_tab(source)[0].state
			is ferrum_qt.ferrum.operation_leases.LeaseState.CANCELLATION_REQUESTED
		)
		_deliver_finished(qapp, worker)
		assert (
			_current_tab(window) is source
			and not source.is_disposed
			and receipts[-1].lease_id == lease.lease_id
			and receipts[-1].state is ferrum_qt.ferrum.operation_leases.LeaseState.CANCELLED
		)
	finally:
		_deliver_finished(qapp, worker)
		_close_window(qapp, window)

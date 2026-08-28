"""Behavioral contracts for lease-owned local document Open."""

# Standard Library
import pathlib

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import pytest
import shiboken6

# local repo modules
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.local_document_open_types
import ferrum_qt.ferrum.operation_leases
import ferrum_qt.main_window
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.themes.theme_loader
import ferrum_qt.themes.theme_manager


_EMPTY_CDML = '<cdml xmlns="urn:ferrum:cdml" version="1.0"/>'


#============================================
class _HeldOpenWorker(PySide6.QtCore.QThread):
	"""A controlled finished boundary for a local Open worker contract."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)
	#============================================
	def __init__(
			self,
			registry: ferrum_qt.ferrum.operation_leases.OperationLeaseRegistry,
			source: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Retain only the observable source ownership during controlled work."""
		super().__init__()
		self._registry = registry
		self._source = source
		self._gate = PySide6.QtCore.QWaitCondition()
		self._gate_lock = PySide6.QtCore.QMutex()
		self._entered = PySide6.QtCore.QSemaphore()
		self._released = False
		self.delivery_cancelled = False
		self.state_at_start: ferrum_qt.ferrum.operation_leases.LeaseState | None = None

	#============================================
	def run(self) -> None:
		"""Observe that source ownership is active before work can begin."""
		lease = next(
			lease
			for lease in self._registry.active_for_tab(self._source)
			if lease.family is ferrum_qt.ferrum.operation_leases.OperationFamily.LOCAL_DOCUMENT_OPEN
		)
		self.state_at_start = lease.state
		self._entered.release()
		self._gate_lock.lock()
		while not self._released:
			self._gate.wait(self._gate_lock)
		self._gate_lock.unlock()

	#============================================
	def cancel_delivery(self) -> None:
		"""Model delivery invalidation without pretending worker preemption."""
		self.delivery_cancelled = True

	#============================================
	def finish_safely(self) -> None:
		"""Publish the exact worker-finished boundary on demand."""
		self._gate_lock.lock()
		self._released = True
		self._gate.wakeAll()
		self._gate_lock.unlock()
		self.wait()

	#============================================
	def wait_until_started(self) -> None:
		"""Wait for the worker to observe its already-acquired source lease."""
		self._entered.acquire()


#============================================
def _make_window(
		qapp: PySide6.QtWidgets.QApplication,
		) -> ferrum_qt.main_window.MainWindow:
	"""Create the canonical Ferrum product window with its bootstrap tab."""
	return ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)


#============================================
def _active_tab(
		window: ferrum_qt.main_window.MainWindow,
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Return the visible active Ferrum document through its central tab host."""
	tabs = window.centralWidget()
	assert isinstance(tabs, PySide6.QtWidgets.QTabWidget)
	tab = tabs.currentWidget()
	assert isinstance(tab, ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab)
	return tab


#============================================
def _action(
		window: ferrum_qt.main_window.MainWindow, label: str,
		) -> PySide6.QtGui.QAction:
	"""Return one visible File command by its author-facing label."""
	return next(
		action
		for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == label
	)


#============================================
def _local_open_lease(
		window: ferrum_qt.main_window.MainWindow,
		source: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> ferrum_qt.ferrum.operation_leases.OperationLease:
	"""Return the active Open ownership of its exact source document."""
	return next(
		lease
		for lease in window._operation_leases.active_for_tab(source)
		if lease.family is ferrum_qt.ferrum.operation_leases.OperationFamily.LOCAL_DOCUMENT_OPEN
	)


#============================================
def _close_window(
		qapp: PySide6.QtWidgets.QApplication,
		window: ferrum_qt.main_window.MainWindow,
		) -> None:
	"""Explicitly discard test-owned documents before ordinary Qt teardown."""
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
def _hold_next_open(
		monkeypatch: pytest.MonkeyPatch,
		window: ferrum_qt.main_window.MainWindow,
		source: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> _HeldOpenWorker:
	"""Install one deterministic worker whose finish is controlled by the test."""
	controller = window._local_document_open_controller
	worker = _HeldOpenWorker(window._operation_leases, source)
	monkeypatch.setattr(
		controller, "_create_local_document_open_worker",
		lambda _path, _route: worker,
	)
	return worker


#============================================
def _hold_queued_opens(
		monkeypatch: pytest.MonkeyPatch,
		window: ferrum_qt.main_window.MainWindow,
		source: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[_HeldOpenWorker, _HeldOpenWorker]:
	"""Install independently controlled workers for two sequential Open requests."""
	first = _HeldOpenWorker(window._operation_leases, source)
	second = _HeldOpenWorker(window._operation_leases, source)
	pending_workers = [first, second]
	controller = window._local_document_open_controller
	monkeypatch.setattr(
		controller, "_create_local_document_open_worker",
		lambda _path, _route: pending_workers.pop(0),
	)
	return first, second


#============================================
def _finish_worker(
		qapp: PySide6.QtWidgets.QApplication, worker: _HeldOpenWorker,
		) -> None:
	"""Deliver the queued Qt finished relay without a clock-based wait."""
	worker.finish_safely()
	qapp.processEvents()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.MetaCall,
	)
	qapp.processEvents()


#============================================
def _release_worker_for_teardown(
		qapp: PySide6.QtWidgets.QApplication, worker: _HeldOpenWorker,
		) -> None:
	"""Release a still-live held worker; finished workers are terminally deleted."""
	if shiboken6.isValid(worker) and worker.isRunning():
		_finish_worker(qapp, worker)


#============================================
def _deliver_prepared(
		qapp: PySide6.QtWidgets.QApplication,
		worker: _HeldOpenWorker, prepared: object,
		) -> None:
	"""Deliver one Rust admission receipt through the Qt-thread controller relay."""
	worker.prepared.emit(prepared)
	qapp.processEvents()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.MetaCall,
	)
	qapp.processEvents()


#============================================
def _deliver_failure(
		qapp: PySide6.QtWidgets.QApplication,
		worker: _HeldOpenWorker,
		failure: ferrum_qt.ferrum.local_document_open_types.
		FerrumNativeLocalDocumentOpenFailure,
		) -> None:
	"""Deliver one typed admission failure through the controller's Qt relay."""
	worker.failed.emit(failure)
	qapp.processEvents()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.MetaCall,
	)
	qapp.processEvents()


#============================================
def test_open_binds_a_live_bootstrap_source_before_worker_start(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Every Open owns the exact visible bootstrap until the worker finishes."""
	source_path = tmp_path / "source.cdml"
	source_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _active_tab(window)
	worker = _hold_next_open(monkeypatch, window, source)
	try:
		assert window._local_document_open_controller.open_file_path(str(source_path))
		worker.wait_until_started()
		lease = _local_open_lease(window, source)
		assert (
			worker.state_at_start is ferrum_qt.ferrum.operation_leases.LeaseState.ACTIVE
			and lease.tab() is source
			and lease.close_policy is ferrum_qt.ferrum.operation_leases.ClosePolicy.BLOCK_UNTIL_SETTLED
		)
		_finish_worker(qapp, worker)
	finally:
		_release_worker_for_teardown(qapp, worker)
		_close_window(qapp, window)


#============================================
def test_file_open_actions_keep_their_registered_identity_and_safe_cancel_help(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The controller preserves menu clients while explaining cancellation safely."""
	window = _make_window(qapp)
	try:
		cancel = _action(window, "Cancel Open")
		assert (
			window._action_registry.get_qt_action("file.open") is _action(window, "Open")
			and window._action_registry.get_qt_action("file.open_current")
			is _action(window, "Open in Current Tab...")
			and window._action_registry.get_qt_action("file.open.cancel") is cancel
		)
		assert "finish" in cancel.toolTip().lower()
	finally:
		_close_window(qapp, window)


#============================================
def test_cancel_retains_source_while_disclosing_safe_finish_and_queue_removal(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Cancelling suppresses delivery but leaves the source leased until finished."""
	first = tmp_path / "first.cdml"
	second = tmp_path / "second.cdml"
	first.write_text(_EMPTY_CDML, encoding="utf-8")
	second.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _active_tab(window)
	worker = _hold_next_open(monkeypatch, window, source)
	try:
		controller = window._local_document_open_controller
		assert controller.open_file_path(str(first))
		worker.wait_until_started()
		assert controller.open_file_path(str(second))
		_action(window, "Cancel Open").trigger()
		lease = _local_open_lease(window, source)
		assert (
			lease.state is ferrum_qt.ferrum.operation_leases.LeaseState.CANCELLATION_REQUESTED
			and source in window._native_tabs_by_page
			and worker.delivery_cancelled
			and "Cancelling" in window.statusBar().currentMessage()
			and "1" in window.statusBar().currentMessage()
		)
		assert (
			not _action(window, "Open").isEnabled()
			and not _action(window, "Open in Current Tab...").isEnabled()
			and not _action(window, "Cancel Open").isEnabled()
		)
		_finish_worker(qapp, worker)
		assert (
			not window._operation_leases.has_active(
				ferrum_qt.ferrum.operation_leases.OperationFamily.LOCAL_DOCUMENT_OPEN,
				source,
			)
			and not controller.has_pending_local_document_open()
			and "Finished" in window.statusBar().currentMessage()
		)
	finally:
		_release_worker_for_teardown(qapp, worker)
		_close_window(qapp, window)


#============================================
def test_new_document_completion_after_tab_switch_does_not_steal_focus(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A finished New Tab Open leaves an author working on another tab in place."""
	source_path = tmp_path / "source.cdml"
	source_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _active_tab(window)
	other = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EMPTY_CDML, "other.cdml",
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	window._register_native_tab(other, activate=False)
	worker = _hold_next_open(monkeypatch, window, source)
	try:
		assert window.open_file_path(str(source_path))
		worker.wait_until_started()
		window._tab_widget.setCurrentWidget(other)
		window.show()
		other.view.viewport().setFocus()
		qapp.processEvents()
		descriptor = window._local_document_open_catalog.descriptor_for_path(
			str(source_path),
		)
		assert descriptor is not None
		prepared = ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
			str(source_path), descriptor.route_handle,
		)
		_deliver_prepared(qapp, worker, prepared)
		_finish_worker(qapp, worker)
		assert (
			_active_tab(window) is other
			and qapp.focusWidget() is other.view
		)
	finally:
		_release_worker_for_teardown(qapp, worker)
		_close_window(qapp, window)


#============================================
def test_queued_failure_cannot_poison_the_following_prepared_receipt(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A later Open owns its receipt and author-facing refusal independently."""
	first_path = tmp_path / "first.cdml"
	second_path = tmp_path / "second.cdml"
	first_path.write_text(_EMPTY_CDML, encoding="utf-8")
	second_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _active_tab(window)
	first_worker, second_worker = _hold_queued_opens(monkeypatch, window, source)
	refusals: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []
	monkeypatch.setattr(window, "_show_edit_refusal", refusals.append)
	try:
		assert window.open_file_path(str(first_path))
		first_worker.wait_until_started()
		assert window.open_file_path(str(second_path))
		failure = (
			ferrum_qt.ferrum.local_document_open_types.
			FerrumNativeLocalDocumentOpenFailure(
				"DocumentInputError", "sensitive-token:" + str(first_path),
				"utf8", None, None, None,
			)
		)
		_deliver_failure(qapp, first_worker, failure)
		_finish_worker(qapp, first_worker)
		second_worker.wait_until_started()
		descriptor = window._local_document_open_catalog.descriptor_for_path(
			str(second_path),
		)
		assert descriptor is not None
		prepared = ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
			str(second_path), descriptor.route_handle,
		)
		_deliver_prepared(qapp, second_worker, prepared)
		_finish_worker(qapp, second_worker)
		opened = next(
			tab for tab in window._native_tabs_by_page.values()
			if tab.file_path == second_path
		)
		presentation = ferrum_qt.dialogs.refusal_presenter.present_refusal(refusals[0])
		assert opened.file_path == second_path
		assert (
			first_path.name in presentation.ordinary_text()
			and str(first_path) not in presentation.ordinary_text()
			and "sensitive-token" not in presentation.ordinary_text()
			and all(refusal.document_name != second_path.name for refusal in refusals)
		)
	finally:
		_release_worker_for_teardown(qapp, first_worker)
		_release_worker_for_teardown(qapp, second_worker)
		_close_window(qapp, window)


#============================================
def test_queued_success_receipt_cannot_be_reused_by_a_silent_follower(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A settled receipt cannot mutate the later request when it has none of its own."""
	first_path = tmp_path / "first.cdml"
	second_path = tmp_path / "second.cdml"
	first_path.write_text(_EMPTY_CDML, encoding="utf-8")
	second_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _active_tab(window)
	first_worker, second_worker = _hold_queued_opens(monkeypatch, window, source)
	refusals: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []
	monkeypatch.setattr(window, "_show_edit_refusal", refusals.append)
	try:
		assert window.open_file_path(str(first_path))
		first_worker.wait_until_started()
		assert window.open_file_path(str(second_path))
		descriptor = window._local_document_open_catalog.descriptor_for_path(
			str(first_path),
		)
		assert descriptor is not None
		prepared = ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
			str(first_path), descriptor.route_handle,
		)
		_deliver_prepared(qapp, first_worker, prepared)
		_finish_worker(qapp, first_worker)
		second_worker.wait_until_started()
		_finish_worker(qapp, second_worker)
		assert (
			any(tab.file_path == first_path for tab in window._native_tabs_by_page.values())
			and not any(tab.file_path == second_path for tab in window._native_tabs_by_page.values())
			and all(refusal.document_name != second_path.name for refusal in refusals)
		)
	finally:
		_release_worker_for_teardown(qapp, first_worker)
		_release_worker_for_teardown(qapp, second_worker)
		_close_window(qapp, window)


#============================================
def test_queued_request_with_a_disposed_source_is_refused_without_worker_launch(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A queued request never re-anchors after its captured source is replaced."""
	first_path = tmp_path / "first.cdml"
	second_path = tmp_path / "second.cdml"
	first_path.write_text(_EMPTY_CDML, encoding="utf-8")
	second_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _active_tab(window)
	first_worker, second_worker = _hold_queued_opens(monkeypatch, window, source)
	completed: list[tuple[str, bool]] = []
	drained = False
	drained_twice = False

	def receive_completion(path: str, success: bool) -> None:
		"""Retain the programmatic receipt without inspecting controller payloads."""
		completed.append((path, success))

	def receive_drain(_success: bool) -> None:
		"""Detect an accidental duplicate batch terminal signal."""
		nonlocal drained, drained_twice
		drained_twice = drained
		drained = True

	window.local_document_open_completed.connect(receive_completion)
	window.local_document_open_queue_drained.connect(receive_drain)
	try:
		assert window.open_file_path(str(first_path), interactive=True)
		first_worker.wait_until_started()
		assert window.open_file_path(str(second_path))
		descriptor = window._local_document_open_catalog.descriptor_for_path(
			str(first_path),
		)
		assert descriptor is not None
		prepared = ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
			str(first_path), descriptor.route_handle,
		)
		_deliver_prepared(qapp, first_worker, prepared)
		_finish_worker(qapp, first_worker)
		assert (
			source.is_disposed
			and not second_worker.isRunning()
			and any(path == str(second_path) and not success for path, success in completed)
			and drained and not drained_twice
			and _action(window, "Open").isEnabled()
		)
	finally:
		window.local_document_open_completed.disconnect(receive_completion)
		window.local_document_open_queue_drained.disconnect(receive_drain)
		_release_worker_for_teardown(qapp, first_worker)
		_release_worker_for_teardown(qapp, second_worker)
		_close_window(qapp, window)


#============================================
def test_conflicting_worker_results_fail_after_cleanup_without_a_generic_refusal(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A protocol violation settles Failed, cleans up, and remains actionable."""
	first_path = tmp_path / "first.cdml"
	second_path = tmp_path / "second.cdml"
	first_path.write_text(_EMPTY_CDML, encoding="utf-8")
	second_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _active_tab(window)
	first_worker, second_worker = _hold_queued_opens(monkeypatch, window, source)
	refusals: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []
	monkeypatch.setattr(window, "_show_edit_refusal", refusals.append)
	try:
		assert window.open_file_path(str(first_path))
		first_worker.wait_until_started()
		descriptor = window._local_document_open_catalog.descriptor_for_path(
			str(first_path),
		)
		assert descriptor is not None
		prepared = ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
			str(first_path), descriptor.route_handle,
		)
		failure = (
			ferrum_qt.ferrum.local_document_open_types.
			FerrumNativeLocalDocumentOpenFailure(
				"DocumentInputError", "protocol-token:" + str(first_path),
				"utf8", None, None, None,
			)
		)
		_deliver_prepared(qapp, first_worker, prepared)
		_deliver_failure(qapp, first_worker, failure)
		assert _active_tab(window) is source and source.file_path is None
		delivery = window._local_document_open_controller._local_document_open_delivery
		assert delivery is not None
		first_worker.finish_safely()
		with pytest.raises(RuntimeError, match="more than one admission result"):
			window._local_document_open_controller._finish_local_document_open_delivery(
				delivery,
			)
		PySide6.QtCore.QCoreApplication.sendPostedEvents(
			None, PySide6.QtCore.QEvent.Type.MetaCall,
		)
		qapp.processEvents()
		assert (
			_active_tab(window) is source
			and source.file_path is None
			and not window._operation_leases.active_for_tab(source)
			and not window._local_document_open_controller.has_pending_local_document_open()
			and not refusals
		)
		assert window.open_file_path(str(second_path))
		second_worker.wait_until_started()
		descriptor = window._local_document_open_catalog.descriptor_for_path(
			str(second_path),
		)
		assert descriptor is not None
		second_prepared = ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
			str(second_path), descriptor.route_handle,
		)
		_deliver_prepared(qapp, second_worker, second_prepared)
		_finish_worker(qapp, second_worker)
		assert any(
			tab.file_path == second_path for tab in window._native_tabs_by_page.values()
		)
	finally:
		_release_worker_for_teardown(qapp, first_worker)
		_release_worker_for_teardown(qapp, second_worker)
		_close_window(qapp, window)


#============================================
def test_non_source_close_does_not_release_the_active_source_lease(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An unrelated tab retains its ordinary Close behavior during local Open."""
	source_path = tmp_path / "source.cdml"
	source_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _active_tab(window)
	other = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EMPTY_CDML, "other.cdml",
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	window._register_native_tab(other, activate=False)
	worker = _hold_next_open(monkeypatch, window, source)
	try:
		assert window._local_document_open_controller.open_file_path(str(source_path))
		worker.wait_until_started()
		result = window._close_native_tab_at(
			window._tab_widget.indexOf(other),
			ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		)
		assert (
			result is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
			and other.is_disposed
			and _local_open_lease(window, source).state
			is ferrum_qt.ferrum.operation_leases.LeaseState.ACTIVE
		)
		_finish_worker(qapp, worker)
	finally:
		_release_worker_for_teardown(qapp, worker)
		_close_window(qapp, window)


#============================================
def test_source_close_requests_cancellation_and_succeeds_only_after_finish(
		qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The source tab remains registered through its safe-finish boundary."""
	source_path = tmp_path / "source.cdml"
	source_path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(qapp)
	source = _active_tab(window)
	worker = _hold_next_open(monkeypatch, window, source)
	try:
		assert window.open_file_path(str(source_path))
		worker.wait_until_started()
		blocked = window._close_native_tab_at(
			window._tab_widget.indexOf(source),
			ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		)
		assert (
			blocked is ferrum_qt.ferrum.close_decision.CloseResult.
			LOCAL_DOCUMENT_OPEN_CANCELLATION_REQUESTED
			and source in window._native_tabs_by_page
			and _local_open_lease(window, source).state
			is ferrum_qt.ferrum.operation_leases.LeaseState.CANCELLATION_REQUESTED
		)
		_finish_worker(qapp, worker)
		closed = window._close_native_tab_at(
			window._tab_widget.indexOf(source),
			ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		)
		assert closed is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
	finally:
		_release_worker_for_teardown(qapp, worker)
		_close_window(qapp, window)

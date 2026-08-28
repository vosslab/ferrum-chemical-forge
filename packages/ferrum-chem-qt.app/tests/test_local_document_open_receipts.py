"""Receipt-bound Local Open publication and replacement regressions."""

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
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.local_document_open_contract as local_open_contract
import ferrum_qt.ferrum.local_document_open_delivery
import ferrum_qt.ferrum.operation_leases
import ferrum_qt.main_window


_EMPTY_CDML = '<cdml xmlns="urn:ferrum:cdml" version="1.0"/>'


#============================================
class _HeldWorker(PySide6.QtCore.QThread):
	"""Keep one admitted Local Open delivery live until test cleanup."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self) -> None:
		"""Create a worker that confirms startup before blocking."""
		super().__init__()
		self._gate = PySide6.QtCore.QWaitCondition()
		self._lock = PySide6.QtCore.QMutex()
		self._started = PySide6.QtCore.QSemaphore()
		self._released = False
		self.delivery_cancelled = False

	#============================================
	def run(self) -> None:
		"""Wait until the test closes this exact worker."""
		self._started.release()
		self._lock.lock()
		while not self._released:
			self._gate.wait(self._lock)
		self._lock.unlock()

	#============================================
	def cancel_delivery(self) -> None:
		"""Provide the delivery cancellation protocol."""
		self.delivery_cancelled = True

	#============================================
	def wait_until_started(self) -> None:
		"""Synchronize startup without a wall-clock delay."""
		self._started.acquire()

	#============================================
	def finish_safely(self) -> None:
		"""Release and join the worker if it is still live."""
		if not shiboken6.isValid(self) or not self.isRunning():
			return
		self._lock.lock()
		self._released = True
		self._gate.wakeAll()
		self._lock.unlock()
		self.wait()


#============================================
def _prepared_cdml(path: pathlib.Path) -> object:
	"""Issue one real Rust local-document admission receipt."""
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
	"""Return the exact current registered native tab."""
	tab = window._tab_widget.currentWidget()
	assert isinstance(tab, ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab)
	return tab


#============================================
def _start_held_open(
		monkeypatch: pytest.MonkeyPatch,
		window: ferrum_qt.main_window.MainWindow,
		path: pathlib.Path, worker: _HeldWorker,
		) -> ferrum_qt.ferrum.local_document_open_delivery.LocalDocumentOpenDelivery:
	"""Start one exact controller transaction with a controlled worker."""
	controller = window._local_document_open_controller
	monkeypatch.setattr(
		controller, "_create_local_document_open_worker", lambda *_args: worker,
	)
	assert window.open_file_path(str(path), interactive=True)
	worker.wait_until_started()
	delivery = controller._local_document_open_delivery
	assert delivery is not None
	return delivery


#============================================
def _finish_worker(qapp: PySide6.QtWidgets.QApplication, worker: _HeldWorker) -> None:
	"""Join a test worker and flush the exact queued terminal callback."""
	worker.finish_safely()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.MetaCall,
	)
	qapp.processEvents()


#============================================
def _flush_deferred_deletes(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Complete controller-owned QObject retirement."""
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.DeferredDelete,
	)
	qapp.processEvents()


#============================================
def _capture_settlements(
		monkeypatch: pytest.MonkeyPatch,
		window: ferrum_qt.main_window.MainWindow,
		) -> list[ferrum_qt.ferrum.operation_leases.OperationLease]:
	"""Observe ordinary controller terminal settlement facts."""
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
def _settle_remaining_local_open_lease(
		window: ferrum_qt.main_window.MainWindow, source: object,
		) -> None:
	"""Release a deliberately refused source lease during test cleanup."""
	controller = window._local_document_open_controller
	for lease in window._operation_leases.active_for_tab(source):
		if lease.family is ferrum_qt.ferrum.operation_leases.OperationFamily.LOCAL_DOCUMENT_OPEN:
			window._operation_leases.settle(
				controller._local_document_open_capability, lease,
				ferrum_qt.ferrum.operation_leases.LeaseState.FAILED,
			)


#============================================
@pytest.mark.parametrize("fault_kind", ["typed", "unexpected"])
def test_new_tab_receipt_transfers_before_presentation_fault(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch,
		fault_kind: str,
		) -> None:
	"""A published NewTab remains valid when its later presentation faults."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	worker = _HeldWorker()
	receipts = _capture_settlements(monkeypatch, main_window)
	candidates: list[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab] = []
	factory = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab.from_admitted_local_open

	def capture_candidate(*args: object, **kwargs: object) -> object:
		candidate = factory(*args, **kwargs)
		candidates.append(candidate)
		return candidate

	def fail_presentation(*_args: object) -> None:
		if fault_kind == "typed":
			raise local_open_contract.LocalOpenPostCommitPresentationError("typed fault")
		raise RuntimeError("unexpected presentation fault")

	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		monkeypatch.setattr(delivery, "_can_replace_pristine", lambda: False)
		monkeypatch.setattr(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			"from_admitted_local_open", staticmethod(capture_candidate),
		)
		delivery._host = dataclasses.replace(
			delivery._host, finish_open_publication=fail_presentation,
		)
		delivery._present_refusal = lambda _request: None
		delivery.stage_prepared(worker, prepared)
		if fault_kind == "unexpected":
			with pytest.raises(RuntimeError, match="unexpected presentation fault"):
				main_window._local_document_open_controller._finish_local_document_open_delivery(
					delivery,
				)
		else:
			main_window._local_document_open_controller._finish_local_document_open_delivery(delivery)
		candidate = candidates[0]
		assert main_window._native_tabs_by_page.get(candidate) is candidate
		assert not candidate.is_disposed and shiboken6.isValid(candidate)
		assert delivery.outcome is local_open_contract.LocalDocumentOpenOutcome.COMPLETED
		assert receipts[0].state is ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED
	finally:
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)


#============================================
@pytest.mark.parametrize("fault_kind", ["typed", "unexpected"])
def test_replacement_receipt_transfers_before_presentation_fault(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch,
		fault_kind: str,
		) -> None:
	"""A committed replacement remains current after later display cleanup faults."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	old = _active_tab(main_window)
	worker = _HeldWorker()
	controller = main_window._local_document_open_controller
	settled: list[ferrum_qt.ferrum.operation_leases.OperationLease] = []
	complete = main_window._operation_leases.complete_prepared_terminal_replacement

	def capture_completion(
			prepared_replacement: object, observer: object,
			) -> None:
		def capture_observer(receipt: object) -> None:
			settled.append(receipt)
			observer(receipt)
		complete(prepared_replacement, capture_observer)

	def fail_presentation(_receipt: object) -> None:
		if fault_kind == "typed":
			raise local_open_contract.LocalOpenPostCommitPresentationError("typed fault")
		raise RuntimeError("unexpected replacement fault")

	monkeypatch.setattr(controller, "_create_local_document_open_worker", lambda *_args: worker)
	monkeypatch.setattr(
		main_window._operation_leases,
		"complete_prepared_terminal_replacement", capture_completion,
	)
	try:
		assert main_window.open_in_current_tab_path(str(path))
		worker.wait_until_started()
		delivery = controller._local_document_open_delivery
		assert delivery is not None
		delivery._host = dataclasses.replace(
			delivery._host, finish_open_replacement=fail_presentation,
		)
		delivery._present_refusal = lambda _request: None
		delivery.stage_prepared(worker, prepared)
		if fault_kind == "unexpected":
			with pytest.raises(RuntimeError, match="unexpected replacement fault"):
				controller._finish_local_document_open_delivery(delivery)
		else:
			controller._finish_local_document_open_delivery(delivery)
		candidate = _active_tab(main_window)
		assert candidate is not old and main_window._native_tabs_by_page.get(candidate) is candidate
		assert not candidate.is_disposed and old.is_disposed
		assert settled[0].state is ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED
		assert delivery.replacement_lease_settled
	finally:
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)


#============================================
def test_new_tab_integration_refusal_unpublishes_then_disposes_candidate(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A pre-publication host refusal leaves delivery as the candidate's owner."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	source = _active_tab(main_window)
	worker = _HeldWorker()
	receipts = _capture_settlements(monkeypatch, main_window)
	candidates: list[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab] = []
	factory = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab.from_admitted_local_open
	finish_registration = main_window._finish_native_tab_registration

	def capture_candidate(*args: object, **kwargs: object) -> object:
		candidate = factory(*args, **kwargs)
		candidates.append(candidate)
		return candidate

	def refuse_after_integration(
			candidate: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		finish_registration(candidate)
		raise ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError("integration refused")

	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		monkeypatch.setattr(delivery, "_can_replace_pristine", lambda: False)
		monkeypatch.setattr(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			"from_admitted_local_open", staticmethod(capture_candidate),
		)
		monkeypatch.setattr(main_window, "_finish_native_tab_registration", refuse_after_integration)
		delivery._present_refusal = lambda _request: None
		delivery.stage_prepared(worker, prepared)
		main_window._local_document_open_controller._finish_local_document_open_delivery(delivery)
		candidate = candidates[0]
		assert (
			_active_tab(main_window) is source
			and candidate not in main_window._native_tabs_by_page
			and main_window._tab_widget.indexOf(candidate) < 0
			and candidate.is_disposed
			and receipts[0].state is ferrum_qt.ferrum.operation_leases.LeaseState.FAILED
		)
	finally:
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)


#============================================
@pytest.mark.parametrize("failure_stage", ["prepare", "dispose"])
def test_replacement_refusal_restores_active_source_then_disposes_candidate(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch,
		failure_stage: str,
		) -> None:
	"""A recoverable replacement refusal preserves source authority for cleanup."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	prepared = _prepared_cdml(path)
	source = _active_tab(main_window)
	worker = _HeldWorker()
	transaction: object | None = None
	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		session, observation, token, source_kind, _summary = prepared.take_admission_v2()
		transaction = ferrum_qt.ferrum.local_document_open_delivery._AdmittedCandidateTransaction(
			delivery,
		)
		transaction.build(session, observation, source_kind, token)
		candidate = transaction._require_candidate()
		if failure_stage == "prepare":
			def refuse_prepare(*_args: object) -> object:
				raise ferrum_qt.ferrum.operation_leases.OperationLeaseError("prepare refused")

			monkeypatch.setattr(
				main_window._operation_leases, "prepare_terminal_replacement", refuse_prepare,
			)
			expected = ferrum_qt.ferrum.operation_leases.OperationLeaseError
		else:
			def refuse_dispose() -> None:
				raise ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError("dispose refused")

			monkeypatch.setattr(source, "dispose", refuse_dispose)
			expected = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError
		with pytest.raises(expected):
			transaction.replace_open_tab(source, main_window._tab_widget.indexOf(source))
		assert (
			_active_tab(main_window) is source
			and main_window._native_tabs_by_page.get(source) is source
			and main_window._operation_leases.active_for_tab(source) == (delivery.intent.lease,)
			and candidate not in main_window._native_tabs_by_page
			and main_window._tab_widget.indexOf(candidate) < 0
		)
		transaction.dispose_uncommitted()
		assert candidate.is_disposed
	finally:
		monkeypatch.undo()
		if transaction is not None:
			transaction.dispose_uncommitted()
		_settle_remaining_local_open_lease(main_window, source)
		main_window._local_document_open_controller._clear_current_local_document_open(
			delivery.intent,
		)
		delivery._finish_callback = lambda _delivery: None
		_finish_worker(qapp, worker)
		delivery.retire()
		_flush_deferred_deletes(qapp)


#============================================
def test_publication_resolution_survives_a_wrapper_fault(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A wrapper fault after real publication cannot reclaim the committed tab."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	worker = _HeldWorker()
	candidates: list[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab] = []
	factory = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab.from_admitted_local_open

	def capture_candidate(*args: object, **kwargs: object) -> object:
		candidate = factory(*args, **kwargs)
		candidates.append(candidate)
		return candidate

	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		publish = delivery._host.publish_open_tab

		def publish_then_raise(candidate: object, resolution: object) -> None:
			publish(candidate, resolution)
			raise RuntimeError("wrapper fault after publication")

		monkeypatch.setattr(delivery, "_can_replace_pristine", lambda: False)
		monkeypatch.setattr(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			"from_admitted_local_open", staticmethod(capture_candidate),
		)
		delivery._host = dataclasses.replace(
			delivery._host, publish_open_tab=publish_then_raise,
		)
		delivery._present_refusal = lambda _request: None
		delivery.stage_prepared(worker, _prepared_cdml(path))
		with pytest.raises(RuntimeError, match="wrapper fault after publication"):
			main_window._local_document_open_controller._finish_local_document_open_delivery(
				delivery,
			)
		candidate = candidates[0]
		assert (
			main_window._native_tabs_by_page.get(candidate) is candidate
			and not candidate.is_disposed
			and delivery.outcome is local_open_contract.LocalDocumentOpenOutcome.COMPLETED
		)
	finally:
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)


#============================================
def test_replacement_resolution_survives_a_wrapper_fault(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A wrapper fault after real replacement cannot unwind its settled swap."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	old = _active_tab(main_window)
	worker = _HeldWorker()
	candidates: list[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab] = []
	factory = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab.from_admitted_local_open

	def capture_candidate(*args: object, **kwargs: object) -> object:
		candidate = factory(*args, **kwargs)
		candidates.append(candidate)
		return candidate

	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		monkeypatch.setattr(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			"from_admitted_local_open", staticmethod(capture_candidate),
		)
		commit = delivery._host.commit_open_replacement

		def commit_then_raise(*args: object) -> None:
			commit(*args)
			raise RuntimeError("wrapper fault after replacement")

		delivery._host = dataclasses.replace(
			delivery._host, commit_open_replacement=commit_then_raise,
		)
		delivery._present_refusal = lambda _request: None
		delivery.stage_prepared(worker, _prepared_cdml(path))
		with pytest.raises(RuntimeError, match="wrapper fault after replacement"):
			main_window._local_document_open_controller._finish_local_document_open_delivery(
				delivery,
			)
		candidate = _active_tab(main_window)
		assert (
			candidate is not old
			and main_window._native_tabs_by_page.get(candidate) is candidate
			and old.is_disposed
			and delivery.replacement_lease_settled
			and delivery.outcome is local_open_contract.LocalDocumentOpenOutcome.COMPLETED
		)
	finally:
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)


#============================================
@pytest.mark.parametrize("replacement", [False, True])
def test_receipt_validation_fault_preserves_the_committed_tab(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch,
		replacement: bool,
		) -> None:
	"""Receipt validation runs after resolution and cannot undo a host commit."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	old = _active_tab(main_window)
	worker = _HeldWorker()
	candidates: list[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab] = []
	factory = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab.from_admitted_local_open

	def capture_candidate(*args: object, **kwargs: object) -> object:
		candidate = factory(*args, **kwargs)
		candidates.append(candidate)
		return candidate

	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		monkeypatch.setattr(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			"from_admitted_local_open", staticmethod(capture_candidate),
		)
		if replacement:
			commit = delivery._host.commit_open_replacement

			def corrupt_commit(*args: object) -> None:
				*commit_args, resolution = args
				class CorruptResolution:
					def accept_replacement(self, receipt: object) -> None:
						resolution.accept_replacement(
							dataclasses.replace(receipt, index=receipt.index + 1),
						)

					def refuse_replacement(self) -> None:
						resolution.refuse_replacement()
				commit(*commit_args, CorruptResolution())

			delivery._host = dataclasses.replace(
				delivery._host, commit_open_replacement=corrupt_commit,
			)
		else:
			publish = delivery._host.publish_open_tab

			def corrupt_publish(candidate: object, resolution: object) -> None:
				class CorruptResolution:
					def accept_publication(self, receipt: object) -> None:
						resolution.accept_publication(
							dataclasses.replace(receipt, index=receipt.index + 1),
						)

					def refuse_publication(self) -> None:
						resolution.refuse_publication()
				publish(candidate, CorruptResolution())

			monkeypatch.setattr(delivery, "_can_replace_pristine", lambda: False)
			delivery._host = dataclasses.replace(
				delivery._host, publish_open_tab=corrupt_publish,
			)
		delivery._present_refusal = lambda _request: None
		delivery.stage_prepared(worker, _prepared_cdml(path))
		with pytest.raises(RuntimeError, match="moved|another tab position"):
			main_window._local_document_open_controller._finish_local_document_open_delivery(
				delivery,
			)
		candidate = candidates[0]
		assert (
			main_window._native_tabs_by_page.get(candidate) is candidate
			and not candidate.is_disposed
			and delivery.outcome is local_open_contract.LocalDocumentOpenOutcome.COMPLETED
		)
		if replacement:
			assert candidate is not old and old.is_disposed and delivery.replacement_lease_settled
		else:
			assert candidate is not old and not old.is_disposed
	finally:
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)


#============================================
def test_double_publication_resolution_is_rejected_without_mutation(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A second resolution faults while preserving the first committed tab."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	worker = _HeldWorker()
	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		publish = delivery._host.publish_open_tab

		def resolve_twice(candidate: object, resolution: object) -> None:
			class DoubleResolution:
				def accept_publication(self, receipt: object) -> None:
					resolution.accept_publication(receipt)
					resolution.accept_publication(receipt)

				def refuse_publication(self) -> None:
					resolution.refuse_publication()
			publish(candidate, DoubleResolution())

		monkeypatch.setattr(delivery, "_can_replace_pristine", lambda: False)
		delivery._host = dataclasses.replace(
			delivery._host, publish_open_tab=resolve_twice,
		)
		delivery._present_refusal = lambda _request: None
		delivery.stage_prepared(worker, _prepared_cdml(path))
		with pytest.raises(RuntimeError, match="host resolution was reused"):
			main_window._local_document_open_controller._finish_local_document_open_delivery(
				delivery,
			)
		candidate = _active_tab(main_window)
		assert (
			main_window._native_tabs_by_page.get(candidate) is candidate
			and not candidate.is_disposed
			and delivery.outcome is local_open_contract.LocalDocumentOpenOutcome.COMPLETED
		)
	finally:
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)


#============================================
@pytest.mark.parametrize("fault_kind", ["return", "raise"])
def test_unresolved_publication_retains_candidate(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch,
		fault_kind: str,
		) -> None:
	"""An unresolved host exit never disposes a possibly published candidate."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	worker = _HeldWorker()
	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)

		def publish_without_resolution(candidate: object, _resolution: object) -> None:
			main_window._register_native_tab(candidate, activate=False)
			if fault_kind == "raise":
				raise RuntimeError("fault after unresolved publication")

		monkeypatch.setattr(delivery, "_can_replace_pristine", lambda: False)
		delivery._host = dataclasses.replace(
			delivery._host, publish_open_tab=publish_without_resolution,
		)
		delivery._present_refusal = lambda _request: None
		delivery.stage_prepared(worker, _prepared_cdml(path))
		error_pattern = (
			"returned without resolution"
			if fault_kind == "return" else "fault after unresolved publication"
		)
		with pytest.raises(RuntimeError, match=error_pattern):
			main_window._local_document_open_controller._finish_local_document_open_delivery(
				delivery,
			)
		candidate = _active_tab(main_window)
		assert main_window._native_tabs_by_page.get(candidate) is candidate and not candidate.is_disposed
	finally:
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)


#============================================
@pytest.mark.parametrize("fault_kind", ["return", "raise"])
def test_unresolved_replacement_retains_committed_candidate(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: ferrum_qt.main_window.MainWindow,
		tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch,
		fault_kind: str,
		) -> None:
	"""An unresolved replacement exit never reclaims an irreversible candidate."""
	path = tmp_path / "opened.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	old = _active_tab(main_window)
	worker = _HeldWorker()
	try:
		delivery = _start_held_open(monkeypatch, main_window, path, worker)
		commit = delivery._host.commit_open_replacement

		def commit_without_resolution(*args: object) -> None:
			*commit_args, resolution = args

			class DroppedResolution:
				"""Consume the host result without resolving the delivery capability."""

				def accept_replacement(self, _receipt: object) -> None:
					"""Drop one real irreversible receipt."""

				def refuse_replacement(self) -> None:
					"""Forward a real pre-commit refusal to its delivery owner."""
					resolution.refuse_replacement()

			commit(*commit_args, DroppedResolution())
			if fault_kind == "raise":
				raise RuntimeError("fault after unresolved replacement")

		delivery._host = dataclasses.replace(
			delivery._host, commit_open_replacement=commit_without_resolution,
		)
		delivery._present_refusal = lambda _request: None
		delivery.stage_prepared(worker, _prepared_cdml(path))
		error_pattern = (
			"returned without resolution"
			if fault_kind == "return" else "fault after unresolved replacement"
		)
		with pytest.raises(RuntimeError, match=error_pattern):
			main_window._local_document_open_controller._finish_local_document_open_delivery(
				delivery,
			)
		candidate = _active_tab(main_window)
		assert (
			candidate is not old
			and main_window._native_tabs_by_page.get(candidate) is candidate
			and not candidate.is_disposed
			and old.is_disposed
			and main_window._operation_leases.active_for_tab(old) == ()
		)
	finally:
		_finish_worker(qapp, worker)
		_flush_deferred_deletes(qapp)

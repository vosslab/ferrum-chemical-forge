#!/usr/bin/env python3
"""Exercise terminal Local Document Open behavior through the staged Qt product."""

# Standard Library
import json
import pathlib
import sys
import types

# local repo modules
import e2e_workspace
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()

# PIP3 modules
import ferrum_chem
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import shiboken6

# local repo modules
import ferrum_qt.config.preferences
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.local_document_open_contract
import ferrum_qt.ferrum.operation_leases
import ferrum_qt.main_window
import ferrum_qt.themes.document_display_palette
import ferrum_qt.themes.theme_manager


_EDITABLE_CDML = """<cdml xmlns='urn:ferrum:cdml' version='1.0'>
  <molecule id='m'><atom id='a' name='C'><point x='10' y='20'/></atom></molecule>
</cdml>
"""
_EMPTY_CDML = '<cdml xmlns="urn:ferrum:cdml" version="1.0"/>'


#============================================
class LocalDocumentOpenLifecycleE2eError(RuntimeError):
	"""Report one failed terminal Local Document Open workflow."""


#============================================
class _HeldWorker(PySide6.QtCore.QThread):
	"""Hold one real controller request at its finished boundary."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self) -> None:
		"""Create one deterministic delivery-only cancellation stand-in."""
		super().__init__()
		self._gate = PySide6.QtCore.QWaitCondition()
		self._lock = PySide6.QtCore.QMutex()
		self._entered = PySide6.QtCore.QSemaphore()
		self._released = False
		self._joined = False
		self.delivery_cancelled = False

	#============================================
	def run(self) -> None:
		"""Wait until this E2E permits the product finished boundary."""
		self._entered.release()
		self._lock.lock()
		while not self._released:
			self._gate.wait(self._lock)
		self._lock.unlock()

	#============================================
	def cancel_delivery(self) -> None:
		"""Match the product worker's delivery-only cancellation contract."""
		self.delivery_cancelled = True

	#============================================
	def wait_until_started(self) -> None:
		"""Synchronize the E2E with the real controller's worker start."""
		self._entered.acquire()

	#============================================
	def finish_safely(self) -> None:
		"""End work and allow the real queued ``finished`` relay to run."""
		if self._joined:
			return
		self._lock.lock()
		self._released = True
		self._gate.wakeAll()
		self._lock.unlock()
		self.wait()
		self._joined = True


#============================================
def _require(condition: bool, message: str) -> None:
	"""Raise one receipt-oriented error when a visible workflow regresses."""
	if not condition:
		raise LocalDocumentOpenLifecycleE2eError(message)


#============================================
def _make_window(app: PySide6.QtWidgets.QApplication) -> ferrum_qt.main_window.MainWindow:
	"""Create the canonical staged Ferrum window."""
	return ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(app),
	)


#============================================
def _current_tab(window: ferrum_qt.main_window.MainWindow) -> object:
	"""Return the visible native document tab."""
	tab = window._tab_widget.currentWidget()
	if type(tab) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		raise LocalDocumentOpenLifecycleE2eError("Ferrum did not select a native document tab")
	return tab


#============================================
def _open_action(window: ferrum_qt.main_window.MainWindow) -> PySide6.QtGui.QAction:
	"""Return the registered public File/Open command by its stable product ID."""
	return window._action_registry.get_qt_action("file.open")


#============================================
def _route_handle(suffix: str) -> object:
	"""Return the Rust-issued route handle accepting one local suffix."""
	return next(
		descriptor.route_handle
		for descriptor in ferrum_chem.DocumentSession.local_document_open_descriptors_v2()
		if suffix in descriptor.suffixes
	)


#============================================
def _prepared_cdml(path: pathlib.Path) -> object:
	"""Issue a one-use Rust admission receipt for a deterministic drawing."""
	return ferrum_chem.DocumentSession.prepare_local_document_open_file_v2(
		str(path), _route_handle(".cdml"),
	)


#============================================
def _wait_for_open(window: ferrum_qt.main_window.MainWindow, path: str,
		start: object) -> bool:
	"""Wait for the public path-specific Local Open completion receipt."""
	outcome: bool | None = None
	completion_loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer(window)
	timeout.setSingleShot(True)

	def receive_completion(opened_path: str, success: bool) -> None:
		"""Accept only the completion receipt for this exact requested file."""
		nonlocal outcome
		if opened_path == path:
			outcome = success
			completion_loop.quit()

	window.local_document_open_completed.connect(receive_completion)
	timeout.timeout.connect(completion_loop.quit)
	try:
		start()
		if outcome is None:
			timeout.start(2500)
			completion_loop.exec()
		timeout.stop()
	finally:
		window.local_document_open_completed.disconnect(receive_completion)
	return outcome is True


#============================================
def _deliver_finished(app: PySide6.QtWidgets.QApplication, worker: _HeldWorker,
		*, process_events: bool = True) -> None:
	"""Flush the product's exact queued worker-finished relay."""
	worker.finish_safely()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.MetaCall,
	)
	if process_events:
		app.processEvents()


#============================================
def _stage_prepared(window: ferrum_qt.main_window.MainWindow, worker: _HeldWorker,
		prepared: object) -> None:
	"""Stage a receipt through the current per-intent delivery boundary."""
	delivery = window._local_document_open_controller._local_document_open_delivery
	if delivery is None:
		raise LocalDocumentOpenLifecycleE2eError("Local Open did not retain an active delivery")
	delivery.stage_prepared(worker, prepared)


#============================================
def _release_worker(app: PySide6.QtWidgets.QApplication, worker: _HeldWorker) -> None:
	"""Release a still-live worker; the product deletes settled workers."""
	if shiboken6.isValid(worker) and worker.isRunning():
		_deliver_finished(app, worker)


#============================================
def _capture_terminal_receipts(window: ferrum_qt.main_window.MainWindow) -> tuple[list[object], object]:
	"""Observe the registry receipt while preserving normal settlement behavior."""
	receipts: list[object] = []
	original_settle = window._operation_leases.settle

	def capture(capability: object, lease: object, terminal: object) -> object:
		receipt = original_settle(capability, lease, terminal)
		receipts.append(receipt)
		return receipt

	window._operation_leases.settle = capture
	return receipts, original_settle


#============================================
def _run_public_open_save_reopen(app: PySide6.QtWidgets.QApplication,
		directory: pathlib.Path) -> None:
	"""Drive File/Open, Save, Rust reopen, authoring, and discard through the UI."""
	source = directory / "ordinary-open.cdml"
	destination = directory / "ordinary-open-copy.cdml"
	source.write_text(_EDITABLE_CDML, encoding="utf-8")
	prefs = ferrum_qt.config.preferences.Preferences.instance()
	previous_recent_paths = prefs.value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES)
	window = _make_window(app)
	initial_tab = _current_tab(window)
	original_open = PySide6.QtWidgets.QFileDialog.getOpenFileName
	PySide6.QtWidgets.QFileDialog.getOpenFileName = staticmethod(
		lambda *_args, **_kwargs: (str(source), "Ferrum CDML (*.cdml)"),
	)
	try:
		window.show()
		app.processEvents()
		_require(_wait_for_open(window, str(source), _open_action(window).trigger),
			"File/Open did not publish a successful CDML completion receipt")
		tab = _current_tab(window)
		_require(
			tab.file_path == source and tab.local_document_origin_token is not None
			and not tab.current_snapshot.is_dirty and initial_tab.is_disposed,
			"File/Open did not atomically replace the pristine bootstrap document",
		)
		_require(not window._operation_leases.active_for_tab(tab),
			"a completed File/Open left an active document lease")
		_require(tab.view.backgroundBrush().color() == window._document_theme_change.palette.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.CANVAS_SURROUND,
		), "opened drawing did not receive the active document palette")
		_require(window.save_active_to_path(str(destination)), "Save did not publish the opened CDML")
		reopened, observation, _origin, _source_kind, _summary = _prepared_cdml(destination).take_admission_v2()
		_require(observation.document.snapshot.digest == reopened.snapshot().digest,
			"Rust could not reopen the just-saved authoritative CDML")
		window.show()
		app.processEvents()
		atom = window._action_registry.get_qt_action("draw.atom_at_point")
		_require(window._window_mode_sync.select_action(atom), "Atom tool did not become active")
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			tab.view.mapFromScene(PySide6.QtCore.QPointF(40.0, 50.0)),
		)
		app.processEvents()
		_require(tab.is_dirty, "authoring after Open did not mark the drawing dirty")
		result = window._close_native_tab_at(
			window._tab_widget.indexOf(tab), ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		)
		_require(result is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED and tab.is_disposed,
			"Discard did not close the author-edited opened document")
	finally:
		PySide6.QtWidgets.QFileDialog.getOpenFileName = original_open
		prefs.set_value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES, previous_recent_paths)
		ferrum_qt_e2e.close_e2e_main_window(window, app)


#============================================
def _run_nested_dirty_cancellation(app: PySide6.QtWidgets.QApplication,
		directory: pathlib.Path) -> None:
	"""Cancel a dirty current-tab replacement from its real nested confirmation loop."""
	path = directory / "dirty-current-open.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(app)
	source = _current_tab(window)
	source._pending_snapshot = types.SimpleNamespace(is_dirty=True)
	source_digest = source.current_snapshot.digest
	controller = window._local_document_open_controller
	worker = _HeldWorker()
	receipts, original_settle = _capture_terminal_receipts(window)
	original_factory = controller._create_local_document_open_worker
	original_exec = PySide6.QtWidgets.QMessageBox.exec
	original_present = controller._present_refusal
	presented: list[object] = []
	states: list[object] = []
	controller._create_local_document_open_worker = lambda _path, _route: worker
	controller._present_refusal = presented.append

	def cancel_during_confirmation(box: PySide6.QtWidgets.QMessageBox) -> int:
		"""Request product cancellation while the real Qt confirmation is nested."""
		def cancel() -> None:
			controller._cancel_local_document_open()
			states.append(next(iter(window._operation_leases.active_for_tab(source))).state)

		PySide6.QtCore.QTimer.singleShot(0, cancel)
		PySide6.QtCore.QTimer.singleShot(0, box.reject)
		return original_exec(box)

	PySide6.QtWidgets.QMessageBox.exec = cancel_during_confirmation
	try:
		_require(window.open_in_current_tab_path(str(path)),
			"Open in Current Tab did not start its dirty-document request")
		worker.wait_until_started()
		lease = next(iter(window._operation_leases.active_for_tab(source)))
		_stage_prepared(window, worker, _prepared_cdml(path))
		_deliver_finished(app, worker)
		_require(
			states == [ferrum_qt.ferrum.operation_leases.LeaseState.CANCELLATION_REQUESTED]
			and _current_tab(window) is source and source in window._native_tabs_by_page
			and source.current_snapshot.digest == source_digest
			and receipts[-1].lease_id == lease.lease_id
			and receipts[-1].state is ferrum_qt.ferrum.operation_leases.LeaseState.CANCELLED
			and not presented,
			"nested dirty-dialog cancellation published or replaced the source document",
		)
	finally:
		PySide6.QtWidgets.QMessageBox.exec = original_exec
		controller._present_refusal = original_present
		controller._create_local_document_open_worker = original_factory
		window._operation_leases.settle = original_settle
		source._pending_snapshot = None
		_release_worker(app, worker)
		ferrum_qt_e2e.close_e2e_main_window(window, app)


#============================================
def _run_postcommit_refresh_recovery(app: PySide6.QtWidgets.QApplication,
		directory: pathlib.Path) -> None:
	"""Prove a postcommit refresh fault leaves the committed replacement current."""
	path = directory / "postcommit-refresh.cdml"
	path.write_text(_EMPTY_CDML, encoding="utf-8")
	window = _make_window(app)
	source = _current_tab(window)
	controller = window._local_document_open_controller
	worker = _HeldWorker()
	receipts, original_settle = _capture_terminal_receipts(window)
	original_complete = window._operation_leases.complete_prepared_terminal_replacement
	original_factory = controller._create_local_document_open_worker
	original_refresh = window._on_native_tab_changed
	original_present = controller._present_refusal
	completed: list[tuple[str, bool]] = []
	presented: list[object] = []
	controller._create_local_document_open_worker = lambda _path, _route: worker
	controller._present_refusal = presented.append
	window.local_document_open_completed.connect(lambda opened_path, success: completed.append((opened_path, success)))

	def capture_completion(prepared: object, observer: object) -> None:
		"""Observe the replacement terminal receipt at its final observer boundary."""
		def capture_observer(receipt: object) -> None:
			receipts.append(receipt)
			observer(receipt)

		original_complete(prepared, capture_observer)

	window._operation_leases.complete_prepared_terminal_replacement = capture_completion

	def fail_postcommit_refresh(_index: int) -> None:
		"""Inject a postcommit-only display refresh failure."""
		raise ferrum_qt.ferrum.local_document_open_contract.LocalOpenPostCommitPresentationError(
			"postcommit refresh failure",
		)

	window._on_native_tab_changed = fail_postcommit_refresh
	try:
		_require(window.open_file_path(str(path), interactive=True), "File/Open did not start")
		worker.wait_until_started()
		lease = next(iter(window._operation_leases.active_for_tab(source)))
		_stage_prepared(window, worker, _prepared_cdml(path))
		_deliver_finished(app, worker, process_events=False)
		candidate = _current_tab(window)
		receipt = receipts[-1]
		_require(
			candidate is not source and candidate in window._native_tabs_by_page and source.is_disposed
			and receipt.lease_id == lease.lease_id and receipt.tab_identity == lease.tab_identity
			and receipt.tab() is source
			and receipt.state is ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED
			and completed == [(str(path), True)],
			"a postcommit refresh fault rewrote the irreversible Local Open completion",
		)
		_require(
			bool(presented) and all(
				request.outcome is ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.DOCUMENT_DISPLAY_FAILED
				and request.technical_details is not None
				and "drawing was opened and is current" in request.technical_details
				and "left unchanged" not in request.technical_details for request in presented
			),
			"postcommit recovery did not provide truthful bounded author guidance",
		)
		_require(source.parent() is None, "retired source remained attached after terminal replacement")
		app.sendPostedEvents(None, PySide6.QtCore.QEvent.Type.DeferredDelete)
		app.processEvents()
		_require(not shiboken6.isValid(source) and shiboken6.isValid(candidate)
			and _current_tab(window) is candidate
			and receipts[-1].state is ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED,
			"postcommit cleanup did not retain the completed replacement")
	finally:
		window._on_native_tab_changed = original_refresh
		controller._present_refusal = original_present
		controller._create_local_document_open_worker = original_factory
		window._operation_leases.settle = original_settle
		window._operation_leases.complete_prepared_terminal_replacement = original_complete
		_release_worker(app, worker)
		ferrum_qt_e2e.close_e2e_main_window(window, app)


#============================================
def main() -> int:
	"""Run the receipt-bounded Local Open lifecycle workflows against Ferrum."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	with e2e_workspace.E2EWorkspaceLease() as temporary:
		directory = pathlib.Path(temporary)
		_run_public_open_save_reopen(app, directory)
		_run_nested_dirty_cancellation(app, directory)
		_run_postcommit_refresh_recovery(app, directory)
	print(json.dumps({"schema": "ferrum-local-document-open-lifecycle-e2e-v1", "status": "ok"}))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except LocalDocumentOpenLifecycleE2eError as exc:
		print(f"e2e_local_document_open_lifecycle: {exc}", file=sys.stderr)
		raise SystemExit(1)

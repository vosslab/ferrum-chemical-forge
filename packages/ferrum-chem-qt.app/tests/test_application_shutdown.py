"""Focused application event-loop shutdown coverage."""

# Standard Library
import json
import pathlib
import threading

# PIP3 modules
import pytest
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.app
import ferrum_qt.main_window
import ferrum_qt.legacy.compatibility_lifecycle
import ferrum_qt.legacy.compatibility_main_window
import ferrum_qt.qt_lifecycle
import ferrum_qt.themes.theme_manager
import ferrum_qt.bridge.worker
import ferrum_qt.window_shared


#============================================
class _ControlledModal:
	"""Record controlled closure of one nested modal widget."""

	def __init__(self) -> None:
		"""Start with an open controlled modal."""
		self.closed = False

	def close(self) -> None:
		"""Record the lifecycle callback closing the modal."""
		self.closed = True


#============================================
class _ControlledTimerApplication:
	"""Minimal event-loop seam for one deterministic controlled-smoke callback."""

	def __init__(self, delivery: ferrum_qt.app._SmokeTimerDelivery) -> None:
		"""Retain the delivery state that quit must observe."""
		self._delivery = delivery
		self.modal = _ControlledModal()
		self.delivery_before_quit = False
		self.quit_requested = False

	def activeModalWidget(self) -> _ControlledModal:
		"""Return the nested modal owned by this controlled lifecycle."""
		return self.modal

	def quit(self) -> None:
		"""Observe delivery state at the exact controlled event-loop exit request."""
		self.delivery_before_quit = self._delivery.timer_fired
		self.quit_requested = True

	def exec(self) -> int:
		"""Return the normal event-loop result only after controlled quit."""
		return 0 if self.quit_requested else 1


#============================================
def test_programmatic_event_loop_exit_retires_live_session_through_window_boundary(
		qapp: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		) -> None:
	"""A direct application quit still drains the window-owned session graph."""
	window = ferrum_qt.legacy.compatibility_main_window.LegacyCompatibilityMainWindow(theme_manager)
	session = window.sessions[0]
	PySide6.QtCore.QTimer.singleShot(0, qapp.quit)

	assert qapp.exec() == 0
	assert ferrum_qt.app._finalize_event_loop_exit(qapp, window, 0) == 0
	assert session.is_disposed


#============================================
def test_shutdown_drains_retired_worker_through_a_nested_qt_event_loop(
		qapp: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		qtbot: object,
		) -> None:
	"""Shutdown retains native work until its queued finished signal releases it."""
	window = ferrum_qt.legacy.compatibility_main_window.LegacyCompatibilityMainWindow(theme_manager)
	session = window.sessions[0]
	started = threading.Event()
	release = threading.Event()

	def controlled_call() -> str:
		"""Hold native completion until shutdown has entered draining."""
		started.set()
		release.wait()
		return "prepared"

	worker = ferrum_qt.bridge.worker.OasaWorker(controlled_call)
	session.track_import_worker(worker)
	worker.finished.connect(lambda: window._release_import_worker(worker))
	worker.start()
	try:
		qtbot.waitUntil(started.is_set)
		assert window.prepare_application_shutdown()
		assert window.shutdown_state is ferrum_qt.window_shared.ShutdownState.DRAINING
		assert window.retiring_worker_count == 1 and not window.sessions
		PySide6.QtCore.QTimer.singleShot(0, release.set)
		assert ferrum_qt.legacy.compatibility_lifecycle.drain_pending_session_deletions(qapp, window)
		assert window.shutdown_state is ferrum_qt.window_shared.ShutdownState.READY
		assert session.is_disposed
	finally:
		release.set()
		try:
			if worker.isRunning():
				worker.wait()
		except RuntimeError:
			pass
		ferrum_qt.qt_lifecycle.delete_qobject_and_wait(qapp, window)


#============================================
def test_successful_smoke_receipt_is_published_atomically_after_timer_success(
		tmp_path: pathlib.Path,
		) -> None:
	"""A zero controlled-shutdown result publishes only the fixed receipt schema."""
	receipt_path = tmp_path / "completion.json"

	ferrum_qt.app._write_smoke_receipt(receipt_path, 0, timer_fired=True)

	receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
	assert receipt["exit_code"] == 0
	assert receipt["schema"] == ferrum_qt.app.SMOKE_RECEIPT_SCHEMA


#============================================
def test_failed_smoke_shutdown_does_not_publish_a_receipt(tmp_path: pathlib.Path) -> None:
	"""A failed retirement result cannot be mistaken for completed Qt lifecycle."""
	receipt_path = tmp_path / "completion.json"

	with pytest.raises(RuntimeError, match="successful controlled shutdown"):
		ferrum_qt.app._write_smoke_receipt(receipt_path, 1, timer_fired=True)

	assert not receipt_path.exists()


#============================================
def test_smoke_receipt_requires_a_delivered_timer(tmp_path: pathlib.Path) -> None:
	"""Receipt publication refuses a clean result without timer delivery evidence."""
	receipt_path = tmp_path / "completion.json"

	with pytest.raises(RuntimeError, match="controlled smoke timer"):
		ferrum_qt.app._write_smoke_receipt(receipt_path, 0, timer_fired=False)

	assert not receipt_path.exists()


#============================================
@pytest.mark.parametrize("timer_seconds", (None, 0.0, float("nan")))
def test_app_rejects_a_receipt_configuration_without_a_valid_timer(
		timer_seconds: float | None, tmp_path: pathlib.Path,
		) -> None:
	"""The app entry boundary requires a finite positive timer for a smoke receipt."""

	with pytest.raises(ValueError, match="finite positive"):
		ferrum_qt.app._validate_smoke_configuration(timer_seconds, tmp_path / "completion.json")


#============================================
def test_early_clean_exit_does_not_publish_a_smoke_receipt(
		monkeypatch: pytest.MonkeyPatch, qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""An ordinary early close is clean but cannot certify timer-driven smoke."""
	receipt_path = tmp_path / "completion.json"
	monkeypatch.setattr(ferrum_qt.app, "_finalize_event_loop_exit", lambda *_args: 0)

	completed = ferrum_qt.app._complete_event_loop_exit(
		qapp, None, 0, receipt_path, timer_fired=False,
	)

	assert completed == 0
	assert not receipt_path.exists()


#============================================
def test_timer_fired_clean_exit_publishes_a_smoke_receipt(
		monkeypatch: pytest.MonkeyPatch, qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""A delivered timer plus successful finalization publishes the fixed receipt."""
	receipt_path = tmp_path / "completion.json"
	monkeypatch.setattr(ferrum_qt.app, "_finalize_event_loop_exit", lambda *_args: 0)

	completed = ferrum_qt.app._complete_event_loop_exit(
		qapp, None, 0, receipt_path, timer_fired=True,
	)

	assert completed == 0
	receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
	assert receipt["exit_code"] == 0
	assert receipt["schema"] == ferrum_qt.app.SMOKE_RECEIPT_SCHEMA


#============================================
def test_scheduled_smoke_callback_records_delivery_before_quit_and_publishes_receipt(
		monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""The captured Qt timer callback proves delivery before controlled quit and receipt."""
	receipt_path = tmp_path / "completion.json"
	delivery = ferrum_qt.app._SmokeTimerDelivery()
	application = _ControlledTimerApplication(delivery)
	callbacks: list = []

	def capture_timer_callback(_milliseconds: int, callback: object) -> None:
		"""Capture the production callback without starting a Qt event loop."""
		callbacks.append(callback)

	monkeypatch.setattr(PySide6.QtCore.QTimer, "singleShot", capture_timer_callback)
	monkeypatch.setattr(ferrum_qt.app, "_finalize_event_loop_exit", lambda *_args: 0)
	ferrum_qt.app._schedule_controlled_smoke_exit(application, 2000, delivery)
	callbacks[0]()
	assert application.modal.closed and not application.quit_requested
	callbacks[1]()
	completed = ferrum_qt.app._complete_event_loop_exit(
		application, None, application.exec(), receipt_path, delivery.timer_fired,
	)

	assert application.delivery_before_quit and completed == 0
	receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
	assert receipt["exit_code"] == 0
	assert receipt["schema"] == ferrum_qt.app.SMOKE_RECEIPT_SCHEMA


#============================================
def test_launch_file_delivery_requires_every_requested_file_to_open(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Controlled startup is incomplete until all requested files report success."""
	delivery = ferrum_qt.app._SmokeTimerDelivery()
	application = _ControlledTimerApplication(delivery)
	callbacks = []

	class ControlledWindow:
		"""Open only one of two requested paths."""

		def open_file_path(self, path: str) -> bool:
			"""Reject the second controlled path."""
			return path == "opens.cdml"

	monkeypatch.setattr(
		PySide6.QtCore.QTimer, "singleShot",
		lambda _milliseconds, callback: callbacks.append(callback),
	)
	ferrum_qt.app._schedule_launch_files(
		application, ControlledWindow(), ["opens.cdml", "fails.cdml"], delivery,
	)
	assert not delivery.launch_files_completed
	callbacks[0]()
	assert not delivery.launch_files_completed and not delivery.launch_files_pending


#============================================
def test_controlled_timer_waits_for_startup_delivery_before_requesting_quit(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A timer closes modal warnings but cannot retire an active launch callback."""
	delivery = ferrum_qt.app._SmokeTimerDelivery(launch_files_pending=True)
	application = _ControlledTimerApplication(delivery)
	callbacks: list = []

	monkeypatch.setattr(
		PySide6.QtCore.QTimer, "singleShot",
		lambda _milliseconds, callback: callbacks.append(callback),
	)
	delivery.timer_fired = True
	ferrum_qt.app._advance_controlled_smoke_exit(application, delivery)

	assert application.modal.closed and not application.quit_requested
	assert len(callbacks) == 1
	delivery.launch_files_pending = False
	callbacks[0]()
	assert len(callbacks) == 2 and not application.quit_requested
	callbacks[1]()
	assert application.quit_requested


#============================================
def test_incomplete_launch_file_delivery_makes_controlled_smoke_fail(
		monkeypatch: pytest.MonkeyPatch, qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A timer cannot certify startup when a requested document did not open."""
	monkeypatch.setattr(ferrum_qt.app, "_finalize_event_loop_exit", lambda *_args: 0)

	completed = ferrum_qt.app._complete_event_loop_exit(
		qapp, None, 0, None, timer_fired=True, launch_files_completed=False,
	)

	assert completed == 1

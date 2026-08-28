"""Shared launch boundary for Ferrum's direct Qt E2E scripts."""

# Standard Library
import os
import sys
import types


#============================================
class FerrumQtE2ECallbackFailure(RuntimeError):
	"""Raise after Qt reports an exception from an E2E callback."""


#============================================
class FerrumQtE2ECleanupError(RuntimeError):
	"""Raise when E2E-owned Qt window teardown violates its lifecycle contract."""


#============================================
class _QtCallbackFailureState:
	"""Keep the process-local callback failure and nested-loop ownership."""

	def __init__(self) -> None:
		self.failure: tuple[type[BaseException], BaseException, types.TracebackType | None] | None = None
		self.active_event_loops: list[object] = []
		self.execution_depth = 0
		self.process_events_depth = 0
		self.failure_raised = False


_CALLBACK_FAILURE_STATE: _QtCallbackFailureState | None = None
_CALLBACK_FAILURE_INSTALLED = False
_ORIGINAL_EXCEPTHOOK = sys.excepthook


#============================================
def _record_callback_failure(
	exc_type: type[BaseException],
	exc_value: BaseException,
	traceback: types.TracebackType | None,
) -> None:
	"""Record the first Qt callback exception and return all active loops."""
	state = _CALLBACK_FAILURE_STATE
	if state is None or state.failure is not None:
		return
	state.failure = (exc_type, exc_value, traceback)
	for event_loop in reversed(state.active_event_loops):
		event_loop.quit()
	from PySide6.QtCore import QCoreApplication
	if QCoreApplication.instance() is not None:
		QCoreApplication.quit()


#============================================
def _callback_excepthook(
	exc_type: type[BaseException],
	exc_value: BaseException,
	traceback: types.TracebackType | None,
) -> None:
	"""Preserve Qt's original traceback while requesting deterministic exit."""
	_record_callback_failure(exc_type, exc_value, traceback)
	_ORIGINAL_EXCEPTHOOK(exc_type, exc_value, traceback)


#============================================
def _raise_callback_failure_if_ready() -> None:
	"""Promote the stored callback exception after its outer execution boundary."""
	state = _CALLBACK_FAILURE_STATE
	if (
		state is None
		or state.failure is None
		or state.failure_raised
		or state.execution_depth != 0
		or state.process_events_depth != 0
	):
		return
	state.failure_raised = True
	exc_type, exc_value, _traceback = state.failure
	raise FerrumQtE2ECallbackFailure(
		f"Qt callback failed: {exc_type.__name__}: {exc_value}"
	) from None


#============================================
def _install_callback_failure_gate() -> None:
	"""Install one idempotent callback-failure gate for this Python process."""
	global _CALLBACK_FAILURE_INSTALLED
	global _CALLBACK_FAILURE_STATE
	if _CALLBACK_FAILURE_INSTALLED:
		return
	from PySide6.QtCore import QCoreApplication, QEventLoop
	_CALLBACK_FAILURE_STATE = _QtCallbackFailureState()
	original_event_loop_exec = QEventLoop.exec
	original_event_loop_exec_legacy = getattr(QEventLoop, "exec_", None)
	original_process_events = QCoreApplication.processEvents
	original_application_exec = QCoreApplication.exec
	original_application_exec_legacy = getattr(QCoreApplication, "exec_", None)

	def run_event_loop(event_loop: object, *args, **kwargs) -> object:
		"""Run one tracked nested Qt event loop."""
		state = _CALLBACK_FAILURE_STATE
		if state is None:
			return original_event_loop_exec(event_loop, *args, **kwargs)
		state.active_event_loops.append(event_loop)
		state.execution_depth += 1
		try:
			return original_event_loop_exec(event_loop, *args, **kwargs)
		finally:
			state.execution_depth -= 1
			state.active_event_loops.remove(event_loop)
			_raise_callback_failure_if_ready()

	def run_event_loop_legacy(event_loop: object, *args, **kwargs) -> object:
		"""Run one tracked legacy-named Qt event loop."""
		if original_event_loop_exec_legacy is None:
			return run_event_loop(event_loop, *args, **kwargs)
		state = _CALLBACK_FAILURE_STATE
		if state is None:
			return original_event_loop_exec_legacy(event_loop, *args, **kwargs)
		state.active_event_loops.append(event_loop)
		state.execution_depth += 1
		try:
			return original_event_loop_exec_legacy(event_loop, *args, **kwargs)
		finally:
			state.execution_depth -= 1
			state.active_event_loops.remove(event_loop)
			_raise_callback_failure_if_ready()

	def process_events(*args, **kwargs) -> object:
		"""Run one tracked direct event-dispatch cycle."""
		state = _CALLBACK_FAILURE_STATE
		if state is None:
			return original_process_events(*args, **kwargs)
		state.process_events_depth += 1
		try:
			return original_process_events(*args, **kwargs)
		finally:
			state.process_events_depth -= 1
			_raise_callback_failure_if_ready()

	def run_application(*args, **kwargs) -> object:
		"""Run the tracked application event loop."""
		state = _CALLBACK_FAILURE_STATE
		if state is None:
			return original_application_exec(*args, **kwargs)
		state.execution_depth += 1
		try:
			return original_application_exec(*args, **kwargs)
		finally:
			state.execution_depth -= 1
			_raise_callback_failure_if_ready()

	def run_application_legacy(*args, **kwargs) -> object:
		"""Run the tracked legacy-named application event loop."""
		if original_application_exec_legacy is None:
			return run_application(*args, **kwargs)
		state = _CALLBACK_FAILURE_STATE
		if state is None:
			return original_application_exec_legacy(*args, **kwargs)
		state.execution_depth += 1
		try:
			return original_application_exec_legacy(*args, **kwargs)
		finally:
			state.execution_depth -= 1
			_raise_callback_failure_if_ready()

	sys.excepthook = _callback_excepthook
	QEventLoop.exec = run_event_loop
	if original_event_loop_exec_legacy is not None:
		QEventLoop.exec_ = run_event_loop_legacy
	QCoreApplication.processEvents = staticmethod(process_events)
	QCoreApplication.exec = staticmethod(run_application)
	if original_application_exec_legacy is not None:
		QCoreApplication.exec_ = staticmethod(run_application_legacy)
	_CALLBACK_FAILURE_INSTALLED = True


#============================================
def select_offscreen_qt_platform() -> None:
	"""Select the test-owned Qt backend before any PySide6 import."""
	os.environ["QT_QPA_PLATFORM"] = "offscreen"
	_install_callback_failure_gate()


#============================================
def close_e2e_main_window(window: object, app: object) -> None:
	"""Discard E2E-owned native tabs before ordinary Qt window teardown."""
	from PySide6.QtCore import QCoreApplication, QEvent
	from ferrum_qt.ferrum.close_decision import CloseDecision, CloseResult

	while window._tab_widget.count():
		before = window._tab_widget.count()
		result = window._close_native_tab_at(0, CloseDecision.DISCARD)
		if result is not CloseResult.CLOSED:
			raise FerrumQtE2ECleanupError(
				"E2E teardown could not discard its native tab: " + result.value,
			)
		if window._tab_widget.count() >= before:
			raise FerrumQtE2ECleanupError(
				"E2E teardown reported a closed native tab without tab-host progress",
			)
	if window._native_tabs_by_page:
		raise FerrumQtE2ECleanupError(
			"E2E teardown left registered native tabs after explicit discard",
		)
	window.close()
	window.deleteLater()
	QCoreApplication.sendPostedEvents(None, QEvent.Type.DeferredDelete)
	app.processEvents()

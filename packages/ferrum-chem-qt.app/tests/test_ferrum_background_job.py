"""Characterization tests for detached Ferrum worker mechanics."""

# Standard Library
import os
import threading


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
import pytest

# local repo modules
from ferrum_qt.ferrum.background_job import FerrumDetachedJobThread
from ferrum_qt.ferrum.background_job import FerrumWorkerFailure


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide the offscreen event loop used for queued worker delivery."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def test_detached_job_delivers_one_success(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""One completed operation emits exactly its detached success value."""
	job = FerrumDetachedJobThread(lambda: "receipt")
	successes = []
	failures = []
	job.succeeded.connect(successes.append)
	job.failed.connect(failures.append)
	job.start()
	assert job.wait(10000)
	qapp.processEvents()

	assert successes == ["receipt"]
	assert failures == []
	job.deleteLater()


#============================================
def test_detached_job_cancellation_before_start_withholds_delivery() -> None:
	"""Cancellation before work begins suppresses both terminal channels."""
	job = FerrumDetachedJobThread(lambda: "receipt")
	successes = []
	job.succeeded.connect(successes.append)
	job.cancel_delivery()
	job.run()

	assert job.delivery_cancelled
	assert successes == []
	job.deleteLater()


#============================================
def test_detached_job_cancellation_during_operation_withholds_delivery() -> None:
	"""Cancellation wins while a native-like operation is still completing."""
	started = threading.Event()
	resume = threading.Event()

	def operation() -> str:
		started.set()
		assert resume.wait(10)
		return "receipt"

	job = FerrumDetachedJobThread(operation)
	successes = []
	job.succeeded.connect(successes.append)
	job.start()
	assert started.wait(10)
	job.cancel_delivery()
	resume.set()
	assert job.wait(10000)

	assert job.delivery_cancelled and successes == []
	job.deleteLater()


#============================================
def test_detached_job_mapper_failure_becomes_generic_failure() -> None:
	"""A broken mapper cannot leak an exception across the Qt boundary."""
	def operation() -> object:
		raise RuntimeError("operation failed")

	def broken_mapper(_error: Exception) -> object:
		raise ValueError("mapper failed")

	job = FerrumDetachedJobThread(operation, broken_mapper)
	failures = []
	job.failed.connect(failures.append)
	job.run()

	assert failures == [FerrumWorkerFailure("ValueError", "mapper failed")]
	job.deleteLater()

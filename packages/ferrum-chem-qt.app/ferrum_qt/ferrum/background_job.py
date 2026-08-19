"""Narrow, handle-free background job mechanics for Ferrum Qt features.

This module deliberately does not know about documents, tabs, relays, or
publication.  Feature owners keep their revision, digest, selection, and
lifecycle fences on the Qt thread.  Cancellation suppresses terminal delivery
and requests interruption; it does not promise to preempt a Rust call already
running in the worker thread.
"""

# Standard Library
import collections.abc
import dataclasses

# PIP3 modules
import PySide6.QtCore


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumWorkerFailure:
	"""Stable, exception-free facts delivered from a detached job."""

	error_type: str
	message: str
	category: str | None = None


_FailureMapper = collections.abc.Callable[[Exception], object]
_Operation = collections.abc.Callable[[], object]


#============================================
class FerrumDetachedJobThread(PySide6.QtCore.QThread):
	"""Run one immutable operation and deliver at most one terminal outcome.

	Subclasses may override :meth:`_emit_success` or :meth:`_emit_failure` only
	to retain a legacy feature signal during the atomic worker migration.  They
	must not add document mutation or Qt-widget work here.
	"""

	succeeded = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, operation: _Operation,
			failure_mapper: _FailureMapper | None = None,
			parent: PySide6.QtCore.QObject | None = None) -> None:
		"""Capture an immutable, zero-argument operation before the thread starts."""
		if not callable(operation):
			raise TypeError("Ferrum detached job operation must be callable")
		if failure_mapper is not None and not callable(failure_mapper):
			raise TypeError("Ferrum detached job failure mapper must be callable")
		super().__init__(parent)
		self._operation = operation
		self._failure_mapper = failure_mapper
		self._delivery_cancelled = False

	#============================================
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether terminal delivery has been invalidated."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Withhold terminal delivery and request cooperative interruption."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def _can_deliver(self) -> bool:
		"""Return whether this job still owns its terminal delivery."""
		return not self._delivery_cancelled and not self.isInterruptionRequested()

	#============================================
	def _map_failure(self, error: Exception) -> object:
		"""Convert a worker exception to immutable, Qt-safe failure facts."""
		if self._failure_mapper is None:
			return FerrumWorkerFailure(type(error).__name__, str(error))
		try:
			return self._failure_mapper(error)
		except Exception as mapper_error:
			return FerrumWorkerFailure(type(mapper_error).__name__, str(mapper_error))

	#============================================
	def _emit_success(self, result: object) -> None:
		"""Emit the shared success protocol for the owner relay."""
		self.succeeded.emit(result)

	#============================================
	def _emit_failure(self, failure: object) -> None:
		"""Emit the shared failure protocol for the owner relay."""
		self.failed.emit(failure)

	#============================================
	def run(self) -> None:
		"""Invoke once; never deliver after cancellation or interruption."""
		try:
			result = self._operation()
		except Exception as error:
			if self._can_deliver():
				self._emit_failure(self._map_failure(error))
			return
		if self._can_deliver():
			self._emit_success(result)

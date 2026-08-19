"""Bounded Ferrum preparation of one local V2000 or V3000 molfile."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
from ferrum_qt.ferrum.background_job import FerrumDetachedJobThread
import ferrum_qt.ferrum.engine as engine


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeMolblockPreparationFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
class FerrumNativeMolblockPreparationWorker(FerrumDetachedJobThread):
	"""Read and prepare one bounded molfile outside the Qt thread."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, path: str, placement: object) -> None:
		"""Capture one exact local path and immutable Ferrum placement."""
		if type(path) is not str or not path:
			raise ValueError("Ferrum molfile preparation requires a nonempty path")
		if type(placement) is not engine.InsertionPlacementV1:
			raise TypeError("Ferrum molfile preparation requires exact Ferrum placement")
		self._path = path
		self._placement = placement
		self._prepare_operation = engine.prepare_molblock_file_v1
		super().__init__(
			lambda: self._prepare_operation(self._path, self._placement),
			lambda error: FerrumNativeMolblockPreparationFailure(
				type(error).__name__, str(error),
			),
		)

	#============================================
	@classmethod
	def _from_fixture(cls, path: str, placement: object,
			prepare_operation: object) -> "FerrumNativeMolblockPreparationWorker":
		"""Construct a worker with one private deterministic test operation."""
		worker = cls(path, placement)
		if not callable(prepare_operation):
			raise TypeError("fixture preparation operation must be callable")
		worker._prepare_operation = prepare_operation
		return worker

	#============================================
	def _emit_success(self, prepared: object) -> None:
		"""Retain the import route's established prepared signal."""
		self.prepared.emit(prepared)

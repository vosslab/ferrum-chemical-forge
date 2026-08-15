"""Bounded Rust-native preparation of one local V2000 or V3000 molfile."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import ferrum_chem


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeMolblockPreparationFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
class FerrumNativeMolblockPreparationWorker(PySide6.QtCore.QThread):
	"""Read and prepare one bounded molfile outside the Qt thread."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, path: str, placement: object) -> None:
		"""Capture one exact local path and immutable Ferrum placement."""
		if type(path) is not str or not path:
			raise ValueError("native molfile preparation requires a nonempty path")
		if type(placement) is not ferrum_chem.InsertionPlacementV1:
			raise TypeError("native molfile preparation requires exact Ferrum placement")
		super().__init__()
		self._path = path
		self._placement = placement
		self._prepare_operation = ferrum_chem.prepare_molblock_file_v1
		self._delivery_cancelled = False

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
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether future delivery has been invalidated."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Invalidate delivery without pretending to preempt native parsing."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def run(self) -> None:
		"""Read and prepare the molfile, emitting at most one current outcome."""
		try:
			prepared = self._prepare_operation(self._path, self._placement)
		except Exception as exc:
			if not self._delivery_cancelled and not self.isInterruptionRequested():
				self.failed.emit(
					FerrumNativeMolblockPreparationFailure(type(exc).__name__, str(exc)),
				)
			return
		if not self._delivery_cancelled and not self.isInterruptionRequested():
			self.prepared.emit(prepared)

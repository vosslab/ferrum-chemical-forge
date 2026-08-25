"""Ferrum background preparation for one native peptide insertion."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
from ferrum_qt.ferrum.background_job import FerrumDetachedJobThread
import ferrum_qt.ferrum.engine as engine


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativePeptidePreparationFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
class FerrumNativePeptidePreparationWorker(FerrumDetachedJobThread):
	"""Prepare one strict native peptide insertion off the Qt thread.

	The exact dialog text and captured placement cross this boundary unchanged.
	Rust owns strict sequence admission, closed-profile plan construction, Ferrum chemistry,
	and the returned handle-free insertion. Cancellation invalidates delivery; it
	does not claim to preempt a Ferrum library call already in progress.
	"""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, sequence: str, placement: object) -> None:
		"""Capture immutable native-peptide inputs before this thread starts."""
		if type(sequence) is not str:
			raise TypeError("Ferrum peptide preparation requires exact text")
		if type(placement) is not engine.InsertionPlacementV1:
			raise TypeError("Ferrum peptide preparation requires exact Ferrum placement")
		self._sequence = sequence
		self._placement = placement
		self._prepare_operation = engine.prepare_ferrum_peptide_insertion_v1
		super().__init__(
			lambda: self._prepare_operation(self._sequence, self._placement),
			lambda error: FerrumNativePeptidePreparationFailure(
				type(error).__name__, str(error),
			),
		)

	#============================================
	@classmethod
	def _from_fixture(cls, sequence: str, placement: object,
			prepare_operation: object) -> "FerrumNativePeptidePreparationWorker":
		"""Construct a worker with one private deterministic test operation."""
		worker = cls(sequence, placement)
		if not callable(prepare_operation):
			raise TypeError("fixture preparation operation must be callable")
		worker._prepare_operation = prepare_operation
		return worker

	#============================================
	def _emit_success(self, prepared: object) -> None:
		"""Retain the import route's established prepared signal."""
		self.prepared.emit(prepared)

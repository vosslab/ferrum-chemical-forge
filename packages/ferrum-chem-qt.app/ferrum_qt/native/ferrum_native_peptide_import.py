"""Rust-native background preparation for one supported peptide-template insertion."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import ferrum_chem


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativePeptidePreparationFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
class FerrumNativePeptidePreparationWorker(PySide6.QtCore.QThread):
	"""Prepare one strict supported peptide template off the Qt thread.

	The exact dialog text and captured placement cross this boundary unchanged.
	Rust owns strict sequence admission, template composition, native chemistry,
	and the returned handle-free insertion. Cancellation invalidates delivery; it
	does not claim to preempt a native library call already in progress.
	"""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, sequence: str, placement: object) -> None:
		"""Capture immutable strict-template inputs before this thread starts."""
		if type(sequence) is not str:
			raise TypeError("native peptide preparation requires exact text")
		if type(placement) is not ferrum_chem.InsertionPlacementV1:
			raise TypeError("native peptide preparation requires exact Ferrum placement")
		super().__init__()
		self._sequence = sequence
		self._placement = placement
		self._prepare_operation = ferrum_chem.prepare_supported_peptide_template_molecule_v1
		self._delivery_cancelled = False

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
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether future delivery has been invalidated."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Invalidate the result while native teardown finishes safely."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def run(self) -> None:
		"""Run native preparation and emit at most one still-current outcome."""
		try:
			prepared = self._prepare_operation(self._sequence, self._placement)
		except Exception as exc:
			if not self._delivery_cancelled and not self.isInterruptionRequested():
				self.failed.emit(
					FerrumNativePeptidePreparationFailure(type(exc).__name__, str(exc)),
				)
			return
		if not self._delivery_cancelled and not self.isInterruptionRequested():
			self.prepared.emit(prepared)

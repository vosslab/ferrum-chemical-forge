"""Ferrum background preparation for one Rust-owned SMILES insertion."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
from ferrum_qt.ferrum.background_job import FerrumDetachedJobThread
import ferrum_qt.ferrum.engine as engine


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeSmilesPreparationFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
class FerrumNativeSmilesPreparationWorker(FerrumDetachedJobThread):
	"""Prepare one complete Ferrum molecule without blocking the Qt thread.

	The compiled Ferrum function releases Python while Ferrum chemistry runs and
	returns an immutable value with no adapter or document-session handles.  A
	cancellation request invalidates delivery; it does not pretend to preempt a
	native library call already in progress.
	"""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, smiles: str, placement: object) -> None:
		"""Capture immutable preparation inputs before this thread starts."""
		if type(smiles) is not str or not smiles.strip():
			raise ValueError("Ferrum SMILES preparation requires nonblank text")
		if type(placement) is not engine.InsertionPlacementV1:
			raise TypeError("Ferrum SMILES preparation requires exact Ferrum placement")
		self._smiles = smiles
		self._placement = placement
		self._prepare_operation = engine.prepare_smiles_molecule_v1
		super().__init__(
			lambda: self._prepare_operation(self._smiles, self._placement),
			lambda error: FerrumNativeSmilesPreparationFailure(
				type(error).__name__, str(error),
			),
		)

	#============================================
	@classmethod
	def _from_fixture(cls, smiles: str, placement: object,
			prepare_operation: object) -> "FerrumNativeSmilesPreparationWorker":
		"""Construct a worker with one private deterministic test operation."""
		worker = cls(smiles, placement)
		if not callable(prepare_operation):
			raise TypeError("fixture preparation operation must be callable")
		worker._prepare_operation = prepare_operation
		return worker

	#============================================
	def _emit_success(self, prepared: object) -> None:
		"""Retain the import route's established prepared signal."""
		self.prepared.emit(prepared)

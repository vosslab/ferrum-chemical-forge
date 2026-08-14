"""OASA-free background preparation for one Rust-owned SMILES insertion."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import ferrum_chem


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeSmilesPreparationFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
class FerrumNativeSmilesPreparationWorker(PySide6.QtCore.QThread):
	"""Prepare one complete native molecule without blocking the Qt thread.

	The compiled Ferrum function releases Python while native chemistry runs and
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
			raise ValueError("native SMILES preparation requires nonblank text")
		if type(placement) is not ferrum_chem.InsertionPlacementV1:
			raise TypeError("native SMILES preparation requires exact Ferrum placement")
		super().__init__()
		self._smiles = smiles
		self._placement = placement
		self._prepare_operation = ferrum_chem.prepare_smiles_molecule_v1
		self._delivery_cancelled = False

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
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether future delivery has been invalidated."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Invalidate the result while allowing native teardown to finish safely."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def run(self) -> None:
		"""Run native preparation and emit at most one still-current outcome."""
		try:
			prepared = self._prepare_operation(self._smiles, self._placement)
		except Exception as exc:
			if not self._delivery_cancelled and not self.isInterruptionRequested():
				self.failed.emit(
					FerrumNativeSmilesPreparationFailure(type(exc).__name__, str(exc)),
				)
			return
		if not self._delivery_cancelled and not self.isInterruptionRequested():
			self.prepared.emit(prepared)

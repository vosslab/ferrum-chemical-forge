"""OASA-free background preparation for one Rust-owned InChI insertion."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import ferrum_chem


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeInchiPreparationFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
class FerrumNativeInchiPreparationWorker(PySide6.QtCore.QThread):
	"""Prepare one complete InChI molecule without blocking the Qt thread."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, inchi: str, placement: object) -> None:
		"""Capture immutable preparation inputs before this thread starts."""
		if type(inchi) is not str or not inchi.strip():
			raise ValueError("native InChI preparation requires nonblank text")
		if type(placement) is not ferrum_chem.InsertionPlacementV1:
			raise TypeError("native InChI preparation requires exact Ferrum placement")
		super().__init__()
		self._inchi = inchi
		self._placement = placement
		self._prepare_operation = ferrum_chem.prepare_inchi_molecule_v1
		self._delivery_cancelled = False

	#============================================
	@classmethod
	def _from_fixture(cls, inchi: str, placement: object,
			prepare_operation: object) -> "FerrumNativeInchiPreparationWorker":
		"""Construct a worker with one private deterministic test operation."""
		worker = cls(inchi, placement)
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
		"""Invalidate delivery without pretending to preempt native chemistry."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def run(self) -> None:
		"""Prepare the InChI molecule, emitting at most one current outcome."""
		try:
			prepared = self._prepare_operation(self._inchi, self._placement)
		except Exception as exc:
			if not self._delivery_cancelled and not self.isInterruptionRequested():
				self.failed.emit(
					FerrumNativeInchiPreparationFailure(type(exc).__name__, str(exc)),
				)
			return
		if not self._delivery_cancelled and not self.isInterruptionRequested():
			self.prepared.emit(prepared)

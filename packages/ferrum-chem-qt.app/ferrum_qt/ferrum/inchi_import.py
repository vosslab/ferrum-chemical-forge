"""Ferrum background preparation for one Rust-owned InChI insertion."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
from ferrum_qt.ferrum.background_job import FerrumDetachedJobThread
import ferrum_qt.ferrum.engine as engine


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeInchiPreparationFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
class FerrumNativeInchiPreparationWorker(FerrumDetachedJobThread):
	"""Prepare one complete InChI molecule without blocking the Qt thread."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, inchi: str, placement: object) -> None:
		"""Capture immutable preparation inputs before this thread starts."""
		if type(inchi) is not str or not inchi.strip():
			raise ValueError("Ferrum InChI preparation requires nonblank text")
		if type(placement) is not engine.InsertionPlacementV1:
			raise TypeError("Ferrum InChI preparation requires exact Ferrum placement")
		self._inchi = inchi
		self._placement = placement
		self._prepare_operation = engine.prepare_inchi_molecule_v1
		super().__init__(
			lambda: self._prepare_operation(self._inchi, self._placement),
			lambda error: FerrumNativeInchiPreparationFailure(
				type(error).__name__, str(error),
			),
		)

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
	def _emit_success(self, prepared: object) -> None:
		"""Retain the import route's established prepared signal."""
		self.prepared.emit(prepared)

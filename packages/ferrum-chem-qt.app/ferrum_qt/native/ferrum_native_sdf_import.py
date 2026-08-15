"""Bounded Rust-native preparation of every record in one local SDF file."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import ferrum_chem


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeSdfPreparationFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
class FerrumNativeSdfPreparationWorker(PySide6.QtCore.QThread):
	"""Read and prepare one bounded multi-record SDF outside the Qt thread."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, path: str, placement: object) -> None:
		"""Capture one exact local path and immutable Ferrum placement."""
		if type(path) is not str or not path:
			raise ValueError("native SDF preparation requires a nonempty path")
		if type(placement) is not ferrum_chem.InsertionPlacementV1:
			raise TypeError("native SDF preparation requires exact Ferrum placement")
		super().__init__()
		self._path = path
		self._placement = placement
		self._delivery_cancelled = False

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
		"""Read and prepare the SDF, emitting at most one current outcome."""
		try:
			prepared = ferrum_chem.prepare_sdf_file_v1(self._path, self._placement)
		except Exception as exc:
			if not self._delivery_cancelled and not self.isInterruptionRequested():
				self.failed.emit(
					FerrumNativeSdfPreparationFailure(type(exc).__name__, str(exc)),
				)
			return
		if not self._delivery_cancelled and not self.isInterruptionRequested():
			self.prepared.emit(prepared)

"""Bounded Ferrum preparation of every record in one local SDF file."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
from ferrum_qt.ferrum.background_job import FerrumDetachedJobThread
import ferrum_qt.ferrum.engine as engine


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeSdfPreparationFailure:
	"""Plain terminal failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str


#============================================
class FerrumNativeSdfPreparationWorker(FerrumDetachedJobThread):
	"""Read and prepare one bounded multi-record SDF outside the Qt thread."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, path: str, placement: object, route_handle: object) -> None:
		"""Capture one exact local source, placement, and registry route handle."""
		if type(path) is not str or not path:
			raise ValueError("Ferrum SDF preparation requires a nonempty path")
		if type(placement) is not engine.InsertionPlacementV1:
			raise TypeError("Ferrum SDF preparation requires exact Ferrum placement")
		self._path = path
		self._placement = placement
		self._route_handle = route_handle
		super().__init__(
			self._prepare_sdf_text,
			lambda error: FerrumNativeSdfPreparationFailure(
				type(error).__name__, str(error),
			),
		)

	#============================================
	def _prepare_sdf_text(self) -> object:
		"""Read descriptor-authorized text, then delegate parsing to Rust."""
		source = engine.DocumentSession.read_local_interchange_utf8_v1(
			self._path, self._route_handle,
		)
		return engine.prepare_sdf_molecules_v1(source, self._placement)

	#============================================
	def _emit_success(self, prepared: object) -> None:
		"""Retain the import route's established prepared signal."""
		self.prepared.emit(prepared)

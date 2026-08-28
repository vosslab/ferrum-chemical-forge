"""Local-document Open request types and detached Rust admission worker."""

# Standard Library
import dataclasses
import os
import pathlib

# PIP3 modules
import PySide6.QtCore

# local repo modules
import ferrum_qt.ferrum.engine as engine
from ferrum_qt.ferrum.background_job import FerrumDetachedJobThread


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeLocalDocumentOpenCatalogV2:
	"""Retain one immutable Rust-issued File/Open descriptor tuple per window."""

	descriptors: tuple[object, ...]

	#============================================
	@classmethod
	def from_rust(cls) -> "FerrumNativeLocalDocumentOpenCatalogV2":
		"""Capture the complete catalog before any Qt File/Open route starts."""
		return cls(tuple(engine.DocumentSession.local_document_open_descriptors_v2()))

	#============================================
	def descriptor_for_path(self, path: str) -> object | None:
		"""Return the one descriptor whose Rust-issued suffix accepts ``path``."""
		suffix = pathlib.Path(path).suffix.lower()
		for descriptor in self.descriptors:
			if suffix in descriptor.suffixes:
				return descriptor
		return None

	#============================================
	def replacement_descriptor_for_path(
			self, path: str,
			) -> object | None:
		"""Return an accepting descriptor only when it permits tab replacement."""
		descriptor = self.descriptor_for_path(path)
		if descriptor is not None and descriptor.allows_current_tab_replacement:
			return descriptor
		return None

	#============================================
	def route_handle_for_suffix(self, suffix: str) -> object:
		"""Return an exact Rust-issued handle for one catalog suffix."""
		if type(suffix) is not str:
			raise TypeError("Ferrum local-document suffix lookup requires exact text")
		for descriptor in self.descriptors:
			if suffix.lower() in descriptor.suffixes:
				return descriptor.route_handle
		raise RuntimeError(f"Ferrum did not publish a local document route for {suffix}")


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeLocalDocumentOpenFailure:
	"""Plain typed failure facts safe to deliver to the Qt thread."""

	error_type: str
	message: str
	stage: str | None
	limit: int | None
	actual: int | None
	observed_at_least: int | None
	category: str | None = None
	detail: str | None = None


#============================================
class FerrumNativeLocalDocumentOpenWorker(FerrumDetachedJobThread):
	"""Admit one bounded local document outside the Qt event thread."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, path: str, route_handle: object) -> None:
		"""Capture one exact path and its catalog-issued opaque route handle."""
		if type(path) is not str or not path or not os.path.isabs(path):
			raise ValueError("Ferrum local-document Open requires a nonempty absolute path")
		if route_handle is None:
			raise ValueError("Ferrum local-document Open requires an API route handle")
		self._path = path
		self._route_handle = route_handle
		super().__init__(
			lambda: engine.DocumentSession.prepare_local_document_open_file_v2(
				self._path, self._route_handle,
			), _local_document_open_failure,
		)

	#============================================
	def _emit_success(self, prepared: object) -> None:
		"""Retain the Open route's established prepared signal."""
		self.prepared.emit(prepared)


#============================================
def _local_document_open_failure(exc: Exception) -> FerrumNativeLocalDocumentOpenFailure:
	"""Copy stable ingress facts without retaining a worker-thread exception."""
	if type(exc) is engine.DocumentInputError:
		return FerrumNativeLocalDocumentOpenFailure(
			type(exc).__name__, str(exc), getattr(exc, "stage", None),
			getattr(exc, "limit", None), getattr(exc, "actual", None),
			getattr(exc, "observed_at_least", None), getattr(exc, "category", None),
			getattr(exc, "detail", None),
		)
	return FerrumNativeLocalDocumentOpenFailure(
		type(exc).__name__, str(exc), None, None, None, None, None, None,
	)

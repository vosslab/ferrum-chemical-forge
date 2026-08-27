"""Local-document Open request types and detached Rust admission worker."""

# Standard Library
import dataclasses
import enum
import os
import pathlib

# PIP3 modules
import PySide6.QtCore

# local repo modules
import ferrum_qt.ferrum.engine as engine
from ferrum_qt.ferrum.background_job import FerrumDetachedJobThread


#============================================
class _LocalDocumentOpenRouteKind(enum.Enum):
	"""Closed Qt request adapter; Rust authenticates the admitted kind later."""

	CDML = "cdml"
	DECODED_CDSVG = "decoded_cdsvg"
	INTERCHANGE = "interchange"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeLocalIngressRegistryV1:
	"""Retain each Rust-issued local-ingress descriptor set for one window."""

	local_document_open_descriptors: tuple[engine.LocalInterchangeOpenDescriptorV1, ...]
	local_interchange_open_descriptors: tuple[engine.LocalInterchangeOpenDescriptorV1, ...]

	#============================================
	@classmethod
	def from_rust(cls) -> "FerrumNativeLocalIngressRegistryV1":
		"""Capture the complete immutable ingress registry before any Qt route starts."""
		return cls(
			tuple(engine.DocumentSession.local_document_open_descriptors_v1()),
			tuple(engine.DocumentSession.local_interchange_open_descriptors_v1()),
		)

	#============================================
	def interchange_route_handle_for_suffix(self, suffix: str) -> object:
		"""Return one Rust-issued handle for an exact registered interchange suffix."""
		for descriptor in self.local_interchange_open_descriptors:
			if suffix not in descriptor.suffixes:
				continue
			if descriptor.route_handle is None:
				raise RuntimeError("Ferrum published an interchange route without a handle")
			return descriptor.route_handle
		raise RuntimeError(f"Ferrum did not publish a local interchange route for {suffix}")


#============================================
def _local_document_open_route_for_path(
		path: str, descriptors: tuple[object, ...] = (),
		) -> _LocalDocumentOpenRouteKind | None:
	"""Select the Rust-issued route kind from its descriptor suffix."""
	suffix = pathlib.Path(path).suffix.lower()
	for descriptor in descriptors:
		if suffix not in descriptor.suffixes:
			continue
		if descriptor.route_handle is not None:
			return _LocalDocumentOpenRouteKind.INTERCHANGE
		return {
			"cdml": _LocalDocumentOpenRouteKind.CDML,
			"decoded_cdsvg": _LocalDocumentOpenRouteKind.DECODED_CDSVG,
		}.get(descriptor.source_kind)
	return None


#============================================
def _current_tab_replacement_route_for_path(
		path: str, descriptors: tuple[object, ...],
		) -> _LocalDocumentOpenRouteKind | None:
	"""Select only a registry route whose direction permits replacement."""
	suffix = pathlib.Path(path).suffix.lower()
	for descriptor in descriptors:
		if suffix in descriptor.suffixes and descriptor.allows_current_tab_replacement:
			return _local_document_open_route_for_path(path, (descriptor,))
	return None


#============================================
def _interchange_route_handle_for_path(
		path: str, descriptors: tuple[object, ...],
		) -> object | None:
	"""Return the API-issued handle whose suffix accepts this path."""
	suffix = pathlib.Path(path).suffix.lower()
	for descriptor in descriptors:
		if suffix in descriptor.suffixes:
			return descriptor.route_handle
	return None


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
	def __init__(
			self, path: str,
			source_kind: _LocalDocumentOpenRouteKind = _LocalDocumentOpenRouteKind.CDML,
			route_handle: object | None = None,
			) -> None:
		"""Capture one exact local path and its closed Rust admission route."""
		if type(path) is not str or not path or not os.path.isabs(path):
			raise ValueError("Ferrum local-document Open requires a nonempty absolute path")
		if type(source_kind) is not _LocalDocumentOpenRouteKind:
			raise TypeError("Ferrum local-document Open requires a source kind")
		if source_kind is _LocalDocumentOpenRouteKind.INTERCHANGE and route_handle is None:
			raise ValueError("Ferrum interchange Open requires an API route handle")
		self._path = path
		self._source_kind = source_kind
		self._prepare_operation = {
			_LocalDocumentOpenRouteKind.CDML:
			engine.DocumentSession.prepare_local_cdml_file_v1,
			_LocalDocumentOpenRouteKind.DECODED_CDSVG:
			engine.DocumentSession.prepare_local_decoded_cdsvg_file_v1,
			_LocalDocumentOpenRouteKind.INTERCHANGE:
			lambda path: engine.DocumentSession.prepare_local_interchange_file_v1(
				path, route_handle,
			),
		}[source_kind]
		super().__init__(
			lambda: self._prepare_operation(self._path), _local_document_open_failure,
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

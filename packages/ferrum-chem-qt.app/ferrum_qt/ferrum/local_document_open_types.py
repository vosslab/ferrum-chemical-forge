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
class _LocalDocumentSourceKind(enum.Enum):
	"""Closed Qt request adapter; Rust authenticates the admitted kind later."""

	CDML = "cdml"
	DECODED_CDSVG = "decoded_cdsvg"
	INTERCHANGE = "interchange"


#============================================
def _local_document_source_kind_for_path(
		path: str, descriptors: tuple[object, ...] = (),
		) -> _LocalDocumentSourceKind | None:
	"""Select a named Rust admission profile solely from the requested suffix."""
	suffix = pathlib.Path(path).suffix.lower()
	if suffix == ".cdml":
		return _LocalDocumentSourceKind.CDML
	if suffix == ".svg":
		return _LocalDocumentSourceKind.DECODED_CDSVG
	if any(suffix in descriptor.suffixes for descriptor in descriptors):
		return _LocalDocumentSourceKind.INTERCHANGE
	return None


#============================================
def _current_tab_replacement_source_kind_for_path(
		path: str,
		) -> _LocalDocumentSourceKind | None:
	"""Select the closed CDML/CDSVG policy for explicit tab replacement."""
	suffix = pathlib.Path(path).suffix.lower()
	if suffix == ".cdml":
		return _LocalDocumentSourceKind.CDML
	if suffix == ".svg":
		return _LocalDocumentSourceKind.DECODED_CDSVG
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
			source_kind: _LocalDocumentSourceKind = _LocalDocumentSourceKind.CDML,
			route_handle: object | None = None,
			) -> None:
		"""Capture one exact local path and its closed Rust admission route."""
		if type(path) is not str or not path or not os.path.isabs(path):
			raise ValueError("Ferrum local-document Open requires a nonempty absolute path")
		if type(source_kind) is not _LocalDocumentSourceKind:
			raise TypeError("Ferrum local-document Open requires a source kind")
		if source_kind is _LocalDocumentSourceKind.INTERCHANGE and route_handle is None:
			raise ValueError("Ferrum interchange Open requires an API route handle")
		self._path = path
		self._source_kind = source_kind
		self._prepare_operation = {
			_LocalDocumentSourceKind.CDML:
			engine.DocumentSession.prepare_local_cdml_file_v1,
			_LocalDocumentSourceKind.DECODED_CDSVG:
			engine.DocumentSession.prepare_local_decoded_cdsvg_file_v1,
			_LocalDocumentSourceKind.INTERCHANGE:
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

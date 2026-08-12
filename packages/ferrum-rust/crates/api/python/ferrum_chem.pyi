"""Typed public API for Ferrum-Chem's compiled document extension."""

import pathlib


class FerrumError(Exception): ...


class DocumentError(FerrumError): ...


class DocumentLoadError(DocumentError): ...


class DocumentSerializationError(DocumentError): ...


class RevisionConflictError(DocumentError):
	expected: int
	actual: int


class RevisionExhaustedError(DocumentError): ...


class HistoryUnavailableError(DocumentError): ...


class OperationValidationError(DocumentError): ...


class InvalidAtomElementError(OperationValidationError): ...


class InvalidDocumentObjectIdError(OperationValidationError):
	object_id: str


class UnknownDocumentObjectError(OperationValidationError):
	object_id: str


class PreparedOperationError(DocumentError): ...


class PreparedOperationConsumedError(PreparedOperationError): ...


class PublicationError(FerrumError):
	path: str
	reason: str


class InvalidDestinationError(PublicationError): ...


class PublicationNotStartedError(PublicationError): ...


class PublicationPossiblyCompletedError(PublicationError): ...


class DocumentSnapshot:
	"""Immutable independent copy of one authoritative CDML revision."""
	cdml: str
	revision: int
	digest: str
	is_dirty: bool


class SessionObservationV1:
	"""Immutable revision-checked observation carrying an owned snapshot."""
	snapshot: DocumentSnapshot


class Publication:
	"""Immutable result of one ordinary save or recovery export."""
	snapshot: DocumentSnapshot
	published_snapshot: DocumentSnapshot
	outcome: str


class DocumentOperationV1:
	"""Closed Rust-owned V1 operation grammar."""
	@staticmethod
	def set_atom_element(atom_id: str, element: str) -> DocumentOperationV1: ...


class PreparedAtomInsertion:
	"""Opaque revision-bound one-use prepared atom insertion."""
	identifier: str


class DocumentSession:
	"""Thread-affine mutable session with synchronous, owned-value methods only."""
	@staticmethod
	def load(cdml: str) -> DocumentSession: ...
	def snapshot(self) -> DocumentSnapshot: ...
	def observe(self, expected_revision: int) -> SessionObservationV1: ...
	def submit(
		self,
		expected_revision: int,
		operation: DocumentOperationV1,
	) -> DocumentSnapshot: ...
	def undo(self, expected_revision: int) -> DocumentSnapshot: ...
	def redo(self, expected_revision: int) -> DocumentSnapshot: ...
	def prepare_create_atom(
		self,
		expected_revision: int,
		molecule_id: str,
		atom_id: str,
		element: str,
	) -> PreparedAtomInsertion: ...
	def commit_create_atom(
		self,
		expected_revision: int,
		prepared: PreparedAtomInsertion,
	) -> DocumentSnapshot: ...
	def save_atomic(
		self,
		path: str | pathlib.Path,
		expected_revision: int,
	) -> Publication: ...
	def recovery_export(
		self,
		path: str | pathlib.Path,
		expected_revision: int,
	) -> Publication: ...

"""Per-tab ownership and teardown boundary for Ferrum-Qt documents."""

# Standard Library
import errno
import dataclasses
import os
import stat

# PIP3 modules

# local repo modules
import ferrum_qt.setup.canvas_setup
import ferrum_qt.setup.mode_setup
import ferrum_qt.canvas.document_projection
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.io.cdml_candidate
import ferrum_qt.io.user_template_catalog
import ferrum_qt.models.backend_revision_history
import ferrum_qt.models.document
import ferrum_qt.models.projection_lifecycle
import ferrum_qt.undo.commands
import ferrum_qt.wavy_geometry
import oasa.cdml_document
import oasa.cdml_ftext
import oasa.cdml_render
import oasa.cdml_writer
import oasa.safe_xml
import oasa.biomolecule_template_placement
import oasa.template_placement


@dataclasses.dataclass(frozen=True)

class PersistentActionOutcome:
	"""Uniform immutable result for a persistent-operation submission."""

	status: str
	message: str
	commit: oasa.cdml_document.CDMLCommit | None
	submitted: bool = False
	structural_result: oasa.cdml_document.CDMLStructuralEditResult | None = None
	failure_kind: str | None = None


@dataclasses.dataclass(frozen=True)
class _PreparedPersistentOperation:
	"""One validated operation waiting for its named backend commit executor."""

	executor_key: str
	expected_revision: int
	value: object
	provisional_selection_keys: frozenset[tuple[str, str]] = frozenset()
	preserve_existing_selection: bool = False

	#============================================
	def __post_init__(self) -> None:
		"""Keep proposed selection correlation data immutable and plain."""
		selection_keys = frozenset(self.provisional_selection_keys)
		if any(
			not isinstance(kind, str) or not isinstance(identifier, str)
			for kind, identifier in selection_keys
		):
			raise TypeError("Provisional selection keys must be string pairs")
		object.__setattr__(self, "provisional_selection_keys", selection_keys)
		if not isinstance(self.preserve_existing_selection, bool):
			raise TypeError("Selection preservation flag must be boolean")


@dataclasses.dataclass(frozen=True)
class CloseState:
	"""Plain backend and provenance facts used for a close decision."""

	backend_dirty: bool
	backend_unseen: bool
	legacy_local_pending: bool
	authoritative_save_eligible: bool

	#============================================
	@property
	def needs_confirmation(self) -> bool:
		"""Return whether closing would discard backend or local pending content."""
		needed = (
			self.backend_dirty
			or self.backend_unseen
			or self.legacy_local_pending
		)
		return needed

	#============================================
	@property
	def uses_recovery_export(self) -> bool:
		"""Return whether a prompted close must use Recovery Export, not Save."""
		return self.needs_confirmation and not self.authoritative_save_eligible


class PreparedNativeCDML:
	"""One-use detached native projection staged from immutable backend CDML.

	Instances are made only by :meth:`DocumentSession.prepare_native_cdml`.
	The detached Qt document remains private until one receiving session consumes
	it.  Callers may inspect the immutable snapshot or canonical CDML, but cannot
	mutate the staged projection before installation.  Installation parses the
	canonical snapshot again into the receiving session's private authority.
	"""

	def __init__(
			self, factory_token: object, snapshot: oasa.cdml_document.CDMLSnapshot,
			document: ferrum_qt.models.document.Document,
			) -> None:
		"""Create a factory-only value with a private detached Qt document."""
		if factory_token is not _PREPARED_NATIVE_FACTORY_TOKEN:
			raise TypeError("PreparedNativeCDML objects must come from native staging")
		self._snapshot = snapshot
		self._document = document
		self._consumed = False

	#============================================
	@property
	def snapshot(self) -> oasa.cdml_document.CDMLSnapshot:
		"""Return the immutable canonical backend snapshot used for staging."""
		return self._snapshot

	#============================================
	@property
	def canonical_cdml(self) -> str:
		"""Return the immutable canonical CDML value staged for installation."""
		return self._snapshot.cdml

	#============================================
	@property
	def consumed(self) -> bool:
		"""Return whether a session has already adopted this staged projection."""
		return self._consumed

	#============================================
	def _peek(
			self,
			) -> tuple[str, ferrum_qt.models.document.Document]:
		"""Return the private staged projection without completing transfer."""
		if self._consumed:
			raise RuntimeError("Prepared native CDML has already been consumed")
		return self._snapshot.cdml, self._document

	#============================================
	def _finalize(self) -> None:
		"""Complete a successful native transfer exactly once."""
		if self._consumed:
			raise RuntimeError("Prepared native CDML has already been consumed")
		self._consumed = True


_PREPARED_NATIVE_FACTORY_TOKEN = object()
_PREPARED_IMPORTED_FACTORY_TOKEN = object()


class PreparedImportedCDML(PreparedNativeCDML):
	"""One-use detached projection staged from an external complete CDML file."""

	def __init__(
			self, factory_token: object, snapshot: oasa.cdml_document.CDMLSnapshot,
			document: ferrum_qt.models.document.Document,
			) -> None:
		if factory_token is not _PREPARED_IMPORTED_FACTORY_TOKEN:
			raise TypeError("PreparedImportedCDML objects must come from import staging")
		self._snapshot = snapshot
		self._document = document
		self._consumed = False


#============================================
class BackendSnapshotPublicationError(RuntimeError):
	"""Report a filesystem result that may have published CDML already."""


#============================================
def _resolved_publication_target(file_path: str) -> str:
	"""Return the target normal writes reach, following an existing symlink."""
	return os.path.realpath(os.path.abspath(file_path))


#============================================
def _write_backend_snapshot(
		file_path: str, snapshot: oasa.cdml_document.CDMLSnapshot,
		) -> None:
	"""Atomically publish one immutable snapshot without changing session state.

	A failure before replacement leaves an existing target unchanged.  A failure
	after replacement is deliberately distinguished because the named file may
	already contain ``snapshot.cdml`` while durability remains unconfirmed.
	"""
	target_path = _resolved_publication_target(file_path)
	target_directory = os.path.dirname(target_path)
	target_mode = None
	try:
		target_status = os.stat(target_path)
	except FileNotFoundError:
		pass
	else:
		if not stat.S_ISREG(target_status.st_mode):
			raise OSError("Backend CDML target is not a regular file: %s" % target_path)
		target_mode = stat.S_IMODE(target_status.st_mode)
	staged_path = None
	try:
		for _attempt in range(100):
			candidate = os.path.join(
				target_directory,
				".%s.ferrum-%s.tmp" % (os.path.basename(target_path), os.urandom(8).hex()),
			)
			try:
				file_descriptor = os.open(
					candidate, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o666,
				)
			except FileExistsError:
				continue
			staged_path = candidate
			break
		else:
			raise OSError("Could not create a unique staged backend CDML file")
		try:
			if target_mode is not None:
				os.fchmod(file_descriptor, target_mode)
			with os.fdopen(file_descriptor, "w", encoding="utf-8") as destination:
				file_descriptor = None
				destination.write(snapshot.cdml)
				destination.flush()
				os.fsync(destination.fileno())
		except Exception:
			if file_descriptor is not None:
				try:
					os.close(file_descriptor)
				except OSError:
					# Staged-path cleanup below remains best effort.  Preserve the
					# write, fchmod, or fdopen diagnostic that triggered this path.
					pass
			raise
		os.replace(staged_path, target_path)
		staged_path = None
		try:
			directory_flags = os.O_RDONLY
			if hasattr(os, "O_DIRECTORY"):
				directory_flags |= os.O_DIRECTORY
			directory_descriptor = os.open(target_directory, directory_flags)
			try:
				os.fsync(directory_descriptor)
			finally:
				os.close(directory_descriptor)
		except OSError as exc:
			if exc.errno not in (errno.EINVAL, errno.ENOTSUP, errno.EOPNOTSUPP, errno.ENOSYS):
				raise BackendSnapshotPublicationError(
					"CDML target was atomically replaced but directory durability "
					"confirmation failed; the target may contain the exact canonical "
					"snapshot, publication durability is unconfirmed, and the publisher "
					"changed no session state",
				) from exc
	finally:
		if staged_path is not None:
			try:
				os.unlink(staged_path)
			except FileNotFoundError:
				pass
			except OSError:
				pass


#============================================

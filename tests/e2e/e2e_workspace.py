"""Private crash-recoverable workspace leases for local E2E workflows."""

# Standard Library
import fcntl
import os
import pathlib
import shutil
import stat
import tempfile
import types


_PARENT_NAME = "ferrum-e2e-workspaces-v1"
_SWEEP_LOCK_NAME = ".ferrum-e2e-sweep.lock"
_OWNER_MARKER_NAME = ".ferrum-e2e-owner.lock"


#============================================
def _private_parent() -> pathlib.Path:
	"""Return the private parent that contains only helper-owned workspaces."""
	parent = pathlib.Path(tempfile.gettempdir()) / _PARENT_NAME
	parent.mkdir(mode=0o700, exist_ok=True)
	parent_status = parent.lstat()
	# ASVS 2.2.1/2.2.2: validate the sole sweep parent at this trusted boundary;
	# ASVS 2.3.4: do so before its lock can authorize shared-workspace recovery.
	if not stat.S_ISDIR(parent_status.st_mode):
		raise RuntimeError(f"E2E workspace parent is not a direct directory: {parent}")
	if parent_status.st_uid != os.geteuid():
		raise RuntimeError(f"E2E workspace parent is not owned by this user: {parent}")
	if stat.S_IMODE(parent_status.st_mode) & 0o077:
		raise RuntimeError(f"E2E workspace parent is not private: {parent}")
	return parent


#============================================
def _open_locked_file(path: pathlib.Path, flags: int) -> int:
	"""Open one regular private lock file and hold its exclusive advisory lock."""
	no_follow = getattr(os, "O_NOFOLLOW", 0)
	descriptor = os.open(path, flags | no_follow, 0o600)
	fcntl.flock(descriptor, fcntl.LOCK_EX)
	return descriptor


#============================================
def _release_lock(descriptor: int) -> None:
	"""Release and close one advisory lock descriptor."""
	fcntl.flock(descriptor, fcntl.LOCK_UN)
	os.close(descriptor)


#============================================
def _is_regular_directory(path: pathlib.Path) -> bool:
	"""Report whether path is a direct regular directory entry, never a symlink."""
	try:
		path_status = path.lstat()
	except FileNotFoundError:
		return False
	return stat.S_ISDIR(path_status.st_mode)


#============================================
def _is_regular_marker(path: pathlib.Path) -> bool:
	"""Report whether path is the exact regular ownership marker file."""
	try:
		marker_status = path.lstat()
	except FileNotFoundError:
		return False
	return stat.S_ISREG(marker_status.st_mode)


#============================================
def _reclaim_abandoned_children(parent: pathlib.Path) -> None:
	"""Remove only marker-owned children whose owner lease lock is no longer held."""
	for child in parent.iterdir():
		if not _is_regular_directory(child):
			continue
		marker = child / _OWNER_MARKER_NAME
		if not _is_regular_marker(marker):
			continue
		try:
			descriptor = os.open(marker, os.O_RDWR | getattr(os, "O_NOFOLLOW", 0))
		except FileNotFoundError:
			continue
		try:
			try:
				fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
			except BlockingIOError:
				continue
			shutil.rmtree(child)
		finally:
			os.close(descriptor)


#============================================
class E2EWorkspaceLease:
	"""Lease one private invocation workspace with crash-safe abandoned-work recovery."""

	def __init__(self) -> None:
		self._workspace: pathlib.Path | None = None
		self._owner_descriptor: int | None = None

	def __enter__(self) -> str:
		"""Allocate and lock one private workspace after reclaiming abandoned peers."""
		parent = _private_parent()
		sweep_descriptor = _open_locked_file(parent / _SWEEP_LOCK_NAME, os.O_RDWR | os.O_CREAT)
		try:
			_reclaim_abandoned_children(parent)
			workspace = pathlib.Path(tempfile.mkdtemp(prefix="workspace-", dir=parent))
			try:
				owner_descriptor = _open_locked_file(
					workspace / _OWNER_MARKER_NAME, os.O_RDWR | os.O_CREAT | os.O_EXCL,
				)
			except BaseException as acquisition_error:
				try:
					shutil.rmtree(workspace)
				except BaseException as cleanup_error:
					acquisition_error.add_note(
						"E2E workspace acquisition cleanup also failed: "
						f"{cleanup_error!r}",
					)
				raise
			self._workspace = workspace
			self._owner_descriptor = owner_descriptor
		finally:
			_release_lock(sweep_descriptor)
		# Native publication opens every parent with O_NOFOLLOW. macOS exposes its
		# temporary directory through the /var symlink, so hand callers the verified
		# physical directory rather than a spelling Rust must reject.
		return os.fspath(workspace.resolve(strict=True))

	def __exit__(
			self, exception_type: type[BaseException] | None,
			exception: BaseException | None, traceback: types.TracebackType | None,
			) -> bool:
		"""Remove the workspace while preserving the primary scenario exception."""
		if self._workspace is None or self._owner_descriptor is None:
			raise RuntimeError("E2E workspace lease exited without an acquired workspace")
		try:
			shutil.rmtree(self._workspace)
		except BaseException as cleanup_error:
			if exception is None:
				raise
			exception.add_note(
				"E2E temporary-workspace cleanup also failed: "
				f"{cleanup_error!r}",
			)
		finally:
			_release_lock(self._owner_descriptor)
			self._owner_descriptor = None
			self._workspace = None
		return False

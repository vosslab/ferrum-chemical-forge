"""Publication operations for one live Rust-owned native document tab."""

# Standard Library
import pathlib

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab_errors


#============================================
class FerrumNativeDocumentTabSavePresentationError(
		ferrum_qt.native.ferrum_native_document_tab_errors.FerrumNativeDocumentTabError,
		):
	"""A confirmed Rust save whose replacement projection could not be installed."""

	#============================================
	def __init__(self, path: str | pathlib.Path, publication: object) -> None:
		"""Retain the confirmed publication for truthful caller recovery messaging."""
		self.path = pathlib.Path(path)
		self.publication = publication
		super().__init__(
			"Rust publication completed, but its replacement render observation "
			"could not be installed",
		)


#============================================
class FerrumNativeDocumentTabPublicationMixin:
	"""Publish saved or recovery CDML without creating a second session authority."""

	#============================================
	def save_atomic(self, path: str | pathlib.Path) -> object:
		"""Publish the current Rust revision and adopt only a confirmed saved state."""
		self._require_live()
		self._require_current_projection()
		snapshot = self.current_snapshot
		publication = self._session.save_atomic(path, snapshot.revision)
		if not publication.outcome.is_confirmed:
			return publication
		observation = self._session.observe_render(publication.snapshot.revision)
		if not self._install_observation(observation):
			raise FerrumNativeDocumentTabSavePresentationError(path, publication)
		self._file_path = pathlib.Path(path)
		self._title = self._file_path.name
		return publication

	#============================================
	def backend_snapshot_for_recovery_export(self) -> object:
		"""Return the exact live Rust snapshot without consulting the Qt projection."""
		self._require_live()
		snapshot = self._session.snapshot()
		return snapshot

	#============================================
	def recovery_export(self, path: str | pathlib.Path, expected_revision: int) -> object:
		"""Copy one revision-gated backend snapshot without Save presentation effects."""
		self._require_live()
		publication = self._session.recovery_export(path, expected_revision)
		return publication

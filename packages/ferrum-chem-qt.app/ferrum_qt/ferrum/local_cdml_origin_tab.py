"""Private local-source identity, provenance, and bootstrap state for Ferrum Open."""

# Standard Library
import pathlib


#============================================
class FerrumNativeLocalCdmlOriginTabMixin:
	"""Retain Qt-local source facts that never enter Rust documents or CDML."""

	#============================================
	def _initialize_local_cdml_origin(self) -> None:
		"""Start with no admitted-file identity and no replacement eligibility."""
		self._local_cdml_origin_token: object | None = None
		self._local_document_source_path: pathlib.Path | None = None
		self._local_document_source_kind: str | None = None
		self._is_initial_placeholder = False

	#============================================
	@property
	def local_cdml_origin_token(self) -> object | None:
		"""Return the opaque Rust-issued identity for one admitted local source."""
		return self._local_cdml_origin_token

	#============================================
	def _adopt_local_cdml_origin_token(self, token: object) -> None:
		"""Retain one private Rust receipt fact for live-tab identity matching."""
		self._require_live()
		if token is None:
			raise TypeError("Ferrum local CDML origin requires a Rust identity token")
		if self._local_cdml_origin_token is not None:
			raise ValueError("Ferrum tab already has a local CDML origin identity")
		self._local_cdml_origin_token = token

	#============================================
	def _adopt_local_document_origin(
			self, path: str | pathlib.Path, source_kind: str, token: object,
			) -> None:
		"""Retain one admitted source without conflating it with CDML publication.

		A decoded CD-SVG is an extraction-only source, so it intentionally has no
		``file_path`` save baseline.  The descriptor token remains independent of a
		later CDML Save As destination.
		"""
		self._require_live()
		origin = pathlib.Path(path)
		if not origin.is_absolute():
			raise ValueError("Ferrum document origins must be absolute paths")
		if source_kind not in {"cdml", "decoded_cdsvg"}:
			raise ValueError("Ferrum document origin has an unknown source kind")
		if self._local_document_source_path is not None:
			raise ValueError("Ferrum tab already has local document source provenance")
		self._adopt_local_cdml_origin_token(token)
		self._local_document_source_path = origin
		self._local_document_source_kind = source_kind
		if source_kind == "cdml":
			self._adopt_loaded_origin_path(origin)
			return
		if origin.suffix.lower() != ".svg":
			raise ValueError("decoded CD-SVG sources must use the .svg extension")
		self._title = origin.name
		self.setToolTip(
			f"Opened from {origin.name}; embedded CDML document. Save writes CDML.",
		)
		self.setAccessibleDescription(
			f"Opened from {origin.name}; embedded CDML document. Save writes CDML.",
		)

	#============================================
	def _refresh_local_document_origin_presentation(self) -> None:
		"""Keep decoded-source provenance visible after a successful CDML Save As."""
		description = self.local_document_source_description
		if description is None:
			return
		self.setToolTip(description)
		self.setAccessibleDescription(description)

	#============================================
	@property
	def local_document_source_description(self) -> str | None:
		"""Return tab-entry guidance for an extraction-only decoded CD-SVG source."""
		if self._local_document_source_kind != "decoded_cdsvg":
			return None
		if self._local_document_source_path is None:
			return None
		if self.file_path is None:
			return (
				f"Opened from {self._local_document_source_path.name}; embedded CDML "
				"document. Save writes CDML."
			)
		return (
			f"Opened from {self._local_document_source_path.name}; embedded CDML document "
			f"saved as {self.file_path.name}."
		)

	#============================================
	def _mark_initial_placeholder(self) -> None:
		"""Mark only the bootstrap page as eligible for first-Open replacement."""
		self._require_live()
		if self.current_snapshot.revision != 0 or self.is_dirty or self.file_path is not None:
			raise ValueError("only a clean revision-zero untitled tab is a bootstrap placeholder")
		self._is_initial_placeholder = True

	#============================================
	def is_pristine_initial_placeholder(self) -> bool:
		"""Return the narrow Qt lifecycle predicate for automatic first Open."""
		return (
			not self._disposed
			and self._is_initial_placeholder
			and self._local_cdml_origin_token is None
			and self.file_path is None
			and not self.is_dirty
			and not self.requires_refresh
			and self.current_snapshot.revision == 0
		)

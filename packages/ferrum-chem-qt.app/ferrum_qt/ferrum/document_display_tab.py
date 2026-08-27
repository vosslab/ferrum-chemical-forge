"""Tab-owned display-palette authority and retained refresh lifecycle."""

# local repo modules
import ferrum_qt.ferrum.document_display_refresh
import ferrum_qt.themes.document_display_palette
import ferrum_qt.themes.theme_manager


#============================================
class FerrumNativeDocumentDisplayTabMixin:
	"""Own one immutable document palette and its attached refreshables."""

	def _initialize_document_display_palette(
			self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Install the exact construction palette before the first scene is built."""
		if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
			raise TypeError("Ferrum document tab requires a document display palette")
		self._document_display_palette = palette
		self._document_display_refreshables = (
			ferrum_qt.ferrum.document_display_refresh.
			DocumentDisplayPaletteRefreshRegistryV1()
		)

	#============================================
	@property
	def document_display_palette(
			self,
			) -> ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
		"""Return this tab's current immutable document-display palette."""
		return self._document_display_palette

	#============================================
	def register_document_display_refreshable(
			self,
			refreshable: ferrum_qt.ferrum.document_display_refresh.DocumentDisplayRefreshableV1,
			) -> None:
		"""Retain one attached transient document-display object for theme refresh."""
		self._document_display_refreshables.register(refreshable)

	#============================================
	def unregister_document_display_refreshable(
			self,
			refreshable: ferrum_qt.ferrum.document_display_refresh.DocumentDisplayRefreshableV1,
			) -> None:
		"""Release one detached transient document-display object."""
		self._document_display_refreshables.unregister(refreshable)

	#============================================
	def apply_theme_change(self, change: object) -> None:
		"""Refresh retained Qt material without asking Rust for document state."""
		if type(change) is not ferrum_qt.themes.theme_manager.ThemeChangeV1:
			raise TypeError("Ferrum document tab requires ThemeChangeV1")
		if type(change.palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
			raise TypeError("Ferrum document tab requires a document display palette")
		self._document_display_palette = change.palette
		self._controller.refresh_display_palette(change.palette)
		self._view.refresh_document_display_palette(change.palette)
		self._document_display_refreshables.refresh(change.palette)

	#============================================
	def _dispose_document_display_refreshables(self) -> None:
		"""Release attached display-object references at the tab lifecycle boundary."""
		self._document_display_refreshables.clear()

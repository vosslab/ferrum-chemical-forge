"""Nominal retained-root contract for tab-owned display palette refresh."""

# local repo modules
import ferrum_qt.themes.document_display_palette


#============================================
class DisplayPaletteRefreshable:
	"""One admitted retained Qt root whose material follows the active palette."""

	#============================================
	def refresh_display_palette(self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Replace retained display material without changing issued geometry."""
		del palette
		raise NotImplementedError("retained palette roots implement refresh_display_palette")

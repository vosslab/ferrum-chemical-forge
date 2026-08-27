"""Qt projection of a closed Rust-issued path preview."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.themes.document_display_palette
import ferrum_qt.ferrum.document_display_refresh


#============================================
def create_overlay(tab: object, overlay: object) -> object:
	"""Paint the ordered Rust path exactly, with no local geometry synthesis."""
	path = PySide6.QtGui.QPainterPath()
	points = overlay.points
	path.moveTo(points[0][0], points[0][1])
	for x, y in points[1:]:
		path.lineTo(x, y)
	if overlay.closed:
		path.closeSubpath()
	item = tab.view.scene().addPath(path)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	refreshable = _PathDisplayRefreshable(item, overlay)
	refreshable.refresh_document_display_palette(_display_palette(tab))
	ferrum_qt.ferrum.document_display_refresh.register_attached_document_display_refreshable(
		tab, item, refreshable,
	)
	return item


#============================================
class _PathDisplayRefreshable:
	"""Replace a path preview's retained tagged material without rebuilding its path."""

	def __init__(self, item: object, overlay: object) -> None:
		"""Retain the item and opaque renderer preview facts."""
		self._item = item
		self._overlay = overlay

	#============================================
	def refresh_document_display_palette(
			self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Refresh only the tagged stroke and optional fill material."""
		pen = PySide6.QtGui.QPen(_paint(palette, self._overlay.stroke_paint))
		pen.setWidthF(self._overlay.stroke_width)
		brush = PySide6.QtGui.QBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
		if self._overlay.fill_paint is not None:
			brush = PySide6.QtGui.QBrush(_paint(palette, self._overlay.fill_paint))
		self._item.setPen(pen)
		self._item.setBrush(brush)


#============================================
def _display_palette(tab: object) -> ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
	"""Return the live tab palette without consulting application chrome."""
	palette = getattr(tab, "document_display_palette", None)
	if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
		raise RuntimeError("Ferrum path preview requires a document display palette")
	return palette


#============================================
def _paint(palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		value: object) -> PySide6.QtGui.QColor:
	"""Resolve one closed Rust V3 paint through the sole Qt display authority."""
	try:
		return palette.resolve_render_paint(value)
	except ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteError as error:
		raise RuntimeError("Ferrum path preview has an invalid render paint") from error

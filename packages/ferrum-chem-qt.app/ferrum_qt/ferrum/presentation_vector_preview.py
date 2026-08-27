"""Disposable Qt projection of Rust-issued ordinary vector overlays."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.themes.document_display_palette
import ferrum_qt.ferrum.document_display_refresh


#============================================
def create_overlay(tab: object, overlay: object) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Paint only the exact shape and appearance facts issued by Rust."""
	import ferrum_qt.ferrum.engine as engine
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum vector preview requires an installed scene")
	path = PySide6.QtGui.QPainterPath()
	if overlay.kind is engine.PresentationVectorKindV1.line:
		path.moveTo(overlay.start_x, overlay.start_y)
		path.lineTo(overlay.end_x, overlay.end_y)
	else:
		rectangle = PySide6.QtCore.QRectF(
			overlay.left, overlay.top,
			overlay.right - overlay.left, overlay.bottom - overlay.top,
		)
		if overlay.kind in (
			engine.PresentationVectorKindV1.rectangle,
			engine.PresentationVectorKindV1.square,
		):
			path.addRect(rectangle)
		else:
			path.addEllipse(rectangle)
	item = scene.addPath(path)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	refreshable = _VectorDisplayRefreshable(item, overlay)
	refreshable.refresh_document_display_palette(_display_palette(tab))
	ferrum_qt.ferrum.document_display_refresh.register_attached_document_display_refreshable(
		tab, item, refreshable,
	)
	return item


#============================================
class _VectorDisplayRefreshable(
		ferrum_qt.ferrum.document_display_refresh.DocumentDisplayRefreshableV1):
	"""Replace a vector preview's tagged material while retaining its exact path."""

	def __init__(self, item: object, overlay: object) -> None:
		"""Retain the Qt path item and renderer-issued preview DTO."""
		self._item = item
		self._overlay = overlay

	#============================================
	def refresh_document_display_palette(
			self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Replace only tagged vector pens and optional fill brushes."""
		pen = PySide6.QtGui.QPen(_paint(palette, self._overlay.stroke_paint))
		pen.setWidthF(self._overlay.stroke_width)
		pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
		pen.setCosmetic(False)
		brush = PySide6.QtGui.QBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
		if self._overlay.fill_paint is not None:
			brush = PySide6.QtGui.QBrush(_paint(palette, self._overlay.fill_paint))
		self._item.setPen(pen)
		self._item.setBrush(brush)


#============================================
def _display_palette(tab: object) -> ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
	"""Return the palette retained by the tab that owns this preview."""
	palette = getattr(tab, "document_display_palette", None)
	if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
		raise RuntimeError("Ferrum vector preview requires a document display palette")
	return palette


#============================================
def _paint(palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		value: object) -> PySide6.QtGui.QColor:
	"""Resolve one closed Rust V3 paint without a raw-color fallback."""
	try:
		return palette.resolve_render_paint(value)
	except ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteError as error:
		raise RuntimeError("Ferrum vector preview has an invalid render paint") from error

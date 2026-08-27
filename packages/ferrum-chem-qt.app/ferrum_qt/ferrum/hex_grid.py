"""Paper-local hex-grid decoration for ordinary Ferrum document views."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.bridge.display_geometry
from ferrum_qt.canvas.display_palette_refreshable import DisplayPaletteRefreshable
import ferrum_qt.config.geometry_units
import ferrum_qt.themes.document_display_palette


GRID_Z_VALUE = -0.5
_GRID_PHYSICAL_LINE_WIDTH_PX = 1.35
_GRID_VERTEX_DIAMETER_PT = 2.6


#============================================
class FerrumNativeHexGridItem(
		PySide6.QtWidgets.QGraphicsItem, DisplayPaletteRefreshable):
	"""Paint one disposable grid from Rust-issued finite display geometry."""

	#============================================
	def __init__(self, paper_rect: PySide6.QtCore.QRectF,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1) -> None:
		"""Cache one bounded paper-local grid and current application colors."""
		super().__init__()
		self.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		self.setAcceptHoverEvents(False)
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable,
			False,
		)
		self._paper_rect = PySide6.QtCore.QRectF(paper_rect)
		self._line_path = _hex_grid_line_path(self._paper_rect)
		self._dot_path = _hex_grid_dot_path(self._paper_rect)
		self._line_pen = PySide6.QtGui.QPen()
		self._dot_pen = PySide6.QtGui.QPen()
		self._dot_brush = PySide6.QtGui.QBrush()
		self.refresh_display_palette(palette)
		self.setZValue(GRID_Z_VALUE)

	#============================================
	def boundingRect(self) -> PySide6.QtCore.QRectF:
		"""Return the exact Rust-owned paper rectangle decorated by this item."""
		return PySide6.QtCore.QRectF(self._paper_rect)

	#============================================
	def paint(self, painter: PySide6.QtGui.QPainter,
			option: PySide6.QtWidgets.QStyleOptionGraphicsItem,
			widget: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Paint cached honeycomb lanes and vertices without child-item state."""
		del option, widget
		painter.save()
		painter.setClipRect(self._paper_rect)
		painter.setPen(self._line_pen)
		painter.setBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
		painter.drawPath(self._line_path)
		painter.setPen(self._dot_pen)
		painter.setBrush(self._dot_brush)
		painter.drawPath(self._dot_path)
		painter.restore()

	#============================================
	def refresh_display_palette(self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1) -> None:
		"""Refresh cached grid pens and brushes from the current display palette."""
		if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
			raise TypeError("Ferrum grid requires a document display palette")
		line_color = palette.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.GRID_LINE,
		)
		line_pen = PySide6.QtGui.QPen(line_color)
		# A cosmetic pen keeps its physical-pixel footprint at ordinary canvas
		# zoom levels. Scene-unit hairlines disappear after Qt antialiasing below
		# 100 percent, even when their source color has adequate contrast.
		line_pen.setWidthF(_GRID_PHYSICAL_LINE_WIDTH_PX)
		line_pen.setCosmetic(True)
		dot_color = palette.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.GRID_DOT_OUTLINE,
		)
		dot_fill_color = palette.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.GRID_DOT_FILL,
		)
		dot_pen = PySide6.QtGui.QPen(dot_color)
		dot_pen.setWidthF(_GRID_PHYSICAL_LINE_WIDTH_PX)
		dot_pen.setCosmetic(True)
		self._line_pen = line_pen
		self._dot_pen = dot_pen
		self._dot_brush = PySide6.QtGui.QBrush(dot_fill_color)
		self.update()


#============================================
def _hex_grid_line_path(paper_rect: PySide6.QtCore.QRectF) -> PySide6.QtGui.QPainterPath:
	"""Build one cached honeycomb path from the existing Rust geometry boundary."""
	path = PySide6.QtGui.QPainterPath()
	edges = ferrum_qt.bridge.display_geometry.hex_grid_edges(
		paper_rect.left(), paper_rect.top(), paper_rect.right(), paper_rect.bottom(),
		ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT,
	)
	for (x1, y1), (x2, y2) in edges:
		path.moveTo(x1, y1)
		path.lineTo(x2, y2)
	return path


#============================================
def _hex_grid_dot_path(paper_rect: PySide6.QtCore.QRectF) -> PySide6.QtGui.QPainterPath:
	"""Build one cached vertex path from the same Rust geometry boundary."""
	path = PySide6.QtGui.QPainterPath()
	points = ferrum_qt.bridge.display_geometry.hex_grid_points(
		paper_rect.left(), paper_rect.top(), paper_rect.right(), paper_rect.bottom(),
		ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT,
	)
	for x, y in points:
		half_diameter = _GRID_VERTEX_DIAMETER_PT / 2.0
		path.addEllipse(
			x - half_diameter, y - half_diameter,
			_GRID_VERTEX_DIAMETER_PT, _GRID_VERTEX_DIAMETER_PT,
		)
	return path

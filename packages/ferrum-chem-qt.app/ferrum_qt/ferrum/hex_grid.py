"""Paper-local hex-grid decoration for ordinary Ferrum document views."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.bridge.display_geometry
import ferrum_qt.config.geometry_units
import ferrum_qt.themes.theme_loader


GRID_Z_VALUE = -0.5
_MINIMUM_GRID_LIGHTNESS_DELTA = 72
_GRID_CONTRAST_ADJUSTMENT_LIMIT = 16
_GRID_PHYSICAL_LINE_WIDTH_PX = 1.35
_GRID_VERTEX_DIAMETER_PT = 2.6


#============================================
class FerrumNativeHexGridItem(PySide6.QtWidgets.QGraphicsItem):
	"""Paint one disposable grid from Rust-issued finite display geometry."""

	#============================================
	def __init__(self, paper_rect: PySide6.QtCore.QRectF) -> None:
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
		self.refresh_application_style()
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
	def refresh_application_style(self) -> None:
		"""Apply a legible transient grid style matching the active paper surface."""
		theme_name = _application_theme_name()
		colors = ferrum_qt.themes.theme_loader.get_grid_colors(theme_name)
		paper_color = PySide6.QtGui.QColor(
			ferrum_qt.themes.theme_loader.get_paper_color(theme_name),
		)
		line_color = _visible_grid_color(
			PySide6.QtGui.QColor(colors["line"]), paper_color,
		)
		line_pen = PySide6.QtGui.QPen(line_color)
		# A cosmetic pen keeps its physical-pixel footprint at ordinary canvas
		# zoom levels. Scene-unit hairlines disappear after Qt antialiasing below
		# 100 percent, even when their source color has adequate contrast.
		line_pen.setWidthF(_GRID_PHYSICAL_LINE_WIDTH_PX)
		line_pen.setCosmetic(True)
		dot_color = _visible_grid_color(
			PySide6.QtGui.QColor(colors["dot_outline"]), paper_color,
		)
		dot_fill_color = _visible_grid_color(
			PySide6.QtGui.QColor(colors["dot_fill"]), paper_color,
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


#============================================
def _application_theme_name() -> str:
	"""Map the active palette to the package's matching grid color family."""
	color = PySide6.QtWidgets.QApplication.palette().color(
		PySide6.QtGui.QPalette.ColorRole.Base,
	)
	return "dark" if color.lightness() < 128 else "light"


#============================================
def _visible_grid_color(
		color: PySide6.QtGui.QColor, paper_color: PySide6.QtGui.QColor,
		) -> PySide6.QtGui.QColor:
	"""Return one paper-legible grid color while retaining passing theme tokens.

	The grid is painted on the themed paper rectangle rather than on Qt's chrome
	palette base.  Repeated small adjustments preserve each token's hue and
	saturation when possible; the explicit HSL endpoint guarantees the published
	minimum lightness separation when a repeated adjustment reaches an endpoint.
	"""
	visible_color = PySide6.QtGui.QColor(color)
	for _unused_index in range(_GRID_CONTRAST_ADJUSTMENT_LIMIT):
		if _grid_lightness_delta(visible_color, paper_color) >= _MINIMUM_GRID_LIGHTNESS_DELTA:
			return visible_color
		if paper_color.lightness() >= 128:
			visible_color = visible_color.darker(110)
		else:
			visible_color = visible_color.lighter(110)
	if paper_color.lightness() >= 128:
		target_lightness = paper_color.lightness() - _MINIMUM_GRID_LIGHTNESS_DELTA
	else:
		target_lightness = paper_color.lightness() + _MINIMUM_GRID_LIGHTNESS_DELTA
	visible_color.setHsl(
		visible_color.hslHue(),
		visible_color.hslSaturation(),
		target_lightness,
		visible_color.alpha(),
	)
	if _grid_lightness_delta(visible_color, paper_color) < _MINIMUM_GRID_LIGHTNESS_DELTA:
		raise RuntimeError("Ferrum grid contrast endpoint did not meet its contract")
	return visible_color


#============================================
def _grid_lightness_delta(
		color: PySide6.QtGui.QColor, paper_color: PySide6.QtGui.QColor,
		) -> int:
	"""Return the deterministic lightness separation used by the grid contract."""
	delta = abs(color.lightness() - paper_color.lightness())
	return delta

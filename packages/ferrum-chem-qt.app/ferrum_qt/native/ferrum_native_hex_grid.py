"""Paper-local hex-grid decoration for ordinary Rust-native document views."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.bridge.display_geometry
import ferrum_qt.config.geometry_units
import ferrum_qt.themes.theme_loader


GRID_Z_VALUE = -0.5


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
		"""Apply grid colors matching the current application palette family."""
		colors = ferrum_qt.themes.theme_loader.get_grid_colors(_application_theme_name())
		line_pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor(colors["line"]))
		line_pen.setWidthF(0.375)
		dot_pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor(colors["dot_outline"]))
		dot_pen.setWidthF(0.375)
		self._line_pen = line_pen
		self._dot_pen = dot_pen
		self._dot_brush = PySide6.QtGui.QBrush(
			PySide6.QtGui.QColor(colors["dot_fill"]),
		)
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
		path.addEllipse(x - 1.0, y - 1.0, 2.0, 2.0)
	return path


#============================================
def _application_theme_name() -> str:
	"""Map the active palette to the package's matching grid color family."""
	color = PySide6.QtWidgets.QApplication.palette().color(
		PySide6.QtGui.QPalette.ColorRole.Base,
	)
	return "dark" if color.lightness() < 128 else "light"

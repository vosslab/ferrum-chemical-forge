"""Disposable Qt projection of a Rust-issued straight Arrow overlay."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
def create_straight_normal_arrow_overlay(
		tab: object, overlay: object,
		) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Paint only Rust-resolved shaft, head vertices, width, and color."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum Arrow preview requires an installed scene")
	path = PySide6.QtGui.QPainterPath()
	path.moveTo(overlay.axis_start_x, overlay.axis_start_y)
	path.lineTo(overlay.axis_end_x, overlay.axis_end_y)
	vertices = overlay.head_vertices
	if len(vertices) == 3:
		path.moveTo(vertices[0][0], vertices[0][1])
		path.lineTo(vertices[1][0], vertices[1][1])
		path.lineTo(vertices[2][0], vertices[2][1])
		path.closeSubpath()
	pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor(overlay.color))
	pen.setWidthF(overlay.width)
	pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
	item = scene.addPath(path, pen)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	return item


#============================================
def create_plus_overlay(tab: object, overlay: object) -> PySide6.QtWidgets.QGraphicsSimpleTextItem:
	"""Paint only the renderer-issued Plus text and explicit paint facts."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum Plus preview requires an installed scene")
	item = scene.addSimpleText(overlay.text)
	font = item.font()
	font.setPointSizeF(overlay.font_size)
	item.setFont(font)
	item.setBrush(PySide6.QtGui.QBrush(PySide6.QtGui.QColor(_qt_color(overlay.color))))
	item.setPos(overlay.origin_x, overlay.origin_y)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	return item


#============================================
def _qt_color(value: str) -> str:
	"""Adapt the renderer's six-digit RGB wire value only at the Qt boundary."""
	if len(value) == 6 and all(character in "0123456789abcdefABCDEF" for character in value):
		return f"#{value}"
	return value

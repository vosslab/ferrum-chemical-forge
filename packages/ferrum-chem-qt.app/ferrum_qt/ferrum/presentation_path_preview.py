"""Qt projection of a closed Rust-issued path preview."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui


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
	pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor(overlay.stroke_color))
	pen.setWidthF(overlay.stroke_width)
	brush = PySide6.QtGui.QBrush()
	if overlay.fill_color is not None:
		brush = PySide6.QtGui.QBrush(PySide6.QtGui.QColor(overlay.fill_color))
	item = tab.view.scene().addPath(path, pen, brush)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	return item

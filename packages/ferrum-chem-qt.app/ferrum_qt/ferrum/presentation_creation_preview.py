"""Disposable Qt projection of Rust-issued straight presentation-arrow overlays."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.engine


#============================================
def create_straight_presentation_arrow_overlay(
		tab: object, overlay: object,
		) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Paint only one closed Rust-issued normal or equilibrium arrow overlay."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum Arrow preview requires an installed scene")
	path = PySide6.QtGui.QPainterPath()
	engine = ferrum_qt.ferrum.engine
	if type(overlay) is engine.NormalArrowGestureOverlayV1:
		_append_issued_axis(path, overlay.axis)
		for head in overlay.heads:
			_append_issued_polygon(path, head)
	elif type(overlay) is engine.EquilibriumArrowGestureOverlayV1:
		_append_issued_axis(path, overlay.lower_axis)
		_append_issued_axis(path, overlay.upper_axis)
		_append_issued_polygon(path, overlay.source_head)
		_append_issued_polygon(path, overlay.destination_head)
	else:
		raise TypeError("Ferrum Arrow preview requires a closed Rust-issued arrow overlay")
	pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor(overlay.color))
	pen.setWidthF(overlay.width)
	pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
	item = scene.addPath(path, pen)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	return item


#============================================
def create_curved_electron_arrow_overlay(
		tab: object, overlay: object,
		) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Paint only the cubic and head points issued by Rust for one electron arrow."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum electron-arrow preview requires an installed scene")
	path = PySide6.QtGui.QPainterPath()
	path.moveTo(overlay.start_x, overlay.start_y)
	path.cubicTo(
		overlay.cubic_control_1_x, overlay.cubic_control_1_y,
		overlay.cubic_control_2_x, overlay.cubic_control_2_y,
		overlay.end_x, overlay.end_y,
	)
	_append_issued_point_sequence(path, overlay.head)
	pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor("#49719c"))
	pen.setWidthF(1.5)
	pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
	item = scene.addPath(path, pen)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	return item


#============================================
def _append_issued_axis(path: PySide6.QtGui.QPainterPath, axis: object) -> None:
	"""Append one Rust-issued axis without deriving endpoints in Qt."""
	path.moveTo(axis.start_x, axis.start_y)
	path.lineTo(axis.end_x, axis.end_y)


#============================================
def _append_issued_polygon(path: PySide6.QtGui.QPainterPath, polygon: object) -> None:
	"""Append one Rust-issued ordered head polygon without changing its geometry."""
	_append_issued_point_sequence(path, polygon.vertices)


#============================================
def _append_issued_point_sequence(path: PySide6.QtGui.QPainterPath, points: object) -> None:
	"""Append one ordered Rust-issued point sequence without calculating vertices."""
	vertices = points
	path.moveTo(vertices[0][0], vertices[0][1])
	for x, y in vertices[1:]:
		path.lineTo(x, y)
	path.closeSubpath()


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

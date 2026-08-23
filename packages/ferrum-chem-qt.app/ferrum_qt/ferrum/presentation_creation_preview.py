"""Disposable Qt projection of Rust-issued straight and curved arrow overlays."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.terminal_arrow


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
def create_terminal_arrow_overlay(
		tab: object, kind: ferrum_qt.ferrum.terminal_arrow.TerminalArrowKind, overlay: object,
		) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Paint only the cubic and head points issued by Rust for one terminal arrow."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum terminal-arrow preview requires an installed scene")
	engine = ferrum_qt.ferrum.engine
	if kind is ferrum_qt.ferrum.terminal_arrow.TerminalArrowKind.ELECTRON:
		expected_type = engine.CurvedElectronArrowOverlayV1
	elif kind is ferrum_qt.ferrum.terminal_arrow.TerminalArrowKind.RETRO:
		expected_type = engine.CurvedRetroArrowOverlayV1
	else:
		expected_type = engine.CurvedNormalReactionArrowOverlayV1
	if type(overlay) is not expected_type:
		raise TypeError(f"Ferrum {kind.description} preview requires its exact Rust projection")
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
def create_curved_equilibrium_arrow_overlay(
		tab: object, overlay: object,
		) -> PySide6.QtWidgets.QGraphicsItemGroup:
	"""Paint the two cubic lanes and heads issued by Rust without deriving geometry."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum curved-equilibrium preview requires an installed scene")
	engine = ferrum_qt.ferrum.engine
	if type(overlay) is not engine.CurvedEquilibriumArrowOverlayV1:
		raise TypeError("Ferrum curved-equilibrium preview requires its exact Rust projection")
	axis_path = PySide6.QtGui.QPainterPath()
	_append_issued_cubic_axis(axis_path, overlay.lower_axis)
	_append_issued_cubic_axis(axis_path, overlay.upper_axis)
	head_path = PySide6.QtGui.QPainterPath()
	_append_issued_point_sequence(head_path, overlay.lower_head)
	_append_issued_point_sequence(head_path, overlay.upper_head)
	pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor("#49719c"))
	pen.setWidthF(1.5)
	pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
	group = PySide6.QtWidgets.QGraphicsItemGroup()
	scene.addItem(group)
	axis_item = PySide6.QtWidgets.QGraphicsPathItem(axis_path, group)
	axis_item.setPen(pen)
	axis_item.setBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
	head_item = PySide6.QtWidgets.QGraphicsPathItem(head_path, group)
	head_item.setPen(PySide6.QtCore.Qt.PenStyle.NoPen)
	head_item.setBrush(PySide6.QtGui.QBrush(pen.color()))
	for item in (axis_item, head_item, group):
		item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	group.setZValue(1_000_000.0)
	return group


#============================================
def _append_issued_axis(path: PySide6.QtGui.QPainterPath, axis: object) -> None:
	"""Append one Rust-issued axis without deriving endpoints in Qt."""
	path.moveTo(axis.start_x, axis.start_y)
	path.lineTo(axis.end_x, axis.end_y)


#============================================
def _append_issued_cubic_axis(path: PySide6.QtGui.QPainterPath, points: object) -> None:
	"""Append one exact four-point cubic axis issued by Rust."""
	if type(points) is not list or len(points) != 4:
		raise TypeError("Ferrum curved-equilibrium preview requires one four-point cubic axis")
	path.moveTo(points[0][0], points[0][1])
	path.cubicTo(
		points[1][0], points[1][1], points[2][0], points[2][1], points[3][0], points[3][1],
	)


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

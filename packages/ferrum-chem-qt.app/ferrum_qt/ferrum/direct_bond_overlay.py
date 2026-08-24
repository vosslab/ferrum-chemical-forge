"""Qt replay of the identifier-free Rust direct-bond preview contract."""

# Standard Library
import math
import re

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_telex
import ferrum_qt.canvas.telex_glyph_outline


_RGB24 = re.compile(r"^[0-9a-f]{6}$")
_LAYERS = {"ordinary", "haworth_front_stroke", "haworth_front_wedge"}
_FACE = "ferrum-telex-regular-v1"


#============================================
class DirectBondOverlayError(ValueError):
	"""Raised when an admitted direct-bond primitive cannot be replayed exactly."""


#============================================
def create_overlay(tab: object, overlay: object) -> PySide6.QtWidgets.QGraphicsItemGroup:
	"""Replay one generic prepared-transition paint overlay without document identity."""
	import ferrum_qt.ferrum.engine as engine
	if type(overlay) is not engine.DocumentPrecommitOverlayV1:
		raise DirectBondOverlayError("direct-bond preview requires DocumentPrecommitOverlayV1")
	if type(overlay.primitives) is not tuple or not overlay.primitives:
		raise DirectBondOverlayError("direct-bond preview requires ordered paint primitives")
	scene = tab.view.scene()
	if scene is None:
		raise DirectBondOverlayError("direct-bond preview requires an installed scene")
	telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(
		tab._controller._telex_resource,
	)
	group = PySide6.QtWidgets.QGraphicsItemGroup()
	try:
		for index, primitive in enumerate(overlay.primitives):
			item = _primitive_item(primitive, engine, telex)
			item.setZValue(float(index))
			item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
			group.addToGroup(item)
	except Exception:
		group.setParentItem(None)
		raise
	scene.addItem(group)
	group.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	group.setZValue(1_000_000.0)
	return group


#============================================
def _primitive_item(primitive: object, engine: object,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Build one noninteractive Qt item from one closed renderer paint primitive."""
	if type(primitive) is not engine.DocumentPrecommitPaintPrimitiveV1:
		raise DirectBondOverlayError("direct-bond preview has an unknown primitive")
	space = primitive.coordinate_space
	anchor = _anchor(space, engine)
	if primitive.display_layer not in _LAYERS:
		raise DirectBondOverlayError("direct-bond preview has an unknown display layer")
	operation = primitive.operation
	if type(operation) is not engine.RenderOperationV2:
		raise DirectBondOverlayError("direct-bond preview has an unknown render operation")
	if space.kind == "scene" and operation.kind == "line":
		path, pen, brush = _line_operation(operation.operation, engine)
	elif space.kind == "scene" and operation.kind == "path":
		path, pen, brush = _path_operation(operation.operation, engine)
	elif space.kind == "atom_local" and operation.kind == "ellipse":
		path, pen, brush = _ellipse_operation(operation.operation, engine)
	elif space.kind == "atom_local" and operation.kind == "mask":
		path, pen, brush = _mask_operation(operation.operation, engine)
	elif space.kind == "atom_local" and operation.kind == "text":
		path, pen, brush = _text_operation(operation.operation, engine, telex)
	else:
		raise DirectBondOverlayError("direct-bond preview operation does not match its coordinate space")
	item = PySide6.QtWidgets.QGraphicsPathItem(path)
	item.setPen(pen)
	item.setBrush(brush)
	item.setPos(anchor)
	return item


#============================================
def _line_operation(value: object, engine: object,
		) -> tuple[PySide6.QtGui.QPainterPath, PySide6.QtGui.QPen, PySide6.QtGui.QBrush]:
	"""Replay one Rust-issued scene line with its complete stroke appearance."""
	if type(value) is not engine.LineOpV1:
		raise DirectBondOverlayError("direct-bond line operation has the wrong DTO type")
	path = PySide6.QtGui.QPainterPath()
	path.moveTo(_point(value.start, engine, "line start"))
	path.lineTo(_point(value.end, engine, "line end"))
	pen = PySide6.QtGui.QPen(_color(value.paint, "line paint"))
	pen.setWidthF(_positive(value.width, "line width"))
	pen.setCosmetic(False)
	return path, pen, PySide6.QtGui.QBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)


#============================================
def _path_operation(value: object, engine: object,
		) -> tuple[PySide6.QtGui.QPainterPath, PySide6.QtGui.QPen, PySide6.QtGui.QBrush]:
	"""Replay one Rust-issued scene path with no local geometric synthesis."""
	if type(value) is not engine.PathOpV2:
		raise DirectBondOverlayError("direct-bond path operation has the wrong DTO type")
	path = _path(value.commands, engine)
	pen = PySide6.QtGui.QPen(PySide6.QtCore.Qt.PenStyle.NoPen)
	if value.stroke_width is not None or value.stroke_paint is not None or value.stroke_line_cap is not None:
		if value.stroke_width is None or value.stroke_paint is None or value.stroke_line_cap is None:
			raise DirectBondOverlayError("direct-bond path has incomplete stroke data")
		pen = PySide6.QtGui.QPen(_color(value.stroke_paint, "path stroke paint"))
		pen.setWidthF(_positive(value.stroke_width, "path stroke width"))
		pen.setCapStyle(_cap_style(value.stroke_line_cap))
		pen.setCosmetic(False)
	brush = PySide6.QtGui.QBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
	if value.fill_paint is not None:
		brush = PySide6.QtGui.QBrush(_color(value.fill_paint, "path fill paint"))
	return path, pen, brush


#============================================
def _ellipse_operation(value: object, engine: object,
		) -> tuple[PySide6.QtGui.QPainterPath, PySide6.QtGui.QPen, PySide6.QtGui.QBrush]:
	"""Replay one atom-local ellipse at the immutable renderer-selected anchor."""
	if type(value) is not engine.EllipseOpV1:
		raise DirectBondOverlayError("direct-bond ellipse operation has the wrong DTO type")
	center = _point(value.center, engine, "ellipse center")
	radius_x = _positive(value.radius_x, "ellipse horizontal radius")
	radius_y = _positive(value.radius_y, "ellipse vertical radius")
	path = PySide6.QtGui.QPainterPath()
	path.addEllipse(center.x() - radius_x, center.y() - radius_y, radius_x * 2.0, radius_y * 2.0)
	rotation = _finite(value.rotation_degrees, "ellipse rotation")
	if rotation != 0.0:
		transform = PySide6.QtGui.QTransform()
		transform.translate(center.x(), center.y())
		transform.rotate(rotation)
		transform.translate(-center.x(), -center.y())
		path = transform.map(path)
	pen = PySide6.QtGui.QPen(PySide6.QtCore.Qt.PenStyle.NoPen)
	if value.stroke_width is not None or value.stroke_paint is not None:
		if value.stroke_width is None or value.stroke_paint is None:
			raise DirectBondOverlayError("direct-bond ellipse has incomplete stroke data")
		pen = PySide6.QtGui.QPen(_color(value.stroke_paint, "ellipse stroke paint"))
		pen.setWidthF(_positive(value.stroke_width, "ellipse stroke width"))
		pen.setCosmetic(False)
	brush = PySide6.QtGui.QBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
	if value.fill_paint is not None:
		brush = PySide6.QtGui.QBrush(_color(value.fill_paint, "ellipse fill paint"))
	return path, pen, brush


#============================================
def _mask_operation(value: object, engine: object,
		) -> tuple[PySide6.QtGui.QPainterPath, PySide6.QtGui.QPen, PySide6.QtGui.QBrush]:
	"""Replay one atom-local renderer mask without inferring a replacement shape."""
	if type(value) is not engine.MaskOpV1:
		raise DirectBondOverlayError("direct-bond mask operation has the wrong DTO type")
	origin = _point(value.origin, engine, "mask origin")
	width = _positive(value.width, "mask width")
	height = _positive(value.height, "mask height")
	path = PySide6.QtGui.QPainterPath()
	path.addRect(origin.x(), origin.y(), width, height)
	return (
		path,
		PySide6.QtGui.QPen(PySide6.QtCore.Qt.PenStyle.NoPen),
		PySide6.QtGui.QBrush(_color(value.paint, "mask paint")),
	)


#============================================
def _text_operation(value: object, engine: object,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		) -> tuple[PySide6.QtGui.QPainterPath, PySide6.QtGui.QPen, PySide6.QtGui.QBrush]:
	"""Replay prepositioned Telex glyph outlines without frontend shaping."""
	if type(value) is not engine.TextOpV1:
		raise DirectBondOverlayError("direct-bond text operation has the wrong DTO type")
	if value.face != _FACE:
		raise DirectBondOverlayError("direct-bond text has an unknown font face")
	origin = _point(value.origin, engine, "text origin")
	try:
		font = telex.raw_font(_positive(value.size, "text size"))
		path = ferrum_qt.canvas.telex_glyph_outline.path_from_runs(value.runs, origin, font)
	except (
		ferrum_qt.canvas.ferrum_telex.FerrumTelexError,
		ferrum_qt.canvas.telex_glyph_outline.TelexGlyphOutlineError,
	) as exc:
		raise DirectBondOverlayError(str(exc)) from exc
	return (
		path,
		PySide6.QtGui.QPen(PySide6.QtCore.Qt.PenStyle.NoPen),
		PySide6.QtGui.QBrush(_color(value.paint, "text paint")),
	)


#============================================
def _path(commands: object, engine: object) -> PySide6.QtGui.QPainterPath:
	"""Copy closed path commands directly into Qt's retained painter path."""
	if type(commands) is not tuple or not commands:
		raise DirectBondOverlayError("direct-bond path commands must be a nonempty tuple")
	path = PySide6.QtGui.QPainterPath()
	for command in commands:
		if type(command) is not engine.ScenePathCommandV2:
			raise DirectBondOverlayError("direct-bond path command has the wrong DTO type")
		if command.kind == "move_to":
			path.moveTo(_point(command.point, engine, "path move point"))
			_require_no_controls(command)
		elif command.kind == "line_to":
			path.lineTo(_point(command.point, engine, "path line point"))
			_require_no_controls(command)
		elif command.kind == "cubic_to":
			path.cubicTo(
				_point(command.control_1, engine, "first cubic control"),
				_point(command.control_2, engine, "second cubic control"),
				_point(command.point, engine, "cubic end point"),
			)
		elif command.kind == "close":
			if command.point is not None or command.control_1 is not None or command.control_2 is not None:
				raise DirectBondOverlayError("direct-bond close command has coordinates")
			path.closeSubpath()
		else:
			raise DirectBondOverlayError("direct-bond path has an unknown command")
	return path


#============================================
def _require_no_controls(command: object) -> None:
	"""Require non-cubic commands to have no unused control points."""
	if command.point is None or command.control_1 is not None or command.control_2 is not None:
		raise DirectBondOverlayError("direct-bond non-cubic path command is malformed")


#============================================
def _anchor(value: object, engine: object) -> PySide6.QtCore.QPointF:
	"""Return the one renderer-issued coordinate-space anchor for a primitive."""
	if type(value) is not engine.DocumentPrecommitOverlayCoordinateSpaceV1:
		raise DirectBondOverlayError("direct-bond preview has an unknown coordinate space")
	if value.kind == "scene":
		if value.anchor is not None:
			raise DirectBondOverlayError("scene direct-bond primitive has an anchor")
		return PySide6.QtCore.QPointF(0.0, 0.0)
	if value.kind == "atom_local":
		return _point(value.anchor, engine, "atom-local anchor")
	raise DirectBondOverlayError("direct-bond preview has an unknown coordinate space")


#============================================
def _point(value: object, engine: object, label: str) -> PySide6.QtCore.QPointF:
	"""Return one finite compiled renderer point."""
	if type(value) is not engine.RenderPointV1:
		raise DirectBondOverlayError(f"{label} has the wrong DTO type")
	return PySide6.QtCore.QPointF(_finite(value.x, f"{label} x"), _finite(value.y, f"{label} y"))


#============================================
def _cap_style(value: object) -> PySide6.QtCore.Qt.PenCapStyle:
	"""Map the renderer's closed line-cap vocabulary without a Qt default."""
	if value == "butt":
		return PySide6.QtCore.Qt.PenCapStyle.FlatCap
	if value == "round":
		return PySide6.QtCore.Qt.PenCapStyle.RoundCap
	if value == "square":
		return PySide6.QtCore.Qt.PenCapStyle.SquareCap
	raise DirectBondOverlayError("direct-bond path has an unknown line cap")


#============================================
def _finite(value: object, label: str) -> float:
	"""Return one finite compiled scalar."""
	if type(value) is not float or not math.isfinite(value):
		raise DirectBondOverlayError(f"{label} must be finite")
	return value


#============================================
def _positive(value: object, label: str) -> float:
	"""Return one positive finite compiled scalar."""
	value = _finite(value, label)
	if value <= 0.0:
		raise DirectBondOverlayError(f"{label} must be positive")
	return value


#============================================
def _color(value: object, label: str) -> PySide6.QtGui.QColor:
	"""Return one exact lowercase renderer RGB paint."""
	if type(value) is not str or _RGB24.fullmatch(value) is None:
		raise DirectBondOverlayError(f"{label} must be lowercase RGB24")
	return PySide6.QtGui.QColor(f"#{value}")

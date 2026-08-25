"""Replay one frozen renderer-owned presentation plan into Qt scene items."""

# Standard Library
import dataclasses
import math
import re

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_telex
import ferrum_qt.canvas.ferrum_presentation_target
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.canvas.items.ferrum_plus_item
import ferrum_qt.canvas.items.ferrum_text_item
from ferrum_qt.canvas.ferrum_render_target import RenderTargetKey


_SCHEMA = "ferrum-presentation-render-plan-v1"
_DIGEST = re.compile(r"^[0-9a-f]{64}$")
_RGB24 = re.compile(r"^[0-9a-f]{6}$")


#============================================
class PresentationRenderPlanError(ValueError):
	"""Raised when one renderer-issued plan cannot be replayed safely."""


@dataclasses.dataclass(slots=True)
class FerrumPresentationScene:
	"""Detached Qt replay of one immutable renderer-owned presentation plan."""

	revision: int
	digest: str
	roots: tuple[PySide6.QtWidgets.QGraphicsItem, ...]
	items: tuple[PySide6.QtWidgets.QGraphicsItem, ...]
	durable_items: dict[tuple[str, str], PySide6.QtWidgets.QGraphicsItem]
	local_items: dict[RenderTargetKey, PySide6.QtWidgets.QGraphicsItem]

	#============================================
	def selected_targets(
			self, scene: PySide6.QtWidgets.QGraphicsScene | None,
			) -> tuple[RenderTargetKey, ...]:
		"""Return selected plan targets without promoting local keys to IDs."""
		selected = ferrum_qt.canvas.graphics_retirement.selected_items_from_captured_scene(scene)
		return tuple(item.target for item in self.items if item in selected and item.isSelected())

	#============================================
	def select_durable(self, targets: tuple[tuple[str, str], ...]) -> None:
		"""Restore selection using only durable target identity from the plan."""
		requested = frozenset(targets)
		for item in self.items:
			target = item.target
			item.setSelected(
				target.is_durable and target.durable_selection_key() in requested,
			)

	#============================================
	def dispose_detached(self) -> None:
		"""Release a never-installed renderer-plan scene through the shared reaper."""
		coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
		coordinator.retire_detached_projection_items(list(self.roots))
		coordinator.raise_if_callback_failed("Ferrum presentation-scene disposal failed")


#============================================
class RendererPlanRootItem(PySide6.QtWidgets.QGraphicsItem):
	"""One selectable root that paints only renderer-issued vector operations."""

	#============================================
	def __init__(self, commands: tuple[tuple[PySide6.QtGui.QPainterPath,
			PySide6.QtGui.QPen | None, PySide6.QtGui.QBrush | None], ...],
			target: RenderTargetKey, bounds: PySide6.QtCore.QRectF) -> None:
		"""Cache validated paths and explicit paints without deriving geometry."""
		super().__init__()
		self._commands = commands
		self._target = target
		self._bounds = bounds
		self._shape = PySide6.QtGui.QPainterPath()
		for path, pen, _brush in commands:
			self._shape.addPath(path)
			if pen is not None:
				stroker = PySide6.QtGui.QPainterPathStroker()
				stroker.setWidth(max(6.0, pen.widthF() + 4.0))
				self._shape.addPath(stroker.createStroke(path))
		self.setZValue(float(target.source_order))
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, True,
		)
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable, False,
		)

	#============================================
	@property
	def target(self) -> RenderTargetKey:
		"""Return the immutable target identity supplied by the renderer."""
		return self._target

	#============================================
	def boundingRect(self) -> PySide6.QtCore.QRectF:
		"""Return renderer-issued finite bounds for Qt scene indexing."""
		return PySide6.QtCore.QRectF(self._bounds)

	#============================================
	def shape(self) -> PySide6.QtGui.QPainterPath:
		"""Return cached paint geometry expanded only for Qt interaction."""
		return PySide6.QtGui.QPainterPath(self._shape)

	#============================================
	def paint(self, painter: PySide6.QtGui.QPainter,
			option: PySide6.QtWidgets.QStyleOptionGraphicsItem,
			widget: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Replay exact paths, strokes, and fills in renderer operation order."""
		del option, widget
		for path, pen, brush in self._commands:
			painter.setPen(PySide6.QtCore.Qt.PenStyle.NoPen if pen is None else pen)
			painter.setBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush if brush is None else brush)
			painter.drawPath(path)

	#============================================
	def dispose(self) -> None:
		"""Provide the established projection-retirement callback contract."""


#============================================
def build_presentation_render_plan(plan: object, telex_resource: object) -> FerrumPresentationScene:
	"""Build detached Qt roots from one exact fenced renderer plan."""
	extension = _ferrum_chem()
	if type(plan) is not extension.PresentationRenderPlanV1:
		raise PresentationRenderPlanError(
			"presentation render plan must be engine.PresentationRenderPlanV1",
		)
	if plan.schema != _SCHEMA:
		raise PresentationRenderPlanError("unknown presentation render-plan schema")
	revision = _revision(plan.revision)
	digest = _digest(plan.digest)
	if type(plan.roots) is not tuple:
		raise PresentationRenderPlanError("presentation render-plan roots must be frozen")
	telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex_resource)
	roots: list[object] = []
	durable_items: dict[tuple[str, str], object] = {}
	local_items: dict[RenderTargetKey, object] = {}
	last_order = -1
	try:
		for root in plan.roots:
			target = _target(root.target, extension)
			bounds = _bounds(root.bounds, extension)
			if target.source_order <= last_order:
				raise PresentationRenderPlanError("presentation render-plan roots are not source ordered")
			last_order = target.source_order
			item = _root_item(root, target, bounds, extension, telex)
			if target.durable_object_id is None:
				if target in local_items:
					raise PresentationRenderPlanError("duplicate local presentation target")
				local_items[target] = item
			else:
				durable_key = target.durable_selection_key()
				if durable_key in durable_items:
					raise PresentationRenderPlanError("duplicate durable presentation target")
				durable_items[durable_key] = item
			roots.append(item)
	except (AttributeError, TypeError, ValueError, PresentationRenderPlanError) as exc:
		for item in roots:
			item.dispose()
		if isinstance(exc, PresentationRenderPlanError):
			raise
		raise PresentationRenderPlanError("invalid frozen presentation render plan") from exc
	return FerrumPresentationScene(
		revision, digest, tuple(roots), tuple(roots), durable_items, local_items,
	)


#============================================
def _root_item(root: object, target: RenderTargetKey, bounds: PySide6.QtCore.QRectF,
		extension: object, telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex) -> object:
	"""Validate one discriminated root and build only its documented variant."""
	kind = getattr(root, "kind", None)
	if kind == "vector":
		if root.plus is not None or root.text is not None or type(root.vector_operations) is not tuple:
			raise PresentationRenderPlanError("vector render root has mixed variants")
		commands = tuple(_vector_operation(operation, extension) for operation in root.vector_operations)
		if not commands:
			raise PresentationRenderPlanError("vector render root has no operations")
		return RendererPlanRootItem(commands, target, bounds)
	if kind == "plus":
		if root.plus is None or root.text is not None or root.vector_operations != ():
			raise PresentationRenderPlanError("plus render root has mixed variants")
		item = ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem._from_observation(
			root.plus, telex,
		)
		return _require_matching_item_target(item, target)
	if kind == "text":
		if root.text is None or root.plus is not None or root.vector_operations != ():
			raise PresentationRenderPlanError("text render root has mixed variants")
		item = ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem._from_observation(
			root.text, telex,
		)
		return _require_matching_item_target(item, target)
	raise PresentationRenderPlanError("unknown presentation render-root kind")


#============================================
def _require_matching_item_target(item: object, target: RenderTargetKey) -> object:
	"""Ensure a specialized renderer root retained the plan root's exact identity."""
	if item.target != target:
		raise PresentationRenderPlanError("specialized render root target differs from plan target")
	return item


#============================================
def _vector_operation(operation: object, extension: object,
		) -> tuple[PySide6.QtGui.QPainterPath, PySide6.QtGui.QPen | None,
		PySide6.QtGui.QBrush | None]:
	"""Replay one closed renderer vector operation into an explicit Qt command."""
	if type(operation) is not extension.PresentationVectorOperationV1:
		raise PresentationRenderPlanError("render vector operation has the wrong DTO type")
	if operation.kind == "path":
		path = _path(operation.commands, extension)
		if operation.center is not None or operation.radius_x is not None or operation.radius_y is not None:
			raise PresentationRenderPlanError("path render operation has ellipse fields")
	elif operation.kind == "ellipse":
		if operation.commands != ():
			raise PresentationRenderPlanError("ellipse render operation has path commands")
		center = _point(operation.center, extension, "ellipse center")
		radius_x = _positive(operation.radius_x, "ellipse horizontal radius")
		radius_y = _positive(operation.radius_y, "ellipse vertical radius")
		path = PySide6.QtGui.QPainterPath()
		path.addEllipse(
			center.x() - radius_x, center.y() - radius_y, radius_x * 2.0, radius_y * 2.0,
		)
	else:
		raise PresentationRenderPlanError("unknown renderer vector operation")
	return path, _pen(operation.stroke, extension), _brush(operation.fill)


#============================================
def _path(commands: object, extension: object) -> PySide6.QtGui.QPainterPath:
	"""Copy a complete renderer path without calculating an alternative path."""
	if type(commands) is not tuple or not commands:
		raise PresentationRenderPlanError("renderer path commands must be a nonempty frozen sequence")
	path = PySide6.QtGui.QPainterPath()
	for command in commands:
		if command.kind == "move_to":
			point = _point(command.point, extension, "path move point")
			if command.control_1 is not None or command.control_2 is not None:
				raise PresentationRenderPlanError("path move command has controls")
			path.moveTo(point)
		elif command.kind == "line_to":
			point = _point(command.point, extension, "path line point")
			if command.control_1 is not None or command.control_2 is not None:
				raise PresentationRenderPlanError("path line command has controls")
			path.lineTo(point)
		elif command.kind == "cubic_to":
			point = _point(command.point, extension, "path cubic end point")
			control_1 = _point(command.control_1, extension, "path first cubic control")
			control_2 = _point(command.control_2, extension, "path second cubic control")
			path.cubicTo(control_1, control_2, point)
		elif command.kind == "close":
			if command.point is not None or command.control_1 is not None or command.control_2 is not None:
				raise PresentationRenderPlanError("path close command has coordinates")
			path.closeSubpath()
		else:
			raise PresentationRenderPlanError("unknown renderer path command")
	return path


#============================================
def _pen(value: object, extension: object) -> PySide6.QtGui.QPen | None:
	"""Copy one complete renderer-issued stroke without Qt defaults."""
	if value is None:
		return None
	if type(value) is not extension.PresentationRenderStrokeV1:
		raise PresentationRenderPlanError("renderer stroke has the wrong DTO type")
	if value.line_cap != "butt" or value.line_join != "miter":
		raise PresentationRenderPlanError("renderer stroke has unsupported line semantics")
	pen = PySide6.QtGui.QPen(_color(value.paint, "renderer stroke paint"))
	pen.setWidthF(_positive(value.width, "renderer stroke width"))
	pen.setCapStyle(PySide6.QtCore.Qt.PenCapStyle.FlatCap)
	pen.setJoinStyle(PySide6.QtCore.Qt.PenJoinStyle.MiterJoin)
	pen.setMiterLimit(_positive(value.miter_limit, "renderer stroke miter limit"))
	return pen


#============================================
def _brush(value: object) -> PySide6.QtGui.QBrush | None:
	"""Copy an optional renderer-issued fill paint."""
	return None if value is None else PySide6.QtGui.QBrush(_color(value, "renderer fill paint"))


#============================================
def _target(value: object, extension: object) -> RenderTargetKey:
	"""Authenticate one exact plan target as the Qt selection identity."""
	try:
		return ferrum_qt.canvas.ferrum_presentation_target.presentation_target_from_dto(
			value, extension,
		)
	except ferrum_qt.canvas.ferrum_presentation_target.PresentationTargetError as exc:
		raise PresentationRenderPlanError(str(exc)) from exc


#============================================
def _bounds(value: object, extension: object) -> PySide6.QtCore.QRectF:
	"""Validate finite positive renderer bounds without recalculating them."""
	if type(value) is not extension.PresentationRenderBoundsV1:
		raise PresentationRenderPlanError("presentation render bounds have the wrong DTO type")
	left, top, right, bottom = (_finite(value.left, "render bound left"),
		_finite(value.top, "render bound top"), _finite(value.right, "render bound right"),
		_finite(value.bottom, "render bound bottom"))
	if right <= left or bottom <= top:
		raise PresentationRenderPlanError("presentation render bounds have no area")
	return PySide6.QtCore.QRectF(left, top, right - left, bottom - top)


#============================================
def _point(value: object, extension: object, label: str) -> PySide6.QtCore.QPointF:
	"""Copy one finite renderer point."""
	if type(value) is not extension.RenderPointV1:
		raise PresentationRenderPlanError(f"{label} has the wrong DTO type")
	return PySide6.QtCore.QPointF(_finite(value.x, f"{label} x"), _finite(value.y, f"{label} y"))


#============================================
def _revision(value: object) -> int:
	"""Validate one frozen nonnegative document revision."""
	if type(value) is not int or value < 0:
		raise PresentationRenderPlanError("presentation render-plan revision is invalid")
	return value


#============================================
def _digest(value: object) -> str:
	"""Validate one lowercase renderer provenance digest."""
	if type(value) is not str or _DIGEST.fullmatch(value) is None:
		raise PresentationRenderPlanError("presentation render-plan digest is invalid")
	return value


#============================================
def _finite(value: object, label: str) -> float:
	"""Return one finite renderer scalar."""
	if type(value) is not float or not math.isfinite(value):
		raise PresentationRenderPlanError(f"{label} must be finite")
	return value


#============================================
def _positive(value: object, label: str) -> float:
	"""Return one positive finite renderer scalar."""
	value = _finite(value, label)
	if value <= 0.0:
		raise PresentationRenderPlanError(f"{label} must be positive")
	return value


#============================================
def _color(value: object, label: str) -> PySide6.QtGui.QColor:
	"""Return one exact lowercase renderer RGB paint."""
	if type(value) is not str or _RGB24.fullmatch(value) is None:
		raise PresentationRenderPlanError(f"{label} must be lowercase #RRGGBB")
	return PySide6.QtGui.QColor(f"#{value}")


#============================================
def _ferrum_chem() -> object:
	"""Load the exact compiled extension behind the Qt engine facade."""
	try:
		import ferrum_qt.ferrum.engine as engine
		return engine.extension_module()
	except ImportError as exc:
		raise PresentationRenderPlanError("Ferrum presentation render-plan binding is unavailable") from exc

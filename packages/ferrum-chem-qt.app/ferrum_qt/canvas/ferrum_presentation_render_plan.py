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
import ferrum_qt.canvas.graphics_disposal
from ferrum_qt.canvas.display_palette_refreshable import DisplayPaletteRefreshable
import ferrum_qt.canvas.items.ferrum_plus_item
import ferrum_qt.canvas.items.ferrum_text_item
import ferrum_qt.themes.document_display_palette
from ferrum_qt.canvas.ferrum_render_target import RenderTargetKey


_SCHEMA = "ferrum-presentation-render-plan-v1"
_PREVIEW_SCHEMA = "ferrum-presentation-preview-render-plan-v1"
_TELEX_FACE = "ferrum-telex-regular-v1"
_DIGEST = re.compile(r"^[0-9a-f]{64}$")


#============================================
class PresentationRenderPlanError(ValueError):
	"""Raised when one renderer-issued plan cannot be replayed safely."""


#============================================
def require_display_palette_refreshable(root: object,
		description: str) -> DisplayPaletteRefreshable:
	"""Admit only persistent roots that implement the closed refresh contract."""
	if not isinstance(root, PySide6.QtWidgets.QGraphicsItem):
		raise PresentationRenderPlanError(f"{description} is not a graphics item")
	if not isinstance(root, DisplayPaletteRefreshable):
		raise PresentationRenderPlanError(
			f"{description} must implement refresh_display_palette",
		)
	return root


@dataclasses.dataclass(frozen=True)
class _VectorCommand:
	"""One immutable vector path with its retained semantic material facts."""

	path: PySide6.QtGui.QPainterPath
	pen: PySide6.QtGui.QPen | None
	brush: PySide6.QtGui.QBrush | None
	stroke_paint: object | None
	fill_paint: object | None


@dataclasses.dataclass(slots=True)
class FerrumPresentationScene:
	"""Detached Qt replay of one immutable renderer-owned presentation plan."""

	revision: int
	digest: str
	roots: tuple[DisplayPaletteRefreshable, ...]
	items: tuple[DisplayPaletteRefreshable, ...]
	durable_items: dict[tuple[str, str], DisplayPaletteRefreshable]
	local_items: dict[RenderTargetKey, DisplayPaletteRefreshable]

	#============================================
	def __post_init__(self) -> None:
		"""Refuse a detached scene that contains an unrefreshable retained root."""
		self.roots = tuple(
			require_display_palette_refreshable(root, "presentation render root")
			for root in self.roots
		)
		self.items = tuple(
			require_display_palette_refreshable(item, "presentation render item")
			for item in self.items
		)

	#============================================
	def selected_targets(
			self, scene: PySide6.QtWidgets.QGraphicsScene | None,
			) -> tuple[RenderTargetKey, ...]:
		"""Return selected plan targets without promoting local keys to IDs."""
		selected = ferrum_qt.canvas.graphics_disposal.selected_items_from_captured_scene(scene)
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
	def refresh_display_palette(self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Refresh retained root materials without replacing renderer geometry."""
		for root in self.roots:
			root.refresh_display_palette(palette)

	#============================================
	def dispose_detached(self) -> None:
		"""Release a never-installed renderer-plan scene through the shared reaper."""
		coordinator = ferrum_qt.canvas.graphics_disposal.GraphicsDisposalCoordinator()
		coordinator.dispose_detached_projection_items(list(self.roots))
		coordinator.raise_if_callback_failed("Ferrum presentation-scene disposal failed")


@dataclasses.dataclass(slots=True)
class FerrumPresentationPreviewScene:
	"""Detached Qt replay of one immutable identifier-free preview plan."""

	roots: tuple[DisplayPaletteRefreshable, ...]

	#============================================
	def __post_init__(self) -> None:
		"""Refuse a detached preview that contains an unrefreshable retained root."""
		self.roots = tuple(
			require_display_palette_refreshable(root, "presentation preview root")
			for root in self.roots
		)

	#============================================
	def refresh_display_palette(self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Refresh detached preview material while retaining its exact path facts."""
		for root in self.roots:
			root.refresh_display_palette(palette)

	#============================================
	def dispose_detached(self) -> None:
		"""Release never-installed preview roots through the shared reaper."""
		coordinator = ferrum_qt.canvas.graphics_disposal.GraphicsDisposalCoordinator()
		coordinator.dispose_detached_projection_items(list(self.roots))
		coordinator.raise_if_callback_failed("Ferrum presentation-preview disposal failed")


#============================================
class RendererPlanRootItem(
		PySide6.QtWidgets.QGraphicsItem, DisplayPaletteRefreshable):
	"""One selectable root that paints only renderer-issued vector operations."""

	#============================================
	def __init__(self, commands: tuple[_VectorCommand, ...],
			target: RenderTargetKey, bounds: PySide6.QtCore.QRectF) -> None:
		"""Cache validated paths and explicit paints without deriving geometry."""
		super().__init__()
		self._commands = commands
		self._target = target
		self._bounds = bounds
		self._shape = PySide6.QtGui.QPainterPath()
		for command in commands:
			self._shape.addPath(command.path)
			if command.pen is not None:
				stroker = PySide6.QtGui.QPainterPathStroker()
				stroker.setWidth(max(6.0, command.pen.widthF() + 4.0))
				self._shape.addPath(stroker.createStroke(command.path))
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
	def refresh_display_palette(self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Replace vector materials from retained tagged paint facts only."""
		self._commands = _refresh_vector_commands(self._commands, palette)
		self.update()

	#============================================
	def paint(self, painter: PySide6.QtGui.QPainter,
			option: PySide6.QtWidgets.QStyleOptionGraphicsItem,
			widget: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Replay exact paths, strokes, and fills in renderer operation order."""
		del option, widget
		for command in self._commands:
			painter.setPen(
				PySide6.QtCore.Qt.PenStyle.NoPen if command.pen is None else command.pen,
			)
			painter.setBrush(
				PySide6.QtCore.Qt.BrushStyle.NoBrush if command.brush is None else command.brush,
			)
			painter.drawPath(command.path)

	#============================================
	def dispose(self) -> None:
		"""Provide the established projection-disposal callback contract."""


#============================================
class RendererPreviewRootItem(
		PySide6.QtWidgets.QGraphicsItem, DisplayPaletteRefreshable):
	"""One noninteractive root that paints preview vector operations exactly."""

	#============================================
	def __init__(self, commands: tuple[_VectorCommand, ...],
			bounds: PySide6.QtCore.QRectF) -> None:
		"""Cache validated preview paths without creating a selection identity."""
		super().__init__()
		self._commands = commands
		self._bounds = bounds
		self.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, False,
		)
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable, False,
		)

	#============================================
	def boundingRect(self) -> PySide6.QtCore.QRectF:
		"""Return renderer-issued finite preview bounds for Qt scene indexing."""
		return PySide6.QtCore.QRectF(self._bounds)

	#============================================
	def refresh_display_palette(self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Replace preview vector materials without moving or rebuilding paths."""
		self._commands = _refresh_vector_commands(self._commands, palette)
		self.update()

	#============================================
	def paint(self, painter: PySide6.QtGui.QPainter,
			option: PySide6.QtWidgets.QStyleOptionGraphicsItem,
			widget: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Replay exact preview paths, strokes, and fills in operation order."""
		del option, widget
		for command in self._commands:
			painter.setPen(
				PySide6.QtCore.Qt.PenStyle.NoPen if command.pen is None else command.pen,
			)
			painter.setBrush(
				PySide6.QtCore.Qt.BrushStyle.NoBrush if command.brush is None else command.brush,
			)
			painter.drawPath(command.path)

	#============================================
	def dispose(self) -> None:
		"""Provide the established projection-disposal callback contract."""


#============================================
class RendererPreviewPlusItem(
		PySide6.QtWidgets.QGraphicsItem, DisplayPaletteRefreshable):
	"""One noninteractive identifier-free Plus preview from the closed DTO."""

	#============================================
	def __init__(self, plus: object, bounds: PySide6.QtCore.QRectF,
			extension: object, telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1) -> None:
		"""Authenticate and paint the sole preview-plus grammar without a target."""
		super().__init__()
		if type(plus) is not extension.PresentationPreviewPlusV1:
			raise PresentationRenderPlanError("preview Plus has the wrong DTO type")
		if plus.text != "+" or plus.face != _TELEX_FACE or type(plus.z) is not int or plus.z != 20:
			raise PresentationRenderPlanError("preview Plus has invalid fixed text semantics")
		anchor = _point(plus.anchor, extension, "preview Plus anchor")
		origin = _point(plus.operation_origin, extension, "preview Plus text origin")
		font = telex.raw_font(_positive(plus.size, "preview Plus text size"))
		glyph_indexes = font.glyphIndexesForString(plus.text)
		if len(glyph_indexes) != 1 or type(glyph_indexes[0]) is not int or glyph_indexes[0] <= 0:
			raise PresentationRenderPlanError("preview Plus has no verified Telex glyph")
		glyph_path = font.pathForGlyph(glyph_indexes[0])
		if glyph_path.isEmpty():
			raise PresentationRenderPlanError("preview Plus Telex glyph has no outline")
		transform = PySide6.QtGui.QTransform()
		transform.translate(anchor.x() + origin.x(), anchor.y() + origin.y())
		self._glyph_path = transform.map(glyph_path)
		self._foreground_paint = plus.paint
		self._background_paint = plus.background
		self._foreground = PySide6.QtGui.QBrush(_paint(palette, plus.paint, "preview Plus foreground"))
		self._background_path = PySide6.QtGui.QPainterPath()
		self._background = None
		if plus.background is not None:
			self._background_path.addRect(bounds)
			self._background = PySide6.QtGui.QBrush(
				_paint(palette, plus.background, "preview Plus background"),
			)
		self._bounds = PySide6.QtCore.QRectF(bounds)
		self.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, False,
		)
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable, False,
		)

	#============================================
	def boundingRect(self) -> PySide6.QtCore.QRectF:
		"""Return renderer-issued preview bounds without deriving a target shape."""
		return PySide6.QtCore.QRectF(self._bounds)

	#============================================
	def refresh_display_palette(self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Replace preview-plus brushes from frozen tagged paint facts."""
		self._foreground = PySide6.QtGui.QBrush(
			_paint(palette, self._foreground_paint, "preview Plus foreground"),
		)
		if self._background_paint is not None:
			self._background = PySide6.QtGui.QBrush(
				_paint(palette, self._background_paint, "preview Plus background"),
			)
		self.update()

	#============================================
	def paint(self, painter: PySide6.QtGui.QPainter,
			option: PySide6.QtWidgets.QStyleOptionGraphicsItem,
			widget: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Paint fixed preview Plus paths without selection affordances."""
		del option, widget
		painter.setPen(PySide6.QtCore.Qt.PenStyle.NoPen)
		if self._background is not None:
			painter.setBrush(self._background)
			painter.drawPath(self._background_path)
		painter.setBrush(self._foreground)
		painter.drawPath(self._glyph_path)

	#============================================
	def dispose(self) -> None:
		"""Provide the established projection-disposal callback contract."""


#============================================
def build_presentation_render_plan(plan: object, telex_resource: object,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1) -> FerrumPresentationScene:
	"""Build detached Qt roots from one exact fenced renderer plan."""
	extension = _ferrum_chem()
	if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
		raise PresentationRenderPlanError("presentation render plan requires a document display palette")
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
	roots: list[DisplayPaletteRefreshable] = []
	durable_items: dict[tuple[str, str], DisplayPaletteRefreshable] = {}
	local_items: dict[RenderTargetKey, DisplayPaletteRefreshable] = {}
	try:
		for root in plan.roots:
			target = _target(root.target, extension)
			bounds = _bounds(root.bounds, extension)
			item = _root_item(root, target, bounds, extension, telex, palette)
			if target in local_items:
				raise PresentationRenderPlanError("duplicate presentation target")
			durable_key = target.durable_selection_key()
			if durable_key in durable_items:
				raise PresentationRenderPlanError("duplicate durable presentation target")
			durable_items[durable_key] = item
			local_items[target] = item
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
def build_presentation_preview_render_plan(
		plan: object, telex_resource: object,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> FerrumPresentationPreviewScene:
	"""Build detached noninteractive roots from one exact preview-only plan."""
	extension = _ferrum_chem()
	if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
		raise PresentationRenderPlanError("presentation preview render plan requires a document display palette")
	if type(plan) is not extension.PresentationPreviewRenderPlanV1:
		raise PresentationRenderPlanError(
			"presentation preview render plan must be engine.PresentationPreviewRenderPlanV1",
		)
	if plan.schema != _PREVIEW_SCHEMA:
		raise PresentationRenderPlanError("unknown presentation preview render-plan schema")
	if type(plan.roots) is not tuple:
		raise PresentationRenderPlanError("presentation preview render-plan roots must be frozen")
	telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex_resource)
	roots: list[DisplayPaletteRefreshable] = []
	try:
		for root in plan.roots:
			if type(root) is not extension.PresentationPreviewRenderRootV1:
				raise PresentationRenderPlanError("preview render root has the wrong DTO type")
			bounds = _bounds(root.bounds, extension)
			roots.append(_preview_root_item(root, bounds, extension, telex, palette))
	except (AttributeError, TypeError, ValueError, PresentationRenderPlanError) as exc:
		for item in roots:
			item.dispose()
		if isinstance(exc, PresentationRenderPlanError):
			raise
		raise PresentationRenderPlanError("invalid frozen presentation preview render plan") from exc
	return FerrumPresentationPreviewScene(tuple(roots))


#============================================
def _root_item(root: object, target: RenderTargetKey, bounds: PySide6.QtCore.QRectF,
		extension: object, telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> DisplayPaletteRefreshable:
	"""Validate one discriminated root and build only its documented variant."""
	kind = getattr(root, "kind", None)
	if kind == "vector":
		if root.plus is not None or root.text is not None or type(root.vector_operations) is not tuple:
			raise PresentationRenderPlanError("vector render root has mixed variants")
		commands = tuple(_vector_operation(operation, extension, palette) for operation in root.vector_operations)
		if not commands:
			raise PresentationRenderPlanError("vector render root has no operations")
		return require_display_palette_refreshable(
			RendererPlanRootItem(commands, target, bounds), "vector render root",
		)
	if kind == "plus":
		if root.plus is None or root.text is not None or root.vector_operations != ():
			raise PresentationRenderPlanError("plus render root has mixed variants")
		item = ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem._from_observation(
			root.plus, telex, palette,
		)
		return _require_matching_item_target(item, target)
	if kind == "text":
		if root.text is None or root.plus is not None or root.vector_operations != ():
			raise PresentationRenderPlanError("text render root has mixed variants")
		item = ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem._from_observation(
			root.text, telex, palette,
		)
		return _require_matching_item_target(item, target)
	raise PresentationRenderPlanError("unknown presentation render-root kind")


#============================================
def _preview_root_item(root: object, bounds: PySide6.QtCore.QRectF, extension: object,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> DisplayPaletteRefreshable:
	"""Validate one identifier-free root and build its documented preview variant."""
	if root.kind == "vector":
		if root.plus is not None or type(root.vector_operations) is not tuple:
			raise PresentationRenderPlanError("preview vector render root has mixed variants")
		commands = tuple(_vector_operation(operation, extension, palette) for operation in root.vector_operations)
		if not commands:
			raise PresentationRenderPlanError("preview vector render root has no operations")
		return require_display_palette_refreshable(
			RendererPreviewRootItem(commands, bounds), "preview vector render root",
		)
	if root.kind == "plus":
		if root.plus is None or root.vector_operations != ():
			raise PresentationRenderPlanError("preview Plus render root has mixed variants")
		return require_display_palette_refreshable(
			RendererPreviewPlusItem(root.plus, bounds, extension, telex, palette),
			"preview Plus render root",
		)
	raise PresentationRenderPlanError("unknown presentation preview render-root kind")


#============================================
def _require_matching_item_target(item: object,
		target: RenderTargetKey) -> DisplayPaletteRefreshable:
	"""Ensure a specialized renderer root retained the plan root's exact identity."""
	if item.target != target:
		raise PresentationRenderPlanError("specialized render root target differs from plan target")
	return require_display_palette_refreshable(item, "specialized render root")


#============================================
def _vector_operation(operation: object, extension: object,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> _VectorCommand:
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
	return _VectorCommand(
		path, _pen(operation.stroke, extension, palette), _brush(operation.fill, palette),
		None if operation.stroke is None else operation.stroke.paint, operation.fill,
	)


#============================================
def _refresh_vector_commands(commands: tuple[_VectorCommand, ...],
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> tuple[_VectorCommand, ...]:
	"""Replace only Qt vector materials while preserving frozen paths and strokes."""
	if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
		raise PresentationRenderPlanError("vector refresh requires a document display palette")
	refreshed: list[_VectorCommand] = []
	for command in commands:
		pen = command.pen
		if command.stroke_paint is not None:
			if pen is None:
				raise PresentationRenderPlanError("vector command lost its required stroke")
			pen = PySide6.QtGui.QPen(pen)
			pen.setColor(_paint(palette, command.stroke_paint, "renderer stroke paint"))
		brush = command.brush
		if command.fill_paint is not None:
			brush = PySide6.QtGui.QBrush(
				_paint(palette, command.fill_paint, "renderer fill paint"),
			)
		refreshed.append(dataclasses.replace(command, pen=pen, brush=brush))
	return tuple(refreshed)


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
def _pen(value: object, extension: object,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1) -> PySide6.QtGui.QPen | None:
	"""Copy one complete renderer-issued stroke without Qt defaults."""
	if value is None:
		return None
	if type(value) is not extension.PresentationRenderStrokeV1:
		raise PresentationRenderPlanError("renderer stroke has the wrong DTO type")
	if value.line_cap != "butt" or value.line_join != "miter":
		raise PresentationRenderPlanError("renderer stroke has unsupported line semantics")
	pen = PySide6.QtGui.QPen(_paint(palette, value.paint, "renderer stroke paint"))
	pen.setWidthF(_positive(value.width, "renderer stroke width"))
	pen.setCapStyle(PySide6.QtCore.Qt.PenCapStyle.FlatCap)
	pen.setJoinStyle(PySide6.QtCore.Qt.PenJoinStyle.MiterJoin)
	pen.setMiterLimit(_positive(value.miter_limit, "renderer stroke miter limit"))
	return pen


#============================================
def _brush(value: object,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1) -> PySide6.QtGui.QBrush | None:
	"""Copy an optional renderer-issued fill paint."""
	return None if value is None else PySide6.QtGui.QBrush(_paint(palette, value, "renderer fill paint"))


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
def _paint(palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		value: object, label: str) -> PySide6.QtGui.QColor:
	"""Resolve one V3 paint through the sole Qt display-palette authority."""
	try:
		return palette.resolve_render_paint(value)
	except ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteError as error:
		raise PresentationRenderPlanError(f"{label} is an invalid tagged render paint") from error


#============================================
def _ferrum_chem() -> object:
	"""Load the exact compiled extension behind the Qt engine facade."""
	try:
		import ferrum_qt.ferrum.engine as engine
		return engine.extension_module()
	except ImportError as exc:
		raise PresentationRenderPlanError("Ferrum presentation render-plan binding is unavailable") from exc

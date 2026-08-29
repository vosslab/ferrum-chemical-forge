"""Strict Qt projection item for one immutable Ferrum RenderBatchV4."""

# Standard Library
import dataclasses
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
from ferrum_qt.canvas.display_palette_refreshable import DisplayPaletteRefreshable
import ferrum_qt.canvas.ferrum_telex
import ferrum_qt.canvas.telex_glyph_outline
import ferrum_qt.themes.document_display_palette


_SCHEMA = "ferrum-render-plan-v4"
_FACE = "ferrum-telex-regular-v1"
_PADDING = 1.0
_SELECTION_WIDTH = 1.5
_HOVER_WIDTH = 1.0
_INK_BOUNDS_TOLERANCE = 0.000_001


#============================================
class FerrumPlanError(ValueError):
	"""Raised when a frozen V4 render batch cannot be projected faithfully."""


@dataclasses.dataclass(frozen=True)
class _Point:
	"""Validated, detached finite coordinates."""

	x: float
	y: float


@dataclasses.dataclass(frozen=True)
class _Target:
	"""Detached durable target identity available to selection projection later."""

	document_object_id: str


@dataclasses.dataclass(frozen=True)
class _Line:
	"""Cached scene line paint command."""

	path: PySide6.QtGui.QPainterPath
	pen: PySide6.QtGui.QPen
	z: int
	paint: object


@dataclasses.dataclass(frozen=True)
class _Fill:
	"""Cached atom-local mask or text outline paint command."""

	path: PySide6.QtGui.QPainterPath
	brush: PySide6.QtGui.QBrush
	z: int
	paint: object


@dataclasses.dataclass(frozen=True)
class _Shape:
	"""Cached atom-local shape with explicit optional outline and fill."""

	path: PySide6.QtGui.QPainterPath
	pen: PySide6.QtGui.QPen | None
	brush: PySide6.QtGui.QBrush | None
	z: int
	stroke_paint: object | None
	fill_paint: object | None


#============================================
class FerrumPlanItem(
		PySide6.QtWidgets.QGraphicsObject, DisplayPaletteRefreshable):
	"""A selectable, immovable projection of one complete frozen render batch.

	The caller supplies a frozen value carrying the parent plan's schema and
	revision, not a mutable map or XML tree.  Construction validates and caches
	every content path and paint state, so ``paint()`` cannot shape, measure, or
	otherwise reinterpret Rust's declarative geometry.
	"""

	#============================================
	def __init__(self, plan: object, batch_index: int, telex_resource: object,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			parent: PySide6.QtWidgets.QGraphicsItem | None = None) -> None:
		"""Validate one frozen batch and cache all Qt-local geometry.

		Raises:
			FerrumPlanError: If the closed V1 shape cannot be faithfully painted.
		"""
		super().__init__(parent)
		validated_plan = _runtime_plan(plan)
		telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex_resource)
		self._initialize(validated_plan, batch_index, telex, palette)

	#============================================
	@classmethod
	def _from_fixture(cls, plan: object, batch_index: int,
			telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			parent: PySide6.QtWidgets.QGraphicsItem | None = None) -> "FerrumPlanItem":
		"""Build an isolated item from test-only frozen fixtures, never app code."""
		item = _FixtureFerrumPlanItem(plan, batch_index, telex, palette, parent)
		return item

	#============================================
	def _initialize(self, plan: object, batch_index: int,
			telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1) -> None:
		"""Initialize an already-authenticated runtime or fixture input boundary."""
		if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
			raise FerrumPlanError("Ferrum plan item requires a document display palette")
		self._plan = plan
		self._batch_index = batch_index
		self._telex = telex
		self._palette = palette
		self._target, self._commands = _copy_batch(plan, batch_index, telex, palette)
		self._content_path = _content_path(self._commands)
		self._shape_path = _shape_for(self._commands, self._content_path)
		self._bounds = self._shape_path.boundingRect().adjusted(
			-_PADDING, -_PADDING, _PADDING, _PADDING,
		)
		self._hovered = False
		self.setFlag(PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, True)
		self.setFlag(PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable, False)
		self.setAcceptHoverEvents(True)

	#============================================
	def refresh_display_palette(self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1) -> None:
		"""Refresh retained Qt pens and brushes from frozen V4 payload operations."""
		if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
			raise FerrumPlanError("Ferrum plan item requires a document display palette")
		self._palette = palette
		self._commands = _refresh_commands(self._commands, palette)
		self.update()

	#============================================
	def target(self) -> object:
		"""Return the detached durable target for a future selection projection."""
		return self._target

	#============================================
	def boundingRect(self) -> PySide6.QtCore.QRectF:
		"""Return the cached bounds derived from the same paths used for paint."""
		return self._bounds

	#============================================
	def shape(self) -> PySide6.QtGui.QPainterPath:
		"""Return the cached exact content-and-line-stroke hit-test geometry."""
		return self._shape_path

	#============================================
	def paint(self, painter: PySide6.QtGui.QPainter,
			option: PySide6.QtWidgets.QStyleOptionGraphicsItem,
			widget: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Paint cached explicit commands, then only Qt-local interaction decoration."""
		for command in self._commands:
			if isinstance(command, _Line):
				painter.setPen(command.pen)
				painter.setBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
				painter.drawPath(command.path)
			elif isinstance(command, _Fill):
				painter.setPen(PySide6.QtCore.Qt.PenStyle.NoPen)
				painter.setBrush(command.brush)
				painter.drawPath(command.path)
			else:
				painter.setPen(
					command.pen if command.pen is not None
					else PySide6.QtCore.Qt.PenStyle.NoPen,
				)
				painter.setBrush(
					command.brush if command.brush is not None
					else PySide6.QtCore.Qt.BrushStyle.NoBrush,
				)
				painter.drawPath(command.path)
		self._paint_decoration(painter)

	#============================================
	def hoverEnterEvent(self, event: PySide6.QtWidgets.QGraphicsSceneHoverEvent) -> None:
		"""Show a Qt-local hover outline without changing render-plan content."""
		self._hovered = True
		self.update()

	#============================================
	def hoverLeaveEvent(self, event: PySide6.QtWidgets.QGraphicsSceneHoverEvent) -> None:
		"""Remove the Qt-local hover outline."""
		self._hovered = False
		self.update()

	#============================================
	def _paint_decoration(self, painter: PySide6.QtGui.QPainter) -> None:
		"""Paint a UI-palette-only overlay after immutable document depiction."""
		if not self.isSelected() and not self._hovered:
			return
		selected = self.isSelected()
		width = _SELECTION_WIDTH if selected else _HOVER_WIDTH
		color = self._palette.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.SELECTION_OUTLINE
			if selected else ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.HOVER_OUTLINE,
		)
		pen = PySide6.QtGui.QPen(color)
		pen.setWidthF(width)
		pen.setCosmetic(False)
		pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine if selected else PySide6.QtCore.Qt.PenStyle.SolidLine)
		painter.setPen(pen)
		painter.setBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
		inset = _PADDING + width / 2.0
		painter.drawRect(self._bounds.adjusted(inset, inset, -inset, -inset))


#============================================
class _FixtureFerrumPlanItem(FerrumPlanItem):
	"""Private construction seam for frozen focused DTO fixtures only."""

	#============================================
	def __init__(self, plan: object, batch_index: int,
			telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			parent: PySide6.QtWidgets.QGraphicsItem | None = None) -> None:
		"""Initialize the base Qt object without exposing fixture validation publicly."""
		PySide6.QtWidgets.QGraphicsObject.__init__(self, parent)
		if not isinstance(telex, ferrum_qt.canvas.ferrum_telex.FerrumTelex):
			raise FerrumPlanError("Ferrum fixture item requires verified Telex bytes")
		self._initialize(plan, batch_index, telex, palette)


#============================================
def _runtime_plan(value: object) -> object:
	"""Accept only the exact compiled frozen ``engine.RenderPlanV4`` type."""
	try:
		import ferrum_qt.ferrum.engine as engine
	except ImportError as error:
		raise FerrumPlanError("Ferrum render plans require the installed ferrum_chem extension") from error
	if type(value) is not engine.RenderPlanV4:
		raise FerrumPlanError("Ferrum plan item requires the frozen ferrum_chem RenderPlanV4")
	if not isinstance(value.batches, tuple):
		raise FerrumPlanError("Ferrum render plan batches must be a frozen tuple")
	previous_paint_order = -1
	for batch in value.batches:
		if type(batch) is not engine.RenderBatchV4:
			raise FerrumPlanError("Ferrum render plan contains a non-frozen batch")
		paint_order = _u32(batch.paint_order, "paint order")
		if paint_order <= previous_paint_order:
			raise FerrumPlanError("Ferrum render plan paint order must be strictly increasing")
		previous_paint_order = paint_order
	return value


#============================================
def _copy_batch(plan: object, batch_index: int,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> tuple[_Target, tuple[_Line | _Fill | _Shape, ...]]:
	"""Copy one indexed plan-owned batch without dict, XML, defaults, or shaping."""
	if plan.schema != _SCHEMA:
		raise FerrumPlanError("Ferrum render plan has an unknown schema")
	_revision(plan.provenance.revision)
	_digest(plan.provenance.digest)
	batch = _batch_at(plan.batches, batch_index)
	target = _target(batch.target)
	content = batch.content
	_validate_coordinate_space(batch.coordinate_space, content)
	commands = _copy_typed_content(content, telex, palette)
	if not commands:
		raise FerrumPlanError("Ferrum render batch must contain an operation")
	return target, tuple(commands)


#============================================
def _validate_coordinate_space(space: object, content: object) -> None:
	"""Validate the redundant batch-space receipt without adopting it as geometry authority."""
	try:
		import ferrum_qt.ferrum.engine as engine
	except ImportError as error:
		raise FerrumPlanError("Ferrum coordinate space requires the installed ferrum_chem extension") from error
	kind = getattr(content, "kind", None)
	if kind in {"atom", "compact_group"}:
		if type(space) is not engine.AtomLocalSpaceV1 or space.kind != "atom_local":
			raise FerrumPlanError("Ferrum atom-local content requires an exact atom-local coordinate space")
		issued_anchor = _point(content.atom_local_anchor, "typed atom-local anchor")
		space_anchor = _point(space.anchor, "coordinate-space atom-local anchor")
		if issued_anchor != space_anchor:
			raise FerrumPlanError("Ferrum coordinate-space anchor differs from typed content anchor")
		return
	if kind == "bond":
		if type(space) is not engine.SceneSpaceV1 or space.kind != "scene":
			raise FerrumPlanError("Ferrum bond content requires an exact scene coordinate space")
		return
	raise FerrumPlanError("Ferrum render content kind cannot select a coordinate space")


#============================================
def _copy_typed_content(content: object,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> list[_Line | _Fill | _Shape]:
	"""Replay one closed V4 content DTO without consulting generic operations."""
	try:
		import ferrum_qt.ferrum.engine as engine
	except ImportError as error:
		raise FerrumPlanError("Ferrum render content requires the installed ferrum_chem extension") from error
	kind = getattr(content, "kind", None)
	if kind == "atom" and type(content) is engine.AtomRenderBatchV1:
		return _copy_atom_content(content, engine, telex, palette)
	if kind == "compact_group" and type(content) is engine.CompactGroupRenderBatchV1:
		return _copy_compact_group_content(content, engine, telex, palette)
	if kind == "bond" and type(content) is engine.BondRenderBatchV1:
		return _copy_bond_content(content, engine, telex, palette)
	raise FerrumPlanError("Ferrum render batch has an unknown or mismatched typed content")


#============================================
def _copy_atom_content(content: object, engine: object,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> list[_Line | _Fill | _Shape]:
	"""Replay atom label/mask/decorations from the atom-only payload authority."""
	anchor = _point(content.atom_local_anchor, "atom-local anchor")
	label = content.label
	if type(label) is not engine.AtomLabelRenderV1:
		raise FerrumPlanError("Ferrum atom batch has no exact typed label")
	if type(label.text) is not engine.TextOpV1:
		raise FerrumPlanError("Ferrum atom label has no exact text operation")
	if not isinstance(label.text.runs, tuple) or not label.text.runs:
		raise FerrumPlanError("Ferrum atom label runs must be a nonempty frozen tuple")
	index = _u32(label.core_element_run_index, "core element run index")
	if index >= len(label.text.runs):
		raise FerrumPlanError("Ferrum atom core element run index is outside label runs")
	core_run = label.text.runs[index]
	if type(core_run) is not engine.TextRunV1 or core_run.script != "baseline":
		raise FerrumPlanError("Ferrum atom core element run must be an exact baseline run")
	_positive(label.bond_ink_clearance, "atom-label bond-ink clearance")
	full_bounds = _ink_bounds(label.full_ink_bounds, engine, "full atom-label ink bounds")
	core_bounds = _ink_bounds(label.core_element_ink_bounds, engine, "core atom-label ink bounds")
	if not _contains_bounds(full_bounds, core_bounds):
		raise FerrumPlanError("Ferrum atom core ink bounds are outside full label ink bounds")
	if (core_bounds.min_x + core_bounds.max_x) / 2.0 != 0.0 or (core_bounds.min_y + core_bounds.max_y) / 2.0 != 0.0:
		raise FerrumPlanError("Ferrum atom core ink bounds are not centered at the local origin")
	commands: list[_Line | _Fill | _Shape] = []
	if label.mask is not None:
		if type(label.mask) is not engine.MaskOpV1:
			raise FerrumPlanError("Ferrum atom label mask has the wrong DTO type")
		commands.append(_copy_operation("mask", label.mask, anchor, telex, palette)[0])
	text_command, _text_z = _copy_operation("text", label.text, anchor, telex, palette)
	if not isinstance(text_command, _Fill):
		raise FerrumPlanError("Ferrum atom label did not produce a fill command")
	commands.append(text_command)
	_validate_label_outlines(label.text, core_run, anchor, telex, full_bounds, core_bounds)
	decorations = content.decorations
	if not isinstance(decorations, tuple):
		raise FerrumPlanError("Ferrum atom decorations must be a frozen tuple")
	for decoration in decorations:
		if type(decoration) is not engine.AtomDecorationRenderOpV1:
			raise FerrumPlanError("Ferrum atom decoration has the wrong DTO type")
		commands.append(_copy_closed_operation(
			decoration, {"text", "line", "ellipse"}, anchor, telex, palette,
		))
	_validate_strict_z(commands)
	return commands


#============================================
def _copy_compact_group_content(content: object, engine: object,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> list[_Line | _Fill | _Shape]:
	"""Replay a compact-group payload only through its closed typed operations."""
	anchor = _point(content.atom_local_anchor, "compact-group atom-local anchor")
	operations = content.typed_operations
	if not isinstance(operations, tuple):
		raise FerrumPlanError("Ferrum compact-group operations must be a frozen tuple")
	commands = []
	for operation in operations:
		if type(operation) is not engine.CompactGroupRenderOpV1:
			raise FerrumPlanError("Ferrum compact-group operation has the wrong DTO type")
		commands.append(_copy_closed_operation(
			operation, {"text", "line", "ellipse"}, anchor, telex, palette,
		))
	_validate_strict_z(commands)
	return commands


#============================================
def _copy_bond_content(content: object, engine: object,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> list[_Line | _Fill | _Shape]:
	"""Validate structural attachment, then replay only the clipped bond ink."""
	axis = content.attachment_axis
	if type(axis) is not engine.BondAttachmentAxisV1:
		raise FerrumPlanError("Ferrum bond batch has no exact attachment axis")
	start = _point(axis.start, "bond attachment-axis start")
	end = _point(axis.end, "bond attachment-axis end")
	if start == end:
		raise FerrumPlanError("Ferrum bond attachment-axis endpoints are coincident")
	operations = content.typed_operations
	if not isinstance(operations, tuple):
		raise FerrumPlanError("Ferrum bond operations must be a frozen tuple")
	commands = []
	for operation in operations:
		if type(operation) is not engine.BondRenderOpV1:
			raise FerrumPlanError("Ferrum bond operation has the wrong DTO type")
		commands.append(_copy_closed_operation(
			operation, {"line", "path", "double_bond_carrier_mark"},
			_Point(0.0, 0.0), telex, palette,
		))
	_validate_strict_z(commands)
	return commands


#============================================
def _copy_closed_operation(operation: object, allowed: set[str], anchor: _Point,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> _Line | _Fill | _Shape:
	"""Copy one typed variant after its owner selected the permitted grammar."""
	kind = getattr(operation, "kind", None)
	if kind not in allowed:
		raise FerrumPlanError("Ferrum typed render operation is invalid for its content")
	try:
		import ferrum_qt.ferrum.engine as engine
	except ImportError as error:
		raise FerrumPlanError("Ferrum render operation requires the installed ferrum_chem extension") from error
	expected_types = {
		"text": engine.TextOpV1,
		"line": engine.LineOpV1,
		"ellipse": engine.EllipseOpV1,
		"path": engine.PathOpV3,
		"double_bond_carrier_mark": engine.LineOpV1,
	}
	if type(operation.operation) is not expected_types[kind]:
		raise FerrumPlanError("Ferrum typed render operation payload has the wrong DTO type")
	command, _z_value = _copy_operation(kind, operation.operation, anchor, telex, palette)
	return command


#============================================
def _validate_label_outlines(text: object, core_run: object, anchor: _Point,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex, full_bounds: object,
		core_bounds: object) -> None:
	"""Compare Telex outlines with Rust-issued label bounds without remeasuring text."""
	font = telex.raw_font(_positive(text.size, "text size"))
	origin = _translated_point(text.origin, anchor, "text origin")
	point = PySide6.QtCore.QPointF(origin.x, origin.y)
	try:
		full_path = ferrum_qt.canvas.telex_glyph_outline.path_from_runs(text.runs, point, font)
		core_path = ferrum_qt.canvas.telex_glyph_outline.path_from_runs((core_run,), point, font)
	except ferrum_qt.canvas.telex_glyph_outline.TelexGlyphOutlineError as exc:
		raise FerrumPlanError(str(exc)) from exc
	_validate_outline_bounds(full_path.boundingRect(), full_bounds, anchor, "full atom label")
	_validate_outline_bounds(core_path.boundingRect(), core_bounds, anchor, "core atom label")


#============================================
def _validate_outline_bounds(rect: PySide6.QtCore.QRectF, bounds: object, anchor: _Point,
		label: str) -> None:
	"""Require Telex outline bounds to agree with Rust geometry within numeric noise."""
	expected = (bounds.min_x + anchor.x, bounds.min_y + anchor.y,
		bounds.max_x + anchor.x, bounds.max_y + anchor.y)
	actual = (rect.left(), rect.top(), rect.right(), rect.bottom())
	if any(abs(actual_value - expected_value) > _INK_BOUNDS_TOLERANCE
			for actual_value, expected_value in zip(actual, expected, strict=True)):
		raise FerrumPlanError(f"Ferrum Telex {label} outline differs from Rust ink bounds")


#============================================
def _validate_strict_z(commands: list[_Line | _Fill | _Shape]) -> None:
	"""Keep the exact Rust paint sequence within one closed batch."""
	previous_z: int | None = None
	for command in commands:
		if previous_z is not None and command.z <= previous_z:
			raise FerrumPlanError("Ferrum render batch z order must be strictly increasing")
		previous_z = command.z


#============================================
def _copy_operation(kind: str, payload: object, anchor: _Point,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> tuple[_Line | _Fill | _Shape, int]:
	"""Detach one exact wire enum operation and construct its cached paint command."""
	if kind in {"line", "double_bond_carrier_mark"}:
		start = _translated_point(payload.start, anchor, "line start")
		end = _translated_point(payload.end, anchor, "line end")
		if start == end:
			raise FerrumPlanError("Ferrum render line endpoints must differ")
		pen = PySide6.QtGui.QPen(_paint(palette, payload.paint))
		pen.setWidthF(_positive(payload.width, "line width"))
		pen.setCosmetic(False)
		pen.setCapStyle(PySide6.QtCore.Qt.PenCapStyle.FlatCap)
		pen.setJoinStyle(PySide6.QtCore.Qt.PenJoinStyle.MiterJoin)
		path = PySide6.QtGui.QPainterPath()
		path.moveTo(start.x, start.y)
		path.lineTo(end.x, end.y)
		z = _z(payload.z)
		return _Line(path, pen, z, payload.paint), z
	if kind == "mask":
		origin = _translated_point(payload.origin, anchor, "mask origin")
		width = _positive(payload.width, "mask width")
		height = _positive(payload.height, "mask height")
		path = PySide6.QtGui.QPainterPath()
		path.addRect(origin.x, origin.y, width, height)
		z = _z(payload.z)
		return _Fill(path, PySide6.QtGui.QBrush(_paint(palette, payload.paint)), z, payload.paint), z
	if kind == "text":
		if payload.face != _FACE:
			raise FerrumPlanError("Ferrum render text requested an unknown font face")
		origin = _translated_point(payload.origin, anchor, "text origin")
		font = telex.raw_font(_positive(payload.size, "text size"))
		try:
			path = ferrum_qt.canvas.telex_glyph_outline.path_from_runs(
				payload.runs, PySide6.QtCore.QPointF(origin.x, origin.y), font,
			)
		except ferrum_qt.canvas.telex_glyph_outline.TelexGlyphOutlineError as exc:
			raise FerrumPlanError(str(exc)) from exc
		z = _z(payload.z)
		return _Fill(path, PySide6.QtGui.QBrush(_paint(palette, payload.paint)), z, payload.paint), z
	if kind == "ellipse":
		center = _translated_point(payload.center, anchor, "ellipse center")
		radius_x = _positive(payload.radius_x, "ellipse x radius")
		radius_y = _positive(payload.radius_y, "ellipse y radius")
		rotation = _finite(payload.rotation_degrees, "ellipse rotation")
		stroke_width = _optional_positive(payload.stroke_width, "ellipse stroke width")
		stroke_paint = _optional_paint(palette, payload.stroke_paint, "ellipse stroke paint")
		fill_paint = _optional_paint(palette, payload.fill_paint, "ellipse fill paint")
		if (stroke_width is None) != (stroke_paint is None):
			raise FerrumPlanError("Ferrum render ellipse outline requires width and paint")
		if stroke_paint is None and fill_paint is None:
			raise FerrumPlanError("Ferrum render ellipse requires an outline or fill")
		path = PySide6.QtGui.QPainterPath()
		path.addEllipse(PySide6.QtCore.QRectF(
			center.x - radius_x, center.y - radius_y,
			radius_x * 2.0, radius_y * 2.0,
		))
		if rotation != 0.0:
			transform = PySide6.QtGui.QTransform()
			transform.translate(center.x, center.y)
			transform.rotate(rotation)
			transform.translate(-center.x, -center.y)
			path = transform.map(path)
		pen = None
		if stroke_width is not None and stroke_paint is not None:
			pen = PySide6.QtGui.QPen(stroke_paint)
			pen.setWidthF(stroke_width)
			pen.setCosmetic(False)
		brush = PySide6.QtGui.QBrush(fill_paint) if fill_paint is not None else None
		z = _z(payload.z)
		return _Shape(path, pen, brush, z, payload.stroke_paint, payload.fill_paint), z
	if kind == "path":
		stroke_width = _optional_positive(payload.stroke_width, "path stroke width")
		stroke_paint = _optional_paint(palette, payload.stroke_paint, "path stroke paint")
		stroke_line_cap = getattr(payload, "stroke_line_cap", None)
		fill_paint = _optional_paint(palette, payload.fill_paint, "path fill paint")
		if (stroke_width is None) != (stroke_paint is None):
			raise FerrumPlanError("Ferrum render path outline requires width and paint")
		if stroke_paint is None and fill_paint is None:
			raise FerrumPlanError("Ferrum render path requires an outline or fill")
		path = _path_from_commands(payload.commands, anchor)
		if fill_paint is not None:
			# Rust fixes filled V2 paths to even-odd; retain that semantic on
			# the cached path used for painting, bounds, and hit testing.
			path.setFillRule(PySide6.QtCore.Qt.FillRule.OddEvenFill)
		pen = None
		if stroke_width is not None and stroke_paint is not None:
			if stroke_line_cap not in {"butt", "round"}:
				raise FerrumPlanError("Ferrum render path stroke cap is invalid")
			pen = PySide6.QtGui.QPen(stroke_paint)
			pen.setWidthF(stroke_width)
			pen.setCosmetic(False)
			pen.setCapStyle({
				"butt": PySide6.QtCore.Qt.PenCapStyle.FlatCap,
				"round": PySide6.QtCore.Qt.PenCapStyle.RoundCap,
			}[stroke_line_cap])
			pen.setJoinStyle(PySide6.QtCore.Qt.PenJoinStyle.MiterJoin)
		brush = PySide6.QtGui.QBrush(fill_paint) if fill_paint is not None else None
		z = _z(payload.z)
		return _Shape(path, pen, brush, z, payload.stroke_paint, payload.fill_paint), z
	raise FerrumPlanError("Ferrum render batch has an unknown operation")


#============================================
def _path_from_commands(commands: object, anchor: _Point) -> PySide6.QtGui.QPainterPath:
	"""Copy finite Rust V2 path geometry without selecting presentation facts."""
	if not isinstance(commands, tuple) or not commands:
		raise FerrumPlanError("Ferrum render path commands must be a nonempty frozen tuple")
	path = PySide6.QtGui.QPainterPath()
	has_drawable = False
	for command in commands:
		if command.kind == "move_to":
			point = _translated_point(command.point, anchor, "path move point")
			path.moveTo(point.x, point.y)
		elif command.kind == "line_to":
			point = _translated_point(command.point, anchor, "path line point")
			path.lineTo(point.x, point.y)
			has_drawable = True
		elif command.kind == "cubic_to":
			control_1 = _translated_point(command.control_1, anchor, "path first control")
			control_2 = _translated_point(command.control_2, anchor, "path second control")
			end = _translated_point(command.point, anchor, "path end point")
			path.cubicTo(control_1.x, control_1.y, control_2.x, control_2.y, end.x, end.y)
			has_drawable = True
		elif command.kind == "close":
			path.closeSubpath()
		else:
			raise FerrumPlanError("Ferrum render path has an unknown command")
	if not has_drawable:
		raise FerrumPlanError("Ferrum render path has no drawable command")
	return path


#============================================
def _content_path(commands: tuple[_Line | _Fill | _Shape, ...]) -> PySide6.QtGui.QPainterPath:
	"""Combine the cached paths used by paint without rebuilding operation geometry."""
	path = PySide6.QtGui.QPainterPath()
	for command in commands:
		path.addPath(command.path)
	return path


#============================================
def _refresh_commands(commands: tuple[_Line | _Fill | _Shape, ...],
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> tuple[_Line | _Fill | _Shape, ...]:
	"""Replace cached materials while retaining every frozen Qt geometry path."""
	refreshed = []
	for command in commands:
		if isinstance(command, _Line):
			pen = PySide6.QtGui.QPen(command.pen)
			pen.setColor(_paint(palette, command.paint))
			refreshed.append(dataclasses.replace(command, pen=pen))
		elif isinstance(command, _Fill):
			refreshed.append(dataclasses.replace(
				command, brush=PySide6.QtGui.QBrush(_paint(palette, command.paint)),
			))
		else:
			pen = command.pen
			if command.stroke_paint is not None:
				if pen is None:
					raise FerrumPlanError("Ferrum shape lost its required cached outline")
				pen = PySide6.QtGui.QPen(pen)
				pen.setColor(_paint(palette, command.stroke_paint))
			brush = command.brush
			if command.fill_paint is not None:
				brush = PySide6.QtGui.QBrush(_paint(palette, command.fill_paint))
			refreshed.append(dataclasses.replace(command, pen=pen, brush=brush))
	return tuple(refreshed)


#============================================
def _shape_for(commands: tuple[_Line | _Fill | _Shape, ...],
		content: PySide6.QtGui.QPainterPath) -> PySide6.QtGui.QPainterPath:
	"""Return cached content plus exact noncosmetic line stroke hit-test geometry."""
	shape = PySide6.QtGui.QPainterPath(content)
	for command in commands:
		if isinstance(command, _Shape):
			pen = command.pen
		elif isinstance(command, _Line):
			pen = command.pen
		else:
			continue
		if pen is None:
			continue
		stroker = PySide6.QtGui.QPainterPathStroker()
		stroker.setWidth(pen.widthF())
		stroker.setCapStyle(pen.capStyle())
		stroker.setJoinStyle(pen.joinStyle())
		shape.addPath(stroker.createStroke(command.path))
	return shape


#============================================
def _target(source: object) -> _Target:
	"""Copy one opaque durable target without exposing structural child kinds."""
	kind = getattr(source, "kind", None)
	if kind != "document_object":
		raise FerrumPlanError("Ferrum render target has an invalid kind")
	document_object_id = getattr(source, "document_object_id", None)
	if type(document_object_id) is not str or not document_object_id:
		raise FerrumPlanError("Ferrum render target document-object identity is invalid")
	return _Target(document_object_id)


#============================================
def _batch_at(batches: tuple[object, ...], batch_index: object) -> object:
	"""Select one plan-owned frozen batch with exact non-bool index semantics."""
	if not isinstance(batch_index, int) or isinstance(batch_index, bool):
		raise FerrumPlanError("Ferrum render batch index must be an integer")
	if batch_index < 0 or batch_index >= len(batches):
		raise FerrumPlanError("Ferrum render batch index is outside its plan")
	batch = batches[batch_index]
	return batch


#============================================
def _point(source: object, label: str) -> _Point:
	"""Validate one finite frozen point without silently coercing its fields."""
	return _Point(_finite(source.x, f"{label} x"), _finite(source.y, f"{label} y"))


#============================================
def _ink_bounds(source: object, engine: object, label: str) -> object:
	"""Validate one exact finite, nonempty Rust-issued label-local ink rectangle."""
	if type(source) is not engine.InkBoundsV1:
		raise FerrumPlanError(f"Ferrum {label} has the wrong DTO type")
	min_x = _finite(source.min_x, f"{label} minimum x")
	min_y = _finite(source.min_y, f"{label} minimum y")
	max_x = _finite(source.max_x, f"{label} maximum x")
	max_y = _finite(source.max_y, f"{label} maximum y")
	if min_x >= max_x or min_y >= max_y:
		raise FerrumPlanError(f"Ferrum {label} must be nonempty")
	return source


#============================================
def _contains_bounds(outer: object, inner: object) -> bool:
	"""Return whether one already-validated rectangle contains another exactly."""
	return (
		outer.min_x <= inner.min_x and outer.min_y <= inner.min_y
		and outer.max_x >= inner.max_x and outer.max_y >= inner.max_y
	)


#============================================
def _translated_point(source: object, anchor: _Point, label: str) -> _Point:
	"""Apply an atom-local anchor once while converting the DTO point."""
	point = _point(source, label)
	return _Point(point.x + anchor.x, point.y + anchor.y)


#============================================
def _finite(value: object, label: str) -> float:
	"""Return one explicit finite numeric input, rejecting bool and coercion."""
	if not isinstance(value, (float, int)) or isinstance(value, bool):
		raise FerrumPlanError(f"Ferrum render {label} must be numeric")
	value = float(value)
	if not math.isfinite(value):
		raise FerrumPlanError(f"Ferrum render {label} must be finite")
	return value


#============================================
def _positive(value: object, label: str) -> float:
	"""Return one explicitly finite positive value."""
	value = _finite(value, label)
	if value <= 0.0:
		raise FerrumPlanError(f"Ferrum render {label} must be positive")
	return value


#============================================
def _optional_positive(value: object, label: str) -> float | None:
	"""Return an optional explicit positive value without inventing a default."""
	if value is None:
		return None
	return _positive(value, label)


#============================================
def _optional_paint(palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		value: object, label: str) -> PySide6.QtGui.QColor | None:
	"""Resolve optional V3 paint without inventing a local color authority."""
	if value is None:
		return None
	try:
		return _paint(palette, value)
	except ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteError as error:
		raise FerrumPlanError(f"Ferrum render {label} is invalid") from error


#============================================
def _revision(value: object) -> None:
	"""Validate the exact u64 revision carried by the authoritative plan."""
	if not isinstance(value, int) or isinstance(value, bool) or value < 0 or value > 2**64 - 1:
		raise FerrumPlanError("Ferrum render revision must be a u64 integer")


#============================================
def _digest(value: object) -> None:
	"""Validate the exact lowercase hexadecimal digest exposed by frozen PyO3 DTOs."""
	if not isinstance(value, str) or len(value) != 64:
		raise FerrumPlanError("Ferrum render provenance digest must be 32-byte hexadecimal")
	if value.lower() != value or any(character not in "0123456789abcdef" for character in value):
		raise FerrumPlanError("Ferrum render provenance digest must be lowercase hexadecimal")


#============================================
def _z(value: object) -> int:
	"""Validate the exact integer z-order grammar."""
	if not isinstance(value, int) or isinstance(value, bool):
		raise FerrumPlanError("Ferrum render z order must be an integer")
	return value


#============================================
def _u32(value: object, label: str) -> int:
	"""Return one exact non-bool unsigned 32-bit integer from the frozen DTO."""
	if type(value) is not int or value < 0 or value >= 2**32:
		raise FerrumPlanError(f"Ferrum render {label} must be a u32 integer")
	return value


#============================================
def _paint(palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		value: object) -> PySide6.QtGui.QColor:
	"""Resolve one tagged V3 paint through the only document display palette."""
	try:
		return palette.resolve_render_paint(value)
	except ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteError as error:
		raise FerrumPlanError("Ferrum render paint is invalid") from error

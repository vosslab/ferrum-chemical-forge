"""Project one exact Ferrum presentation observation into disposable Qt items.

The production boundary begins with a ``SessionDocumentObservationV1`` from the
current ``ferrum_chem`` extension.  It deliberately has no XML, mapping, or
Python-model adapter: Rust has already resolved supported direct-root facts.
"""

# Standard Library
import dataclasses
import math
import re

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.ferrum.engine


_PRESENTATION_STACK_SCHEMA_V1 = "ferrum-presentation-stack-v1"
_RGB24 = re.compile(r"^#[0-9a-f]{6}$")
_DIGEST = re.compile(r"^[0-9a-f]{64}$")
_PROVENANCE = frozenset(("root", "standard", "builtin"))
_RECORD_KINDS = frozenset((
	"arrow", "plus", "text", "polyline", "rectangle", "square", "oval", "circle",
	"polygon",
))
_BOX_KINDS = frozenset(("rectangle", "square", "oval", "circle"))
_U32_RANGE = range(2**32)


#============================================
class PresentationProjectionError(ValueError):
	"""A current Ferrum presentation observation violates the V1 contract."""


@dataclasses.dataclass(frozen=True, slots=True)
class PresentationTarget:
	"""One immutable backend target retained by a disposable Qt item."""

	id: str | None
	projection_key: str
	source_id: str | None
	source_order: int
	record_kind: str


@dataclasses.dataclass(frozen=True, slots=True)
class PresentationIssue:
	"""One backend diagnostic with no corresponding graphics item."""

	target: PresentationTarget
	code: str
	detail: str


@dataclasses.dataclass(frozen=True, slots=True)
class BracketPair:
	"""One exact Rust-observed relationship between two durable polylines."""

	pair_id: str
	member_ids: tuple[str, str]
	style: str
	line_width: float | None
	line_color: str | None


class PolylineProjectionItem(PySide6.QtWidgets.QGraphicsPathItem):
	"""One immutable, selectable, non-movable segmented presentation polyline."""

	#============================================
	def __init__(self, path: PySide6.QtGui.QPainterPath,
			pen: PySide6.QtGui.QPen,
			target: PresentationTarget) -> None:
		"""Create a fully specified item with no document-model callback."""
		super().__init__(path)
		self._target = target
		self.setPen(pen)
		self.setBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
		self.setZValue(float(target.source_order))
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, True,
		)
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable, False,
		)

	#============================================
	@property
	def target(self) -> PresentationTarget:
		"""Return the immutable backend target represented by this item."""
		return self._target

	#============================================
	def dispose(self) -> None:
		"""Provide the shared graphics-retirement callback contract."""


class ShapeProjectionItem(PySide6.QtWidgets.QGraphicsPathItem):
	"""One immutable selectable closed vector shape from Rust geometry."""

	#============================================
	def __init__(self, path: PySide6.QtGui.QPainterPath,
			pen: PySide6.QtGui.QPen, brush: PySide6.QtGui.QBrush,
			target: PresentationTarget) -> None:
		"""Create a closed-shape item without interpreting persistent state."""
		super().__init__(path)
		self._target = target
		self.setPen(pen)
		self.setBrush(brush)
		self.setZValue(float(target.source_order))
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, True,
		)
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable, False,
		)

	#============================================
	@property
	def target(self) -> PresentationTarget:
		"""Return the immutable backend target represented by this item."""
		return self._target

	#============================================
	def dispose(self) -> None:
		"""Provide the shared graphics-retirement callback contract."""


class ArrowProjectionItem(PySide6.QtWidgets.QGraphicsItem):
	"""One immutable normal arrow whose axis and heads are fully Rust-issued."""

	#============================================
	def __init__(self, axis_path: PySide6.QtGui.QPainterPath,
			head_path: PySide6.QtGui.QPainterPath, pen: PySide6.QtGui.QPen,
			target: PresentationTarget) -> None:
		"""Cache complete display and selection geometry without arrow defaults."""
		super().__init__()
		self._axis_path = axis_path
		self._head_path = head_path
		self._pen = pen
		self._target = target
		stroker = PySide6.QtGui.QPainterPathStroker()
		stroker.setWidth(max(8.0, pen.widthF() + 6.0))
		self._interaction_path = stroker.createStroke(axis_path)
		self._interaction_path.addPath(head_path)
		self._bounds = self._interaction_path.boundingRect()
		self.setZValue(float(target.source_order))
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, True,
		)
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable, False,
		)

	#============================================
	@property
	def target(self) -> PresentationTarget:
		"""Return the immutable backend target represented by this item."""
		return self._target

	#============================================
	@property
	def axis_path(self) -> PySide6.QtGui.QPainterPath:
		"""Return a copy of the exact backend-issued shortened axis path."""
		return PySide6.QtGui.QPainterPath(self._axis_path)

	#============================================
	@property
	def head_path(self) -> PySide6.QtGui.QPainterPath:
		"""Return a copy of the exact backend-issued filled head geometry."""
		return PySide6.QtGui.QPainterPath(self._head_path)

	#============================================
	@property
	def pen(self) -> PySide6.QtGui.QPen:
		"""Return a copy of the explicit backend-issued arrow stroke."""
		return PySide6.QtGui.QPen(self._pen)

	#============================================
	def boundingRect(self) -> PySide6.QtCore.QRectF:
		"""Return cached interaction bounds for the complete arrow."""
		return PySide6.QtCore.QRectF(self._bounds)

	#============================================
	def shape(self) -> PySide6.QtGui.QPainterPath:
		"""Return the cached selectable axis-and-head geometry."""
		return PySide6.QtGui.QPainterPath(self._interaction_path)

	#============================================
	def paint(self, painter: PySide6.QtGui.QPainter,
			option: PySide6.QtWidgets.QStyleOptionGraphicsItem,
			widget: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Paint cached backend geometry and a Qt-local selection decoration."""
		del widget
		if option.state & PySide6.QtWidgets.QStyle.StateFlag.State_Selected:
			color = PySide6.QtWidgets.QApplication.palette().highlight().color()
			color.setAlpha(80)
			highlight = PySide6.QtGui.QPen(color, max(8.0, self._pen.widthF() + 6.0))
			painter.setPen(highlight)
			painter.setBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
			painter.drawPath(self._axis_path)
		painter.setPen(self._pen)
		painter.setBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
		painter.drawPath(self._axis_path)
		painter.setPen(PySide6.QtCore.Qt.PenStyle.NoPen)
		painter.setBrush(PySide6.QtGui.QBrush(self._pen.color()))
		painter.drawPath(self._head_path)

	#============================================
	def dispose(self) -> None:
		"""Provide the shared graphics-retirement callback contract."""


PresentationProjectionItem = ArrowProjectionItem | PolylineProjectionItem | ShapeProjectionItem


@dataclasses.dataclass(slots=True)
class FerrumPresentationProjection:
	"""A complete detached projection from one authoritative observation."""

	revision: int
	digest: str
	roots: tuple[PresentationProjectionItem, ...]
	items: tuple[PresentationProjectionItem, ...]
	durable_items: dict[str, PresentationProjectionItem]
	local_items: dict[str, PresentationProjectionItem]
	bracket_pairs: tuple[BracketPair, ...]
	issues: tuple[PresentationIssue, ...]

	#============================================
	def selected_targets(
			self, scene: PySide6.QtWidgets.QGraphicsScene | None,
			) -> tuple[PresentationTarget, ...]:
		"""Return selected local targets without promoting id-less keys to IDs."""
		selected = ferrum_qt.canvas.graphics_retirement.selected_items_from_captured_scene(scene)
		return tuple(item.target for item in self.items if item in selected and item.isSelected())

	#============================================
	def select_durable(self, identifiers: tuple[str, ...]) -> None:
		"""Select only durable backend IDs in this projection's current item map."""
		for item in self.items:
			item.setSelected(item.target.id in identifiers)

	#============================================
	def dispose_detached(self) -> None:
		"""Release an uninstalled projection through the established reaper."""
		coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
		coordinator.retire_detached_projection_items(list(self.roots))
		coordinator.raise_if_callback_failed("Ferrum presentation disposal failed")


#============================================
def build_presentation_projection(observation: object) -> FerrumPresentationProjection:
	"""Build a detached projection from one exact PyO3 document observation."""
	stack, revision, digest, extension = _validate_observation(observation)
	items: list[PresentationProjectionItem] = []
	durable_items: dict[str, PresentationProjectionItem] = {}
	local_items: dict[str, PresentationProjectionItem] = {}
	last_source_order = -1
	try:
		for root in stack.roots:
			if getattr(root, "kind", None) == "plus":
				target = _validate_source_plus(root, extension)
				item = None
			elif getattr(root, "kind", None) == "text":
				target = _validate_source_text(root, extension)
				item = None
			else:
				item = _build_root(root, extension)
				target = item.target
			if target.source_order <= last_source_order:
				raise PresentationProjectionError("presentation roots are not source ordered")
			last_source_order = target.source_order
			if item is None:
				continue
			if target.id is None:
				if target.projection_key in local_items:
					raise PresentationProjectionError("duplicate local projection key")
				local_items[target.projection_key] = item
			else:
				if target.id in durable_items:
					raise PresentationProjectionError("duplicate durable presentation ID")
				durable_items[target.id] = item
			items.append(item)
		bracket_pairs = _bracket_pairs(stack.bracket_pairs, extension)
		_validate_round_bracket_roots(stack, bracket_pairs)
		issues = tuple(_issue(issue, extension) for issue in stack.issues)
	except (AttributeError, TypeError, ValueError, PresentationProjectionError) as exc:
		_retire_failed_detached(items)
		if isinstance(exc, PresentationProjectionError):
			raise
		raise PresentationProjectionError("invalid frozen presentation DTO") from exc
	roots = tuple(items)
	return FerrumPresentationProjection(
		revision, digest, roots, tuple(items), durable_items, local_items,
		bracket_pairs, issues,
	)


#============================================
def _validate_observation(observation: object) -> tuple[object, int, str, object]:
	"""Authenticate one observation and its same-snapshot presentation facts."""
	extension = _ferrum_chem()
	if type(observation) is not extension.SessionDocumentObservationV1:
		raise PresentationProjectionError(
			"presentation observation must be engine.SessionDocumentObservationV1",
		)
	snapshot = observation.snapshot
	projection = observation.projection
	if type(snapshot) is not extension.DocumentSnapshot:
		raise PresentationProjectionError("presentation observation snapshot has the wrong DTO type")
	if type(projection) is not extension.DocumentProjectionV1:
		raise PresentationProjectionError("presentation observation projection has the wrong DTO type")
	revision = _revision(snapshot.revision)
	digest = _digest(snapshot.digest)
	if _revision(projection.revision) != revision or _digest(projection.digest) != digest:
		raise PresentationProjectionError("document projection provenance differs from snapshot")
	stack = projection.presentation_stack
	if type(stack) is not extension.PresentationStackProjectionV1:
		raise PresentationProjectionError("presentation stack has the wrong DTO type")
	if stack.schema != _PRESENTATION_STACK_SCHEMA_V1:
		raise PresentationProjectionError("unknown presentation stack schema")
	if _revision(stack.revision) != revision or _digest(stack.digest) != digest:
		raise PresentationProjectionError("presentation stack provenance differs from snapshot")
	if (
		type(stack.roots) is not list
		or type(stack.bracket_pairs) is not list
		or type(stack.issues) is not list
	):
		raise PresentationProjectionError("presentation stack sequences have the wrong DTO type")
	return stack, revision, digest, extension


#============================================
def _bracket_pairs(values: list[object], extension: object) -> tuple[BracketPair, ...]:
	"""Copy exact derived pair facts without inferring relationships from geometry."""
	result = []
	seen_pairs = set()
	seen_members = set()
	for value in values:
		if type(value) is not extension.BracketPairProjectionV1:
			raise PresentationProjectionError("bracket pair has the wrong DTO type")
		if type(value.pair_id) is not str or not value.pair_id:
			raise PresentationProjectionError("bracket pair ID is invalid")
		if (
			type(value.member_ids) is not list
			or len(value.member_ids) != 2
			or any(type(identifier) is not str or not identifier
					for identifier in value.member_ids)
			or value.member_ids[0] != value.pair_id
			or value.member_ids[0] == value.member_ids[1]
		):
			raise PresentationProjectionError("bracket member identities are invalid")
		if value.style is extension.DocumentBracketStyleV1.rectangular:
			style = "rectangular"
		elif value.style is extension.DocumentBracketStyleV1.round:
			style = "round"
		else:
			raise PresentationProjectionError("bracket style is invalid")
		if (
			value.line_width is not None
			and (
				type(value.line_width) is not float
				or not math.isfinite(value.line_width)
				or value.line_width <= 0.0
			)
		):
			raise PresentationProjectionError("bracket common width is invalid")
		if (
			value.line_color is not None
			and (
				type(value.line_color) is not str
				or _RGB24.fullmatch(value.line_color) is None
			)
		):
			raise PresentationProjectionError("bracket common color is invalid")
		members = tuple(value.member_ids)
		if value.pair_id in seen_pairs or seen_members.intersection(members):
			raise PresentationProjectionError("bracket relationship is duplicated")
		seen_pairs.add(value.pair_id)
		seen_members.update(members)
		result.append(BracketPair(
			value.pair_id, members, style, value.line_width, value.line_color,
		))
	return tuple(result)


#============================================
def _validate_round_bracket_roots(stack: object,
		pairs: tuple[BracketPair, ...]) -> None:
	"""Require every round pair member to have exactly one round-root payload."""
	expected = tuple(
		identifier for pair in pairs if pair.style == "round"
		for identifier in pair.member_ids
	)
	actual = tuple(
		root.polyline.target.source_id for root in stack.roots
		if root.kind == "round_bracket"
	)
	if (
			len(set(expected)) != len(expected)
			or len(set(actual)) != len(actual)
			or set(expected) != set(actual)
		):
		raise PresentationProjectionError(
			"round bracket roots differ from their projected pair members",
		)


#============================================
def _build_root(root: object, extension: object) -> PresentationProjectionItem:
	"""Dispatch one exact closed root kind without a structural-model fallback."""
	if type(root) is not extension.PresentationRootProjectionV1:
		raise PresentationProjectionError("presentation root has the wrong DTO type")
	if root.kind == "arrow":
		return _build_arrow(root, extension)
	if root.kind == "polyline":
		return _build_polyline(root, extension)
	if root.kind == "wavy":
		return _build_wavy(root, extension)
	if root.kind == "round_bracket":
		return _build_round_bracket(root, extension)
	if root.kind in _BOX_KINDS:
		return _build_box_shape(root, extension)
	if root.kind == "polygon":
		return _build_polygon(root, extension)
	raise PresentationProjectionError("unsupported presentation root kind")


#============================================
def _validate_source_plus(root: object, extension: object) -> PresentationTarget:
	"""Validate source facts while reserving all glyph layout for the render API."""
	if type(root) is not extension.PresentationRootProjectionV1:
		raise PresentationProjectionError("presentation root has the wrong DTO type")
	if any(value is not None for value in (
		root.arrow, root.text, root.polyline, root.shape, root.polygon,
	)):
		raise PresentationProjectionError("plus root carries another root payload")
	plus = root.plus
	if type(plus) is not extension.PlusProjectionV1:
		raise PresentationProjectionError("plus payload has the wrong DTO type")
	target = _target(plus.target, extension, "plus")
	_point(plus.anchor, extension)
	font = plus.font
	if type(font) is not extension.PresentationFontV1:
		raise PresentationProjectionError("plus font has the wrong DTO type")
	if font.family is not None and (type(font.family) is not str or not font.family):
		raise PresentationProjectionError("plus font family is invalid")
	if font.family_provenance not in _PROVENANCE:
		raise PresentationProjectionError("plus font-family provenance is unknown")
	if type(font.size) not in (int, float) or not math.isfinite(font.size) or font.size <= 0.0:
		raise PresentationProjectionError("plus font size must be finite and positive")
	if font.size_provenance not in _PROVENANCE or font.color_provenance not in _PROVENANCE:
		raise PresentationProjectionError("plus font provenance is unknown")
	if type(font.color) is not str or _RGB24.fullmatch(font.color) is None:
		raise PresentationProjectionError("plus font color must be explicit lowercase #rrggbb")
	_brush(plus.background, extension)
	return target


#============================================
def _validate_source_text(root: object, extension: object) -> PresentationTarget:
	"""Validate Text source facts while reserving all glyph layout for the render API."""
	if type(root) is not extension.PresentationRootProjectionV1:
		raise PresentationProjectionError("Text root has the wrong DTO type")
	if any(value is not None for value in (
		root.arrow, root.plus, root.polyline, root.shape, root.polygon,
	)):
		raise PresentationProjectionError("Text root carries another root payload")
	text = root.text
	if type(text) is not extension.TextProjectionV1:
		raise PresentationProjectionError("Text payload has the wrong DTO type")
	target = _target(text.target, extension, "text")
	_point(text.anchor, extension)
	if type(text.runs) is not tuple or not text.runs:
		raise PresentationProjectionError("Text source runs must be a frozen nonempty tuple")
	for run in text.runs:
		if type(run) is not extension.PresentationTextRunV1:
			raise PresentationProjectionError("Text source run has the wrong DTO type")
		if type(run.text) is not str or not run.text or type(run.styles) is not tuple:
			raise PresentationProjectionError("Text source run is invalid")
		if any(style not in {"bold", "italic", "subscript", "superscript"}
				for style in run.styles):
			raise PresentationProjectionError("Text source run has an unknown style")
	font = text.font
	if type(font) is not extension.PresentationTextFontV1:
		raise PresentationProjectionError("Text font has the wrong DTO type")
	if font.family is not None and (type(font.family) is not str or not font.family):
		raise PresentationProjectionError("Text font family is invalid")
	if font.family_provenance not in _PROVENANCE:
		raise PresentationProjectionError("Text font-family provenance is unknown")
	if type(font.size) not in (int, float) or not math.isfinite(font.size) or font.size <= 0.0:
		raise PresentationProjectionError("Text font size must be finite and positive")
	if font.size_provenance not in _PROVENANCE or font.color_provenance not in _PROVENANCE:
		raise PresentationProjectionError("Text font provenance is unknown")
	if type(font.color) is not str or _RGB24.fullmatch(font.color) is None:
		raise PresentationProjectionError("Text font color must be explicit lowercase #rrggbb")
	_brush(text.background, extension)
	return target


#============================================
def _build_arrow(root: object, extension: object) -> ArrowProjectionItem:
	"""Build one Arrow directly from its authenticated Rust geometry variant."""
	if any(value is not None for value in (
		root.polyline, root.shape, root.polygon, root.plus, root.text,
	)):
		raise PresentationProjectionError("arrow root carries another root payload")
	arrow = root.arrow
	if type(arrow) is not extension.ArrowProjectionV1:
		raise PresentationProjectionError("arrow payload has the wrong DTO type")
	target = _target(arrow.target, extension, "arrow")
	source_points = _arrow_points(arrow.source_path, extension, "source")
	geometry = arrow.geometry
	if type(geometry) is not extension.ArrowDisplayGeometryV1:
		raise PresentationProjectionError("arrow geometry has the wrong DTO type")
	if geometry.kind == "normal":
		if any(value is not None for value in (
			geometry.equilibrium, geometry.curved_equilibrium, geometry.curved_terminal,
		)) or \
				type(geometry.normal) is not extension.NormalArrowDisplayGeometryV1:
			raise PresentationProjectionError("normal arrow geometry payload is invalid")
		normal = geometry.normal
		axis_points = _arrow_points(normal.axis_path, extension, "axis")
		if len(source_points) != len(axis_points):
			raise PresentationProjectionError("arrow source and axis path lengths differ")
		if type(normal.start_head) is not bool or type(normal.end_head) is not bool:
			raise PresentationProjectionError("arrow head flags are not exact booleans")
		shape = normal.head_shape
		if type(shape) is not extension.ArrowHeadShapeV1:
			raise PresentationProjectionError("arrow head shape has the wrong DTO type")
		shape_values = (shape.line_inset, shape.total_length, shape.half_width)
		if any(type(value) not in (int, float) or not math.isfinite(value) for value in shape_values):
			raise PresentationProjectionError("arrow head shape is not finite")
		if shape.line_inset <= 0.0 or shape.total_length < shape.line_inset or shape.half_width <= 0.0:
			raise PresentationProjectionError("arrow head shape is invalid")
		heads = normal.heads
		expected_positions = []
		if normal.start_head:
			expected_positions.append("start")
		if normal.end_head:
			expected_positions.append("end")
		if [head.position for head in heads] != expected_positions:
			raise PresentationProjectionError("arrow head sequence differs from its flags")
	elif geometry.kind == "equilibrium":
		if any(value is not None for value in (
			geometry.normal, geometry.curved_equilibrium, geometry.curved_terminal,
		)) or \
				type(geometry.equilibrium) is not extension.EquilibriumArrowDisplayGeometryV1:
			raise PresentationProjectionError("equilibrium arrow geometry payload is invalid")
		equilibrium = geometry.equilibrium
		if type(equilibrium.axes) is not list or len(equilibrium.axes) != 2:
			raise PresentationProjectionError("equilibrium arrow requires two issued axes")
		axis_points = tuple(point for axis in equilibrium.axes
				for point in _arrow_points(axis, extension, "equilibrium axis"))
		heads = equilibrium.heads
		if [head.position for head in heads] != ["start", "end"]:
			raise PresentationProjectionError("equilibrium arrow requires opposing issued heads")
	elif geometry.kind == "curved_equilibrium":
		curved_equilibrium = geometry.curved_equilibrium
		if any(value is not None for value in (
				geometry.normal, geometry.equilibrium, geometry.curved_terminal,
			)) or type(curved_equilibrium) is not extension.CurvedEquilibriumArrowDisplayGeometryV1:
			raise PresentationProjectionError("curved equilibrium arrow geometry payload is invalid")
		if len(source_points) != 3:
			raise PresentationProjectionError("curved equilibrium arrow source path requires three points")
		if type(curved_equilibrium.axes) is not list or len(curved_equilibrium.axes) != 2:
			raise PresentationProjectionError("curved equilibrium arrow requires two issued cubic axes")
		axis_points = tuple(point for axis in curved_equilibrium.axes
				for point in _arrow_points(axis, extension, "curved equilibrium arrow axis"))
		if any(len(_arrow_points(axis, extension, "curved equilibrium arrow axis")) != 4
				for axis in curved_equilibrium.axes):
			raise PresentationProjectionError("curved equilibrium arrow axes require one cubic segment each")
		heads = curved_equilibrium.heads
		if [head.position for head in heads] != ["start", "end"]:
			raise PresentationProjectionError("curved equilibrium arrow requires opposing issued heads")
	elif geometry.kind == "curved_terminal":
		terminal = geometry.curved_terminal
		if geometry.normal is not None or geometry.equilibrium is not None or \
				geometry.curved_equilibrium is not None or \
				type(terminal) is not extension.CurvedTerminalArrowDisplayGeometryV1:
			raise PresentationProjectionError("curved terminal arrow geometry payload is invalid")
		if type(terminal.kind) is not extension.CurvedTerminalArrowDisplayKindV1 or \
				terminal.kind not in (
					extension.CurvedTerminalArrowDisplayKindV1.electron,
					extension.CurvedTerminalArrowDisplayKindV1.retro,
					extension.CurvedTerminalArrowDisplayKindV1.curved_normal_reaction,
				):
			raise PresentationProjectionError("curved terminal arrow kind is invalid")
		if len(source_points) != 3:
			raise PresentationProjectionError("curved terminal arrow source path requires three points")
		axis_points = _arrow_points(terminal.axis_path, extension, "curved terminal arrow axis")
		if len(axis_points) != 4:
			raise PresentationProjectionError("curved terminal arrow axis requires one issued cubic segment")
		if type(terminal.head) is not extension.ArrowHeadV1 or terminal.head.position != "end":
			raise PresentationProjectionError("curved terminal arrow requires one terminal issued head")
		heads = [terminal.head]
	else:
		raise PresentationProjectionError("arrow geometry kind is unknown")
	if type(heads) is not list or any(type(head) is not extension.ArrowHeadV1 for head in heads):
		raise PresentationProjectionError("arrow head sequence has the wrong DTO type")
	head_path = PySide6.QtGui.QPainterPath()
	for head in heads:
		if type(head.points) is not list:
			raise PresentationProjectionError("arrow head points have the wrong DTO type")
		if len(head.points) != 4:
			raise PresentationProjectionError("normal-arrow head requires four points")
		points = tuple(_point(point, extension) for point in head.points)
		head_path.moveTo(points[0])
		for point in points[1:]:
			head_path.lineTo(point)
		head_path.closeSubpath()
	axis_path = PySide6.QtGui.QPainterPath()
	if geometry.kind == "normal":
		axis_path.moveTo(axis_points[0])
		for point in axis_points[1:]:
			axis_path.lineTo(point)
	elif geometry.kind == "equilibrium":
		for axis in geometry.equilibrium.axes:
			points = _arrow_points(axis, extension, "equilibrium axis")
			axis_path.moveTo(points[0])
			for point in points[1:]:
				axis_path.lineTo(point)
	elif geometry.kind == "curved_equilibrium":
		for axis in geometry.curved_equilibrium.axes:
			points = _arrow_points(axis, extension, "curved equilibrium arrow axis")
			axis_path.moveTo(points[0])
			axis_path.cubicTo(points[1], points[2], points[3])
	else:
		axis_path.moveTo(axis_points[0])
		axis_path.cubicTo(axis_points[1], axis_points[2], axis_points[3])
	return ArrowProjectionItem(axis_path, head_path, _pen(arrow.stroke, extension), target)


#============================================
def _arrow_points(value: object, extension: object,
		description: str) -> tuple[PySide6.QtCore.QPointF, ...]:
	"""Validate one exact Rust-issued path payload for any ArrowPathV1 family."""
	if type(value) is not extension.ArrowPathV1 or type(value.points) is not list:
		raise PresentationProjectionError(f"arrow {description} path has the wrong DTO type")
	if len(value.points) < 2:
		raise PresentationProjectionError(f"arrow {description} path requires two points")
	return tuple(_point(point, extension) for point in value.points)


#============================================
def _build_polyline(root: object, extension: object) -> PolylineProjectionItem:
	"""Validate and build one segmented Rust-projected polyline without defaults."""
	return _build_polyline_path(root, extension, "polyline")


#============================================
def _build_wavy(root: object, extension: object) -> PolylineProjectionItem:
	"""Build one Wavy item from its exact authored Rust-projected point path."""
	return _build_polyline_path(root, extension, "Wavy")


#============================================
def _build_round_bracket(root: object, extension: object) -> PolylineProjectionItem:
	"""Build one cubic side only from a Rust-issued valid round bracket member."""
	return _build_polyline_path(root, extension, "round bracket", exact_points=4)


#============================================
def _build_polyline_path(
		root: object, extension: object, description: str,
		*, exact_points: int | None = None,
		) -> PolylineProjectionItem:
	"""Build one closed polyline-family path without interpreting persistent data."""
	if any(value is not None for value in (
		root.arrow, root.shape, root.polygon, root.plus, root.text,
	)):
		raise PresentationProjectionError(f"{description} root carries another root payload")
	polyline = root.polyline
	if type(polyline) is not extension.PolylineProjectionV1:
		raise PresentationProjectionError(f"{description} payload has the wrong DTO type")
	target = _target(polyline.target, extension, "polyline")
	path = polyline.path
	if type(path) is not extension.PolylinePathV1 or type(path.points) is not list:
		raise PresentationProjectionError(f"{description} path has the wrong DTO type")
	if len(path.points) < 2:
		raise PresentationProjectionError(f"{description} path requires at least two points")
	if exact_points is not None and len(path.points) != exact_points:
		raise PresentationProjectionError(
			f"{description} path requires exactly {exact_points} points",
		)
	points = tuple(_point(point, extension) for point in path.points)
	if description == "round bracket":
		paint_path = _replay_presentation_path(
			ferrum_qt.ferrum.engine.lower_round_bracket_presentation_path_v1(root),
			extension,
		)
	else:
		paint_path = _polyline_path(points)
	return PolylineProjectionItem(
		paint_path, _pen(polyline.stroke, extension), target,
	)


#============================================
def _polyline_path(points: tuple[PySide6.QtCore.QPointF, ...]) -> PySide6.QtGui.QPainterPath:
	"""Replay one ordered sequence of already-issued straight path points."""
	path = PySide6.QtGui.QPainterPath(points[0])
	for point in points[1:]:
		path.lineTo(point)
	return path


#============================================
def _replay_presentation_path(value: object, extension: object) -> PySide6.QtGui.QPainterPath:
	"""Replay only the frozen MoveTo, LineTo, and CubicTo command grammar."""
	if type(value) is not extension.PresentationPathV1:
		raise PresentationProjectionError("presentation path has the wrong DTO type")
	if value.kind != "authored_spline" or type(value.commands) is not tuple:
		raise PresentationProjectionError("presentation path has an invalid replay contract")
	if not value.commands:
		raise PresentationProjectionError("presentation path has no commands")
	path = PySide6.QtGui.QPainterPath()
	for index, command in enumerate(value.commands):
		if type(command) is not extension.PresentationPathCommandV1:
			raise PresentationProjectionError("presentation path command has the wrong DTO type")
		if command.kind == "move_to":
			if index != 0 or command.control_1 is not None or command.control_2 is not None:
				raise PresentationProjectionError("presentation path move command is malformed")
			path.moveTo(_point(command.point, extension))
		elif command.kind == "line_to":
			if index == 0 or command.control_1 is not None or command.control_2 is not None:
				raise PresentationProjectionError("presentation path line command is malformed")
			path.lineTo(_point(command.point, extension))
		elif command.kind == "cubic_to":
			if index == 0:
				raise PresentationProjectionError("presentation path cubic command lacks a start")
			path.cubicTo(
				_point(command.control_1, extension), _point(command.control_2, extension),
				_point(command.point, extension),
			)
		else:
			raise PresentationProjectionError("presentation path command is unsupported")
	return path


#============================================
def _build_box_shape(root: object, extension: object) -> ShapeProjectionItem:
	"""Build one exact bound-based rectangle or ellipse family item."""
	if any(value is not None for value in (
		root.arrow, root.polyline, root.polygon, root.plus, root.text,
	)):
		raise PresentationProjectionError("box-shape root carries another root payload")
	shape = root.shape
	if type(shape) is not extension.BoxShapeProjectionV1:
		raise PresentationProjectionError("box-shape payload has the wrong DTO type")
	target = _target(shape.target, extension, root.kind)
	bounds = shape.bounds
	if type(bounds) is not extension.PresentationBoundsV1:
		raise PresentationProjectionError("shape bounds have the wrong DTO type")
	values = (bounds.left, bounds.top, bounds.right, bounds.bottom)
	if any(type(value) not in (int, float) or not math.isfinite(value) for value in values):
		raise PresentationProjectionError("shape bounds are not finite")
	if bounds.left > bounds.right or bounds.top > bounds.bottom:
		raise PresentationProjectionError("shape bounds are not normalized")
	rectangle = PySide6.QtCore.QRectF(
		float(bounds.left), float(bounds.top),
		float(bounds.right - bounds.left), float(bounds.bottom - bounds.top),
	)
	path = PySide6.QtGui.QPainterPath()
	if root.kind in ("rectangle", "square"):
		path.addRect(rectangle)
	else:
		path.addEllipse(rectangle)
	return ShapeProjectionItem(
		path, _pen(shape.stroke, extension), _brush(shape.fill, extension), target,
	)


#============================================
def _build_polygon(root: object, extension: object) -> ShapeProjectionItem:
	"""Build one explicitly closed polygon from every ordered Rust point."""
	if any(value is not None for value in (
		root.arrow, root.polyline, root.shape, root.plus, root.text,
	)):
		raise PresentationProjectionError("polygon root carries another root payload")
	polygon = root.polygon
	if type(polygon) is not extension.PolygonProjectionV1:
		raise PresentationProjectionError("polygon payload has the wrong DTO type")
	target = _target(polygon.target, extension, "polygon")
	path_value = polygon.path
	if type(path_value) is not extension.PolygonPathV1 or type(path_value.points) is not list:
		raise PresentationProjectionError("polygon path has the wrong DTO type")
	if len(path_value.points) < 3:
		raise PresentationProjectionError("polygon requires at least three points")
	points = tuple(_point(point, extension) for point in path_value.points)
	path = PySide6.QtGui.QPainterPath(points[0])
	for point in points[1:]:
		path.lineTo(point)
	path.closeSubpath()
	return ShapeProjectionItem(
		path, _pen(polygon.stroke, extension), _brush(polygon.fill, extension), target,
	)


#============================================
def _pen(value: object, extension: object) -> PySide6.QtGui.QPen:
	"""Copy one complete backend-resolved noncosmetic stroke."""
	if type(value) is not extension.PresentationStrokeV1:
		raise PresentationProjectionError("presentation stroke has the wrong DTO type")
	if type(value.color) is not str or _RGB24.fullmatch(value.color) is None:
		raise PresentationProjectionError("presentation color must be explicit lowercase #rrggbb")
	if type(value.width) not in (int, float) or not math.isfinite(value.width) or value.width <= 0.0:
		raise PresentationProjectionError("presentation width must be finite and positive")
	if value.color_provenance not in _PROVENANCE or value.width_provenance not in _PROVENANCE:
		raise PresentationProjectionError("presentation stroke provenance is unknown")
	pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor(value.color), float(value.width))
	pen.setCosmetic(False)
	return pen


#============================================
def _brush(value: object, extension: object) -> PySide6.QtGui.QBrush:
	"""Copy one explicit backend-resolved fill without a palette fallback."""
	if type(value) is not extension.PresentationFillV1:
		raise PresentationProjectionError("presentation fill has the wrong DTO type")
	if value.color_provenance not in _PROVENANCE:
		raise PresentationProjectionError("presentation fill provenance is unknown")
	if value.color is None:
		return PySide6.QtGui.QBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
	if type(value.color) is not str or _RGB24.fullmatch(value.color) is None:
		raise PresentationProjectionError("presentation fill must be lowercase #rrggbb or absent")
	return PySide6.QtGui.QBrush(PySide6.QtGui.QColor(value.color))


#============================================
def _target(value: object, extension: object,
		expected_kind: str | None = None) -> PresentationTarget:
	"""Copy one authenticated durable-or-local target into scene-local state."""
	if type(value) is not extension.PresentationTargetV1:
		raise PresentationProjectionError("presentation target has the wrong DTO type")
	if value.id is not None and (type(value.id) is not str or not value.id):
		raise PresentationProjectionError("presentation ID is invalid")
	if type(value.projection_key) is not str or not value.projection_key:
		raise PresentationProjectionError("presentation projection key is invalid")
	if value.source_id is not None and (type(value.source_id) is not str or not value.source_id):
		raise PresentationProjectionError("presentation source ID is invalid")
	if (value.id is None) != (value.source_id is None):
		raise PresentationProjectionError("presentation durable target provenance is invalid")
	if type(value.source_order) is not int or value.source_order not in _U32_RANGE:
		raise PresentationProjectionError("presentation source order is invalid")
	if type(value.record_kind) is not str or value.record_kind not in _RECORD_KINDS:
		raise PresentationProjectionError("presentation record kind is invalid")
	if expected_kind is not None and value.record_kind != expected_kind:
		raise PresentationProjectionError("presentation root and target kinds differ")
	return PresentationTarget(
		value.id, value.projection_key, value.source_id, value.source_order, value.record_kind,
	)


#============================================
def _point(value: object, extension: object) -> PySide6.QtCore.QPointF:
	"""Copy one finite Rust point into direct Qt scene coordinates."""
	if type(value) is not extension.Point3V1:
		raise PresentationProjectionError("presentation point has the wrong DTO type")
	coordinates = (value.x, value.y, value.z)
	if any(
			type(coordinate) not in (int, float) or not math.isfinite(coordinate)
			for coordinate in coordinates
		):
		raise PresentationProjectionError("presentation point is not finite")
	return PySide6.QtCore.QPointF(float(value.x), float(value.y))


#============================================
def _issue(value: object, extension: object) -> PresentationIssue:
	"""Copy one backend issue into visible state without creating an item."""
	if type(value) is not extension.PresentationProjectionIssueV1:
		raise PresentationProjectionError("presentation issue has the wrong DTO type")
	if type(value.code) is not str or not value.code or type(value.detail) is not str:
		raise PresentationProjectionError("presentation issue is invalid")
	return PresentationIssue(_target(value.target, extension), value.code, value.detail)


#============================================
def _revision(value: object) -> int:
	"""Validate one exact u64 revision copied from Rust."""
	if type(value) is not int or value < 0 or value >= 2**64:
		raise PresentationProjectionError("presentation revision is invalid")
	return value


#============================================
def _digest(value: object) -> str:
	"""Validate one exact lowercase structural digest copied from Rust."""
	if type(value) is not str or _DIGEST.fullmatch(value) is None:
		raise PresentationProjectionError("presentation digest is invalid")
	return value


#============================================
def _retire_failed_detached(items: list[PresentationProjectionItem]) -> None:
	"""Dispose every partially prepared item without touching a live projection."""
	if not items:
		return
	coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
	coordinator.retire_detached_projection_items(items)


#============================================
def _ferrum_chem() -> object:
	"""Load the installed direct extension only at the production boundary."""
	try:
		import ferrum_qt.ferrum.engine as engine
	except ImportError as exc:
		raise PresentationProjectionError("Ferrum presentation binding is unavailable") from exc
	return engine

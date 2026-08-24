"""Disposable Qt item for one API-issued fixed-content plus render."""

# Standard Library
import math
import re

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_telex
import ferrum_qt.canvas.ferrum_presentation_target
import ferrum_qt.canvas.telex_glyph_outline


_FACE = "ferrum-telex-regular-v1"
_RGB24 = re.compile(r"^[0-9a-f]{6}$")
_PADDING = 1.0


#============================================
class FerrumPlusItemError(ValueError):
	"""A frozen plus render cannot be painted without frontend interpretation."""


class FerrumPlusItem(PySide6.QtWidgets.QGraphicsObject):
	"""Selectable plus using only verified glyph IDs, origins, bounds, and paints."""

	#============================================
	def __init__(self, plus: object, telex_resource: object,
			parent: PySide6.QtWidgets.QGraphicsItem | None = None) -> None:
		"""Authenticate one extension-owned plus render and cache its complete paths."""
		super().__init__(parent)
		extension = _ferrum_chem()
		if type(plus) is not extension.DocumentPlusRenderV1:
			raise FerrumPlusItemError("plus render must be engine.DocumentPlusRenderV1")
		telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex_resource)
		self._initialize(plus, extension, telex)

	#============================================
	@classmethod
	def _from_observation(cls, plus: object,
			telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex) -> "FerrumPlusItem":
		"""Reuse a controller-authenticated Telex face for one exact runtime DTO."""
		item = cls.__new__(cls)
		PySide6.QtWidgets.QGraphicsObject.__init__(item)
		extension = _ferrum_chem()
		if type(plus) is not extension.DocumentPlusRenderV1:
			raise FerrumPlusItemError("plus render must be engine.DocumentPlusRenderV1")
		if not isinstance(telex, ferrum_qt.canvas.ferrum_telex.FerrumTelex):
			raise FerrumPlusItemError("plus render requires verified Telex bytes")
		item._initialize(plus, extension, telex)
		return item

	#============================================
	def _initialize(self, plus: object, extension: object,
			telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex) -> None:
		"""Copy a verified plus into immutable Qt-local paths and paint values."""
		self._target = _target(plus.target, extension)
		anchor = _point(plus.anchor, extension, "plus anchor")
		operation = plus.operation
		if type(operation) is not extension.TextOpV1 or operation.face != _FACE:
			raise FerrumPlusItemError("plus render requires the verified Telex text operation")
		if operation.z != 20 or operation.paint is None:
			raise FerrumPlusItemError("plus render text operation has invalid paint order")
		if type(operation.runs) is not tuple or len(operation.runs) != 1:
			raise FerrumPlusItemError("plus render requires one frozen text run")
		run = operation.runs[0]
		if type(run) is not extension.TextRunV1 or run.text != "+" or run.script != "baseline":
			raise FerrumPlusItemError("plus render requires the closed baseline plus run")
		if type(run.glyphs) is not tuple or len(run.glyphs) != 1:
			raise FerrumPlusItemError("plus render requires one frozen glyph")
		if type(run.glyphs[0]) is not extension.GlyphPlacementV1:
			raise FerrumPlusItemError("plus glyph has the wrong DTO type")
		font = telex.raw_font(_positive(operation.size, "plus text size"))
		try:
			self._glyph_path = ferrum_qt.canvas.telex_glyph_outline.path_from_runs(
				operation.runs, _point(operation.origin, extension, "plus text origin"), font,
			)
		except ferrum_qt.canvas.telex_glyph_outline.TelexGlyphOutlineError as exc:
			raise FerrumPlusItemError(str(exc)) from exc
		self._foreground = PySide6.QtGui.QBrush(_color(operation.paint, "plus foreground"))
		bounds = plus.bounds
		if type(bounds) is not extension.PresentationTextBoundsV1:
			raise FerrumPlusItemError("plus ink bounds have the wrong DTO type")
		left = _finite(bounds.left, "plus left bound")
		top = _finite(bounds.top, "plus top bound")
		right = _finite(bounds.right, "plus right bound")
		bottom = _finite(bounds.bottom, "plus bottom bound")
		if left >= right or top >= bottom or not (left <= 0.0 <= right and top <= 0.0 <= bottom):
			raise FerrumPlusItemError("plus ink bounds are invalid")
		self._background_path = PySide6.QtGui.QPainterPath()
		self._background = None
		if plus.background is not None:
			self._background_path.addRect(left, top, right - left, bottom - top)
			self._background = PySide6.QtGui.QBrush(
				_color(plus.background, "plus background"),
			)
		self._interaction_path = PySide6.QtGui.QPainterPath(self._glyph_path)
		self._interaction_path.addPath(self._background_path)
		issued_bounds = PySide6.QtCore.QRectF(left, top, right - left, bottom - top)
		paint_bounds = issued_bounds.united(self._glyph_path.boundingRect())
		self._bounds = paint_bounds.adjusted(
			-_PADDING, -_PADDING, _PADDING, _PADDING,
		)
		self.setPos(anchor)
		self.setZValue(float(self._target.source_order))
		self.setFlag(PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, True)
		self.setFlag(PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable, False)

	#============================================
	@property
	def target(self) -> object:
		"""Return the authenticated presentation target represented by this item."""
		return self._target

	#============================================
	@property
	def glyph_path(self) -> PySide6.QtGui.QPainterPath:
		"""Return a copy of the cached renderer-issued glyph outline path."""
		return PySide6.QtGui.QPainterPath(self._glyph_path)

	#============================================
	@property
	def background_path(self) -> PySide6.QtGui.QPainterPath:
		"""Return a copy of the explicit backend bounds used for background paint."""
		return PySide6.QtGui.QPainterPath(self._background_path)

	#============================================
	@property
	def foreground_color(self) -> PySide6.QtGui.QColor:
		"""Return the explicit renderer paint without consulting the UI palette."""
		return PySide6.QtGui.QColor(self._foreground.color())

	#============================================
	@property
	def background_color(self) -> PySide6.QtGui.QColor | None:
		"""Return the explicit optional background paint."""
		if self._background is None:
			return None
		return PySide6.QtGui.QColor(self._background.color())

	#============================================
	def boundingRect(self) -> PySide6.QtCore.QRectF:
		"""Return the API-issued ink bounds with fixed interaction padding."""
		return PySide6.QtCore.QRectF(self._bounds)

	#============================================
	def shape(self) -> PySide6.QtGui.QPainterPath:
		"""Return cached glyph/background geometry without recomputing text layout."""
		return PySide6.QtGui.QPainterPath(self._interaction_path)

	#============================================
	def paint(self, painter: PySide6.QtGui.QPainter,
			option: PySide6.QtWidgets.QStyleOptionGraphicsItem,
			widget: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Paint cached explicit content and a Qt-local selection outline."""
		del widget
		painter.setPen(PySide6.QtCore.Qt.PenStyle.NoPen)
		if self._background is not None:
			painter.setBrush(self._background)
			painter.drawPath(self._background_path)
		painter.setBrush(self._foreground)
		painter.drawPath(self._glyph_path)
		if option.state & PySide6.QtWidgets.QStyle.StateFlag.State_Selected:
			color = PySide6.QtWidgets.QApplication.palette().highlight().color()
			pen = PySide6.QtGui.QPen(color, 1.5)
			pen.setCosmetic(False)
			painter.setPen(pen)
			painter.setBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
			painter.drawRect(self._bounds.adjusted(1.75, 1.75, -1.75, -1.75))

	#============================================
	def dispose(self) -> None:
		"""Provide the shared graphics-retirement callback contract."""


#============================================
def _target(value: object, extension: object) -> object:
	"""Authenticate the exact plus target through the presentation boundary."""
	try:
		target = ferrum_qt.canvas.ferrum_presentation_target.presentation_target_from_dto(
			value, extension, "plus",
		)
	except (AttributeError, TypeError, ValueError) as exc:
		raise FerrumPlusItemError("plus target is invalid") from exc
	return target


#============================================
def _point(value: object, extension: object,
		description: str) -> PySide6.QtCore.QPointF:
	"""Copy one exact finite renderer point."""
	if type(value) is not extension.RenderPointV1:
		raise FerrumPlusItemError(f"{description} has the wrong DTO type")
	x = _finite(value.x, f"{description} x")
	y = _finite(value.y, f"{description} y")
	return PySide6.QtCore.QPointF(x, y)


#============================================
def _color(value: object, description: str) -> PySide6.QtGui.QColor:
	"""Copy one explicit lowercase paint without a palette fallback."""
	if type(value) is not str or _RGB24.fullmatch(value) is None:
		raise FerrumPlusItemError(f"{description} must be lowercase six-digit Rgb24")
	return PySide6.QtGui.QColor(f"#{value}")


#============================================
def _positive(value: object, description: str) -> float:
	"""Return one finite positive scalar."""
	value = _finite(value, description)
	if value <= 0.0:
		raise FerrumPlusItemError(f"{description} must be positive")
	return value


#============================================
def _finite(value: object, description: str) -> float:
	"""Return one finite non-boolean scalar."""
	if type(value) not in (int, float) or not math.isfinite(value):
		raise FerrumPlusItemError(f"{description} must be finite")
	return float(value)


#============================================
def _ferrum_chem() -> object:
	"""Load the installed direct extension only at the public boundary."""
	try:
		import ferrum_qt.ferrum.engine as engine
	except ImportError as exc:
		raise FerrumPlusItemError("Ferrum plus rendering requires ferrum_chem") from exc
	return engine

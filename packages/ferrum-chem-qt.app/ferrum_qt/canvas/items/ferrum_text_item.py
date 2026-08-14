"""Disposable Qt item for one API-issued direct-root Text render."""

# Standard Library
import math
import re

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_presentation_projection
import ferrum_qt.canvas.ferrum_telex
import ferrum_qt.canvas.telex_glyph_outline


_FACE = "ferrum-telex-regular-v1"
_RGB24 = re.compile(r"^[0-9a-f]{6}$")
_PADDING = 1.0


#============================================
class FerrumTextItemError(ValueError):
	"""A frozen Text render cannot be painted without frontend interpretation."""


class FerrumTextItem(PySide6.QtWidgets.QGraphicsObject):
	"""Selectable direct-root Text painted from backend-issued Telex glyph facts."""

	#============================================
	def __init__(self, text_render: object, telex_resource: object,
			parent: PySide6.QtWidgets.QGraphicsItem | None = None) -> None:
		"""Authenticate one extension-owned Text render and cache complete paths."""
		super().__init__(parent)
		extension = _ferrum_chem()
		if type(text_render) is not extension.DocumentTextRenderV1:
			raise FerrumTextItemError(
				"Text render must be ferrum_chem.DocumentTextRenderV1",
			)
		telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex_resource)
		self._initialize(text_render, extension, telex)

	#============================================
	@classmethod
	def _from_observation(cls, text_render: object,
			telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex) -> "FerrumTextItem":
		"""Reuse a controller-authenticated Telex face for one exact runtime DTO."""
		item = cls.__new__(cls)
		PySide6.QtWidgets.QGraphicsObject.__init__(item)
		extension = _ferrum_chem()
		if type(text_render) is not extension.DocumentTextRenderV1:
			raise FerrumTextItemError(
				"Text render must be ferrum_chem.DocumentTextRenderV1",
			)
		if not isinstance(telex, ferrum_qt.canvas.ferrum_telex.FerrumTelex):
			raise FerrumTextItemError("Text render requires verified Telex bytes")
		item._initialize(text_render, extension, telex)
		return item

	#============================================
	def _initialize(self, text_render: object, extension: object,
			telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex) -> None:
		"""Copy a verified Text render into immutable Qt-local paths and paints."""
		self._target = _target(text_render.target, extension)
		anchor = _point(text_render.anchor, extension, "Text anchor")
		operation = text_render.operation
		if type(operation) is not extension.PresentationTextOpV1:
			raise FerrumTextItemError("Text render has the wrong operation DTO type")
		if operation.face != _FACE or operation.z != 20:
			raise FerrumTextItemError("Text render has an unsupported face or paint order")
		if type(operation.runs) is not tuple or not operation.runs:
			raise FerrumTextItemError("Text render requires frozen glyph runs")
		for run in operation.runs:
			if type(run) is not extension.PresentationGlyphRunV1:
				raise FerrumTextItemError("Text render has the wrong glyph-run DTO type")
			if type(run.glyphs) is not tuple or any(
				type(glyph) is not extension.GlyphPlacementV1 for glyph in run.glyphs
			):
				raise FerrumTextItemError("Text render has invalid frozen glyphs")
		font = telex.raw_font(_positive(operation.size, "Text size"))
		try:
			self._glyph_path = (
				ferrum_qt.canvas.telex_glyph_outline.path_from_presentation_runs(
					operation.runs, font,
				)
			)
		except ferrum_qt.canvas.telex_glyph_outline.TelexGlyphOutlineError as exc:
			raise FerrumTextItemError(str(exc)) from exc
		self._foreground = PySide6.QtGui.QBrush(_color(operation.paint, "Text foreground"))
		bounds = text_render.bounds
		if type(bounds) is not extension.PresentationTextBoundsV1:
			raise FerrumTextItemError("Text bounds have the wrong DTO type")
		left = _finite(bounds.left, "Text left bound")
		top = _finite(bounds.top, "Text top bound")
		right = _finite(bounds.right, "Text right bound")
		bottom = _finite(bounds.bottom, "Text bottom bound")
		if left >= right or top >= bottom:
			raise FerrumTextItemError("Text bounds must be finite and nonempty")
		self._background_path = PySide6.QtGui.QPainterPath()
		self._background = None
		if text_render.background is not None:
			self._background_path.addRect(left, top, right - left, bottom - top)
			self._background = PySide6.QtGui.QBrush(
				_color(text_render.background, "Text background"),
			)
		self._interaction_path = PySide6.QtGui.QPainterPath(self._glyph_path)
		self._interaction_path.addPath(self._background_path)
		issued_bounds = PySide6.QtCore.QRectF(left, top, right - left, bottom - top)
		self._bounds = issued_bounds.united(self._glyph_path.boundingRect()).adjusted(
			-_PADDING, -_PADDING, _PADDING, _PADDING,
		)
		self.setPos(anchor)
		self.setZValue(float(self._target.source_order))
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, True,
		)
		self.setFlag(
			PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable, False,
		)

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
	def foreground_color(self) -> PySide6.QtGui.QColor:
		"""Return the explicit renderer paint without a palette fallback."""
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
		"""Return the backend-issued layout bounds with fixed interaction padding."""
		return PySide6.QtCore.QRectF(self._bounds)

	#============================================
	def shape(self) -> PySide6.QtGui.QPainterPath:
		"""Return cached glyph and background geometry without text layout."""
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
	"""Authenticate the exact Text target through the presentation boundary."""
	try:
		target = ferrum_qt.canvas.ferrum_presentation_projection._target(
			value, extension, "text",
		)
	except (AttributeError, TypeError, ValueError) as exc:
		raise FerrumTextItemError("Text target is invalid") from exc
	return target


#============================================
def _point(value: object, extension: object,
		description: str) -> PySide6.QtCore.QPointF:
	"""Copy one exact finite renderer point."""
	if type(value) is not extension.RenderPointV1:
		raise FerrumTextItemError(f"{description} has the wrong DTO type")
	return PySide6.QtCore.QPointF(
		_finite(value.x, f"{description} x"),
		_finite(value.y, f"{description} y"),
	)


#============================================
def _color(value: object, description: str) -> PySide6.QtGui.QColor:
	"""Copy one explicit lowercase paint without a palette fallback."""
	if type(value) is not str or _RGB24.fullmatch(value) is None:
		raise FerrumTextItemError(f"{description} must be lowercase six-digit Rgb24")
	return PySide6.QtGui.QColor(f"#{value}")


#============================================
def _positive(value: object, description: str) -> float:
	"""Return one finite positive scalar."""
	value = _finite(value, description)
	if value <= 0.0:
		raise FerrumTextItemError(f"{description} must be positive")
	return value


#============================================
def _finite(value: object, description: str) -> float:
	"""Return one finite non-boolean scalar."""
	if type(value) not in (int, float) or not math.isfinite(value):
		raise FerrumTextItemError(f"{description} must be finite")
	return float(value)


#============================================
def _ferrum_chem() -> object:
	"""Load the installed direct extension only at the public boundary."""
	try:
		import ferrum_chem
	except ImportError as exc:
		raise FerrumTextItemError("Ferrum Text rendering requires ferrum_chem") from exc
	return ferrum_chem

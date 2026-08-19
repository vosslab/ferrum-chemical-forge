"""Disposable page background built only from Rust-issued physical geometry."""

# Standard Library
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
class FerrumPaperItemError(ValueError):
	"""Raised when a frozen paper projection cannot define one finite page."""


#============================================
class FerrumPaperItem(PySide6.QtWidgets.QGraphicsRectItem):
	"""One noninteractive UI page surface behind every document root."""

	#============================================
	def __init__(self, layout: object) -> None:
		"""Copy one exact frozen Rust page without interpreting CDML attributes."""
		try:
			import ferrum_qt.ferrum.engine as engine
		except ImportError as exc:
			raise FerrumPaperItemError("Ferrum paper binding is unavailable") from exc
		if type(layout) is not engine.PaperLayoutProjectionV1:
			raise FerrumPaperItemError(
				"paper background requires the frozen Ferrum paper layout",
			)
		page = layout.page
		if type(page) is not engine.PaperPageV1:
			raise FerrumPaperItemError("paper layout has the wrong page DTO")
		if page.issue is not None and type(page.issue) is not engine.PaperPageIssueV1:
			raise FerrumPaperItemError("paper compatibility issue has the wrong DTO")
		self._initialize(page)

	#============================================
	@classmethod
	def _from_fixture(cls, layout: object) -> "FerrumPaperItem":
		"""Build a focused-test page without weakening the public exact-type boundary."""
		item = cls.__new__(cls)
		item._initialize(layout.page)
		return item

	#============================================
	def _initialize(self, page: object) -> None:
		"""Cache finite page geometry and palette-only decoration."""
		values = (page.scene_left, page.scene_top, page.scene_right, page.scene_bottom)
		if any(type(value) is not float or not math.isfinite(value) for value in values):
			raise FerrumPaperItemError("paper scene rectangle must be finite")
		left, top, right, bottom = values
		if right <= left or bottom <= top:
			raise FerrumPaperItemError("paper scene rectangle must have positive area")
		PySide6.QtWidgets.QGraphicsRectItem.__init__(
			self, left, top, right - left, bottom - top,
		)
		palette = PySide6.QtWidgets.QApplication.palette()
		paper_color = palette.color(PySide6.QtGui.QPalette.ColorRole.Base)
		outline_color = palette.color(PySide6.QtGui.QPalette.ColorRole.Mid)
		pen = PySide6.QtGui.QPen(outline_color)
		pen.setCosmetic(True)
		pen.setWidthF(1.0)
		self.setPen(pen)
		self.setBrush(PySide6.QtGui.QBrush(paper_color))
		self.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		self.setFlag(PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, False)
		self.setZValue(-1.0)
		self._issue = page.issue

	#============================================
	@property
	def issue(self) -> object | None:
		"""Return the typed Rust compatibility issue, when fallback geometry was used."""
		return self._issue

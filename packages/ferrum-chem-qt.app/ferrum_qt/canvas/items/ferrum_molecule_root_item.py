"""Noninteractive Qt ownership root for one Rust-measured molecule plan."""

# Standard Library
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# Local application modules
import ferrum_qt.ferrum.engine as engine


#============================================
class FerrumMoleculeRootItemError(ValueError):
	"""Raised when Rust-issued molecule-root display facts are invalid."""


#============================================
class FerrumMoleculeRootItem(PySide6.QtWidgets.QGraphicsItem):
	"""Own molecule member items without taking geometry or selection authority."""

	#============================================
	def __init__(self, molecule: object, bounds: object) -> None:
		"""Copy exact frozen Rust root identity and measured painted bounds."""
		if type(molecule) is not engine.MoleculeRenderRootV1:
			raise FerrumMoleculeRootItemError(
				"molecule ownership root requires the frozen Ferrum root",
			)
		if type(bounds) is not engine.MoleculeContentBoundsV1:
			raise FerrumMoleculeRootItemError(
				"molecule ownership root requires Rust-measured content bounds",
			)
		self._initialize(molecule.document_object_id, bounds)

	#============================================
	@classmethod
	def _from_fixture(cls, molecule: object, bounds: object) -> "FerrumMoleculeRootItem":
		"""Build focused frozen fixtures without weakening the runtime boundary."""
		item = cls.__new__(cls)
		item._initialize(molecule.document_object_id, bounds)
		return item

	#============================================
	def _initialize(self, document_object_id: object, bounds: object) -> None:
		"""Cache one immutable identity and finite positive Rust bounds."""
		if type(document_object_id) is not str or not document_object_id:
			raise FerrumMoleculeRootItemError("molecule ownership identity is invalid")
		values = (bounds.left, bounds.top, bounds.right, bounds.bottom)
		if any(type(value) is not float or not math.isfinite(value) for value in values):
			raise FerrumMoleculeRootItemError("molecule content bounds must be finite")
		left, top, right, bottom = values
		if right <= left or bottom <= top:
			raise FerrumMoleculeRootItemError(
				"molecule content bounds must have positive area",
			)
		PySide6.QtWidgets.QGraphicsItem.__init__(self)
		self._document_object_id = document_object_id
		self._bounds = PySide6.QtCore.QRectF(left, top, right - left, bottom - top)
		self.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		self.setHandlesChildEvents(False)
		self.setFlag(PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, False)
		self.setFlag(PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable, False)

	#============================================
	@property
	def document_object_id(self) -> str:
		"""Return the copied opaque Rust document-root identity."""
		return self._document_object_id

	#============================================
	def boundingRect(self) -> PySide6.QtCore.QRectF:
		"""Return only the authoritative Rust-measured painted bounds."""
		return PySide6.QtCore.QRectF(self._bounds)

	#============================================
	def paint(self, painter: PySide6.QtGui.QPainter,
			option: PySide6.QtWidgets.QStyleOptionGraphicsItem,
			widget: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Paint nothing; child render items own all visible presentation."""
		pass

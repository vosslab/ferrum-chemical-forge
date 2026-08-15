"""Disposable projection-only preview for complete-root translation."""

# Standard Library
import dataclasses
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeTranslationSelection:
	"""Captured Rust anchor receipt plus projection-only complete-root bounds."""

	targets: tuple[object, ...]
	durable_selection: tuple[tuple[str, str], ...]
	source_revision: int
	source_digest: str
	anchor_x: float
	anchor_y: float
	bounds: tuple[tuple[float, float, float, float], ...]


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeTranslationPreview:
	"""One scene-owned group of dashed non-authoritative root bounds."""

	selection: FerrumNativeTranslationSelection
	root: PySide6.QtWidgets.QGraphicsItemGroup


#============================================
def create_translation_preview(tab: object,
		selection: FerrumNativeTranslationSelection) -> FerrumNativeTranslationPreview:
	"""Create one local bounds overlay above the unchanged authoritative scene."""
	if type(selection) is not FerrumNativeTranslationSelection:
		raise TypeError("translation preview requires exact captured selection facts")
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("native document has no current scene")
	color = PySide6.QtWidgets.QApplication.palette().color(
		PySide6.QtGui.QPalette.ColorRole.Highlight,
	)
	pen = PySide6.QtGui.QPen(color)
	pen.setWidthF(1.5)
	pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
	pen.setCosmetic(False)
	fill = PySide6.QtGui.QColor(color)
	fill.setAlpha(48)
	root = PySide6.QtWidgets.QGraphicsItemGroup()
	root.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	root.setZValue(1_000_000.0)
	for x, y, width, height in selection.bounds:
		item = PySide6.QtWidgets.QGraphicsRectItem(root)
		item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		item.setPen(pen)
		item.setBrush(PySide6.QtGui.QBrush(fill))
		item.setRect(x, y, width, height)
	scene.addItem(root)
	return FerrumNativeTranslationPreview(selection, root)


#============================================
def update_translation_preview(preview: FerrumNativeTranslationPreview,
		dx: float, dy: float) -> None:
	"""Translate only the disposable overlay by one finite scene-point delta."""
	if type(preview) is not FerrumNativeTranslationPreview:
		raise TypeError("translation preview requires exact local preview state")
	if (
			type(dx) is not float
			or type(dy) is not float
			or not math.isfinite(dx)
			or not math.isfinite(dy)
		):
		raise TypeError("translation preview requires finite float deltas")
	preview.root.setPos(dx, dy)

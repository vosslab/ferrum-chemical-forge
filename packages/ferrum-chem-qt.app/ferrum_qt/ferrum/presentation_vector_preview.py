"""Disposable Qt projection of Rust-issued ordinary vector overlays."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
def create_overlay(tab: object, overlay: object) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Paint only the exact shape and appearance facts issued by Rust."""
	import ferrum_qt.ferrum.engine as engine
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum vector preview requires an installed scene")
	path = PySide6.QtGui.QPainterPath()
	if overlay.kind is engine.PresentationVectorKindV1.line:
		path.moveTo(overlay.start_x, overlay.start_y)
		path.lineTo(overlay.end_x, overlay.end_y)
	else:
		rectangle = PySide6.QtCore.QRectF(
			overlay.left, overlay.top,
			overlay.right - overlay.left, overlay.bottom - overlay.top,
		)
		if overlay.kind in (
			engine.PresentationVectorKindV1.rectangle,
			engine.PresentationVectorKindV1.square,
		):
			path.addRect(rectangle)
		else:
			path.addEllipse(rectangle)
	pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor(overlay.stroke_color))
	pen.setWidthF(overlay.stroke_width)
	pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
	pen.setCosmetic(False)
	brush = PySide6.QtGui.QBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
	if overlay.fill_color is not None:
		brush = PySide6.QtGui.QBrush(PySide6.QtGui.QColor(overlay.fill_color))
	item = scene.addPath(path, pen, brush)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	return item

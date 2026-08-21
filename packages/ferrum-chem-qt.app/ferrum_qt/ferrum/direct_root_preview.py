"""Disposable Qt overlays for Rust-issued direct-root interaction facts."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
def create_direct_root_bounds_preview(tab: object, bounds: object) -> PySide6.QtWidgets.QGraphicsItemGroup:
	"""Draw only bounds supplied by the opaque Rust selection or preview value."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum document has no current scene")
	color = PySide6.QtWidgets.QApplication.palette().color(
		PySide6.QtGui.QPalette.ColorRole.Highlight,
	)
	pen = PySide6.QtGui.QPen(color)
	pen.setWidthF(1.5)
	pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
	fill = PySide6.QtGui.QColor(color)
	fill.setAlpha(40)
	root = PySide6.QtWidgets.QGraphicsItemGroup()
	root.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	root.setZValue(1_000_000.0)
	for value in bounds:
		item = PySide6.QtWidgets.QGraphicsRectItem(root)
		item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		item.setPen(pen)
		item.setBrush(PySide6.QtGui.QBrush(fill))
		item.setRect(
			float(value.left), float(value.top),
			float(value.right - value.left), float(value.bottom - value.top),
		)
	scene.addItem(root)
	return root


#============================================
def create_direct_root_selection_preview(tab: object, selection: object) -> PySide6.QtWidgets.QGraphicsItemGroup:
	"""Project immutable Rust selection bounds without inspecting scene items."""
	bounds = tuple(root.bounds for root in selection.roots)
	return create_direct_root_bounds_preview(tab, bounds)

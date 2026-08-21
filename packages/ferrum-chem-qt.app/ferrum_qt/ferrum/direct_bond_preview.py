"""Disposable Qt projection of Rust-issued direct normal-bond overlays."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
def create_overlay(tab: object, overlay: object) -> PySide6.QtWidgets.QGraphicsLineItem:
	"""Paint one exact Rust overlay without deriving a candidate endpoint locally."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum direct-bond preview requires an installed scene")
	color = PySide6.QtWidgets.QApplication.palette().color(
		PySide6.QtGui.QPalette.ColorRole.Highlight,
	)
	pen = PySide6.QtGui.QPen(color)
	pen.setWidthF(1.5)
	pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
	item = scene.addLine(
		overlay.start_x, overlay.start_y, overlay.end_x, overlay.end_y, pen,
	)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	return item

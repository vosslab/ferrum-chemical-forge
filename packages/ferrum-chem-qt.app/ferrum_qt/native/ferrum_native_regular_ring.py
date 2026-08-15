"""Thin native projection helpers for Rust-owned detached regular-ring receipts."""

import math

import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

import ferrum_qt.config.geometry_units


def prepare_cyclohexane(tab: object, center: PySide6.QtCore.QPointF) -> object:
	"""Ask Rust for the exact C6 candidate and copied preview vertices."""
	if not math.isfinite(center.x()) or not math.isfinite(center.y()):
		raise ValueError("Choose a finite empty page location to insert a separate ring.")
	return tab.prepare_regular_ring(
		6, float(center.x()), float(center.y()),
		ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT,
	)


def create_preview(tab: object, prepared: object) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Paint only the Rust receipt's exact polygon; it has no document ownership."""
	vertices = prepared.vertices
	if len(vertices) < 3:
		raise ValueError("Rust regular-ring receipt has no polygon")
	path = PySide6.QtGui.QPainterPath(PySide6.QtCore.QPointF(vertices[0].x, vertices[0].y))
	for vertex in vertices[1:]:
		path.lineTo(vertex.x, vertex.y)
	path.closeSubpath()
	pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor("#49719c"))
	pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
	pen.setWidthF(1.5)
	item = tab.view.scene().addPath(path, pen)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	return item

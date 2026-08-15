"""Disposable Qt projection of source-owned directed-bond preview operations."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
def create_directed_preview(tab: object, start: PySide6.QtCore.QPointF,
		end: PySide6.QtCore.QPointF, presentation: object) -> PySide6.QtWidgets.QGraphicsItem:
	"""Copy private Rust V2 preview operations into one disposable scene item group."""
	import ferrum_chem
	operations = ferrum_chem.native_directed_bond_preview_v1(
		float(start.x()), float(start.y()), float(end.x()), float(end.y()), presentation,
	)
	group = PySide6.QtWidgets.QGraphicsItemGroup()
	for operation in operations:
		if operation.kind == "path":
			group.addToGroup(_path_item(operation.operation))
		elif operation.kind == "line":
			group.addToGroup(_line_item(operation.operation))
		else:
			raise ValueError("Rust directed-bond preview returned an unsupported operation")
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("native directed-bond preview requires an installed scene")
	scene.addItem(group)
	group.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	group.setZValue(1_000_000.0)
	return group


#============================================
def _path_item(operation: object) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Copy one received V2 path without selecting geometry or paint locally."""
	path = PySide6.QtGui.QPainterPath()
	path.setFillRule(PySide6.QtCore.Qt.FillRule.OddEvenFill)
	for command in operation.commands:
		if command.kind == "move_to":
			path.moveTo(command.point.x, command.point.y)
		elif command.kind == "line_to":
			path.lineTo(command.point.x, command.point.y)
		elif command.kind == "cubic_to":
			path.cubicTo(
				command.control_1.x, command.control_1.y,
				command.control_2.x, command.control_2.y,
				command.point.x, command.point.y,
			)
		elif command.kind == "close":
			path.closeSubpath()
		else:
			raise ValueError("Rust directed-bond preview path has an unknown command")
	item = PySide6.QtWidgets.QGraphicsPathItem(path)
	if operation.fill_paint is not None:
		item.setBrush(PySide6.QtGui.QBrush(PySide6.QtGui.QColor("#" + operation.fill_paint)))
	if operation.stroke_paint is not None and operation.stroke_width is not None:
		item.setPen(PySide6.QtGui.QPen(
			PySide6.QtGui.QColor("#" + operation.stroke_paint), operation.stroke_width,
		))
	else:
		item.setPen(PySide6.QtCore.Qt.PenStyle.NoPen)
	return item


#============================================
def _line_item(operation: object) -> PySide6.QtWidgets.QGraphicsLineItem:
	"""Copy one received finite source-owned line preview operation."""
	item = PySide6.QtWidgets.QGraphicsLineItem(
		operation.start.x, operation.start.y, operation.end.x, operation.end.y,
	)
	item.setPen(PySide6.QtGui.QPen(
		PySide6.QtGui.QColor("#" + operation.paint), operation.width,
	))
	return item

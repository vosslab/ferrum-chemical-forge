"""Native-client helpers for a Rust-owned standalone D-glucose Haworth receipt."""

import math

import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_qt.canvas.items.ferrum_plan_item


def prepare_recipe(tab: object, recipe: str,
		center: PySide6.QtCore.QPointF) -> object:
	"""Ask Rust for one exact translated detached Haworth drawing."""
	if not math.isfinite(center.x()) or not math.isfinite(center.y()):
		raise ValueError("Choose a finite empty page location to insert D-glucose.")
	return tab.prepare_standalone_haworth(recipe, float(center.x()), float(center.y()))


def create_preview(tab: object, prepared: object) -> PySide6.QtWidgets.QGraphicsItemGroup:
	"""Paint only frozen Rust V2 operations, including Haworth cap and layer facts."""
	group = PySide6.QtWidgets.QGraphicsItemGroup()
	for batch in prepared.preview_plan.batches:
		layer = {"ordinary": 0.0, "haworth_front_stroke": 0.1,
			"haworth_front_wedge": 0.2}.get(batch.display_layer)
		if layer is None:
			raise ValueError("Rust Haworth preview has an unknown display layer")
		for operation in batch.operations:
			command, z = ferrum_qt.canvas.items.ferrum_plan_item._copy_operation(
				operation, ferrum_qt.canvas.items.ferrum_plan_item._Point(0.0, 0.0), None,
			)
			item = PySide6.QtWidgets.QGraphicsPathItem(command.path)
			pen = getattr(command, "pen", None)
			brush = getattr(command, "brush", None)
			item.setPen(pen if pen is not None else PySide6.QtCore.Qt.PenStyle.NoPen)
			item.setBrush(brush if brush is not None else PySide6.QtCore.Qt.BrushStyle.NoBrush)
			item.setZValue(float(z) + layer)
			group.addToGroup(item)
	bounds = group.boundingRect()
	if (not math.isfinite(bounds.x()) or not math.isfinite(bounds.y())
			or not math.isfinite(bounds.width()) or not math.isfinite(bounds.height())
			or bounds.isNull()):
		raise ValueError("Rust Haworth V2 preview has invalid bounds")
	group.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	group.setZValue(1_000_000.0)
	tab.view.scene().addItem(group)
	return group

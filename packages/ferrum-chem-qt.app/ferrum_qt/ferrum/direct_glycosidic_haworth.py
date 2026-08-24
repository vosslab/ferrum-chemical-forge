"""Receipt-only preview painting for direct-glycosidic Haworth placement."""

import math

import PySide6.QtCore
import PySide6.QtWidgets

import ferrum_qt.canvas.items.ferrum_plan_item


def create_preview(tab: object, prepared: object) -> PySide6.QtWidgets.QGraphicsItemGroup:
	"""Paint only the frozen V2 operations supplied by Rust's anchored receipt."""
	group = PySide6.QtWidgets.QGraphicsItemGroup()
	for batch in prepared.preview_plan.batches:
		layer = {
			"ordinary": 0.0,
			"haworth_front_stroke": 0.1,
			"haworth_front_wedge": 0.2,
		}.get(batch.display_layer)
		if layer is None:
			raise ValueError("Rust direct-glycosidic Haworth preview has an unknown layer")
		for operation in batch.operations:
			command, z_value = ferrum_qt.canvas.items.ferrum_plan_item._copy_operation(
				operation, ferrum_qt.canvas.items.ferrum_plan_item._Point(0.0, 0.0), None,
			)
			item = PySide6.QtWidgets.QGraphicsPathItem(command.path)
			pen = getattr(command, "pen", None)
			brush = getattr(command, "brush", None)
			item.setPen(pen if pen is not None else PySide6.QtCore.Qt.PenStyle.NoPen)
			item.setBrush(brush if brush is not None else PySide6.QtCore.Qt.BrushStyle.NoBrush)
			item.setZValue(float(z_value) + layer)
			group.addToGroup(item)
	bounds = group.boundingRect()
	if (
		not math.isfinite(bounds.x()) or not math.isfinite(bounds.y())
		or not math.isfinite(bounds.width()) or not math.isfinite(bounds.height())
		or bounds.isNull()
	):
		raise ValueError("Rust direct-glycosidic Haworth preview has invalid bounds")
	group.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	group.setZValue(1_000_000.0)
	tab.view.scene().addItem(group)
	return group

"""Disposable Qt projection of renderer-owned molecule render plans."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

import ferrum_qt.canvas.items.ferrum_plan_item


#============================================
def create_plan_overlay(
		tab: object, plan: object, batch_z_order: bool = False,
		) -> PySide6.QtWidgets.QGraphicsItemGroup:
	"""Project every frozen renderer batch without reinterpreting its operations.

	``batch_z_order`` preserves callers whose renderer-issued batches require
	explicit local stacking within the disposable overlay.
	"""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum molecule-plan overlay requires an installed scene")
	group = PySide6.QtWidgets.QGraphicsItemGroup()
	for index in range(len(plan.batches)):
		item = ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem(
			plan, index, tab._controller._telex_resource,
		)
		item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		if batch_z_order:
			item.setZValue(float(index))
		group.addToGroup(item)
	scene.addItem(group)
	group.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	group.setZValue(1_000_000.0)
	return group

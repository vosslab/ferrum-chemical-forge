"""Thin Ferrum projection helpers for renderer-admitted regular-ring receipts."""

import math

import PySide6.QtCore
import PySide6.QtWidgets

import ferrum_qt.config.geometry_units
import ferrum_qt.ferrum.direct_bond_preview


def prepare_cyclohexane(tab: object, center: PySide6.QtCore.QPointF) -> object:
	"""Ask Rust for the exact C6 candidate and renderer-issued preview plan."""
	if not math.isfinite(center.x()) or not math.isfinite(center.y()):
		raise ValueError("Choose a finite empty page location to insert a separate ring.")
	return tab.prepare_regular_ring(
		6, float(center.x()), float(center.y()),
		ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT,
	)


def create_preview(tab: object, prepared: object) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Paint only the renderer-issued molecule operations for the pending ring."""
	plan = prepared.render_plan
	operations = tuple(operation for batch in plan.batches for operation in batch.operations)
	if not operations:
		raise ValueError("Ferrum renderer omitted regular-ring preview operations")
	return ferrum_qt.ferrum.direct_bond_preview.create_issued_operations_overlay(tab, operations)

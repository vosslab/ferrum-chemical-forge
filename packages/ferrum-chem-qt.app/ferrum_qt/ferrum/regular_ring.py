"""Thin Ferrum helpers for generic regular-ring operations."""

import math

import PySide6.QtCore
import ferrum_qt.config.geometry_units


def insert_cyclohexane(tab: object, center: PySide6.QtCore.QPointF) -> object:
	"""Commit one C6 ring through the tab's generic operation authority."""
	if not math.isfinite(center.x()) or not math.isfinite(center.y()):
		raise ValueError("Choose a finite empty page location to insert a separate ring.")
	return tab.insert_regular_ring(
		6, float(center.x()), float(center.y()),
		ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT,
	)

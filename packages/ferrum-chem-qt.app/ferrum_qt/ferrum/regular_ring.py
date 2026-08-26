"""Thin Ferrum helpers for generic regular-ring operations."""

import math

import PySide6.QtCore
import ferrum_qt.config.geometry_units


REGULAR_RING_NAMES = {
	3: "Cyclopropane",
	4: "Cyclobutane",
	5: "Cyclopentane",
	6: "Cyclohexane",
	7: "Cycloheptane",
	8: "Cyclooctane",
}


def display_name(size: int) -> str:
	"""Return the public name for one admitted regular-ring size."""
	if type(size) is not int or size not in REGULAR_RING_NAMES:
		raise ValueError("Choose one regular ring from C3 through C8.")
	return REGULAR_RING_NAMES[size]


def insert_regular_ring(tab: object, size: int,
		center: PySide6.QtCore.QPointF) -> object:
	"""Commit one admitted ring through the tab's generic operation authority."""
	display_name(size)
	if not math.isfinite(center.x()) or not math.isfinite(center.y()):
		raise ValueError("Choose a finite empty page location to insert a separate ring.")
	return tab.insert_regular_ring(
		size, float(center.x()), float(center.y()),
		ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT,
	)

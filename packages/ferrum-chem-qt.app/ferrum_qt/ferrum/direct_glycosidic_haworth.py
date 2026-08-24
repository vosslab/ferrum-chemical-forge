"""Native-client replay for direct-glycosidic Haworth transitions."""

import PySide6.QtWidgets

def create_preview(tab: object, overlay: object) -> PySide6.QtWidgets.QGraphicsItemGroup:
	"""Paint the generic identifier-free precommit overlay before redemption."""
	import ferrum_qt.ferrum.direct_bond_overlay
	return ferrum_qt.ferrum.direct_bond_overlay.create_overlay(tab, overlay)

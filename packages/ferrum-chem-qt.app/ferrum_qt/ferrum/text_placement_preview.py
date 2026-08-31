"""Disposable Qt overlay for one Rust-issued standalone Text preview."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.items.ferrum_text_item
import ferrum_qt.ferrum.document_display_refresh


#============================================
def create_text_placement_overlay(tab: object, overlay: object) -> PySide6.QtWidgets.QGraphicsItem:
	"""Paint exact backend glyph geometry without Qt text measurement or layout."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum Text preview requires an installed scene")
	item = ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem(
		overlay, tab._controller._font_resource, tab.document_display_palette,
	)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setFlag(PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, False)
	item.setZValue(1_000_000.0)
	scene.addItem(item)
	refreshable = ferrum_qt.ferrum.document_display_refresh.DocumentDisplayDelegatingRefreshableV1(
		item,
	)
	ferrum_qt.ferrum.document_display_refresh.register_attached_document_display_refreshable(
		tab, item, refreshable,
	)
	return item

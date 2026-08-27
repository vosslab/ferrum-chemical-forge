"""Disposable Qt overlays for Rust-issued direct-root interaction facts."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.themes.document_display_palette
import ferrum_qt.ferrum.document_display_refresh


#============================================
def create_direct_root_bounds_preview(tab: object, bounds: object) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Draw only bounds supplied by the opaque Rust selection or preview value."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum document has no current scene")
	path = PySide6.QtGui.QPainterPath()
	for value in bounds:
		path.addRect(
			float(value.left), float(value.top),
			float(value.right - value.left), float(value.bottom - value.top),
		)
	root = PySide6.QtWidgets.QGraphicsPathItem(path)
	root.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	root.setZValue(1_000_000.0)
	scene.addItem(root)
	refreshable = ferrum_qt.ferrum.document_display_refresh.DocumentDisplayRoleMaterialRefreshableV1(
		(root,),
		ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PREVIEW_OUTLINE,
		ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PREVIEW_FILL,
		1.5, PySide6.QtCore.Qt.PenStyle.DashLine,
	)
	refreshable.refresh_document_display_palette(_display_palette(tab))
	ferrum_qt.ferrum.document_display_refresh.register_attached_document_display_refreshable(
		tab, root, refreshable,
	)
	return root


#============================================
def create_direct_root_selection_preview(tab: object, selection: object) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Project immutable Rust selection bounds without inspecting scene items."""
	bounds = tuple(root.bounds for root in selection.roots)
	return create_direct_root_bounds_preview(tab, bounds)


#============================================
def _display_palette(tab: object) -> ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
	"""Return the explicit palette for transient document interaction chrome."""
	palette = getattr(tab, "document_display_palette", None)
	if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
		raise RuntimeError("Ferrum direct-root preview requires a document display palette")
	return palette

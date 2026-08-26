"""Disposable Qt replay of renderer-owned presentation preview plans."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_presentation_render_plan


#============================================
def create_presentation_preview(tab: object, plan: object) -> PySide6.QtWidgets.QGraphicsItem:
	"""Install one renderer-issued visual-root preview plan without rebuilding geometry."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum presentation preview requires an installed scene")
	replay = ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_preview_render_plan(
		plan, tab._controller._telex_resource,
	)
	if len(replay.roots) != 1:
		replay.dispose_detached()
		raise TypeError("Ferrum presentation preview requires one renderer root")
	item = replay.roots[0]
	scene.addItem(item)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	return item


#============================================
def create_arrow_preview(tab: object, plan: object) -> PySide6.QtWidgets.QGraphicsItem:
	"""Install one renderer-issued arrow preview plan."""
	return create_presentation_preview(tab, plan)


#============================================

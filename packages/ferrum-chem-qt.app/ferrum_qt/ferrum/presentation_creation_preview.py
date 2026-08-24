"""Disposable Qt replay of renderer-owned presentation preview plans."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_presentation_render_plan


#============================================
def create_arrow_preview(tab: object, plan: object) -> PySide6.QtWidgets.QGraphicsItem:
	"""Install one renderer-issued one-root preview plan without rebuilding geometry."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum Arrow preview requires an installed scene")
	replay = ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_render_plan(
		plan, tab._controller._telex_resource,
	)
	if len(replay.roots) != 1:
		replay.dispose_detached()
		raise TypeError("Ferrum Arrow preview requires one renderer root")
	item = replay.roots[0]
	scene.addItem(item)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	return item


#============================================
def create_plus_overlay(tab: object, overlay: object) -> PySide6.QtWidgets.QGraphicsSimpleTextItem:
	"""Paint only the renderer-issued Plus text and explicit paint facts."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum Plus preview requires an installed scene")
	item = scene.addSimpleText(overlay.text)
	font = item.font()
	font.setPointSizeF(overlay.font_size)
	item.setFont(font)
	item.setBrush(PySide6.QtGui.QBrush(PySide6.QtGui.QColor(_qt_color(overlay.color))))
	item.setPos(overlay.origin_x, overlay.origin_y)
	item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	item.setZValue(1_000_000.0)
	return item


#============================================
def _qt_color(value: str) -> str:
	"""Adapt the renderer's six-digit RGB wire value only at the Qt boundary."""
	if len(value) == 6 and all(character in "0123456789abcdefABCDEF" for character in value):
		return f"#{value}"
	return value

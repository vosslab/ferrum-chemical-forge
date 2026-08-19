"""Reusable action clients for canvas zoom without view ownership."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


SLIDER_MIN = 10
SLIDER_MAX = 1000
SLIDER_STEP = 5
SLIDER_DEFAULT = 100


#============================================
def _action_client(registry: object | None, action_id: str) -> PySide6.QtGui.QAction | None:
	"""Return one live shared action without constructing a replacement action."""
	if registry is None:
		return None
	get_qt_action = getattr(registry, "get_qt_action", None)
	if not callable(get_qt_action):
		raise TypeError("Ferrum zoom controls need an ActionRegistry-like client")
	action = get_qt_action(action_id)
	if action is not None and not isinstance(action, PySide6.QtGui.QAction):
		raise TypeError(f"Ferrum action '{action_id}' is not a QAction")
	return action


#============================================
class ZoomControls(PySide6.QtWidgets.QWidget):
	"""Expose existing View actions and one injected absolute-zoom callback."""

	zoom_percent_requested = PySide6.QtCore.Signal(int)

	#============================================
	def __init__(self, registry: object | None = None,
			parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Build compact action clients from the supplied registry."""
		super().__init__(parent)
		self.setAccessibleName(self.tr("Zoom controls"))
		self._actions = {
			"out": _action_client(registry, "view.zoom_out"),
			"reset": _action_client(registry, "view.reset_zoom"),
			"in": _action_client(registry, "view.zoom_in"),
			"page": _action_client(registry, "view.zoom_page"),
			"content": _action_client(registry, "view.zoom_content"),
		}
		layout = PySide6.QtWidgets.QHBoxLayout(self)
		layout.setContentsMargins(2, 0, 2, 0)
		layout.setSpacing(4)
		self._buttons = {
			"out": self._make_button("-", "Zoom out", "Decrease display zoom"),
			"reset": self._make_button("100%", "Reset zoom", "Reset display zoom to 100%"),
			"in": self._make_button("+", "Zoom in", "Increase display zoom"),
			"page": self._make_button("Page", "Zoom to page", "Fit the active page"),
			"content": self._make_button("Content", "Zoom to content", "Fit active document content"),
		}
		for key in ("out", "reset", "in", "page", "content"):
			button = self._buttons[key]
			button.clicked.connect(lambda _checked=False, action_key=key: self._trigger(action_key))
			layout.addWidget(button)
		self._slider = PySide6.QtWidgets.QSlider(PySide6.QtCore.Qt.Orientation.Horizontal, self)
		self._slider.setObjectName("zoom-percentage-slider")
		self._slider.setRange(SLIDER_MIN, SLIDER_MAX)
		self._slider.setSingleStep(SLIDER_STEP)
		self._slider.setValue(SLIDER_DEFAULT)
		self._slider.setMinimumWidth(48)
		self._slider.setMaximumWidth(120)
		self._slider.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		self._slider.setAccessibleName(self.tr("Zoom percentage slider"))
		self._slider.setAccessibleDescription(self.tr("Set display zoom from 10% to 1000%"))
		self._slider.valueChanged.connect(self.zoom_percent_requested.emit)
		layout.addWidget(self._slider)
		for key, action in self._actions.items():
			if action is not None:
				action.changed.connect(lambda action_key=key: self._sync_action(action_key))
			self._sync_action(key)

	#============================================
	def _make_button(self, text: str, accessible_name: str,
			description: str) -> PySide6.QtWidgets.QToolButton:
		"""Create one reachable text client without taking action ownership."""
		button = PySide6.QtWidgets.QToolButton(self)
		button.setText(self.tr(text))
		button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		button.setAccessibleName(self.tr(accessible_name))
		button.setAccessibleDescription(self.tr(description))
		button.setToolTip(self.tr(description))
		return button

	#============================================
	def _trigger(self, key: str) -> None:
		"""Trigger the one window-owned action if the registry supplied it."""
		action = self._actions[key]
		if action is not None:
			action.trigger()

	#============================================
	def _sync_action(self, key: str) -> None:
		"""Mirror shared action availability to its visible action client."""
		action = self._actions[key]
		button = self._buttons[key]
		button.setEnabled(action is not None and action.isEnabled())

	#============================================
	def update_zoom_display(self, percent: float | None) -> None:
		"""Project an observed percentage without querying or retaining the view."""
		if percent is None:
			self._buttons["reset"].setText(self.tr("--"))
			self._slider.setEnabled(False)
			return
		bounded = max(SLIDER_MIN, min(round(percent), SLIDER_MAX))
		self._buttons["reset"].setText(self.tr(f"{bounded}%"))
		blocked = self._slider.blockSignals(True)
		self._slider.setValue(bounded)
		self._slider.blockSignals(blocked)
		self._slider.setEnabled(
			self._actions["reset"] is not None and self._actions["reset"].isEnabled(),
		)

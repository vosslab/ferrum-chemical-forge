"""Normalized native-mode input adapters shared by Ferrum Qt tools."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.modes.base_mode


#============================================
class _NormalizedLinePointerEvent:
	"""Position and modifier adapter for existing line gesture helpers."""

	def __init__(self, point: ferrum_qt.modes.base_mode.ScenePoint, tab: object,
			modifiers: PySide6.QtCore.Qt.KeyboardModifiers) -> None:
		"""Project a normalized scene point into the existing view helper contract."""
		view_point = tab.view.mapFromScene(point.x, point.y)
		self._position = PySide6.QtCore.QPointF(view_point)
		self._modifiers = modifiers

	def position(self) -> PySide6.QtCore.QPointF:
		"""Return the viewport point expected by established gesture methods."""
		return self._position

	def modifiers(self) -> PySide6.QtCore.Qt.KeyboardModifiers:
		"""Return the Qt modifiers captured by the controller-owned adapter."""
		return self._modifiers


#============================================
def dispatch_native_mode_input(owner: object, watched: PySide6.QtCore.QObject,
		event: PySide6.QtCore.QEvent) -> bool:
	"""Normalize one active canvas input without inspecting feature identity."""
	tab = owner._active_native_tab()
	if tab is None or watched is not owner._controller_native_viewport:
		return False
	controller = owner._window_mode_sync
	if controller.active_state.mode_id is None:
		return False
	if isinstance(event, PySide6.QtGui.QKeyEvent):
		key = event.key()
		key_names = {
			PySide6.QtCore.Qt.Key.Key_Escape: "Escape",
			PySide6.QtCore.Qt.Key.Key_Return: "Return",
			PySide6.QtCore.Qt.Key.Key_Enter: "Enter",
			PySide6.QtCore.Qt.Key.Key_Delete: "Delete",
			PySide6.QtCore.Qt.Key.Key_Backspace: "Backspace",
		}
		name = key_names.get(key)
		if name is None:
			return False
		return controller.handle_key(name, event.modifiers())
	if not isinstance(event, PySide6.QtGui.QMouseEvent):
		return False
	phase_by_type = {
		PySide6.QtCore.QEvent.Type.MouseButtonPress:
		ferrum_qt.modes.base_mode.PointerPhase.PRESS,
		PySide6.QtCore.QEvent.Type.MouseMove:
		ferrum_qt.modes.base_mode.PointerPhase.MOVE,
		PySide6.QtCore.QEvent.Type.MouseButtonRelease:
		ferrum_qt.modes.base_mode.PointerPhase.RELEASE,
	}
	phase = phase_by_type.get(event.type())
	if phase is None:
		return False
	if phase is not ferrum_qt.modes.base_mode.PointerPhase.MOVE and (
		event.button() is not PySide6.QtCore.Qt.MouseButton.LeftButton
	):
		return False
	point = tab.view.mapToScene(event.position().toPoint())
	pointer = ferrum_qt.modes.base_mode.PointerInput(
		phase, ferrum_qt.modes.base_mode.ScenePoint(float(point.x()), float(point.y())),
		True, event.modifiers(),
	)
	return controller.handle_pointer(pointer)


#============================================
def dispatch_line_mode_intent(owner: object, context: ferrum_qt.modes.base_mode.ModeContext,
		intent: ferrum_qt.modes.base_mode.ModeIntent) -> None:
	"""Apply one controller-normalized line intent through existing Rust seams."""
	del context
	active = owner._line_gesture_intent
	if active is None:
		raise RuntimeError("Ferrum line-mode dispatch has no live line gesture.")
	if type(intent) is not ferrum_qt.modes.base_mode.ModeIntent:
		raise RuntimeError("Ferrum line-mode dispatch requires a normalized ModeIntent.")
	operation_prefix = f"line.{active.tool.value}."
	if not intent.operation_id.startswith(operation_prefix):
		raise RuntimeError(
			"Ferrum line-mode dispatch received an intent for a different active tool.",
		)
	operation = intent.operation_id.removeprefix(operation_prefix)
	if operation in ("key.enter", "key.return"):
		if intent.points:
			raise RuntimeError("Ferrum line-mode key intents must not include pointer points.")
		owner._complete_click_presentation_gesture(active)
		return
	if operation not in ("press", "move", "release"):
		raise RuntimeError("Ferrum line-mode dispatch received an unsupported operation.")
	if len(intent.points) != 1 or type(intent.points[0]) is not ferrum_qt.modes.base_mode.ScenePoint:
		raise RuntimeError("Ferrum line-mode pointer intents require exactly one ScenePoint.")
	event = _NormalizedLinePointerEvent(intent.points[0], active.tab, intent.modifiers)
	if operation == "press":
		if (
			active.tool in owner._completion_click_actions
			and (
				active.path_gesture is not None
				or active.presentation_gesture is not None
				or active.curved_equilibrium_arrow is not None
				or active.terminal_arrow is not None
			)
		):
			owner._append_click_presentation_point(event)
			return
		owner._start_line_gesture(event)
	elif operation == "move":
		owner._update_line_gesture(event)
	elif active.tool not in owner._draw_path_actions:
		owner._complete_line_gesture(event)

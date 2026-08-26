"""Feature-local normalized input controller for Rust structural selection."""

# Standard Library
import dataclasses

# local repo modules
import ferrum_qt.modes.base_mode


#============================================
@dataclasses.dataclass(slots=True)
class StructureSelectionMode(ferrum_qt.modes.base_mode.InteractionMode):
	"""Retain one drag start point while the shared adapter owns Qt input."""

	mode_id: ferrum_qt.modes.base_mode.ModeId = ferrum_qt.modes.base_mode.ModeId.EDIT
	_start_point: ferrum_qt.modes.base_mode.ScenePoint | None = None

	#============================================
	def enter(self, context: ferrum_qt.modes.base_mode.ModeContext) -> None:
		"""Clear a prior drag before structural selection becomes active."""
		del context
		self._start_point = None

	#============================================
	def exit(self, context: ferrum_qt.modes.base_mode.ModeContext) -> None:
		"""Discard only local gesture state when structural selection exits."""
		del context
		self._start_point = None

	#============================================
	def key_intent(self, key: str, context: ferrum_qt.modes.base_mode.ModeContext,
			) -> ferrum_qt.modes.base_mode.ModeIntent | None:
		"""Dispatch deletion through the feature endpoint and leave other keys alone."""
		del context
		if key not in ("Delete", "Backspace"):
			return None
		intent = ferrum_qt.modes.base_mode.ModeIntent("selection.delete", ())
		return intent

	#============================================
	def pointer_intent(self, pointer: ferrum_qt.modes.base_mode.PointerInput,
			context: ferrum_qt.modes.base_mode.ModeContext,
			) -> ferrum_qt.modes.base_mode.ModeIntent | None:
		"""Turn shared normalized input into feature-owned click and marquee intents."""
		del context
		if not pointer.primary_button:
			return None
		if pointer.phase is ferrum_qt.modes.base_mode.PointerPhase.PRESS:
			self._start_point = pointer.point
			intent = ferrum_qt.modes.base_mode.ModeIntent(
				"selection.press", (pointer.point,), pointer.modifiers,
			)
			return intent
		if pointer.phase is ferrum_qt.modes.base_mode.PointerPhase.MOVE:
			if self._start_point is None:
				return None
			intent = ferrum_qt.modes.base_mode.ModeIntent(
				"selection.move", (pointer.point,), pointer.modifiers,
			)
			return intent
		if pointer.phase is not ferrum_qt.modes.base_mode.PointerPhase.RELEASE:
			return None
		start_point = self._start_point
		self._start_point = None
		if start_point is None or start_point == pointer.point:
			intent = ferrum_qt.modes.base_mode.ModeIntent(
				"selection.release", (pointer.point,), pointer.modifiers,
			)
			return intent
		intent = ferrum_qt.modes.base_mode.ModeIntent(
			"selection.marquee", (start_point, pointer.point), pointer.modifiers,
		)
		return intent

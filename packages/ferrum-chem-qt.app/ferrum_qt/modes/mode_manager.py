"""Coordinate Ferrum interaction modes without owning document state."""

# Standard Library
import collections.abc

# local repo modules
import ferrum_qt.modes.base_mode
import ferrum_qt.modes.controllers


#============================================
ModeDispatcher = collections.abc.Callable[
	[
		ferrum_qt.modes.base_mode.ModeContext,
		ferrum_qt.modes.base_mode.ModeIntent,
	],
	None,
]


#============================================
class ModeManager:
	"""Own transient Qt tool state and delegate all durable work to a host.

	The injected dispatcher is the only mutation boundary.  In application code
	it should call an existing Ferrum tab/Rust operation callable selected by the
	operation ID; this manager never imports that tab or the extension module.
	"""

	def __init__(self, dispatcher: ModeDispatcher,
			modes: tuple[ferrum_qt.modes.base_mode.InteractionMode, ...] | None = None,
			) -> None:
		"""Build a manager with one controller per stable mode ID."""
		if not callable(dispatcher):
			raise TypeError("Ferrum mode dispatcher must be callable")
		if modes is None:
			modes = ferrum_qt.modes.controllers.default_modes()
		self._dispatcher = dispatcher
		self._modes = self._index_modes(modes)
		self._active_mode_id: ferrum_qt.modes.base_mode.ModeId | None = None

	@property
	def active_mode_id(self) -> ferrum_qt.modes.base_mode.ModeId | None:
		"""Return the active tool ID, or None when no tool owns interaction."""
		active_mode_id = self._active_mode_id
		return active_mode_id

	def activate(self, mode_id: ferrum_qt.modes.base_mode.ModeId,
			context: ferrum_qt.modes.base_mode.ModeContext,
			) -> None:
		"""Switch tools, retiring only the prior controller's transient state."""
		if type(mode_id) is not ferrum_qt.modes.base_mode.ModeId:
			raise TypeError("Ferrum mode ID must be an exact ModeId")
		if mode_id not in self._modes:
			raise ValueError(f"Ferrum mode has no input controller: {mode_id}")
		active_mode_id = self._active_mode_id
		if active_mode_id is mode_id:
			return
		if active_mode_id is not None:
			active_mode = self._modes.get(active_mode_id)
			if active_mode is not None:
				active_mode.exit(context)
		self._active_mode_id = mode_id
		self._modes[mode_id].enter(context)

	def synchronize_presentation(self, mode_id: ferrum_qt.modes.base_mode.ModeId,
			context: ferrum_qt.modes.base_mode.ModeContext,
			) -> None:
		"""Reflect a host-owned tool in shared chrome without claiming its input."""
		if type(mode_id) is not ferrum_qt.modes.base_mode.ModeId:
			raise TypeError("Ferrum mode ID must be an exact ModeId")
		active_mode_id = self._active_mode_id
		if active_mode_id is mode_id:
			return
		if active_mode_id is not None:
			active_mode = self._modes.get(active_mode_id)
			if active_mode is not None:
				active_mode.exit(context)
		self._active_mode_id = mode_id
		mode = self._modes.get(mode_id)
		if mode is not None:
			mode.enter(context)

	def cancel(self, context: ferrum_qt.modes.base_mode.ModeContext) -> bool:
		"""Leave the current tool without changing the Rust document."""
		active_mode_id = self._active_mode_id
		if active_mode_id is None:
			return False
		mode = self._modes.get(active_mode_id)
		if mode is not None:
			mode.exit(context)
		self._active_mode_id = None
		return True

	def handle_key(self, key: str,
			context: ferrum_qt.modes.base_mode.ModeContext,
			) -> bool:
		"""Handle escape locally or dispatch a controller-provided key intent."""
		if type(key) is not str:
			raise TypeError("Ferrum mode keys must be strings")
		if key == "Escape":
			return self.cancel(context)
		mode = self._active_mode()
		if mode is None:
			return False
		intent = mode.key_intent(key, context)
		return self._dispatch_intent(context, intent)

	def handle_pointer(self, pointer: ferrum_qt.modes.base_mode.PointerInput,
			context: ferrum_qt.modes.base_mode.ModeContext,
			) -> bool:
		"""Dispatch an operation after the active controller recognizes a gesture."""
		if type(pointer) is not ferrum_qt.modes.base_mode.PointerInput:
			raise TypeError("Ferrum mode pointer input must be exact PointerInput")
		mode = self._active_mode()
		if mode is None:
			return False
		intent = mode.pointer_intent(pointer, context)
		self._dispatch_intent(context, intent)
		return True

	def _active_mode(self) -> ferrum_qt.modes.base_mode.InteractionMode | None:
		"""Return the active controller without exposing the controller registry."""
		active_mode_id = self._active_mode_id
		if active_mode_id is None:
			return None
		mode = self._modes.get(active_mode_id)
		return mode

	def _dispatch_intent(self, context: ferrum_qt.modes.base_mode.ModeContext,
			intent: ferrum_qt.modes.base_mode.ModeIntent | None,
			) -> bool:
		"""Call the host boundary only for a completed semantic intent."""
		if intent is None:
			return False
		self._dispatcher(context, intent)
		return True

	@staticmethod
	def _index_modes(
			modes: tuple[ferrum_qt.modes.base_mode.InteractionMode, ...],
			) -> dict[
				ferrum_qt.modes.base_mode.ModeId,
				ferrum_qt.modes.base_mode.InteractionMode,
			]:
		"""Reject duplicate or non-mode controllers at the isolated boundary."""
		if type(modes) is not tuple:
			raise TypeError("Ferrum modes must be supplied as a tuple")
		indexed: dict[
			ferrum_qt.modes.base_mode.ModeId,
			ferrum_qt.modes.base_mode.InteractionMode,
		] = {}
		for mode in modes:
			mode_id = getattr(mode, "mode_id", None)
			if type(mode_id) is not ferrum_qt.modes.base_mode.ModeId:
				raise TypeError("Ferrum mode controllers require an exact ModeId")
			if mode_id in indexed:
				raise ValueError(f"Duplicate Ferrum mode controller: {mode_id}")
			indexed[mode_id] = mode
		if not indexed:
			raise ValueError("Ferrum mode manager requires at least one controller")
		return indexed

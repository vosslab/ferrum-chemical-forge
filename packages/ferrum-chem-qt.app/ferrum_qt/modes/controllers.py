"""Bounded Ferrum tool-intent controllers with no document ownership."""

# Standard Library
import dataclasses

# local repo modules
import ferrum_qt.modes.base_mode


#============================================
class _SingleClickMode(ferrum_qt.modes.base_mode.InteractionMode):
	"""Dispatch one supported operation when the primary pointer is released."""

	def __init__(self, mode_id: ferrum_qt.modes.base_mode.ModeId,
			operation_id: str) -> None:
		"""Create a stateless controller for an existing operation seam."""
		self.mode_id = mode_id
		self._operation_id = operation_id

	def enter(self, context: ferrum_qt.modes.base_mode.ModeContext) -> None:
		"""Enter without retaining host or document state."""

	def exit(self, context: ferrum_qt.modes.base_mode.ModeContext) -> None:
		"""Exit without retaining host or document state."""

	def key_intent(self, key: str,
			context: ferrum_qt.modes.base_mode.ModeContext,
			) -> ferrum_qt.modes.base_mode.ModeIntent | None:
		"""Leave keys to the host except a semantic activation key."""
		if key != "Enter":
			return None
		intent = ferrum_qt.modes.base_mode.ModeIntent(self._operation_id, ())
		return intent

	def pointer_intent(self, pointer: ferrum_qt.modes.base_mode.PointerInput,
			context: ferrum_qt.modes.base_mode.ModeContext,
			) -> ferrum_qt.modes.base_mode.ModeIntent | None:
		"""Return one click intent; coordinate mapping belongs to the Qt adapter."""
		if not pointer.primary_button or pointer.phase is not ferrum_qt.modes.base_mode.PointerPhase.RELEASE:
			return None
		intent = ferrum_qt.modes.base_mode.ModeIntent(
			self._operation_id, (pointer.point,), pointer.modifiers,
		)
		return intent


#============================================
@dataclasses.dataclass(slots=True)
class _DragMode(ferrum_qt.modes.base_mode.InteractionMode):
	"""Own only an in-progress start point for a two-point Rust operation."""

	mode_id: ferrum_qt.modes.base_mode.ModeId
	operation_id: str
	_start_point: ferrum_qt.modes.base_mode.ScenePoint | None = None

	def enter(self, context: ferrum_qt.modes.base_mode.ModeContext) -> None:
		"""Clear any stale local pointer capture before a new interaction."""
		self._start_point = None

	def exit(self, context: ferrum_qt.modes.base_mode.ModeContext) -> None:
		"""Discard local pointer capture without mutating Rust state."""
		self._start_point = None

	def key_intent(self, key: str,
			context: ferrum_qt.modes.base_mode.ModeContext,
			) -> ferrum_qt.modes.base_mode.ModeIntent | None:
		"""Drag tools have no direct key operation."""
		return None

	def pointer_intent(self, pointer: ferrum_qt.modes.base_mode.PointerInput,
			context: ferrum_qt.modes.base_mode.ModeContext,
			) -> ferrum_qt.modes.base_mode.ModeIntent | None:
		"""Capture press/release geometry and dispatch only a completed gesture."""
		if not pointer.primary_button:
			return None
		if pointer.phase is ferrum_qt.modes.base_mode.PointerPhase.PRESS:
			self._start_point = pointer.point
			return None
		if pointer.phase is not ferrum_qt.modes.base_mode.PointerPhase.RELEASE:
			return None
		start_point = self._start_point
		self._start_point = None
		if start_point is None:
			return None
		intent = ferrum_qt.modes.base_mode.ModeIntent(
			self.operation_id, (start_point, pointer.point), pointer.modifiers,
		)
		return intent


#============================================
class AtomMode(_SingleClickMode):
	"""Controller seam for the existing Rust-backed atom-placement operation."""

	def __init__(self) -> None:
		"""Configure the atom placement semantic operation."""
		super().__init__(
			ferrum_qt.modes.base_mode.ModeId.ATOM, "atom.place",
		)


#============================================
class DrawMode(_DragMode):
	"""Controller seam for the existing Rust-backed bond-drawing operation."""

	def __init__(self) -> None:
		"""Configure the two-point bond operation."""
		super().__init__(
			ferrum_qt.modes.base_mode.ModeId.DRAW, "bond.draw",
		)


#============================================
class EditMode(_SingleClickMode):
	"""Controller seam for existing Rust-observation selection editing."""

	def __init__(self) -> None:
		"""Configure the generic edit-selection operation."""
		super().__init__(
			ferrum_qt.modes.base_mode.ModeId.EDIT, "selection.edit",
		)


#============================================
class ArrowMode(_SingleClickMode):
	"""Controller seam for existing Rust-backed selected-arrow properties."""

	def __init__(self) -> None:
		"""Configure the selected-arrow editing operation."""
		super().__init__(
			ferrum_qt.modes.base_mode.ModeId.ARROW, "arrow.edit_selected",
		)


#============================================
class VectorMode(_SingleClickMode):
	"""Controller seam for existing Rust-backed selected-vector properties."""

	def __init__(self) -> None:
		"""Configure the selected-vector editing operation."""
		super().__init__(
			ferrum_qt.modes.base_mode.ModeId.VECTOR, "vector.edit_selected",
		)


#============================================
class BracketMode(_DragMode):
	"""Controller seam for existing Rust-backed bracket-pair creation."""

	def __init__(self) -> None:
		"""Configure the two-point bracket creation operation."""
		super().__init__(
			ferrum_qt.modes.base_mode.ModeId.BRACKET, "bracket.create",
		)


#============================================
def default_modes() -> tuple[ferrum_qt.modes.base_mode.InteractionMode, ...]:
	"""Return the bounded controller set backed by current Ferrum operations."""
	controllers = (
		AtomMode(), DrawMode(), EditMode(), ArrowMode(), VectorMode(), BracketMode(),
	)
	return controllers

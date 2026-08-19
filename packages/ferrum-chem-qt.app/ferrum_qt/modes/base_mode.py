"""Small, document-free contracts for Ferrum canvas interaction modes."""

# Standard Library
import dataclasses
import enum


#============================================
class ModeId(enum.StrEnum):
	"""Stable Ferrum tool IDs independent of Qt action text."""

	ATOM = "atom"
	DRAW = "draw"
	EDIT = "edit"
	ARROW = "arrow"
	VECTOR = "vector"
	BRACKET = "bracket"


#============================================
class PointerPhase(enum.StrEnum):
	"""Pointer phases normalized by the Qt event adapter."""

	PRESS = "press"
	MOVE = "move"
	RELEASE = "release"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class ScenePoint:
	"""One already-normalized scene point with no Qt object ownership."""

	x: float
	y: float


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class PointerInput:
	"""One primary-pointer event supplied by a thin Qt boundary adapter."""

	phase: PointerPhase
	point: ScenePoint
	primary_button: bool = True


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class ModeContext:
	"""Immutable Rust observation plus an opaque host dispatch context.

	The manager only passes these values through to the injected dispatcher.  It
	does not inspect, cache, or mutate a document/session/tab object.
	"""

	observation: object
	dispatch_context: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class ModeIntent:
	"""One semantic request for an existing Ferrum tab/Rust operation seam."""

	operation_id: str
	points: tuple[ScenePoint, ...]


#============================================
class InteractionMode:
	"""Lifecycle hooks for one local canvas-tool state machine."""

	mode_id: ModeId

	def enter(self, context: ModeContext) -> None:
		"""Start a mode without mutating the chemistry document."""
		raise NotImplementedError

	def exit(self, context: ModeContext) -> None:
		"""Discard transient interaction state without document mutation."""
		raise NotImplementedError

	def key_intent(self, key: str, context: ModeContext) -> ModeIntent | None:
		"""Return one semantic operation for a normalized key, if any."""
		raise NotImplementedError

	def pointer_intent(
			self, pointer: PointerInput, context: ModeContext,
			) -> ModeIntent | None:
		"""Return one semantic operation for a normalized pointer event."""
		raise NotImplementedError

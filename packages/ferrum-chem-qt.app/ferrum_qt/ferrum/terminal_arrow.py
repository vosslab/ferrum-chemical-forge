"""Closed Qt boundary adapter for Rust-owned curved terminal arrows."""

# Standard Library
import dataclasses
import enum


#============================================
class TerminalArrowKind(enum.Enum):
	"""The supported Rust-owned curved terminal-arrow families."""

	ELECTRON = "electron"
	RETRO = "retro"
	NORMAL_REACTION = "curved-normal"

	@property
	def action_name(self) -> str:
		"""Return the distinct public QAction name for this closed kind."""
		if self is TerminalArrowKind.ELECTRON:
			return "Draw Curved Electron Arrow"
		if self is TerminalArrowKind.RETRO:
			return "Draw Curved Retro Arrow"
		return "Draw Curved Reaction Arrow"

	@property
	def description(self) -> str:
		"""Return the user-facing noun used in status and refusal text."""
		if self is TerminalArrowKind.ELECTRON:
			return "curved electron arrow"
		if self is TerminalArrowKind.RETRO:
			return "curved retro arrow"
		return "curved reaction arrow"

	@classmethod
	def from_line_tool_value(cls, value: str) -> "TerminalArrowKind":
		"""Map the public terminal-arrow tools into this closed Rust lifecycle."""
		if value == "draw_curved_electron_arrow":
			return cls.ELECTRON
		if value == "draw_curved_retro_arrow":
			return cls.RETRO
		if value == "draw_curved_reaction_arrow":
			return cls.NORMAL_REACTION
		raise ValueError(f"Ferrum terminal-arrow lifecycle does not support {value!r}")


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class TerminalArrowState:
	"""Qt-local point capture for one opaque Rust terminal-arrow operation."""

	kind: TerminalArrowKind
	points: tuple[tuple[float, float], ...] = ()

	def append(self, point: tuple[float, float]) -> "TerminalArrowState":
		"""Capture at most three authored points; Rust owns geometric refusal."""
		if len(self.points) >= 3:
			return self
		return dataclasses.replace(self, points=self.points + (point,))


#============================================
class TerminalArrowOperation:
	"""Dispatch one closed terminal-arrow kind to its explicit Rust session API."""

	@staticmethod
	def begin(session: object, revision: int, digest: str, kind: TerminalArrowKind,
			start: tuple[float, float], control: tuple[float, float]) -> object:
		"""Begin the native operation after Qt has captured start and control points."""
		if kind is TerminalArrowKind.ELECTRON:
			return session.begin_curved_electron_arrow_gesture_v1(
				revision, digest, start[0], start[1], control[0], control[1],
			)
		if kind is TerminalArrowKind.RETRO:
			return session.begin_curved_retro_arrow_gesture_v1(
				revision, digest, start[0], start[1], control[0], control[1],
			)
		return session.begin_curved_normal_reaction_arrow_gesture_v1(
			revision, digest, start[0], start[1], control[0], control[1],
		)

	@staticmethod
	def preview(session: object, kind: TerminalArrowKind, gesture: object,
			end: tuple[float, float]) -> object:
		"""Ask Rust for the complete immutable preview projection."""
		if kind is TerminalArrowKind.ELECTRON:
			return session.preview_curved_electron_arrow_gesture_v1(gesture, end[0], end[1])
		if kind is TerminalArrowKind.RETRO:
			return session.preview_curved_retro_arrow_gesture_v1(gesture, end[0], end[1])
		return session.preview_curved_normal_reaction_arrow_gesture_v1(gesture, end[0], end[1])

	@staticmethod
	def prepare(session: object, kind: TerminalArrowKind, gesture: object, preview: object) -> object:
		"""Renderer-preflight one opaque Rust candidate."""
		if kind is TerminalArrowKind.ELECTRON:
			return session.prepare_curved_electron_arrow_gesture_v1(gesture, preview)
		if kind is TerminalArrowKind.RETRO:
			return session.prepare_curved_retro_arrow_gesture_v1(gesture, preview)
		return session.prepare_curved_normal_reaction_arrow_gesture_v1(gesture, preview)

	@staticmethod
	def commit(session: object, kind: TerminalArrowKind, prepared: object) -> object:
		"""Redeem one opaque Rust receipt exactly once."""
		if kind is TerminalArrowKind.ELECTRON:
			return session.commit_curved_electron_arrow_gesture_v1(prepared)
		if kind is TerminalArrowKind.RETRO:
			return session.commit_curved_retro_arrow_gesture_v1(prepared)
		return session.commit_curved_normal_reaction_arrow_gesture_v1(prepared)


#============================================
def is_native_error(kind: TerminalArrowKind, error: Exception) -> bool:
	"""Accept only the exact native error class for one closed arrow kind."""
	import ferrum_qt.ferrum.engine as engine
	if kind is TerminalArrowKind.ELECTRON:
		return type(error) is engine.CurvedElectronArrowGestureError
	if kind is TerminalArrowKind.RETRO:
		return type(error) is engine.CurvedRetroArrowGestureError
	return type(error) is engine.CurvedNormalReactionArrowGestureError


#============================================
def needs_endpoint(state: TerminalArrowState, error: Exception) -> bool:
	"""Retain two-point authoring only for Rust's typed geometry correction route."""
	if len(state.points) != 2:
		return False
	import ferrum_qt.ferrum.engine as engine
	if not is_native_error(state.kind, error):
		return False
	if state.kind is TerminalArrowKind.ELECTRON:
		category = engine.CurvedElectronArrowGestureCategoryV1
		recovery = engine.CurvedElectronArrowGestureRecoveryV1
	elif state.kind is TerminalArrowKind.RETRO:
		category = engine.CurvedRetroArrowGestureCategoryV1
		recovery = engine.CurvedRetroArrowGestureRecoveryV1
	else:
		category = engine.CurvedNormalReactionArrowGestureCategoryV1
		recovery = engine.CurvedNormalReactionArrowGestureRecoveryV1
	return error.category in (
		category.collapsed_span, category.control_too_near_chord,
	) and error.recovery == recovery.change_geometry


#============================================
def refusal_message(kind: TerminalArrowKind, error: Exception) -> str:
	"""Map one exact native refusal to its closed actionable recovery wording."""
	import ferrum_qt.ferrum.engine as engine
	description = kind.description.capitalize()
	if is_native_error(kind, error):
		if kind is TerminalArrowKind.ELECTRON:
			category = engine.CurvedElectronArrowGestureCategoryV1
		elif kind is TerminalArrowKind.RETRO:
			category = engine.CurvedRetroArrowGestureCategoryV1
		else:
			category = engine.CurvedNormalReactionArrowGestureCategoryV1
		if error.category in (
				category.collapsed_span, category.control_too_near_chord,
				category.exceeds_geometry_limit,
			):
			return f"{description} is unchanged. Choose a clearly curved, finite three-point gesture."
		if error.category in (category.stale_snapshot, category.session_conflict):
			return f"{description} is unchanged. Refresh the Rust view and start the tool again."
		return f"{description} is unchanged. Adjust the three points and try again."
	return f"{description} is unchanged. Restart the tool and try again."

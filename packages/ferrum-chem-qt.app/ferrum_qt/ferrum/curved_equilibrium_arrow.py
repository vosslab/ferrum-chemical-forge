"""Closed Qt boundary adapter for Rust-owned curved equilibrium arrows."""

# Standard Library
import dataclasses


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class CurvedEquilibriumArrowState:
	"""Qt-local point capture for one opaque Rust equilibrium-arrow operation."""

	points: tuple[tuple[float, float], ...] = ()

	#============================================
	def append(self, point: tuple[float, float]) -> "CurvedEquilibriumArrowState":
		"""Capture at most three authored points; Rust owns geometric refusal."""
		if len(self.points) >= 3:
			return self
		return dataclasses.replace(self, points=self.points + (point,))


#============================================
class CurvedEquilibriumArrowOperation:
	"""Dispatch the dedicated equilibrium lifecycle to explicit Rust APIs."""

	#============================================
	@staticmethod
	def begin(session: object, revision: int, digest: str, start: tuple[float, float],
			control: tuple[float, float]) -> object:
		"""Begin one native operation after Qt captures start and control."""
		return session.begin_curved_equilibrium_arrow_gesture_v1(
			revision, digest, start[0], start[1], control[0], control[1],
		)

	#============================================
	@staticmethod
	def preview(session: object, gesture: object, end: tuple[float, float]) -> object:
		"""Ask Rust for its complete immutable two-lane preview."""
		return session.preview_curved_equilibrium_arrow_gesture_v1(gesture, end[0], end[1])

	#============================================
	@staticmethod
	def prepare(session: object, gesture: object, preview: object) -> object:
		"""Renderer-preflight one opaque native candidate."""
		return session.prepare_curved_equilibrium_arrow_gesture_v1(gesture, preview)

	#============================================
	@staticmethod
	def commit(session: object, prepared: object) -> object:
		"""Redeem one opaque native receipt exactly once."""
		return session.commit_curved_equilibrium_arrow_gesture_v1(prepared)


#============================================
def is_native_error(error: Exception) -> bool:
	"""Accept only the exact native curved-equilibrium refusal class."""
	import ferrum_qt.ferrum.engine as engine
	return type(error) is engine.CurvedEquilibriumArrowGestureError


#============================================
def refusal_message(error: Exception) -> str:
	"""Map one native refusal into closed actionable non-modal wording."""
	import ferrum_qt.ferrum.engine as engine
	if not is_native_error(error):
		return "Curved equilibrium arrow is unchanged. Restart the tool and try again."
	category = engine.CurvedEquilibriumArrowGestureCategoryV1
	if error.category in (category.collapsed_span, category.control_too_near_chord,
			category.exceeds_geometry_limit):
		return "Curved equilibrium arrow is unchanged. Choose a clearly curved, finite three-point gesture."
	recovery = engine.CurvedEquilibriumArrowGestureRecoveryV1
	if error.recovery is recovery.refresh_and_restart:
		return "Curved equilibrium arrow is unchanged. Refresh the document and start the tool again."
	if error.recovery is recovery.document_unchanged:
		return "Curved equilibrium arrow is unchanged. Restart the tool and try again."
	raise RuntimeError("Ferrum returned an unknown curved-equilibrium refusal category")

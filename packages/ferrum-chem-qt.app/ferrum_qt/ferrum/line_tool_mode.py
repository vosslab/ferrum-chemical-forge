"""Normalized input controller for one active Ferrum line authoring tool."""

# local repo modules
import ferrum_qt.modes.base_mode


#============================================
class LineToolMode(ferrum_qt.modes.base_mode.InteractionMode):
	"""Emit every normalized input phase to the active line-tool feature seam."""

	def __init__(self, mode_id: ferrum_qt.modes.base_mode.ModeId, tool: object) -> None:
		"""Keep only the stable presentation tool identity between inputs."""
		self.mode_id = mode_id
		tool_value = getattr(tool, "value", None)
		if type(tool_value) is not str or not tool_value:
			raise TypeError("Ferrum line-tool modes require a stable tool value.")
		self._operation_prefix = f"line.{tool_value}"

	def enter(self, context: ferrum_qt.modes.base_mode.ModeContext) -> None:
		"""Enter without retaining a document or Qt object."""

	def exit(self, context: ferrum_qt.modes.base_mode.ModeContext) -> None:
		"""Leave cleanup and document ownership to the line feature endpoint."""

	def key_intent(self, key: str, context: ferrum_qt.modes.base_mode.ModeContext,
			) -> ferrum_qt.modes.base_mode.ModeIntent | None:
		"""Forward meaningful line-tool keys through the normalized controller."""
		if key not in ("Enter", "Return"):
			return None
		return ferrum_qt.modes.base_mode.ModeIntent(
			f"{self._operation_prefix}.key.{key.lower()}", (),
		)

	def pointer_intent(self, pointer: ferrum_qt.modes.base_mode.PointerInput,
			context: ferrum_qt.modes.base_mode.ModeContext,
			) -> ferrum_qt.modes.base_mode.ModeIntent | None:
		"""Forward each primary-pointer phase without retaining geometry locally."""
		if not pointer.primary_button:
			return None
		return ferrum_qt.modes.base_mode.ModeIntent(
			f"{self._operation_prefix}.{pointer.phase.value}", (pointer.point,), pointer.modifiers,
		)
